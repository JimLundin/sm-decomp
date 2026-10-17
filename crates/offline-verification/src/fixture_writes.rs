//! Fixture writes borrow validated records and payloads from one opened binary.
//! Generators materialize an editable vector only when they mutate those writes.
use super::Write;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, sync::Arc};

#[derive(Clone)]
enum Storage {
    Editable(Vec<Write>),
    Packed { bytes: Arc<Vec<u8>>, records: usize, count: usize, payload: usize },
}

#[derive(Clone)]
pub struct Writes(Storage);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct WriteRef<'a> {
    pub offset: usize,
    pub data: &'a [u8],
}

impl Default for Writes {
    fn default() -> Self {
        Self(Storage::Editable(Vec::new()))
    }
}

impl Writes {
    /// Only the binary reader constructs this, after validating every extent.
    pub(crate) fn packed(bytes: Arc<Vec<u8>>, records: usize, count: usize, payload: usize) -> Self {
        Self(Storage::Packed { bytes, records, count, payload })
    }

    pub fn len(&self) -> usize {
        match &self.0 {
            Storage::Editable(writes) => writes.len(),
            Storage::Packed { count, .. } => *count,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter { writes: self, next: 0 }
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Write> {
        self.editable().iter_mut()
    }

    fn editable(&mut self) -> &mut Vec<Write> {
        if matches!(self.0, Storage::Packed { .. }) {
            self.0 =
                Storage::Editable(self.iter().map(|w| Write { offset: w.offset, data: w.data.to_vec() }).collect());
        }
        let Storage::Editable(writes) = &mut self.0 else { unreachable!() };
        writes
    }

    pub fn push(&mut self, write: Write) {
        self.editable().push(write);
    }

    pub fn clear(&mut self) {
        self.0 = Storage::Editable(Vec::new());
    }
}

pub struct Iter<'a> {
    writes: &'a Writes,
    next: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = WriteRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.writes.len() {
            return None;
        }
        let index = self.next;
        self.next += 1;
        Some(match &self.writes.0 {
            Storage::Editable(writes) => WriteRef { offset: writes[index].offset, data: &writes[index].data },
            Storage::Packed { bytes, records, payload, .. } => {
                let at = records + index * 16;
                let offset = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
                let len = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()) as usize;
                let blob = u64::from_le_bytes(bytes[at + 8..at + 16].try_into().unwrap()) as usize;
                WriteRef { offset, data: &bytes[payload + blob..payload + blob + len] }
            }
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = self.writes.len() - self.next;
        (left, Some(left))
    }
}
impl ExactSizeIterator for Iter<'_> {}

impl<'a> IntoIterator for &'a Writes {
    type Item = WriteRef<'a>;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl IntoIterator for Writes {
    type Item = Write;
    type IntoIter = std::vec::IntoIter<Write>;
    fn into_iter(mut self) -> Self::IntoIter {
        std::mem::take(self.editable()).into_iter()
    }
}
impl From<Vec<Write>> for Writes {
    fn from(writes: Vec<Write>) -> Self {
        Self(Storage::Editable(writes))
    }
}
impl FromIterator<Write> for Writes {
    fn from_iter<T: IntoIterator<Item = Write>>(iter: T) -> Self {
        Self::from(iter.into_iter().collect::<Vec<_>>())
    }
}
impl Extend<Write> for Writes {
    fn extend<T: IntoIterator<Item = Write>>(&mut self, iter: T) {
        self.editable().extend(iter);
    }
}
impl PartialEq for Writes {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}
impl fmt::Debug for Writes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
impl Serialize for Writes {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter())
    }
}
impl<'de> Deserialize<'de> for Writes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<Write>::deserialize(deserializer).map(Self::from)
    }
}
