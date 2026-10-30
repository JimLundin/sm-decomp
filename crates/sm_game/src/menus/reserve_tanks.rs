//! Reserve refill and equipment-menu controls; RAM remains authoritative.
#![forbid(unsafe_code)]
use super::menu_tilemap;
use super::prelude::*;
use crate::ram::{checked_word as at, word, words as fields};

pub fn prepare_menu<H: Host + ?Sized>(game: &mut H) {
    word! {game; reg_BG4HOFS = word!(game.reg_BG1HOFS); reg_BG4VOFS = word!(game.reg_BG1VOFS); }
    word! { game, 0; reg_BG1HOFS, reg_BG1VOFS }
    if word!(game.samus_max_reserve_health) != 0 && word!(game.reserve_health_mode) != 0 {
        let address = if word!(game.reserve_health_mode) == 1 {
            0x82bf2a
        } else {
            0x82bf22
        };
        let label = menu_tilemap::native_record(address, game);
        menu_tilemap::reserve_label(|index| label.word(2 * index), game);
    }
    word!(game.pausemenu_item_selector_animation_frame = 0);
    word!(
        game.pausemenu_item_selector_animation_timer =
            u16::from(menu_tilemap::native_record(0x82c10c, game).byte(0))
    );
    word!(game.pausemenu_reserve_tank_animation_frame = 0);
    word!(
        game.pausemenu_reserve_tank_animation_timer =
            u16::from(menu_tilemap::native_record(0x82c165, game).byte(0))
    );
    if word!(game.samus_max_reserve_health) != 0 {
        word!(game.pausemenu_equipment_category_item = 0);
    } else {
        'category: for (category, count, masks, collected) in [
            (1, 5, 0x82c04c, fields::collected_beams),
            (2, 6, 0x82c056, fields::collected_items),
        ] {
            if category == 1 && word!(game.hyper_beam_flag) != 0 {
                continue;
            }
            for index in 0..count {
                if crate::cartridge::rom_table_word!(game, masks, index) & at!(game, collected;)
                    != 0
                {
                    word!(
                        game.pausemenu_equipment_category_item =
                            u16::from_ne_bytes([0, index as u8]) | category
                    );
                    break 'category;
                }
            }
        }
        // No matching beam/suit leaves the previous selection unchanged.
    }
    if word!(game.samus_reserve_health) != 0 {
        menu_tilemap::reserve_arrow(menu_tilemap::ReserveArrow::On, game);
        menu_tilemap::queue_equipment_wireframe(game);
    }
}

pub fn toggle_mode<H: Host + ?Sized>(game: &mut H) {
    if word!(game.joypad1_newkeys) & kButton_A as u16 == 0
        || word!(game.samus_max_reserve_health) == 0
    {
        return;
    }
    game.queue_sound(0, (0x37) as u8, 6);
    let automatic = word!(game.reserve_health_mode) != 1;
    word!(game.reserve_health_mode = if automatic { 1 } else { 2 });
    game.hud_reserve_mode(automatic);
    let first = if automatic { 0x3d56 } else { 0x3d46 };
    menu_tilemap::reserve_label(|index| first + index as u16, game);
}

// Manual refill retains its zero-valued health write; automatic refill skips it.
fn transfer_one<H: Host + ?Sized>(adjust_zero: bool, game: &mut H) -> bool {
    if (word!(game.samus_health += 1, reread) as i16)
        .wrapping_sub(word!(game.samus_max_health) as i16)
        >= 0
    {
        word!(game.samus_health = word!(game.samus_max_health));
        return false;
    }
    let remaining = word!(game.samus_reserve_health -= 1, reread);
    if remaining as i16 > 0 {
        return true;
    }
    if adjust_zero || remaining != 0 {
        word!(game.samus_health += word!(game.samus_reserve_health));
    }
    false
}

pub fn transfer_manual<H: Host + ?Sized>(game: &mut H) {
    if word!(game.pausemenu_reserve_tank_delay_ctr) == 0 {
        if word!(game.joypad1_newkeys) & kButton_A as u16 == 0 {
            return;
        }
        word!(
            game.pausemenu_reserve_tank_delay_ctr =
                (i32::from(word!(game.samus_reserve_health)) + 7 & 0xfff8) as u16
        );
    }
    if word!(game.pausemenu_reserve_tank_delay_ctr -= 1, reread) & 7 == 7 {
        game.queue_sound(2, (0x2d) as u8, 6);
    }
    if transfer_one(true, game) {
        return;
    }
    word! { game, 0; samus_reserve_health, pausemenu_reserve_tank_delay_ctr }
    menu_tilemap::reserve_arrow_tiles(false, game);
    word!(game.pausemenu_equipment_category_item = 0);
}

pub fn refill_auto<H: Host + ?Sized>(game: &mut H) -> u8 {
    if word!(game.samus_reserve_health) != 0 {
        if word!(game.nmi_frame_counter_word) & 7 == 0 {
            game.queue_sound(2, (0x2d) as u8, 3);
        }
        if !transfer_one(false, game) {
            word!(game.samus_reserve_health = 0);
            return (word!(game.samus_reserve_health) == 0) as u8;
        }
    }
    (word!(game.samus_reserve_health) == 0) as u8
}
