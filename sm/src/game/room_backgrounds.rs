//! Retained fixed clear/DMA leaves for room entry; command execution is owned.
use crate::game::prelude::*;
use crate::game::room_entry::content::LoadMode;
use crate::game::sm_80::CopyToVramNow;

pub(crate) unsafe fn clear_fx(mode: LoadMode, game: &mut Game) {
    unsafe {
        match mode {
            LoadMode::Initial => clear_tilemap_with_dma(0x5880, 0xa2f7, 0xa2f8, 0x780, game),
            LoadMode::Door => {
                word_transfers!(game, descending, 960;
                    little(0x477e) <- value(6222), little(0x4efe) <- value(6222),
                );
                CopyToVramNow(0x5880, 0x7e4000, 0xf00, game);
            }
        }
    }
}
pub(crate) unsafe fn clear_bg2(mode: LoadMode, game: &mut Game) {
    unsafe {
        match mode {
            LoadMode::Initial => clear_tilemap_with_dma(0x4800, 0xa29a, 0xa29a, 0x800, game),
            LoadMode::Door => {
                word_transfers!(game, descending, 1024;
                    little(0x47fe) <- value(824), little(0x4ffe) <- value(824),
                );
                CopyToVramNow(0x4800, 0x7e4000, 0x1000, game);
            }
        }
    }
}
pub(crate) fn clear_dual_bg(game: &mut Game) {
    word_transfers!(game, descending, 1024; xray(1023) <- value(824), xray(2047) <- value(824));
}

unsafe fn clear_tilemap_with_dma(
    destination: u16,
    low: u16,
    high: u16,
    size: u16,
    game: &mut Game,
) {
    unsafe {
        crate::rtl::register_writes! { game;
            word(VMADDL as u16, destination); word(DMAP1 as u16, 0x1808);
            word(A1T1L as u16, low); word(A1B1 as u16, 0x80);
            word(DAS1L as u16, size); byte(VMAIN as u16, 0);
            byte(MDMAEN as u16, 2); word(VMADDL as u16, destination);
            word(DMAP1 as u16, 0x1908); word(A1T1L as u16, high);
            word(A1B1 as u16, 0x80); word(DAS1L as u16, size);
            byte(VMAIN as u16, 0x80); byte(MDMAEN as u16, 2);
        }
    }
}
