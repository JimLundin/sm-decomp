//! Imported standing art and a small video-owner harness for the interrupted cut history.

use std::collections::BTreeMap;

use enum_map::{EnumMap, enum_map};
use samus_appearance::{
    ArtDisplacement, BodyHalf, BodyObject, BodyPlacement, BodyRow, ElevatorArt, ElevatorDrawing,
    ElevatorPictureError, ElevatorProjection, ObjectSize, ObjectStyle, ScreenPoint, ShaftView,
    StandingLook, StandingPoseArt, StandingStepArt, Step, elevator_standing_picture,
};
use serde::Deserialize;
use types::{Action, Aim, Body, Facing, Gun, Legs, Pair, Pose, Side};

use super::{behaviour, read_file};

#[derive(Deserialize)]
pub(super) struct ElevatorVector {
    id: String,
    source: String,
    pub(super) entries: Vec<String>,
    #[serde(flatten)]
    case: Case,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Case {
    Drawing(DrawingCase),
    OutsidePose(OutsidePoseCase),
    MissingStep(MissingStepCase),
    Interrupted(InterruptedCase),
    Recorded(RecordedCut),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DrawingCase {
    look: (Facing, u8),
    centre: [i32; 2],
    pivot: [i32; 2],
    camera: [i32; 2],
    view: [i16; 2],
    vertical_displacement: i32,
    upper: Vec<Piece>,
    lower: Vec<Piece>,
    upper_rows: Vec<String>,
    lower_rows: Vec<String>,
    origin: [i32; 2],
    picture: Vec<Piece>,
    rows: Vec<String>,
}

#[derive(Clone, Copy, Deserialize)]
enum Size {
    Small,
    Large,
}

#[derive(Clone, Deserialize)]
struct Piece(i32, i32, u16, u8, Size, u8, bool, bool);

impl Piece {
    fn object<P>(&self, position: P) -> BodyObject<P> {
        BodyObject {
            position,
            style: ObjectStyle {
                tile: self.2,
                palette: self.3,
                size: match self.4 {
                    Size::Small => ObjectSize::Small,
                    Size::Large => ObjectSize::Large,
                },
                priority: self.5,
                flips: Pair {
                    x: self.6,
                    y: self.7,
                },
            },
        }
    }

    fn placement(&self) -> BodyPlacement {
        self.object(ScreenPoint(pair([self.0, self.1].map(i64::from))))
    }
}

fn pair<T>([x, y]: [T; 2]) -> Pair<T> {
    Pair { x, y }
}

fn standing_pose(facing: Facing) -> Pose {
    Pose {
        body: Body::Upright,
        action: Action::Still,
        facing,
        aim: Aim::Forward,
        gun: Gun::Stowed,
        legs: Legs::Together,
    }
}

fn standing_side(facing: Facing) -> Side {
    match facing {
        Facing::Left => Side::Left,
        Facing::Right => Side::Right,
        Facing::Screen => panic!("standing art must specify a side"),
    }
}

const IDENTITY_PROJECTION: ElevatorProjection = ElevatorProjection {
    centre: Pair { x: 0, y: 0 },
    pivot: Pair { x: 0, y: 0 },
    camera_origin: Pair { x: 0, y: 0 },
    carried_view: ShaftView {
        diagonal: 256,
        cross: 0,
    },
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OutsidePoseCase {
    declined_poses: Vec<(Body, Action, Facing, Aim, Gun, Legs)>,
}

impl OutsidePoseCase {
    fn check(&self, id: &str, art: &ElevatorArt<&str>) {
        for &(body, action, facing, aim, gun, legs) in &self.declined_poses {
            let pose = Pose {
                body,
                action,
                facing,
                aim,
                gun,
                legs,
            };
            let drawing = ElevatorDrawing::Locked {
                pose,
                step: Step(0),
                projection: IDENTITY_PROJECTION,
            };
            assert_eq!(
                elevator_standing_picture(drawing, art),
                Err(ElevatorPictureError::UnsupportedPose),
                "{id}: {pose:?}"
            );
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MissingStepCase {
    missing_looks: Vec<(Facing, u8)>,
}

impl MissingStepCase {
    fn check(&self, id: &str, art: &ElevatorArt<&str>) {
        for &(facing, step) in &self.missing_looks {
            let drawing = standing_drawing((facing, step), IDENTITY_PROJECTION);
            let look = StandingLook {
                side: standing_side(facing),
                step: Step(step),
            };
            assert_eq!(
                elevator_standing_picture(drawing, art),
                Err(ElevatorPictureError::MissingArt(look)),
                "{id}: {look:?}"
            );
        }
    }
}

fn half<'a>(pieces: &[Piece], rows: &'a [String]) -> BodyHalf<&'a str> {
    let (first, second) = match rows {
        [first] => (first.as_str(), None),
        [first, second] => (first.as_str(), Some(second.as_str())),
        _ => panic!("expected one or two strips for a body half"),
    };
    BodyHalf {
        pieces: pieces
            .iter()
            .map(|piece| piece.object(pair([piece.0, piece.1])))
            .collect(),
        first_strip: first,
        second_strip: second,
    }
}

fn standing_drawing(
    (facing, step): (Facing, u8),
    projection: ElevatorProjection,
) -> ElevatorDrawing {
    ElevatorDrawing::Locked {
        pose: standing_pose(facing),
        step: Step(step),
        projection,
    }
}

fn art<'a>(cases: impl Iterator<Item = &'a DrawingCase>) -> ElevatorArt<&'a str> {
    let empty = StandingPoseArt {
        displacement: ArtDisplacement(0),
        steps: BTreeMap::new(),
    };
    let mut art = ElevatorArt {
        left: empty.clone(),
        right: empty,
    };
    for case in cases {
        let (facing, step) = case.look;
        let pose = match standing_side(facing) {
            Side::Left => &mut art.left,
            Side::Right => &mut art.right,
        };
        pose.displacement = ArtDisplacement(case.vertical_displacement);
        assert!(
            pose.steps
                .insert(
                    Step(step),
                    StandingStepArt {
                        upper: half(&case.upper, &case.upper_rows),
                        lower: half(&case.lower, &case.lower_rows),
                    }
                )
                .is_none()
        );
    }
    art
}

impl DrawingCase {
    fn check(&self, id: &str, art: &ElevatorArt<&str>) {
        let [diagonal, cross] = self.view;
        let projection = ElevatorProjection {
            centre: pair(self.centre),
            pivot: pair(self.pivot),
            camera_origin: pair(self.camera),
            carried_view: ShaftView { diagonal, cross },
        };
        assert_eq!(
            projection.origin(ArtDisplacement(self.vertical_displacement)),
            ScreenPoint(pair(self.origin.map(i64::from))),
            "{id}: origin"
        );
        let output = elevator_standing_picture(standing_drawing(self.look, projection), art)
            .expect(id)
            .expect("new body work");
        assert_eq!(output.placements, picture(&self.picture), "{id}: picture");
        assert_eq!(
            output
                .strips
                .into_values()
                .flatten()
                .copied()
                .collect::<Vec<_>>(),
            self.rows,
            "{id}: rows"
        );
    }
}

fn picture(pieces: &[Piece]) -> Vec<BodyPlacement> {
    pieces.iter().map(Piece::placement).collect()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rows {
    #[serde(rename = "UpperFirst")]
    upper_first: Option<String>,
    #[serde(rename = "UpperSecond")]
    upper_second: Option<String>,
    #[serde(rename = "LowerFirst")]
    lower_first: Option<String>,
    #[serde(rename = "LowerSecond")]
    lower_second: Option<String>,
}

impl Rows {
    fn upper(&self) -> Vec<String> {
        self.upper_first
            .iter()
            .chain(&self.upper_second)
            .cloned()
            .collect()
    }

    fn lower(&self) -> Vec<String> {
        self.lower_first
            .iter()
            .chain(&self.lower_second)
            .cloned()
            .collect()
    }
}

fn by_row(rows: &Rows) -> EnumMap<BodyRow, Option<&str>> {
    enum_map! {
        BodyRow::UpperFirst => rows.upper_first.as_deref(),
        BodyRow::UpperSecond => rows.upper_second.as_deref(),
        BodyRow::LowerFirst => rows.lower_first.as_deref(),
        BodyRow::LowerSecond => rows.lower_second.as_deref(),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InterruptedCase {
    live_picture: Vec<Piece>,
    live_rows: Rows,
    drawing: Vec<Piece>,
    strips: Rows,
    cut: Vec<Cut>,
}

#[derive(Clone, Copy, Deserialize)]
enum Opportunity {
    Prepare,
    Suspend,
    Publish,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cut {
    opportunity: Opportunity,
    submitted: bool,
    pending: bool,
    live_picture: Vec<Piece>,
    live_rows: Rows,
}

impl InterruptedCase {
    fn check(&self, id: &str) {
        // This history supplies already projected input art. Use an identity view and split
        // those two pieces into their respective halves to submit them through appearance.
        let (upper, lower) = self.drawing.split_at(1);
        let step = 0;
        let upper_rows = self.strips.upper();
        let lower_rows = self.strips.lower();
        let pose = StandingPoseArt {
            displacement: ArtDisplacement(0),
            steps: BTreeMap::from([(
                Step(step),
                StandingStepArt {
                    upper: half(upper, &upper_rows),
                    lower: half(lower, &lower_rows),
                },
            )]),
        };
        self.check_cuts(
            id,
            standing_drawing((Facing::Left, step), IDENTITY_PROJECTION),
            &ElevatorArt {
                left: pose.clone(),
                right: pose,
            },
        );
    }

    fn check_cuts(&self, id: &str, draw: ElevatorDrawing, art: &ElevatorArt<&str>) {
        let mut pending = None;
        let mut live_picture = picture(&self.live_picture);
        let mut live_rows = by_row(&self.live_rows);
        for (index, cut) in self.cut.iter().enumerate() {
            let opportunity = match cut.opportunity {
                Opportunity::Prepare => draw,
                Opportunity::Suspend | Opportunity::Publish => ElevatorDrawing::NoNewDrawing,
            };
            let submission = elevator_standing_picture(opportunity, art)
                .unwrap_or_else(|error| panic!("{id} cut {index}: {error:?}"));
            assert_eq!(
                submission.is_some(),
                cut.submitted,
                "{id} cut {index}: submitted"
            );
            if submission.is_some() {
                pending = submission;
            }
            if matches!(cut.opportunity, Opportunity::Publish) {
                let work = pending.take().expect("publication of prepared room work");
                live_picture = work.placements;
                for (row, label) in work
                    .strips
                    .into_iter()
                    .filter_map(|(row, strip)| strip.copied().map(|label| (row, label)))
                {
                    live_rows[row] = Some(label);
                }
            }
            assert_eq!(pending.is_some(), cut.pending, "{id} cut {index}: pending");
            assert_eq!(
                live_picture,
                picture(&cut.live_picture),
                "{id} cut {index}: live picture"
            );
            assert_eq!(
                live_rows,
                by_row(&cut.live_rows),
                "{id} cut {index}: live rows"
            );
        }
    }
}

/// Native observations omit the projection inputs and imported art, so they cannot replay
/// a fresh drawing. The suspended observation can still check that no new work is submitted.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedCut {
    #[serde(default)]
    suspended: bool,
    #[serde(rename = "look")]
    _look: (Facing, u8),
    #[serde(rename = "centre")]
    _centre: [i32; 2],
    #[serde(rename = "origin")]
    _origin: [i32; 2],
    #[serde(rename = "piece_counts")]
    _piece_counts: [usize; 2],
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "report executable and observation-only elevator vectors"
)]
fn elevator_body_vectors() {
    let (file, _) = read_file();
    let art = art(file
        .elevator_body
        .iter()
        .filter_map(|vector| match &vector.case {
            Case::Drawing(case) => Some(case),
            Case::OutsidePose(_)
            | Case::MissingStep(_)
            | Case::Interrupted(_)
            | Case::Recorded(_) => None,
        }));
    let mut checked = 0;
    let mut observation_only = 0;
    for vector in &file.elevator_body {
        let id = format!("{} ({})", vector.id, vector.source);
        for entry in &vector.entries {
            behaviour(entry);
        }
        match &vector.case {
            Case::Drawing(case) => case.check(&id, &art),
            Case::OutsidePose(case) => case.check(&id, &art),
            Case::MissingStep(case) => case.check(&id, &art),
            Case::Interrupted(case) => case.check(&id),
            Case::Recorded(case) if case.suspended => {
                assert_eq!(
                    elevator_standing_picture(ElevatorDrawing::NoNewDrawing, &art),
                    Ok(None),
                    "{id}: no submission"
                );
            }
            Case::Recorded(_) => {
                observation_only += 1;
                continue;
            }
        }
        checked += 1;
    }
    println!("{checked} elevator vectors checked ({observation_only} observation-only)");
}

#[test]
fn elevator_reader_rejects_unknown_keys() {
    let recorded = "id='test'\nsource='cut'\nentries=[]\nlook=['Left',0]\ncentre=[0,0]\norigin=[0,0]\npiece_counts=[4,3]";
    assert!(toml::from_str::<ElevatorVector>(recorded).is_ok());
    assert!(toml::from_str::<ElevatorVector>(&format!("{recorded}\nunknown=1")).is_err());
    assert!(toml::from_str::<Rows>("UpperFirst='Strip'\nUnknown='Strip'").is_err());
}
