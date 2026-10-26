//! Original upload handshake through the emulated APU bus.
use crate::game_state::Game;

/// Requires the selected live APU/SPC backend for the ordered upload.
pub(crate) unsafe fn upload_bank(addr: u32, game: &mut Game) {
    unsafe {
        if !game.g_use_my_apu_code {
            upload(addr, game);
            return;
        }
        crate::rtl::RtlApuUpload(addr, game);
    }
}
/// Requires the live APU backend; callers retain their frame/interrupt ownership.
pub(crate) unsafe fn write_sound_port(channel: u8, value: u8, game: &mut Game) {
    unsafe {
        crate::rtl::RtlApuWrite(
            crate::snes::registers::APUI01 + u32::from(channel),
            value,
            game,
        );
    }
}

/// The physical backend must be selected before installing the ROM patches.
pub(crate) unsafe fn upload(mut source: u32, game: &mut Game) {
    unsafe {
        unsafe fn word(game: &mut Game, source: u32) -> u16 {
            unsafe {
                u16::from_le_bytes([
                    crate::snes::snes::snes_read(game.g_snes, source, game),
                    crate::snes::snes::snes_read(
                        game.g_snes,
                        source.wrapping_add(1) & 0xffffff,
                        game,
                    ),
                ])
            }
        }
        unsafe fn read(game: &mut Game, port: u8) -> u8 {
            unsafe { crate::snes::snes::snes_readBBus(game.g_snes, 0x40 + port, game) }
        }
        unsafe fn write(game: &mut Game, port: u8, val: u8) {
            unsafe { crate::rtl::RtlApuWrite(0x2140 + u32::from(port), val, game) }
        }
        fn advance(source: &mut u32, count: u32, game: &mut Game) {
            for _ in 0..count {
                let y = (*source as u16).wrapping_add(1);
                if y == 0 {
                    // The original helper returns immediately on its first wrap.
                    game.g_ram[2] = game.g_ram[2].wrapping_add(1);
                    *source = (u32::from(game.g_ram[2]) << 16) | 0x8000;
                    break;
                }
                *source = (*source & 0xff0000) | u32::from(y);
            }
        }
        game.g_ram[0..3].copy_from_slice(&source.to_le_bytes()[..3]);
        game.g_ram[0x617] = 255;
        game.g_ram[0x618] = 255;
        write(game, 0, 255);
        game.g_ram[0x641] = 0;
        game.g_ram[0x642] = 0x30;
        let mut budget = 0x3000u16;
        loop {
            let lo = read(game, 0);
            let hi = read(game, 1);
            if u16::from_le_bytes([lo, hi]) == 0xbbaa {
                break;
            }
            budget = budget.wrapping_sub(1);
            game.g_ram[0x641..0x643].copy_from_slice(&budget.to_le_bytes());
            if budget == 0 {
                // The initial handshake timeout never returns to the caller.
                loop {
                    core::hint::spin_loop();
                }
            }
        }
        let mut token = 0xcc;
        loop {
            let mut count = word(game, source);
            advance(&mut source, 2, game);
            let target = word(game, source);
            advance(&mut source, 2, game);
            write(game, 2, target as u8);
            write(game, 3, (target >> 8) as u8);
            write(game, 1, u8::from(count != 0));
            write(game, 0, token);
            let mut timeout = 0x1000;
            let mut timedout = false;
            do_until! { read(game, 0) == token;
                timeout -= 1;
                if timeout == 0 {
                    timedout = true;
                    break;
                }
            }
            if count == 0 || timedout {
                // Completion stores use the source data bank, including ROM banks.
                for port in 1..=3 {
                    crate::snes::snes::snes_write(
                        game.g_snes,
                        (source & 0xff0000) | 0x2140 | port,
                        0,
                        game,
                    );
                }
                break;
            }
            token = 0;
            loop {
                let val = crate::snes::snes::snes_read(game.g_snes, source, game);
                advance(&mut source, 1, game);
                write(game, 0, token);
                write(game, 1, val);
                count = count.wrapping_sub(1);
                if count == 0 {
                    break;
                }
                while read(game, 0) != token {}
                token = token.wrapping_add(1);
            }
            while read(game, 0) != token {}
            token = token.wrapping_add(4);
            if token == 0 {
                token = 4;
            }
        }
        game.g_ram[0x617] = 0;
        game.g_ram[0x618] = 0;
    }
}
