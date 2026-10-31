//! Legacy content and effect adapters for owned player transitions.
use crate::game::player_transitions::landing;
use crate::game::player_transitions::xray;
use crate::{
    game::{
        player_transitions::{
            commands,
            content::{
                AnimationProgram, AnimationTable, Content, LandingRegion, LandingRule,
                TransitionTable,
            },
            effects::{BounceFault, Effects, GameEffects, GrappleTransition, Host, XrayRestore},
        },
        session_runtime::Runtime,
        visual_host::{self, Source},
    },
    rom::RomRecord,
};

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
        self.legacy_native_record(0x91, cursor)
    }
    fn landing_region(&self) -> LandingRegion {
        landing::area_region(self.session().room.area)
    }
    fn landing_room_rule(&self, region: LandingRegion) -> LandingRule {
        landing::room_rule(region, self.session().room.entry.room_index)
    }
}
pub(crate) fn command(a: u16, game: &mut crate::game_state::Game) -> u16 {
    command_in(&mut Runtime::new(game), a)
}
/// Run Samus pose command `a` on a lease that is already open.
pub(crate) fn command_in(runtime: &mut Runtime<'_>, a: u16) -> u16 {
    crate::game::player_transitions::command(runtime, a, commands::finish_ceres_arrival)
}

impl Effects for Runtime<'_> {
    fn invalid_demo_instruction(&mut self) -> u16 {
        self.foreign(crate::game::animated_tiles::native::unreachable)
    }
    fn random_word(&mut self) -> u16 {
        self.foreign(crate::game_state::Game::next_random)
    }
    fn grapple_transition(&mut self, transition: GrappleTransition) {
        self.owned_call(|runtime| match transition {
            GrappleTransition::SwingPose => sm_game::grapple_swing::position_swing(runtime),
            GrappleTransition::AnchoredPose => sm_game::grapple_swing::position_anchored(runtime),
        });
    }
    fn invalid_bounce(&mut self, _site: BounceFault, _value: u16) {
        self.foreign(|game| {
            crate::rtl::Unreachable(game);
        });
    }
}
impl Host for Runtime<'_> {}
impl sm_game::runtime_access::OwnedCall for Runtime<'_> {
    fn owned_call<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        Runtime::owned_call(self, action)
    }
}
pub(crate) use sm_game::player_transitions::input::dispatch_input;
pub(crate) use sm_game::player_transitions::input::finish_frame as finish_player_frame;

pub(crate) fn face_forward(game: &mut crate::game_state::Game) {
    crate::game::player_transitions::equipment::face_forward(&mut Runtime::new(game));
}

pub(crate) fn mother_brain(event: u16, game: &mut crate::game_state::Game) {
    crate::game::player_transitions::mother_brain::react(&mut Runtime::new(game), event);
}

pub(crate) fn finish_xray(game: &mut crate::game_state::Game) {
    crate::game::player_transitions::xray::finish(&mut Runtime::new(game));
}

pub(crate) fn damage_player(amount: u16, game: &mut crate::game_state::Game) {
    sm_game::samus_periodic_damage::contact(&mut Runtime::new(game), amount);
}
pub(crate) fn initialize(game: &mut crate::game_state::Game) {
    sm_game::player_transitions::initialize(&mut Runtime::new(game));
}
impl Runtime<'_> {
    pub(crate) fn request_xray(&mut self) {
        xray::request_scope(self);
    }
}
impl GameEffects for Runtime<'_> {
    fn pose_command(&mut self, command: u16) {
        self.foreign(|game| crate::game::session_runtime::player_command(command, game));
    }
    fn spawn_hdma_object(&mut self, definition: &crate::types::HdmaObjectDefinition) {
        self.foreign(|game| crate::game::visual_runtime::spawn_hdma(0x91, definition, game));
    }
    fn update_beam_graphics(&mut self) {
        self.foreign(crate::game::visual_runtime::update_beam_tiles_and_palette);
    }
    fn draw_xray_window_table(&mut self, window: sm_game::beam_windows::XrayWindow) {
        self.foreign(|game| sm_game::beam_windows::calculate_xray_window(window, game));
    }
    fn restore_xray_background(&mut self, step: XrayRestore) {
        use crate::game::xray_background::{finish_restore, restore_left, restore_right};
        use sm_game::hdma_program::with_current_object;
        match step {
            XrayRestore::Left => with_current_object(self, restore_left),
            XrayRestore::Right => self.foreign(restore_right),
            XrayRestore::Finish => with_current_object(self, finish_restore),
        }
    }
    fn spawn_xray_scope_hdma(&mut self) {
        self.foreign(|game| {
            let image = game.capture_native_rom_binding();
            let table = crate::rom::lorom_offset(0x91caf2) as usize;
            crate::game::visual_runtime::spawn_hdma_words(
                0x91,
                |index| {
                    let image = image.as_ref().expect("native ROM must be registered");
                    image.table_word(table, index as isize)
                },
                game,
            )
        });
    }
}
