use serde::{Deserialize, Serialize};

use super::{Model, Pinned, Unpinned, pin_holds, shape};
use crate::{Canonical, DecodeError, Segment, from_bytes, to_bytes};

/// An invented model State, with each form a model's State takes.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
struct Lift {
    floor: Floor,
    phase: Phase,
    riders: Vec<u16>,
    waiting: Option<Call>,
    lights: [bool; 3],
    load: (i8, u32),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Floor(u8);

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Parked,
    Moving(Direction),
    Doors {
        open: bool,
        wait: u16,
    },
    Stuck(Floor, Option<Call>),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
struct Call {
    from: Floor,
    to: Floor,
    urgent: Urgency,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
enum Urgency {
    Routine,
    Fire,
}

struct LiftPin;

impl Pinned for LiftPin {
    type State = Lift;
    const NAME: &'static str = "lift";
    const SHAPE: u64 = 0;
}

fn busy() -> Lift {
    Lift {
        floor: Floor(3),
        phase: Phase::Doors {
            open: true,
            wait: 0x1234,
        },
        riders: vec![7, 0xbeef],
        waiting: Some(Call {
            from: Floor(1),
            to: Floor(9),
            urgent: Urgency::Fire,
        }),
        lights: [true, false, true],
        load: (-2, 0x0102_0304),
    }
}

#[test]
fn a_model_state_encodes_by_the_derives_rules_and_decodes_back() {
    let encoding = to_bytes(&Model::<LiftPin>(busy()));
    let payload = [
        vec![3],                            // floor
        vec![2, 1, 0x34, 0x12],             // Doors, open, wait
        vec![2, 0, 0, 0, 7, 0, 0xef, 0xbe], // riders: a length, then each
        vec![1, 1, 9, 1],                   // Some, from, to, Fire
        vec![1, 0, 1],                      // lights: an array has no length
        vec![0xfe, 4, 3, 2, 1],             // load
    ]
    .concat();
    let framed = [
        u32::try_from(payload.len()).unwrap().to_le_bytes().to_vec(),
        payload,
    ]
    .concat();
    assert_eq!(encoding, framed);
    assert_eq!(from_bytes::<Model<LiftPin>>(&encoding), Ok(Model(busy())));
    let parked = Model::<LiftPin>::default();
    assert_eq!(from_bytes(&to_bytes(&parked)), Ok(parked));
}

#[test]
fn bytes_that_dont_hold_the_state_are_refused_by_the_codecs_errors() {
    let mut encoding = to_bytes(&Model::<LiftPin>(busy()));
    // The phase's tag, past Doors and Stuck.
    encoding[5] = 7;
    assert_eq!(
        from_bytes::<Model<LiftPin>>(&encoding),
        Err(DecodeError::UnknownVariant {
            type_name: "Phase",
            tag: 7
        })
    );
    encoding[5] = 2;
    // Doors' open flag.
    encoding[6] = 2;
    assert_eq!(
        from_bytes::<Model<LiftPin>>(&encoding),
        Err(DecodeError::InvalidBool(2))
    );
    encoding[6] = 1;
    // A frame one byte short, or one byte long.
    let short = encoding.len() - 1;
    assert_eq!(
        from_bytes::<Model<LiftPin>>(&encoding[..short]),
        Err(DecodeError::UnexpectedEnd)
    );
    encoding[0] += 1;
    encoding.push(0);
    assert_eq!(
        from_bytes::<Model<LiftPin>>(&encoding),
        Err(DecodeError::TrailingBytes(1))
    );
}

#[test]
fn the_diff_names_the_first_differing_field_as_the_derive_does() {
    let left = Model::<LiftPin>(busy());
    let diff = |change: fn(&mut Lift)| {
        let mut right = busy();
        change(&mut right);
        left.diff(&Model(right))
            .map(|difference| (difference.path().to_vec(), difference.to_string()))
    };
    assert_eq!(left.diff(&left.clone()), None);
    assert_eq!(
        diff(|lift| lift.phase = Phase::Doors {
            open: true,
            wait: 5
        }),
        Some((
            vec![
                Segment::Field("phase"),
                Segment::Variant("Doors"),
                Segment::Field("wait")
            ],
            String::from("phase.Doors.wait: 3412 != 0500")
        ))
    );
    assert_eq!(
        diff(|lift| lift.phase = Phase::Parked),
        Some((
            vec![Segment::Field("phase")],
            String::from("phase: 02 != 00")
        ))
    );
    assert_eq!(
        diff(|lift| lift.riders.push(1)),
        Some((
            vec![Segment::Field("riders"), Segment::Len],
            String::from("riders.len: 02000000 != 03000000")
        ))
    );
    assert_eq!(
        diff(|lift| lift.riders[1] = 2),
        Some((
            vec![Segment::Field("riders"), Segment::Index(1)],
            String::from("riders[1]: efbe != 0200")
        ))
    );
    assert_eq!(
        diff(|lift| lift.waiting = None),
        Some((
            vec![Segment::Field("waiting")],
            String::from("waiting: 01 != 00")
        ))
    );
    assert_eq!(
        diff(|lift| lift.floor = Floor(4)),
        Some((
            vec![Segment::Field("floor"), Segment::Field("0")],
            String::from("floor.0: 03 != 04")
        ))
    );
    assert_eq!(
        diff(|lift| lift.load.1 = 0),
        Some((
            vec![Segment::Field("load"), Segment::Index(1)],
            String::from("load[1]: 04030201 != 00000000")
        ))
    );
}

// Other shapes under the same serde names, as a State changed between two builds would be.

#[derive(Serialize, Deserialize)]
#[serde(rename = "Phase")]
enum Same {
    Parked,
    Moving(Direction),
    Doors { open: bool, wait: u16 },
    Stuck(Floor, Option<Call>),
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "Phase")]
enum Reordered {
    Moving(Direction),
    Parked,
    Doors { open: bool, wait: u16 },
    Stuck(Floor, Option<Call>),
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "Phase")]
enum Wider {
    Parked,
    Moving(Direction),
    Doors { open: bool, wait: u32 },
    Stuck(Floor, Option<Call>),
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "Urgency")]
enum MoreUrgent {
    Routine,
    Fire,
    Flood,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "Call")]
struct OtherCall {
    from: Floor,
    to: Floor,
    urgent: MoreUrgent,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "Phase")]
enum DeepChange {
    Parked,
    Moving(Direction),
    Doors { open: bool, wait: u16 },
    Stuck(Floor, Option<OtherCall>),
}

/// A type serde reads by trying each form, which says nothing of its shape.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Loose {
    Number(u8),
}

#[test]
fn the_shape_follows_names_order_types_and_every_variant() {
    let lift = shape::<Lift>().unwrap();
    assert_eq!(shape::<Lift>(), Ok(lift));
    let phase = shape::<Phase>().unwrap();
    assert_eq!(shape::<Same>(), Ok(phase));
    assert_ne!(shape::<Reordered>().unwrap(), phase);
    assert_ne!(shape::<Wider>().unwrap(), phase);
    // A change deep in a variant only the later passes reach still moves the shape.
    assert_ne!(shape::<DeepChange>().unwrap(), phase);
}

#[test]
fn a_pin_holds_only_while_it_is_the_traced_shape() {
    struct Current;
    impl Pinned for Current {
        type State = Lift;
        const NAME: &'static str = "lift";
        const SHAPE: u64 = 0;
    }
    let traced = shape::<Lift>().unwrap();
    assert_eq!(pin_holds::<Current>(), Err(Unpinned::Stale { traced }));
    assert_ne!(Model::<Current>::SCHEMA, Model::<LiftPin>::SCHEMA ^ 1);
}

#[test]
fn a_state_the_codec_cant_place_is_refused_whole() {
    #[derive(Serialize, Deserialize, Default)]
    struct Skips {
        #[serde(skip_serializing_if = "Option::is_none")]
        note: Option<u8>,
        after: u16,
    }
    struct SkipsPin;
    impl Pinned for SkipsPin {
        type State = Skips;
        const NAME: &'static str = "skips";
        const SHAPE: u64 = 0;
    }
    let encoding = to_bytes(&Model::<SkipsPin>(Skips::default()));
    assert_eq!(encoding, u32::MAX.to_le_bytes());
    assert!(matches!(
        from_bytes::<Model<SkipsPin>>(&encoding),
        Err(DecodeError::Refused)
    ));

    // A type that reads anything can't be traced, so it can't be pinned.
    assert!(shape::<Loose>().is_err());
}

#[test]
fn a_table_by_enum_is_its_entries_in_variant_order_and_traces_with_its_key() {
    use enum_map::{Enum, EnumMap, enum_map};

    #[derive(Serialize, Deserialize, Enum, Clone, Copy, Debug, PartialEq, Eq)]
    enum Shaft {
        East,
        West,
    }
    #[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
    struct Shafts(EnumMap<Shaft, u16>);
    struct ShaftsPin;
    impl Pinned for ShaftsPin {
        type State = Shafts;
        const NAME: &'static str = "shafts";
        const SHAPE: u64 = 0;
    }
    let shafts = Shafts(enum_map! { Shaft::East => 0x0102, Shaft::West => 0x0304 });
    let encoding = to_bytes(&Model::<ShaftsPin>(shafts));
    assert_eq!(encoding, [4, 0, 0, 0, 2, 1, 4, 3]);
    assert_eq!(
        from_bytes::<Model<ShaftsPin>>(&encoding).map(|model| model.0),
        Ok(Shafts(
            enum_map! { Shaft::East => 0x0102, Shaft::West => 0x0304 }
        ))
    );
    assert!(shape::<Shafts>().is_ok());
}
