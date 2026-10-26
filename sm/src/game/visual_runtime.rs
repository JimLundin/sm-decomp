//! Visual adapters on the one aggregate Session, with explicit legacy effects.
use crate::game::actors::access::{AccessMode::LittleEndian as Le, ActorAccess, ActorKey as Key};
use crate::game::player_transitions::xray::BeamPhase;
use crate::game::projectile_presentation_ram as sprite_output;
use crate::game::xray_background::{self, finish_restore, restore_left, restore_right};
use crate::game::{
    explosion_window::{self, Phase},
    liquid_effects, mother_brain_visuals, scrolling_sky,
};
use crate::ram_map::{checked_word, word, words as ram_words};
use crate::rtl::{Unreachable, words};
use crate::{
    game::{
        animated_tiles,
        constants::bank_86::addr_kEproj_TourianStatueDustClouds,
        enemy_projectiles::{self, SpawnEprojWithRoomGfx},
        gameplay_flag_runtime::{CheckEventHappened, SetEventHappened},
        palette_effects, player_projectiles,
        room_mechanisms::spawn_hardcoded,
        session_runtime::Runtime,
        visual_host::{self, AnimationMember, PaletteMember, Source, StatueProjectile},
        visual_state::{
            CeresExplosion, CinematicFade, CinematicObjects, CinematicTilemap, EndingGraphics,
            EyeBeamPhase, HazePhase, HdmaWave, PaletteBuffer, TourianStatuePhase, Upload, Weather,
        },
    },
    game_state::Game,
    rom::RomRecord,
    types::{SpawnHardcodedPlmArgs, SpawnHdmaObject_Args},
};

pub(crate) unsafe fn spawn_bg3_scroll(game: &mut Game) {
    unsafe { spawn_hdma_last(0x88, &SpawnHdmaObject_Args::new(0x43, 0x11, 0xd8d0), game) }
}

crate::forward_calls! { pub(crate) unsafe fn (game: &mut Game);
    initialize_ceres_elevator => spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x40, 5, 0xd939), game);
    initialize_draygon_layer => spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0, 0x2c, 0xdf4f), game);
    initialize_morph_eye_beam => spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x41, 0x26, 0xe8ec), game);
    initialize_intro_crossfade => spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0, 0x31, 0xec03), game);
}

pub(crate) unsafe fn initialize_ceres_ridley(game: &mut Game) {
    unsafe {
        spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x40, 5, 0xd906), game);
        spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0, 0x2c, 0xd91d), game);
    }
}

pub(crate) unsafe fn initialize_bomb_torizo_haze(game: &mut Game) {
    unsafe {
        spawn_hdma(0x88, &SpawnHdmaObject_Args::new(2, 0x32, 0xdd4a), game);
        spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0, 0x32, 0xdd62), game);
    }
}

pub(crate) unsafe fn initialize_room_haze(game: &mut Game) {
    let program = if crate::game::gameplay_flag_runtime::CheckBossBitForCurArea(1, game) & 1 != 0 {
        0xdeeb
    } else {
        0xded3
    };
    unsafe {
        spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x40, 0x32, program), game);
    }
}

pub(crate) unsafe fn initialize_phantoon_motion(direction: u16, speed: u16, game: &mut Game) {
    use crate::game::actors::access::AccessMode::Native;
    {
        let mut runtime = Runtime::new(game);
        runtime.edit_actor(Key::from_legacy(0x80), Native, |actor| {
            actor.parameters[0] = direction
        });
        runtime.edit_actor(Key::from_legacy(0xc0), Native, |actor| {
            actor.variables[3] = 0;
            actor.variables[4] = 0;
            actor.pre_instruction = speed;
        });
    }
    unsafe {
        spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x42, 0xf, 0xe4a8), game);
    }
}

pub(crate) unsafe fn initialize_title_gradient(game: &mut Game) {
    unsafe {
        spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0, 0x32, 0xeb73), game);
        spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0, 0x31, 0xeb85), game);
    }
}

pub(crate) unsafe fn initialize_intro_wave(game: &mut Game) {
    word!(game.g_word_7E0D9C = 1);
    word_transfers!(game, ascending, 1; native(0xd9e) <- value(0x4000));
    word!(game.loop_counter_transfer_enemies_to_vram = 8);
    Runtime::new(game).initialize_wave_pair(0xd82, 0x9800);
    unsafe {
        spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x42, 0x11, 0xec8a), game);
    }
}

fn hdma_definition(args: &SpawnHdmaObject_Args) -> [u16; 2] {
    [
        u16::from_ne_bytes([args.dma_control, args.target_register]),
        args.instruction_list,
    ]
}

// Hardware-facing entry points require a live, exclusively accessed SNES backend.
pub(crate) unsafe fn spawn_hdma(bank: u8, args: &SpawnHdmaObject_Args, game: &mut Game) -> u16 {
    unsafe {
        let words = hdma_definition(args);
        spawn_hdma_words(bank, |index| words[index], game)
    }
}

pub(crate) unsafe fn spawn_hdma_last(bank: u8, args: &SpawnHdmaObject_Args, game: &mut Game) {
    unsafe {
        let words = hdma_definition(args);
        initialize_hdma(5, bank, |index| words[index], game);
    }
}

pub(crate) unsafe fn spawn_hdma_words(
    bank: u8,
    read_word: impl Fn(usize) -> u16,
    game: &mut Game,
) -> u16 {
    unsafe {
        match (0..6).find(|&slot| checked_word!(game.hdma_object_channels_bitmask[slot]) == 0) {
            Some(slot) => initialize_hdma(slot, bank, read_word, game),
            None => u16::MAX,
        }
    }
}

unsafe fn initialize_hdma(
    slot: usize,
    bank: u8,
    read_word: impl Fn(usize) -> u16,
    game: &mut Game,
) -> u16 {
    checked_word!(game.hdma_object_pre_instructions[slot] = 0x84b8);
    checked_word!(game.hdma_object_pre_instruction_bank[slot] = 0x88);
    // A native definition reads its program here, and control only after publication.
    checked_word!(game.hdma_object_instruction_list_pointers[slot] = read_word(1));
    checked_word!(game.hdma_object_instruction_timers[slot] = 1);
    for field in [0x1908, 0x1914, 0x1920, 0x192c, 0x1938] {
        checked_word!(game, field + 2 * slot; = 0);
    }
    checked_word!(game.hdma_object_channels_bitmask[slot] = 4 << slot);
    let channel = 0x20 + 0x10 * slot as u16;
    checked_word!(game.hdma_object_bank_slot[slot] = (u16::from(bank) << 8) | channel);
    let control = read_word(0);
    unsafe {
        crate::rtl::WriteRegWord(crate::rtl::DMAP0 as u16 + channel, control, game);
    }
    2 * slot as u16
}

pub(crate) fn set_hdma_enabled(enabled: bool, game: &mut Game) {
    let flags = checked_word!(game.hdma_objects_enable_flag);
    checked_word!(game.hdma_objects_enable_flag = (flags & 0x7fff) | (u16::from(enabled) << 15));
}

pub(crate) fn clear_hdma_channels(game: &mut Game) {
    game.g_ram[0x85] = 0;
    word_transfers!(game, descending, 6; le(hdma_object_channels_bitmask, 5) <- value(0));
}

pub(crate) unsafe fn clear_hdma(game: &mut Game) {
    unsafe {
        crate::rtl::register_writes! { game;
            byte(crate::rtl::MDMAEN as u16, 0); byte(crate::rtl::HDMAEN as u16, 0);
        }
    }
    clear_hdma_channels(game);
}

pub(crate) unsafe fn initialize_room_effects(game: &mut Game) {
    use crate::ram_map::word;
    use crate::rtl::{WMADDH, WMADDL, WMADDM};
    word! { game, 0; earthquake_sfx_index, earthquake_sfx_timer }
    if matches!(
        word!(game.room_ptr),
        0x9804 | 0x96ba | 0xb32e | 0xb457 | 0xdd58 | 0xdede
    ) {
        word!(game.earthquake_sfx_timer = 0xffff);
    }
    word!(game.debug_disable_minimap = 0);
    for channel in (0x20..0x80u16).step_by(0x10) {
        for register in [0, 1, 2, 3, 4, 5, 6, 8, 9] {
            let value = if register == 1 { 0x13 } else { 0 };
            unsafe {
                crate::rtl::WriteReg(0x4300 + channel + register, value, game);
            }
        }
    }
    word! {game; fx_y_subpos = 0; fx_y_pos = -1i32 as u16; lava_acid_y_subpos = 0;
    lava_acid_y_pos = -1i32 as u16; }
    word! { game, 0;
        hud_bg3_xpos, hud_bg3_ypos, bg3_xpos, bg3_ypos, irq_enable_mode7, camera_distance_index,
        tourian_entrance_statue_animstate, tourian_entrance_statue_finished, earthquake_timer,
        phantom_related_layer_flag, power_bomb_explosion_status, power_bomb_flag,
        power_bomb_explosion_radius, fx_tilemap_ptr, fx_type,
    }
    word!(game.fx_base_y_subpos = 0x8000);
    word! { game, 0;
        fx_base_y_pos, fx_target_y_pos, fx_y_vel, fx_liquid_options, fx_timer, tide_phase,
        fx_y_suboffset, fx_y_offset,
    }
    word!(game.fx_layer_blending_config_a = 2);
    word! { game, 0;
        reg_BG3HOFS, reg_BG3VOFS, layer2_x_pos, layer2_y_pos, room_loading_irq_handler,
    }
    word_transfers! { game, ascending, 1;
        native(0x602) <- value(0x8800), native(0x605) <- value(0x8800),
        native(0x601) <- value(0x83e1), native(0x604) <- value(0x83e1),
    }
    unsafe {
        crate::rtl::register_writes! { game;
            byte(WMADDL as u16, 0xf0); byte(WMADDM as u16, 0xff);
            byte(WMADDH as u16, 1);
        }
    }
    game.g_ram[0x85] = 0;
    set_fixed_color([32, 64, 0x80], game);
    game.g_ram[0x57] = 0;
    game.g_ram[0x69] = 19;
    game.g_ram[0x6f] = 0;
    game.g_ram[0x72] = 0;
    game.g_ram[0x5d] = 0;
    game.g_ram[0x5e] = 4;
    game.g_ram[0x59] = 73;
    game.g_ram[0x5a] = 90;
    game.g_ram[0x5b] = 90;
}

pub(crate) unsafe fn update_hdma(game: &mut Game) {
    unsafe {
        crate::game::music::HandleMusicQueue(game);
    }
    if checked_word!(game.time_is_frozen_flag) == 0
        && checked_word!(game.power_bomb_explosion_status) & 0x4000 != 0
    {
        checked_word!(game.power_bomb_explosion_status = 0x8000);
        unsafe {
            spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x40, 0x28, 0x8ace), game);
            spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x40, 0x29, 0x8b80), game);
        }
    }
    checked_word!(game.fx_layer_blending_config_c = checked_word!(game.fx_layer_blending_config_a));
    if checked_word!(game.hdma_objects_enable_flag) & 0x8000 != 0 {
        game.g_ram[0x85] = 0;
        let mut slot = 0usize;
        loop {
            checked_word!(game.hdma_object_index = slot as u16);
            let channels = game.g_ram[0x18b4 + slot];
            if channels != 0 {
                game.g_ram[0x85] |= channels;
                unsafe {
                    run_hdma_program(slot as u8, game);
                }
                slot = usize::from(checked_word!(game.hdma_object_index));
            }
            slot += 2;
            if slot == 12 {
                break;
            }
        }
        unsafe {
            crate::game::layer_blending::update(game);
        }
    }
}

crate::game::address_dispatch! {
    unsafe fn run_hdma_call(ea: u32, k: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    {
        8972135 => mother_brain_visuals::initialize_rainbow_beam;
        8948500 => explosion_window::begin_pre_flash; 8948530 => explosion_window::begin_prescaled_pre_flash;
        8948537 => explosion_window::begin_explosion; 8948551 => explosion_window::begin_prescaled_explosion;
        8948558 => [explosion_window::finish_power_bomb(k, game)]; 8954596 => explosion_window::begin_crystal_flash;
        8954633 => explosion_window::begin_explosion; 8954647 => [explosion_window::finish_crystal_flash(k, game)];
        8970129 => [()]; 8970130 => [()]; 8972070 => [()];
        9554681 => [xray_background::freeze_and_backup(k, game)];
        9554716 => xray_background::capture_bg1_right;
        9554775 => xray_background::capture_bg1_left; 9554830 => xray_background::build_overlay;
        9556179 => xray_background::capture_bg2_right; 9556339 => xray_background::upload_left; 9556384 => xray_background::upload_right;
        9556668 => xray_background::set_backdrop; 9557650 => [Runtime::new(game).initialize_suit_pickup_hdma()];
    }
}
crate::game::address_dispatch! {
    unsafe fn run_hdma_pre_action(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        8491739 => [()]; 8946872 => [()]; 10785852 => [()]; 9556607 => xray_background::setup_blending;
        8947439 => [Runtime::new(game).advance_xray_background()]; 8947506 => [Runtime::new(game).advance_xray_beam(BeamPhase::Initial)];
        8947540 => [Runtime::new(game).advance_xray_beam(BeamPhase::Widening)];
        8947627 => [Runtime::new(game).advance_xray_beam(BeamPhase::Full)];
        8948020 => restore_left;
        8948154 => restore_right;
        8948232 => finish_restore;
        8948623 => explosion_window::set_power_bomb_blending;
        8948632 => explosion_window::power_bomb_afterglow;
        8949225 => [explosion_window::update_scaled(k, Phase::PowerBomb, game)];
        8949426 => [explosion_window::update_prescaled(k, Phase::PowerBomb, game)];
        8949983 => [explosion_window::update_scaled(k, Phase::PreFlash, game)];
        8950184 => [explosion_window::update_prescaled(k, Phase::PreFlash, game)];
        8954681 => explosion_window::set_crystal_flash_blending;
        8954717 => explosion_window::crystal_flash_afterglow;
        8955218 => [explosion_window::update_scaled(k, Phase::CrystalFlash, game)];
        8955459 => scrolling_sky::update_vertical_scroll; 8955507 => scrolling_sky::update_horizontal_scroll;
        8957362 => scrolling_sky::update_parallax; 8957370 => scrolling_sky::update_parallax;
        8957378 => scrolling_sky::advance_parallax;
        8958140 => [advance_fireflea_lighting(game)];
        8958896 => liquid_effects::lava_surface;
        8959189 => liquid_effects::lava_background; 8963214 => liquid_effects::water_surface;
        8963465 => liquid_effects::water_background;
        8963556 => liquid_effects::water_wave;
        8963638 => liquid_effects::water_flat;
        8968609 => [scroll_weather(k, Weather::Rain, game)];
        8968775 => [scroll_weather(k, Weather::Spores, game)];
        8969014 => [scroll_weather(k, Weather::Fog, game)];
        8969175 => [advance_tourian_statue(k, TourianStatuePhase::Waiting, game)]; 8969251 => [advance_tourian_statue(k, TourianStatuePhase::Shaking, game)];
        8969321 => [advance_tourian_statue(k, TourianStatuePhase::Sinking, game)]; 8969402 => [advance_tourian_statue(k, TourianStatuePhase::Finished, game)];
        8969539 => [word!(game.fx_layer_blending_config_c = 44)];
        8969744 => [Runtime::new(game).begin_haze(k, 0x80)];
        8969749 => [Runtime::new(game).begin_haze(k, 0x20)];
        8969773 => [Runtime::new(game).advance_haze(k, HazePhase::FadingIn)];
        8969844 => [Runtime::new(game).advance_haze(k, HazePhase::FadedIn)];
        8969878 => [Runtime::new(game).advance_haze(k, HazePhase::FadingOut)];
        8970132 => [Runtime::new(game).select_draygon_layer(k)]; 8970278 => [Runtime::new(game).advance_suit_pickup(false)];
        8970332 => [Runtime::new(game).advance_suit_pickup(true)]; 8971337 => [Runtime::new(game).select_phantoon_layer()];
        8971623 => [Runtime::new(game).advance_hdma_wave(k, HdmaWave::Phantoon)]; 8972220 => [mother_brain_visuals::advance_rainbow_beam(game)];
        0x88e9e6 => [Runtime::new(game).advance_eye_beam(k, EyeBeamPhase::Widening)];
        0x88ea3c => [Runtime::new(game).advance_eye_beam(k, EyeBeamPhase::Full)];
        0x88eacb => [Runtime::new(game).advance_eye_beam(k, EyeBeamPhase::Fading)];
        8973232 => [advance_title_gradient(true, game)];
        8973266 => [advance_title_gradient(false, game)];
        8973341 => [advance_intro_crossfade(game)]; 8972070 => [()];
        8973494 => [Runtime::new(game).advance_hdma_wave(k, HdmaWave::Intro)];
    }
}

unsafe fn run_hdma_program(slot: u8, game: &mut Game) {
    use crate::game::enemy_script::{EnemyScript as Script, bank_base};
    let index = usize::from(slot >> 1);
    let pre_instruction = (u32::from(checked_word!(game.hdma_object_pre_instruction_bank[index]))
        << 16)
        | u32::from(checked_word!(game.hdma_object_pre_instructions[index]));
    unsafe {
        run_hdma_pre_action(pre_instruction, u16::from(slot), game);
    }
    let timer = checked_word!(game.hdma_object_instruction_timers[index]);
    checked_word!(game.hdma_object_instruction_timers[index] = timer.wrapping_sub(1));
    if timer != 1 {
        return;
    }
    let base = bank_base(game.g_ram[0x18c1 + usize::from(slot)]);
    let image = game.capture_native_rom();
    let program = checked_word!(game.hdma_object_instruction_list_pointers[index]);
    let position = base + program as isize;
    let mut script = Script::at(
        image.clone(),
        usize::try_from(position).expect("HDMA program outside ROM"),
    );
    while script.word(0) & 0x8000 != 0 {
        let opcode = u32::from(script.word(0)) | 0x880000;
        let next =
            unsafe { execute_hdma_instruction(opcode, u16::from(slot), script.advance(2), game) };
        let Some(next) = next.resolve(&image, base) else {
            return;
        };
        script = next;
    }
    checked_word!(game.hdma_object_instruction_timers[index] = script.word(0));
    checked_word!(game.hdma_object_table_pointers[index] = script.word(2));
    checked_word!(
        game.hdma_object_instruction_list_pointers[index] =
            script.advance(4).offset_from(base) as u16
    );
}

unsafe fn execute_hdma_instruction(
    opcode: u32,
    slot: u16,
    script: crate::game::enemy_script::EnemyScript,
    game: &mut Game,
) -> crate::game::enemy_script::EnemyScript {
    use crate::game::enemy_script::{EnemyScript as Script, bank_base};
    let index = usize::from(slot >> 1);
    match opcode {
        8969006 => {}
        8947049 => {
            checked_word!(game.hdma_object_channels_bitmask[index] = 0);
            return Script::Stop;
        }
        8947056 => {
            checked_word!(game.hdma_object_pre_instructions[index] = script.word(0));
            game.g_ram[0x18fc + usize::from(slot)] = script.byte(2);
            return script.advance(3);
        }
        8947076 => checked_word!(game.hdma_object_pre_instructions[index] = 0x858a),
        8947124 => {
            unsafe {
                run_hdma_call(script.load24(0), slot, game);
            }
            return script.advance(3);
        }
        8947180 | 8947185 | 8947207 | 8947215 => {
            let relative = matches!(opcode, 8947185 | 8947215);
            if matches!(opcode, 8947207 | 8947215) {
                let timer = checked_word!(game.hdma_object_timers[index]);
                checked_word!(game.hdma_object_timers[index] = timer.wrapping_sub(1));
                if timer == 1 {
                    return script.advance(if relative { 1 } else { 2 });
                }
            }
            return if relative {
                script.advance(script.byte(0) as i8 as isize)
            } else {
                Script::jump(script.word(0))
            };
        }
        8947222 => {
            game.g_ram[0x1908 + usize::from(slot)] = script.byte(0);
            return script.advance(1);
        }
        8947234 | 8947255 | 8947285 | 8947306 => {
            let register = match opcode {
                8947234 => crate::rtl::DMAP0,
                8947255 => crate::rtl::BBAD0,
                8947285 => crate::rtl::A1B0,
                _ => crate::rtl::DAS00,
            } as u16;
            let channel = u16::from(game.g_ram[0x18c0 + 2 * index]);
            unsafe {
                crate::rtl::WriteReg(register + channel, script.byte(0), game);
            }
            return script.advance(1);
        }
        8947276 => {
            checked_word!(game.hdma_object_table_pointers[index] = script.word(0));
            return script.advance(2);
        }
        8947330 => {
            let base = bank_base(game.g_ram[0x18c1 + usize::from(slot)]);
            // Sleep refreshes the bank/source after callbacks, retaining linear position.
            let _binding = game.capture_native_rom_binding();
            checked_word!(
                game.hdma_object_instruction_list_pointers[index] =
                    (script.offset_from(base) - 2) as u16
            );
            return Script::Stop;
        }
        8955500 | 8959182 | 8963175 | 8963458 => checked_word!(game.hdma_object_B[index] = 1),
        8958889 => checked_word!(game.hdma_object_C[index] = 112),
        8968470 | 8968521 => checked_word!(game.hdma_data_table_in_ceres = 9),
        8968577 => {
            checked_word!(
                game.hdma_object_D[index] = crate::rtl::rom_table_word!(
                    game,
                    0x88d992,
                    ((checked_word!(game.random_number) >> 1 & 6) >> 1) as isize
                )
            );
        }
        8969419 => {
            checked_word!(game.hdma_object_C[index] = 0);
            checked_word!(game.hdma_object_A[index] = 0);
            let opened = crate::game::gameplay_flag_runtime::CheckEventHappened(0xa, game) != 0;
            let offset: u16 = if opened { 0xff10 } else { 0 };
            checked_word!(game.hdma_object_B[index] = offset);
            let [low, high] = offset.to_ne_bytes();
            game.g_ram[0x9e00] = low;
            game.g_ram[0x9e01] = high;
            if opened {
                return Script::jump(script.word(0));
            }
            let [low, high] = 1u16.to_ne_bytes();
            game.g_ram[0xcd20] = low;
            game.g_ram[0xcd21] = high;
            return script.advance(2);
        }
        8971453 => Runtime::new(game).initialize_phantoon_wave(slot),
        0x88e917 => Runtime::new(game).initialize_eye_beam(slot),
        8973471 => Runtime::new(game).reset_hdma_wave(slot),
        8973215 => {
            game.g_ram[0x6f] = 0;
            refresh_title_gradient(game);
        }
        _ => {
            crate::rtl::Unreachable(game);
            return Script::Stop;
        }
    }
    script
}

impl visual_host::Record for RomRecord {
    fn byte(&self, displacement: usize) -> u8 {
        RomRecord::byte(self, displacement)
    }
    fn native_word(&self, displacement: usize) -> u16 {
        RomRecord::word(self, displacement)
    }
    fn bytes<const N: usize>(&self, displacement: usize) -> [u8; N] {
        let start = self
            .base
            .checked_add(displacement)
            .expect("ROM copy offset overflow");
        let end = start.checked_add(N).expect("ROM copy extent overflow");
        assert!(end <= self.image.len(), "ROM copy source extent");
        core::array::from_fn(|index| self.image.read(start + index))
    }
}
impl visual_host::Content for Runtime<'_> {
    type Record = RomRecord;
    fn record(&mut self, source: Source, bank: u8, address: u16) -> RomRecord {
        match source {
            Source::Loaded => player_projectiles::host::Host::record(self, bank, address),
            Source::Native => player_projectiles::host::Host::native_record(self, bank, address),
        }
    }
    fn legacy_native_record(&mut self, bank: u8, address: u16) -> RomRecord {
        Runtime::legacy_native_record(self, bank, address)
    }
    fn table(&self, source: Source, address: u32) -> RomRecord {
        let image = match source {
            Source::Loaded => self.loaded_image().expect("ROM must be loaded"),
            Source::Native => self.native_image(),
        };
        player_projectiles::content::captured(image, address)
    }
}
fn palette_address(member: PaletteMember, handle: u16) -> usize {
    let base = match member {
        PaletteMember::Definition => 0x1e7d,
        PaletteMember::ColorOffset => 0x1e8d,
        PaletteMember::PreAction => 0x1ead,
        PaletteMember::Program => 0x1ebd,
        PaletteMember::Delay => 0x1ecd,
        PaletteMember::Counter => 0x1edd,
    };
    base + usize::from(handle & !1)
}
fn animation_address(member: AnimationMember, handle: u16) -> usize {
    let base = match member {
        AnimationMember::Definition => 0x1ef5,
        AnimationMember::Counter => 0x1f01,
        AnimationMember::Program => 0x1f0d,
        AnimationMember::Delay => 0x1f19,
        AnimationMember::Source => 0x1f25,
    };
    base + usize::from(handle & !1)
}
impl visual_host::Aliases for Runtime<'_> {
    fn palette_word(&self, member: PaletteMember, handle: u16) -> u16 {
        self.word(palette_address(member, handle))
    }
    fn set_palette_word_slice(&mut self, member: PaletteMember, handle: u16, value: u16) {
        self.write_slice_le(palette_address(member, handle), value);
    }
    fn set_palette_counter_byte(&mut self, handle: u16, value: u8) {
        self.set_byte(0x1edd + usize::from(handle), value);
    }
    fn palette_flag_native(&self, displacement: u16) -> u16 {
        self.native_word(0x1e79 + usize::from(displacement))
    }
    fn animation_word(&self, member: AnimationMember, handle: u16) -> u16 {
        self.word(animation_address(member, handle))
    }
    fn set_animation_word_prefix(&mut self, member: AnimationMember, handle: u16, value: u16) {
        self.write_word_le(value, animation_address(member, handle));
    }
    fn set_animation_counter_byte(&mut self, handle: u16, value: u8) {
        self.set_byte(0x1f01 + usize::from(handle), value);
    }
    fn set_palette_color_slice(&mut self, displacement: u16, value: u16) {
        self.write_slice_le(0xc000 + usize::from(displacement & !1), value);
    }
    fn set_color_prefix(&mut self, buffer: PaletteBuffer, index: usize, value: u16) {
        let base: usize = match buffer {
            PaletteBuffer::Current => 0xc000,
            PaletteBuffer::Target => 0xc200,
        };
        let at = base
            .checked_add(index.checked_mul(2).expect("palette index overflow"))
            .expect("palette address overflow");
        self.write_word_le(value, at);
    }
}
impl visual_host::Effects for Runtime<'_> {
    fn draw_samus_sprite(&mut self, index: u16, x: u16, y: u16) {
        self.foreign(|game| sprite_output::samus(index, (x, y), game, &mut sprite_output::Native));
    }
    fn draw_atmospheric_tile(&mut self, slot: usize, kind: u16) {
        use visual_host::{Content, Record};
        let frame_offset =
            2 * u16::from(self.session().atmospheric.slots[slot].frame_and_kind as u8);
        let saved = self.word(0x590);
        let x = self.session().atmospheric.slots[slot]
            .x
            .wrapping_sub(self.word(0x911))
            .wrapping_sub(4);
        if x as i16 >= 0
            && self.session().atmospheric.slots[slot]
                .x
                .wrapping_sub(self.word(0x911))
                .wrapping_sub(260)
                & 0x8000
                != 0
        {
            let entry = 0x370 + usize::from(self.word(0x590));
            self.set_byte(entry, x as u8);
            // X may alias the y coordinate, camera, cursor or another owned field.
            let y = self.session().atmospheric.slots[slot]
                .y
                .wrapping_sub(self.word(0x915))
                .wrapping_sub(4);
            if y as i16 >= 0
                && self.session().atmospheric.slots[slot]
                    .y
                    .wrapping_sub(self.word(0x915))
                    .wrapping_sub(260)
                    & 0x8000
                    != 0
            {
                self.set_byte(entry + 1, y as u8);
                let pointer = self
                    .table(Source::Loaded, 0x908bff)
                    .native_word(2 * usize::from(kind));
                let tile = self
                    .legacy_native_record(0x90, pointer.wrapping_add(frame_offset))
                    .native_word(0);
                self.write_slice_native(entry + 2, tile);
                self.write_word_le(saved.wrapping_add(4), 0x590);
                crate::game::session_access::SessionAccess::probe(self, "atmosphere.tile.written");
            }
        }
    }

    fn sound(&mut self, channel: u8, limit: u8, sound: u16) {
        player_projectiles::host::Host::sound(self, channel, limit, sound);
    }
    fn music_delayed8(&mut self, track: u16) {
        self.foreign(|game| crate::game::music::QueueMusic_Delayed8(track, game));
    }
    fn resume_movement_sound(&mut self) {
        #[cfg(sm_verify_dll)]
        samus_observer::event("command", 0x1c);
        self.foreign(|game| {
            crate::game::session_runtime::player_command(0x1c, game);
        });
    }
    fn upload(&mut self, request: Upload) {
        self.foreign(|game| animated_tiles::native::upload(request, game));
    }
    fn invalid_palette_instruction(&mut self) {
        self.foreign(|game| {
            crate::rtl::Unreachable(game);
        });
    }
    fn invalid_animation_instruction(&mut self) -> u16 {
        self.foreign(animated_tiles::native::unreachable)
    }
    fn color_written(&mut self, _index: u16, _value: u16) {}
}
impl visual_host::Composition for Runtime<'_> {
    fn spawn_palette(&mut self, definition: u16) {
        palette_effects::execution::spawn(self, definition);
    }
    fn spawn_statue_projectile(&mut self, kind: StatueProjectile, parameter: u16) {
        use crate::game::constants::bank_86::{
            addr_kEproj_TourianStatueEyeGlow, addr_kEproj_TourianStatueSoul,
        };
        let definition = match kind {
            StatueProjectile::EyeGlow => addr_kEproj_TourianStatueEyeGlow,
            StatueProjectile::Soul => addr_kEproj_TourianStatueSoul,
        };
        enemy_projectiles::lifecycle::spawn_room(self, definition as u16, parameter);
    }
}
impl visual_host::Host for Runtime<'_> {}

#[forbid(unsafe_code)]
fn queue_cannon_tiles(runtime: &mut Runtime<'_>, tiles: player_projectiles::cannon::Tiles) {
    use player_projectiles::cannon::Tiles;
    use visual_host::Content;
    let (size, destination) = match tiles {
        Tiles::Beam(_) => (256, 0x6300),
        Tiles::Cannon(_) => (32, 0x61f0),
    };
    let saved = runtime.word(0x330);
    runtime.write_slice_native(0xd0 + usize::from(saved), size);
    // Size publication precedes source acquisition and every source read.
    let source = match tiles {
        Tiles::Beam(index) => runtime
            .table(Source::Loaded, 0x90c3b1)
            .word(usize::from(index) * 2),
        Tiles::Cannon(pointer) => runtime.legacy_native_record(0x90, pointer).word(0),
    };
    runtime.write_slice_native(0xd0 + usize::from(saved.wrapping_add(2)), source);
    runtime.set_byte(0xd0 + usize::from(saved.wrapping_add(4)), 0x9a);
    runtime.write_slice_native(0xd0 + usize::from(saved.wrapping_add(5)), destination);
    runtime.write_word_le(saved.wrapping_add(7), 0x330);
}

#[forbid(unsafe_code)]
fn cannon_tile(runtime: &mut Runtime<'_>, x: u16, y: u16, body_y: u16, character: u16) {
    let saved = runtime.word(0x590);
    let x = x
        .wrapping_add(runtime.session().player.x)
        .wrapping_sub(runtime.word(0x911));
    if x < 256 {
        let entry = 0x370 + usize::from(runtime.word(0x590));
        runtime.set_byte(entry, x as u8);
        let y = y
            .wrapping_add(runtime.session().player.y)
            .wrapping_sub(body_y)
            .wrapping_sub(runtime.word(0x915));
        if y < 256 {
            runtime.set_byte(entry + 1, y as u8);
            runtime.write_slice_native(entry + 2, character);
            runtime.write_word_le(saved.wrapping_add(4), 0x590);
        }
    }
}

pub(crate) fn update_arm_cannon(game: &mut Game) {
    player_projectiles::cannon::update(&mut Runtime::new(game));
}
pub(crate) fn draw_player_with_cannon(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    // Layer order is captured before atmospheric output can alias the mode word.
    let mode = runtime.session().projectiles.controls.cannon_drawing_mode & 0xf;
    crate::game::player_transitions::draw_atmospheric(&mut runtime);
    if mode != 0 && mode != 2 {
        player_projectiles::cannon::draw(&mut runtime, cannon_tile, queue_cannon_tiles);
    }
    runtime.foreign(crate::game::samus_animation::Samus_Draw);
    if mode == 2 {
        player_projectiles::cannon::draw(&mut runtime, cannon_tile, queue_cannon_tiles);
    }
}
pub(crate) fn update_beam_tiles_and_palette(game: &mut Game) {
    player_projectiles::cannon::update_beam(&mut Runtime::new(game), queue_cannon_tiles);
}
#[path = "../verification/animated_runtime_checks.rs"]
mod animation_guard_checks;
#[path = "../verification/palette_runtime_checks.rs"]
mod palette_guard_checks;
pub(crate) fn check_palette_guard(game: &mut Game) -> bool {
    palette_guard_checks::run(game)
}
pub(crate) fn check_animation_guard(game: &mut Game) -> bool {
    animation_guard_checks::run(game)
}
#[path = "../verification/samus_palette_observer.rs"]
pub(crate) mod samus_observer;

pub(crate) const METROID_PAL_1: [u16; 16] = [
    0x1000, 0x6bf5, 0x2e41, 0x2da1, 0x2d01, 0x5e5f, 0x183f, 0x1014, 0x80a, 0x404, 0x4f9f, 0x3ed8,
    0x2e12, 0x6f70, 0x7fff, 0x5ee0,
];
pub(crate) const METROID_PAL_2: [u16; 16] = [
    0x3800, 0x6bf5, 0x6e1, 0x641, 0x5a1, 0x5e5f, 0x183f, 0x1014, 0x80a, 0x404, 0x4f9f, 0x3ed8,
    0x2e12, 0x6f70, 0x7fff, 0x5ee0,
];
pub(crate) const METROID_PAL_3: [u16; 16] = [
    0x3800, 0x77f8, 0x1344, 0x12a4, 0x1204, 0x6abf, 0x249f, 0x1c77, 0x146d, 0x1067, 0x5bff, 0x4b38,
    0x3a72, 0x7bd3, 0x7fff, 0x6b43,
];
pub(crate) const METROID_PAL_4: [u16; 16] = [
    0x3800, 0x7ffb, 0x1fa7, 0x1f07, 0x1e67, 0x771f, 0x30ff, 0x28da, 0x20d0, 0x1cca, 0x67ff, 0x579b,
    0x46d5, 0x7bd6, 0x7fff, 0x77a6,
];

pub(crate) fn title_metroid_cry(sound: u16, cursor: u16, game: &mut Game) -> u16 {
    crate::game::music::QueueSfx3_Max6(sound, game);
    cursor
}

pub(crate) fn title_metroid_palette(palette: [u16; 16], cursor: u16, game: &mut Game) -> u16 {
    word_transfers!(game, descending, 16; le(palette_buffer, 63) <- table(palette, 15));
    cursor
}

fn refresh_title_gradient(game: &mut Game) {
    let mut cursor = crate::rtl::rom_table_word!(
        game,
        0x8cbc5d,
        ((word!(game.cinematic_var6) & 0xf0) >> 4) as isize
    );
    for slot in 0usize.. {
        let value = cinematic_native_record::<0x8c>(cursor, game).word(0);
        word!(game.hdma_table_1[slot] = value);
        if value == 0 {
            break;
        }
        cursor = cursor.wrapping_add(2);
    }
}

pub(crate) fn advance_title_gradient(backdrop: bool, game: &mut Game) {
    if backdrop {
        refresh_title_gradient(game);
    }
    if checked_word!(game.cinematic_function) == 0x9b68 {
        Runtime::new(game).advance_hdma_program();
    }
}

pub(crate) fn advance_intro_crossfade(game: &mut Game) {
    if checked_word!(game.eproj_x_pos[0]) == 1 {
        Runtime::new(game).advance_hdma_program();
    }
}

const FIREFLEA_FLASH: [u16; 12] = [
    0, 0x100, 0x200, 0x300, 0x400, 0x500, 0x600, 0x500, 0x400, 0x300, 0x200, 0x100,
];
const FIREFLEA_DARKNESS: [u16; 6] = [0, 0x600, 0xc00, 0x1200, 0x1800, 0x1900];

pub(crate) fn darken_fireflea_room(game: &mut Game) {
    if word!(game.fireflea_darkness_level).wrapping_sub(12) & 0x8000 != 0 {
        word!(game.fireflea_darkness_level += 2);
    }
}

pub(crate) unsafe fn initialize_fireflea_lighting(game: &mut Game) {
    word! {game; fireflea_flashing_timer = 6; fireflea_flashing_index = 0; }
    unsafe {
        spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x42, 0x11, 0xb0ac), game);
    }
    word_transfers!(game, ascending, 1; native(0x9e00) <- value(0));
    word! { game, 0; UNUSED_word_7E177C, fireflea_darkness_level }
    word!(game.UNUSED_word_7E1780 = 24);
    word!(game.UNUSED_word_7E1782 = FIREFLEA_FLASH[0]);
}

pub(crate) fn advance_fireflea_lighting(game: &mut Game) {
    word!(game.fx_layer_blending_config_c = word!(game.fx_layer_blending_config_c) & 0xff00 | 0xc);
    if word!(game.time_is_frozen_flag) != 0 {
        return;
    }
    if word!(game.fireflea_flashing_timer -= 1, reread) == 0 {
        word!(game.fireflea_flashing_timer = 6);
        let next = if word!(game.fireflea_darkness_level).wrapping_sub(10) & 0x8000 != 0 {
            let next = word!(game.fireflea_flashing_index).wrapping_add(1);
            if next >= 12 { 0 } else { next }
        } else {
            6
        };
        word!(game.fireflea_flashing_index = next);
    }
    let shade = (FIREFLEA_DARKNESS[usize::from(word!(game.fireflea_darkness_level) >> 1)]
        .wrapping_add(FIREFLEA_FLASH[usize::from(word!(game.fireflea_flashing_index))])
        >> 8) as u8;
    game.g_ram[0x75] = shade | 0x80;
    game.g_ram[0x76] = shade | 0x40;
    game.g_ram[0x74] = shade | 0x20;
}

// Shooting-star scratch records stay in RAM. Native scalar loads use from_le
// on checked LE reads; only the original compound updates use LE arithmetic.
pub(crate) fn initialize_shooting_stars(game: &mut Game) {
    for index in 0..40 {
        let at = 0xe0c + 16 * index;
        word_transfers!(game, ascending, 1;
            native(at) <- value(index as u16),
            native(at + 12) <- value(0),
            native(at + 14) <- value(0),
        );
        let delay = crate::rtl::rom_table_word!(game, 0x8be9cf, (4 * index + 3) as isize);
        word_transfers!(game, ascending, 1; native(at + 10) <- value(if delay == 0 { 32 } else { delay }));
        if delay != 0 {
            checked_word!(game, at; = checked_word!(game, at;) | 0x8000);
        }
        for (offset, value) in [(2, 128), (6, 128), (4, 0), (8, 0)] {
            word_transfers!(game, ascending, 1; native(at + offset) <- value(value));
        }
    }
}

pub(crate) fn advance_shooting_stars(game: &mut Game) {
    if word!(game.eproj_pre_instr[1]) == 0 {
        return;
    }
    for at in (0xe0c..0x108c).step_by(16) {
        let identity = u16::from_le(checked_word!(game, at;));
        if (identity as i16) < 0 {
            let timer = u16::from_le(checked_word!(game, at + 10;)).wrapping_sub(1);
            word_transfers!(game, ascending, 1; native(at + 10) <- value(timer));
            if (timer as i16) < 0 {
                word_transfers!(game, ascending, 1; native(at + 10) <- value(32));
                word_transfers!(game, ascending, 1; native(at) <- value(u16::from(game.g_ram[at])));
            }
            continue;
        }
        let table = 4 * isize::from(identity as u8);
        let fast = ((i32::from(identity & 0xff00) - 1024) & 0x8000) == 0;
        checked_word!(game, at + 12; += {
            let acceleration = crate::rtl::rom_table_word!(game, 0x8be9cf, table);
            if fast {
                acceleration.wrapping_add(crate::rtl::rom_table_word!(game, 0x8be9cf, table))
            } else {
                acceleration
            }
        });
        let y_speed = if fast {
            crate::rtl::rom_table_word!(game, 0x8be9cf, table + 1)
                .wrapping_add(u16::from_le(checked_word!(game, at + 14;)))
        } else {
            u16::from_le(checked_word!(game, at + 14;))
        };
        word_transfers!(game, ascending, 1; native(at + 14) <- value(
            crate::rtl::rom_table_word!(game, 0x8be9cf, table + 1).wrapping_add(y_speed)
        ));
        for (position, fraction, velocity) in [(2, 4, 12), (6, 8, 14)] {
            let speed = u16::from_le(checked_word!(game, at + velocity;));
            let whole = speed.to_ne_bytes()[1] as i8 as u16;
            let (fractional, carry) =
                (speed << 8).overflowing_add(u16::from_le(checked_word!(game, at + fraction;)));
            word_transfers!(game, ascending, 1; native(at + fraction) <- value(fractional));
            checked_word!(game, at + position; += whole.wrapping_add(u16::from(carry)));
        }
    }
    let mut output = word!(game.oam_next_ptr);
    for at in (0xe0c..0x108c).step_by(16) {
        if (u16::from_le(checked_word!(game, at;)) as i16) < 0 {
            continue;
        }
        let x = u16::from_le(checked_word!(game, at + 2;)) as i16;
        let oam = 0x370 + usize::from(output);
        let mut y = 0i16;
        if game.g_ram[at + 3] != 0
            || (i32::from(x) - 4) & 0xff00 != 0
            || {
                // This overlapping word write precedes even the Y rejection.
                word_transfers!(game, ascending, 1; native(oam) <- value((x as u16).wrapping_sub(4)));
                y = u16::from_le(checked_word!(game, at + 6;)) as i16;
                game.g_ram[at + 7] != 0
            }
            || (i32::from(y) - 4) & 0xff00 != 0
        {
            for (offset, value) in [
                (10, 32),
                (2, 128),
                (6, 128),
                (4, 0),
                (8, 0),
                (12, 0),
                (14, 0),
            ] {
                word_transfers!(game, ascending, 1; native(at + offset) <- value(value));
            }
            word_transfers!(game, ascending, 1; native(at) <- value(u16::from(game.g_ram[at])));
            continue;
        }
        word_transfers!(game, ascending, 1; native(oam + 1) <- value((y as u16).wrapping_sub(4)));
        let timer = u16::from_le(checked_word!(game, at + 10;)).wrapping_sub(1);
        word_transfers!(game, ascending, 1; native(at + 10) <- value(timer));
        if timer as i16 <= 0 {
            let identity = u16::from_le(checked_word!(game, at;));
            word_transfers!(game, ascending, 1; native(at + 10) <- value(
                crate::rtl::rom_table_word!(game, 0x8be9cf,
                    4 * isize::from(u16::from_le(checked_word!(game, at;)) as u8) + 2)
            ));
            word_transfers!(game, ascending, 1; native(at) <- value(identity.wrapping_add(512)));
        }
        if game.g_ram[at + 1] != 0 {
            word_transfers!(game, ascending, 1; native(oam + 2) <- value(
                crate::rtl::rom_table_word!(game, 0x8be9a7,
                    ((u16::from_le(checked_word!(game, at;)) & 0xff00) >> 9) as isize)
            ));
            output = output.wrapping_add(4);
        }
    }
    word!(game.oam_next_ptr = output);
}

pub(crate) fn shake_room_with_sound(kind: u16, game: &mut Game) {
    static SOUNDS: [u16; 17] = [
        0x46, 1, 0x46, 3, 0x46, 2, 0x46, 1, 0x46, 1, 0x46, 2, 0x46, 2, 0x46, 1, 0x8000,
    ];
    if word!(game.earthquake_sfx_timer) & 0x8000 == 0
        && word!(game.earthquake_sfx_timer -= 1, reread) & 0x8000 != 0
    {
        let mut index = word!(game.earthquake_sfx_index);
        let mut sound = SOUNDS[usize::from(index >> 1)];
        if sound & 0x8000 != 0 {
            index = 0;
            sound = SOUNDS[0];
        }
        crate::game::music::QueueSfx2_Max6(sound, game);
        word!(
            game.earthquake_sfx_timer = SOUNDS[usize::from(index >> 1) + 1]
                .wrapping_add(index & 1)
                .wrapping_add(word!(game.random_number) & 3)
        );
        word!(game.earthquake_sfx_index = index.wrapping_add(4));
    }
    word!(game.earthquake_type = kind);
    word!(game.earthquake_timer |= 0x20);
}

pub(crate) unsafe fn initialize_tourian_statue(game: &mut Game) {
    unsafe {
        if CheckEventHappened(0xa, game) & 1 != 0 {
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(6, 12, 0xb777), game);
            word_transfers!(game, ascending, 1; native(0xcd20) <- value(514));
        }
        game.g_ram[0x59] = 74;
        word!(game.fx_rising_function_bank_88 = 0xc428);
        word!(game.fx_y_pos = word!(game.fx_base_y_pos));
        spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x42, 0x11, 0xd856), game);
        spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x42, 0x10, 0xdcfa), game);
        spawn_bg3_scroll(game);
    }
}

pub(crate) unsafe fn advance_tourian_statue(
    mut slot: u16,
    phase: TourianStatuePhase,
    game: &mut Game,
) {
    use TourianStatuePhase::*;
    if matches!(phase, Shaking | Sinking) {
        shake_room_with_sound(13, game);
    }
    match phase {
        Waiting => {
            if (6..=9).all(|event| CheckEventHappened(event, game) & 1 != 0) {
                word!(game.tourian_entrance_statue_animstate |= 0x10);
                if word!(game.tourian_entrance_statue_animstate) & 0x8000 == 0 {
                    word!(game.hdma_object_C[(slot >> 1)] = 300);
                    slot = word!(game.hdma_object_index);
                    advance_statue_program(word!(game.hdma_object_index), game);
                }
            }
        }
        Shaking => {
            let counter_slot = slot as u8;
            if word!(game.hdma_object_C[(counter_slot >> 1)] -= 1, reread) & 0x8000 != 0 {
                for _ in 0..4 {
                    SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueDustClouds as u16, 0, game);
                }
                advance_statue_program(u16::from(counter_slot), game);
            }
        }
        Sinking => {
            if word!(game.time_is_frozen_flag) != 0 {
                return;
            }
            let low = 0x1914 + usize::from(slot & !1);
            let high = 0x1920 + usize::from(slot & !1);
            let fraction = u16::from_ne_bytes([game.g_ram[low], game.g_ram[low + 1]]);
            let whole = u16::from_ne_bytes([game.g_ram[high], game.g_ram[high + 1]]);
            let position = (u32::from(fraction) | u32::from(whole) << 16).wrapping_sub(0x4000);
            word_transfers!(game, ascending, 1;
                native(low) <- value(position as u16),
                native(high) <- value((position >> 16) as u16),
            );
            if word!(game.hdma_object_B[(slot >> 1)]) == 0xff10 {
                unsafe {
                    spawn_hardcoded(SpawnHardcodedPlmArgs::new(6, 12, 0xb773), game);
                }
                SetEventHappened(0xa, game);
                advance_statue_program(slot, game);
            }
        }
        Finished => {
            word!(game.tourian_entrance_statue_finished = 0x8000);
            word_transfers!(game, ascending, 1; native(0xcd20) <- value(514));
        }
    }
    let scroll = word!(game.layer1_y_pos).wrapping_add(word!(game.hdma_object_B[(slot >> 1)]));
    word_transfers!(game, ascending, 1; native(0x9e00) <- value(scroll));
}

fn advance_statue_program(slot: u16, game: &mut Game) {
    word!(game.hdma_object_instruction_timers[(slot >> 1)] = 1);
    word!(game.hdma_object_instruction_list_pointers[(slot >> 1)] += 2);
}

pub(crate) unsafe fn initialize_weather(weather: Weather, game: &mut Game) {
    use crate::game::constants::bank_87::{addr_kAnimtiles_Rain, addr_kAnimtiles_Spores};
    let (control, program, tiles) = match weather {
        Weather::Rain => (0x43, 0xd96c, Some(addr_kAnimtiles_Rain)),
        Weather::Spores => (0x42, 0xda2d, Some(addr_kAnimtiles_Spores)),
        Weather::Fog => (0x43, 0xdb19, None),
    };
    game.g_ram[0x5b] = 92;
    unsafe {
        spawn_hdma(
            0x88,
            &SpawnHdmaObject_Args::new(control, 0x11, program),
            game,
        );
        if let Some(tiles) = tiles {
            spawn_bg3_scroll(game);
            animated_tiles::spawn(tiles as u16, game);
        }
    }
}

pub(crate) fn scroll_weather(slot: u16, weather: Weather, game: &mut Game) {
    use crate::ram_map::{checked_word, words as ram};
    game.g_ram[0x5b] = 92;
    checked_word!(game.fx_layer_blending_config_c = checked_word!(game.fx_layer_blending_config_b));
    if checked_word!(game.time_is_frozen_flag) != 0 {
        return;
    }
    let runtime = &mut Runtime::new(game);
    let object = usize::from(slot as u8 & !1);
    let phase_base = if matches!(weather, Weather::Spores) {
        0x192c
    } else {
        0x1914
    };
    let rain = matches!(weather, Weather::Rain);
    // Finish each axis, including rain's camera history, before reading the next.
    for (axis, camera, output) in [
        (0, ram::layer1_y_pos, ram::bg3_ypos),
        (12, ram::layer1_x_pos, ram::bg3_xpos),
    ] {
        let phase = phase_base + object + axis;
        let history = 0x1944 + object + axis;
        let origin = if rain {
            runtime.word(history).wrapping_sub(runtime.word(camera))
        } else {
            runtime.word(camera)
        };
        let scroll = origin.wrapping_add(runtime.byte(phase + 1) as i8 as u16);
        runtime.write_slice_le(output, scroll);
        let previous_phase = runtime.word(phase);
        let delta = match (weather, axis) {
            (Weather::Rain, 0) => 0u16.wrapping_sub(1536),
            (_, 0) => 0u16.wrapping_sub(64),
            (Weather::Rain, _) => runtime.word(0x1938 + object),
            (Weather::Fog, _) => 80,
            // Spores retain the original idempotent horizontal-phase store.
            (Weather::Spores, _) => 0,
        };
        runtime.write_slice_le(phase, previous_phase.wrapping_add(delta));
        if rain {
            runtime.write_slice_le(history, runtime.word(camera));
        }
    }
}

// Keep bits 16..23 of the magnitude product before restoring the sine sign.
fn hdma_wave_offset(sample: i16, amplitude_low: u8, amplitude_high: u8) -> u16 {
    let amplitude = u16::from_le_bytes([amplitude_low, amplitude_high]);
    let product = u32::from(sample.unsigned_abs()) * u32::from(amplitude);
    let offset = (product >> 16) as u8 as u16;
    if sample < 0 {
        offset.wrapping_neg()
    } else {
        offset
    }
}

impl Runtime<'_> {
    pub(crate) fn initialize_wave_pair(&mut self, base: usize, primary: u16) {
        let source = (0xc00000 | u32::from(primary)).to_ne_bytes();
        self.write_slice_le(base, 192);
        self.write_slice_le(base + 6, 192);
        for at in [base + 1, base + 7] {
            // Keep high-half, low-word, alternate-pointer publication order.
            self.write_slice(at + 2, &source[2..]);
            self.write_slice_native(at, primary);
            self.write_slice_le(at + 3, primary.wrapping_add(128));
        }
        self.write_slice_le(base + 12, 0);
    }

    pub(crate) fn reset_hdma_wave(&mut self, slot: u16) {
        let object = usize::from(slot & !1);
        for (field, value) in [(0x1914, 0xfffe), (0x1920, 1), (0x192c, 0), (0x1938, 0)] {
            self.write_slice_le(field + object, value);
        }
    }

    pub(crate) fn initialize_phantoon_wave(&mut self, slot: u16) {
        use crate::game::actors::access::AccessMode::Native;
        let motion = Key::from_legacy(0x80);
        let controller = Key::from_legacy(0x40);
        let mode = self.resolve_actor(motion, Native).parameters[0];
        self.edit_actor(controller, Native, |actor| actor.parameters[0] = mode);
        if self.resolve_actor(motion, Native).parameters[0] & 1 != 0 {
            self.initialize_wave_pair(0x9000, 0x9100);
        } else {
            for at in [0x9000, 0x9006, 0x900c, 0x900f, 0x9012, 0x9015] {
                self.write_slice_le(at, 160);
            }
            for at in [0x9001, 0x9007] {
                self.write_slice(at + 2, &0xa09100u32.to_ne_bytes()[2..]);
                self.write_slice_native(at, 0x9100);
            }
            for at in [0x900d, 0x9013] {
                self.write_slice_native(at, 0x9100);
            }
            for at in [0x9004, 0x900a] {
                self.write_slice_le(at, 0x9140);
            }
            for at in [0x9010, 0x9016] {
                self.write_slice_native(at, 0x9140);
            }
            self.write_slice_le(0x9018, 0);
        }
        self.reset_hdma_wave(slot);
    }

    pub(crate) fn select_phantoon_layer(&mut self) {
        use crate::game::actors::access::AccessMode::Native;
        use crate::ram_map::words as ram;
        if self.word(ram::phantom_related_layer_flag) & 0x4000 != 0 {
            self.write_slice_le(ram::fx_layer_blending_config_c, 26);
        } else {
            // The native first byte belongs to EnemyData record 3, parameter 1.
            let alpha = self
                .resolve_actor(Key::from_legacy(0xc0), Native)
                .parameters[0]
                .to_ne_bytes()[0];
            match alpha {
                0 => self.write_slice_le(ram::fx_layer_blending_config_c, 4),
                0xff => {
                    self.write_slice_le(ram::fx_layer_blending_config_c, 4);
                    let object = usize::from(self.word(ram::hdma_object_index) & !1);
                    self.write_slice_le(
                        0x18cc + object,
                        self.word(0x18cc + object).wrapping_add(2),
                    );
                    self.write_slice_le(0x18e4 + object, 1);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn select_draygon_layer(&mut self, slot: u16) {
        use crate::game::actors::access::AccessMode::Native;
        let actor = self.resolve_actor(Key::from_legacy(0), Native);
        let x = i32::from(actor.x.whole) - i32::from(self.inputs().camera_x);
        let y = i32::from(actor.y.whole) - i32::from(self.inputs().camera_y);
        let visible = actor.properties & 0x200 == 0
            && (x + 64) as i16 >= 0
            && (x - 320) & 0x8000 != 0
            && (y + 16) as i16 >= 0
            && (y - 304) & 0x8000 != 0;
        let program = if !visible {
            0xdf71
        } else if (i32::from(y as i16) - 40) & 0x8000 != 0 {
            0xdf6b
        } else if (i32::from(y as i16) - 192) & 0x8000 != 0 {
            0xdf5f
        } else {
            0xdf65
        };
        let object = usize::from(slot & !1);
        self.write_slice_le(0x18e4 + object, 1);
        self.write_slice_le(0x18cc + object, program);
    }

    pub(crate) fn advance_hdma_wave(&mut self, slot: u16, wave: HdmaWave) {
        use crate::game::actors::access::AccessMode::Native;
        use crate::game::player_projectiles::access::Store;
        use crate::game::projectile_geometry::SINE_COSINE;
        use crate::ram_map::{word_arrays as arrays, words as ram};
        let controller = Key::from_legacy(0x40);
        let motion = Key::from_legacy(0xc0);
        let phantoon = matches!(wave, HdmaWave::Phantoon);
        let mode = if phantoon {
            self.resolve_actor(controller, Native).parameters[0]
        } else {
            self.word(0xd9c)
        };
        if mode == 0 {
            self.advance_hdma_program();
            return;
        }
        let step = if phantoon && mode & 1 == 0 { 8 } else { 4 };
        let object = usize::from(slot & !1);
        let advance = if phantoon {
            let delay = self.word(0x1920 + object);
            self.write_slice_le(0x1920 + object, delay.wrapping_sub(1));
            if delay == 1 {
                self.write_slice_le(0x1920 + object, 1);
            }
            delay == 1
        } else {
            true
        };
        if advance {
            let speed = if phantoon {
                self.resolve_actor(motion, Native).pre_instruction
            } else {
                self.word(ram::loop_counter_transfer_enemies_to_vram)
            };
            let phase = speed
                .wrapping_mul(2)
                .wrapping_add(self.word(0x1914 + object))
                & 0x1ff;
            self.write_slice_le(0x1914 + object, phase);
        }
        let mut phase = self.word(0x1914 + object);
        // Cinematic BG3 data is separate from Phantoon's BG2 table.
        let (scroll, store, table) = if phantoon {
            (ram::reg_BG2HOFS, Store::SliceNative, ram::g_word_7E9100)
        } else {
            (ram::reg_BG3HOFS, Store::SliceLe, arrays::hdma_table_1)
        };
        for row in 0..256 / step {
            let sample = SINE_COSINE[usize::from(phase >> 1) + 64];
            let (low, high) = if phantoon {
                let amplitude = self.resolve_actor(motion, Native).variables[3];
                (amplitude as u8, amplitude.to_ne_bytes()[1])
            } else {
                (self.byte(0xd9e), self.byte(0xd9f))
            };
            let offset = hdma_wave_offset(sample, low, high);
            phase = phase.wrapping_add(step) & 0x1ff;
            let value = self.word(scroll).wrapping_add(offset);
            self.store(table + 2 * usize::from(row), value, store);
        }
        // Phantoon samples its mode again after the first half is published.
        let narrow = phantoon && self.resolve_actor(controller, Native).parameters[0] & 1 == 0;
        let rows = if narrow { 32 } else { 64 };
        for row in (0..rows).rev() {
            let origin = self.word(scroll).wrapping_add(self.word(scroll));
            let sample = if phantoon {
                self.native_word(table + 2 * row)
            } else {
                self.word(table + 2 * row)
            };
            let value = origin.wrapping_sub(sample);
            self.store(table + 2 * (rows + row), value, store);
        }
    }

    pub(crate) fn begin_haze(&mut self, slot: u16, color: u16) {
        let object = usize::from(slot as u8 & !1);
        self.write_slice_le(0x1920 + object, color);
        self.write_slice_le(0x1914 + object, 0);
        if self.session().room.entry.transition_selector == 0xe737 {
            self.write_slice_le(0x18f0 + object, 0xde2d);
            self.advance_haze(slot, HazePhase::FadingIn);
        }
    }

    pub(crate) fn advance_haze(&mut self, slot: u16, phase: HazePhase) {
        self.write_slice(0x74, &[0x20, 0x40, 0x80]);
        self.write_slice_le(0x1986, 44);
        let byte_slot = usize::from(slot as u8);
        let object = byte_slot & !1;
        let level = 0x1914 + object;
        match phase {
            HazePhase::FadedIn => {
                if self.session().room.entry.transition_selector == 0xe2db {
                    self.write_slice_le(0x18f0 + object, 0xde96);
                }
                return;
            }
            HazePhase::FadingIn if self.word(level) == 16 => {
                self.write_slice_le(0x18f0 + object, 0xde74);
                return;
            }
            HazePhase::FadingOut if self.word(level) == 0 => return,
            _ => {}
        }
        // Color samples use the byte slot even when its word slot is rounded down.
        let color = self.byte(0x1920 + byte_slot);
        let mut shade = self.byte(0x1914 + byte_slot) as i8;
        for row in (0..16).rev() {
            self.set_byte(0x9d00 + row, color | shade as u8);
            shade = shade.wrapping_sub(1);
            if shade < 0 {
                shade = 0;
            }
        }
        let delta = if matches!(phase, HazePhase::FadingIn) {
            1
        } else {
            u16::MAX
        };
        self.write_slice_le(level, self.word(level).wrapping_add(delta));
    }

    fn advance_hdma_program(&mut self) {
        let current = usize::from(self.word(0x18b2) & !1);
        let program = 0x18cc + current;
        self.write_slice_le(program, self.word(program).wrapping_add(2));
        self.write_slice_le(0x18e4 + current, 1);
    }

    fn add_eye_beam_fixed(&mut self, whole: usize, fraction: usize, increment: u32) -> u32 {
        let value = (u32::from(self.word(fraction)) | u32::from(self.word(whole)) << 16)
            .wrapping_add(increment);
        self.write_slice_le(fraction, value as u16);
        self.write_slice_le(whole, (value >> 16) as u16);
        value
    }

    pub(crate) fn initialize_eye_beam(&mut self, slot: u16) {
        for (color, fixed, value) in [
            (0x1914, 0x74, 0x30),
            (0x1915, 0x75, 0x50),
            (0x1920, 0x76, 0x80),
        ] {
            self.set_byte(color + usize::from(slot), value);
            self.set_byte(fixed, value);
        }
        self.write_slice_le(0x1986, 16);
        // The original stores also clear the byte after the HDMA terminator.
        self.write_slice(
            0x9000,
            &[0xe4, 0x00, 0x91, 0xe4, 0xc8, 0x91, 0x98, 0x90, 0x92, 0, 0],
        );
        self.edit_actor(Key::from_legacy(0x40), Le, |actor| actor.variables[2] = 1);
        let even_slot = usize::from(slot & !1);
        self.write_slice_le(0x192c + even_slot, 0);
        self.write_slice_le(0x1938 + even_slot, 0);
        self.write_slice(0x9080, &[0; 4]);
        self.write_slice_le(0x9090, 0);
    }

    pub(crate) fn advance_eye_beam(&mut self, slot: u16, phase: EyeBeamPhase) {
        use visual_host::Content;
        self.write_slice_le(0x1986, 16);
        let even_slot = usize::from(slot & !1);
        match phase {
            EyeBeamPhase::Widening => {
                let speed = self.add_eye_beam_fixed(0x9080, 0x9082, 0x4000);
                let width = self.add_eye_beam_fixed(0x192c + even_slot, 0x1938 + even_slot, speed);
                if ((width >> 16) as u16).wrapping_sub(4) as i16 >= 0 {
                    // Keep the fractional width when the integer part reaches the limit.
                    self.write_slice_le(0x192c + even_slot, 4);
                    self.advance_hdma_program();
                }
                self.draw_eye_beam(slot);
            }
            EyeBeamPhase::Full => {
                if self.resolve_actor(Key::from_legacy(0x40), Le).variables[2] == 0 {
                    self.advance_hdma_program();
                } else {
                    self.draw_eye_beam(slot);
                    let index = usize::from(self.word(0x9090).wrapping_mul(4));
                    let colors = self.table(Source::Loaded, 0x88ea8b);
                    for (lane, target) in [0x1914, 0x1915, 0x1920].into_iter().enumerate() {
                        self.set_byte(target + usize::from(slot), colors.byte(index + lane));
                    }
                    // Color destinations can overlap the index, so advance its live value.
                    self.write_slice_le(0x9090, self.word(0x9090).wrapping_add(1) & 0xf);
                }
            }
            EyeBeamPhase::Fading => {
                if self.byte(0x1915 + usize::from(slot)) == 0x40 {
                    self.write_slice(0x74, &[0x20, 0x40, 0x80]);
                    self.write_slice_le(0x1938 + even_slot, 0);
                    self.write_slice_le(0x192c + even_slot, 0);
                    for entry in (0x9100..0x9300).step_by(2).rev() {
                        self.write_slice_le(entry, 0xff);
                    }
                    self.write_slice(0x9000, &[0; 10]);
                    self.advance_hdma_program();
                } else {
                    self.draw_eye_beam(slot);
                    let current = usize::from(self.word(0x18b2));
                    // Colors use byte slots, including odd current object indices.
                    for (color, black) in [(0x1914, 0x20), (0x1915, 0x40), (0x1920, 0x80)] {
                        let value = self.byte(color + current);
                        if value != black {
                            self.set_byte(color + current, value.wrapping_sub(1));
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn draw_eye_beam(&mut self, slot: u16) {
        let actor = self.resolve_actor(Key::from_legacy(0x40), Le);
        let angle = actor.variables[3];
        let width = self.word(0x192c + usize::from(slot & !1));
        let y = actor.y.whole.wrapping_sub(self.inputs().camera_y);
        let x = actor.x.whole.wrapping_sub(self.inputs().camera_x);
        self.foreign(|game| {
            crate::game::beam_windows::calculate_xray_window(
                x,
                y,
                angle,
                width,
                x >= 256,
                0x9100,
                game,
            )
        });
        // Byte-indexed colors can alias the generated table or an imported owner.
        for (target, source) in [(0x74, 0x1914), (0x75, 0x1915), (0x76, 0x1920)] {
            self.set_byte(target, self.byte(source + usize::from(slot)));
        }
    }
}

pub(crate) fn draw_player_frame(game: &mut Game) {
    use crate::game::{player_transitions::state::DrawMode, samus_animation as body};
    update_arm_cannon(game);
    let h = &mut Runtime::new(game);
    match h.session().player.transitions.draw_handler {
        DrawMode::Normal | DrawMode::Grapple => {
            let mode = h.session().player.transitions.draw_handler;
            if mode == DrawMode::Normal {
                h.foreign(player_projectiles::flare);
            }
            let grapple = h.session().player.grapple;
            if mode == DrawMode::Normal || (grapple.wrapping_add(0x37aa) as i16) >= 0 {
                h.foreign(draw_player_with_cannon);
                h.foreign(body::Samus_DrawEchoes);
            } else {
                if matches!(grapple, 0xc703 | 0xc759) {
                    h.foreign(|game| unsafe { crate::game::sm_9b::GrappleBeamFunc_BF1B(game) });
                }
                h.foreign(player_projectiles::grapple_flare);
                h.foreign(draw_player_with_cannon);
                h.foreign(crate::game::sm_9b::UpdateGrappleBeamTilesAndIncrFlameCtr);
                if h.session().player.grapple_length != 0 {
                    h.foreign(crate::game::grapple_swing::draw_rope);
                }
            }
        }
        DrawMode::ShinesparkEnd => {
            h.foreign(body::Samus_Draw);
            for index in [2, 0] {
                h.foreign(|game| body::Samus_DrawShinesparkCrashEchoes(index, game));
            }
        }
        DrawMode::Frozen => {
            h.foreign(body::Samus_DrawWhenNotAnimatingOrDying);
        }
        DrawMode::Elevator if h.inputs().frame & 1 == 0 => {
            h.foreign(body::Samus_DrawWhenNotAnimatingOrDying);
        }
        DrawMode::Disabled | DrawMode::Empty | DrawMode::Elevator => {}
        DrawMode::Unknown(_) => {
            h.foreign(|game| crate::rtl::Unreachable(game));
        }
    }
    h.foreign(player_projectiles::draw_projectiles);
    h.owned_call(crate::game::session_runtime::finish_player_frame);
}

struct CinematicObjectLayout {
    slots: u16,
    current: usize,
    instructions: usize,
    parameter: usize,
    cleared: &'static [usize],
}

impl CinematicObjects {
    fn layout(self) -> CinematicObjectLayout {
        use crate::ram_map::{word_arrays as a, words as w};
        match self {
            Self::Sprites => CinematicObjectLayout {
                slots: 16,
                current: w::cinematic_obj_index,
                instructions: a::cinematicspr_instr_ptr,
                parameter: w::cinematic_spawn_param,
                cleared: &[
                    a::cinematicspr_whattodraw,
                    a::cinematicspr_goto_timer,
                    a::cinematicspr_arr6,
                    a::cinematicspr_arr7,
                ],
            },
            Self::Mode7 => CinematicObjectLayout {
                slots: 2,
                current: w::mode7_cur_index,
                instructions: a::mode7_obj_instr_ptr,
                parameter: w::mode7_spawn_param,
                cleared: &[a::mode7_obj_goto_timer],
            },
            Self::Background => CinematicObjectLayout {
                slots: 4,
                current: w::cinematicbg_var2,
                instructions: a::cinematicbg_instr_ptr,
                parameter: w::cinematicbg_var3,
                cleared: &[a::cinematicbg_arr1, a::cinematicbg_arr6],
            },
        }
    }

    pub(crate) unsafe fn spawn(self, definition: u16, parameter: u16, game: &mut Game) -> u8 {
        let layout = self.layout();
        checked_word!(game, layout.parameter; = parameter);
        for slot in (0..layout.slots).rev() {
            if checked_word!(game, layout.instructions + 2 * usize::from(slot);) == 0 {
                return unsafe { initialize_cinematic_object(self, definition, slot * 2, game) };
            }
        }
        1
    }

    pub(crate) unsafe fn advance(self, game: &mut Game) {
        use crate::game::visual_state::{CinematicObjects::Background, CinematicTilemap};
        if matches!(self, Background) && word!(game.cinematic_enable_objs) & 0x8000 == 0 {
            return;
        }
        let layout = self.layout();
        let mut object = i32::from(2 * (layout.slots - 1));
        while object >= 0 {
            checked_word!(game, layout.current; = object as u16);
            if checked_word!(game, layout.instructions + 2 * (object as usize >> 1);) != 0 {
                unsafe {
                    advance_cinematic_object(self, object as u16, game);
                }
                // A callback may redirect the scan outside its original slot range.
                object = i32::from(checked_word!(game, layout.current;));
            }
            object -= 2;
        }
        if matches!(self, Background) {
            if word!(game.cinematic_enable_bg_tilemap) & 0x8000 != 0 {
                queue_cinematic_tilemap(CinematicTilemap::Text, game);
            }
            queue_cinematic_tilemap(CinematicTilemap::Eyes, game);
        }
    }
}

const RINKA_POSITIONS: [(u16, u16); 4] = [(112, 72), (192, 56), (128, 48), (232, 80)];
const RINKA_X_SPEED: [u32; 4] = [0x8000, 0xffff8000, 0x8000, 0xffff8000];

pub(crate) fn initialize_intro_rinka(object: u16, game: &mut Game) {
    let variant = usize::from(word!(game.cinematic_spawn_param));
    let slot = usize::from(object >> 1);
    word!(game.cinematicspr_goto_timer[slot] = word!(game.cinematic_spawn_param));
    place_cinematic_sprite(object, RINKA_POSITIONS[variant], 3584, game);
}

pub(crate) fn start_intro_rinka(object: u16, cursor: u16, game: &mut Game) -> u16 {
    let misses_samus = word!(game.cinematicspr_goto_timer[(object >> 1)]) != 0;
    word!(
        game.cinematicspr_preinstr_func[(object >> 1)] = if misses_samus { 0xb93b } else { 0xb8d8 }
    );
    cursor
}

pub(crate) fn chase_intro_baby_y(object: u16, y_offset: i32, game: &mut Game) {
    let slot = usize::from(object >> 1);
    if ((word!(game.samus_y_pos) as i32 - y_offset - word!(game.cinematicbg_arr8[slot]) as i32)
        as i16)
        < 0
    {
        if word!(game.cinematic_var17) as i32 + 543 & 0x8000 == 0 {
            word!(game.cinematic_var17, wrapping_sub(32));
        }
    } else if word!(game.cinematic_var17) as i32 - 544 & 0x8000 != 0 {
        word!(game.cinematic_var17, wrapping_add(32));
    }
    let velocity = ((word!(game.cinematic_var17) as i16 as i32) << 8) as u32;
    advance_cinematic_coordinate(object, true, velocity, game);
}

pub(crate) fn advance_intro_baby_pursuit(object: u16, game: &mut Game) {
    let slot = usize::from(object >> 1);
    if word!(game.cinematic_var13) == 0 {
        word! {game; cinematicspr_instr_timer[slot] = 1;
        cinematicspr_instr_ptr[slot] = 0xce53; cinematic_var15 = 0; }
        return;
    }
    if word!(game.cinematicspr_goto_timer[slot]) as i32 - 128 & 0x8000 != 0 {
        let timer = word!(game.cinematicspr_goto_timer[slot]).wrapping_add(1);
        word!(game.cinematicspr_goto_timer[slot] = timer);
        if timer & 0x3f == 0 {
            crate::game::music::QueueSfx3_Max6(0x23, game);
        }
    }
    if (word!(game.samus_x_pos) as i16).wrapping_sub(word!(game.cinematicbg_arr7[slot]) as i16) < 0
    {
        if word!(game.cinematic_var14) as i32 + 639 & 0x8000 == 0 {
            word!(game.cinematic_var14 -= 32);
        }
    } else if word!(game.cinematic_var14) as i32 - 640 & 0x8000 != 0 {
        word!(game.cinematic_var14 += 32);
    }
    word!(
        game.cinematic_var15 = if word!(game.cinematic_var14) & 0x8000 != 0 {
            1
        } else {
            u16::MAX
        }
    );
    let velocity = ((word!(game.cinematic_var14) as i16 as i32) << 8) as u32;
    advance_cinematic_coordinate(object, false, velocity, game);
    chase_intro_baby_y(object, 8, game);
}

pub(crate) fn advance_intro_rinka(object: u16, misses_samus: bool, game: &mut Game) {
    let slot = usize::from(object >> 1);
    let horizontal = if misses_samus {
        RINKA_X_SPEED[usize::from(word!(game.cinematicspr_goto_timer[slot]))]
    } else {
        0x8000
    };
    advance_cinematic_coordinate(object, false, horizontal, game);
    advance_cinematic_coordinate(object, true, 0x8000, game);
    if misses_samus {
        if !(word!(game.cinematicbg_arr8[slot]) as i32 - 16 & 0x8000 != 0
            || word!(game.cinematicbg_arr8[slot]) as i32 - 208 & 0x8000 == 0
            || word!(game.cinematicspr_preinstr_func[0]) == 0xb80f)
        {
            return;
        }
    } else if (word!(game.cinematicbg_arr7[slot]) as i32 + 8 - (word!(game.samus_x_pos) as i32 - 5))
        & 0x8000
        != 0
    {
        if word!(game.cinematicspr_preinstr_func[0]) != 0xb80f {
            return;
        }
    } else {
        word! { game, 11; samus_invincibility_timer, samus_knockback_timer }
        word!(game.knockback_x_dir = 1);
    }
    word!(game.cinematicspr_instr_timer[slot] = 1);
    word!(game.cinematicspr_instr_ptr[slot] = 0xce53);
}

// Native split words retain low-before-high reads and writes.
fn add_native_pair(high: usize, low: usize, velocity: u32, game: &mut Game) {
    let fraction = u16::from_ne_bytes([game.g_ram[low], game.g_ram[low + 1]]);
    let whole = u16::from_ne_bytes([game.g_ram[high], game.g_ram[high + 1]]);
    let position = (u32::from(fraction) | (u32::from(whole) << 16)).wrapping_add(velocity);
    word_transfers!(game, ascending, 1;
        native(low) <- value(position as u16),
        native(high) <- value((position >> 16) as u16),
    );
}

fn advance_cinematic_coordinate(object: u16, vertical: bool, velocity: u32, game: &mut Game) {
    let offset = usize::from(object & !1) + if vertical { 0x20 } else { 0 };
    add_native_pair(0x1a7d + offset, 0x1add + offset, velocity, game);
}

pub(crate) fn initialize_intro_brain_particle(object: u16, big: bool, game: &mut Game) {
    let parameter = usize::from(word!(game.cinematic_spawn_param));
    let slot = usize::from(object >> 1);
    word!(game.cinematicspr_goto_timer[slot] = word!(game.cinematic_spawn_param));
    let particles: &[(u16, u16, u16)] = if big {
        &[
            (56, 111, 1),
            (72, 95, 16),
            (40, 119, 32),
            (48, 95, 48),
            (64, 119, 64),
        ]
    } else {
        &[(72, 111, 1), (40, 115, 8), (40, 103, 16)]
    };
    word! { game;
        cinematicbg_arr7[slot] = particles[parameter].0;
        cinematicbg_arr8[slot] = particles[parameter].1;
        cinematicspr_instr_timer[slot] = particles[parameter].2;
        cinematicbg_arr9[slot] = 2560;
    }
}

pub(crate) fn initialize_ceres_explosion(object: u16, pattern: CeresExplosion, game: &mut Game) {
    let slot = usize::from(object >> 1);
    let parameter = if matches!(pattern, CeresExplosion::Final) {
        0
    } else {
        let parameter = usize::from(word!(game.cinematic_spawn_param));
        word!(game.cinematicspr_goto_timer[slot] = word!(game.cinematic_spawn_param));
        parameter
    };
    let (x, y, delays): (&[i16], &[i16], &[u16]) = match pattern {
        CeresExplosion::First => (
            &[16, -16, 16, -16, 0],
            &[-16, 16, 16, -16, 0],
            &[1, 16, 32, 48, 64],
        ),
        CeresExplosion::Second => (
            &[14, 8, -16, -8, 0, 16, -12, -8],
            &[-8, 12, 12, -14, 0, 14, 4, -16],
            &[],
        ),
        CeresExplosion::Third => (&[8, 12, -8, -12], &[-4, 8, -10, 12], &[1, 4, 8, 16]),
        CeresExplosion::Final => (&[0], &[0], &[]),
    };
    if !delays.is_empty() {
        word!(game.cinematicspr_instr_timer[slot] = delays[parameter]);
    }
    // Keep the Y lookup and camera reads after the X store for aliased slots.
    word! { game;
        cinematicbg_arr7[slot] = (x[parameter] as i32 + word!(game.reg_M7X) as i32
            - word!(game.cinematic_var8) as i32) as u16;
        cinematicbg_arr8[slot] = (y[parameter] as i32 + word!(game.reg_M7Y) as i32
            - word!(game.cinematic_var10) as i32) as u16;
        cinematicbg_arr9[slot] = 2560;
    }
}

pub(crate) fn advance_ceres_station(object: u16, game: &mut Game) {
    if word!(game.cinematic_function) == 0xbdf9 {
        let slot = usize::from(object >> 1);
        word!(game.cinematicspr_goto_timer[slot] += 128);
        let velocity = ((word!(game.cinematicspr_goto_timer[slot]) as i16 as i32) << 8) as u32;
        advance_cinematic_coordinate(object, true, velocity, game);
        advance_cinematic_coordinate(object, false, velocity, game);
        for low in [ram_words::cinematic_var7, ram_words::cinematic_var9] {
            add_native_pair(low + 2, low, velocity, game);
        }
    }
}

pub(crate) fn advance_zebes_flight(stage: u8, game: &mut Game) {
    if stage == 0 {
        word! {game; cinematic_var8 = 62; cinematic_var10 = -112i16 as u16;
        cinematic_var5 = 32; cinematic_var6 = 16; cinematic_function = 0xc9f9; }
        return;
    }
    let (horizontal, threshold, limit, increment, next) = [
        (0xffff8000, ram_words::cinematic_var6, 1152, 4, 0xca36),
        (0xffff8000, ram_words::cinematic_var8, -128, 16, 0xca85),
        (0x2000, ram_words::cinematic_var6, 0x2000, 32, 0xcad0),
    ][usize::from(stage - 1)];
    add_native_pair(0x1997, 0x1995, 0x2000, game);
    add_native_pair(0x1993, 0x1991, horizontal, game);
    let negative = (word!(@read game, threshold) as i32 - limit) & 0x8000 != 0;
    if negative == (stage == 2) {
        if stage == 3 {
            game.g_ram[0x69] = 16;
        }
        word!(game.cinematic_function = next);
        if stage == 3 {
            word!(game.cinematic_var4 = 64);
        }
    } else {
        word!(game.cinematic_var6 += increment);
        if stage == 2 && word!(game.cinematic_var5) != 224 {
            word!(game.cinematic_var5 = (word!(game.cinematic_var5) as u8).wrapping_sub(1) as u16);
        }
    }
}

pub(crate) fn advance_zebes_destruction_zoom(
    speed: u16,
    next_scene: u16,
    fade_in: Option<u16>,
    game: &mut Game,
) {
    if word!(game.cinematic_var12) & 1 == 0 {
        word!(game.cinematic_var5 = (word!(game.cinematic_var5) as u8).wrapping_sub(1) as u16);
    }
    word!(game.cinematic_var6 += speed);
    if word!(game.cinematic_var6) as i32 - 384 & 0x8000 == 0
        && advance_cinematic_fade(CinematicFade::FastOut, game)
    {
        word!(game.cinematic_function = next_scene);
    }
    if let Some(next) = fade_in {
        // Both fades may run this frame; the later transition wins.
        if advance_cinematic_fade(CinematicFade::FastIn, game) {
            word!(game.cinematic_function = next);
        }
    }
}

pub(crate) fn advance_zebes_stars(object: u16, moving: bool, game: &mut Game) {
    let slot = usize::from(object >> 1);
    if moving {
        let offset = 2 * slot;
        add_native_pair(0x1b7d + offset, 0x1afd + offset, -32i32 as u32, game);
        let velocity = (u32::from(word!(game.cinematicspr_goto_timer[slot])) << 16)
            | u32::from(word!(game.cinematicspr_arr7[slot]));
        advance_cinematic_coordinate(object, false, velocity, game);
        wait_cinematic_scene(object, 0xde63, None, game);
    } else if word!(game.cinematic_function) == 0xdca5 {
        word!(game.cinematicspr_preinstr_func[slot] = 0xf375);
        word!(game.cinematicspr_arr7[slot] = 0x4000);
        word!(game.cinematicspr_goto_timer[slot] = 0);
    }
}

pub(crate) fn drift_cinematic_sprite(
    object: u16,
    horizontal: u32,
    vertical: Option<u32>,
    wrap_horizontal: bool,
    game: &mut Game,
) {
    if let Some(velocity) = vertical {
        advance_cinematic_coordinate(object, true, velocity, game);
    }
    advance_cinematic_coordinate(object, false, horizontal, game);
    if wrap_horizontal {
        word!(game.cinematicbg_arr7[(object >> 1)] &= 0x1ff);
    }
}

pub(crate) fn initialize_ending_sprite_pair(object: u16, leftward: bool, game: &mut Game) {
    let position = if leftward { (530, 231) } else { (246, 519) };
    place_cinematic_sprite(object, position, 3584, game);
    word!(game.cinematicspr_goto_timer[(object >> 1)] = 8);
}

pub(crate) fn advance_ending_sprite_pair(slot: usize, leftward: bool, game: &mut Game) -> bool {
    let (direction, threshold, stop_x, stop_y) =
        [(1, 395, 394, 367), (-1, 382, 382, 383)][usize::from(!leftward)];
    let step = (i32::from(word!(game.cinematicspr_goto_timer[slot])) * direction) as u16;
    word!(game.cinematicbg_arr8[slot] += step);
    let x = word!(game.cinematicbg_arr7[slot]).wrapping_sub(step);
    word!(game.cinematicbg_arr7[slot] = x);
    let arrived = (i32::from(x) - threshold & 0x8000 != 0) == leftward;
    if arrived {
        word! {game; cinematicbg_arr7[slot] = stop_x; cinematicbg_arr8[slot] = stop_y;
        cinematicspr_preinstr_func[slot] = 0x93d9; }
    } else {
        word!(game.cinematicspr_goto_timer[slot] += 2);
    }
    arrived
}

pub(crate) fn update_ending_critters(object: u16, initialize: bool, game: &mut Game) {
    let slot = usize::from(object >> 1);
    if initialize {
        word! { game, 128; cinematicbg_arr8[slot], cinematicbg_arr7[slot] }
        word! {game; cinematicbg_arr9[slot] = 3584; cinematicspr_goto_timer[slot] = 260; }
        return;
    }
    advance_cinematic_coordinate(object, true, 128, game);
    word!(game.cinematicspr_arr6[slot] = word!(game.cinematicspr_arr6[slot]));
    if word!(game.cinematicbg_arr7[slot] += 1, reread) as i32 - 272 & 0x8000 == 0 {
        word! {game; cinematicspr_instr_timer[slot] = 1; cinematicspr_instr_ptr[slot] = 0xee9b; }
    }
}

pub(crate) fn fade_ending_gunship_backdrop(game: &mut Game) {
    let expired = word!(game.cinematic_var4 -= 1, reread) & 0x8000 != 0;
    if word!(game.cinematic_var4) == 0 || expired {
        fade_cinematic_fixed_color(game);
        if game.g_ram[0x74] == 32 && game.g_ram[0x75] == 64 && game.g_ram[0x76] == 0x80 {
            game.g_ram[0x6f] = 0;
            game.g_ram[0x72] = 0;
        }
        word!(game.cinematic_var4 = 8);
    }
}

pub(crate) fn advance_ending_gunship_spin(slow: bool, game: &mut Game) -> bool {
    if slow || word!(game.cinematic_var13) == 0 {
        fade_ending_gunship_backdrop(game);
    } else {
        word!(game.cinematic_var13 -= 1);
    }
    let (rotation, mask) = if slow { (1, 7) } else { (4, 15) };
    if !slow || word!(game.cinematic_var5) != 224 {
        word!(
            game.cinematic_var5 =
                u16::from((word!(game.cinematic_var5) as u8).wrapping_sub(rotation))
        );
    }
    let index = usize::from(word!(game.cinematic_var14));
    let velocity = if slow {
        [1i32, 1, 1, -1, -1, 1, 1, -1][index] * 0x10000
    } else {
        [1i32, 1, 1, 1, -1, -1, 1, 1, 1, 1, -1, -1, 1, 1, -1, -1][index] * 0x8000
    };
    let fraction = ram_words::cinematic_var7;
    add_native_pair(fraction + 2, fraction, velocity as u32, game);
    word!(game.cinematic_var14 = word!(game.cinematic_var14).wrapping_add(1) & mask);
    word!(game.cinematic_var6 -= if slow { 2 } else { 8 });
    if word!(game.cinematic_var6) as i32 - (if slow { 1184 } else { 1456 }) & 0x8000 == 0 {
        return false;
    }
    if slow {
        word! {game; cinematic_function = 0xddcd; cinematic_var14 = 0x8000; cinematic_var13 = 0; }
    } else {
        word! {game; cinematic_var13 = 1; cinematic_var14 = 0; cinematic_function = 0xdd42; }
    }
    true
}

pub(crate) fn advance_ending_samus_jump(object: u16, phase: u8, game: &mut Game) {
    if phase == 2 {
        advance_ending_graphics(EndingGraphics::Samus, game);
    }
    let low = ram_words::samus_y_subspeed;
    add_native_pair(low + 2, low, 0x3800, game);
    let velocity = words!(game.samus_y_speed, samus_y_subspeed);
    advance_cinematic_coordinate(object, true, velocity, game);
    let slot = usize::from(object >> 1);
    if phase == 2 {
        if word!(game.cinematicbg_arr8[slot]) as i32 - 136 & 0x8000 == 0 {
            word! {game; cinematicbg_arr8[slot] = 136; cinematicspr_instr_timer[slot] = 1;
            cinematicspr_instr_ptr[slot] = 0xed9d; cinematicspr_preinstr_func[slot] = 0x93d9; }
        }
    } else if word!(game.cinematicbg_arr8[slot]) as i32 + 80 & 0x8000 != 0 {
        if phase == 0 {
            game.g_ram[0x52] = 3;
        }
        word! {game; cinematicspr_instr_timer[slot] = 1;
        cinematicspr_instr_ptr[slot] = if phase == 0 { 0xed95 } else { 0xee9b }; }
        if phase == 0 {
            word! {game; cinematicspr_preinstr_func[slot] = 0xf5dd; cinematicbg_arr9[slot] = 3072; }
        }
    }
}

pub(crate) fn position_ending_samus_head(
    object: u16,
    jumping: bool,
    cursor: u16,
    game: &mut Game,
) -> u16 {
    let slot = usize::from(object >> 1) + 1;
    let under_ten = word!(game.game_time_hours) as i32 - 10 & 0x8000 != 0;
    let (x, y) = [[(118, 120), (120, 120)], [(120, 114), (121, 116)]][usize::from(jumping)]
        [usize::from(under_ten)];
    word! {game; cinematicbg_arr7[slot] = x; cinematicbg_arr8[slot] = y; }
    cursor
}

pub(crate) fn fade_postcredits_samus_suit(game: &mut Game) {
    if word!(game.cinematic_var4) & 3 == 0 {
        fade_cinematic_colors(0x40, 0x10, false, game);
        if word!(game.game_time_hours) as i32 - 3 & 0x8000 == 0 {
            fade_cinematic_colors(0x1c0, 0x10, true, game);
        }
        fade_cinematic_colors(0x1a0, 0x10, true, game);
        compose_cinematic_palette(game);
    }
}

pub(crate) fn advance_postcredits_samus_message(fade_suit: bool, game: &mut Game) {
    if fade_suit {
        fade_postcredits_samus_suit(game);
    }
    if word!(game.cinematic_var4 -= 1, reread) as i32 - 65 & 0x8000 != 0 {
        word_transfers!(game, descending, 288; native(0x347e) <- value(127));
        word_transfers!(game, ascending, 64; pause_map(384) <- rom(0x8cdedb, 0));
        queue_cinematic_tilemap(CinematicTilemap::FullText, game);
        set_cinematic_blend(1, 0, 0, 0, game);
        word! {game; cinematic_var13 = 180; cinematic_function = 0xe2dd; }
    }
}

pub(crate) fn advance_intro_mother_brain(object: u16, stage: u8, game: &mut Game) -> bool {
    let slot = usize::from(object >> 1);
    if stage != 2 {
        let timer = word!(game.cinematicspr_arr6[slot]);
        if timer != 0 {
            if timer & 1 != 0 {
                word_transfers!(game, descending, 16; native(0xc1fe) <- rom(0x8ce3e9, 255));
            } else {
                word_transfers!(game, descending, 16; native(0xc1fe) <- value(0x7fff));
            }
            word!(game.cinematicspr_arr6[slot] -= 1);
        }
    }
    if stage == 0 {
        let Some(projectile) = (0..5).rfind(|&i| word!(game.projectile_type[i]) & 0xfff == 256)
        else {
            return false;
        };
        if word!(game.projectile_x_pos[projectile]) as i32 - 84 & 0x8000 == 0 {
            return false;
        }
        crate::game::player_projectiles::retire(2 * projectile as u16, game);
        word!(game.cinematicspr_arr6[slot] = 8);
    } else {
        if word!(game.cinematic_var12) & 1 != 0 {
            word!(game.cinematic_var10 -= 4);
        } else {
            word!(game.cinematic_var10 += 4);
        }
        if stage == 2 {
            crate::game::intro_pages::wait_for_scene_end(object, true, game);
            return false;
        }
    }
    let count = word!(game.cinematicspr_goto_timer[slot]).wrapping_add(1);
    word!(game.cinematicspr_goto_timer[slot] = count);
    if stage == 0 && count == 4 {
        word! {game; cinematicspr_goto_timer[slot] = 0; cinematicspr_preinstr_func[slot] = 0xb80f; }
        return true;
    }
    if stage == 1 && count as i32 - 128 & 0x8000 == 0 {
        word! {game; cinematicspr_instr_timer[slot] = 1; cinematicspr_instr_ptr[slot] = 0xcb19; }
    }
    false
}

pub(crate) fn wait_cinematic_scene(
    object: u16,
    scene: u16,
    next_preinstruction: Option<u16>,
    game: &mut Game,
) {
    if word!(game.cinematic_function) == scene {
        let slot = usize::from(object >> 1);
        if let Some(next) = next_preinstruction {
            word!(game.cinematicspr_preinstr_func[slot] = next);
        } else {
            word!(game.cinematicspr_instr_timer[slot] = 1);
            word!(game.cinematicspr_instr_ptr[slot] = 0xee9b);
        }
    }
}

pub(crate) fn launch_cinematic_sprite(
    object: u16,
    acceleration: u16,
    advance_scene: bool,
    game: &mut Game,
) {
    let slot = usize::from(object >> 1);
    word!(game.cinematicspr_goto_timer[slot] += acceleration);
    let velocity = -((word!(game.cinematicspr_goto_timer[slot]) as i16 as i32) << 8) as u32;
    advance_cinematic_coordinate(object, true, velocity, game);
    if (word!(game.cinematicbg_arr8[slot]) as i32 + 128) & 0x8000 != 0 {
        word!(game.cinematicspr_instr_timer[slot] = 1);
        word!(game.cinematicspr_instr_ptr[slot] = 0xce53);
        if advance_scene {
            word!(game.cinematic_function = 0xcadf);
        }
    }
}

// Cloud setup reads its parameter after the first position store, even for aliased slots.
pub(crate) fn initialize_ending_cloud_band(object: u16, y: u16, game: &mut Game) {
    let slot = usize::from(object >> 1);
    word!(game.cinematicbg_arr8[slot] = y);
    let alternate = word!(game.cinematic_spawn_param) != 0;
    word!(game.cinematicbg_arr7[slot] = if alternate { 384 } else { 128 });
    word!(game.cinematicbg_arr9[slot] = 2560);
    if alternate {
        word!(game.cinematicspr_instr_ptr[slot] += 4);
    }
}

pub(crate) fn initialize_ending_cloud_side(object: u16, right: bool, game: &mut Game) {
    let slot = usize::from(object >> 1);
    word!(game.cinematicbg_arr7[slot] = if right { 320 } else { -64i32 as u16 });
    let alternate = word!(game.cinematic_spawn_param) != 0;
    let y: u16 = if right { 192 } else { 64 };
    let displacement = if right { 256 } else { -256i32 as u16 };
    word!(game.cinematicbg_arr8[slot] = y.wrapping_add(if alternate { displacement } else { 0 }));
    word!(game.cinematicbg_arr9[slot] = 2560);
}

pub(crate) fn wait_ending_cloud(object: u16, band: bool, next: u16, game: &mut Game) {
    let threshold = if band { 176 } else { 96 };
    let below = word!(game.cinematic_var6).wrapping_sub(threshold) & 0x8000 != 0;
    if below == band {
        word!(game.cinematicspr_preinstr_func[(object >> 1)] = next);
    }
}

pub(crate) fn move_ending_cloud(object: u16, velocity: (i16, i16), game: &mut Game) {
    let slot = usize::from(object >> 1);
    // Retain fractional self-stores before advancing each whole coordinate.
    word!(game.cinematicspr_arr7[slot] = word!(game.cinematicspr_arr7[slot]));
    word!(game.cinematicbg_arr8[slot] += velocity.1 as u16);
    if velocity.0 != 0 {
        word!(game.cinematicspr_arr6[slot] = word!(game.cinematicspr_arr6[slot]));
        word!(game.cinematicbg_arr7[slot] += velocity.0 as u16);
    }
}

pub(crate) fn prepare_ending_time(game: &mut Game) {
    let hours = word!(game.game_time_hours);
    word! { game; substate = hours / 10; suit_pickup_light_beam_pos = hours % 10; }
    let minutes = word!(game.game_time_minutes);
    word_transfers!(game, ascending, 1;
        native(0xdf0) <- value(minutes / 10), native(0xdf2) <- value(minutes % 10),
    );
}

pub(crate) fn initialize_ending_time_glyph(object: u16, glyph: u8, game: &mut Game) {
    if glyph != 2 {
        let at = 0xdec + 2 * usize::from(glyph - u8::from(glyph > 2));
        let bytes = [game.g_ram[at], game.g_ram[at + 1]];
        // Hours are LE; minute digits retain native word order.
        let digit = if glyph < 2 {
            u16::from_le_bytes(bytes)
        } else {
            u16::from_ne_bytes(bytes)
        };
        word!(game.cinematicspr_instr_ptr[(object >> 1)] += digit.wrapping_mul(8));
    }
    place_cinematic_sprite(object, (156 + 8 * u16::from(glyph), 160), 0, game);
}

pub(crate) fn advance_postcredits_zoom(zooming: bool, game: &mut Game) {
    if zooming && word!(game.cinematic_var14) != 0 {
        word!(game.cinematic_var14 -= 1);
        fade_cinematic_colors(0x180, 0x10, false, game);
        if word!(game.cinematic_var4) != 0 {
            compose_cinematic_palette(game);
        }
    }
    if !zooming || word!(game.cinematic_var4) == 0 {
        if word!(game.cinematic_var17) != 0 {
            word!(game.cinematic_var17 -= 1);
            fade_cinematic_colors(0x1e0, 0x10, false, game);
            compose_cinematic_palette(game);
        }
    } else {
        word!(game.cinematic_var4 -= 1);
    }
    if zooming {
        word!(game.cinematic_var5 = u16::from((word!(game.cinematic_var5) as u8).wrapping_sub(8)));
        word!(game.cinematic_var6 -= 64);
        if word!(game.cinematic_var6) as i32 - 24 & 0x8000 != 0 {
            word! {game; cinematic_var6 = 24; cinematic_function = 0xe48a;
            cinematic_var4 = 180; cinematic_var14 = 0; }
        }
        return;
    }
    advance_ending_graphics(EndingGraphics::Background, game);
    if word!(game.cinematic_var4 -= 1, reread) == 0 {
        game.g_ram[0x55] = 1;
        set_cinematic_blend(17, 0, 0, 49, game);
        set_fixed_color([63, 95, 159], game);
        game.g_ram[0x5f] = 0;
        game.g_ram[0x58] = 78;
        game.g_ram[0x5d] = 84;
        word_transfers!(game, ascending, 16; le(palette_buffer, 192) <- rom(0x8ce7e9, 192));
        word! { game, 0; cinematic_var8, cinematic_var10 }
        word! {game; cinematic_function = 0xe504; cinematic_var4 = 32; }
        enable_cinematic_background(game);
        set_cinematic_tilemap_updates(true, game);
        word!(game.cinematicbg_var3 = 19456);
        word_transfers!(game, descending, 64; native(0x337e) <- value(127));
        queue_cinematic_tilemap(CinematicTilemap::FullText, game);
    }
}

pub(crate) fn advance_ending_logo_palette(game: &mut Game) -> bool {
    let pair = (word!(game.cinematic_var4).wrapping_mul(4) >> 1) as isize;
    let second = crate::rtl::rom_table_word!(game, 0x8be5e7, pair + 1);
    let first = crate::rtl::rom_table_word!(game, 0x8be5e7, pair);
    for (mut source, offset) in [(first, 16usize), (second, 240)] {
        for index in (0..16).rev() {
            let color = cinematic_native_record::<0x8c>(source, game).word(0);
            // The second band originally wrote native RAM words.
            let color = if offset == 16 { color } else { color.to_le() };
            word!(game.palette_buffer[offset + index] = color);
            source = source.wrapping_sub(2);
        }
    }
    word!(game.cinematic_var4 += 1, reread) as i32 - 16 & 0x8000 == 0
}

pub(crate) fn start_title_pan(stage: usize, cursor: u16, game: &mut Game) -> u16 {
    let (next, zoom, x, y) = [
        (0x9d17, 72, 315, 225),
        (0x9d90, 96, 44, -155i16 as u16),
        (0x9e12, 96, -177i16 as u16, -160i16 as u16),
        (0x9e8b, 67, 0, 0),
    ][stage];
    game.g_ram[0x69] = 17;
    word! {game; cinematic_function = next; cinematic_var6 = zoom;
    cinematic_var8 = x; cinematic_var7 = 0; cinematic_var10 = y; cinematic_var9 = 0;
    g_word_7E199B = if stage < 2 { -2i16 as u16 } else { 0 };
    g_word_7E1999 = if stage < 2 { 0x8000 } else { 0 }; }
    if stage >= 2 {
        word! {game; g_word_7E199F = u16::from(stage == 2);
        g_word_7E199D = if stage == 2 { 0x8000 } else { 0 }; }
    }
    if matches!(stage, 0 | 2) {
        set_cinematic_backdrop(true, game);
    }
    cursor
}

pub(crate) fn advance_title_pan(stage: usize, game: &mut Game) -> bool {
    let (offset, cutoff) = [(0, 7), (0, 176), (4, -163)][stage];
    let velocity = words!((
        word!(@read game, ram_words::g_word_7E199B + offset),
        word!(@read game, ram_words::g_word_7E1999 + offset),
    ));
    let whole = ram_words::cinematic_var8 + offset;
    let fraction = ram_words::cinematic_var7 + offset;
    add_native_pair(whole, fraction, velocity, game);
    let negative = (word!(@read game, whole) as i32 + cutoff) & 0x8000 != 0;
    // The vertical pan ends on the opposite side of its cutoff.
    negative != (stage == 2)
}

pub(crate) fn request_title_skip(game: &mut Game) {
    if word!(game.cinematic_function) as i32 + 0x60d8 & 0x8000 != 0
        && word!(game.joypad1_newkeys) & 0x9080 != 0
        && word!(game.cinematic_var18) == 0
    {
        word! {game; cinematic_var18 = 1; screen_fade_delay = 0; screen_fade_counter = 2; }
    }
}

pub(crate) fn advance_title_wait(game: &mut Game) {
    let expired = word!(game.demo_timer -= 1, reread) & 0x8000 != 0;
    if word!(game.demo_timer) == 0 || expired {
        word!(game.cinematic_function = 0x9fae);
    } else {
        if word!(game.joypad1_newkeys) & 0x9080 == 0 {
            return;
        }
        word!(game.cinematic_function = 0x9f52);
    }
    word! { game, 2; screen_fade_delay, screen_fade_counter }
}

pub(crate) fn initialize_title_sprite(object: u16, variant: usize, game: &mut Game) {
    let (y, palette, effect) =
        [(48, 1024, 0xe194), (81, 512, 0xe198), (196, 2048, 0xe19c)][variant];
    place_cinematic_sprite(object, (128, y), palette, game);
    palette_effects::spawn(effect, game);
}

pub(crate) fn place_cinematic_sprite(
    object: u16,
    position: (u16, u16),
    palette: u16,
    game: &mut Game,
) {
    let slot = usize::from(object >> 1);
    word!(game.cinematicbg_arr7[slot] = position.0);
    word!(game.cinematicbg_arr8[slot] = position.1);
    word!(game.cinematicbg_arr9[slot] = palette);
}

pub(crate) unsafe fn spawn_cinematic_sprite_at(
    definition: u16,
    slot: u16,
    parameter: u16,
    game: &mut Game,
) -> u8 {
    word!(game.cinematic_spawn_param = parameter);
    unsafe { initialize_cinematic_object(CinematicObjects::Sprites, definition, slot, game) }
}

unsafe fn initialize_cinematic_object(
    objects: CinematicObjects,
    definition: u16,
    object: u16,
    game: &mut Game,
) -> u8 {
    use crate::game::sm_8b as bank;
    use CinematicObjects::*;
    let layout = objects.layout();
    let offset = usize::from(object & !1);
    let stride = 2 * usize::from(layout.slots);
    unsafe {
        let definition = crate::rtl::rom_bank_ptr::<0x8b>(definition, game);
        checked_word!(game, layout.instructions + stride + offset;
            = (definition.add(2) as *const u16).read_unaligned());
        checked_word!(game, layout.instructions + offset;
            = (definition.add(4) as *const u16).read_unaligned());
        checked_word!(game, layout.instructions + 2 * stride + offset; = 1);
        for field in layout.cleared {
            checked_word!(game, field + offset; = 0);
        }
        // Setup is read after publication, and Mode7 alone receives a word index.
        let setup = u32::from((definition as *const u16).read_unaligned()) | 0x8b0000;
        match objects {
            Sprites => bank::CallCinematicSpriteObjectSetup(setup, object, game),
            Mode7 => bank::CallCinematicSpriteInit(setup, object >> 1, game),
            Background => {
                if setup != 0x8b93d9 {
                    Unreachable(game);
                }
            }
        }
    }
    0
}

pub(crate) fn clear_cinematic_sprites(game: &mut Game) {
    word_transfers!(game, descending, 16;
        le(cinematicspr_instr_ptr, 15) <- value(0),
        le(cinematicspr_whattodraw, 15) <- value(0),
    );
}

pub(crate) fn clear_cinematic_background(fill: u16, game: &mut Game) {
    word_transfers!(game, descending, 1024; native(0x37fe) <- value(fill));
    word_transfers!(game, descending, 4;
        le(cinematicbg_arr1, 3) <- value(0),
        le(cinematicbg_instr_ptr, 3) <- value(0),
    );
    word!(game.cinematicbg_var1 = 0);
}

pub(crate) fn enable_cinematic_background(game: &mut Game) {
    word!(game.cinematic_enable_objs |= 0x8000);
}

pub(crate) fn set_cinematic_tilemap_updates(enabled: bool, game: &mut Game) {
    word!(game.cinematic_enable_bg_tilemap, |old| (old & 0x7fff)
        | (u16::from(enabled) << 15));
}

unsafe fn advance_cinematic_object(objects: CinematicObjects, object: u16, game: &mut Game) {
    use crate::game::sm_8b as bank;
    use CinematicObjects::*;
    let layout = objects.layout();
    let stride = 2 * usize::from(layout.slots);
    unsafe {
        let pre = u32::from(checked_word!(game,
            layout.instructions + stride + usize::from(object & !1);))
            | 0x8b0000;
        match objects {
            Sprites => bank::CallCinematicSprPreInstr(pre, object, game),
            Mode7 => bank::CallMode7PreInstr(pre, object, game),
            Background => advance_cinematic_background_pre(pre, object, game),
        }
        let selected = checked_word!(game, layout.current;);
        let offset = usize::from(selected & !1);
        let timer_at = layout.instructions + 2 * stride + offset;
        let timer = checked_word!(game, timer_at;);
        checked_word!(game, timer_at; = timer.wrapping_sub(1));
        if timer != 1 {
            return;
        }
        let rom_bank = if matches!(objects, Background) {
            0x8c
        } else {
            0x8b
        };
        let mut address = checked_word!(game, layout.instructions + offset;);
        let duration = loop {
            let instruction = (crate::rtl::rom_ptr_with_bank(rom_bank, address, game)
                as *const u16)
                .read_unaligned();
            if instruction & 0x8000 == 0 {
                break instruction;
            }
            let operand = address.wrapping_add(2);
            address = match (objects, instruction) {
                (Sprites, 0x9438) | (Mode7, 0x956b) | (Background, 0x9698) => {
                    if !matches!(objects, Mode7) {
                        checked_word!(game, layout.cleared[0] + offset; = 0);
                    }
                    checked_word!(game, layout.instructions + offset; = 0);
                    0
                }
                (Sprites, 0x9442) => {
                    checked_word!(game, layout.instructions + offset; = operand.wrapping_sub(2));
                    0
                }
                (Sprites, 0x944c | 0x94d6) | (Mode7, 0x9572 | 0x9597) => {
                    let plane = if matches!(instruction, 0x94d6 | 0x9597) {
                        3
                    } else {
                        1
                    };
                    checked_word!(game, layout.instructions + plane * stride + offset;
                        = (crate::rtl::rom_bank_ptr::<0x8b>(operand, game) as *const u16).read_unaligned());
                    operand.wrapping_add(2)
                }
                (Sprites, 0x9457) | (Mode7, 0x957d) => {
                    checked_word!(game, layout.instructions + stride + offset;
                        = if matches!(objects, Sprites) { 0x945f } else { 0x9585 });
                    operand
                }
                (Sprites, 0x94a2 | 0x94bc | 0x94c3 | 0x94cd)
                | (Mode7, 0x9586 | 0x958d)
                | (Background, 0x971e) => {
                    let relative = matches!(instruction, 0x94a2 | 0x94cd);
                    let skip = matches!(instruction, 0x94c3 | 0x94cd | 0x958d) && {
                        let counter_at = layout.instructions + 3 * stride + offset;
                        let counter = checked_word!(game, counter_at;);
                        checked_word!(game, counter_at; = counter.wrapping_sub(1));
                        counter == 1
                    };
                    if skip {
                        // Skipping a jump must not read its ROM operand.
                        operand.wrapping_add(if relative { 1 } else { 2 })
                    } else {
                        let target = crate::rtl::rom_ptr_with_bank(rom_bank, operand, game);
                        if relative {
                            operand.wrapping_add((*target as i8) as u16)
                        } else {
                            (target as *const u16).read_unaligned()
                        }
                    }
                }
                _ => {
                    let command = u32::from(instruction) | 0x8b0000;
                    match objects {
                        Sprites => bank::CallCinematicSprInstr(command, selected, operand, game),
                        Mode7 => bank::CallMode7Instr(command, selected, operand, game),
                        Background => {
                            cinematic_background_command(command, selected, operand, game)
                        }
                    }
                }
            };
            if address == 0 {
                return;
            }
        };
        checked_word!(game, timer_at; = duration);
        let payload = crate::rtl::rom_ptr_with_bank(rom_bank, address, game);
        match objects {
            Sprites => word!(
                game.cinematicspr_whattodraw[(selected >> 1)] =
                    (payload.add(2) as *const u16).read_unaligned()
            ),
            Mode7 => crate::game::mode7_transfers::enqueue_rom(
                0x8b,
                (payload.add(2) as *const u16).read_unaligned(),
                game,
            ),
            Background => {
                word!(
                    game.cinematicbg_arr1[(selected >> 1)] =
                        (payload.add(4) as *const u16).read_unaligned()
                );
                draw_cinematic_background(
                    selected,
                    (payload.add(2) as *const u16).read_unaligned(),
                    game,
                );
            }
        }
        let size = if matches!(objects, Background) { 6 } else { 4 };
        checked_word!(game, layout.instructions + offset; = address.wrapping_add(size));
    }
}

pub(super) fn cinematic_native_record<const BANK: u8>(cursor: u16, game: &mut Game) -> RomRecord {
    let address = (u32::from(BANK) << 16) | u32::from(cursor);
    if address & 0x8000 == 0 {
        crate::game::session_runtime::native_rom_diagnostic(address, game);
    }
    player_projectiles::content::captured(game.capture_native_rom(), address)
}

// The credits stream shares the background pre-instructions and text tilemap.
// RAM remains authoritative across script callbacks and the ending handoff.
pub(crate) fn prepare_credit_roll(definition: u16, game: &mut Game) -> u32 {
    word_transfers!(game, descending, 1024; native(0x37fe) <- value(0x7f));
    word! { game, 0;
        cinematic_var26, cinematic_var21, cinematic_var22, cinematic_var23, cinematic_var24,
    }
    word!(game.cinematicbg_var3 = 18432);
    word! { game, 0;
        eproj_pre_instr[0], cinematic_var7, cinematic_var8, cinematic_var9, cinematic_var10,
    }
    word!(game.cinematic_var25 |= 0x8000);
    word! { game, 3; screen_fade_delay, screen_fade_counter }
    let record = cinematic_native_record::<0x8b>(definition, game);
    word! {game; cinematic_var24 = record.word(2); cinematic_var21 = record.word(4);
    cinematic_var22 = 1; cinematic_var23 = 0; }
    0x8b0000 | u32::from(record.word(0))
}

// False leaves the legacy invalid-opcode report to the bank, before its upload.
pub(crate) fn advance_credit_roll(game: &mut Game) -> bool {
    // The original pair reads native low then high and writes low before high.
    let (fraction, carry) = u16::from_le(word!(game.cinematic_var9)).overflowing_add(0x8000);
    let whole = u16::from_le(word!(game.cinematic_var10)).wrapping_add(u16::from(carry));
    word!(game.cinematic_var9 = fraction.to_le());
    word!(game.cinematic_var10 = whole.to_le());
    if (i32::from(word!(game.cinematic_var10)) - i32::from(word!(game.eproj_pre_instr[0])) - 8)
        & 0x8000
        != 0
    {
        return true;
    }
    word!(game.eproj_pre_instr[0] = word!(game.cinematic_var10));
    let mut cursor = word!(game.cinematic_var21);
    loop {
        let record = cinematic_native_record::<0x8c>(cursor, game);
        let command = record.word(0);
        if record.word(0) & 0x8000 == 0 {
            break;
        }
        cursor = cursor.wrapping_add(2);
        match 0x8b0000 | u32::from(command) {
            0x8b99fe => {
                word!(game.cinematic_var21 = 0);
                return true;
            }
            0x8b9a06 => cursor = cinematic_native_record::<0x8c>(cursor, game).word(0),
            0x8b9a0d => {
                cursor = if word!(game.cinematic_var23 -= 1, reread) != 0 {
                    cinematic_native_record::<0x8c>(cursor, game).word(0)
                } else {
                    cursor.wrapping_add(2)
                };
            }
            0x8b9a17 => {
                word!(game.cinematic_var23 = cinematic_native_record::<0x8c>(cursor, game).word(0));
                cursor = cursor.wrapping_add(2);
            }
            0x8bf6fe => {
                word!(game.cinematic_var25 &= !0x8000);
                word_transfers!(game, ascending, 252; le(palette_buffer, 4) <- rom(0x8ce7e9, 4));
                game.g_ram[0x69] = 0; // reg_TM
                game.g_ram[0x51] = 0x80; // reg_INIDISP
                word! { game, 0;
                    screen_fade_delay, screen_fade_counter, cinematic_var9, cinematic_var10,
                }
                word! {game; eproj_pre_instr[1] = 1; cinematic_function = 0xe0f4;
                cinematic_var4 = 60; }
            }
            _ => return false,
        }
        if cursor == 0 {
            return true;
        }
    }
    let destination = u16::from(word!(game.cinematic_var26) as u8) * 0x40;
    let source = 0x10000 + u32::from(cinematic_native_record::<0x8c>(cursor, game).word(2));
    word_transfers!(game, ascending, 32; native(0x3000 + destination) <- native(source));
    word!(game.cinematic_var26 = word!(game.cinematic_var26).wrapping_add(1) & 0x1f);
    word!(game.cinematic_var21 = cursor.wrapping_add(4));
    true
}

pub(crate) unsafe fn advance_cinematic_background_pre(command: u32, object: u16, game: &mut Game) {
    match command {
        0x8b93d9 => {}
        0x8bb4bc => {
            if word!(game.cinematic_function) == 0xb207 || word!(game.cinematic_function) == 0xa390
            {
                word!(game.cinematicbg_instr_ptr[(object >> 1)] = 0xd613);
                word!(game.cinematicbg_instr_timer[(object >> 1)] = 1);
                word!(game.cinematicbg_preinstr[(object >> 1)] = 0x93d9);
            }
        }
        _ => {
            Unreachable(game);
        }
    }
}

unsafe fn draw_cinematic_background(object: u16, position: u16, game: &mut Game) {
    let definition = word!(game.cinematicbg_arr1[(object >> 1)]);
    let command = cinematic_native_record::<0x8c>(definition, game).word(0);
    match command {
        0x8849 => {}
        0x884d | 0x88b7 | 0x88fd => {
            if command == 0x884d {
                word!(game.cinematicbg_var1 = u16::from(word!(game.cinematicbg_var1) == 0));
                crate::game::intro_pages::spawn_text_glow(definition, position, game);
                let record = cinematic_native_record::<0x8c>(
                    word!(game.cinematicbg_instr_ptr[(object >> 1)]),
                    game,
                );
                let bottom = if (record.word(6) as i16) < 0 {
                    word!(game.cinematicbg_arr7[15] = 8);
                    8 * (u16::from(record.byte(3)) + 2)
                } else {
                    word!(game.cinematicbg_arr7[15] = 8 * u16::from(record.byte(8)));
                    8 * u16::from(record.byte(9))
                };
                word!(game.cinematicbg_arr8[15] = bottom.wrapping_sub(8));
                if record.word(4) != 0xd67d && word!(game.cinematicbg_var1) != 0 {
                    crate::game::music::QueueSfx3_Max6(0xd, game);
                }
            }
            draw_cinematic_rectangle(command != 0x88fd, definition, position, game);
        }
        0x896b | 0x89cf => {
            let mut destination =
                u16::from(position as u8) + 128 * u16::from(position.to_ne_bytes()[1]);
            let record = cinematic_native_record::<0x8c>(definition, game);
            let width = u16::from(record.byte(2));
            let height = u16::from(record.byte(3));
            let columns = command == 0x89cf;
            let (length, mut remaining) = if columns {
                (height, i32::from(width))
            } else {
                (width, i32::from(height))
            };
            let mut source = definition.wrapping_add(4);
            let mut tail = word!(game.mode7_vram_write_queue_tail);
            loop {
                for (field, value) in [
                    (0, 128),
                    (1, source),
                    (3, 139),
                    (4, length),
                    (6, destination),
                    (8, if columns { 2 } else { 0 }),
                ] {
                    crate::game::mode7_transfers::write_word(tail, field, value, game);
                }
                tail = tail.wrapping_add(9);
                source = source.wrapping_add(length);
                destination = destination.wrapping_add(if columns { 1 } else { 128 });
                // Row counts wrap at u16; column counts retain their signed decrement.
                remaining = if columns {
                    remaining - 1
                } else {
                    i32::from((remaining as u16).wrapping_sub(1))
                };
                if remaining == 0 {
                    break;
                }
            }
            word!(game.mode7_vram_write_queue_tail = tail);
        }
        _ => {
            Unreachable(game);
        }
    }
}

unsafe fn draw_cinematic_item_percentage(game: &mut Game) {
    use crate::c_library::__assert_fail;
    use core::ffi::c_char;
    let mut count = 0u16;
    for index in (0..=4).rev() {
        let address = crate::rtl::rom_table_word!(game, 0x8be70d, index);
        unsafe {
            crate::native_assertion! {
                '_c2rust_label: address < 0x2000;
                b"addr < 0x2000\0", b"src/sm_rtl.h\0", 51, b"uint8 *RomPtr_RAM(uint16_t)\0"
            };
        }
        count = count.wrapping_add(crate::rtl::SnesDivide(
            u16::from_le(checked_word!(game, usize::from(address);)),
            crate::rtl::rom_table_word!(game, 0x8be717, index) as u8,
        ));
    }
    for index in (0..=10).rev() {
        if crate::rtl::rom_table_word!(game, 0x8be721, index) & word!(game.collected_items) != 0 {
            count = count.wrapping_add(1);
        }
    }
    for index in (0..=4).rev() {
        if crate::rtl::rom_table_word!(game, 0x8be737, index) & word!(game.collected_beams) != 0 {
            count = count.wrapping_add(1);
        }
    }
    for (column, digit, visible) in [
        (462, count / 100, count >= 100),
        (463, count / 10 % 10, count >= 10),
        (464, count % 10, true),
    ] {
        if visible {
            let index = (digit.wrapping_mul(4) >> 1) as isize;
            for row in 0..2 {
                word_transfers!(game, ascending, 1; pause_map(column + row * 32) <- value(
                    crate::rtl::rom_table_word!(game, 0x8be741, index + row)
                ));
            }
        }
    }
    word_transfers!(game, ascending, 1; pause_map(465) <- value(14442), pause_map(497) <- value(14458));
}

unsafe fn cinematic_background_command(
    command: u32,
    object: u16,
    operand: u16,
    game: &mut Game,
) -> u16 {
    use crate::game::{intro_pages, sm_8b as bank};
    unsafe {
        match command {
            9154004 => intro_pages::reset_text_marker(game),
            9168423 => draw_cinematic_item_percentage(game),
            9168745 => {
                if word!(game.japanese_text_flag) != 0 {
                    word_transfers!(game, descending, 64; pause_map(799) <- rom(0x8cdf5b, 63));
                }
            }
            9168768 => {
                word_transfers!(game, descending, 64; native(0x363e) <- value(127));
                word!(game.cinematic_function = 0xe797);
            }
            9154115 | 9154169 | 9154676 | 9154739 | 9154971 | 9155112 => {
                let definition = match command {
                    9154115 => bank::addr_kMode7ObjectDef_8BD401,
                    9154169 => bank::addr_kMode7ObjectDef_8BD407,
                    9154676 => bank::addr_kMode7ObjectDef_8BD413,
                    9154739 => bank::addr_kMode7ObjectDef_8BD41F,
                    9154971 => bank::addr_kMode7ObjectDef_8BD42B,
                    _ => bank::addr_kMode7ObjectDef_8BD437,
                };
                return intro_pages::create_japanese_text(definition as u16, operand, game);
            }
            9154139 | 9154193 | 9154700 | 9154763 | 9154995 => {
                let (next, continuation) = match command {
                    9154139 => (0xaeb8, None),
                    9154193 => (0xaf6c, Some(0xd40d)),
                    9154700 => (0xb0f2, Some(0xd419)),
                    9154763 => (0xb123, Some(0xd425)),
                    _ => (0xb1da, Some(0xd431)),
                };
                return intro_pages::spawn_marker(next, continuation, operand, game);
            }
            9155136 => return intro_pages::finish_pages(operand, game),
            9158806 => return bank::CinematicCommonInstr_Func69(object, operand, game),
            9158818 => return bank::CinematicCommonInstr_Func70(object, operand, game),
            _ => return Unreachable(game) as u16,
        }
    }
    operand
}

pub(crate) fn queue_cinematic_tilemap(
    tiles: crate::game::visual_state::CinematicTilemap,
    game: &mut Game,
) {
    use crate::game::visual_state::CinematicTilemap::*;
    if !matches!(tiles, Japanese | JapaneseIntro) && game.g_ram[0x55] == 7 {
        return;
    }
    let (size, source) = match tiles {
        Text => (1920, 0x3000),
        FullText => (2048, 0x3000),
        Eyes => (128, 0x3b40),
        Japanese => (1536, 0x4000),
        JapaneseIntro => (256, 0x3000),
    };
    crate::game::display_refresh::queue_native_transfer(
        word!(game.vram_write_queue_tail),
        size,
        |_| source,
        126,
        // Earlier queue writes can overlap this destination field.
        |game| match tiles {
            Eyes => 18848,
            Japanese => 16768,
            JapaneseIntro => 20192,
            _ => word!(game.cinematicbg_var3),
        },
        game,
    );
}

pub(crate) fn advance_ending_graphics(graphics: EndingGraphics, game: &mut Game) {
    use EndingGraphics::*;
    let limit = if matches!(graphics, Samus) { 16 } else { 6 };
    if (matches!(graphics, Background) && word!(game.cinematic_var17) != 0)
        || (i32::from(word!(game.cinematic_var14)) - limit) & 0x8000 == 0
    {
        return;
    }
    let frame = word!(game.cinematic_var14);
    let index = match graphics {
        Background => 4 * i32::from(frame),
        Samus => i32::from(frame.wrapping_mul(2) >> 1),
    };
    let tail = word!(game.vram_write_queue_tail);
    // Each ROM word stays between the preceding queue stores; bank indices wrap separately.
    for field in [0u16, 2, 4, 5] {
        let value = match (graphics, field) {
            (Background, _) => crate::rtl::rom_table_word!(
                game,
                0x8be45a,
                match field {
                    0 => index,
                    2 => index + 1,
                    4 => i32::from(frame.wrapping_mul(4)) + 2,
                    _ => index + 3,
                } as isize
            ),
            (Samus, 0) => 2048,
            (Samus, 2) => crate::rtl::rom_table_word!(game, 0x8bf6b8, index as isize),
            (Samus, 4) => 127,
            (Samus, _) => crate::rtl::rom_table_word!(game, 0x8bf6d8, index as isize),
        };
        let destination = 0xd0 + usize::from(tail.wrapping_add(field));
        if field == 4 {
            game.g_ram[destination] = value as u8;
        } else {
            word_transfers!(game, ascending, 1; native(destination) <- value(value));
        }
    }
    word!(game.vram_write_queue_tail = tail.wrapping_add(7));
    word!(game.cinematic_var14 += 1);
}

fn scaled_cinematic_value(value: u16, scale: u16) -> u16 {
    ((i32::from(value as i16) * i32::from(scale as i16)) >> 8) as u16
}

pub(crate) fn update_cinematic_transform(rotated: bool, game: &mut Game) {
    use crate::game::projectile_geometry::SINE_COSINE;
    if rotated {
        let angle = (word!(game.cinematic_var5) as u8).wrapping_add(64);
        let cosine = scaled_cinematic_value(
            SINE_COSINE[usize::from(angle) + 64] as u16,
            word!(game.cinematic_var6),
        );
        word!(game.reg_M7A = cosine);
        word!(game.reg_M7D = word!(game.reg_M7A));
        let angle = word!(game.cinematic_var5) as u8;
        let sine = scaled_cinematic_value(
            SINE_COSINE[usize::from(angle) + 64] as u16,
            word!(game.cinematic_var6),
        );
        word! {game; reg_M7B = sine; reg_M7C = sine.wrapping_neg(); }
    } else {
        word! {game; reg_M7A = word!(game.cinematic_var6); reg_M7D = word!(game.cinematic_var6); }
        word! { game, 0; reg_M7B, reg_M7C }
    }
    word!(game.reg_BG1HOFS = word!(game.cinematic_var8));
    word!(game.reg_BG1VOFS = word!(game.cinematic_var10));
}

pub(crate) fn project_cinematic_point(
    point: crate::types::Point16U,
    game: &mut Game,
) -> crate::types::Point16U {
    let x = point.x.wrapping_sub(word!(game.reg_M7X));
    let y = word!(game.reg_M7Y).wrapping_sub(point.y);
    let projected_x = scaled_cinematic_value(x, word!(game.reg_M7A))
        .wrapping_add(scaled_cinematic_value(word!(game.reg_M7B), y));
    let projected_y = scaled_cinematic_value(word!(game.reg_M7C), x)
        .wrapping_add(scaled_cinematic_value(word!(game.reg_M7A), y));
    crate::types::Point16U {
        x: projected_x.wrapping_add(word!(game.reg_M7X)),
        y: word!(game.reg_M7Y).wrapping_sub(projected_y),
    }
}

pub(crate) fn advance_cinematic_fade(
    fade: crate::game::visual_state::CinematicFade,
    game: &mut Game,
) -> bool {
    use crate::game::visual_state::CinematicFade::*;
    let slow = matches!(fade, SlowIn | SlowOut);
    if slow {
        // Only the low delay byte decrements, with the signed-byte wrap retained.
        game.g_ram[0x723] = (word!(game.screen_fade_delay) as u8).wrapping_sub(1);
        if (game.g_ram[0x723] as i8) > 0 {
            return false;
        }
    }
    let increasing = matches!(fade, FastIn | SlowIn);
    let brightness = game.g_ram[0x51];
    if !increasing && brightness & 15 == 0 {
        return true;
    }
    let step = if slow {
        1
    } else {
        word!(game.screen_fade_counter) as u8
    };
    let (next, complete) = if increasing {
        let next = brightness.wrapping_add(step) & 31;
        (next.min(15), next >= 15)
    } else {
        let next = (brightness & 15).wrapping_sub(step);
        let complete = (next as i8) <= 0;
        (if complete { 0x80 } else { next }, complete)
    };
    game.g_ram[0x51] = next;
    if slow {
        game.g_ram[0x723] = if complete {
            0
        } else {
            word!(game.screen_fade_counter) as u8
        };
        if complete {
            game.g_ram[0x725] = 0;
        }
    }
    complete
}

pub(crate) fn capture_cinematic_palette(game: &mut Game) {
    word_transfers!(game, ascending, 256; native(0x2200) <- le(palette_buffer, 0));
}

pub(crate) fn prepare_cinematic_palette_fade(game: &mut Game) {
    for index in 0..256 {
        let color = word!(game.tilemap_stuff[index + 256]);
        for channel in 0..3 {
            let component = (color >> (5 * channel)) & 31;
            let current = index + 512 + 256 * channel;
            word!(game.tilemap_stuff[current] = component << 8);
            word!(game.tilemap_stuff[current + 768] = component << 3);
        }
    }
}

fn cinematic_color_indices(mut offset: u16, count: u16) -> impl Iterator<Item = usize> {
    // The do-while count admits zero (65,536 entries) and wrapping odd byte offsets.
    (0..=count.wrapping_sub(1)).map(move |_| {
        let index = usize::from(offset >> 1);
        offset = offset.wrapping_add(2);
        index
    })
}

pub(crate) fn clear_cinematic_colors(offset: u16, count: u16, game: &mut Game) {
    for index in cinematic_color_indices(offset, count) {
        for plane in [256, 512, 768, 1024] {
            word!(game.tilemap_stuff[index + plane] = 0);
        }
    }
}

pub(crate) fn fade_cinematic_colors(offset: u16, count: u16, increasing: bool, game: &mut Game) {
    for index in cinematic_color_indices(offset, count) {
        for plane in [512, 768, 1024] {
            let current = index + plane;
            if increasing {
                let step = word!(game.tilemap_stuff[current + 768]);
                word!(
                    game.tilemap_stuff[current] =
                        step.wrapping_add(word!(game.tilemap_stuff[current])) & 0x1fff
                );
            } else {
                word!(game.tilemap_stuff[current] -= word!(game.tilemap_stuff[current + 768]));
            }
        }
    }
}

pub(crate) fn compose_cinematic_palette(game: &mut Game) {
    for index in 0..256 {
        // Red uses the physical high byte, independently of native word endianness.
        let red = u16::from(game.g_ram[0x2401 + 2 * index] & 31);
        let green = (word!(game.tilemap_stuff[index + 768]) >> 3) & 0x3e0;
        let blue = (word!(game.tilemap_stuff[index + 1024]) << 2) & 0x7c00;
        let color = red | green | blue;
        word!(game.tilemap_stuff[index] = color);
        word!(game.palette_buffer[index] = color);
    }
}

// Hardware setup remains unsafe: Game construction alone does not attach a PPU.
pub(crate) unsafe fn configure_cinematic_display(
    display: crate::game::visual_state::CinematicDisplay,
    game: &mut Game,
) {
    use crate::game::visual_state::CinematicDisplay::*;
    use crate::rtl::{
        BG1SC, BG2SC, BG3SC, BG4SC, BG12NBA, BG34NBA, BGMODE, CGADSUB, CGWSEL, COLDATA, INIDISP,
        M7SEL, OBSEL, SETINI, TM, TMW, TS, VMAIN,
    };
    let title = display == Title;
    let intro = display == Intro;
    let escape = display == Escape;
    let tiled = matches!(display, Landing | Credits);
    let sprites = match display {
        Escape => 0xa3,
        Credits => 0,
        _ => 3,
    };
    let mode = match display {
        Intro => 9,
        Landing | Credits => 1,
        _ => 7,
    };
    let selection = if matches!(display, Title | Station) {
        0x80
    } else {
        0
    };
    unsafe {
        crate::rtl::register_writes! { game;
            mirror(INIDISP as u16, 0x80, 0x51, false); mirror(OBSEL as u16, sprites, 0x52, false);
            mirror(BGMODE as u16, mode, 0x55, false); mirror(M7SEL as u16, selection, 0x5f, false);
        }
        match display {
            Title | Station | Escape => {
                game.g_ram[0x58] = 0;
                crate::rtl::register_writes! { game;
                    mirror(BG1SC as u16, 0, 0x59, false); mirror(BG2SC as u16, 0, 0x5a, false);
                    mirror(BG3SC as u16, 0, 0x5c, false); mirror(BG4SC as u16, 0, 0x5d, false);
                    mirror(BG12NBA as u16, 0, 0x5e, false); byte(BG34NBA as u16, 0);
                }
            }
            Intro => {
                crate::rtl::register_writes! { game;
                    mirror(BG1SC as u16, 0x50, 0x58, true); mirror(BG2SC as u16, 0x48, 0x59, true);
                    mirror(BG3SC as u16, 0x4c, 0x5a, true);
                }
                game.g_ram[0x5c] = 0;
                crate::rtl::register_writes! { game;
                    mirror(BG4SC as u16, 0, 0x5d, false); byte(BG12NBA as u16, 0);
                    mirror(BG34NBA as u16, 4, 0x5e, true);
                }
            }
            Landing => {
                crate::rtl::write_register_and_mirror(BG1SC as u16, 0x5c, 0x58, true, game);
                game.g_ram[0x59] = 0;
                crate::rtl::register_writes! { game;
                    mirror(BG2SC as u16, 0, 0x5a, false); mirror(BG3SC as u16, 0, 0x5c, false);
                    byte(BG4SC as u16, 0); mirror(BG12NBA as u16, 6, 0x5d, true);
                    mirror(BG34NBA as u16, 0, 0x5e, true);
                }
            }
            Credits => {
                crate::rtl::register_writes! { game;
                    mirror(BG1SC as u16, 0x48, 0x58, true); mirror(BG2SC as u16, 0x4c, 0x59, true);
                    mirror(BG3SC as u16, 0x24, 0x5a, true); mirror(BG4SC as u16, 0, 0x5c, true);
                    mirror(BG12NBA as u16, 0x54, 0x5d, true); mirror(BG34NBA as u16, 2, 0x5e, true);
                }
            }
        }
        let layers = match display {
            Title => 0x10,
            Intro => 4,
            Credits => 1,
            _ => 0x11,
        };
        crate::rtl::register_writes! { game;
            mirror(TM as u16, layers, 0x69, false); mirror(TS as u16, 0, 0x6b, false);
        }
        if tiled {
            for at in [0x60, 0x61, 0x62, 0x6c, 0x6d, 0x6f, 0x72] {
                game.g_ram[at] = 0;
            }
            if display == Credits {
                set_fixed_color([32, 64, 0x80], game);
                word!(game.oam_next_ptr = 0);
                reset_cinematic_transform(128, 128, game);
                word! { game; cinematic_var5 = 0; cinematic_var6 = 1; }
                word! { game, 0;
                    reg_BG1HOFS, reg_BG1VOFS, reg_BG2HOFS, reg_BG2VOFS,
                    cinematic_var7, cinematic_var8, cinematic_var9, cinematic_var10,
                }
            } else {
                word! { game, 0; reg_BG1HOFS, reg_BG1VOFS }
            }
            return;
        }
        crate::rtl::register_writes! { game;
            mirror(TMW as u16, 0, 0x6c, false); byte(VMAIN as u16, 0);
            mirror(CGWSEL as u16, 0, 0x6e, false); mirror(CGADSUB as u16, 0, 0x71, false);
        }
        if escape {
            game.g_ram[0x6f] = 0;
            game.g_ram[0x72] = 0;
        }
        crate::rtl::WriteReg(COLDATA as u16, 0xe0, game);
        if display != Station {
            crate::rtl::write_register_and_mirror(SETINI as u16, 0, 0x77, false, game);
        }
    }
    if !title {
        if !escape {
            game.g_ram[0x6f] = 0;
            game.g_ram[0x72] = 0;
        }
        set_fixed_color([32, 64, 0x80], game);
        if !escape {
            game.g_ram[0x57] = 0;
        }
    }
    if title || intro {
        let fill = if title { 0 } else { 0x2f };
        word_transfers!(game, ascending, 0x3ff; native(0x3000) <- value(fill));
    }
    match display {
        Title => {
            word! { game, 0;
                oam_next_ptr, menu_option_index, screen_fade_delay, screen_fade_counter,
                pausemenu_start_lr_pressed_highlight_timer, pausemenu_lr_animation_timer,
                pausemenu_item_selector_animation_timer, pausemenu_reserve_tank_animation_timer,
                UNUSED_word_7E0731, UNUSED_word_7E073D, pausemenu_lr_animation_frame,
                pausemenu_item_selector_animation_frame, pausemenu_reserve_tank_animation_frame,
                UNUSED_word_7E0745, pausemenu_shoulder_button_highlight, pausemenu_button_label_mode,
                pausemenu_equipment_category_item, pausemenu_reserve_tank_delay_ctr, UNUSED_word_7E0759,
                samus_reserve_health, samus_reserve_missiles, cinematic_var5,
            }
        }
        Intro => {
            word! { game, 0; oam_next_ptr, cinematic_var5 }
        }
        Escape => {
            word! { game, 0; oam_next_ptr, screen_fade_delay, screen_fade_counter, cinematic_var5 }
        }
        _ => {}
    }
    if display != Station {
        word!(game.cinematic_var6 = 256);
    }
    word! { game, 0;
        cinematic_var7, cinematic_var8, cinematic_var9, cinematic_var10, g_word_7E1999,
        g_word_7E199B, g_word_7E199D, g_word_7E199F,
    }
    if matches!(display, Intro | Station) {
        word!(game.fx_layer_blending_config_a = 0);
    }
    if intro {
        word! { game, 0; reg_BG2HOFS, reg_BG2VOFS, reg_BG3HOFS, reg_BG3VOFS }
    }
}

pub(crate) unsafe fn reset_cinematic_transform(x: u16, y: u16, game: &mut Game) {
    use crate::rtl::{M7A, M7B, M7C, M7D, M7X, M7Y, WriteRegWord};
    unsafe {
        WriteRegWord(M7A as u16, 256, game);
        word!(game.reg_M7A = 256);
        WriteRegWord(M7B as u16, 0, game);
        word!(game.reg_M7B = 0);
        WriteRegWord(M7C as u16, 0, game);
        word!(game.reg_M7C = 0);
        WriteRegWord(M7D as u16, 256, game);
        word!(game.reg_M7D = 256);
        WriteRegWord(M7X as u16, x, game);
        word!(game.reg_M7X = x);
        WriteRegWord(M7Y as u16, y, game);
        word!(game.reg_M7Y = y);
    }
}

pub(crate) fn set_fixed_color(channels: [u8; 3], game: &mut Game) {
    for (channel, value) in channels.into_iter().enumerate() {
        game.g_ram[0x74 + channel] = value;
    }
}

pub(crate) fn set_cinematic_blend(
    main_screen: u8,
    sub_screen: u8,
    color_window: u8,
    color_math: u8,
    game: &mut Game,
) {
    game.g_ram[0x69] = main_screen;
    game.g_ram[0x6b] = sub_screen;
    game.g_ram[0x6f] = color_window;
    game.g_ram[0x72] = color_math;
}

pub(crate) fn set_cinematic_backdrop(lit: bool, game: &mut Game) {
    game.g_ram[0x6f] = 0;
    game.g_ram[0x72] = if lit { 0x81 } else { 0 };
    let color = if lit { [56, 88, 0x80] } else { [32, 64, 0x80] };
    set_fixed_color(color, game);
}

pub(crate) fn draw_cinematic_rectangle(
    text: bool,
    mut definition: u16,
    position: u16,
    game: &mut Game,
) {
    let mut row = 2 * u16::from(position as u8) + 64 * u16::from(position.to_ne_bytes()[1]);
    let header = cinematic_native_record::<0x8c>(definition, game);
    let width = u16::from(header.byte(2));
    let mut rows = i32::from(header.byte(3));
    loop {
        let mut destination = row;
        let mut remaining = width;
        loop {
            // The cursor wraps before the next lookup; the word at +4 does not.
            let tile = cinematic_native_record::<0x8c>(definition, game).word(4);
            let at = if text { 0x3000 } else { 0x3800 } + usize::from(destination);
            word_transfers!(game, ascending, 1; native(at) <- value(tile));
            definition = definition.wrapping_add(2);
            destination = destination.wrapping_add(2);
            remaining = remaining.wrapping_sub(1);
            if remaining == 0 {
                break;
            }
        }
        // Text used a wrapping u16 row counter; background used signed i32.
        rows = if text {
            i32::from((rows as u16).wrapping_sub(1))
        } else {
            rows - 1
        };
        if rows == 0 {
            break;
        }
        row = row.wrapping_add(64);
    }
}

pub(crate) fn fade_cinematic_fixed_color(game: &mut Game) {
    for (lane, black) in [0x20u8, 0x40, 0x80].into_iter().enumerate() {
        let at = 0x74 + lane;
        // Keep decrement and clamp as separate stores in channel order.
        game.g_ram[at] = game.g_ram[at].wrapping_sub(1);
        if game.g_ram[at].wrapping_sub(black) & 0x80 != 0 {
            game.g_ram[at] = black;
        }
    }
}

pub(crate) unsafe fn draw_intro_sprites(game: &mut Game) {
    let samus = word!(game.cinematic_var15);
    if samus & 0x8000 != 0 {
        draw_player_frame(game);
        player_projectiles::draw_projectiles(game);
    }
    draw_cinematic_sprites(false, game);
    if samus != 0 && samus & 0x8000 == 0 {
        draw_player_frame(game);
        player_projectiles::draw_projectiles(game);
    }
}

pub(crate) fn draw_cinematic_sprites(ending: bool, game: &mut Game) {
    use crate::game::sprite_drawing::Clip;
    for slot in (0..16usize).rev() {
        let sprite = word!(game.cinematicspr_whattodraw[slot]);
        if sprite == 0 {
            continue;
        }
        let palette = word!(game.cinematicbg_arr9[slot]);
        let x = word!(game.cinematicbg_arr7[slot]).wrapping_sub(word!(game.layer1_x_pos));
        let y = word!(game.cinematicbg_arr8[slot]).wrapping_sub(word!(game.layer1_y_pos));
        if y.wrapping_add(128) < 0x1ff {
            let clip = if y & 0xff00 != 0 {
                Clip::BoundedOutside
            } else {
                Clip::Bounded
            };
            sprite_output::paletted(0x8c, sprite, (x, y), palette, clip, game);
        }
    }
    if ending
        && word!(game.cinematic_function).wrapping_sub(0xda02) & 0x8000 != 0
        && word!(game.oam_next_ptr) < 0x200
    {
        let first = usize::from(word!(game.oam_next_ptr) >> 2);
        let group = usize::from(word!(game.oam_next_ptr) >> 5);
        word!(game.oam_ext[group] |= 0x5555u16 << (2 * (first & 7)));
        for group in group + 1..16 {
            word!(game.oam_ext[group] = 0x5555);
        }
        // The extension writes precede rereading the first unused OAM entry.
        let first = usize::from(word!(game.oam_next_ptr) >> 2);
        word_transfers!(game, 4, 128 - first; native(0x370 + 4 * first) <- value(0x80));
    }
}
