//! The codec, diff and schema of derived types, seen from a crate that derives them.

use canonical::{Canonical, DecodeError, Schema, Segment, from_bytes, to_bytes};

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

/// One shape of a type per module, so each can keep the type's name.
mod shapes {
    pub mod original {
        #[derive(canonical::Canonical, Debug, PartialEq, Eq)]
        pub struct Point {
            pub x: u16,
            pub y: i16,
        }
    }

    pub mod same {
        #[derive(canonical::Canonical, Debug, PartialEq, Eq)]
        pub struct Point {
            pub x: u16,
            pub y: i16,
        }
    }

    pub mod renamed {
        #[derive(canonical::Canonical, Debug, PartialEq, Eq)]
        pub struct Point {
            pub x: u16,
            pub z: i16,
        }
    }

    pub mod reordered {
        #[derive(canonical::Canonical, Debug, PartialEq, Eq)]
        pub struct Point {
            pub y: i16,
            pub x: u16,
        }
    }

    pub mod retyped {
        #[derive(canonical::Canonical, Debug, PartialEq, Eq)]
        pub struct Point {
            pub x: u16,
            pub y: u16,
        }
    }

    /// Holds a `Point`, so a change below it shows in its own schema.
    #[derive(canonical::Canonical, Debug, PartialEq, Eq)]
    pub struct Outer<P> {
        pub point: P,
    }

    /// A family marker, as a slot store's type parameter.
    #[derive(canonical::Canonical, Debug, PartialEq, Eq)]
    pub struct Plm;

    /// Another family marker.
    #[derive(canonical::Canonical, Debug, PartialEq, Eq)]
    pub struct Enemy;

    /// A label that carries its family only as a type.
    #[derive(canonical::Canonical, Debug, PartialEq, Eq)]
    pub struct Label<F> {
        pub at: u16,
        pub family: std::marker::PhantomData<F>,
    }
}

#[test]
fn identical_shapes_have_equal_schemas() {
    use shapes::{original, same};
    assert_eq!(original::Point::SCHEMA, same::Point::SCHEMA);
    let point = original::Point { x: 1, y: -1 };
    let copy = same::Point { x: 1, y: -1 };
    assert_eq!(to_bytes(&point), to_bytes(&copy));
}

#[test]
fn a_rename_reorder_or_retype_changes_the_schema() {
    use shapes::{original, renamed, reordered, retyped};
    let schemas = [
        original::Point::SCHEMA,
        renamed::Point::SCHEMA,
        reordered::Point::SCHEMA,
        retyped::Point::SCHEMA,
    ];
    for (position, schema) in schemas.iter().enumerate() {
        assert!(!schemas[position + 1..].contains(schema), "{position}");
    }
    // Every shape still encodes; only its schema tells them apart.
    let bytes = [
        to_bytes(&original::Point { x: 1, y: 2 }),
        to_bytes(&renamed::Point { x: 1, z: 2 }),
        to_bytes(&reordered::Point { y: 2, x: 1 }),
        to_bytes(&retyped::Point { x: 1, y: 2 }),
    ];
    assert_eq!(bytes[0], [1, 0, 2, 0]);
    assert_eq!(bytes[1], bytes[3]);
}

#[test]
fn a_change_below_a_type_changes_its_schema() {
    use shapes::{Outer, original, retyped};
    assert_ne!(
        Outer::<original::Point>::SCHEMA,
        Outer::<retyped::Point>::SCHEMA
    );
    assert_ne!(<[u8; 3]>::SCHEMA, <[u8; 4]>::SCHEMA);
    assert_ne!(<Vec<u8>>::SCHEMA, <Vec<u16>>::SCHEMA);
    assert_ne!(<Option<u8>>::SCHEMA, u8::SCHEMA);
}

#[test]
fn a_type_parameter_changes_the_schema_even_as_a_marker() {
    use shapes::{Enemy, Label, Plm};
    assert_ne!(Label::<Plm>::SCHEMA, Label::<Enemy>::SCHEMA);
    let plm = Label::<Plm> {
        at: 3,
        family: std::marker::PhantomData,
    };
    let enemy = Label::<Enemy> {
        at: 3,
        family: std::marker::PhantomData,
    };
    assert_eq!(to_bytes(&plm), to_bytes(&enemy));
    assert_eq!(to_bytes(&Plm), to_bytes(&Enemy));
}

#[test]
fn a_schema_decodes_only_its_own_value() {
    use shapes::{original, retyped};
    let written = to_bytes(&Schema::<original::Point>::new());
    assert_eq!(written, original::Point::SCHEMA.to_le_bytes());
    assert_eq!(
        from_bytes::<Schema<original::Point>>(&written),
        Ok(Schema::new())
    );
    assert_eq!(
        from_bytes::<Schema<retyped::Point>>(&written),
        Err(DecodeError::SchemaMismatch {
            expected: retyped::Point::SCHEMA,
            found: original::Point::SCHEMA,
        })
    );
}
