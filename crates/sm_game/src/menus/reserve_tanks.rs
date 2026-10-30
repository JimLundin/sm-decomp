//! Reserve refill and equipment-menu controls; RAM remains authoritative.
#![forbid(unsafe_code)]
use super::menu_tilemap;
use super::prelude::*;

pub fn prepare_menu<H: Host + ?Sized>(game: &mut H) {
    scroll! {game; Bg4H = scroll!(game.Bg1H); Bg4V = scroll!(game.Bg1V); }
    scroll! { game, 0; Bg1H, Bg1V }
    if inventory!(game.MaxReserveHealth) != 0 && inventory!(game.ReserveMode) != 0 {
        let address = if inventory!(game.ReserveMode) == 1 {
            0x82bf2a
        } else {
            0x82bf22
        };
        let label = menu_tilemap::native_record(address, game);
        menu_tilemap::reserve_label(|index| label.word(2 * index), game);
    }
    crate::menu!(game.PausemenuItemSelectorAnimationFrame = 0);
    crate::menu!(
        game.PausemenuItemSelectorAnimationTimer =
            u16::from(menu_tilemap::native_record(0x82c10c, game).byte(0))
    );
    crate::menu!(game.PausemenuReserveTankAnimationFrame = 0);
    crate::menu!(
        game.PausemenuReserveTankAnimationTimer =
            u16::from(menu_tilemap::native_record(0x82c165, game).byte(0))
    );
    if inventory!(game.MaxReserveHealth) != 0 {
        crate::menu!(game.PausemenuEquipmentCategoryItem = 0);
    } else {
        'category: for (category, count, masks, collected) in [
            (1, 5, 0x82c04c, crate::inventory::Word::CollectedBeams),
            (2, 6, 0x82c056, crate::inventory::Word::CollectedItems),
        ] {
            if category == 1 && inventory!(game.HyperBeam) != 0 {
                continue;
            }
            for index in 0..count {
                if crate::menu_lists::equipment_bit(game, masks, index)
                    & crate::inventory::get(game, collected)
                    != 0
                {
                    crate::menu!(
                        game.PausemenuEquipmentCategoryItem =
                            u16::from_ne_bytes([0, index as u8]) | category
                    );
                    break 'category;
                }
            }
        }
        // No matching beam/suit leaves the previous selection unchanged.
    }
    if inventory!(game.ReserveHealth) != 0 {
        menu_tilemap::reserve_arrow(menu_tilemap::ReserveArrow::On, game);
        menu_tilemap::queue_equipment_wireframe(game);
    }
}

pub fn toggle_mode<H: Host + ?Sized>(game: &mut H) {
    if game.observed_inputs().pressed & K_BUTTON_A as u16 == 0
        || inventory!(game.MaxReserveHealth) == 0
    {
        return;
    }
    game.queue_sound(0, 0x37_u8, 6);
    let automatic = inventory!(game.ReserveMode) != 1;
    inventory!(game.ReserveMode = if automatic { 1 } else { 2 });
    crate::hud::menu_reserve_mode(automatic, game);
    let first = if automatic { 0x3d56 } else { 0x3d46 };
    menu_tilemap::reserve_label(|index| first + index as u16, game);
}

// Manual refill retains its zero-valued health write; automatic refill skips it.
fn transfer_one<H: Host + ?Sized>(adjust_zero: bool, game: &mut H) -> bool {
    if (inventory!(game.Health += 1, reread) as i16).wrapping_sub(inventory!(game.MaxHealth) as i16)
        >= 0
    {
        inventory!(game.Health = inventory!(game.MaxHealth));
        return false;
    }
    let remaining = inventory!(game.ReserveHealth -= 1, reread);
    if remaining as i16 > 0 {
        return true;
    }
    if adjust_zero || remaining != 0 {
        inventory!(game.Health += inventory!(game.ReserveHealth));
    }
    false
}

pub fn transfer_manual<H: Host + ?Sized>(game: &mut H) {
    if crate::menu!(game.PausemenuReserveTankDelayCtr) == 0 {
        if game.observed_inputs().pressed & K_BUTTON_A as u16 == 0 {
            return;
        }
        crate::menu!(
            game.PausemenuReserveTankDelayCtr =
                ((i32::from(inventory!(game.ReserveHealth)) + 7) & 0xfff8) as u16
        );
    }
    if crate::menu!(game.PausemenuReserveTankDelayCtr -= 1, reread) & 7 == 7 {
        game.queue_sound(2, 0x2d_u8, 6);
    }
    if transfer_one(true, game) {
        return;
    }
    inventory!(game.ReserveHealth = 0);
    crate::menu!(game.PausemenuReserveTankDelayCtr = 0);
    menu_tilemap::reserve_arrow_tiles(false, game);
    crate::menu!(game.PausemenuEquipmentCategoryItem = 0);
}

pub fn refill_auto<H: Host + ?Sized>(game: &mut H) -> u8 {
    if inventory!(game.ReserveHealth) != 0 {
        if game.observed_inputs().frame & 7 == 0 {
            game.queue_sound(2, 0x2d_u8, 3);
        }
        if !transfer_one(false, game) {
            inventory!(game.ReserveHealth = 0);
            return (inventory!(game.ReserveHealth) == 0) as u8;
        }
    }
    (inventory!(game.ReserveHealth) == 0) as u8
}
