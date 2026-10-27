//! Original upload handshake through the emulated APU bus.
use crate::execution::Execution;
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
        unsafe fn word(execution: &mut Execution<'_>, source: u32) -> u16 {
            unsafe {
                u16::from_le_bytes([
                    execution.read(source),
                    execution.read(source.wrapping_add(1) & 0xffffff),
                ])
            }
        }
        unsafe fn read(execution: &mut Execution<'_>, port: u8) -> u8 {
            unsafe { execution.read_bbus(0x40 + port) }
        }
        unsafe fn write(execution: &mut Execution<'_>, port: u8, val: u8) {
            unsafe { crate::rtl::apu_write(execution, 0x2140 + u32::from(port), val) }
        }
        fn advance(source: &mut u32, count: u32, execution: &mut Execution<'_>) {
            for _ in 0..count {
                let y = (*source as u16).wrapping_add(1);
                if y == 0 {
                    // The original helper returns immediately on its first wrap.
                    execution
                        .ram()
                        .write_byte(2, execution.ram().read_byte(2).wrapping_add(1));
                    *source = (u32::from(execution.ram().read_byte(2)) << 16) | 0x8000;
                    break;
                }
                *source = (*source & 0xff0000) | u32::from(y);
            }
        }
        game.ram().copy_in(0, &source.to_le_bytes()[..3]);
        game.ram().write_byte(0x617, 255);
        game.ram().write_byte(0x618, 255);
        let mut execution = game.execution();
        write(&mut execution, 0, 255);
        execution.ram().write_byte(0x641, 0);
        execution.ram().write_byte(0x642, 0x30);
        let mut budget = 0x3000u16;
        loop {
            let lo = read(&mut execution, 0);
            let hi = read(&mut execution, 1);
            if u16::from_le_bytes([lo, hi]) == 0xbbaa {
                break;
            }
            budget = budget.wrapping_sub(1);
            execution.ram().copy_in(0x641, &budget.to_le_bytes());
            if budget == 0 {
                // The initial handshake timeout never returns to the caller.
                loop {
                    core::hint::spin_loop();
                }
            }
        }
        let mut token = 0xcc;
        loop {
            let mut count = word(&mut execution, source);
            advance(&mut source, 2, &mut execution);
            let target = word(&mut execution, source);
            advance(&mut source, 2, &mut execution);
            write(&mut execution, 2, target as u8);
            write(&mut execution, 3, (target >> 8) as u8);
            write(&mut execution, 1, u8::from(count != 0));
            write(&mut execution, 0, token);
            let mut timeout = 0x1000;
            let mut timedout = false;
            do_until! { read(&mut execution, 0) == token;
                timeout -= 1;
                if timeout == 0 {
                    timedout = true;
                    break;
                }
            }
            if count == 0 || timedout {
                // Completion stores use the source data bank, including ROM banks.
                for port in 1..=3 {
                    execution.write((source & 0xff0000) | 0x2140 | port, 0);
                }
                break;
            }
            token = 0;
            loop {
                let val = execution.read(source);
                advance(&mut source, 1, &mut execution);
                write(&mut execution, 0, token);
                write(&mut execution, 1, val);
                count = count.wrapping_sub(1);
                if count == 0 {
                    break;
                }
                while read(&mut execution, 0) != token {}
                token = token.wrapping_add(1);
            }
            while read(&mut execution, 0) != token {}
            token = token.wrapping_add(4);
            if token == 0 {
                token = 4;
            }
        }
        execution.ram().write_byte(0x617, 0);
        execution.ram().write_byte(0x618, 0);
    }
}
