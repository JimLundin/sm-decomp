//! The hurt colours frames: each one frame of her hurt colours, played through the one update
//! with her appearance State saved and restored around it. Her row's colours are invented
//! fixtures, told apart by their red level.

use enum_map::EnumMap;
use samus_appearance::{
    AppearanceState, Figure, Frame, FrameOutputs, GrappleActivity, HurtCount, HurtOpportunity,
    MoveEvent, PulseColours, Rules, Suit,
};
use serde::Deserialize;
use types::{Action, Aim, Colour, Facing, Gun, Legs, Pose, RowColours, Set, Sprite, Turn};

use super::{
    Library, VectorInputs, action, asked, behaviour, frame, named, play, read_file, sounds,
    untouched,
};

/// One frame of her hurt colours, as the file's header gives it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HurtVector {
    id: String,
    source: String,
    pub(super) entries: Vec<String>,
    before: Before,
    energy_fell: bool,
    push_started: bool,
    #[serde(default)]
    cinematic: bool,
    pose_after: Option<PoseAfter>,
    #[serde(default)]
    items: Vec<String>,
    grapple: Option<GrappleActivity>,
    after: After,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Before {
    count: u8,
}

/// Her pose as the frame ends, by its body and action; the rest as the default's.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PoseAfter {
    body: String,
    action: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct After {
    count: u8,
    row: Fixture,
    sounds: Vec<String>,
}

/// An invented row of 16 colours, as the file names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Fixture {
    /// Her row before the frame.
    Before,
    /// Her hurt colours.
    Hurt,
    /// Her Power suit's colours.
    Suit,
    /// The cinematic's grey colours.
    Grey,
}

/// Every fixture the file names.
const FIXTURES: [Fixture; 4] = [Fixture::Before, Fixture::Hurt, Fixture::Suit, Fixture::Grey];

/// The red level of each invented row; the other suits' rows, which no frame shows, have their
/// own.
const fn red_level(fixture: Fixture) -> u16 {
    match fixture {
        Fixture::Before => 1,
        Fixture::Hurt => 2,
        Fixture::Suit => 3,
        Fixture::Grey => 4,
    }
}

/// The Varia and Gravity suits' invented rows' red levels.
const OTHER_SUITS: [u16; 2] = [5, 6];

/// An invented row: colour `i` holds red at `red`, green `i` and blue 0, so every colour of a
/// row and every row differs.
fn row(red: u16) -> RowColours {
    RowColours(std::array::from_fn(|slot| {
        let slot = u16::try_from(slot).expect("expected a row's colour below 16");
        Colour::from_channels([red, slot, 0])
    }))
}

/// The invented colours of her hurt pulse, the content every frame of the file reads.
pub(super) fn invented_pulse_colours() -> PulseColours {
    let [varia, gravity] = OTHER_SUITS.map(row);
    PulseColours {
        hurt: row(red_level(Fixture::Hurt)),
        suits: EnumMap::from_fn(|suit| match suit {
            Suit::Power => row(red_level(Fixture::Suit)),
            Suit::Varia => varia,
            Suit::Gravity => gravity,
        }),
        cinematic_grey: row(red_level(Fixture::Grey)),
    }
}

/// An action as the file writes it for a hurt frame. The file writes a turn without the turn
/// she is in; the rule reads only whether she spins, so it stands here as a turn on the ground.
fn pulse_action(name: &str) -> Action {
    if name == "Turning" {
        Action::Turning(Turn::Ground)
    } else {
        action(name)
    }
}

/// Her figure as the frame ends: the file's body and action, or upright running by default,
/// facing right, unaimed, in the plain sprite.
fn figure_after(given: Option<&PoseAfter>) -> Figure {
    let (body, action) = given.map_or((types::Body::Upright, Action::Running), |pose| {
        (named(&pose.body), pulse_action(&pose.action))
    });
    Figure {
        pose: Pose {
            body,
            action,
            facing: Facing::Right,
            aim: Aim::Forward,
            gun: Gun::Stowed,
            legs: Legs::Together,
        },
        sprite: Sprite::Plain,
    }
}

/// The pose the frame's inputs are read with, before her figure as the frame ends replaces it.
const READ_POSE: &str = "Upright Running Right Forward";

/// The frame of `vector`: her hurt colours alone, in the Power suit, after a frame that leaves
/// her in its figure. Her energy falls by one where the file says it fell.
fn hurt_frame(vector: &HurtVector) -> Frame {
    let given = VectorInputs {
        pose: READ_POSE.to_owned(),
        items: vector.items.clone(),
        cinematic: vector.cinematic,
        ..VectorInputs::default()
    };
    let mut frame = frame(&given, Set::default());
    let figure = figure_after(vector.pose_after.as_ref());
    frame.inputs.figure = figure;
    frame.inputs.figure_after = figure;
    if vector.energy_fell {
        frame.inputs.previous_energy = frame.inputs.energy.saturating_add(1);
    }
    frame.inputs.moves[MoveEvent::PushStarted] = vector.push_started;
    frame.hurt = HurtOpportunity::Observed {
        suit: Suit::Power,
        grapple: vector.grapple.unwrap_or(GrappleActivity::Dormant),
    };
    frame
}

/// The fixture her row holds after the frame: the one the frame gave it, or "before" where it
/// left her row as it is.
fn row_after(outputs: &FrameOutputs, id: &str) -> Fixture {
    let Some(colours) = outputs.her_colours else {
        return Fixture::Before;
    };
    FIXTURES
        .into_iter()
        .find(|&fixture| row(red_level(fixture)) == colours)
        .unwrap_or_else(|| panic!("{id}: expected her row to take a fixture, found {colours:?}"))
}

fn hurt_count(frames: u8) -> HurtCount {
    HurtCount::new(frames)
        .unwrap_or_else(|| panic!("expected a hurt count of 0 to 59, found {frames}"))
}

/// One hurt frame from its count: the count, her row and what the speaker plays as `after` lists
/// it, the rest of her State as it was, and no other output.
fn run(vector: &HurtVector, library: &Library) {
    let id = &vector.id;
    let before = AppearanceState {
        hurt_count: hurt_count(vector.before.count),
        ..untouched()
    };
    let frame = hurt_frame(vector);
    let (after, outputs) = play(before, &frame, &library.without_clips(), &Rules::VANILLA);
    assert_eq!(
        after,
        AppearanceState {
            hurt_count: hurt_count(vector.after.count),
            ..before
        },
        "{id} ({}): expected the hurt count `after` gives, the rest of her State as it was",
        vector.source
    );
    assert_eq!(
        row_after(&outputs, id),
        vector.after.row,
        "{id}: expected her row as `after` gives it"
    );
    assert_eq!(
        asked(&outputs),
        sounds(&vector.after.sounds),
        "{id}: expected, in order, what the speaker plays as `after` lists it"
    );
    assert_eq!(outputs.contact, None, "{id}: expected no graphical contact");
    assert_eq!(outputs.dry_colours, None, "{id}: expected no palette reset");
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "report the number of supplied hurt colours frames executed"
)]
fn every_hurt_colours_frame_passes() {
    let (file, library) = read_file();
    assert!(
        !file.hurt_colours.is_empty(),
        "expected hurt colours frames in the file"
    );
    for vector in &file.hurt_colours {
        assert_ne!(vector.source, "", "{}: expected a source", vector.id);
        for entry in &vector.entries {
            behaviour(entry);
        }
        run(vector, &library);
    }
    println!("{} hurt colours frames passed", file.hurt_colours.len());
}

/// The hurt count saves as its number and restores only within 0 to 59.
#[test]
fn the_hurt_count_restores_only_in_range() {
    let state = AppearanceState {
        hurt_count: hurt_count(HurtCount::MOST),
        ..AppearanceState::INITIAL
    };
    let text = toml::to_string(&state).expect("expected the State to save");
    assert!(
        text.contains("hurt_count = 59"),
        "expected the hurt count saved: {text}"
    );
    assert!(
        toml::from_str::<AppearanceState>(&text.replace("hurt_count = 59", "hurt_count = 60"))
            .is_err(),
        "expected a hurt count of 60 refused"
    );
    assert_eq!(HurtCount::new(60), None, "expected 60 refused");
}
