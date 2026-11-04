//! Retained fixed clear/DMA leaves for room entry; command execution is owned.
use crate::game::prelude::*;
use sm_game::presentation_workspace::{Access as _, Point};
use sm_game::room_entry::content::LoadMode;

pub(crate) unsafe fn clear_fx(mode: LoadMode, game: &mut Game) {
    unsafe {
        match mode {
            LoadMode::Initial => clear_tilemap_with_dma(0x5880, 0xa2f7, 0xa2f8, 0x780, game),
            LoadMode::Door => {
                // The 1920 words $7E:4000..$4F00, which the original fills
                // from the top down; nothing reads them between the stores.
                game.screen_fill(Point::physical(0x4000), 1920, 6222);
                sm_game::display::transfer::copy_to_vram_now(0x5880, 0x7e4000, 0xf00, None, game);
            }
        }
    }
}
pub(crate) unsafe fn clear_bg2(mode: LoadMode, game: &mut Game) {
    unsafe {
        match mode {
            LoadMode::Initial => clear_tilemap_with_dma(0x4800, 0xa29a, 0xa29a, 0x800, game),
            LoadMode::Door => {
                // The 2048 words $7E:4000..$5000, filled top down in the original.
                game.screen_fill(Point::physical(0x4000), 2048, 824);
                sm_game::display::transfer::copy_to_vram_now(0x4800, 0x7e4000, 0x1000, None, game);
            }
        }
    }
}
pub(crate) fn clear_dual_bg(game: &mut Game) {
    // The 2048 words $7E:4000..$5000, filled top down in the original.
    game.screen_fill_native(Point::physical(0x4000), 2048, 824);
}

unsafe fn clear_tilemap_with_dma(
    destination: u16,
    low: u16,
    high: u16,
    size: u16,
    game: &mut Game,
) {
    unsafe {
        let assets = game.world().prepared_room_assets();
        crate::rtl::register_writes! { game;
            word(VMADDL as u16, destination); word(DMAP1 as u16, 0x1808);
            word(A1T1L as u16, low); word(A1B1 as u16, 0x80);
            word(DAS1L as u16, size); byte(VMAIN as u16, 0);
        }
        crate::game::display_refresh::start_room_dma(2, assets.as_deref(), game);
        crate::rtl::register_writes! { game;
            word(VMADDL as u16, destination);
            word(DMAP1 as u16, 0x1908); word(A1T1L as u16, high);
            word(A1B1 as u16, 0x80); word(DAS1L as u16, size);
            byte(VMAIN as u16, 0x80);
        }
        crate::game::display_refresh::start_room_dma(2, assets.as_deref(), game);
    }
}
