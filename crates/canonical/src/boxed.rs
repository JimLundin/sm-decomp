//! Heap ownership carries no persisted meaning beyond the complete value it owns.

use crate::{Canonical, DecodeError, Difference, Reader};

impl<T: Canonical> Canonical for Box<T> {
    const SCHEMA: u64 = T::SCHEMA;

    fn encode(&self, out: &mut Vec<u8>) {
        self.as_ref().encode(out);
    }

    fn decode(input: &mut Reader<'_>) -> Result<Self, DecodeError> {
        T::decode(input).map(Self::new)
    }

    fn diff(&self, other: &Self) -> Option<Difference> {
        self.as_ref().diff(other.as_ref())
    }
}
