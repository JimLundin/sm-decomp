//! A clean model's State in the engine's State, through serde (MODEL §1 decisions 21 and 29).
//!
//! A clean crate can't derive [`Canonical`]: it depends only on `types`, std and the registry
//! crates the clean room allows, serde among them. So a model's State derives serde's
//! `Serialize` and `Deserialize`, and the engine holds it as a [`Model`], whose encoding,
//! decoding and diff follow the same rules as the derive's (the crate docs): fixed-width
//! little-endian integers, a `bool` as 0 or 1, an enum as a one-byte tag in declaration order
//! then its fields, an `Option` as a tag then its value, a sequence or map as a `u32` length then
//! its elements, a struct, tuple or array as its fields with no length.
//!
//! [`Canonical::SCHEMA`] is a constant, and serde shows a type's shape only by running it, so
//! the engine pins each model's shape ([`Pinned::SHAPE`]) and a test holds the pin to the shape
//! [`shape`] traces from the type: a State that changes shape fails that test until the pin is
//! moved, and a Checkpoint written before it then no longer opens.

use std::fmt;

use serde::de::{self, Deserialize, DeserializeOwned, DeserializeSeed, IntoDeserializer, Visitor};
use serde::ser::{self, Serialize};

use crate::{Canonical, DecodeError, Difference, Reader, SchemaHasher};

mod leaves;
mod shape;
#[cfg(test)]
mod tests;

use leaves::{first_difference, leaves};
pub use shape::shape;

/// A model's State type, with its shape as the engine pins it.
pub trait Pinned {
    /// The model's State.
    type State: Serialize + DeserializeOwned;
    /// The model's name, mixed into the schema.
    const NAME: &'static str;
    /// The State's shape as [`shape`] traces it, written down.
    const SHAPE: u64;
}

/// Whether the pinned shape is still the State's: `Err` with the shape traced now when it
/// isn't, or when the State can't be traced.
///
/// # Errors
/// [`Unpinned`] when the pin is stale or the State's shape can't be traced.
pub fn pin_holds<P: Pinned>() -> Result<(), Unpinned> {
    match shape::<P::State>() {
        Ok(traced) if traced == P::SHAPE => Ok(()),
        Ok(traced) => Err(Unpinned::Stale { traced }),
        Err(error) => Err(Unpinned::Untraceable(error)),
    }
}

/// Why a model's pinned shape doesn't hold.
#[derive(Debug, PartialEq, Eq)]
pub enum Unpinned {
    /// The State's shape is now `traced`: move the pin to it.
    Stale {
        /// The shape traced from the State today.
        traced: u64,
    },
    /// The State can't be traced, so it can't be held this way.
    Untraceable(Unencodable),
}

/// A model's State, held in the engine's State as the model gives it.
pub struct Model<P: Pinned>(pub P::State);

impl<P: Pinned> Clone for Model<P>
where
    P::State: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<P: Pinned> fmt::Debug for Model<P>
where
    P::State: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<P: Pinned> PartialEq for Model<P>
where
    P::State: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<P: Pinned> Eq for Model<P> where P::State: Eq {}

impl<P: Pinned> Default for Model<P>
where
    P::State: Default,
{
    fn default() -> Self {
        Self(P::State::default())
    }
}

/// The length a State that can't be encoded is written with: no State is that long, so its
/// decode fails rather than reading another State's bytes.
const UNENCODABLE: u32 = u32::MAX;

/// The bytes are framed by their length, so a decode reads exactly the State's and a State
/// serde can't encode is written as [`UNENCODABLE`], which no decode accepts.
impl<P: Pinned> Canonical for Model<P> {
    const SCHEMA: u64 = SchemaHasher::new()
        .name("Model")
        .name(P::NAME)
        .schema(P::SHAPE)
        .finish();

    fn encode(&self, out: &mut Vec<u8>) {
        let mut payload = Vec::new();
        match self.0.serialize(&mut Encoder { out: &mut payload }) {
            Ok(()) => {
                out.extend_from_slice(&length(payload.len()));
                out.append(&mut payload);
            }
            Err(_) => out.extend_from_slice(&UNENCODABLE.to_le_bytes()),
        }
    }

    fn decode(input: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let len = u32::decode(input)?;
        if len == UNENCODABLE {
            return Err(DecodeError::Refused);
        }
        let len = usize::try_from(len).map_err(|_| DecodeError::UnexpectedEnd)?;
        let mut payload = Reader::new(input.take_slice(len)?);
        let state = P::State::deserialize(&mut Decoder {
            input: &mut payload,
        })
        .map_err(|error| match error {
            Unencodable::Decode(error) => error,
            Unencodable::Custom(_) => DecodeError::Refused,
        })?;
        match payload.remaining() {
            0 => Ok(Self(state)),
            left => Err(DecodeError::TrailingBytes(left)),
        }
    }

    fn diff(&self, other: &Self) -> Option<Difference> {
        match (leaves(&self.0), leaves(&other.0)) {
            (Ok(left), Ok(right)) => first_difference(&left, &right),
            _ => (crate::to_bytes(self) != crate::to_bytes(other))
                .then(|| Difference::here(self, other)),
        }
    }
}

/// Why serde couldn't encode, decode or trace a State.
#[derive(Debug, PartialEq, Eq)]
pub enum Unencodable {
    /// The canonical bytes don't hold a value.
    Decode(DecodeError),
    /// The State's own serde code, or a form the codec doesn't take, refused it.
    Custom(String),
}

impl fmt::Display for Unencodable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(error) => error.fmt(f),
            Self::Custom(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for Unencodable {}

impl ser::Error for Unencodable {
    fn custom<T: fmt::Display>(message: T) -> Self {
        Self::Custom(message.to_string())
    }
}

impl de::Error for Unencodable {
    fn custom<T: fmt::Display>(message: T) -> Self {
        Self::Custom(message.to_string())
    }
}

impl From<DecodeError> for Unencodable {
    fn from(error: DecodeError) -> Self {
        Self::Decode(error)
    }
}

/// A length as the codec writes it: a `u32`, saturating as the derive's `Vec` does.
fn length(len: usize) -> [u8; 4] {
    u32::try_from(len).unwrap_or(u32::MAX).to_le_bytes()
}

/// An enum's tag as the codec writes it: one byte, in declaration order.
fn tag(index: u32) -> u8 {
    u8::try_from(index).unwrap_or(u8::MAX)
}

/// Writes a value's canonical bytes.
struct Encoder<'o> {
    out: &'o mut Vec<u8>,
}

/// A sequence or map being written: its length first, filled in at the end, since serde
/// needn't know it in advance.
struct Counted<'e, 'o> {
    encoder: &'e mut Encoder<'o>,
    at: usize,
    count: usize,
}

impl<'e, 'o> Counted<'e, 'o> {
    fn start(encoder: &'e mut Encoder<'o>) -> Self {
        let at = encoder.out.len();
        encoder.out.extend_from_slice(&[0; 4]);
        Self {
            encoder,
            at,
            count: 0,
        }
    }

    fn element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unencodable> {
        self.count = self.count.saturating_add(1);
        value.serialize(&mut *self.encoder)
    }

    fn end(self) -> Result<(), Unencodable> {
        let len = length(self.count);
        let slot = self
            .encoder
            .out
            .get_mut(self.at..self.at.saturating_add(len.len()))
            .ok_or_else(|| Unencodable::Custom(String::from("a length's place is gone")))?;
        slot.copy_from_slice(&len);
        Ok(())
    }
}

macro_rules! encode_le {
    ($($method:ident: $ty:ty),* $(,)?) => {$(
        fn $method(self, value: $ty) -> Result<(), Unencodable> {
            self.out.extend_from_slice(&value.to_le_bytes());
            Ok(())
        }
    )*};
}

impl<'e, 'o> ser::Serializer for &'e mut Encoder<'o> {
    type Ok = ();
    type Error = Unencodable;
    type SerializeSeq = Counted<'e, 'o>;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Counted<'e, 'o>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    encode_le!(
        serialize_i8: i8, serialize_i16: i16, serialize_i32: i32, serialize_i64: i64,
        serialize_i128: i128, serialize_u8: u8, serialize_u16: u16, serialize_u32: u32,
        serialize_u64: u64, serialize_u128: u128,
    );

    fn serialize_bool(self, value: bool) -> Result<(), Unencodable> {
        self.out.push(u8::from(value));
        Ok(())
    }

    fn serialize_f32(self, value: f32) -> Result<(), Unencodable> {
        self.serialize_u32(value.to_bits())
    }

    fn serialize_f64(self, value: f64) -> Result<(), Unencodable> {
        self.serialize_u64(value.to_bits())
    }

    fn serialize_char(self, value: char) -> Result<(), Unencodable> {
        self.serialize_u32(u32::from(value))
    }

    fn serialize_str(self, value: &str) -> Result<(), Unencodable> {
        self.serialize_bytes(value.as_bytes())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<(), Unencodable> {
        self.out.extend_from_slice(&length(value.len()));
        self.out.extend_from_slice(value);
        Ok(())
    }

    fn serialize_none(self) -> Result<(), Unencodable> {
        self.out.push(0);
        Ok(())
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), Unencodable> {
        self.out.push(1);
        value.serialize(self)
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
        self.out.push(tag(index));
        Ok(())
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Unencodable> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<(), Unencodable> {
        self.out.push(tag(index));
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Counted<'e, 'o>, Unencodable> {
        Ok(Counted::start(self))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self, Unencodable> {
        Ok(self)
    }

    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self, Unencodable> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self, Unencodable> {
        self.out.push(tag(index));
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Counted<'e, 'o>, Unencodable> {
        Ok(Counted::start(self))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self, Unencodable> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self, Unencodable> {
        self.out.push(tag(index));
        Ok(self)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl ser::SerializeSeq for Counted<'_, '_> {
    type Ok = ();
    type Error = Unencodable;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unencodable> {
        self.element(value)
    }

    fn end(self) -> Result<(), Unencodable> {
        Counted::end(self)
    }
}

impl ser::SerializeMap for Counted<'_, '_> {
    type Ok = ();
    type Error = Unencodable;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Unencodable> {
        self.element(key)
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unencodable> {
        value.serialize(&mut *self.encoder)
    }

    fn end(self) -> Result<(), Unencodable> {
        Counted::end(self)
    }
}

impl ser::SerializeTuple for &mut Encoder<'_> {
    type Ok = ();
    type Error = Unencodable;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unencodable> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Unencodable> {
        Ok(())
    }
}

impl ser::SerializeTupleStruct for &mut Encoder<'_> {
    type Ok = ();
    type Error = Unencodable;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unencodable> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Unencodable> {
        Ok(())
    }
}

impl ser::SerializeTupleVariant for &mut Encoder<'_> {
    type Ok = ();
    type Error = Unencodable;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Unencodable> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Unencodable> {
        Ok(())
    }
}

/// A skipped field would leave the fields after it in the wrong places.
fn skipped(key: &'static str) -> Unencodable {
    Unencodable::Custom(format!(
        "field {key} is skipped, which the codec can't place"
    ))
}

impl ser::SerializeStruct for &mut Encoder<'_> {
    type Ok = ();
    type Error = Unencodable;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Unencodable> {
        value.serialize(&mut **self)
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), Unencodable> {
        Err(skipped(key))
    }

    fn end(self) -> Result<(), Unencodable> {
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut Encoder<'_> {
    type Ok = ();
    type Error = Unencodable;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Unencodable> {
        value.serialize(&mut **self)
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), Unencodable> {
        Err(skipped(key))
    }

    fn end(self) -> Result<(), Unencodable> {
        Ok(())
    }
}

/// Reads a value from its canonical bytes, by the type's own hints: the format doesn't
/// describe itself.
struct Decoder<'r, 'a> {
    input: &'r mut Reader<'a>,
}

impl Decoder<'_, '_> {
    fn len(&mut self) -> Result<usize, Unencodable> {
        let len = u32::decode(self.input)?;
        usize::try_from(len).map_err(|_| Unencodable::Decode(DecodeError::UnexpectedEnd))
    }

    fn tag(&mut self) -> Result<u8, Unencodable> {
        Ok(u8::decode(self.input)?)
    }
}

/// The elements of a sequence, map, tuple or struct still to read.
struct Elements<'d, 'r, 'a> {
    decoder: &'d mut Decoder<'r, 'a>,
    left: usize,
}

impl<'de> de::SeqAccess<'de> for Elements<'_, '_, '_> {
    type Error = Unencodable;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Unencodable> {
        if self.left == 0 {
            return Ok(None);
        }
        self.left = self.left.saturating_sub(1);
        seed.deserialize(&mut *self.decoder).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.left)
    }
}

impl<'de> de::MapAccess<'de> for Elements<'_, '_, '_> {
    type Error = Unencodable;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Unencodable> {
        if self.left == 0 {
            return Ok(None);
        }
        self.left = self.left.saturating_sub(1);
        seed.deserialize(&mut *self.decoder).map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Unencodable> {
        seed.deserialize(&mut *self.decoder)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.left)
    }
}

/// An enum's variant, its tag checked against the variants the type has.
struct Variant<'d, 'r, 'a> {
    decoder: &'d mut Decoder<'r, 'a>,
    name: &'static str,
    count: usize,
}

impl<'de, 'd, 'r, 'a> de::EnumAccess<'de> for Variant<'d, 'r, 'a> {
    type Error = Unencodable;
    type Variant = &'d mut Decoder<'r, 'a>;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Unencodable> {
        let tag = self.decoder.tag()?;
        if usize::from(tag) >= self.count {
            return Err(Unencodable::Decode(DecodeError::UnknownVariant {
                type_name: self.name,
                tag,
            }));
        }
        let index: de::value::U32Deserializer<Unencodable> = u32::from(tag).into_deserializer();
        Ok((seed.deserialize(index)?, self.decoder))
    }
}

impl<'de> de::VariantAccess<'de> for &mut Decoder<'_, '_> {
    type Error = Unencodable;

    fn unit_variant(self) -> Result<(), Unencodable> {
        Ok(())
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<T::Value, Unencodable> {
        seed.deserialize(self)
    }

    fn tuple_variant<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        visitor.visit_seq(Elements {
            decoder: self,
            left: len,
        })
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        visitor.visit_seq(Elements {
            decoder: self,
            left: fields.len(),
        })
    }
}

macro_rules! decode_le {
    ($($method:ident: $ty:ty => $visit:ident),* $(,)?) => {$(
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
            visitor.$visit(<$ty>::from_le_bytes(self.input.take()?))
        }
    )*};
}

/// A form the format can't read: it doesn't say what it holds.
fn undescribed() -> Unencodable {
    Unencodable::Custom(String::from(
        "the codec doesn't describe itself: a type must say what it reads",
    ))
}

impl<'de> de::Deserializer<'de> for &mut Decoder<'_, '_> {
    type Error = Unencodable;

    decode_le!(
        deserialize_i8: i8 => visit_i8, deserialize_i16: i16 => visit_i16,
        deserialize_i32: i32 => visit_i32, deserialize_i64: i64 => visit_i64,
        deserialize_i128: i128 => visit_i128, deserialize_u8: u8 => visit_u8,
        deserialize_u16: u16 => visit_u16, deserialize_u32: u32 => visit_u32,
        deserialize_u64: u64 => visit_u64, deserialize_u128: u128 => visit_u128,
    );

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Unencodable> {
        Err(undescribed())
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        match self.tag()? {
            0 => visitor.visit_bool(false),
            1 => visitor.visit_bool(true),
            byte => Err(Unencodable::Decode(DecodeError::InvalidBool(byte))),
        }
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        visitor.visit_f32(f32::from_bits(u32::from_le_bytes(self.input.take()?)))
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        visitor.visit_f64(f64::from_bits(u64::from_le_bytes(self.input.take()?)))
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        let code = u32::from_le_bytes(self.input.take()?);
        let character = char::from_u32(code)
            .ok_or_else(|| Unencodable::Custom(format!("{code:#x} is not a character")))?;
        visitor.visit_char(character)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        let len = self.len()?;
        let bytes = self.input.take_slice(len)?;
        let text = std::str::from_utf8(bytes)
            .map_err(|_| Unencodable::Custom(String::from("a string isn't UTF-8")))?;
        visitor.visit_str(text)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        let len = self.len()?;
        visitor.visit_bytes(self.input.take_slice(len)?)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        match self.tag()? {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(self),
            tag => Err(Unencodable::Decode(DecodeError::UnknownVariant {
                type_name: "Option",
                tag,
            })),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        let left = self.len()?;
        visitor.visit_seq(Elements {
            decoder: self,
            left,
        })
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        visitor.visit_seq(Elements {
            decoder: self,
            left: len,
        })
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Unencodable> {
        let left = self.len()?;
        visitor.visit_map(Elements {
            decoder: self,
            left,
        })
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Unencodable> {
        visitor.visit_enum(Variant {
            decoder: self,
            name,
            count: variants.len(),
        })
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Unencodable> {
        Err(undescribed())
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(
        self,
        _visitor: V,
    ) -> Result<V::Value, Unencodable> {
        Err(undescribed())
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}
