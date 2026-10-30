//! Visual adapters on the one aggregate Session, with explicit legacy effects.
pub(crate) use crate::game::cinematic::{
    advance_cinematic_fade, clear_cinematic_colors, compose_cinematic_palette,
    fade_cinematic_colors, prepare_cinematic_palette_fade, queue_cinematic_tilemap,
    set_cinematic_blend, set_cinematic_tilemap_updates,
};
pub(crate) use crate::game::cinematic_display::{
    configure_cinematic_display, draw_intro_sprites, reset_cinematic_transform,
};
pub(crate) use crate::game::cinematic_objects::{
    CinematicObjects, advance_cinematic_background_pre, advance_intro_mother_brain,
    draw_cinematic_sprites, execute_cinematic_sprite_instruction, spawn_cinematic_burst,
    spawn_cinematic_sprite_at,
};
pub(crate) use crate::game::credits::{advance_credits, prepare_credit_roll};
pub(crate) use crate::game::ending_graphics::advance_ending_logo_palette;
pub(crate) use crate::game::ending_screens::{
    advance_ending_gunship_spin, advance_ending_samus_jump, advance_ending_sprite_pair,
    advance_postcredits_samus_message, advance_postcredits_samus_reveal, advance_postcredits_zoom,
    fade_ending_gunship_backdrop, move_ending_cloud, update_ending_critters, wait_ending_cloud,
};
use crate::game::imported::word as table_word;
pub(crate) use crate::game::intro_objects::{
    advance_ceres_station, advance_intro_baby_pursuit, advance_intro_landing_mosaic,
    advance_intro_rinka, advance_zebes_destruction_zoom, advance_zebes_flight, advance_zebes_stars,
    chase_intro_baby_y, launch_cinematic_sprite, update_cinematic_transform, wait_cinematic_scene,
};
use crate::game::player_transitions::xray::BeamPhase;
use crate::game::projectile_presentation_ram as sprite_output;
pub(crate) use crate::game::room_visuals::{
    advance_fireflea_lighting, advance_tourian_statue, darken_fireflea_room,
    initialize_bomb_torizo_haze, initialize_ceres_ridley, initialize_fireflea_lighting,
    initialize_room_haze, initialize_title_gradient, initialize_tourian_statue, initialize_weather,
    shake_room_with_sound,
};
pub(crate) use crate::game::shooting_star_motion::{
    advance_shooting_stars, initialize_shooting_stars,
};
pub(crate) use crate::game::title_screen::{
    advance_title_pan, advance_title_wait, request_title_skip, start_title_pan,
};
use crate::game::xray_background::{self, finish_restore, restore_left, restore_right};
use crate::game::{
    explosion_window::{self, Phase},
    liquid_effects, mother_brain_visuals, scrolling_sky,
};
use crate::ram_map::{checked_word, word};
use crate::rtl::Unreachable;
use sm_data::hdma_objects::BG3_X_VELOCITY_ADDRESS;
use sm_data::title_screen::GRADIENT_ADDRESS;
use sm_game::actors::access::{AccessMode::LittleEndian as Le, ActorAccess, ActorKey as Key};
pub(crate) use sm_game::cinematic::{
    advance_cinematic_coordinate, capture_cinematic_palette, clear_cinematic_background,
    clear_cinematic_sprites, drift_cinematic_sprite, enable_cinematic_background,
    fade_cinematic_fixed_color, scaled_cinematic_value, set_cinematic_backdrop, set_fixed_color,
};

use crate::{
    game::{
        animated_tiles, enemy_projectiles, palette_effects, player_projectiles,
        session_runtime::Runtime,
        visual_host::{self, AnimationMember, PaletteMember, Source, StatueProjectile},
        visual_state::{
            EyeBeamPhase, HazePhase, HdmaWave, PaletteBuffer, TourianStatuePhase, Upload, Weather,
        },
    },
    game_state::Game,
    rom::RomRecord,
    types::SpawnHdmaObject_Args,
};

pub(crate) fn spawn_bg3_scroll(game: &mut Game) {
    spawn_hdma_last(0x88, &SpawnHdmaObject_Args::new(0x43, 0x11, 0xd8d0), game)
}

crate::forward_calls! { pub(crate) fn (game: &mut Game);
    initialize_ceres_elevator => spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x40, 5, 0xd939), game);
    initialize_draygon_layer => {
        spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0, 0x2c, 0xdf4f), game);
        word!(game.room_loading_irq_handler = 12);
        spawn_bg3_scroll(game);
    };
    initialize_morph_eye_beam => spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x41, 0x26, 0xe8ec), game);
    initialize_intro_crossfade => spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0, 0x31, 0xec03), game);
}

pub(crate) fn initialize_phantoon_motion(direction: u16, speed: u16, game: &mut Game) {
    use sm_game::actors::access::AccessMode::Native;
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
    spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x42, 0xf, 0xe4a8), game);
}

pub(crate) fn initialize_intro_wave(game: &mut Game) {
    word!(game.g_word_7E0D9C = 1);
    word_transfers!(game, ascending, 1; native(0xd9e) <- value(0x4000));
    word!(game.loop_counter_transfer_enemies_to_vram = 8);
    Runtime::new(game).initialize_wave_pair(0xd82, 0x9800);
    spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x42, 0x11, 0xec8a), game);
}

pub(crate) fn advance_crocomire(game: &mut Game) {
    use crate::game::actors::crocomire_reactions::{self as crocomire};
    use crate::game::enemy_layouts::EnemyLayout72 as Crocomire;
    use crate::ram_map::slot;
    static DEFINITION: SpawnHdmaObject_Args = SpawnHdmaObject_Args::new(0x42, 0x10, 0x9563);
    if let Some(request) = crocomire::execute_phase(game) {
        crate::game::music::QueueSfx2_Max6(request as u16, game);
        crocomire::prepare_scroll_hdma(game);
        slot!(
            game,
            Crocomire[0].var_1F = spawn_hdma(0xa4, &DEFINITION, game)
        );
    }
    crocomire::samus_collision(game);
    crocomire::update_palette(game);
}

fn hdma_definition(args: &SpawnHdmaObject_Args) -> [u16; 2] {
    [
        u16::from_ne_bytes([args.dma_control, args.target_register]),
        args.instruction_list,
    ]
}

// Allocation publishes RAM first and checks the owned backend at each register byte.
pub(crate) fn spawn_hdma(bank: u8, args: &SpawnHdmaObject_Args, game: &mut Game) -> u16 {
    let words = hdma_definition(args);
    spawn_hdma_words(bank, |index| words[index], game)
}

pub(crate) fn spawn_hdma_last(bank: u8, args: &SpawnHdmaObject_Args, game: &mut Game) {
    let words = hdma_definition(args);
    initialize_hdma(5, bank, |index| words[index], game);
}

pub(crate) fn spawn_hdma_words(bank: u8, read_word: impl Fn(usize) -> u16, game: &mut Game) -> u16 {
    match (0..6).find(|&slot| checked_word!(game.hdma_object_channels_bitmask[slot]) == 0) {
        Some(slot) => initialize_hdma(slot, bank, read_word, game),
        None => u16::MAX,
    }
}

fn initialize_hdma(
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
    let register = crate::rtl::DMAP0 as u16 + channel;
    crate::rtl::write_dma_register(register, control as u8, game);
    crate::rtl::write_dma_register(register + 1, (control >> 8) as u8, game);
    2 * slot as u16
}

pub(crate) fn set_hdma_enabled(enabled: bool, game: &mut Game) {
    let flags = checked_word!(game.hdma_objects_enable_flag);
    checked_word!(game.hdma_objects_enable_flag = (flags & 0x7fff) | (u16::from(enabled) << 15));
}

pub(crate) fn clear_hdma_channels(game: &mut Game) {
    game.ram().write_byte(0x85, 0);
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
    game.ram().write_byte(0x85, 0);
    set_fixed_color([32, 64, 0x80], game);
    game.ram().write_byte(0x57, 0);
    game.ram().write_byte(0x69, 19);
    game.ram().write_byte(0x6f, 0);
    game.ram().write_byte(0x72, 0);
    game.ram().write_byte(0x5d, 0);
    game.ram().write_byte(0x5e, 4);
    game.ram().write_byte(0x59, 73);
    game.ram().write_byte(0x5a, 90);
    game.ram().write_byte(0x5b, 90);
}

pub(crate) unsafe fn update_hdma(game: &mut Game) {
    unsafe {
        crate::game::music::HandleMusicQueue(game);
    }
    if checked_word!(game.time_is_frozen_flag) == 0
        && checked_word!(game.power_bomb_explosion_status) & 0x4000 != 0
    {
        checked_word!(game.power_bomb_explosion_status = 0x8000);
        spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x40, 0x28, 0x8ace), game);
        spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x40, 0x29, 0x8b80), game);
    }
    checked_word!(game.fx_layer_blending_config_c = checked_word!(game.fx_layer_blending_config_a));
    if checked_word!(game.hdma_objects_enable_flag) & 0x8000 != 0 {
        game.ram().write_byte(0x85, 0);
        let mut slot = 0usize;
        loop {
            checked_word!(game.hdma_object_index = slot as u16);
            let channels = game.ram().read_byte(0x18b4 + slot);
            if channels != 0 {
                game.ram()
                    .write_byte(0x85, game.ram().read_byte(0x85) | channels);
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
        crate::game::layer_blending::update(game);
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
    use sm_game::enemy_script::{EnemyScript as Script, bank_base};
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
    let base = bank_base(game.ram().read_byte(0x18c1 + usize::from(slot)));
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
    script: sm_game::enemy_script::EnemyScript,
    game: &mut Game,
) -> sm_game::enemy_script::EnemyScript {
    use sm_game::enemy_script::{EnemyScript as Script, bank_base};
    let index = usize::from(slot >> 1);
    match opcode {
        8969006 => {}
        8947049 => {
            checked_word!(game.hdma_object_channels_bitmask[index] = 0);
            return Script::Stop;
        }
        8947056 => {
            checked_word!(game.hdma_object_pre_instructions[index] = script.word(0));
            {
                let value = script.byte(2);
                game.ram().write_byte(0x18fc + usize::from(slot), value);
            }
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
            {
                let value = script.byte(0);
                game.ram().write_byte(0x1908 + usize::from(slot), value);
            }
            return script.advance(1);
        }
        8947234 | 8947255 | 8947285 | 8947306 => {
            let register = match opcode {
                8947234 => crate::rtl::DMAP0,
                8947255 => crate::rtl::BBAD0,
                8947285 => crate::rtl::A1B0,
                _ => crate::rtl::DAS00,
            } as u16;
            let channel = u16::from(game.ram().read_byte(0x18c0 + 2 * index));
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
            let base = bank_base(game.ram().read_byte(0x18c1 + usize::from(slot)));
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
                game.hdma_object_D[index] = table_word(
                    &game.visual_tables.bg3_x_velocities,
                    game,
                    BG3_X_VELOCITY_ADDRESS,
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
            game.ram().write_byte(0x9e00, low);
            game.ram().write_byte(0x9e01, high);
            if opened {
                return Script::jump(script.word(0));
            }
            let [low, high] = 1u16.to_ne_bytes();
            game.ram().write_byte(0xcd20, low);
            game.ram().write_byte(0xcd21, high);
            return script.advance(2);
        }
        8971453 => Runtime::new(game).initialize_phantoon_wave(slot),
        0x88e917 => Runtime::new(game).initialize_eye_beam(slot),
        8973471 => Runtime::new(game).reset_hdma_wave(slot),
        8973215 => {
            game.ram().write_byte(0x6f, 0);
            refresh_title_gradient(game);
        }
        _ => {
            crate::rtl::Unreachable(game);
            return Script::Stop;
        }
    }
    script
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
impl sm_game::scene::atmosphere::Host for Runtime<'_> {
    type Error = std::convert::Infallible;
    fn frame_and_kind(&mut self, slot: usize) -> Result<u16, Self::Error> {
        Ok(sm_game::session_access::SessionAccess::atmospheric(self).slots[slot].frame_and_kind)
    }
    fn cursor(&mut self) -> Result<u16, Self::Error> {
        Ok(self.word(0x590))
    }
    fn x(&mut self, slot: usize) -> Result<u16, Self::Error> {
        Ok(sm_game::session_access::SessionAccess::atmospheric(self).slots[slot].x)
    }
    fn camera_x(&mut self) -> Result<u16, Self::Error> {
        Ok(self.word(0x911))
    }
    fn y(&mut self, slot: usize) -> Result<u16, Self::Error> {
        Ok(sm_game::session_access::SessionAccess::atmospheric(self).slots[slot].y)
    }
    fn camera_y(&mut self) -> Result<u16, Self::Error> {
        Ok(self.word(0x915))
    }
    fn write_x(&mut self, entry: usize, value: u8) -> Result<(), Self::Error> {
        self.set_byte(0x370 + entry, value);
        Ok(())
    }
    fn write_y(&mut self, entry: usize, value: u8) -> Result<(), Self::Error> {
        self.set_byte(0x371 + entry, value);
        Ok(())
    }
    fn tile_pointer(&mut self, kind: u16) -> Result<u16, Self::Error> {
        use visual_host::{Content, Record};
        Ok(self
            .table(Source::Loaded, 0x908bff)
            .native_word(2 * usize::from(kind)))
    }
    fn read_tile(&mut self, address: u16) -> Result<u16, Self::Error> {
        use visual_host::Record;
        Ok(self.legacy_native_record(0x90, address).native_word(0))
    }
    fn write_tile(&mut self, entry: usize, value: u16) -> Result<(), Self::Error> {
        self.write_slice_native(0x372 + entry, value);
        Ok(())
    }
    fn publish_cursor(&mut self, value: u16) -> Result<(), Self::Error> {
        self.write_word_le(value, 0x590);
        Ok(())
    }
    fn written(&mut self) -> Result<(), Self::Error> {
        sm_game::session_access::SessionAccess::probe(self, "atmosphere.tile.written");
        Ok(())
    }
}

impl visual_host::Effects for Runtime<'_> {
    #[cfg(sm_verify_dll)]
    fn samus_palette_event(&mut self, kind: &'static str, argument: u16) {
        samus_observer::event(kind, argument);
    }
    #[cfg(sm_verify_dll)]
    fn samus_palette_inject(&mut self, name: &'static str) {
        samus_observer::inject(name, self.session_mut());
    }
    #[cfg(sm_verify_dll)]
    fn palette_publications_observed(&self) -> bool {
        crate::player_transitions_rust_observer::active()
    }
    #[cfg(sm_verify_dll)]
    fn palette_publication(&mut self, kind: u16, at: usize, before: u16, after: u16) {
        crate::player_transitions_rust_observer::publication(kind, at, before, after);
    }
    #[cfg(sm_verify_dll)]
    fn observe_speed_boost_cancel(&mut self, values: [u16; 3]) {
        crate::player_collision_movement_cases::observe_owned_cancel(values);
    }
    fn draw_samus_sprite(&mut self, index: u16, x: u16, y: u16) {
        if self.has_owned_display() {
            let producer = self
                .session()
                .presentation
                .publications()
                .expect("Samus publications")
                .body;
            self.draw_owned_samus(producer, index, (x, y));
            return;
        }
        // Retain the complete native export/action/import interval, including null maps.
        self.foreign(|game| sprite_output::samus(index, (x, y), game, &mut sprite_output::Native));
    }
    fn draw_atmospheric_tile(&mut self, slot: usize, kind: u16) {
        sm_game::scene::atmosphere::draw_tile(self, slot, kind).unwrap();
    }

    fn sound(&mut self, channel: u8, limit: u8, sound: u16) {
        player_projectiles::host::Host::sound(self, channel, limit, sound);
    }
    fn music_delayed8(&mut self, track: u16) {
        if self.has_owned_display() {
            self.owned_call(|runtime| {
                crate::game::music::requests::enqueue(track, 8, true, runtime)
                    .expect("audio enqueue");
            });
            return;
        }
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

fn draw_player_with_cannon(runtime: &mut Runtime<'_>) {
    // Layer order is captured before atmospheric output can alias the mode word.
    let mode = runtime.session().projectiles.controls.cannon_drawing_mode & 0xf;
    crate::game::player_transitions::draw_atmospheric(runtime);
    if mode != 0 && mode != 2 {
        player_projectiles::cannon::draw(runtime, cannon_tile, queue_cannon_tiles);
    }
    runtime.foreign(crate::game::samus_animation::Samus_Draw);
    if mode == 2 {
        player_projectiles::cannon::draw(runtime, cannon_tile, queue_cannon_tiles);
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

fn refresh_title_gradient(game: &mut Game) {
    let mut cursor = table_word(
        &game.visual_tables.title_gradients,
        game,
        GRADIENT_ADDRESS,
        ((word!(game.cinematic_var6) & 0xf0) >> 4) as isize,
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

// Shooting-star scratch records stay in RAM. Native scalar loads use from_le
// on checked LE reads; only the original compound updates use LE arithmetic.
pub(crate) fn scroll_weather(slot: u16, weather: Weather, game: &mut Game) {
    use crate::ram_map::{checked_word, words as ram};
    game.ram().write_byte(0x5b, 92);
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
        use sm_game::actors::access::AccessMode::Native;
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
        use crate::ram_map::words as ram;
        use sm_game::actors::access::AccessMode::Native;
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
        use sm_game::actors::access::AccessMode::Native;
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
        use crate::ram_map::{word_arrays as arrays, words as ram};
        use sm_game::actors::access::AccessMode::Native;
        use sm_game::player_projectiles::access::Store;
        use sm_game::projectile_geometry::SINE_COSINE;
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
    Runtime::new(game).owned_player_call(draw_player_frame_owned);
}

pub(crate) fn draw_player_frame_owned(h: &mut Runtime<'_>) {
    use crate::game::{player_transitions::state::DrawMode, samus_animation as body};
    player_projectiles::cannon::update(h);
    h.refresh_player_observations();
    match h.session().player.transitions.draw_handler {
        DrawMode::Normal | DrawMode::Grapple => {
            let mode = h.session().player.transitions.draw_handler;
            if mode == DrawMode::Normal {
                h.foreign(player_projectiles::flare);
            }
            let grapple = h.session().player.grapple;
            if mode == DrawMode::Normal || (grapple.wrapping_add(0x37aa) as i16) >= 0 {
                h.owned_player_call(draw_player_with_cannon);
                h.foreign(body::Samus_DrawEchoes);
            } else {
                if matches!(grapple, 0xc703 | 0xc759) {
                    h.foreign(crate::game::grapple_swing::restore_start_positions);
                }
                h.foreign(player_projectiles::grapple_flare);
                h.owned_player_call(draw_player_with_cannon);
                h.foreign(crate::game::grapple_swing::update_tiles);
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

// Native split words retain low-before-high reads and writes.
// Cloud setup reads its parameter after the first position store, even for aliased slots.
pub(crate) fn cinematic_native_record<const BANK: u8>(cursor: u16, game: &mut Game) -> RomRecord {
    let address = (u32::from(BANK) << 16) | u32::from(cursor);
    if address & 0x8000 == 0 {
        crate::game::session_runtime::native_rom_diagnostic(address, game);
    }
    player_projectiles::content::captured(game.capture_native_rom(), address)
}

pub(crate) fn project_cinematic_point(
    point: crate::types::Point16U,
    game: &mut Game,
) -> crate::types::Point16U {
    project_cinematic_point_reads(point, |at| {
        crate::game::session_encoding::read(&|at| game.ram().read_byte(at), at)
    })
}
pub(crate) fn project_cinematic_point_reads(
    point: crate::types::Point16U,
    mut read: impl FnMut(usize) -> u16,
) -> crate::types::Point16U {
    let x = point.x.wrapping_sub(read(0x80));
    let y = read(0x82).wrapping_sub(point.y);
    let projected_x =
        scaled_cinematic_value(x, read(0x78)).wrapping_add(scaled_cinematic_value(read(0x7a), y));
    let projected_y =
        scaled_cinematic_value(read(0x7c), x).wrapping_add(scaled_cinematic_value(read(0x78), y));
    crate::types::Point16U {
        x: projected_x.wrapping_add(read(0x80)),
        y: read(0x82).wrapping_sub(projected_y),
    }
}

// Hardware setup remains unsafe: Game construction alone does not attach a PPU.
/// Position Crocomire's second body part and align its BG2 image with the camera.
pub(crate) fn align_crocomire_background(game: &mut Game) {
    use crate::game::enemy_layouts::EnemyLayout72 as Crocomire;
    use crate::ram_map::slot;
    let primary = i32::from(word!(game.cur_enemy_index));
    let secondary = i32::from(word!(game.cur_enemy_index)) + 64;
    slot!(
        game,
        Crocomire[secondary].base.x_pos = slot!(game, Crocomire[secondary].var_A)
            .wrapping_add(slot!(game, Crocomire[primary].base.x_pos))
    );
    slot!(
        game,
        Crocomire[secondary].base.y_pos = slot!(game, Crocomire[primary].base.y_pos)
    );
    let relative =
        slot!(game, Crocomire[primary].base.x_pos).wrapping_sub(word!(game.layer1_x_pos)) as i16;
    let offset = 51u16.wrapping_sub(relative as u16);
    word!(
        game.reg_BG2HOFS = if (-128..384).contains(&relative) && crate::rtl::abs16(offset) < 284 {
            offset
        } else {
            256
        }
    );
}
