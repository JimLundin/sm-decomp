//! Runs the fresh and suspended wall-jump dust draws, comparing State and picture work.

use core::num;

use enum_map::EnumMap;
use samus_appearance::{
    AppearanceState, Dust, DustArrival, DustClip, DustDrawing, DustLook, DustPlacement, FramePart,
    RoomPoint, Rules,
};
use serde::Deserialize;
use types::{Pair, Side, set_of};

use super::{behaviour, frame, kept, moving_content, play, read_file, untouched};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DustVector {
    id: String,
    source: String,
    pub(super) entries: Vec<String>,
    before: Puff,
    inputs: DrawInputs,
    after: Puff,
    placement: Picture,
    #[serde(default)]
    clip: ClipFixture,
    #[serde(default)]
    restore: bool,
}

#[derive(Deserialize, Default)]
enum ClipFixture {
    #[default]
    Reference,
    Adjusted,
}

impl ClipFixture {
    fn content(&self) -> DustClip {
        match self {
            Self::Reference => DustClip::reference(),
            Self::Adjusted => DustClip(EnumMap::from_array([1, 4, 2, 3].map(|visits| {
                num::NonZeroU16::new(visits).expect("expected positive visit length")
            }))),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Puff {
    centre: Option<[u16; 2]>,
    look: Option<DustLook>,
    visits_left: Option<num::NonZeroU16>,
}

impl Puff {
    fn state(&self) -> Option<Dust> {
        match (self.centre, self.look, self.visits_left) {
            (None, None, None) => None,
            (Some(centre), Some(look), Some(visits_left)) => Some(Dust {
                centre: point(centre),
                look,
                visits_left,
            }),
            _ => panic!("expected all three dust quantities or none"),
        }
    }
}

#[derive(Deserialize, Default)]
enum Arrival {
    #[default]
    None,
    DryTouchdown,
    WallJump,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DrawInputs {
    view: [u16; 2],
    fresh: Option<bool>,
    #[serde(default)]
    arrival: Arrival,
    facing: Option<Side>,
    centre: Option<[u16; 2]>,
    half_height: Option<u16>,
}

impl DrawInputs {
    fn drawing(&self, clip: DustClip) -> DustDrawing {
        let arrival = match (&self.arrival, self.facing, self.centre, self.half_height) {
            (Arrival::None, None, None, None) => DustArrival::None,
            (Arrival::DryTouchdown, None, None, None) => DustArrival::DryTouchdown,
            (Arrival::WallJump, Some(facing), Some(centre), Some(half_height)) => {
                DustArrival::WallJump {
                    facing,
                    centre: point(centre),
                    half_height,
                }
            }
            _ => panic!("expected complete pose geometry only for a wall-jump arrival"),
        };
        if self.fresh.unwrap_or(true) {
            DustDrawing::Fresh {
                arrival,
                view: point(self.view),
                clip,
            }
        } else {
            DustDrawing::Skipped
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Picture {
    left: Option<u8>,
    top: Option<u8>,
    look: Option<DustLook>,
}

impl Picture {
    fn placement(&self) -> DustPlacement {
        match (self.left, self.top, self.look) {
            (None, None, None) => DustPlacement::None,
            (Some(left), None, None) => DustPlacement::HorizontalOnly { left },
            (Some(left), Some(top), Some(look)) => DustPlacement::SmallSprite { left, top, look },
            _ => {
                panic!("expected no placement, only a horizontal coordinate, or a complete sprite")
            }
        }
    }
}

fn point([x, y]: [u16; 2]) -> RoomPoint {
    Pair {
        x: num::Wrapping(x),
        y: num::Wrapping(y),
    }
}

impl DustVector {
    fn run(&self) {
        let before = AppearanceState {
            dust: self.before.state(),
            ..untouched()
        };
        let mut state = if self.restore { kept(before) } else { before };
        let placement = state.draw_wall_jump_dust(self.inputs.drawing(self.clip.content()));
        assert_eq!(
            state,
            AppearanceState {
                dust: self.after.state(),
                ..before
            },
            "{} ({}): appearance State after drawing",
            self.id,
            self.source
        );
        assert_eq!(
            placement,
            self.placement.placement(),
            "{} ({}): dust placement",
            self.id,
            self.source
        );
    }
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "report the number of supplied wall-jump dust vectors executed"
)]
fn wall_jump_dust_vectors() {
    let (file, _) = read_file();
    for vector in &file.wall_jump_dust {
        for entry in &vector.entries {
            behaviour(entry);
        }
        vector.run();
    }
    println!(
        "{} wall-jump dust vectors passed",
        file.wall_jump_dust.len()
    );
}

#[test]
fn a_frame_keeps_the_puff_until_drawing_or_initialization() {
    let (file, library) = read_file();
    let case = &file.appearance_work[0];
    let before = AppearanceState {
        dust: Some(Dust {
            centre: point([210, 615]),
            look: DustLook::First,
            visits_left: num::NonZeroU16::MIN,
        }),
        ..untouched()
    };
    let content = moving_content(&case.id, &case.inputs, &library);
    let drawing = frame(&case.inputs, set_of([FramePart::Drawing]));
    let (carried, _) = play(before, &drawing, &content, &Rules::VANILLA);
    assert_eq!(
        carried.dust, before.dust,
        "dust waits for its own drawing opportunity"
    );
    let initialization = frame(&case.inputs, set_of([FramePart::Initialization]));
    let (initialized, _) = play(carried, &initialization, &content, &Rules::VANILLA);
    assert_eq!(initialized.dust, None, "dust starts absent");
}
