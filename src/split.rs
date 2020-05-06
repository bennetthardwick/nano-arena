use super::{ArenaAccess, Idx};
use std::borrow::Borrow;

pub struct ArenaSplit<'a, T, A: ArenaAccess<T>> {
    pub(crate) selected: Idx,
    pub(crate) arena: &'a mut A,
    pub(crate) __type: std::marker::PhantomData<T>,
}

impl<T, A: ArenaAccess<T>> ArenaSplit<'_, T, A> {
    pub fn split_at<'a, I: Borrow<Idx>>(
        &'a mut self,
        selected: I,
    ) -> Option<(&mut T, ArenaSplit<'a, T, Self>)> {
        if selected.borrow() == &self.selected {
            None
        } else {
            if let Some(value) = self.arena.get_mut(selected.borrow()) {
                Some((
                    unsafe { (value as *mut T).as_mut().unwrap() },
                    ArenaSplit {
                        selected: selected.borrow().clone(),
                        arena: self,
                        __type: Default::default(),
                    },
                ))
            } else {
                None
            }
        }
    }
}

impl<T, A: ArenaAccess<T>> ArenaAccess<T> for ArenaSplit<'_, T, A> {
    fn get<I: Borrow<Idx>>(&self, index: I) -> Option<&T> {
        if index.borrow() == &self.selected {
            None
        } else {
            self.arena.get(index)
        }
    }

    fn get_mut<I: Borrow<Idx>>(&mut self, index: I) -> Option<&mut T> {
        if index.borrow() == &self.selected {
            None
        } else {
            self.arena.get_mut(index)
        }
    }
}
