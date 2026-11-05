//! Boot/reset sequencing over the authoritative compatibility state.
use crate::game_state::Game;
use crate::ram_map::word;
use crate::rtl::MEMSEL;
use sm_game::display::control::Brightness;
use sm_game::display::preparation::Field as DisplayField;

fn clear_work_ram(bytes: usize, game: &mut Game) {
    let bug_fix_counter = word!(game.BUG_FIX_COUNTER);
    for offset in 0..bytes {
        game.ram().write_byte(offset, 0);
    }
    word!(game.BUG_FIX_COUNTER = bug_fix_counter);
}

/// Requires live, exclusively accessed display/audio hardware and native boot content.
pub(crate) unsafe fn resume(game: &mut Game) -> u8 {
    unsafe {
        crate::verify_probe::hit("coroutine.reset");
        // Callbacks may change RAM; dispatch uses the captured resume byte.
        let resume = game.ram().read_byte(0x611);
        if resume == 1 {
            crate::rtl::write_register_and_mirror(
                MEMSEL as u16,
                1,
                DisplayField::MemorySelect,
                false,
                game,
            );
            clear_work_ram(0x2000, game);
        }
        if matches!(resume, 1 | 2) {
            use sm_game::cinematic_states::{
                end_nintendo_logo, nintendo_logo, nintendo_logo_ending,
            };
            // Each logo frame runs on a lease of its own, before boot's RAM
            // clear; its last frame only blanks the screen.
            let mut lease = crate::game::session_runtime::Runtime::new(game);
            let yielded: u8 = if nintendo_logo_ending(&lease) {
                end_nintendo_logo(&mut lease)
            } else {
                nintendo_logo(&mut lease)
            };
            drop(lease);
            if yielded != 0 {
                game.ram().write_byte(0x611, 2);
                return yielded;
            }
        }
        if matches!(resume, 1..=3) {
            crate::native_audio::emit(
                crate::native_audio::initialize_cue(),
                sm_data::CuePhase::Initialization,
                game,
            );
            crate::game::display_transfers::set_brightness(Brightness::BLANK_FULL, game);
            clear_work_ram(0x10000, game);
            // The NMI's disable, the brightness mirror and the display
            // registers come first on the frame's lease
            // (`display_refresh::initialize_boot_registers`): they store only
            // the display mirrors ($51..$86), which none of the stores below
            // touch but the OAM address mirror's clear, which moves with them,
            // and no register write reads RAM.
            // The tile buffers at $7E:3000, $4000 and $6000, 1024 words of
            // $1C2F each, in address order: the next lease adopts them.
            use sm_game::presentation_workspace::{Point, Role};
            for target in [
                Point::new(Role::Pause, 0),
                Point::new(Role::TilemapStaging, 0),
                Point::new(Role::TilemapStaging, 0x2000),
            ] {
                let at = target.compatibility_address().expect("screen offset");
                for (offset, byte) in 0x1c2f_u16
                    .to_ne_bytes()
                    .repeat(1024)
                    .into_iter()
                    .enumerate()
                {
                    game.compatibility_write_byte(at + offset, byte);
                }
            }
            for base in [0x643, 0x646, 0x649, 0x64d, 0x650] {
                for offset in 0..3 {
                    game.ram().write_byte(base + offset, 0);
                }
            }
            for address in [0x656, 0x666, 0x676] {
                game.ram().write_byte(address, 0);
            }
            // The OAM address mirror's clear ($53) follows the display setup on
            // the lease (`gameplay_frame::finish_boot`).
            word! { game, 0; OAM_NEXT_PTR }
            word! { game, 0;
                NMI_COPY_SAMUS_HALVES, NMI_COPY_SAMUS_TOP_HALF_SRC, NMI_COPY_SAMUS_BOTTOM_HALF_SRC,
            }
            // The rest of boot's work, from the object table's clear to boot's
            // resume byte, is the frame lease's first step
            // (`gameplay_frame::finish_boot`). The object table and the Samus
            // upload words are separate bytes, so their stores trade places.
            return 0;
        }
        game.ram().write_byte(0x611, 2);
        0
    }
}
