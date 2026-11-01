//! Reserve controls on the independently owned inventory domain; automatic refill retains its adapter.
use super::menu_tilemap;
use super::prelude::*;
use crate::inventory::Inventory;
use crate::menu_state::MenuState;
use crate::session_cohort;

pub fn prepare_menu<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    scroll! {game; Bg4H = scroll!(game.Bg1H); Bg4V = scroll!(game.Bg1V); }
    scroll! { game, 0; Bg1H, Bg1V }
    let inventory = game.reserve_state();
    if inventory.capacity != 0 && inventory.mode != 0 {
        let address = if inventory.mode == 1 {
            0x82bf2a
        } else {
            0x82bf22
        };
        let label = menu_tilemap::native_record(address, game);
        menu_tilemap::reserve_label(|index| label.word(2 * index), game);
    }
    menu.pausemenu_item_selector_animation_frame = 0;
    menu.pausemenu_item_selector_animation_timer =
        u16::from(menu_tilemap::native_record(0x82c10c, game).byte(0));
    menu.pausemenu_reserve_tank_animation_frame = 0;
    menu.pausemenu_reserve_tank_animation_timer =
        u16::from(menu_tilemap::native_record(0x82c165, game).byte(0));
    if inventory.capacity != 0 {
        menu.pausemenu_equipment_category_item = 0;
    } else {
        'category: for (category, count, masks, collected) in [
            (1, 5, 0x82c04c, inventory.beams),
            (2, 6, 0x82c056, inventory.equipment),
        ] {
            if category == 1 && inventory.hyper_beam != 0 {
                continue;
            }
            for index in 0..count {
                if crate::menu_lists::equipment_bit(game, masks, index) & collected != 0 {
                    menu.pausemenu_equipment_category_item =
                        u16::from_ne_bytes([0, index as u8]) | category;
                    break 'category;
                }
            }
        }
        // No matching beam/suit leaves the previous selection unchanged.
    }
    if inventory.reserve != 0 {
        menu_tilemap::reserve_arrow(menu_tilemap::ReserveArrow::On, game);
        menu_tilemap::queue_equipment_wireframe(game);
    }
}

pub fn toggle_mode<H: Host + ?Sized>(game: &mut H) {
    let inventory = game.reserve_state();
    if game.observed_inputs().pressed & K_BUTTON_A as u16 == 0 || inventory.capacity == 0 {
        return;
    }
    game.queue_sound(0, 0x37_u8, 6);
    let automatic = inventory.mode != 1;
    let change = game
        .reserve_state()
        .select_mode(if automatic { 1 } else { 2 });
    game.commit_reserve(change);
    crate::hud::menu_reserve_mode(automatic, game);
    let first = if automatic { 0x3d56 } else { 0x3d46 };
    menu_tilemap::reserve_label(|index| first + index as u16, game);
}

/// Move one unit of reserve health into Samus's health; true while the
/// reserve still holds more. Manual refill retains its zero-valued health
/// write; automatic refill skips it.
fn transfer_one(adjust_zero: bool, inventory: &mut Inventory) -> bool {
    let health = &mut inventory.health;
    health.current = health.current.wrapping_add(1);
    if (health.current as i16).wrapping_sub(health.maximum as i16) >= 0 {
        health.current = health.maximum;
        return false;
    }
    inventory.reserve_health = inventory.reserve_health.wrapping_sub(1);
    let remaining = inventory.reserve_health;
    if remaining as i16 > 0 {
        return true;
    }
    if adjust_zero || remaining != 0 {
        inventory.health.current = inventory.health.current.wrapping_add(remaining);
    }
    false
}

pub fn transfer_manual<H: Host + ?Sized>(menu: &mut MenuState, game: &mut H) {
    if menu.pausemenu_reserve_tank_delay_ctr == 0 {
        if game.observed_inputs().pressed & K_BUTTON_A as u16 == 0 {
            return;
        }
        let reserve = game.reserve_state().reserve;
        menu.pausemenu_reserve_tank_delay_ctr = ((i32::from(reserve) + 7) & 0xfff8) as u16;
    }
    menu.pausemenu_reserve_tank_delay_ctr = menu.pausemenu_reserve_tank_delay_ctr.wrapping_sub(1);
    if menu.pausemenu_reserve_tank_delay_ctr & 7 == 7 {
        game.queue_sound(2, 0x2d_u8, 6);
    }
    crate::probe::hit("reserve_tanks.transfer_manual");
    let (change, more) = game.reserve_state().transfer_manual();
    game.commit_reserve(change);
    if more {
        return;
    }
    menu.pausemenu_reserve_tank_delay_ctr = 0;
    menu_tilemap::reserve_arrow_tiles(false, game);
    menu.pausemenu_equipment_category_item = 0;
}

pub fn refill_auto<H: Host + ?Sized>(game: &mut H) -> u8 {
    session_cohort::with(game, |game, inventory: &mut Inventory| {
        if inventory.reserve_health != 0 {
            if game.observed_inputs().frame & 7 == 0 {
                game.queue_sound(2, 0x2d_u8, 3);
            }
            if !transfer_one(false, inventory) {
                inventory.reserve_health = 0;
            }
        }
        u8::from(inventory.reserve_health == 0)
    })
}
