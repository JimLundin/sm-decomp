//! Vector rows for the tilting room: overlay placement and rumble, room rotation,
//! elevator reach, and vent drawing offsets.

use core::num::Wrapping;

use boss_ceres_ridley::{
    AlternatingLook, BurstPoint, BurstWait, ColourSet, DoorPalette, EarlierShaftPose,
    ElevatorAnimation, ElevatorLock, FadeSteps, LightsInputs, MountedVent, Rules, RumbleParticle,
    RumbleTurns, ShaftOverlay, ShaftOverlayPhase, ShaftPlacement, ShaftRotation, ShaftView, Stage,
    StartShaftRotation, TiltDirection, TiltSample, TiltTrack, Vent, VentDrawOffset, VentFacing,
    elevator_reached, overlay_figure, place_shaft_overlay, turn_shaft_overlay, turn_shaft_rotation,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use types::{Facing, Pair, Px, Side, VerticalSpeed};

use super::source::Source;

#[test]
fn shaft_turn_publishes_after_vent_and_locks_only_in_play() {
    let rules = Rules::default();
    let anchor = point([99, 99]);
    let mut vent = MountedVent::new(Vent::place(VentFacing::Left, anchor, 0));
    let mut rotation = ShaftRotation {
        active: false,
        next: 0,
        direction: TiltDirection::Forward,
        hold: 0,
        view: ShaftView {
            pivot: point([100, 100]),
            matrix: [256, 0, 0, 256],
        },
    };
    let samples = [
        TiltSample {
            hold: 0,
            diagonal: 250,
            cross: 13,
        },
        TiltSample {
            hold: 0,
            diagonal: 256,
            cross: 0,
        },
    ];
    let track = TiltTrack::new(&samples).unwrap();
    let her = ShaftPlacement {
        centre: point([144, 75]),
        vertical_speed: VerticalSpeed(types::ZERO),
        facing: Facing::Left,
        earlier_pose: EarlierShaftPose::Other,
    };
    vent.run(anchor, rotation.view, None, &rules);
    let turn =
        boss_ceres_ridley::turn_shaft(&mut rotation, Some(StartShaftRotation), track, Some(her));
    assert_eq!(vent.offset, VentDrawOffset::ZERO);
    assert_eq!(turn.view.matrix, [250, 13, -13, 250]);
    assert_eq!(
        turn.elevator.unwrap().animation,
        ElevatorAnimation::StartStanding
    );

    vent.run(anchor, turn.view, None, &rules);
    let during_scene = boss_ceres_ridley::turn_shaft(&mut rotation, None, track, None);
    assert_eq!(vent.offset, VentDrawOffset(Pair { x: 0, y: 1 }));
    assert_eq!(during_scene.view.matrix, [256, 0, 0, 256]);
    assert_eq!(during_scene.elevator, None);
}

fn point([x, y]: [u16; 2]) -> Pair<Px> {
    Pair {
        x: Wrapping(x),
        y: Wrapping(y),
    }
}

fn coordinates(p: Pair<Px>) -> [u16; 2] {
    [p.x.0, p.y.0]
}

fn restore<T: Serialize + DeserializeOwned>(state: &T) -> T {
    toml::from_str(&toml::to_string(state).unwrap()).unwrap()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OverlayVector {
    pub id: String,
    pub source: Source,
    pub entries: Vec<String>,
    before: Option<OverlayPhaseRow>,
    input: OverlayInput,
    after: OverlayPhaseRow,
    output: OverlayOutput,
}

#[derive(Deserialize)]
#[serde(tag = "phase", deny_unknown_fields)]
enum OverlayPhaseRow {
    Opening,
    Rumbling {
        remaining: u8,
        wait: u8,
        point: usize,
    },
    Hidden,
}

impl OverlayPhaseRow {
    fn phase(&self) -> ShaftOverlayPhase {
        match *self {
            Self::Opening => ShaftOverlayPhase::Opening,
            Self::Hidden => ShaftOverlayPhase::Hidden,
            Self::Rumbling {
                remaining,
                wait,
                point,
            } => ShaftOverlayPhase::Rumbling {
                remaining: RumbleTurns::new(remaining).unwrap(),
                wait: BurstWait::new(wait).unwrap(),
                point: [
                    BurstPoint::First,
                    BurstPoint::Second,
                    BurstPoint::Third,
                    BurstPoint::Fourth,
                ][point],
            },
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverlayInput {
    #[serde(default)]
    place: bool,
    anchor: [u16; 2],
    count: Option<u16>,
    fade_steps: Option<u8>,
    drawn_random: Option<u16>,
}

#[derive(Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
enum PlatformRow {
    First,
    Second,
}

#[derive(Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
enum ParticleRow {
    Smoke,
    SmallExplosion,
}

#[derive(Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
enum DoorColoursRow {
    Escape,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
struct OverlayOutput {
    #[serde(default)]
    upload: bool,
    door_colours: Option<DoorColoursRow>,
    #[serde(default)]
    figure: bool,
    beacon: Option<u16>,
    platform: Option<PlatformRow>,
    particle: Option<ParticleRow>,
    centre: Option<[u16; 2]>,
    sound_limit: Option<u8>,
    #[serde(default)]
    start_rotation: bool,
}

impl OverlayVector {
    pub fn check(&self) {
        let anchor = point(self.input.anchor);
        let (overlay, actual) = if self.input.place {
            self.place(anchor)
        } else {
            self.turn(anchor)
        };
        assert_eq!(
            overlay,
            ShaftOverlay {
                anchor,
                phase: self.after.phase()
            },
            "{}: overlay state",
            self.id
        );
        assert_eq!(actual, self.output, "{}: overlay outputs", self.id);
    }

    fn place(&self, anchor: Pair<Px>) -> (ShaftOverlay, OverlayOutput) {
        assert!(self.before.is_none());
        let placed = place_shaft_overlay(Stage::Escape, anchor).unwrap();
        assert_eq!(placed.door_colours.set, ColourSet::Escape);
        assert_eq!(placed.door_colours.palette, DoorPalette::Doors);
        assert_eq!(overlay_figure(&placed.overlay), None);
        let boss_ceres_ridley::OverlaySpriteUpload = placed.upload;
        (
            placed.overlay,
            OverlayOutput {
                upload: true,
                door_colours: Some(DoorColoursRow::Escape),
                ..OverlayOutput::default()
            },
        )
    }

    fn turn(&self, anchor: Pair<Px>) -> (ShaftOverlay, OverlayOutput) {
        let saved = ShaftOverlay {
            anchor,
            phase: self.before.as_ref().unwrap().phase(),
        };
        let mut overlay = restore(&saved);
        assert_eq!(overlay, saved);
        let mut random = self.input.drawn_random;
        let turn = turn_shaft_overlay(
            &mut overlay,
            LightsInputs {
                stage: Stage::Escape,
                finished_frames: self.input.count.unwrap_or(0),
                fade_steps_taken: FadeSteps::new(self.input.fade_steps.unwrap_or(0)).unwrap(),
            },
            || random.take().expect("only a due burst may draw"),
            &Rules::default(),
        );
        assert!(random.is_none(), "{}: expected a random draw", self.id);
        if let Some(figure) = turn.figure {
            assert_eq!(figure, anchor, "{}: figure anchor", self.id);
        }
        (
            overlay,
            OverlayOutput {
                figure: turn.figure.is_some(),
                beacon: turn
                    .lights
                    .and_then(|lights| lights.beacon)
                    .map(boss_ceres_ridley::BeaconStep::place),
                platform: turn
                    .lights
                    .and_then(|lights| lights.platform)
                    .map(|look| match look {
                        AlternatingLook::First => PlatformRow::First,
                        AlternatingLook::Second => PlatformRow::Second,
                    }),
                particle: turn.burst.map(|burst| match burst.particle {
                    RumbleParticle::Smoke => ParticleRow::Smoke,
                    RumbleParticle::SmallExplosion => ParticleRow::SmallExplosion,
                }),
                centre: turn.burst.map(|burst| coordinates(burst.centre)),
                sound_limit: turn.burst.map(|burst| burst.sound_limit),
                start_rotation: turn.start_rotation.is_some(),
                ..OverlayOutput::default()
            },
        )
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RotationVector {
    pub id: String,
    pub source: Source,
    pub entries: Vec<String>,
    track: Vec<TiltRow>,
    #[serde(default)]
    start: bool,
    before: RotationRow,
    after: RotationRow,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TiltRow {
    hold: u16,
    cross: i16,
    diagonal: i16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RotationRow {
    active: bool,
    next: usize,
    forward: bool,
    hold: u16,
    matrix: [i16; 4],
}

impl RotationRow {
    fn state(&self) -> ShaftRotation {
        ShaftRotation {
            active: self.active,
            next: self.next,
            direction: if self.forward {
                TiltDirection::Forward
            } else {
                TiltDirection::Backward
            },
            hold: self.hold,
            view: ShaftView {
                pivot: point([123, 231]),
                matrix: self.matrix,
            },
        }
    }
}

impl RotationVector {
    pub fn check(&self) {
        let samples: Vec<_> = self
            .track
            .iter()
            .map(|row| TiltSample {
                hold: row.hold,
                cross: row.cross,
                diagonal: row.diagonal,
            })
            .collect();
        let track = TiltTrack::new(&samples).unwrap();
        let saved = self.before.state();
        let mut state = restore(&saved);
        assert_eq!(state, saved);
        let view = turn_shaft_rotation(&mut state, self.start.then_some(StartShaftRotation), track);
        let expected = self.after.state();
        assert_eq!(state, expected, "{}: shaft rotation", self.id);
        assert_eq!(view, expected.view, "{}: published view", self.id);
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReachVector {
    pub id: String,
    pub source: Source,
    pub entries: Vec<String>,
    input: ReachInput,
    arrival: Option<ArrivalRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReachInput {
    active: bool,
    ordinary_play: bool,
    whole_centre: [u16; 2],
    speed: i32,
    facing: Facing,
    earlier_pose: EarlierPoseRow,
}

#[derive(Deserialize, Copy, Clone)]
enum EarlierPoseRow {
    Other,
    StandingLeft,
    StandingRight,
}

#[derive(Deserialize, Copy, Clone)]
enum StandingRow {
    StandingLeft,
    StandingRight,
}

#[derive(Deserialize, Copy, Clone)]
enum AnimationRow {
    Preserve,
    StartStanding,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArrivalRow {
    standing_pose: StandingRow,
    animation: AnimationRow,
    hold: u8,
    scene_fade: [u8; 2],
}

impl ArrivalRow {
    fn lock(&self) -> ElevatorLock {
        ElevatorLock {
            standing: match self.standing_pose {
                StandingRow::StandingLeft => Side::Left,
                StandingRow::StandingRight => Side::Right,
            },
            animation: match self.animation {
                AnimationRow::Preserve => ElevatorAnimation::Preserve,
                AnimationRow::StartStanding => ElevatorAnimation::StartStanding,
            },
            hold: self.hold,
            scene_fade: self.scene_fade,
        }
    }
}

impl ReachVector {
    pub fn check(&self) {
        let placement = ShaftPlacement {
            centre: point(self.input.whole_centre),
            vertical_speed: VerticalSpeed(Wrapping(self.input.speed.cast_unsigned())),
            facing: self.input.facing,
            earlier_pose: match self.input.earlier_pose {
                EarlierPoseRow::Other => EarlierShaftPose::Other,
                EarlierPoseRow::StandingLeft => EarlierShaftPose::Standing(Side::Left),
                EarlierPoseRow::StandingRight => EarlierShaftPose::Standing(Side::Right),
            },
        };
        let actual = elevator_reached(
            self.input.active,
            self.input.ordinary_play.then_some(placement),
        );
        assert_eq!(
            actual,
            self.arrival.as_ref().map(ArrivalRow::lock),
            "{}: elevator lock",
            self.id
        );
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionVector {
    pub id: String,
    pub source: Source,
    pub entries: Vec<String>,
    before: [i32; 2],
    input: ProjectionInput,
    after: [i32; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectionInput {
    ran: bool,
    anchor: [u16; 2],
    pivot: [u16; 2],
    diagonal: i16,
    cross: i16,
}

impl ProjectionVector {
    pub fn check(&self) {
        let anchor = point(self.input.anchor);
        let mut vent = MountedVent::new(Vent::place(VentFacing::Left, anchor, 0));
        assert_eq!(vent.offset, VentDrawOffset::ZERO);
        let [x, y] = self.before;
        vent.offset = restore(&VentDrawOffset(Pair { x, y }));
        let ProjectionInput {
            diagonal,
            cross,
            pivot,
            ran,
            ..
        } = self.input;
        let view = ShaftView {
            pivot: point(pivot),
            matrix: [diagonal, cross, cross.wrapping_neg(), diagonal],
        };
        let camera = if ran { anchor } else { point([1000, 1000]) };
        vent.run(camera, view, None, &Rules::default());
        let Pair { x, y } = vent.offset.0;
        assert_eq!([x, y], self.after, "{}: projected offset", self.id);
        assert_eq!(vent.vent.position, anchor, "{}: contact anchor", self.id);
        assert_eq!(
            matches!(
                vent.vent.phase,
                boss_ceres_ridley::VentPhase::AwaitingFirstPuff(_)
            ),
            ran
        );
    }
}
