use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

struct IdxInner {
    index: AtomicUsize,
    removed: AtomicBool,
}

#[derive(Clone)]
pub struct Idx {
    inner: Arc<IdxInner>,
}

pub struct Arena<T> {
    values: Vec<(Arc<IdxInner>, T)>,
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

impl<T> Arena<T> {
    pub fn alloc(&mut self, value: T) -> Idx {
        let len = self.values.len();

        let inner = Arc::new(IdxInner {
            index: AtomicUsize::new(len),
            removed: AtomicBool::new(false),
        });

        self.values.push((inner.clone(), value));

        Idx { inner }
    }

    pub fn swap_remove(&mut self, index: &Idx) -> T {
        let atomic_index = &index.inner;

        if atomic_index.removed.load(Ordering::Relaxed) {
            panic!("Trying to remove index that has already been removed!");
        }

        let index = atomic_index.index.load(Ordering::Relaxed);

        let (removed_index, value) = self.values.swap_remove(index);

        if self.values.len() > 0 {
            self.values[index].0.index.store(index, Ordering::Relaxed);
        }

        removed_index.removed.store(true, Ordering::Relaxed);

        value
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

    pub fn remove(&mut self, index: &Idx) -> T {
        let atomic_index = &index.inner;

        if atomic_index.removed.load(Ordering::Relaxed) {
            panic!("Trying to remove index that has already been removed!");
        }

        let index = atomic_index.index.load(Ordering::Relaxed);

        let (removed_index, value) = self.values.remove(index);

        for (index, (idx, _)) in self.values.iter().enumerate().skip(index) {
            idx.index.store(index, Ordering::Relaxed);
        }

        removed_index.removed.store(true, Ordering::Relaxed);

        value
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
