//! Appearance requests at actual opportunities, separate from audio sending/replies.
use super::{Clips, figure_of};
use crate::state::{State, samus::Samus};
use canonical::Canonical;
use samus_appearance::{
    CinematicActivity, GraphicalContact, Opportunity, OpportunityOutput, SoundRequest,
};
use types::{Action, AnimationSound, Set};

/// Neighbouring writers excluded from the ordinary appearance producer path.
#[derive(Clone, Copy, Debug, enum_map::Enum)]
pub enum ExcludedWriter {
    /// Cinematic suppression.
    Cinematic,
    /// Boss suppression.
    Boss,
    /// Special colour suppression.
    SpecialColours,
    /// Charge drawing.
    Charge,
    /// Retained charging resume.
    ChargingResume,
    /// Echo drawing.
    Echoes,
    /// Debug drawing.
    Debug,
    /// Time-up drawing.
    TimeUp,
    /// A liquid surface is present.
    Liquid,
}
/// Independently decoded seed exclusions for the bounded ordinary Ceres path.
#[derive(Clone, Copy, Debug, Default)]
pub struct SeedFacts {
    /// Actual active neighbouring writers, never inferred from output silence.
    pub active: Set<ExcludedWriter>,
}

/// A sealed absence certificate; ordinary owners cannot create these excluded writers.
/// The admitted Calm Ceres route has no escape countdown or Zebes time-up event writer.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capability {
    checked: Checked,
}
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
struct Checked;
impl Capability {
    /// Reject every active excluded seed writer.
    #[must_use]
    pub fn from_seed(facts: SeedFacts) -> Option<Self> {
        (!facts.active.values().any(|&active| active)).then_some(Self { checked: Checked })
    }
    /// Restored State must still satisfy the owning path's bounded quantities.
    #[must_use]
    pub fn holds(self, state: &State) -> bool {
        state.room.area == data::room::Area::Ceres
            && state.ceres == crate::state::room::CeresStage::Calm
            && state.samus.equipment.equipped == crate::state::samus::Items::default()
            && state.samus.drawing == crate::state::samus::DrawingHandler::Default
            && state.samus.animation.0.0.stage != types::BoostStage::Boosting
    }
}

/// Each distinct source request survives, in source order, including animation requests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// `WallJumpBranch` or `BoostingLap`.
    Animation(AnimationSound),
    /// Contact, selection, drawing or first passage.
    Appearance(SoundRequest),
}
/// Transient producer outputs; never an audio queue, retry or acknowledgement.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outputs {
    /// Ordered semantic requests for the later sound owner.
    pub requests: Vec<Request>,
    /// Graphical contacts for part 2, independent of sound suppression.
    pub contacts: Vec<GraphicalContact>,
}
impl Outputs {
    /// Preserve both independent outputs of an opportunity.
    pub fn retain(&mut self, output: OpportunityOutput) {
        self.requests.extend(output.sound.map(Request::Appearance));
        self.contacts.extend(output.contact);
    }
    /// Preserve an animation request at its own phase.
    pub fn animation(&mut self, sound: Option<AnimationSound>) {
        self.requests.extend(sound.map(Request::Animation));
    }
}
/// Running movement opportunity, before pose selection and animation update.
pub fn contact(samus: &Samus, clips: &Clips, _: Capability, out: &mut Outputs) {
    if samus.movement != data::pose::MovementType::Running {
        return;
    }
    let Some(imported) = clips.of(samus.pose) else {
        return;
    };
    out.retain(
        Opportunity::Contact {
            figure: imported.figure,
            playhead: samus.animation.0.0.playhead,
            stage: samus.animation.0.0.stage,
            clip: &imported.clip,
            suppressors: Set::default(),
        }
        .read(),
    );
}
/// An actual selection, including sprite changes, with certified inactive neighbours.
pub fn selected(
    from: crate::state::samus::PoseId,
    samus: &Samus,
    _: Capability,
    out: &mut Outputs,
) {
    let Some((from, selected)) = figure_of(from).zip(figure_of(samus.pose)) else {
        return;
    };
    out.retain(
        Opportunity::PoseSelected {
            from,
            selected,
            items: crate::run_progress_bridge::switched_on(&samus.equipment),
            submerged: false,
            cinematic: CinematicActivity::Inactive,
        }
        .read(),
    );
}
/// Normal drawing reads remembered movement, even when animation has frozen.
pub fn normal_draw(samus: &Samus, _: Capability, out: &mut Outputs) {
    let previous = sound_lifetime_action(samus.preparation_movement);
    let current = sound_lifetime_action(samus.movement);
    out.retain(Opportunity::NormalDraw { previous, current }.read());
}
/// Only spin/wall lifetime is observed by end/entry requests. Retained movement, rather
/// than the pose's nominal action, supplies this projection.
const fn sound_lifetime_action(movement: data::pose::MovementType) -> Action {
    match movement {
        data::pose::MovementType::SpinJumping => Action::Spinning,
        data::pose::MovementType::WallJumping => Action::WallJumping,
        data::pose::MovementType::Standing
        | data::pose::MovementType::Running
        | data::pose::MovementType::NormalJumping
        | data::pose::MovementType::MorphBallOnGround
        | data::pose::MovementType::Crouching
        | data::pose::MovementType::Falling
        | data::pose::MovementType::Unused07
        | data::pose::MovementType::MorphBallFalling
        | data::pose::MovementType::Unused09
        | data::pose::MovementType::KnockbackOrCrystalFlashEnding
        | data::pose::MovementType::Unused0B
        | data::pose::MovementType::Unused0C
        | data::pose::MovementType::Unused0D
        | data::pose::MovementType::TurningAroundOnGround
        | data::pose::MovementType::CrouchingTransition
        | data::pose::MovementType::Moonwalking
        | data::pose::MovementType::SpringBallOnGround
        | data::pose::MovementType::SpringBallInAir
        | data::pose::MovementType::SpringBallFalling
        | data::pose::MovementType::RanIntoWall
        | data::pose::MovementType::Grappling
        | data::pose::MovementType::TurningAroundJumping
        | data::pose::MovementType::TurningAroundFalling
        | data::pose::MovementType::DamageBoost
        | data::pose::MovementType::GrabbedByDraygon
        | data::pose::MovementType::Shinespark => Action::Still,
    }
}
/// R1 must invoke this once before holding sounds; no dispatch exists until its runtime lands.
pub fn passage_entry(
    current: Action,
    shoot_held: bool,
    charge: samus_appearance::Charge,
    out: &mut Outputs,
) {
    out.retain(
        Opportunity::PassageEntry {
            current,
            shoot_held,
            charge,
        }
        .read(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_excluded_writer_refuses_admission() {
        let inactive = SeedFacts::default();
        assert!(Capability::from_seed(inactive).is_some());
        let writers = [
            ExcludedWriter::Cinematic,
            ExcludedWriter::Boss,
            ExcludedWriter::SpecialColours,
            ExcludedWriter::Charge,
            ExcludedWriter::ChargingResume,
            ExcludedWriter::Echoes,
            ExcludedWriter::Debug,
            ExcludedWriter::TimeUp,
            ExcludedWriter::Liquid,
        ];
        for writer in writers {
            let mut facts = inactive;
            facts.active[writer] = true;
            assert!(Capability::from_seed(facts).is_none());
        }
    }
    struct Content {
        clip: samus_appearance::Clip,
        paces: samus_appearance::Paces,
    }
    impl samus_appearance::Content for Content {
        fn clip(&self, _: &samus_appearance::Figure) -> &samus_appearance::Clip {
            &self.clip
        }
        fn paces(&self) -> &samus_appearance::Paces {
            &self.paces
        }
    }
    #[test]
    fn contact_update_selection_and_draw_form_one_ordered_work() {
        use samus_appearance::{
            Clip, Control, Dash, Frames, Hold, Inputs, Instruction, LapsLeft, Liquid, Pace, Paces,
            Playhead, Rules, SoundsWaiting, Step,
        };
        let hold = Hold::new(7).unwrap();
        let content = Content {
            clip: Clip {
                instructions: vec![
                    Instruction::Show(hold),
                    Instruction::Control(Control::Again(Step::FIRST)),
                ],
                kick: None,
                fast_fall: None,
                rising: None,
                contacts: std::iter::once(Step::FIRST).collect(),
            },
            paces: Paces {
                plain: Pace(vec![hold]),
                boosted: enum_map::EnumMap::from_fn(|_| Pace(vec![hold])),
            },
        };
        let running = figure_of(crate::state::samus::PoseId(9)).unwrap();
        assert_eq!(running.pose.action, Action::Running);
        let spinning = figure_of(crate::state::samus::PoseId(0x19)).unwrap();
        let mut samus = Samus {
            pose: crate::state::samus::PoseId(0x19),
            movement: data::pose::MovementType::SpinJumping,
            ..Samus::default()
        };
        samus.animation =
            crate::state::samus::Animation::restore(Step::FIRST, 1, types::BoostStage::Stage0, 0);
        let before = samus.animation.0.0;
        let mut out = Outputs::default();
        out.retain(
            Opportunity::Contact {
                figure: running,
                playhead: before.playhead,
                stage: before.stage,
                clip: &content.clip,
                suppressors: Set::default(),
            }
            .read(),
        );
        let inputs = Inputs {
            figure: running,
            figure_after: spinning,
            moves: Set::default(),
            dash: Dash::Off,
            run_held: false,
            heading_down: false,
            vertical_speed: types::ZERO,
            energy: 99,
            items: Set::default(),
            liquid: Liquid::NONE,
            sounds_waiting: SoundsWaiting::NONE,
        };
        let (after, sound) = samus_appearance::update(before, &inputs, &content, &Rules::VANILLA);
        samus.animation.0.0 = after;
        out.animation(sound);
        let capability = Capability::from_seed(SeedFacts::default()).unwrap();
        selected(crate::state::samus::PoseId(9), &samus, capability, &mut out);
        normal_draw(&samus, capability, &mut out);
        assert_eq!(
            out.requests,
            [
                Request::Appearance(SoundRequest::Footstep),
                Request::Appearance(SoundRequest::PlainSpin)
            ]
        );
        assert_eq!(
            out.contacts,
            [GraphicalContact {
                figure: running,
                playhead: before.playhead
            }]
        );
        assert_eq!(
            samus.animation.0.0,
            samus_appearance::State {
                playhead: Playhead {
                    step: Step::FIRST,
                    time_left: Frames(7)
                },
                stage: types::BoostStage::Stage0,
                laps_left: LapsLeft::WithoutBooster
            }
        );
    }
    #[test]
    fn animation_and_drawing_requests_survive_aggregation() {
        use samus_appearance::{
            Branch, Clip, Control, Dash, Hold, Inputs, Instruction, Liquid, Pace, Paces, Rules,
            SoundsWaiting, Step,
        };
        let mut out = Outputs::default();
        let hold = Hold::new(7).unwrap();
        let content = Content {
            clip: Clip {
                instructions: vec![
                    Instruction::Show(hold),
                    Instruction::Control(Control::Branch(Branch {
                        spin: Step::FIRST,
                        space_jump: Step::FIRST,
                        screw_attack: Step::FIRST,
                    })),
                ],
                kick: None,
                fast_fall: None,
                rising: None,
                contacts: std::collections::BTreeSet::default(),
            },
            paces: Paces {
                plain: Pace(vec![hold]),
                boosted: enum_map::EnumMap::from_fn(|_| Pace(vec![hold])),
            },
        };
        let figure = figure_of(crate::state::samus::PoseId(0x19)).unwrap();
        let inputs = Inputs {
            figure,
            figure_after: figure,
            moves: Set::default(),
            dash: Dash::Off,
            run_held: false,
            heading_down: false,
            vertical_speed: types::ZERO,
            energy: 99,
            items: Set::default(),
            liquid: Liquid::NONE,
            sounds_waiting: SoundsWaiting::NONE,
        };
        let before =
            crate::state::samus::Animation::restore(Step::FIRST, 1, types::BoostStage::Stage0, 0);
        let (after, sound) =
            samus_appearance::update(before.0.0, &inputs, &content, &Rules::VANILLA);
        assert_eq!(after.playhead.time_left, samus_appearance::Frames(7));
        out.animation(sound);
        out.retain(
            Opportunity::NormalDraw {
                previous: Action::WallJumping,
                current: Action::Running,
            }
            .read(),
        );
        assert_eq!(
            out.requests,
            [
                Request::Animation(AnimationSound::PlainSpin),
                Request::Appearance(SoundRequest::SpinEnded)
            ]
        );
    }
    #[test]
    fn drawing_reads_preparation_and_preserves_frozen_animation() {
        let capability = Capability::from_seed(SeedFacts::default()).unwrap();
        let mut samus = Samus {
            movement: data::pose::MovementType::Running,
            preparation_movement: data::pose::MovementType::SpinJumping,
            ..Samus::default()
        };
        samus.previous.movement = data::pose::MovementType::Running;
        let retained = samus.animation;
        for _ in 0..2 {
            let mut out = Outputs::default();
            normal_draw(&samus, capability, &mut out);
            assert_eq!(out.requests, [Request::Appearance(SoundRequest::SpinEnded)]);
            assert_eq!(samus.animation, retained);
            assert_eq!(
                samus.preparation_movement,
                data::pose::MovementType::SpinJumping
            );
        }
        samus.preparation_movement = samus.movement;
        let mut out = Outputs::default();
        normal_draw(&samus, capability, &mut out);
        assert_eq!(out.requests, Vec::new());
    }
    #[test]
    fn restored_capability_rejects_changed_owned_quantities() {
        let capability = Capability::from_seed(SeedFacts::default()).unwrap();
        let mut state = State::default();
        state.room.area = data::room::Area::Ceres;
        assert!(capability.holds(&state));
        state.room.area = data::room::Area::Brinstar;
        assert!(!capability.holds(&state));
        state.room.area = data::room::Area::Ceres;
        state.samus.animation.0.0.stage = types::BoostStage::Boosting;
        assert!(!capability.holds(&state));
        state.samus.animation.0.0.stage = types::BoostStage::Stage0;
        state.samus.drawing = crate::state::samus::DrawingHandler::Hidden;
        assert!(!capability.holds(&state));
    }
    #[test]
    fn frozen_passage_and_drawing_keep_both_requests() {
        let mut out = Outputs::default();
        passage_entry(
            Action::Running,
            false,
            samus_appearance::Charge::new(0).unwrap(),
            &mut out,
        );
        out.retain(
            Opportunity::NormalDraw {
                previous: Action::Spinning,
                current: Action::Running,
            }
            .read(),
        );
        assert_eq!(
            out.requests,
            [
                Request::Appearance(SoundRequest::SamusSilence),
                Request::Appearance(SoundRequest::SpinEnded)
            ]
        );
    }
}
