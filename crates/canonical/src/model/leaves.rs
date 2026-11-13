//! A State's leaves, for [`super::Model`]'s diff: each primitive's canonical bytes with its
//! path, in encoding order.
//!
//! An enum's tag and a sequence's length are leaves of their own, placed before what they
//! hold, so two values of one type share their leaves' paths up to their first difference, and
//! that difference has one path on both sides: a field, a tag or a length, as the derive names
//! it (a variant and then its field, `Len` for a length).

use serde::ser::{self, Serialize};

use super::{Encoder, Unencodable, length, tag};
use crate::{Difference, Segment};

/// A primitive's path and canonical bytes.
pub(super) type Leaf = (Vec<Segment>, Vec<u8>);

/// The leaves of `value`.
pub(super) fn leaves<T: Serialize + ?Sized>(value: &T) -> Result<Vec<Leaf>, Unencodable> {
    let mut leaves = Leaves::default();
    value.serialize(&mut leaves)?;
    Ok(leaves.out)
}

/// The first leaf where `left` and `right` differ, as a [`Difference`].
pub(super) fn first_difference(left: &[Leaf], right: &[Leaf]) -> Option<Difference> {
    let found = left
        .iter()
        .zip(right)
        .find(|(left, right)| left != right)
        .map(|((left_path, left), (right_path, right))| {
            // One type's leaves share their paths up to the first difference; should they
            // part, the difference is placed where they still agree.
            let path = left_path
                .iter()
                .zip(right_path)
                .take_while(|(left, right)| left == right)
                .map(|(segment, _)| *segment)
                .collect();
            Difference {
                path,
                left: left.clone(),
                right: right.clone(),
            }
        });
    found.or_else(|| {
        (left.len() != right.len()).then(|| Difference {
            path: Vec::new(),
            left: left.iter().flat_map(|(_, bytes)| bytes.clone()).collect(),
            right: right.iter().flat_map(|(_, bytes)| bytes.clone()).collect(),
        })
    })
}

/// The names the derive gives a tuple's fields, by position.
const POSITIONS: [&str; 16] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15",
];

/// A tuple field's segment: its position's name, or its index past the names.
fn position(index: usize) -> Segment {
    POSITIONS
        .get(index)
        .map_or(Segment::Index(index), |name| Segment::Field(name))
}

#[derive(Default)]
struct Leaves {
    path: Vec<Segment>,
    out: Vec<Leaf>,
}

impl Leaves {
    fn leaf(&mut self, bytes: Vec<u8>) {
        self.out.push((self.path.clone(), bytes));
    }

    /// A primitive's leaf: its canonical bytes at the current path.
    fn primitive<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unencodable> {
        let mut bytes = Vec::new();
        value.serialize(&mut Encoder { out: &mut bytes })?;
        self.leaf(bytes);
        Ok(())
    }

    /// `value`'s leaves under `segment`.
    fn under<T: Serialize + ?Sized>(
        &mut self,
        segment: Segment,
        value: &T,
    ) -> Result<(), Unencodable> {
        self.path.push(segment);
        let result = value.serialize(&mut *self);
        self.path.pop();
        result
    }

    /// An enum's tag as a leaf, then the path into its variant.
    fn enter(&mut self, index: u32, variant: &'static str) {
        self.leaf(vec![tag(index)]);
        self.path.push(Segment::Variant(variant));
    }
}

/// A compound value's fields being walked: a tuple, struct or variant, or a sequence or map,
/// whose length leaf goes in at `start` once it is known.
struct Compound<'l> {
    leaves: &'l mut Leaves,
    index: usize,
    /// Whether a variant was entered, to leave at the end.
    variant: bool,
    /// For a sequence or map, where its length leaf goes.
    start: Option<usize>,
}

impl<'l> Compound<'l> {
    const fn fields(leaves: &'l mut Leaves, variant: bool) -> Self {
        Self {
            leaves,
            index: 0,
            variant,
            start: None,
        }
    }

    const fn counted(leaves: &'l mut Leaves) -> Self {
        let start = Some(leaves.out.len());
        Self {
            leaves,
            index: 0,
            variant: false,
            start,
        }
    }

    const fn next(&mut self) -> usize {
        let index = self.index;
        self.index = self.index.saturating_add(1);
        index
    }

    /// Puts the length leaf in, if any, and leaves the variant, if one was entered.
    fn finish(self) {
        if let Some(start) = self.start {
            let mut path = self.leaves.path.clone();
            path.push(Segment::Len);
            let start = start.min(self.leaves.out.len());
            self.leaves
                .out
                .insert(start, (path, length(self.index).to_vec()));
        }
        if self.variant {
            self.leaves.path.pop();
        }
    }
}

macro_rules! primitive {
    ($($method:ident: $ty:ty),* $(,)?) => {$(
        fn $method(self, value: $ty) -> Result<(), Unencodable> {
            self.primitive(&value)
        }
    )*};
}

impl<'l> ser::Serializer for &'l mut Leaves {
    type Ok = ();
    type Error = Unencodable;
    type SerializeSeq = Compound<'l>;
    type SerializeTuple = Compound<'l>;
    type SerializeTupleStruct = Compound<'l>;
    type SerializeTupleVariant = Compound<'l>;
    type SerializeMap = Compound<'l>;
    type SerializeStruct = Compound<'l>;
    type SerializeStructVariant = Compound<'l>;

    primitive!(
        serialize_bool: bool, serialize_i8: i8, serialize_i16: i16, serialize_i32: i32,
        serialize_i64: i64, serialize_i128: i128, serialize_u8: u8, serialize_u16: u16,
        serialize_u32: u32, serialize_u64: u64, serialize_u128: u128, serialize_f32: f32,
        serialize_f64: f64, serialize_char: char,
    );

    fn serialize_str(self, value: &str) -> Result<(), Unencodable> {
        self.primitive(value)
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<(), Unencodable> {
        let mut bytes = length(value.len()).to_vec();
        bytes.extend_from_slice(value);
        self.leaf(bytes);
        Ok(())
    }

    fn serialize_none(self) -> Result<(), Unencodable> {
        self.leaf(vec![0]);
        Ok(())
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), Unencodable> {
        self.leaf(vec![1]);
        self.under(Segment::Variant("Some"), value)
    }

    fn serialize_unit(self) -> Result<(), Unencodable> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Unencodable> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        index: u32,
        _variant: &'static str,
    ) -> Result<(), Unencodable> {
        self.leaf(vec![tag(index)]);
        Ok(())
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Unencodable> {
        self.under(position(0), value)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Unencodable> {
        self.enter(index, variant);
        let result = self.under(position(0), value);
        self.path.pop();
        result
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Compound<'l>, Unencodable> {
        Ok(Compound::counted(self))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Compound<'l>, Unencodable> {
        Ok(Compound::fields(self, false))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Compound<'l>, Unencodable> {
        Ok(Compound::fields(self, false))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Compound<'l>, Unencodable> {
        self.enter(index, variant);
        Ok(Compound::fields(self, true))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Compound<'l>, Unencodable> {
        Ok(Compound::counted(self))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Compound<'l>, Unencodable> {
        Ok(Compound::fields(self, false))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Compound<'l>, Unencodable> {
        self.enter(index, variant);
        Ok(Compound::fields(self, true))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl ser::SerializeSeq for Compound<'_> {
    type Ok = ();
    type Error = Unencodable;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unencodable> {
        let index = self.next();
        self.leaves.under(Segment::Index(index), value)
    }

    fn end(self) -> Result<(), Unencodable> {
        self.finish();
        Ok(())
    }
}

impl ser::SerializeTuple for Compound<'_> {
    type Ok = ();
    type Error = Unencodable;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unencodable> {
        let index = self.next();
        self.leaves.under(Segment::Index(index), value)
    }

    fn end(self) -> Result<(), Unencodable> {
        self.finish();
        Ok(())
    }
}

impl ser::SerializeTupleStruct for Compound<'_> {
    type Ok = ();
    type Error = Unencodable;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unencodable> {
        let index = self.next();
        self.leaves.under(position(index), value)
    }

    fn end(self) -> Result<(), Unencodable> {
        self.finish();
        Ok(())
    }
}

impl ser::SerializeTupleVariant for Compound<'_> {
    type Ok = ();
    type Error = Unencodable;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unencodable> {
        let index = self.next();
        self.leaves.under(position(index), value)
    }

    fn end(self) -> Result<(), Unencodable> {
        self.finish();
        Ok(())
    }
}

impl ser::SerializeMap for Compound<'_> {
    type Ok = ();
    type Error = Unencodable;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Unencodable> {
        let index = self.index;
        self.leaves.path.push(Segment::Index(index));
        let result = self.leaves.under(Segment::Field("key"), key);
        self.leaves.path.pop();
        result
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unencodable> {
        let index = self.next();
        self.leaves.path.push(Segment::Index(index));
        let result = self.leaves.under(Segment::Field("value"), value);
        self.leaves.path.pop();
        result
    }

    fn end(self) -> Result<(), Unencodable> {
        self.finish();
        Ok(())
    }
}

impl ser::SerializeStruct for Compound<'_> {
    type Ok = ();
    type Error = Unencodable;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Unencodable> {
        self.leaves.under(Segment::Field(key), value)
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), Unencodable> {
        Err(super::skipped(key))
    }

    fn end(self) -> Result<(), Unencodable> {
        self.finish();
        Ok(())
    }
}

impl ser::SerializeStructVariant for Compound<'_> {
    type Ok = ();
    type Error = Unencodable;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Unencodable> {
        self.leaves.under(Segment::Field(key), value)
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), Unencodable> {
        Err(super::skipped(key))
    }

    fn end(self) -> Result<(), Unencodable> {
        self.finish();
        Ok(())
    }
}
