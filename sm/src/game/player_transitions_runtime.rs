//! Legacy content and effect adapters for owned player transitions.
use crate::game::player_transitions::xray::{self, BeamPhase};
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
        self.foreign(crate::game::visual_runtime::update_beam_tiles_and_palette);
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

    /// Frozen crouch and turn input uses the same owned X-ray transition.
    pub(crate) fn handle_input(&mut self) {
        if matches!(self.session().player.movement_and_facing >> 8, 5 | 21)
            && self.inputs().frozen != 0
        {
            self.owned_call(Self::handle_xray_input);
        } else {
            crate::game::player_transitions::handle_input(self);
        }
    }
}

pub(crate) fn face_forward(game: &mut crate::game_state::Game) {
    crate::game::player_transitions::equipment::face_forward(&mut Runtime::new(game));
}

pub(crate) fn mother_brain(event: u16, game: &mut crate::game_state::Game) {
    crate::game::player_transitions::mother_brain::react(&mut Runtime::new(game), event);
}

pub(crate) fn initialize(game: &mut crate::game_state::Game) {
    use crate::{
        game::{player_transitions::state::*, samus_palette, visual_state::PaletteBuffer},
        ram_map::words as ram,
    };
    let mut rt = Runtime::new(game);
    let debug_invincibility = rt.word(ram::debug_invincibility);
    // This compatibility workspace is only part of Session, including byte aliases.
    for address in (0xa02..=0xe0b).rev() {
        rt.set_byte(address, 0);
    }
    if rt.word(ram::game_state) != 40 {
        let ceres = rt.word(ram::loading_game_state) == 34;
        let t = &mut rt.session_mut().player.transitions;
        t.frame_alpha = FrameAlphaMode::Empty;
        t.frame_beta = if ceres { 0xe8dc } else { 0xe86a };
        t.draw_handler = 0xeb52;
        t.momentum = MomentumAction::from_raw(if ceres { u16::MAX } else { 0 });
        t.interruption = InterruptionAction::from_raw(0);
        t.hurt = HurtAction::from_raw(0);
        if ceres {
            samus_palette::load_suit(&mut rt, PaletteBuffer::Current);
        }
        rt.session_mut().player.transitions.input_mode = InputMode::Normal;
        if !ceres {
            rt.write_slice_le(ram::debug_invincibility, debug_invincibility);
        }
    }
    rt.session_mut()
        .player
        .transitions
        .invalidate_pose_requests();
    rt.session_mut().player.transitions.frame_gamma = if rt.session().room.area == 6 {
        FrameGammaMode::EscapeTimer
    } else {
        FrameGammaMode::Disabled
    };
    rt.session_mut().player.movement_handler = MovementMode::Normal;
    rt.write_slice_le(ram::UNUSED_word_7E0A5E, (-2764i16) as u16);
    rt.write_slice_le(ram::samus_prev_health_for_flash, 50);
    rt.session_mut().player.palette.visor_timer = 1537;
    for shot in &mut rt.session_mut().projectiles.shots {
        shot.motion = 0xb169;
    }
    let p = &mut rt.session_mut().player;
    p.grapple = 0xc4f0;
    p.collision.horizontal_slope_flags = 3;
    p.palette.hurt_flash = 0;
    p.special_palette_flags = 0;
    p.camera_distance.fill(crate::game::session::FixedMotion {
        fraction: 0,
        whole: 1,
    });
    for index in (0..256).rev() {
        rt.suit_window_word(index, 255);
    }
    rt.session_mut().player.contact.y_acceleration.fraction = 0x1c00;
    rt.session_mut().player.contact.y_acceleration.whole = 0;
    rt.write_slice_le(ram::fx_y_pos, u16::MAX);
    rt.write_slice_le(ram::lava_acid_y_pos, u16::MAX);
    rt.foreign(crate::game::visual_runtime::update_beam_tiles_and_palette);
    rt.write_slice_le(ram::cinematic_function, 0);
    let p = &mut rt.session_mut().player;
    p.pose = 0;
    p.movement_and_facing = 0;
    p.history = Default::default();
    rt.write_slice_le(ram::enemy_index_to_shake, u16::MAX);
    rt.session_mut().projectiles.controls.selected_weapon = 0;
    rt.session_mut().projectiles.controls.automatic_weapon = 0;
    let p = &mut rt.session_mut().player;
    p.invincibility = 0;
    p.knockback_timer = 0;
    p.palette.hurt_flash = 0;
    rt.write_slice_le(ram::debug_invincibility, 0);
    if rt.word(ram::game_state) == 40 {
        rt.load_demo_data();
    }
    rt.write_slice_le(
        ram::samus_prev_health_for_flash,
        rt.session().inventory.health.current,
    );
}

pub(crate) fn xray_input(game: &mut crate::game_state::Game) {
    Runtime::new(game).handle_xray_input();
}

pub(crate) fn finish_xray(game: &mut crate::game_state::Game) {
    crate::game::player_transitions::xray::finish(&mut Runtime::new(game));
}

impl Runtime<'_> {
    pub(crate) fn advance_xray_beam(&mut self, phase: BeamPhase) {
        use crate::ram_map::words as ram;
        if self.inputs().run_button & self.inputs().held == 0 {
            let demo = &mut self.session_mut().player.demo;
            demo.pre_action = if phase == BeamPhase::Full {
                demo.pre_action.wrapping_add(1)
            } else {
                3
            };
            return;
        }
        if phase == BeamPhase::Widening {
            xray::widen(&mut self.session_mut().player.demo);
        } else if phase == BeamPhase::Full {
            let up = self.word(ram::button_config_up) & self.inputs().held != 0;
            if up || self.word(ram::button_config_down) & self.inputs().held != 0 {
                let player = &mut self.session_mut().player;
                xray::steer(&mut player.transitions.xray_angle, player.demo.held, up);
            }
        }
        self.foreign(|game| unsafe { crate::game::sm_88::CalculateXrayHdmaTable(game) });
        if phase == BeamPhase::Initial {
            let demo = &mut self.session_mut().player.demo;
            demo.pre_action = demo.pre_action.wrapping_add(1);
        }
    }

    fn handle_xray_input(&mut self) {
        use crate::ram_map::words as ram;
        let binding = if self.session().player.movement_and_facing as u8 == 4 {
            ram::button_config_right
        } else {
            ram::button_config_left
        };
        let turn = self.word(binding) & self.inputs().held != 0;
        crate::game::player_transitions::xray::turn(self, turn);
    }

    pub(crate) fn request_xray(&mut self) {
        if self.inputs().frozen != 0
            || self.inputs().run_button & self.inputs().held == 0
            || !self.initialize_xray()
        {
            return;
        }
        self.foreign(|game| unsafe {
            crate::game::sm_88::SpawnHdmaObject(
                0x91,
                crate::rtl::rom_ptr!(game; 0x91caf2).cast(),
                game,
            )
        });
    }

    pub(crate) fn initialize_xray(&mut self) -> bool {
        use crate::{
            game::player_transitions::{state::InterruptionAction, xray},
            ram_map::words as ram,
        };
        if !xray::request(self) {
            return false;
        }
        self.write_slice_le(ram::time_is_frozen_flag, 1);
        self.session_mut().player.transitions.interruption = InterruptionAction::from_raw(5);
        for index in (0..256).rev() {
            self.suit_window_word(index, 255);
        }
        xray::set_world_running(self.session_mut(), false);
        self.write_slice_native(0xa88, 1);
        self.set_byte(0xa8a, 0x98);
        self.write_slice_native(0xa8b, 0);
        self.session_mut().player.demo.previous_held = 0x98c8;
        self.write_slice_native(0xa8e, 0x9098);
        self.set_byte(0xa90, 0x99);
        self.write_slice_le(ram::hdma_var_1, 0);
        let demo = &mut self.session_mut().player.demo;
        demo.pre_action = 0;
        demo.delay = 0;
        demo.program = 0;
        demo.held = 0;
        demo.pressed = 0;
        self.session_mut().player.transitions.xray_angle =
            if self.session().player.movement_and_facing as u8 == 4 {
                192
            } else {
                64
            };
        true
    }
}

pub(crate) fn damage_player(amount: u16, game: &mut crate::game_state::Game) {
    Runtime::new(game).apply_contact_damage(amount);
}

impl Runtime<'_> {
    pub(crate) fn apply_contact_damage(&mut self, amount: u16) {
        let frozen = self.inputs().frozen;
        if crate::game::samus_periodic_damage::direct(self.session_mut(), frozen, amount).is_err() {
            crate::game::player_projectiles::host::Host::invalid_damage(self);
        }
    }
}
