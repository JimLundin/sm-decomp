//! Boot/reset sequencing over the authoritative compatibility state.
use crate::game::sprite_drawing::{clear_high_table, finish_frame};
use crate::game::{save_slots, sm_8b::InitializeIoDisplayLogo_Async};
use crate::game_state::Game;
use crate::native_audio::upload_bank as APU_UploadBank;
use crate::ram_map::word;
use crate::rtl::{
    APUI00, APUI01, APUI02, APUI03, BG1HOFS, BG1SC, BG1VOFS, BG2HOFS, BG2SC, BG2VOFS, BG3HOFS,
    BG3SC, BG3VOFS, BG4HOFS, BG4SC, BG4VOFS, BG12NBA, BG34NBA, BGMODE, CGADSUB, CGWSEL, COLDATA,
    INIDISP, M7A, M7B, M7C, M7D, M7SEL, M7X, M7Y, MEMSEL, MOSAIC, NMITIMEN, RtlApuWrite, SETINI,
    TM, TMW, TS, TSW, VMAIN, W12SEL, W34SEL, WBGLOG, WH0, WH1, WH2, WH3, WOBJLOG, WOBJSEL,
    WriteReg,
};

fn clear_work_ram(bytes: usize, game: &mut Game) {
    let bug_fix_counter = word!(game.bug_fix_counter);
    for offset in 0..bytes {
        game.g_ram[offset] = 0;
    }
    word!(game.bug_fix_counter = bug_fix_counter);
}

/// Requires live, exclusively accessed display/audio hardware and native boot content.
pub(crate) unsafe fn resume(game: &mut Game) -> u8 {
    unsafe {
        crate::verify_probe::hit("coroutine.reset");
        // Callbacks may change RAM; dispatch uses the captured resume byte.
        let resume = game.g_ram[0x611];
        if resume == 1 {
            crate::rtl::write_register_and_mirror(MEMSEL as u16, 1, 0x86, false, game);
            clear_work_ram(0x2000, game);
        }
        if matches!(resume, 1 | 2) {
            let yielded: u8 = InitializeIoDisplayLogo_Async(game);
            if yielded != 0 {
                game.g_ram[0x611] = 2;
                return yielded;
            }
        }
        if matches!(resume, 1..=3) {
            APU_UploadBank(0xcf8000, game);
            WriteReg(INIDISP as u16, 0x8f, game);
            clear_work_ram(0x10000, game);
            crate::rtl::write_register_and_mirror(NMITIMEN as u16, 0, 0x84, false, game);
            game.g_ram[0x51] = 0x8f;
            initialize_registers(game);
            for target in [0x3000, 0x4000, 0x6000] {
                word_transfers!(game, ascending, 1024; native(target) <- value(0x1c2f));
            }
            for base in [0x643, 0x646, 0x649, 0x64d, 0x650] {
                for offset in 0..3 {
                    game.g_ram[base + offset] = 0;
                }
            }
            for address in [0x656, 0x666, 0x676] {
                game.g_ram[address] = 0;
            }
            word! { game, 0; oam_next_ptr, reg_OAMaddr_UNUSED }
            clear_high_table(game);
            finish_frame(game);
            word! { game, 0;
                nmi_copy_samus_halves, nmi_copy_samus_top_half_src, nmi_copy_samus_bottom_half_src,
            }
            crate::game::display_refresh::enable(true, game);
            RtlApuWrite(APUI00, 0, game);
            RtlApuWrite(APUI01, 0, game);
            RtlApuWrite(APUI02, 0, game);
            RtlApuWrite(APUI03, 0, game);
            word!(game.random_number = 97);
            crate::game::music::reset_delays(game);
            word!(game.enable_debug = 0);
            save_slots::verify_boot(game);
            word! { game, 0; debug_disable_sounds, sound_handler_downtime }
        }
        game.g_ram[0x611] = 2;
        0
    }
}

unsafe fn initialize_registers(game: &mut Game) {
    unsafe {
        crate::game::display_refresh::initialize_io_and_oam(Some(0x8f), game);
        crate::rtl::register_writes! { game;
            mirror(BGMODE as u16, 9, 0x55, false); mirror(MOSAIC as u16, 0, 0x57, false);
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
            byte(VMAIN as u16, 0); mirror(M7SEL as u16, 0, 0x5f, false);
            byte(M7A as u16, 0); byte(M7B as u16, 0);
            byte(M7C as u16, 0); byte(M7D as u16, 0);
            byte(M7X as u16, 0); byte(M7Y as u16, 0);
            mirror(W12SEL as u16, 0, 0x60, false); mirror(W34SEL as u16, 0, 0x61, false);
            mirror(WOBJSEL as u16, 0, 0x62, false); mirror(WH0 as u16, 0, 0x63, false);
            byte(WH1 as u16, 0xf8);
        }
        game.g_ram[0x64] = 0xf8;
        crate::rtl::register_writes! { game;
            mirror(WH2 as u16, 0, 0x65, false); mirror(WH3 as u16, 0, 0x66, false);
            mirror(WBGLOG as u16, 0, 0x67, false); mirror(WOBJLOG as u16, 0, 0x68, false);
            mirror(TM as u16, 0x11, 0x69, false); mirror(TMW as u16, 0x11, 0x6c, false);
            mirror(TS as u16, 2, 0x6b, false); mirror(TSW as u16, 2, 0x6d, false);
            mirror(CGWSEL as u16, 2, 0x6e, false); byte(CGADSUB as u16, 0xa1);
        }
        game.g_ram[0x71] = 0xa1;
        crate::rtl::register_writes! { game;
            byte(COLDATA as u16, 0xe0); byte(COLDATA as u16, 0xe0);
            byte(COLDATA as u16, 0x80);
        }
        game.g_ram[0x74] = 0x80;
        WriteReg(COLDATA as u16, 0x40, game);
        game.g_ram[0x75] = 64;
        WriteReg(COLDATA as u16, 0x20, game);
        game.g_ram[0x76] = 32;
        crate::rtl::write_register_and_mirror(SETINI as u16, 0, 0x77, false, game);
    }
}
