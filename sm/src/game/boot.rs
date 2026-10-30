//! Boot/reset sequencing over the authoritative compatibility state.
use crate::game_state::Game;
use crate::ram_map::word;
use crate::rtl::{
    APUI00, APUI01, APUI02, APUI03, BG1HOFS, BG1SC, BG1VOFS, BG2HOFS, BG2SC, BG2VOFS, BG3HOFS,
    BG3SC, BG3VOFS, BG4HOFS, BG4SC, BG4VOFS, BG12NBA, BG34NBA, CGADSUB, CGWSEL, M7A, M7B, M7C, M7D,
    M7X, M7Y, MEMSEL, MOSAIC, NMITIMEN, RtlApuWrite, SETINI, VMAIN, W12SEL, W34SEL, WBGLOG, WH0,
    WH1, WH2, WH3, WOBJLOG, WOBJSEL,
};
use sm_game::display::control::{
    BgMode, Brightness, FixedColor, Layers, MainScreen, MainWindowMask, Mode7Settings, SubScreen,
    SubWindowMask,
};
use sm_game::menus::save_slots;
use sm_game::sprite_drawing::{clear_high_table, finish_frame};

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
            crate::rtl::write_register_and_mirror(MEMSEL as u16, 1, 0x86, false, game);
            clear_work_ram(0x2000, game);
        }
        if matches!(resume, 1 | 2) {
            let yielded: u8 = sm_game::cinematic_states::nintendo_logo(game);
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
            crate::rtl::write_register_and_mirror(NMITIMEN as u16, 0, 0x84, false, game);
            game.ram().write_byte(0x51, 0x8f);
            initialize_registers(game);
            for target in [0x3000, 0x4000, 0x6000] {
                word_transfers!(game, ascending, 1024; native(target) <- value(0x1c2f));
            }
            for base in [0x643, 0x646, 0x649, 0x64d, 0x650] {
                for offset in 0..3 {
                    game.ram().write_byte(base + offset, 0);
                }
            }
            for address in [0x656, 0x666, 0x676] {
                game.ram().write_byte(address, 0);
            }
            word! { game, 0; OAM_NEXT_PTR, REG_OA_MADDR_UNUSED }
            clear_high_table(game);
            finish_frame(game);
            word! { game, 0;
                NMI_COPY_SAMUS_HALVES, NMI_COPY_SAMUS_TOP_HALF_SRC, NMI_COPY_SAMUS_BOTTOM_HALF_SRC,
            }
            crate::game::display_refresh::enable(true, game);
            RtlApuWrite(APUI00, 0, game);
            RtlApuWrite(APUI01, 0, game);
            RtlApuWrite(APUI02, 0, game);
            RtlApuWrite(APUI03, 0, game);
            word!(game.RANDOM_NUMBER = 97);
            crate::game::music::reset_delays(game);
            word!(game.ENABLE_DEBUG = 0);
            save_slots::verify_boot(game);
            word! { game, 0; DEBUG_DISABLE_SOUNDS, SOUND_HANDLER_DOWNTIME }
        }
        game.ram().write_byte(0x611, 2);
        0
    }
}

unsafe fn initialize_registers(game: &mut Game) {
    unsafe {
        crate::game::display_refresh::initialize_io_and_oam(Some(0x8f), game);
        crate::rtl::register_writes! { game;
            show_mirror(BgMode::MODE1_BG3_PRIORITY, 0x55, false); mirror(MOSAIC as u16, 0, 0x57, false);
            mirror(BG1SC as u16, 0x40, 0x58, false); mirror(BG2SC as u16, 0x44, 0x59, false);
            mirror(BG3SC as u16, 0x48, 0x5a, false); mirror(BG4SC as u16, 0, 0x5c, false);
            mirror(BG12NBA as u16, 0, 0x5d, false); mirror(BG34NBA as u16, 5, 0x5e, false);
            byte(BG1HOFS as u16, 0); byte(BG1HOFS as u16, 0);
            byte(BG1VOFS as u16, 0); byte(BG1VOFS as u16, 0);
            byte(BG2HOFS as u16, 0); byte(BG2HOFS as u16, 0);
            byte(BG2VOFS as u16, 0); byte(BG2VOFS as u16, 0);
            byte(BG3HOFS as u16, 0); byte(BG3HOFS as u16, 0);
            byte(BG3VOFS as u16, 0); byte(BG3VOFS as u16, 0);
            byte(BG4HOFS as u16, 0); byte(BG4HOFS as u16, 0);
            byte(BG4VOFS as u16, 0); byte(BG4VOFS as u16, 0);
            byte(VMAIN as u16, 0); show_mirror(Mode7Settings::WRAP, 0x5f, false);
            byte(M7A as u16, 0); byte(M7B as u16, 0);
            byte(M7C as u16, 0); byte(M7D as u16, 0);
            byte(M7X as u16, 0); byte(M7Y as u16, 0);
            mirror(W12SEL as u16, 0, 0x60, false); mirror(W34SEL as u16, 0, 0x61, false);
            mirror(WOBJSEL as u16, 0, 0x62, false); mirror(WH0 as u16, 0, 0x63, false);
            byte(WH1 as u16, 0xf8);
        }
        game.ram().write_byte(0x64, 0xf8);
        crate::rtl::register_writes! { game;
            mirror(WH2 as u16, 0, 0x65, false); mirror(WH3 as u16, 0, 0x66, false);
            mirror(WBGLOG as u16, 0, 0x67, false); mirror(WOBJLOG as u16, 0, 0x68, false);
            show_mirror(MainScreen(Layers::of(true, false, false, false, true)), 0x69, false); show_mirror(MainWindowMask(Layers::of(true, false, false, false, true)), 0x6c, false);
            show_mirror(SubScreen(Layers::of(false, true, false, false, false)), 0x6b, false); show_mirror(SubWindowMask(Layers::of(false, true, false, false, false)), 0x6d, false);
            mirror(CGWSEL as u16, 2, 0x6e, false); byte(CGADSUB as u16, 0xa1);
        }
        game.ram().write_byte(0x71, 0xa1);
        let blue = FixedColor::blue(0);
        crate::game::display_transfers::set_fixed_color(
            &[FixedColor::all(0), FixedColor::all(0), blue],
            game,
        );
        game.ram().write_byte(0x74, blue.byte());
        let green = FixedColor::green(0);
        crate::game::display_transfers::set_fixed_color(&[green], game);
        game.ram().write_byte(0x75, green.byte());
        let red = FixedColor::red(0);
        crate::game::display_transfers::set_fixed_color(&[red], game);
        game.ram().write_byte(0x76, red.byte());
        crate::rtl::write_register_and_mirror(SETINI as u16, 0, 0x77, false, game);
    }
}
