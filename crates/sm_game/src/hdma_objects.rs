//! HDMA objects: allocation, channel setup, the per-frame program interpreter and the room effect reset.
// The original's names and assignment shapes are kept.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_assignments)]
use crate::cinematic::cinematic_native_record;
use crate::display::registers::{A1B0, BBAD0, DAS00, DMAP0, HDMAEN, MDMAEN};
use crate::imported::word as table_word;
#[allow(unused_imports)]
use crate::ram::RawRam as _;
use crate::ram::{checked_word, word};

/// The HDMA effects whose state the host's session view projects.
#[derive(Clone, Copy)]
pub enum HdmaEffect {
    /// Step the current object's program past its wait.
    AdvanceProgram,
    InitializePhantoonWave,
    InitializeEyeBeam,
    ResetWave,
    InitializeWavePair {
        base: usize,
        primary: u16,
    },
    SetPhantoonMotion {
        direction: u16,
        speed: u16,
    },
    ScrollWeather(crate::visual_state::Weather),
}
use crate::cinematic::set_fixed_color;
use crate::menus::prelude::SpawnHdmaObject_Args;
use sm_data::hdma_objects::BG3_X_VELOCITY_ADDRESS;
use sm_data::title_screen::GRADIENT_ADDRESS;

pub fn spawn_bg3_scroll<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    spawn_hdma_last(0x88, &SpawnHdmaObject_Args::new(0x43, 0x11, 0xd8d0), game)
}

fn hdma_definition(args: &SpawnHdmaObject_Args) -> [u16; 2] {
    [
        u16::from_ne_bytes([args.dma_control, args.target_register]),
        args.instruction_list,
    ]
}

// Allocation publishes RAM first and checks the owned backend at each register byte.
pub fn spawn_hdma<H: crate::menus::host::Host + ?Sized>(
    bank: u8,
    args: &SpawnHdmaObject_Args,
    game: &mut H,
) -> u16 {
    let words = hdma_definition(args);
    spawn_hdma_words(bank, |index| words[index], game)
}

pub fn spawn_hdma_last<H: crate::menus::host::Host + ?Sized>(
    bank: u8,
    args: &SpawnHdmaObject_Args,
    game: &mut H,
) {
    let words = hdma_definition(args);
    initialize_hdma(5, bank, |index| words[index], game);
}

pub fn spawn_hdma_words<H: crate::menus::host::Host + ?Sized>(
    bank: u8,
    read_word: impl Fn(usize) -> u16,
    game: &mut H,
) -> u16 {
    match (0..6).find(|&slot| checked_word!(game.hdma_object_channels_bitmask[slot]) == 0) {
        Some(slot) => initialize_hdma(slot, bank, read_word, game),
        None => u16::MAX,
    }
}

fn initialize_hdma<H: crate::menus::host::Host + ?Sized>(
    slot: usize,
    bank: u8,
    read_word: impl Fn(usize) -> u16,
    game: &mut H,
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
    let register = DMAP0 as u16 + channel;
    game.write_dma_register(register, control as u8);
    game.write_dma_register(register + 1, (control >> 8) as u8);
    2 * slot as u16
}

pub fn set_hdma_enabled<H: crate::menus::host::Host + ?Sized>(enabled: bool, game: &mut H) {
    let flags = checked_word!(game.hdma_objects_enable_flag);
    checked_word!(game.hdma_objects_enable_flag = (flags & 0x7fff) | (u16::from(enabled) << 15));
}

pub fn clear_hdma_channels<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    game.ram().write_byte(0x85, 0);
    crate::word_transfers!(game, descending, 6; le(hdma_object_channels_bitmask, 5) <- value(0));
}

pub fn clear_hdma<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        crate::display::sequence::register_writes! { game;
            byte(MDMAEN as u16, 0); byte(HDMAEN as u16, 0);
        }
    }
    clear_hdma_channels(game);
}

pub fn initialize_room_effects<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    use crate::display::registers::{WMADDH, WMADDL, WMADDM};
    use crate::ram::word;
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
            {
                game.write_register(0x4300 + channel + register, value);
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
    crate::word_transfers! { game, ascending, 1;
        native(0x602) <- value(0x8800), native(0x605) <- value(0x8800),
        native(0x601) <- value(0x83e1), native(0x604) <- value(0x83e1),
    }
    {
        crate::display::sequence::register_writes! { game;
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

pub fn update_hdma<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    {
        game.handle_music_queue();
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
                {
                    run_hdma_program(slot as u8, game);
                }
                slot = usize::from(checked_word!(game.hdma_object_index));
            }
            slot += 2;
            if slot == 12 {
                break;
            }
        }
        game.update_layer_blending();
    }
}

fn run_hdma_program<H: crate::menus::host::Host + ?Sized>(slot: u8, game: &mut H) {
    use crate::enemy_script::{EnemyScript as Script, bank_base};
    let index = usize::from(slot >> 1);
    let pre_instruction = (u32::from(checked_word!(game.hdma_object_pre_instruction_bank[index]))
        << 16)
        | u32::from(checked_word!(game.hdma_object_pre_instructions[index]));
    {
        game.hdma_pre_action(pre_instruction, u16::from(slot));
    }
    let timer = checked_word!(game.hdma_object_instruction_timers[index]);
    checked_word!(game.hdma_object_instruction_timers[index] = timer.wrapping_sub(1));
    if timer != 1 {
        return;
    }
    let base = bank_base(game.ram().read_byte(0x18c1 + usize::from(slot)));
    let image = game.native_image();
    let program = checked_word!(game.hdma_object_instruction_list_pointers[index]);
    let position = base + program as isize;
    let mut script = Script::at(
        image.clone(),
        usize::try_from(position).expect("HDMA program outside ROM"),
    );
    while script.word(0) & 0x8000 != 0 {
        let opcode = u32::from(script.word(0)) | 0x880000;
        let next = { execute_hdma_instruction(opcode, u16::from(slot), script.advance(2), game) };
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

fn execute_hdma_instruction<H: crate::menus::host::Host + ?Sized>(
    opcode: u32,
    slot: u16,
    script: crate::enemy_script::EnemyScript,
    game: &mut H,
) -> crate::enemy_script::EnemyScript {
    use crate::enemy_script::{EnemyScript as Script, bank_base};
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
            {
                game.hdma_call(script.load24(0), slot);
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
                8947234 => DMAP0,
                8947255 => BBAD0,
                8947285 => A1B0,
                _ => DAS00,
            } as u16;
            let channel = u16::from(game.ram().read_byte(0x18c0 + 2 * index));
            {
                game.write_register(register + channel, script.byte(0));
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
            let _binding = game.native_image_binding();
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
                    &game.visual_tables().bg3_x_velocities,
                    game,
                    BG3_X_VELOCITY_ADDRESS,
                    ((checked_word!(game.random_number) >> 1 & 6) >> 1) as isize
                )
            );
        }
        8969419 => {
            checked_word!(game.hdma_object_C[index] = 0);
            checked_word!(game.hdma_object_A[index] = 0);
            let opened = game.event_happened(0xa) != 0;
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
        8971453 => game.hdma_effect(HdmaEffect::InitializePhantoonWave, slot),
        0x88e917 => game.hdma_effect(HdmaEffect::InitializeEyeBeam, slot),
        8973471 => game.hdma_effect(HdmaEffect::ResetWave, slot),
        8973215 => {
            game.ram().write_byte(0x6f, 0);
            refresh_title_gradient(game);
        }
        _ => {
            game.rtl_unreachable();
            return Script::Stop;
        }
    }
    script
}

fn refresh_title_gradient<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let mut cursor = table_word(
        &game.visual_tables().title_gradients,
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

pub fn advance_title_gradient<H: crate::menus::host::Host + ?Sized>(backdrop: bool, game: &mut H) {
    if backdrop {
        refresh_title_gradient(game);
    }
    if checked_word!(game.cinematic_function) == 0x9b68 {
        game.hdma_effect(HdmaEffect::AdvanceProgram, 0);
    }
}

pub fn advance_intro_crossfade<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if checked_word!(game.eproj_x_pos[0]) == 1 {
        game.hdma_effect(HdmaEffect::AdvanceProgram, 0);
    }
}
