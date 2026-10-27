//! Scanline phases for the HUD, room display and suspended door transitions.
use crate::game::{room_entry, session_runtime};
use crate::game_state::Game;
use crate::ram_map::{checked_word as at, word};
use crate::rtl::{BG3SC, CGADSUB, CGWSEL, HTIMEL, NMITIMEN, TM, VTIMEL, WriteReg};

fn pending(fallback: u16, game: &mut Game) -> u16 {
    let next = word!(game.irqhandler_next_handler);
    word!(game.irqhandler_next_handler = 0);
    if next == 0 { fallback } else { next }
}

fn room_layers(game: &Game) -> u8 {
    if (word!(game.previous_cre_bitset) | word!(game.cre_bitset)) & 1 != 0 {
        16
    } else {
        17
    }
}

unsafe fn schedule(next: u16, vertical: u16, horizontal: u16, game: &mut Game) {
    word!(game.cur_irq_handler = next);
    unsafe {
        crate::rtl::register_writes! { game;
            word(VTIMEL as u16, vertical); word(HTIMEL as u16, horizontal);
        }
    }
}

pub(crate) fn disable(game: &mut Game) {
    at!(game, 0x84; = at!(game, 0x84;) & !0x30);
}

/// Requires a live, exclusively accessed SNES backend.
pub(crate) unsafe fn enable(immediate: bool, game: &mut Game) {
    unsafe {
        crate::rtl::register_writes! { game;
            word(VTIMEL as u16, 0); word(HTIMEL as u16, 152);
        }
        at!(game, 0x84; = at!(game, 0x84;) | 0x30);
        if immediate {
            WriteReg(NMITIMEN as u16, game.ram().read_byte(0x84), game);
        }
    }
}

/// Resuming a door may update the display; its backend must remain live and exclusive.
pub(crate) unsafe fn follow_transition(game: &mut Game) {
    room_entry::protocol::resume(
        &mut session_runtime::Runtime::new(game),
        room_entry::EntrySignal::ScrollIrq,
    );
}

/// Requires a live, exclusively accessed SNES backend for the entire interrupt.
pub(crate) unsafe fn advance(game: &mut Game) {
    // Odd identities alias the preceding even handler. Pending overrides remain full-width.
    let handler = word!(game.cur_irq_handler) >> 1;
    unsafe {
        let (next, vertical, horizontal) = match handler {
            0 => (pending(0, game), 0, 0),
            1 => {
                disable(game);
                (0, 0, 0)
            }
            2 => {
                crate::rtl::register_writes! { game;
                    byte(BG3SC as u16, 0x5a); byte(CGWSEL as u16, 0);
                    byte(CGADSUB as u16, 0); byte(TM as u16, 4);
                }
                (6, 31, 152)
            }
            3 => {
                crate::rtl::register_writes! { game;
                    byte(CGWSEL as u16, game.ram().read_byte(0x70)); byte(CGADSUB as u16, game.ram().read_byte(0x73));
                    byte(BG3SC as u16, game.ram().read_byte(0x5b)); byte(TM as u16, game.ram().read_byte(0x6a));
                }
                (pending(4, game), 0, 152)
            }
            4 => {
                crate::rtl::register_writes! { game;
                    byte(BG3SC as u16, 0x5a); byte(TM as u16, 4);
                    byte(CGWSEL as u16, 0); byte(CGADSUB as u16, 0);
                }
                (10, 31, 152)
            }
            5 => {
                WriteReg(TM as u16, room_layers(game), game);
                (pending(8, game), 0, 152)
            }
            6 | 8 | 11 => {
                crate::rtl::register_writes! { game;
                    byte(TM as u16, 4); byte(CGWSEL as u16, 0); byte(CGADSUB as u16, 0);
                }
                (handler * 2 + 2, 31, 152)
            }
            7 => {
                crate::rtl::register_writes! { game;
                    byte(BG3SC as u16, game.ram().read_byte(0x5b)); byte(CGWSEL as u16, game.ram().read_byte(0x70));
                    byte(CGADSUB as u16, game.ram().read_byte(0x73));
                }
                (pending(12, game), 0, 152)
            }
            9 | 12 => {
                let layers = room_layers(game);
                crate::rtl::register_writes! { game;
                    byte(TM as u16, layers); byte(CGWSEL as u16, 0); byte(CGADSUB as u16, 0);
                }
                if word!(game.door_transition_flag) & 0x8000 == 0 {
                    follow_transition(game);
                }
                (handler * 2 + 2, if handler == 9 { 216 } else { 160 }, 152)
            }
            10 | 13 => (pending(handler * 2 - 4, game), 0, 152),
            _ => panic!("invalid scanline IRQ handler: {handler}"),
        };
        schedule(next, vertical, horizontal, game);
    }
}
