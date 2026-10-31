//! Legacy content and effect adapters for owned player transitions.
use crate::game::player_transitions::landing;
use crate::game::player_transitions::state::DrawMode;
use crate::game::player_transitions::xray::{self, BeamPhase, BlendPhase};
use crate::{
    game::{
        player_transitions::{
            content::{
                AnimationProgram, AnimationTable, Content, LandingRegion, LandingRule,
                TransitionTable,
            },
            effects::{BounceFault, Effects, GrappleTransition, Host},
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
    crate::game::player_transitions::command(runtime, a, Runtime::finish_ceres_arrival)
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

impl Runtime<'_> {
    fn finish_ceres_arrival(&mut self) {
        self.owned_call(|runtime| {
            use {
                crate::game::enemy_projectiles::lifecycle, sm_game::room_mechanisms::host::Effects,
            };
            let parameter = lifecycle::spawn_from_enemy(runtime, 0, 0, 0xa387);
            lifecycle::spawn_from_enemy(runtime, parameter, 0, 0xa395);
            runtime.session_mut().minimap.disabled = 0;
            Effects::delayed_room_music(runtime, 32);
        });
    }

    pub(crate) fn begin_suit_pickup(&mut self, gravity: bool) {
        use crate::{game::player_transitions::suit, types::HdmaObjectDefinition};
        suit::prepare(self, gravity, xray::set_window_word);
        self.foreign(|game| crate::game::session_runtime::player_command(0x15, game));
        suit::center(self);
        visual_host::Effects::sound(self, 2, 6, 0x56);
        let program = if gravity { 0xd67a } else { 0xd5a2 };
        let args = HdmaObjectDefinition::new(0x41, 0x26, program);
        self.foreign(|game| crate::game::visual_runtime::spawn_hdma(0x91, &args, game));
    }

    pub(crate) fn advance_suit_pickup(&mut self, gravity: bool) {
        use crate::ram_map::{word_arrays as arrays, words as ram};
        if crate::game::player_transitions::suit::advance(self, gravity, xray::set_window_word) {
            crate::game::player_transitions::suit::publish_colors(self);
            self.write_slice_le(ram::FX_LAYER_BLENDING_CONFIG_C, 18);
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
            ram::DEMO_INPUT_PREV,
            ram::DEMO_INPUT_PREV_NEW,
            ram::DEMO_BACKUP_PREV_CONTROLLER_INPUT,
        ] {
            self.write_slice_le(address, 0);
        }
        xray::set_window_word(self, 0, 255);
        self.session_mut().shared_sequence.stage = 0;
        self.session_mut().shared_sequence.value = 0;
        self.session_mut().presentation.palette_mut().suit_red_green = 0;
        self.session_mut()
            .presentation
            .palette_mut()
            .crystal_flash_delay = 0;
        let offset = usize::from(self.word(ram::HDMA_OBJECT_INDEX) >> 1) * 2;
        let program = arrays::HDMA_OBJECT_INSTRUCTION_LIST_POINTERS + offset;
        self.write_slice_le(program, self.word(program).wrapping_add(2));
        self.write_slice_le(arrays::HDMA_OBJECT_INSTRUCTION_TIMERS + offset, 1);
        self.foreign(|game| crate::game::session_runtime::player_command(0xb, game));
    }

    pub(crate) fn load_demo_data(&mut self) {
        use crate::{game::player_transitions::demo, ram_map::words as ram};
        let scene = self.word(ram::DEMO_SCENE);
        let set = self.word(ram::DEMO_SET);
        demo::load_data(self, set, scene);
        let set = self.word(ram::DEMO_SET);
        let scene = self.word(ram::DEMO_SCENE);
        if demo::initialize_scene(self, set, scene) {
            self.owned_call(crate::game::player_transitions::equipment::face_forward);
            self.session_mut().player.transitions.draw_handler = DrawMode::Normal;
        }
        demo::finish_setup(self);
        self.foreign(crate::game::visual_runtime::update_beam_tiles_and_palette);
        for (field, value) in [
            (ram::BUTTON_CONFIG_UP, 0x0800),
            (ram::BUTTON_CONFIG_DOWN, 0x0400),
            (ram::BUTTON_CONFIG_LEFT, 0x0200),
            (ram::BUTTON_CONFIG_RIGHT, 0x0100),
            (ram::BUTTON_CONFIG_SHOOT_X, 0x0040),
            (ram::BUTTON_CONFIG_JUMP_A, 0x0080),
            (ram::BUTTON_CONFIG_RUN_B, 0x8000),
            (ram::BUTTON_CONFIG_ITEMCANCEL_Y, 0x4000),
            (ram::BUTTON_CONFIG_ITEMSWITCH, 0x2000),
            (ram::BUTTON_CONFIG_AIM_UP_R, 0x0010),
            (ram::BUTTON_CONFIG_AIM_DOWN_L, 0x0020),
            (ram::UNUSED_WORD_7E09E8, 1),
            (ram::DEBUG_FLAG, 1),
            (ram::MOONWALK_FLAG, 0),
            (ram::UNUSED_WORD_7E0DF8, 0),
            (ram::UNUSED_WORD_7E0DFA, 0),
            (ram::UNUSED_WORD_7E0DFC, 0),
        ] {
            self.write_slice_le(field, value);
        }
    }

    pub(crate) fn handle_demo_input(&mut self) {
        use crate::{game::player_transitions::demo, ram_map::words as ram};
        if demo::advance(self) {
            self.write_slice_le(
                ram::JOYPAD1_INPUT_SAMUSFILTER,
                self.session().player.demo.previous_held,
            );
            self.write_slice_le(
                ram::JOYPAD1_NEWINPUT_SAMUSFILTER,
                self.session().player.demo.previous_pressed,
            );
            self.write_slice_le(ram::JOYPAD1_LASTKEYS, self.session().player.demo.held);
            self.session_mut().player.demo.previous_held = self.session().player.demo.held;
            self.write_slice_le(ram::JOYPAD1_NEWKEYS, self.session().player.demo.pressed);
            self.session_mut().player.demo.previous_pressed = self.session().player.demo.pressed;
        }
        self.handle_input();
    }

    /// Frozen crouch and turn input uses the same owned X-ray transition.
    pub(crate) fn handle_input(&mut self) {
        if matches!(self.session().player.movement_and_facing >> 8, 5 | 21)
            && self.inputs().frozen != 0
        {
            self.owned_call(|runtime| xray::handle_input(runtime));
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
    let debug_invincibility = rt.word(ram::DEBUG_INVINCIBILITY);
    // This compatibility workspace is only part of Session, including byte aliases.
    for address in (0xa02..=0xe0b).rev() {
        rt.set_byte(address, 0);
    }
    if rt.word(ram::GAME_STATE) != 40 {
        let ceres = rt.word(ram::LOADING_GAME_STATE) == 34;
        let t = &mut rt.session_mut().player.transitions;
        t.frame_alpha = FrameAlphaMode::Empty;
        t.frame_beta = if ceres {
            FrameBetaMode::Minimap
        } else {
            FrameBetaMode::Fanfare
        };
        t.draw_handler = DrawMode::Normal;
        t.momentum = MomentumAction::from_raw(if ceres { u16::MAX } else { 0 });
        t.interruption = InterruptionAction::from_raw(0);
        t.hurt = HurtAction::from_raw(0);
        if ceres {
            samus_palette::load_suit(&mut rt, PaletteBuffer::Current);
        }
        rt.session_mut().player.transitions.input_mode = InputMode::Normal;
        if !ceres {
            rt.write_slice_le(ram::DEBUG_INVINCIBILITY, debug_invincibility);
        }
    }
    let transitions = &mut rt.session_mut().player.transitions;
    transitions.invalidate_pose_requests();
    rt.session_mut().player.transitions.frame_gamma = if rt.session().room.area == 6 {
        FrameGammaMode::EscapeTimer
    } else {
        FrameGammaMode::Disabled
    };
    rt.session_mut().player.movement_handler = MovementMode::Normal;
    rt.write_slice_le(ram::UNUSED_WORD_7E0A5E, (-2764i16) as u16);
    rt.write_slice_le(ram::SAMUS_PREV_HEALTH_FOR_FLASH, 50);
    rt.session_mut().presentation.palette_mut().visor_timer = 1537;
    for shot in &mut rt.session_mut().projectiles.shots {
        shot.motion = 0xb169;
    }
    let state = rt.session_mut();
    let p = &mut state.player;
    p.grapple = 0xc4f0;
    p.collision.horizontal_slope_flags = 3;
    state.presentation.palette_mut().hurt_flash = 0;
    p.special_palette_flags = 0;
    p.camera_distance.fill(sm_game::geometry::FixedMotion {
        fraction: 0,
        whole: 1,
    });
    xray::clear_window(&mut rt);
    rt.session_mut().player.contact.y_acceleration.fraction = 0x1c00;
    rt.session_mut().player.contact.y_acceleration.whole = 0;
    rt.session_mut().room.liquid.surface.whole = u16::MAX;
    rt.session_mut().room.liquid.lava_acid.whole = u16::MAX;
    rt.foreign(crate::game::visual_runtime::update_beam_tiles_and_palette);
    rt.write_slice_le(ram::CINEMATIC_FUNCTION, 0);
    let state = rt.session_mut();
    let p = &mut state.player;
    p.pose = 0;
    p.movement_and_facing = 0;
    p.history = Default::default();
    rt.write_slice_le(ram::ENEMY_INDEX_TO_SHAKE, u16::MAX);
    rt.session_mut().projectiles.controls.selected_weapon = 0;
    rt.session_mut().projectiles.controls.automatic_weapon = 0;
    let state = rt.session_mut();
    let p = &mut state.player;
    p.invincibility = 0;
    p.knockback_timer = 0;
    state.presentation.palette_mut().hurt_flash = 0;
    rt.write_slice_le(ram::DEBUG_INVINCIBILITY, 0);
    if rt.word(ram::GAME_STATE) == 40 {
        rt.load_demo_data();
    }
    rt.write_slice_le(
        ram::SAMUS_PREV_HEALTH_FOR_FLASH,
        rt.session().inventory.health.current,
    );
}

pub(super) fn dispatch_input(h: &mut Runtime<'_>) {
    use crate::game::player_transitions::state::InputMode;
    use crate::ram_map::words as ram;
    match h.session().player.transitions.input_mode {
        InputMode::Normal => h.handle_input(),
        InputMode::Demo => h.handle_demo_input(),
        InputMode::Xray => xray::handle_input(h),
        InputMode::AutoJump => {
            let pressed = h.inputs().pressed;
            let timer = h.session().player.contact.auto_jump;
            if timer != 0 && (timer.wrapping_sub(9) as i16) < 0 {
                h.write_slice_le(ram::JOYPAD1_NEWKEYS, pressed | h.inputs().jump_button);
                h.session_mut().player.contact.auto_jump = 0;
            }
            h.handle_input();
            h.write_slice_le(ram::JOYPAD1_NEWKEYS, pressed);
            h.session_mut().player.transitions.input_mode = InputMode::Normal;
        }
        InputMode::Disabled => {}
        InputMode::Unknown(_) => {
            sm_game::player_projectiles::host::Host::unknown(h);
        }
    }
}

pub(crate) fn finish_xray(game: &mut crate::game_state::Game) {
    crate::game::player_transitions::xray::finish(&mut Runtime::new(game));
}

impl Runtime<'_> {
    pub(crate) fn draw_xray_window(&mut self) {
        let player = &self.session().player;
        let facing = player.movement_and_facing as u8;
        let x = player
            .x
            .wrapping_sub(self.inputs().camera_x)
            .wrapping_add(if facing == 4 { 65533 } else { 3 });
        let y = player.y.wrapping_sub(self.inputs().camera_y).wrapping_sub(
            if player.movement_and_facing >> 8 == 5 {
                12
            } else {
                16
            },
        );
        let offscreen = (x as i16) < 0 || x >= 256;
        if (x as i16) < 0 && facing == 4 || (x as i16) >= 256 && facing == 8 {
            xray::clear_window(self);
            return;
        }
        let angle = player.transitions.xray_angle;
        let width = player.demo.held;
        self.foreign(|game| {
            let window = sm_game::beam_windows::XrayWindow {
                x,
                y,
                angle,
                width,
                off_screen: offscreen,
                table: 0x9800,
            };
            sm_game::beam_windows::calculate_xray_window(window, game)
        });
    }

    pub(crate) fn advance_xray_background(&mut self) {
        use crate::game::xray_background::{finish_restore, restore_left, restore_right};
        use sm_game::hdma_program::with_current_object;
        xray::apply_blending(self, BlendPhase::Beam);
        let stages: [fn(&mut Self); 6] = [
            |r| r.advance_xray_beam(BeamPhase::Initial),
            |r| r.advance_xray_beam(BeamPhase::Widening),
            |r| r.advance_xray_beam(BeamPhase::Full),
            |r| with_current_object(r, restore_left),
            |r| r.foreign(restore_right),
            |r| with_current_object(r, finish_restore),
        ];
        // Retain the six-entry bounds check after the blending writes.
        stages[usize::from(self.session().player.demo.pre_action)](self);
    }

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
            let up = self.word(ram::BUTTON_CONFIG_UP) & self.inputs().held != 0;
            if up || self.word(ram::BUTTON_CONFIG_DOWN) & self.inputs().held != 0 {
                let player = &mut self.session_mut().player;
                xray::steer(&mut player.transitions.xray_angle, player.demo.held, up);
            }
        }
        self.draw_xray_window();
        if phase == BeamPhase::Initial {
            let demo = &mut self.session_mut().player.demo;
            demo.pre_action = demo.pre_action.wrapping_add(1);
        }
    }

    pub(crate) fn request_xray(&mut self) {
        if self.inputs().frozen != 0
            || self.inputs().run_button & self.inputs().held == 0
            || !xray::initialize(self)
        {
            return;
        }
        self.foreign(|game| {
            let image = game.capture_native_rom_binding();
            crate::game::visual_runtime::spawn_hdma_words(
                0x91,
                |index| {
                    image
                        .as_ref()
                        .expect("native ROM must be registered")
                        .table_word(crate::rom::lorom_offset(0x91caf2) as usize, index as isize)
                },
                game,
            )
        });
    }
}

pub(crate) fn damage_player(amount: u16, game: &mut crate::game_state::Game) {
    sm_game::samus_periodic_damage::contact(&mut Runtime::new(game), amount);
}

pub(crate) fn finish_player_frame(h: &mut Runtime<'_>) {
    use crate::{
        game::{player_transitions, progress_flags},
        ram_map::words as ram,
    };
    let inputs = h.inputs().clone();
    player_transitions::finish_input(h.session_mut(), &inputs);
    h.write_slice_le(ram::JOYPAD1_INPUT_SAMUSFILTER, inputs.held);
    h.write_slice_le(ram::JOYPAD1_NEWINPUT_SAMUSFILTER, inputs.pressed);
    h.owned_call(sm_game::player_projectiles::feedback::movement_sound);
    if h.word(ram::ENABLE_DEBUG) != 0 {
        if matches!(h.session().player.pose, 0 | 0x9b) {
            if h.word(ram::JOYPAD2_LAST) & 0x30 == 0x30 && h.word(ram::JOYPAD2_NEW_KEYS) & 0x80 != 0
            {
                h.write_slice_le(ram::DEBUG_INVINCIBILITY, 7);
            }
        } else {
            if (h.word(ram::DEBUG_INVINCIBILITY).wrapping_sub(7) as i16) >= 0 {
                return;
            }
            h.write_slice_le(ram::DEBUG_INVINCIBILITY, 0);
        }
    }
    if h.owned_call(|h| progress_flags::event_happened(h, 0xe))
        && h.session().player.transitions.frame_gamma
            == player_transitions::state::FrameGammaMode::DrawEscapeTimer
        && h.word(ram::GAME_STATE) != 35
    {
        h.write_slice_le(ram::GAME_STATE, 35);
    }
}
