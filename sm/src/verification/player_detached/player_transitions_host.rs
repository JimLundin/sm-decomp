//! Carrier-only authored provider over the existing detached Composed owner.
//! No replacement gameplay is implemented or executed by this file.
#![forbid(unsafe_code)]
use sm_game::{player_transitions::{content::{
            AnimationProgram, AnimationTable, Content, LandingRegion, LandingRule, TransitionTable,
        }, effects::{BounceFault, Effects, GrappleTransition, Host}}, visual_host::Source};
use crate::{
    Composed,
    rom::{RomImage, RomRecord},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TransitionEvent {
    LoadedPose,
    Table(TransitionTable),
    AnimationTable(AnimationTable),
    LowWindow(u16),
    Program(u16),
    Random,
    Grapple(GrappleTransition),
    InvalidBounce(BounceFault, u16),
}
#[derive(Default)]
pub(crate) struct TransitionContract {
    // Native binding remains Composed.image, already used by collision pose.
    // Optional distinct Loaded binding; None preserves existing fixture setup.
    pub(crate) loaded_image: Option<RomImage>,
    pub(crate) loaded_pose_base: Option<usize>,
    pub(crate) transition_tables: Vec<(TransitionTable, usize)>,
    pub(crate) animation_tables: Vec<(AnimationTable, usize)>,
    pub(crate) programs: Vec<(u16, usize)>,
    pub(crate) landing_regions: Vec<(u16, LandingRegion)>,
    pub(crate) landing_rules: Vec<(LandingRegion, u16, u16, LandingRule)>,
    pub(crate) on_low_window: Option<fn(&mut Composed, u16)>,
    pub(crate) on_random: Option<fn(&mut Composed) -> u16>,
    pub(crate) on_grapple: Option<fn(&mut Composed, GrappleTransition)>,
    pub(crate) on_invalid_bounce: Option<fn(&mut Composed, BounceFault, u16)>,
    pub(crate) events: Vec<TransitionEvent>,
}
impl Composed {
    pub(crate) fn transition_image(&self, source: Source) -> RomImage {
        match source {
            Source::Loaded => self
                .transition_contract
                .loaded_image
                .as_ref()
                .unwrap_or(&self.image)
                .clone(),
            Source::Native => self.image.clone(),
        }
    }
}
impl Content for Composed {
    fn loaded_pose_table(&mut self) -> RomRecord {
        self.transition_contract
            .events
            .push(TransitionEvent::LoadedPose);
        RomRecord {
            image: self.transition_image(Source::Loaded),
            base: self
                .transition_contract
                .loaded_pose_base
                .expect("unprepared loaded pose table"),
        }
    }
    fn transition_table(&mut self, table: TransitionTable) -> RomRecord {
        self.transition_contract
            .events
            .push(TransitionEvent::Table(table));
        let base = self
            .transition_contract
            .transition_tables
            .iter()
            .find(|(key, _)| *key == table)
            .expect("unprepared transition table")
            .1;
        let source = match table {
            TransitionTable::InputPrograms
            | TransitionTable::BlockFallPoses
            | TransitionTable::BlockLandPoses => Source::Loaded,
            _ => Source::Native,
        };
        RomRecord {
            image: self.transition_image(source),
            base,
        }
    }
    fn animation_table(&mut self, table: AnimationTable) -> RomRecord {
        self.transition_contract
            .events
            .push(TransitionEvent::AnimationTable(table));
        let base = self
            .transition_contract
            .animation_tables
            .iter()
            .find(|(key, _)| *key == table)
            .expect("unprepared animation table")
            .1;
        RomRecord {
            image: self.transition_image(Source::Loaded),
            base,
        }
    }
    fn animation_program(&mut self, program: AnimationProgram, advance: u16) -> RomRecord {
        let cursor = program.0.wrapping_add(advance);
        if cursor & 0x8000 == 0 {
            self.transition_contract
                .events
                .push(TransitionEvent::LowWindow(cursor));
            (self
                .transition_contract
                .on_low_window
                .expect("unprepared transition low-window diagnostic"))(self, cursor);
        }
        self.transition_contract
            .events
            .push(TransitionEvent::Program(cursor));
        let base = self
            .transition_contract
            .programs
            .iter()
            .find(|(key, _)| *key == cursor)
            .expect("unprepared animation program")
            .1;
        // Capture AFTER a prepared diagnostic may replace the native binding.
        RomRecord {
            image: self.transition_image(Source::Native),
            base,
        }
    }
    fn landing_region(&self) -> LandingRegion {
        let area = self.world.player.room.area;
        self.transition_contract
            .landing_regions
            .iter()
            .find(|(key, _)| *key == area)
            .expect("unprepared landing region")
            .1
    }
    fn landing_room_rule(&self, region: LandingRegion) -> LandingRule {
        let room = &self.world.player.room;
        self.transition_contract
            .landing_rules
            .iter()
            .find(|(key, area, index, _)| {
                *key == region && *area == room.area && *index == room.entry.room_index
            })
            .expect("unprepared landing room policy")
            .3
    }
}
impl Effects for Composed {
    fn invalid_demo_instruction(&mut self) -> u16 {
        panic!("unprepared demo instruction diagnostic")
    }
    fn random_word(&mut self) -> u16 {
        self.transition_contract
            .events
            .push(TransitionEvent::Random);
        (self
            .transition_contract
            .on_random
            .expect("unprepared transition random effect"))(self)
    }
    fn grapple_transition(&mut self, transition: GrappleTransition) {
        self.transition_contract
            .events
            .push(TransitionEvent::Grapple(transition));
        (self
            .transition_contract
            .on_grapple
            .expect("unprepared transition grapple effect"))(self, transition);
    }
    fn invalid_bounce(&mut self, site: BounceFault, value: u16) {
        self.transition_contract
            .events
            .push(TransitionEvent::InvalidBounce(site, value));
        (self
            .transition_contract
            .on_invalid_bounce
            .expect("unprepared transition bounce diagnostic"))(self, site, value);
    }
}
impl Host for Composed {}
struct HostWitness<H: Host>(core::marker::PhantomData<H>);
const _: Option<HostWitness<Composed>> = None;
