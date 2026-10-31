//! Kraid's pause and unpause hooks (bank $A7:C1FB-$A7:C32F): pausing reads
//! his BG2 tiles back from VRAM; unpausing restores the room's background,
//! his tiles and his top half's tilemap under forced blank, erasing the rows
//! he has already sunk through.
//!
//! Was `UnpauseHook_Kraid_*`, `Kraid_UnpauseHook_IsSinking`,
//! `Kraid_TransferTopHalfToVram` and `PauseHook_Kraid` in the original
//! decompilation.
use crate::actors::kraid_sinking;
use crate::actors::layouts::KraidLayout;
use crate::display_refresh::{GraphicsAsset, enqueue_readback, set_blank, wait};
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

/// What the hooks need: the display refresh and actor RAM.
pub trait Host: crate::display_refresh::Host + crate::actors::host::ActorHost {}
impl<T: crate::display_refresh::Host + crate::actors::host::ActorHost + ?Sized> Host for T {}

type CoroutineRet = u8;

/// The BG tilemap and tile base address register mirrors (BG2SC, BG12NBA).
const BG2_TILEMAP_BASE: usize = 0x59;
const BG12_TILES_BASE: usize = 0x5d;
/// The hooks' coroutine state ($0613): 0 to start, else the NMI wait they
/// resume at: 1 erasing rows, 2 the first forced-blank wait, 3 the wait after
/// clearing it, 4 the sinking hook's last wait.
const HOOK_STATE: usize = 0x613;
/// Kraid's room background ($A7:A716) and the standard BG3 tiles ($9A:B200).
const ROOM_BACKGROUND: u32 = 0xa7_a716;
const STANDARD_BG3_TILES: u32 = 0x9a_b200;
/// Kraid's BG2 tiles and his top half's tilemap in work RAM.
const KRAID_TILES: u32 = 0x7e_5000;
const KRAID_TOP_HALF_TILEMAP: u32 = 0x7e_2000;

/// Unpause hook - Kraid is dead ($A7:C1FB): the room background and the
/// standard BG3 tiles, then his top half. Was `UnpauseHook_Kraid_IsDead`.
pub fn unpause_dead<H: Host + ?Sized>(game: &mut H) -> CoroutineRet {
    crate::probe::hit("kraid_pause.unpause_dead");
    if game.ram().read_byte(HOOK_STATE) != 3 {
        let yielded = blank_and_wait(true, 2, game);
        if yielded != 0 {
            return yielded;
        }
        let tiles =
            ((16 * (game.ram().read_byte(BG12_TILES_BASE) as i32 & 0xf) + 63) as u8 as u16) << 8;
        GraphicsAsset::VramWords(ROOM_BACKGROUND, tiles, 0x200).load(game);
        GraphicsAsset::VramWords(STANDARD_BG3_TILES, 0x4000, 0x800).load(game);
    }
    transfer_top_half(game)
}

/// Unpause hook - Kraid is alive ($A7:C24E): his BG2 tiles, then his top half.
/// The tile base isn't masked, which works because BG1's base is 0. Was
/// `UnpauseHook_Kraid_IsAlive`.
pub fn unpause_alive<H: Host + ?Sized>(game: &mut H) -> CoroutineRet {
    crate::probe::hit("kraid_pause.unpause_alive");
    if game.ram().read_byte(HOOK_STATE) != 3 {
        let yielded = blank_and_wait(true, 2, game);
        if yielded != 0 {
            return yielded;
        }
        load_kraid_tiles(game);
    }
    transfer_top_half(game)
}

/// Set or clear forced blank and wait for NMI ($80:836F, $80:8382): the
/// first call yields at resume point `point`; the call that resumes after
/// the NMI returns 0.
fn blank_and_wait<H: Host + ?Sized>(blank: bool, point: u8, game: &mut H) -> CoroutineRet {
    if game.ram().read_byte(HOOK_STATE) != point {
        set_blank(blank, game);
    }
    let yielded = wait(true, game);
    game.ram()
        .write_byte(HOOK_STATE, if yielded != 0 { point } else { 0 });
    yielded
}

fn load_kraid_tiles<H: Host + ?Sized>(game: &mut H) {
    let tiles = (game.ram().read_byte(BG12_TILES_BASE).wrapping_add(62) as u16) << 8;
    GraphicsAsset::VramWords(KRAID_TILES, tiles, 0x400).load(game);
}

/// Transfer Kraid top half tilemap to VRAM ($A7:C278), then clear forced
/// blank and wait for NMI. Was `Kraid_TransferTopHalfToVram`.
pub fn transfer_top_half<H: Host + ?Sized>(game: &mut H) -> CoroutineRet {
    crate::probe::hit("kraid_pause.transfer_top_half");
    if game.ram().read_byte(HOOK_STATE) != 3 {
        let tilemap = ((game.ram().read_byte(BG2_TILEMAP_BASE) & 0xfc) as u16) << 8;
        GraphicsAsset::VramWords(KRAID_TOP_HALF_TILEMAP, tilemap, 0x800).load(game);
    }
    blank_and_wait(false, 3, game)
}

/// Unpause hook - Kraid is sinking ($A7:C2A0): his BG2 tiles, then one frame
/// per row he has sunk through, erasing it. A coroutine: a nonzero result
/// yields, and the next call resumes erasing. Was
/// `Kraid_UnpauseHook_IsSinking`.
pub fn unpause_sinking<H: Host + ?Sized>(game: &mut H) -> CoroutineRet {
    crate::probe::hit("kraid_pause.unpause_sinking");
    'done: {
        let mut queue_transfer = match game.ram().read_byte(HOOK_STATE) {
            0 | 2 => {
                let yielded = blank_and_wait(true, 2, game);
                if yielded != 0 {
                    return yielded;
                }
                load_kraid_tiles(game);
                if (slot!(game, KraidLayout[0].base.y_pos) as i16)
                    .wrapping_sub(kraid_sinking::sinking_row(0, game).word(0) as i16)
                    < 0
                {
                    break 'done;
                }
                word! {game; MY_COUNTER = 0; MY_COUNTER2 = word!(game.VRAM_WRITE_QUEUE_TAIL); }
                true
            }
            1 => false,
            _ => break 'done,
        };
        loop {
            if queue_transfer {
                let row = kraid_sinking::sinking_row(usize::from(word!(game.MY_COUNTER)), game);
                if !(row.word(0) != 0xffff
                    && (slot!(game, KraidLayout[0].base.y_pos) as i16)
                        .wrapping_sub(row.word(0) as i16)
                        >= 0)
                {
                    break 'done;
                }
                kraid_sinking::queue_row_clear(word!(game.MY_COUNTER2), row.word(2), game);
            }
            let yielded = wait(true, game);
            if yielded != 0 {
                crate::probe::hit("kraid_pause.sinking_row_wait");
                game.ram().write_byte(HOOK_STATE, 1);
                return yielded;
            }
            word!(game.MY_COUNTER += 1);
            queue_transfer = true;
        }
    }
    // The last forced-blank wait ($A7:C31F) leaves the screen blank.
    crate::probe::hit("kraid_pause.sinking_last_wait");
    blank_and_wait(true, 4, game)
}

/// Pause hook - Kraid ($A7:C325): read his BG2 tiles back to $7E:5000. Was
/// `PauseHook_Kraid`.
pub fn pause<H: Host + ?Sized>(game: &mut H) {
    enqueue_readback(BG12_TILES_BASE, 15872, 0x5000, 1024, false, game);
}
