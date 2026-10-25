//! Legacy content and effect adapters for owned player transitions.
use crate::{
    game::{
        player_transitions::{
            content::{
                AnimationProgram, AnimationTable, Content, LandingRegion, LandingRule,
                TransitionTable,
            },
            effects::{BounceFault, Effects, GrappleTransition, Host},
        },
        session_access::SessionAccess,
        session_runtime::Runtime,
        visual_host::{self, Source},
    },
    rom::RomRecord,
};

static CRATERIA_LANDING_FLAGS: [u8; 17] = [1, 0, 0, 0, 0, 2, 0, 4, 0, 4, 4, 4, 4, 0, 4, 0, 0];

impl Content for Runtime<'_> {
    fn loaded_pose_table(&mut self) -> RomRecord {
        visual_host::Content::table(self, Source::Loaded, 0x91b629)
    }
    fn transition_table(&mut self, table: TransitionTable) -> RomRecord {
        let (source, address) = match table {
            TransitionTable::InputPrograms => (Source::Loaded, 0x919ee2),
            TransitionTable::BlockFallPoses => (Source::Loaded, 0x91e921),
            TransitionTable::BlockLandPoses => (Source::Loaded, 0x91e9f3),
            TransitionTable::StandingTurn => (Source::Native, 0x91f9c2),
            TransitionTable::CrouchingTurn => (Source::Native, 0x91f9cc),
            TransitionTable::JumpingTurn => (Source::Native, 0x91f9d6),
            TransitionTable::FallingTurn => (Source::Native, 0x91f9e0),
            TransitionTable::MoonwalkJumpTurn => (Source::Native, 0x91f9ea),
        };
        visual_host::Content::table(self, source, address)
    }
    fn animation_table(&mut self, table: AnimationTable) -> RomRecord {
        let address = match table {
            AnimationTable::PosePrograms => 0x91b010,
            AnimationTable::RunningWithoutBooster => 0x91b5d1,
            AnimationTable::RunningBoosterStages => 0x91b5de,
            AnimationTable::BoosterStageDurations => 0x91b61f,
        };
        visual_host::Content::table(self, Source::Loaded, address)
    }
    fn animation_program(&mut self, program: AnimationProgram, advance: u16) -> RomRecord {
        let cursor = program.0.wrapping_add(advance);
        if cursor & 0x8000 == 0 {
            // Actual diagnostic and verifier observation occur before native
            // acquisition. Do not use native_record's reduced invalid_window.
            self.foreign(|game| unsafe {
                crate::rtl::rom_bank_ptr::<0x91>(cursor, game);
            });
        }
        visual_host::Content::table(self, Source::Native, 0x910000 | u32::from(cursor))
    }
    fn landing_region(&self) -> LandingRegion {
        use LandingRegion::*;
        [
            Crateria, Brinstar, Dust, Dust, WaterSpray, Tourian, Clear, Clear,
        ][usize::from(self.session().room.area)]
    }
    fn landing_room_rule(&self, region: LandingRegion) -> LandingRule {
        let room = self.session().room.entry.room_index;
        match region {
            LandingRegion::Clear => LandingRule::Clear,
            LandingRegion::Dust => LandingRule::Dust,
            LandingRegion::WaterSpray => LandingRule::WaterSpray,
            LandingRegion::Crateria => {
                // Core asks for this room policy only after cinematic==0.
                if room == 28 {
                    return LandingRule::Dust;
                }
                if (room as i16).wrapping_sub(16) < 0 {
                    // Retain the actual source table and full exceptional index.
                    let flags = CRATERIA_LANDING_FLAGS[usize::from(room)];
                    if flags & 1 != 0 {
                        return LandingRule::WaterSprayWhenFxTen;
                    }
                    if flags & 2 != 0 {
                        return LandingRule::WaterSprayAtOrBelow944;
                    }
                    if flags & 4 != 0 {
                        return LandingRule::WaterSpray;
                    }
                }
                LandingRule::Clear
            }
            LandingRegion::Brinstar if room == 8 => LandingRule::Dust,
            LandingRegion::Brinstar | LandingRegion::Tourian => {
                if room.wrapping_sub(5) & 0x8000 == 0
                    && (room.wrapping_sub(9) & 0x8000 != 0 || room == 11)
                {
                    LandingRule::Dust
                } else {
                    LandingRule::Clear
                }
            }
        }
    }
}
impl Effects for Runtime<'_> {
    fn random_word(&mut self) -> u16 {
        self.foreign(|game| unsafe { crate::game::sm_80::NextRandom(game) })
    }
    fn grapple_transition(&mut self, transition: GrappleTransition) {
        self.foreign(|game| unsafe {
            match transition {
                GrappleTransition::SwingPose => crate::game::sm_9b::GrappleBeamFunc_BD95(game),
                GrappleTransition::AnchoredPose => crate::game::sm_9b::GrappleBeamFunc_BEEB(game),
            }
        });
    }
    fn invalid_bounce(&mut self, _site: BounceFault, _value: u16) {
        self.foreign(|game| unsafe {
            crate::rtl::Unreachable(game);
        });
    }
}
impl Host for Runtime<'_> {}

impl Runtime<'_> {
    /// X-ray turning retains its existing legacy implementation and foreign interval.
    pub(crate) fn handle_input(&mut self) {
        if matches!(self.session().player.movement_and_facing >> 8, 5 | 21)
            && self.inputs().frozen != 0
        {
            self.foreign(|game| unsafe { crate::game::sm_91::Samus_Func20(game) });
        } else {
            crate::game::player_transitions::handle_input(self);
        }
    }
}
