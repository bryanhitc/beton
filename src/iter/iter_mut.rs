use std::mem::MaybeUninit;

use crate::indexer::Occupied;
use crate::{Key, Slab};

/// A mutable iterator over items in the `Slab`.
#[derive(Debug)]
pub struct IterMut<'a, T> {
    occupied: Occupied<'a>,
    entries: core::slice::IterMut<'a, MaybeUninit<T>>,
    /// What index did we last index? We need this to advance the slice
    /// iterator.
    prev_index: Option<usize>,
}

impl<'a, T> IterMut<'a, T> {
    pub(crate) fn new(slab: &'a mut Slab<T>) -> Self {
        let occupied = slab.index.occupied();
        let entries = slab.entries.iter_mut();
        Self {
            occupied,
            entries,
            prev_index: None,
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (Key, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        // Get the next index and update all cursors
        let index = self.occupied.next()?;
        let relative_index = match self.prev_index.replace(index) {
            None => 0,
            Some(prev_index) => index - prev_index - 1,
        };

        // SAFETY: we just confirmed that there was in fact an entry at this index
        self.entries
            .nth(relative_index)
            .map(|t| (index.into(), unsafe { t.assume_init_mut() }))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iter_mut() {
        let mut slab = crate::Slab::new();
        slab.insert(1);
        let key = slab.insert(2);
        slab.insert(3);
        slab.remove(key);
        let mut iter = IterMut::new(&mut slab);
        assert_eq!(iter.next(), Some((0.into(), &mut 1)));
        assert_eq!(iter.next(), Some((2.into(), &mut 3)));
        assert_eq!(iter.next(), None);
    }
}
