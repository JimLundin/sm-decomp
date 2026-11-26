//! Restore the clean shaft state into the carried room and replay the same native inputs.

use super::Run;
#[cfg(test)]
use super::{appearance, connected, movement, open_rows, polled, read_lock, seed_controller};
#[cfg(test)]
use crate::replay::result::Outcome;
use boss_ceres_ridley::{MountedVent, ShaftOverlay, ShaftRotation, TiltTrack, VentDrawOffset};
use canonical::model::{Model, Pinned};
use canonical::{Canonical, from_bytes, to_bytes};
#[cfg(test)]
use game::production::played::Outputs;
use game::production::played::{Content, shaft};
use serde::{Deserialize, Serialize};

#[cfg(test)]
const RETURN: &str = "chain-any-0006-E0B5";
#[cfg(test)]
const HALL: &str = "chain-any-0010-DF8D";
#[cfg(test)]
const SHAFT: &str = "chain-any-0011-DF45";
#[cfg(test)]
const HALL_START: u64 = 2631;
#[cfg(test)]
const SHAFT_START: u64 = 2910;

#[derive(Serialize, Deserialize)]
struct ShaftOwner {
    overlay: ShaftOverlay,
    rotation: ShaftRotation,
    offsets: Vec<VentDrawOffset>,
    locked: bool,
}

struct ShaftOwnerPin;

impl Pinned for ShaftOwnerPin {
    type State = ShaftOwner;
    const NAME: &'static str = "shaft actors witness";
    const SHAPE: u64 = 0xbd95_220a_f496_51f0;
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Refused {
    Content,
    Cursor,
    Encoding,
}

pub(super) struct Checkpoint {
    art: data::assets::shaft::Shaft,
    entry: Vec<u8>,
    clock_picture: Vec<u8>,
    actors: Option<Vec<u8>>,
}

impl Checkpoint {
    pub(super) fn capture(run: &Run<'_>) -> Result<Self, Refused> {
        let actors = run.state.station.escape_shaft.as_ref().map(|shaft| {
            to_bytes(&Model::<ShaftOwnerPin>(ShaftOwner {
                overlay: shaft.overlay,
                rotation: shaft.rotation,
                offsets: shaft.vents.iter().map(|vent| vent.offset).collect(),
                locked: shaft.locked,
            }))
        });
        Ok(Self {
            art: run
                .played
                .shaft_overlay_art
                .clone()
                .ok_or(Refused::Content)?,
            entry: to_bytes(&run.state.passage.room.entry),
            clock_picture: to_bytes(&run.state.clock_picture),
            actors,
        })
    }

    fn check_content(&self, content: &Content) -> Result<(), Refused> {
        let track = content.shaft_track.as_ref().ok_or(Refused::Content)?;
        if content.shaft_overlay_art.as_ref() != Some(&self.art)
            || *track != shaft::track(&self.art)
            || TiltTrack::new(track).is_none()
        {
            return Err(Refused::Content);
        }
        Ok(())
    }

    pub(super) fn restore<'a>(&self, live: &Run<'a>) -> Result<Run<'a>, Refused> {
        self.check_content(&live.played)?;
        // Retain the room envelope, including the suspended passage, physical video and
        // ordinary vent phases. The shaft fields and encoded pictures come from the bytes.
        let mut restored = live.clone();
        restored.state.passage.room.entry = decode(&self.entry)?;
        restored.state.clock_picture = decode(&self.clock_picture)?;
        restored.state.station.escape_shaft =
            match (&self.actors, restored.state.station.escape_shaft.take()) {
                (Some(bytes), Some(carried)) => {
                    let Model(owner) = decode::<Model<ShaftOwnerPin>>(bytes)?;
                    if self.art.rotation.get(owner.rotation.next).is_none() {
                        return Err(Refused::Cursor);
                    }
                    if owner.offsets.len() != carried.vents.len() {
                        return Err(Refused::Encoding);
                    }
                    Some(shaft::EscapeShaft {
                        overlay: owner.overlay,
                        rotation: owner.rotation,
                        locked: owner.locked,
                        vents: carried
                            .vents
                            .into_iter()
                            .zip(owner.offsets)
                            .map(|(mounted, offset)| MountedVent {
                                vent: mounted.vent,
                                offset,
                            })
                            .collect(),
                    })
                }
                (None, None) => None,
                (Some(_), None) | (None, Some(_)) => return Err(Refused::Encoding),
            };
        Ok(restored)
    }
}

fn decode<T: Canonical>(bytes: &[u8]) -> Result<T, Refused> {
    from_bytes(bytes).map_err(|_| Refused::Encoding)
}

#[test]
#[ignore = "pinned record cache and player ROM"]
fn shaft_actor_checkpoints_continue_from_the_live_return() -> Result<(), Outcome> {
    let sources = super::tests::sources();
    let lock = read_lock(&sources)?;
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom)?,
        appearance: appearance::Content::load(&sources.rom)?,
    };
    let mut rows = open_rows(&sources, &lock, RETURN)?;
    assert_eq!(
        canonical::model::trace_shape::<ShaftOwnerPin>().unwrap(),
        ShaftOwnerPin::SHAPE
    );
    let start = 1690_u64;
    let mut previous = rows.record(start)?;
    let mut live = Run::admit(&previous, &content, &sources.rom, RETURN)?;
    let mut controller = seed_controller(&previous)?;
    let mut restored: Vec<((&str, u64), Run<'_>)> = Vec::new();
    let mut placement = Vec::new();
    let mut reach = Vec::new();
    let checkpoints = [
        (HALL, 222),
        (HALL, 224),
        (HALL, 225),
        (HALL, 234),
        (HALL, 235),
        (HALL, 238),
        (HALL, 239),
        (HALL, 240),
        (HALL, 241),
        (HALL, 278),
        (SHAFT, 0),
        (SHAFT, 33),
        (SHAFT, 34),
        (SHAFT, 93),
        (SHAFT, 94),
        (SHAFT, 96),
        (SHAFT, 162),
        (SHAFT, 163),
        (SHAFT, 164),
    ];

    let mut suffix_cuts = 0_usize;
    for cut in start.saturating_add(1)..=SHAFT_START + 165 {
        let current = rows.record(cut)?;
        assert_eq!(current.video_frame, previous.video_frame.wrapping_add(1));
        let (row, local) = rows.place(cut)?;
        let place = (row.as_str(), local);
        let (fields, outputs) =
            live.step_with_outputs(controller, &previous, &current, cut, place)?;
        check_work_edge(place, &current, &outputs);
        suffix_cuts = suffix_cuts.saturating_add(restored.len());
        for ((origin, origin_cut), run) in &mut restored {
            let (replayed, effects) =
                run.step_with_outputs(controller, &previous, &current, cut, place)?;
            assert!(
                replayed == fields,
                "{origin}/{origin_cut} ordered observations at {row}/{local}"
            );
            assert!(
                effects == outputs,
                "{origin}/{origin_cut} production outputs at {row}/{local}"
            );
            assert!(
                run.state == live.state,
                "{origin}/{origin_cut} production State at {row}/{local}"
            );
            assert!(
                run.return_timing == live.return_timing
                    && run.warning_picture == live.warning_picture,
                "{origin}/{origin_cut} retained runner at {row}/{local}"
            );
        }

        if outputs.shaft_overlay_placement.is_some() {
            placement.push(cut);
        }
        if outputs.elevator_reach.is_some() {
            reach.push(cut);
        }
        if place == (SHAFT, 163) {
            check_admission(&live);
        }
        if let Some(&checkpoint) = checkpoints.iter().find(|&&checkpoint| checkpoint == place) {
            restored.push((
                checkpoint,
                Checkpoint::capture(&live).unwrap().restore(&live).unwrap(),
            ));
        }
        controller = controller.read(current.pad.0, polled(&current));
        previous = current;
    }

    assert_eq!(placement, [HALL_START + 225]);
    assert_eq!(reach, [SHAFT_START + 164]);
    assert_eq!(restored.len(), checkpoints.len());
    super::tests::diagnostic(format_args!(
        "SHAFT RESTORE {} checkpoints, {suffix_cuts} matching suffix cuts through shaft/165, placement hall/225, reach shaft/164",
        restored.len(),
    ));
    Ok(())
}

#[cfg(test)]
fn check_admission(live: &Run<'_>) {
    let mut saved = Checkpoint::capture(live).unwrap();
    let mut mismatched = live.clone();
    mismatched.played.shaft_track.as_mut().unwrap().reverse();
    assert!(matches!(saved.restore(&mismatched), Err(Refused::Content)));
    mismatched.played.shaft_track = live.played.shaft_track;
    let art = mismatched.played.shaft_overlay_art.as_mut().unwrap();
    *art.overlay.first_mut().unwrap() ^= 1;
    assert!(matches!(saved.restore(&mismatched), Err(Refused::Content)));

    let Model(mut owner) =
        decode::<Model<ShaftOwnerPin>>(saved.actors.as_deref().unwrap()).unwrap();
    owner.rotation.next = saved.art.rotation.len();
    saved.actors = Some(to_bytes(&Model::<ShaftOwnerPin>(owner)));
    assert!(matches!(saved.restore(live), Err(Refused::Cursor)));
}

#[cfg(test)]
fn check_work_edge(place: (&str, u64), record: &oracle_record::FrameRecord, outputs: &Outputs) {
    match place {
        (HALL, 239) => {
            assert!(!super::finished(record));
            assert!(
                outputs
                    .shaft_actors
                    .is_some_and(|turn| turn.figure.is_some())
            );
        }
        (HALL, 240) => {
            assert!(super::finished(record));
            assert!(outputs.shaft_actors.is_none());
        }
        (HALL, 241) => {
            assert!(
                outputs
                    .shaft_actors
                    .is_some_and(|turn| turn.burst.is_some())
            );
        }
        _ => {}
    }
}
