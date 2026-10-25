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
pub(crate) fn command(a: u16, game: &mut crate::game_state::Game) -> u16 {
    crate::game::player_transitions::command(
        &mut Runtime::new(game),
        a,
        Runtime::finish_ceres_arrival,
    )
}

impl Effects for Runtime<'_> {
    fn invalid_demo_instruction(&mut self) -> u16 {
        self.foreign(crate::game::animated_tiles::native::unreachable)
    }
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
    fn finish_ceres_arrival(&mut self) {
        self.owned_call(|runtime| {
            use crate::game::{enemy_projectiles::lifecycle, room_mechanisms::host::Effects};
            let parameter = lifecycle::spawn_from_enemy(runtime, 0, 0, 0xa387);
            lifecycle::spawn_from_enemy(runtime, parameter, 0, 0xa395);
            runtime.session_mut().minimap.disabled = 0;
            Effects::delayed_room_music(runtime, 32);
        });
    }

    /// Ordered compatibility output; the core supplies semantic window-word indices.
    pub(crate) fn suit_window_word(&mut self, index: u16, value: u16) {
        self.write_slice_le(0x9800 + usize::from(index) * 2, value);
    }

    pub(crate) fn begin_suit_pickup(&mut self, gravity: bool) {
        use crate::{game::player_transitions::suit, types::SpawnHdmaObject_Args};
        suit::prepare(self, gravity, Self::suit_window_word);
        self.foreign(|game| crate::game::session_runtime::player_command(0x15, game));
        suit::center(self);
        visual_host::Effects::sound(self, 2, 6, 0x56);
        let program = if gravity { 0xd67a } else { 0xd5a2 };
        let args = SpawnHdmaObject_Args::new(0x41, 0x26, program);
        self.foreign(|game| unsafe {
            crate::game::sm_88::SpawnHdmaObject(0x91, (&raw const args).cast(), game)
        });
    }

    pub(crate) fn initialize_suit_pickup_hdma(&mut self) {
        for (address, value) in [
            (0x69, 19),
            (0x6c, 19),
            (0x6b, 4),
            (0x6d, 4),
            (0x6e, 16),
            (0x60, 0),
            (0x61, 2),
            (0x62, 32),
            (0x71, 51),
        ] {
            self.set_byte(address, value);
        }
        self.publish_suit_colors();
        for (address, value) in [(0xa88, 0x9800e4u32), (0xa8b, 0x98c8e4), (0xa8e, 0x999098)] {
            self.write_slice_native(address, value as u16);
            self.set_byte(address + 2, (value >> 16) as u8);
        }
        self.write_slice_le(crate::ram_map::words::hdma_var_1, 0);
    }

    fn publish_suit_colors(&mut self) {
        for lane in 0..3 {
            self.set_byte(0x74 + lane, self.byte(0xdf0 + lane));
        }
    }

    pub(crate) fn advance_suit_pickup(&mut self, gravity: bool) {
        use crate::ram_map::{word_arrays as arrays, words as ram};
        if crate::game::player_transitions::suit::advance(self, gravity, Self::suit_window_word) {
            self.publish_suit_colors();
            self.write_slice_le(ram::fx_layer_blending_config_c, 18);
            return;
        }
        if !gravity {
            visual_host::Effects::music_delayed8(self, 3);
        }
        for (address, value) in [(0x76, 0x80), (0x75, 64), (0x74, 32)] {
            self.set_byte(address, value);
        }
        self.write_slice_native(0xa88, 1);
        self.set_byte(0xa8a, 0x98);
        self.write_slice_native(0xa8b, 0);
        for address in [
            ram::demo_input_prev,
            ram::demo_input_prev_new,
            ram::demo_backup_prev_controller_input,
        ] {
            self.write_slice_le(address, 0);
        }
        self.suit_window_word(0, 255);
        self.session_mut().shared_sequence.stage = 0;
        self.session_mut().shared_sequence.value = 0;
        self.session_mut().player.palette.suit_red_green = 0;
        self.session_mut().player.palette.crystal_flash_delay = 0;
        let offset = usize::from(self.word(ram::hdma_object_index) >> 1) * 2;
        let program = arrays::hdma_object_instruction_list_pointers + offset;
        self.write_slice_le(program, self.word(program).wrapping_add(2));
        self.write_slice_le(arrays::hdma_object_instruction_timers + offset, 1);
        self.foreign(|game| crate::game::session_runtime::player_command(0xb, game));
    }

    pub(crate) fn load_demo_data(&mut self) {
        use crate::{game::player_transitions::demo, ram_map::words as ram};
        let scene = self.word(ram::demo_scene);
        let set = self.word(ram::demo_set);
        demo::load_data(self, set, scene);
        let set = self.word(ram::demo_set);
        let scene = self.word(ram::demo_scene);
        if demo::initialize_scene(self, set, scene) {
            self.owned_call(crate::game::player_transitions::equipment::face_forward);
            self.session_mut().player.transitions.draw_handler = 0xeb52;
        }
        demo::finish_setup(self);
        self.foreign(|game| unsafe { crate::game::sm_90::UpdateBeamTilesAndPalette(game) });
        for (field, value) in [
            (ram::button_config_up, 0x0800),
            (ram::button_config_down, 0x0400),
            (ram::button_config_left, 0x0200),
            (ram::button_config_right, 0x0100),
            (ram::button_config_shoot_x, 0x0040),
            (ram::button_config_jump_a, 0x0080),
            (ram::button_config_run_b, 0x8000),
            (ram::button_config_itemcancel_y, 0x4000),
            (ram::button_config_itemswitch, 0x2000),
            (ram::button_config_aim_up_R, 0x0010),
            (ram::button_config_aim_down_L, 0x0020),
            (ram::UNUSED_word_7E09E8, 1),
            (ram::debug_flag, 1),
            (ram::moonwalk_flag, 0),
            (ram::UNUSED_word_7E0DF8, 0),
            (ram::UNUSED_word_7E0DFA, 0),
            (ram::UNUSED_word_7E0DFC, 0),
        ] {
            self.write_slice_le(field, value);
        }
    }

    pub(crate) fn handle_demo_input(&mut self) {
        use crate::{game::player_transitions::demo, ram_map::words as ram};
        if demo::advance(self) {
            self.write_slice_le(
                ram::joypad1_input_samusfilter,
                self.session().player.demo.previous_held,
            );
            self.write_slice_le(
                ram::joypad1_newinput_samusfilter,
                self.session().player.demo.previous_pressed,
            );
            self.write_slice_le(ram::joypad1_lastkeys, self.session().player.demo.held);
            self.session_mut().player.demo.previous_held = self.session().player.demo.held;
            self.write_slice_le(ram::joypad1_newkeys, self.session().player.demo.pressed);
            self.session_mut().player.demo.previous_pressed = self.session().player.demo.pressed;
        }
        self.handle_input();
    }

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

pub(crate) fn face_forward(game: &mut crate::game_state::Game) {
    crate::game::player_transitions::equipment::face_forward(&mut Runtime::new(game));
}
