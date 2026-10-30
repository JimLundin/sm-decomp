//! Room visual effects: HDMA effect setup, Fireflea lighting, earthquakes, the Tourian statue and weather.
// The original's names and assignment shapes are kept.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_assignments)]
use crate::constants::bank_86::addr_kEproj_TourianStatueDustClouds;
use crate::menus::prelude::SpawnHdmaObject_Args;
#[allow(unused_imports)]
use crate::ram::RawRam as _;
use crate::ram::word;
use crate::visual_state::{TourianStatuePhase, Weather};

pub fn initialize_ceres_ridley<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    game.spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x40, 5, 0xd906));
    game.spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0, 0x2c, 0xd91d));
}

pub fn initialize_bomb_torizo_haze<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    game.spawn_hdma(0x88, &SpawnHdmaObject_Args::new(2, 0x32, 0xdd4a));
    game.spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0, 0x32, 0xdd62));
}

pub fn initialize_room_haze<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let program = if game.boss_bit_for_current_area(1) & 1 != 0 {
        0xdeeb
    } else {
        0xded3
    };
    game.spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x40, 0x32, program));
}

pub fn initialize_title_gradient<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    game.spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0, 0x32, 0xeb73));
    game.spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0, 0x31, 0xeb85));
}

const FIREFLEA_FLASH: [u16; 12] = [
    0, 0x100, 0x200, 0x300, 0x400, 0x500, 0x600, 0x500, 0x400, 0x300, 0x200, 0x100,
];

const FIREFLEA_DARKNESS: [u16; 6] = [0, 0x600, 0xc00, 0x1200, 0x1800, 0x1900];

pub fn darken_fireflea_room<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if word!(game.fireflea_darkness_level).wrapping_sub(12) & 0x8000 != 0 {
        word!(game.fireflea_darkness_level += 2);
    }
}

pub fn initialize_fireflea_lighting<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    word! {game; fireflea_flashing_timer = 6; fireflea_flashing_index = 0; }
    game.spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x42, 0x11, 0xb0ac));
    crate::word_transfers!(game, ascending, 1; native(0x9e00) <- value(0));
    word! { game, 0; UNUSED_word_7E177C, fireflea_darkness_level }
    word!(game.UNUSED_word_7E1780 = 24);
    word!(game.UNUSED_word_7E1782 = FIREFLEA_FLASH[0]);
}

pub fn advance_fireflea_lighting<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
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
    game.ram().write_byte(0x75, shade | 0x80);
    game.ram().write_byte(0x76, shade | 0x40);
    game.ram().write_byte(0x74, shade | 0x20);
}

pub fn shake_room_with_sound<H: crate::menus::host::Host + ?Sized>(kind: u16, game: &mut H) {
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
        game.queue_sound(1, (sound) as u8, 6);
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

pub fn initialize_tourian_statue<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    if game.event_happened(0xa) & 1 != 0 {
        game.spawn_hardcoded_plm(6, 12, 0xb777);
        crate::word_transfers!(game, ascending, 1; native(0xcd20) <- value(514));
    }
    game.ram().write_byte(0x59, 74);
    word!(game.fx_rising_function_bank_88 = 0xc428);
    word!(game.fx_y_pos = word!(game.fx_base_y_pos));
    game.spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x42, 0x11, 0xd856));
    game.spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x42, 0x10, 0xdcfa));
    game.spawn_bg3_scroll();
}

pub fn advance_tourian_statue<H: crate::menus::host::Host + ?Sized>(
    mut slot: u16,
    phase: TourianStatuePhase,
    game: &mut H,
) {
    use TourianStatuePhase::*;
    if matches!(phase, Shaking | Sinking) {
        shake_room_with_sound(13, game);
    }
    match phase {
        Waiting => {
            if (6..=9).all(|event| game.event_happened(event) & 1 != 0) {
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
                    game.spawn_room_enemy_projectile(addr_kEproj_TourianStatueDustClouds as u16, 0);
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
            let fraction =
                u16::from_ne_bytes([game.ram().read_byte(low), game.ram().read_byte(low + 1)]);
            let whole =
                u16::from_ne_bytes([game.ram().read_byte(high), game.ram().read_byte(high + 1)]);
            let position = (u32::from(fraction) | u32::from(whole) << 16).wrapping_sub(0x4000);
            crate::word_transfers!(game, ascending, 1;
                native(low) <- value(position as u16),
                native(high) <- value((position >> 16) as u16),
            );
            if word!(game.hdma_object_B[(slot >> 1)]) == 0xff10 {
                {
                    game.spawn_hardcoded_plm(6, 12, 0xb773);
                }
                game.set_event_happened(0xa);
                advance_statue_program(slot, game);
            }
        }
        Finished => {
            word!(game.tourian_entrance_statue_finished = 0x8000);
            crate::word_transfers!(game, ascending, 1; native(0xcd20) <- value(514));
        }
    }
    let scroll = word!(game.layer1_y_pos).wrapping_add(word!(game.hdma_object_B[(slot >> 1)]));
    crate::word_transfers!(game, ascending, 1; native(0x9e00) <- value(scroll));
}

fn advance_statue_program<H: crate::menus::host::Host + ?Sized>(slot: u16, game: &mut H) {
    word!(game.hdma_object_instruction_timers[(slot >> 1)] = 1);
    word!(game.hdma_object_instruction_list_pointers[(slot >> 1)] += 2);
}

pub fn initialize_weather<H: crate::menus::host::Host + ?Sized>(weather: Weather, game: &mut H) {
    use crate::constants::bank_87::{addr_kAnimtiles_Rain, addr_kAnimtiles_Spores};
    let (control, program, tiles) = match weather {
        Weather::Rain => (0x43, 0xd96c, Some(addr_kAnimtiles_Rain)),
        Weather::Spores => (0x42, 0xda2d, Some(addr_kAnimtiles_Spores)),
        Weather::Fog => (0x43, 0xdb19, None),
    };
    game.ram().write_byte(0x5b, 92);
    game.spawn_hdma(0x88, &SpawnHdmaObject_Args::new(control, 0x11, program));
    if let Some(tiles) = tiles {
        game.spawn_bg3_scroll();
        game.spawn_animated_tiles(tiles as u16);
    }
}
