//! A table holding one value for each member of a key enum: each area, or each supply.

use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

/// An enum of `N` members that keys a table.
pub trait Key<const N: usize>: Copy {
    /// The one of `row` that belongs to this key; `row` holds one for each key, in the key's
    /// `ALL` order.
    fn pick<T>(self, row: [T; N]) -> T;
}

/// One value for each key, read and written as `table[key]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Table<K, V, const N: usize> {
    /// The values, in the key's `ALL` order.
    values: [V; N],
    keys: PhantomData<K>,
}

impl<K, V, const N: usize> Table<K, V, N> {
    /// The table of `values`, given in the key's `ALL` order.
    #[must_use]
    pub const fn new(values: [V; N]) -> Self {
        Self {
            values,
            keys: PhantomData,
        }
    }
}

impl<K, V: Default, const N: usize> Default for Table<K, V, N> {
    fn default() -> Self {
        Self::new(core::array::from_fn(|_| V::default()))
    }
}

impl<K: Key<N>, V, const N: usize> Index<K> for Table<K, V, N> {
    type Output = V;

    fn index(&self, key: K) -> &V {
        key.pick(self.values.each_ref())
    }
}

impl<K: Key<N>, V, const N: usize> IndexMut<K> for Table<K, V, N> {
    fn index_mut(&mut self, key: K) -> &mut V {
        key.pick(self.values.each_mut())
    }
}
