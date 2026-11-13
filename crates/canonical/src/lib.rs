//! The one canonical codec of the State (K6): encode, decode, diff and schema.
//!
//! Every State type, and every `data` type the State holds, derives [`Canonical`] with
//! `#[derive(canonical::Canonical)]`. It is its own crate, not part of `game`, so `data` can
//! derive it without depending on `game`. The Checkpoint is this encoding of the whole State
//! inside its envelope (schema, fingerprint), and lockstep uses [`Canonical::diff`] to name
//! the first field where the engine and the decoded oracle State part.
//!
//! The encoding is byte-exact and has no per-type version: [`Canonical::SCHEMA`], a hash of
//! the type's structure, versions a whole State, and [`Schema`] carries it in an envelope.
//! Integers are fixed-width little-endian, `bool` is one byte that must be 0 or 1, an enum is
//! a one-byte tag (declaration order) followed by its fields, `Option` is a tag then its
//! value, a `Vec` is a `u32` length then its elements, and a fixed array is its elements with
//! no length.

use std::fmt;
use std::marker::PhantomData;

pub use canonical_derive::Canonical;

pub mod model;

/// A type with one canonical encoding, a fallible decoder, a field-path diff and a schema.
pub trait Canonical: Sized {
    /// A hash of this type's structure: its kind and name, its variants' and fields' names in
    /// order and their types' schemas. Two types with equal schemas encode alike.
    const SCHEMA: u64;

    /// Appends this value's canonical bytes to `out`.
    fn encode(&self, out: &mut Vec<u8>);

    /// Reads one value from the front of `input`.
    ///
    /// # Errors
    /// [`DecodeError`] when the bytes end early or hold a value the type can't have.
    fn decode(input: &mut Reader<'_>) -> Result<Self, DecodeError>;

    /// The first field, in declaration order, where `self` and `other` differ.
    fn diff(&self, other: &Self) -> Option<Difference>;
}

/// The canonical bytes of `value`.
pub fn to_bytes<T: Canonical>(value: &T) -> Vec<u8> {
    let mut out = Vec::new();
    value.encode(&mut out);
    out
}

/// Decodes a whole value from `bytes`, which must hold nothing after it.
///
/// # Errors
/// [`DecodeError`] when the bytes don't hold exactly one `T`.
pub fn from_bytes<T: Canonical>(bytes: &[u8]) -> Result<T, DecodeError> {
    let mut input = Reader::new(bytes);
    let value = T::decode(&mut input)?;
    match input.remaining() {
        0 => Ok(value),
        left => Err(DecodeError::TrailingBytes(left)),
    }
}

/// Why canonical bytes didn't decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// The bytes ended inside a value.
    UnexpectedEnd,
    /// A `bool` byte other than 0 or 1.
    InvalidBool(u8),
    /// An enum or `Option` tag with no variant.
    UnknownVariant {
        /// The enum's name.
        type_name: &'static str,
        /// The tag read.
        tag: u8,
    },
    /// A fixed array's elements didn't fill it.
    ArrayLength,
    /// Bytes left over after a whole value.
    TrailingBytes(usize),
    /// Bytes a model's own serde code refused, or a model's State the codec couldn't
    /// write ([`model`]).
    Refused,
    /// An envelope written by a different shape of the type it holds.
    SchemaMismatch {
        /// This build's schema.
        expected: u64,
        /// The schema the bytes were written with.
        found: u64,
    },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd => write!(f, "the bytes end inside a value"),
            Self::InvalidBool(byte) => write!(f, "bool byte {byte:#04x} is neither 0 nor 1"),
            Self::UnknownVariant { type_name, tag } => {
                write!(f, "{type_name} has no variant {tag}")
            }
            Self::ArrayLength => write!(f, "an array's elements don't fill it"),
            Self::TrailingBytes(count) => write!(f, "{count} bytes after the value"),
            Self::Refused => write!(f, "a model's State refused the bytes"),
            Self::SchemaMismatch { expected, found } => {
                write!(
                    f,
                    "schema {found:#018x} is not this build's {expected:#018x}"
                )
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Builds a [`Canonical::SCHEMA`] at compile time: FNV-1a over the structure's names, counts
/// and the schemas of its parts, each in a self-delimiting form so no two shapes collide by
/// concatenation.
#[derive(Clone, Copy, Debug)]
#[must_use]
pub struct SchemaHasher(u64);

impl SchemaHasher {
    /// FNV-1a's 64-bit offset basis.
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    /// FNV-1a's 64-bit prime.
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    /// A hasher that has seen nothing.
    pub const fn new() -> Self {
        Self(Self::OFFSET)
    }

    const fn bytes(self, mut bytes: &[u8]) -> Self {
        let mut hash = self.0;
        while let [byte, rest @ ..] = bytes {
            hash =
                (hash ^ u64::from_le_bytes([*byte, 0, 0, 0, 0, 0, 0, 0])).wrapping_mul(Self::PRIME);
            bytes = rest;
        }
        Self(hash)
    }

    /// Mixes in a count as eight little-endian bytes, the same on every target.
    pub const fn count(self, count: usize) -> Self {
        let bytes = count.to_le_bytes();
        let mut hasher = self.bytes(&bytes);
        let mut padding = size_of::<u64>().saturating_sub(bytes.len());
        while padding > 0 {
            hasher = hasher.bytes(&[0]);
            padding = padding.saturating_sub(1);
        }
        hasher
    }

    /// Mixes in a name, prefixed by its length.
    pub const fn name(self, name: &str) -> Self {
        self.count(name.len()).bytes(name.as_bytes())
    }

    /// Mixes in a part's schema.
    pub const fn schema(self, schema: u64) -> Self {
        self.bytes(&schema.to_le_bytes())
    }

    /// The schema.
    #[must_use]
    pub const fn finish(self) -> u64 {
        self.0
    }
}

impl Default for SchemaHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// `T`'s schema, as an envelope carries it.
///
/// It encodes [`Canonical::SCHEMA`] as a `u64` and decodes only that value, so bytes written
/// by another shape of `T` fail with [`DecodeError::SchemaMismatch`] before `T` is read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Schema<T>(PhantomData<T>);

impl<T: Canonical> Schema<T> {
    /// The schema of this build's `T`.
    #[must_use]
    pub const fn new() -> Self {
        Self(PhantomData)
    }

    /// Its value, `T::SCHEMA`.
    #[must_use]
    pub const fn value(self) -> u64 {
        T::SCHEMA
    }
}

impl<T: Canonical> Canonical for Schema<T> {
    const SCHEMA: u64 = SchemaHasher::new()
        .name("Schema")
        .schema(T::SCHEMA)
        .finish();

    fn encode(&self, out: &mut Vec<u8>) {
        T::SCHEMA.encode(out);
    }

    fn decode(input: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match u64::decode(input)? {
            found if found == T::SCHEMA => Ok(Self::new()),
            found => Err(DecodeError::SchemaMismatch {
                expected: T::SCHEMA,
                found,
            }),
        }
    }

    // Every value of a `Schema<T>` is the same: this build's `T::SCHEMA`.
    fn diff(&self, _other: &Self) -> Option<Difference> {
        None
    }
}

/// The unread rest of a canonical encoding.
#[derive(Clone, Copy, Debug)]
pub struct Reader<'a> {
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    /// A reader over `bytes`.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { rest: bytes }
    }

    /// How many bytes are still unread.
    #[must_use]
    pub const fn remaining(&self) -> usize {
        self.rest.len()
    }

    /// Takes the next `N` bytes.
    ///
    /// # Errors
    /// [`DecodeError::UnexpectedEnd`] when fewer than `N` are left.
    pub fn take<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let (bytes, rest) = self
            .rest
            .split_first_chunk::<N>()
            .ok_or(DecodeError::UnexpectedEnd)?;
        self.rest = rest;
        Ok(*bytes)
    }

    /// Takes the next `len` bytes.
    ///
    /// # Errors
    /// [`DecodeError::UnexpectedEnd`] when fewer than `len` are left.
    pub fn take_slice(&mut self, len: usize) -> Result<&'a [u8], DecodeError> {
        let (bytes, rest) = self
            .rest
            .split_at_checked(len)
            .ok_or(DecodeError::UnexpectedEnd)?;
        self.rest = rest;
        Ok(bytes)
    }
}

/// One step of a path into the State.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Segment {
    /// A named field, or the position of an unnamed one.
    Field(&'static str),
    /// The enum variant both sides hold.
    Variant(&'static str),
    /// An element of an array or `Vec`.
    Index(usize),
    /// A `Vec`'s length.
    Len,
}

/// The first place two values differ: its path and both sides' canonical bytes there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Difference {
    path: Vec<Segment>,
    left: Vec<u8>,
    right: Vec<u8>,
}

impl Difference {
    /// A difference at the current value itself, holding both sides' encodings.
    pub fn here<T: Canonical>(left: &T, right: &T) -> Self {
        Self {
            path: Vec::new(),
            left: to_bytes(left),
            right: to_bytes(right),
        }
    }

    /// The same difference, seen from the value that contains it at `segment`.
    #[must_use]
    pub fn within(mut self, segment: Segment) -> Self {
        self.path.insert(0, segment);
        self
    }

    /// The path from the compared value down to the difference.
    #[must_use]
    pub fn path(&self) -> &[Segment] {
        &self.path
    }

    /// The left side's canonical bytes at the path.
    #[must_use]
    pub fn left(&self) -> &[u8] {
        &self.left
    }

    /// The right side's canonical bytes at the path.
    #[must_use]
    pub fn right(&self) -> &[u8] {
        &self.right
    }
}

/// The path as `samus.vertical.Rising[2].len`, for logs and lockstep reports.
impl fmt::Display for Difference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (position, segment) in self.path.iter().enumerate() {
            match (position, segment) {
                (0, Segment::Field(name) | Segment::Variant(name)) => write!(f, "{name}")?,
                (_, Segment::Field(name) | Segment::Variant(name)) => write!(f, ".{name}")?,
                (0, Segment::Len) => write!(f, "len")?,
                (_, Segment::Len) => write!(f, ".len")?,
                (_, Segment::Index(index)) => write!(f, "[{index}]")?,
            }
        }
        if self.path.is_empty() {
            write!(f, "(value)")?;
        }
        write!(f, ": ")?;
        write_hex(f, &self.left)?;
        write!(f, " != ")?;
        write_hex(f, &self.right)
    }
}

fn write_hex(f: &mut fmt::Formatter<'_>, bytes: &[u8]) -> fmt::Result {
    bytes.iter().try_for_each(|byte| write!(f, "{byte:02x}"))
}

macro_rules! canonical_integer {
    ($($integer:ty),*) => {$(
        impl Canonical for $integer {
            const SCHEMA: u64 = SchemaHasher::new().name(stringify!($integer)).finish();

            fn encode(&self, out: &mut Vec<u8>) {
                out.extend_from_slice(&self.to_le_bytes());
            }

            fn decode(input: &mut Reader<'_>) -> Result<Self, DecodeError> {
                input.take().map(Self::from_le_bytes)
            }

            fn diff(&self, other: &Self) -> Option<Difference> {
                (self != other).then(|| Difference::here(self, other))
            }
        }
    )*};
}

canonical_integer!(u8, u16, u32, u64, i8, i16, i32, i64);

impl Canonical for bool {
    const SCHEMA: u64 = SchemaHasher::new().name("bool").finish();

    fn encode(&self, out: &mut Vec<u8>) {
        out.push(u8::from(*self));
    }

    fn decode(input: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match u8::decode(input)? {
            0 => Ok(false),
            1 => Ok(true),
            byte => Err(DecodeError::InvalidBool(byte)),
        }
    }

    fn diff(&self, other: &Self) -> Option<Difference> {
        (self != other).then(|| Difference::here(self, other))
    }
}

impl<T: Canonical> Canonical for Option<T> {
    const SCHEMA: u64 = SchemaHasher::new()
        .name("Option")
        .schema(T::SCHEMA)
        .finish();

    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            None => out.push(0),
            Some(value) => {
                out.push(1);
                value.encode(out);
            }
        }
    }

    fn decode(input: &mut Reader<'_>) -> Result<Self, DecodeError> {
        match u8::decode(input)? {
            0 => Ok(None),
            1 => T::decode(input).map(Some),
            tag => Err(DecodeError::UnknownVariant {
                type_name: "Option",
                tag,
            }),
        }
    }

    fn diff(&self, other: &Self) -> Option<Difference> {
        match (self, other) {
            (Some(left), Some(right)) => left
                .diff(right)
                .map(|difference| difference.within(Segment::Variant("Some"))),
            (None, None) => None,
            (Some(_), None) | (None, Some(_)) => Some(Difference::here(self, other)),
        }
    }
}

/// The first differing element of two equal-length sequences, by index.
fn diff_elements<T: Canonical>(left: &[T], right: &[T]) -> Option<Difference> {
    left.iter()
        .zip(right)
        .enumerate()
        .find_map(|(index, (left, right))| {
            left.diff(right)
                .map(|difference| difference.within(Segment::Index(index)))
        })
}

impl<T: Canonical, const N: usize> Canonical for [T; N] {
    const SCHEMA: u64 = SchemaHasher::new()
        .name("array")
        .count(N)
        .schema(T::SCHEMA)
        .finish();

    fn encode(&self, out: &mut Vec<u8>) {
        for element in self {
            element.encode(out);
        }
    }

    fn decode(input: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let elements = (0..N)
            .map(|_| T::decode(input))
            .collect::<Result<Vec<T>, _>>()?;
        elements.try_into().map_err(|_| DecodeError::ArrayLength)
    }

    fn diff(&self, other: &Self) -> Option<Difference> {
        diff_elements(self, other)
    }
}

impl<T: Canonical> Canonical for Vec<T> {
    const SCHEMA: u64 = SchemaHasher::new().name("Vec").schema(T::SCHEMA).finish();

    // A State `Vec` is bounded far below `u32::MAX` elements (Session invariants), so the
    // saturation never changes the length; it only keeps this path free of panics.
    fn encode(&self, out: &mut Vec<u8>) {
        u32::try_from(self.len()).unwrap_or(u32::MAX).encode(out);
        for element in self {
            element.encode(out);
        }
    }

    fn decode(input: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let len = usize::try_from(u32::decode(input)?).map_err(|_| DecodeError::UnexpectedEnd)?;
        // Corrupt input can claim any length; reserve no more than the bytes could hold.
        let mut elements = Self::with_capacity(len.min(input.remaining()));
        for _ in 0..len {
            elements.push(T::decode(input)?);
        }
        Ok(elements)
    }

    fn diff(&self, other: &Self) -> Option<Difference> {
        if self.len() == other.len() {
            diff_elements(self, other)
        } else {
            let len = |elements: &Self| u32::try_from(elements.len()).unwrap_or(u32::MAX);
            Some(Difference::here(&len(self), &len(other)).within(Segment::Len))
        }
    }
}

/// A type marker carries no State. Its schema is fixed, since `T` needn't be `Canonical`; a
/// derived generic type mixes in its own parameters' schemas instead.
impl<T> Canonical for PhantomData<T> {
    const SCHEMA: u64 = SchemaHasher::new().name("PhantomData").finish();

    fn encode(&self, _out: &mut Vec<u8>) {}

    fn decode(_input: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self)
    }

    fn diff(&self, _other: &Self) -> Option<Difference> {
        None
    }
}
