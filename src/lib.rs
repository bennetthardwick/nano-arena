use std::borrow::Borrow;
use std::iter::FromIterator;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

struct IdxInner {
    index: AtomicUsize,
    removed: AtomicBool,
}

impl IdxInner {
    fn index(&self) -> Option<usize> {
        let removed = self.removed.load(Ordering::Relaxed);
        if !removed {
            Some(self.index.load(Ordering::Relaxed))
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Idx {
    inner: Arc<IdxInner>,
}

impl Idx {
    fn index(&self) -> Option<usize> {
        self.inner.index()
    }
}

pub struct Arena<T> {
    values: Vec<(Arc<IdxInner>, T)>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self { values: vec![] }
    }
}

fn choose_second_member_of_tuple_mut<A, B>((_, value): &mut (A, B)) -> &mut B {
    value
}

fn choose_second_member_of_tuple_ref<A, B>((_, value): &(A, B)) -> &B {
    value
}

pub struct IterMut<'a, T> {
    iterator: std::iter::Map<
        std::slice::IterMut<'a, (std::sync::Arc<IdxInner>, T)>,
        &'a dyn Fn(&mut (Arc<IdxInner>, T)) -> &mut T,
    >,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

pub struct Iter<'a, T> {
    iterator: std::iter::Map<
        std::slice::Iter<'a, (std::sync::Arc<IdxInner>, T)>,
        &'a dyn Fn(&(Arc<IdxInner>, T)) -> &T,
    >,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

impl<T> FromIterator<T> for Arena<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Arena {
            values: iter
                .into_iter()
                .enumerate()
                .map(|(index, value)| (create_idx(index), value))
                .collect(),
        }
    }
}
fn create_idx(index: usize) -> Arc<IdxInner> {
    Arc::new(IdxInner {
        index: AtomicUsize::new(index),
        removed: AtomicBool::new(false),
    })
}

impl<T> Arena<T> {
    pub fn new() -> Arena<T> {
        Self { values: vec![] }
    }

    pub fn with_capacity(capacity: usize) -> Arena<T> {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }

    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    pub fn alloc_with_idx<F: FnOnce(Idx) -> T>(&mut self, func: F) -> Idx {
        let len = self.values.len();
        let inner = create_idx(len);
        let idx = Idx {
            inner: inner.clone(),
        };
        self.values.push((inner.clone(), func(idx)));
        Idx { inner }
    }

    pub fn alloc(&mut self, value: T) -> Idx {
        let len = self.values.len();
        let inner = create_idx(len);
        self.values.push((inner.clone(), value));
        Idx { inner }
    }

    pub fn get<I: Borrow<Idx>>(&self, index: I) -> Option<&T> {
        index
            .borrow()
            .index()
            .and_then(|index| self.values.get(index).and_then(|(_, value)| Some(value)))
    }

    pub fn get_mut<'a, I: Borrow<Idx>>(&'a mut self, index: I) -> Option<&'a mut T> {
        if let Some(index) = index.borrow().index() {
            self.values
                .get_mut(index)
                .and_then(|(_, value)| Some(value))
        } else {
            None
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, T> {
        IterMut {
            iterator: self
                .values
                .iter_mut()
                .map(&choose_second_member_of_tuple_mut),
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter {
            iterator: self.values.iter().map(&choose_second_member_of_tuple_ref),
        }
    }

    pub fn to_vec(self) -> Vec<T> {
        self.into()
    }

    pub fn cleanup(&mut self) {
        self.values.retain(|(idx, _)| idx.index().is_some())
    }

    pub fn remove(&mut self, index: &Idx) -> T {
        if let Some(index) = index.borrow().index() {
            let (removed_index, value) = self.values.remove(index);

            for (index, (idx, _)) in self.values.iter().enumerate().skip(index) {
                idx.index.store(index, Ordering::Relaxed);
            }

            removed_index.removed.store(true, Ordering::Relaxed);

            value
        } else {
            panic!("Trying to remove index that has already been removed!");
        }
    }

    pub fn swap<A: Borrow<Idx>, B: Borrow<Idx>>(&mut self, a: A, b: B) {
        if let Some((a_index, b_index)) = a
            .borrow()
            .index()
            .and_then(|a| b.borrow().index().map(|b| (a, b)))
        {
            self.values.swap(a_index, b_index);
            self.values[a_index]
                .0
                .index
                .store(b_index, Ordering::Relaxed);
            self.values[b_index]
                .0
                .index
                .store(a_index, Ordering::Relaxed);
        }
    }

    pub fn apply_ordering<I>(&mut self, ordering: &Vec<Idx>) {
        assert!(ordering.len() == self.values.len());

        let mut old_arena = Arena::<T>::with_capacity(self.capacity());
        std::mem::swap(&mut old_arena.values, &mut self.values);

        for idx in ordering.iter() {
            self.values
                .push((create_idx(self.values.len()), old_arena.swap_remove(idx)))
        }
    }

    pub fn swap_remove<I: Borrow<Idx>>(&mut self, index: I) -> T {
        if let Some(index) = index.borrow().index() {
            let (removed_index, value) = self.values.swap_remove(index);

            if self.values.len() > 0 {
                self.values[index].0.index.store(index, Ordering::Relaxed);
            }

            removed_index.removed.store(true, Ordering::Relaxed);

            value
        } else {
            panic!("Trying to remove index that has already been removed!");
        }
    }
}

impl<T> Into<Vec<T>> for Arena<T> {
    fn into(self) -> Vec<T> {
        // Set all the indexes to removed, since we can't use them anymore
        for (idx, _) in self.values.iter() {
            idx.removed.store(true, Ordering::Relaxed);
        }

        // Grab all the values and turn them into an array
        self.values.into_iter().map(|(_, value)| value).collect()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
