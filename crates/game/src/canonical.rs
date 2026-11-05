//! The one canonical codec of the State (K6): encode, decode and diff.
//!
//! Every State type derives [`Canonical`] with `#[derive(canonical::Canonical)]`. The
//! Checkpoint is this encoding of the whole State inside its envelope (schema, fingerprint),
//! and lockstep uses [`Canonical::diff`] to name the first field where the engine and the
//! decoded oracle State part.
//!
//! The encoding is byte-exact and has no per-type version: the Checkpoint's schema field
//! versions the State as a whole. Integers are fixed-width little-endian, `bool` is one
//! byte that must be 0 or 1, an enum is a one-byte tag (declaration order) followed by its
//! fields, `Option` is a tag then its value, a `Vec` is a `u32` length then its elements, and
//! a fixed array is its elements with no length.

use std::fmt;
use std::marker::PhantomData;

/// A type with one canonical encoding, a fallible decoder and a field-path diff.
pub trait Canonical: Sized {
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
        }
    }
}

impl std::error::Error for DecodeError {}

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

/// A type marker carries no State.
impl<T> Canonical for PhantomData<T> {
    fn encode(&self, _out: &mut Vec<u8>) {}

    fn decode(_input: &mut Reader<'_>) -> Result<Self, DecodeError> {
        Ok(Self)
    }

    fn diff(&self, _other: &Self) -> Option<Difference> {
        None
    }
}

/// `data`'s newtypes, which can't derive `Canonical` there because `data` can't depend on
/// `game`. Each encodes as its one field, like a derived newtype.
macro_rules! canonical_data_newtype {
    ($($newtype:ident($inner:ty)),* $(,)?) => {$(
        impl Canonical for data::id::$newtype {
            fn encode(&self, out: &mut Vec<u8>) {
                self.0.encode(out);
            }

            fn decode(input: &mut Reader<'_>) -> Result<Self, DecodeError> {
                <$inner>::decode(input).map(Self)
            }

            fn diff(&self, other: &Self) -> Option<Difference> {
                self.0.diff(&other.0)
            }
        }
    )*};
}

canonical_data_newtype!(
    RoomId(u16),
    DoorwayId(u16),
    SpeciesId(u16),
    ProgramId(u16),
    Line(u16),
    Cycles(u32),
);

#[cfg(test)]
mod tests {
    use super::{Canonical, DecodeError, Segment, from_bytes, to_bytes};
    use canonical::Canonical;

    /// A newtype, as for a slot index or a frame count.
    #[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
    struct Speed(u16);

    /// An enum with unit, tuple and struct variants.
    #[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
    enum Vertical {
        /// On the ground.
        Grounded,
        Rising(Speed),
        Falling {
            speed: Speed,
            since: u8,
        },
        Knocked(Speed, bool),
    }

    #[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
    struct Enemy {
        /// Attributes and visibility on a field are skipped.
        pub(crate) health: u16,
        frozen: Option<u8>,
    }

    #[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
    struct Samus {
        x: i16,
        vertical: Vertical,
    }

    #[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
    struct Marker;

    /// A State-shaped root: a nested struct, a fixed array of records and a bounded `Vec`.
    #[derive(Canonical, Clone, Debug, PartialEq, Eq)]
    struct State {
        samus: Samus,
        enemies: [Enemy; 3],
        frames: Vec<Vertical>,
        marker: Marker,
    }

    fn sample() -> State {
        State {
            samus: Samus {
                x: -2,
                vertical: Vertical::Falling {
                    speed: Speed(0x0123),
                    since: 4,
                },
            },
            enemies: [
                Enemy {
                    health: 20,
                    frozen: None,
                },
                Enemy {
                    health: 30,
                    frozen: Some(7),
                },
                Enemy {
                    health: 0,
                    frozen: None,
                },
            ],
            frames: vec![
                Vertical::Grounded,
                Vertical::Rising(Speed(5)),
                Vertical::Knocked(Speed(9), true),
            ],
            marker: Marker,
        }
    }

    #[test]
    fn round_trip_is_identity() {
        let state = sample();
        assert_eq!(from_bytes::<State>(&to_bytes(&state)), Ok(state));
    }

    #[test]
    fn encoding_is_byte_exact() {
        let samus = Samus {
            x: -2,
            vertical: Vertical::Falling {
                speed: Speed(0x0123),
                since: 4,
            },
        };
        assert_eq!(to_bytes(&samus), [0xfe, 0xff, 2, 0x23, 0x01, 4]);
        assert_eq!(to_bytes(&vec![Some(true), None]), [2, 0, 0, 0, 1, 1, 0]);
        assert_eq!(to_bytes(&Marker), [0u8; 0]);
    }

    #[test]
    fn decode_rejects_bad_bytes() {
        assert_eq!(
            from_bytes::<Vertical>(&[4]),
            Err(DecodeError::UnknownVariant {
                type_name: "Vertical",
                tag: 4
            })
        );
        assert_eq!(from_bytes::<bool>(&[2]), Err(DecodeError::InvalidBool(2)));
        assert_eq!(from_bytes::<u16>(&[1]), Err(DecodeError::UnexpectedEnd));
        assert_eq!(
            from_bytes::<u8>(&[1, 2]),
            Err(DecodeError::TrailingBytes(1))
        );
        assert_eq!(
            from_bytes::<Vec<u8>>(&[0xff, 0xff, 0xff, 0xff]),
            Err(DecodeError::UnexpectedEnd)
        );
    }

    #[test]
    fn equal_values_have_no_difference() {
        assert_eq!(sample().diff(&sample()), None);
    }

    #[test]
    fn diff_names_a_field_inside_a_variant() {
        let mut changed = sample();
        changed.samus.vertical = Vertical::Falling {
            speed: Speed(0x0124),
            since: 4,
        };
        let difference = sample().diff(&changed).unwrap();
        assert_eq!(
            difference.path(),
            [
                Segment::Field("samus"),
                Segment::Field("vertical"),
                Segment::Variant("Falling"),
                Segment::Field("speed")
            ]
        );
        assert_eq!(
            (difference.left(), difference.right()),
            (&[0x23, 0x01][..], &[0x24, 0x01][..])
        );
        assert_eq!(
            difference.to_string(),
            "samus.vertical.Falling.speed: 2301 != 2401"
        );
    }

    #[test]
    fn diff_stops_at_a_different_variant() {
        let mut changed = sample();
        changed.samus.vertical = Vertical::Grounded;
        let difference = sample().diff(&changed).unwrap();
        assert_eq!(difference.to_string(), "samus.vertical: 02230104 != 00");
    }

    #[test]
    fn diff_indexes_arrays_and_reports_the_first_field() {
        let mut changed = sample();
        changed.enemies[2].health = 1;
        changed.enemies[1].frozen = Some(8);
        assert_eq!(
            sample().diff(&changed).unwrap().to_string(),
            "enemies[1].frozen.Some: 07 != 08"
        );
    }

    #[test]
    fn diff_reports_a_vec_length_then_elements() {
        let mut changed = sample();
        changed.frames.push(Vertical::Grounded);
        assert_eq!(
            sample().diff(&changed).unwrap().to_string(),
            "frames.len: 03000000 != 04000000"
        );
        let mut changed = sample();
        changed.frames[2] = Vertical::Knocked(Speed(9), false);
        assert_eq!(
            sample().diff(&changed).unwrap().path(),
            [
                Segment::Field("frames"),
                Segment::Index(2),
                Segment::Variant("Knocked"),
                Segment::Field("1")
            ]
        );
    }

    #[test]
    fn a_newtype_is_transparent_in_paths() {
        let difference = Vertical::Rising(Speed(1))
            .diff(&Vertical::Rising(Speed(2)))
            .unwrap();
        assert_eq!(difference.path(), [Segment::Variant("Rising")]);
        assert_eq!(Speed(1).diff(&Speed(2)).unwrap().path(), []);
    }
}
