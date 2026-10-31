//! Ordered native bank installation and physical upload control.
#![forbid(unsafe_code)]
use crate::{apu_ram::ApuRam, rom::RomImage, types::NativePlayerState};

pub fn transfer(image: RomImage, mut source: usize, mut write: impl FnMut(u16, u8)) {
    let mut byte = || {
        let value = image.read(source);
        source = source.checked_add(1).expect("audio upload offset overflow");
        value
    };
    loop {
        // Retain the native byte order of the original word loads.
        let count = u16::from_ne_bytes([byte(), byte()]);
        if count == 0 {
            return;
        }
        let mut target = u16::from_ne_bytes([byte(), byte()]);
        for _ in 0..count {
            write(target, byte());
            target = target.wrapping_add(1);
        }
    }
}

/// Install bytes lazily into the existing APU owner, then commit player state.
/// Register writes are immediate; a failed read retains every earlier effect.
pub fn install_bank(
    state: &mut NativePlayerState,
    ram: &ApuRam,
    image: RomImage,
    source: usize,
    mut write: impl FnMut(u8, u8),
) {
    write(44, 0);
    write(60, 0);
    write(92, 0xff);
    transfer(image, source, |target, value| ram.write(target, value));
    state.port_to_snes[0] = 0;
    state.input_ports[3] = 255;
    state.input_ports[2] = state.input_ports[3];
    state.input_ports[1] = state.input_ports[2];
    state.input_ports[0] = state.input_ports[1];
    state.music_ptr_toplevel = u16::from_ne_bytes([ram.read_at(0x581e), ram.read_at(0x581f)]);
    state.counter_sf0c = 2;
    state.key_off |= !state.is_chan_on;
}

/// One synchronous physical observation or store; no routing link is cached.
pub(crate) enum PhysicalAccess {
    ReadSource(u32),
    ReadPort0,
    ReadPort1,
    WritePort0(u8),
    WritePort1(u8),
    WritePort2(u8),
    WritePort3(u8),
    CompletePort1(u32),
    CompletePort2(u32),
    CompletePort3(u32),
}

/// These WRAM effects precede acquisition of the live backend interval.
pub(crate) fn prepare_physical(source: u32, ram: &crate::game_state::WorkRam) {
    ram.copy_in(0, &source.to_le_bytes()[..3]);
    ram.write_byte(0x617, 255);
    ram.write_byte(0x618, 255);
}

/// Drive the original physical handshake against the same shared WRAM owner.
/// The callback observes live bus/device state once per operation. Timeouts and
/// failures retain their original effects; the initial timeout never returns.
pub(crate) fn physical(
    mut source: u32,
    ram: &crate::game_state::WorkRam,
    mut access: impl FnMut(PhysicalAccess) -> u8,
) {
    fn word(access: &mut impl FnMut(PhysicalAccess) -> u8, source: u32) -> u16 {
        u16::from_le_bytes([
            access(PhysicalAccess::ReadSource(source)),
            access(PhysicalAccess::ReadSource(
                source.wrapping_add(1) & 0xffffff,
            )),
        ])
    }
    fn advance(source: &mut u32, count: u32, ram: &crate::game_state::WorkRam) {
        for _ in 0..count {
            let y = (*source as u16).wrapping_add(1);
            if y == 0 {
                // The original helper returns immediately on its first wrap.
                ram.write_byte(2, ram.read_byte(2).wrapping_add(1));
                *source = (u32::from(ram.read_byte(2)) << 16) | 0x8000;
                break;
            }
            *source = (*source & 0xff0000) | u32::from(y);
        }
    }

    access(PhysicalAccess::WritePort0(255));
    ram.write_byte(0x641, 0);
    ram.write_byte(0x642, 0x30);
    let mut budget = 0x3000u16;
    loop {
        let lo = access(PhysicalAccess::ReadPort0);
        let hi = access(PhysicalAccess::ReadPort1);
        if u16::from_le_bytes([lo, hi]) == 0xbbaa {
            break;
        }
        budget = budget.wrapping_sub(1);
        ram.copy_in(0x641, &budget.to_le_bytes());
        if budget == 0 {
            // The initial handshake timeout never returns to the caller.
            loop {
                core::hint::spin_loop();
            }
        }
    }
    let mut token = 0xcc;
    loop {
        let mut count = word(&mut access, source);
        advance(&mut source, 2, ram);
        let target = word(&mut access, source);
        advance(&mut source, 2, ram);
        access(PhysicalAccess::WritePort2(target as u8));
        access(PhysicalAccess::WritePort3((target >> 8) as u8));
        access(PhysicalAccess::WritePort1(u8::from(count != 0)));
        access(PhysicalAccess::WritePort0(token));
        let mut timeout = 0x1000;
        let mut timedout = false;
        do_until! { access(PhysicalAccess::ReadPort0) == token;
            timeout -= 1;
            if timeout == 0 {
                timedout = true;
                break;
            }
        }
        if count == 0 || timedout {
            // Completion stores use the source data bank, including ROM banks.
            access(PhysicalAccess::CompletePort1(source));
            access(PhysicalAccess::CompletePort2(source));
            access(PhysicalAccess::CompletePort3(source));
            break;
        }
        token = 0;
        loop {
            let val = access(PhysicalAccess::ReadSource(source));
            advance(&mut source, 1, ram);
            access(PhysicalAccess::WritePort0(token));
            access(PhysicalAccess::WritePort1(val));
            count = count.wrapping_sub(1);
            if count == 0 {
                break;
            }
            while access(PhysicalAccess::ReadPort0) != token {}
            token = token.wrapping_add(1);
        }
        while access(PhysicalAccess::ReadPort0) != token {}
        token = token.wrapping_add(4);
        if token == 0 {
            token = 4;
        }
    }
    ram.write_byte(0x617, 0);
    ram.write_byte(0x618, 0);
}
