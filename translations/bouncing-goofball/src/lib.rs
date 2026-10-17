//! Complete A2 Bouncing Goofball actor, with native proximity/sound capabilities.
const SIZE: usize = 0x20000;
const CURRENT: usize = 0xe54;
const ENEMIES: usize = 0xf78;
const EXTENDED: usize = 0x7800;
const INIT_A: usize = 0x1106df;
const INIT_C: usize = 0x1106ef;
const CURVE: usize = 0x110701;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Init,
    Calibrate,
    Main,
    State0,
    State1,
    CloseX,
    IdleAnimation,
    ActiveAnimation,
    InstructionNoop,
    InstructionActivate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Done,
    Proximity(u16),
    Continue(u16),
}
/// The host reuses its native proximity/SFX implementations and complete WRAM
/// publication/copyback. Availability is checked before any owned mutation.
pub trait Services {
    fn available(&self, entry: Entry) -> bool;
    fn close_x(&mut self, ram: &mut [u8], k: u16, distance: u16) -> Result<u16, String>;
    fn sfx2_max6(&mut self, ram: &mut [u8], sound: u16) -> Result<(), String>;
}
fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn valid(k: u16) -> bool {
    k < 0x800 && k & 63 == 0
}
fn animation(ram: &mut [u8], b: usize, list: u16) {
    put(ram, b + 0x1a, list);
    put(ram, b + 0x1c, 1);
    put(ram, b + 0x18, 0);
}
fn curve(rom: &[u8], index: u16) -> Option<u8> {
    // Original C reads ROM before replacing the byte when signed(index-23)>=0.
    let byte = *rom.get(CURVE + usize::from(index))?;
    Some(if (index.wrapping_sub(23) as i16) >= 0 { 255 } else { byte })
}
fn product(rom: &[u8], index: u16, a: u16) -> Option<u16> {
    Some(u16::from(curve(rom, index)?) * u16::from(a as u8))
}
/// Prove the do/while's exact16-bit state trajectory reaches its signed stop.
/// Admission supports trajectories terminating within one65536-step E cycle.
/// A trajectory needing more steps refuses unchanged; no claim is made that
/// every foreign table/parameter combination terminates in original C.
fn calibration(rom: &[u8], mut e: u16, mut d: u16, a: u16, c: u16) -> Option<(u16, u16, u16)> {
    let initial = (e, d);
    for _ in 0..65536 {
        let b = product(rom, e, a)?;
        d = d.wrapping_add(b);
        e = e.wrapping_add(1);
        if (d.wrapping_sub(c) as i16) >= 0 {
            return Some((e, d, b));
        }
        if (e, d) == initial {
            return None;
        }
    }
    None
}
fn calibrate(ram: &mut [u8], rom: &[u8], b: usize, x: usize) -> Result<(), String> {
    let (e, d, last) =
        calibration(rom, word(ram, b + 0x38), word(ram, b + 0x36), word(ram, b + 0x30), word(ram, b + 0x34))
            .ok_or("Bouncing Goofball calibration lost admitted finite ROM domain")?;
    put(ram, b + 0x32, last);
    put(ram, b + 0x36, d);
    put(ram, b + 0x38, e);
    put(ram, x, e);
    put(ram, x + 4, 1);
    put(ram, x + 8, 1);
    Ok(())
}
fn state(ram: &mut [u8], rom: &[u8], b: usize, x: usize, down: bool) -> Result<(), String> {
    let e = word(ram, b + 0x38);
    let index = if down { e.wrapping_add(1) } else { e };
    let p = product(rom, index, word(ram, b + 0x30)).ok_or("Bouncing Goofball state lost admitted ROM read")?;
    put(ram, b + 0x32, p);
    let y = word(ram, b + 6);
    put(ram, b + 6, if down { y.wrapping_add(p >> 8) } else { y.wrapping_sub(p >> 8) });
    if down {
        put(ram, b + 0x38, index);
        if (index.wrapping_sub(word(ram, x)) as i16) >= 0 {
            put(ram, x + 2, 1);
            put(ram, x + 4, 1);
            put(ram, b + 0x3a, 0);
        }
    } else {
        let next = e.wrapping_sub(1);
        put(ram, b + 0x38, next);
        if next & 0x8000 != 0 {
            put(ram, x + 2, 0);
        }
    }
    Ok(())
}
fn current(ram: &[u8]) -> Result<(u16, usize, usize), String> {
    let k = word(ram, CURRENT);
    if !valid(k) {
        return Err("Bouncing Goofball native service changed current slot outside domain".into());
    }
    Ok((k, ENEMIES + usize::from(k), EXTENDED + usize::from(k)))
}
/// None refuses unchanged; accepted errors retain ordered effects and prohibit
/// fallback. Caller authenticates immutable runtime ROM and bank-A2 jp identity.
pub fn execute(
    entry: Entry,
    k: u16,
    jp: u16,
    ram: &mut [u8],
    rom: &[u8],
    services: &mut impl Services,
) -> Result<Option<Output>, String> {
    if ram.len() != SIZE {
        return Ok(None);
    }
    // No-op instruction performs no current-slot lookup in original C.
    if entry == Entry::InstructionNoop {
        return Ok(Some(Output::Continue(jp)));
    }
    let (cur, b, x) = match current(ram) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    let (b, x) = if entry == Entry::Calibrate {
        if !valid(k) {
            return Ok(None);
        }
        (ENEMIES + usize::from(k), EXTENDED + usize::from(k))
    } else {
        (b, x)
    };
    if entry == Entry::Init {
        let parameter = word(ram, b + 0x3c);
        if parameter as u8 >= 8
            || parameter >> 8 >= 9
            || rom.get(INIT_A..INIT_A + 16).is_none()
            || rom.get(INIT_C..INIT_C + 18).is_none()
        {
            return Ok(None);
        }
    }
    let lazy = entry == Entry::Main && word(ram, x + 10) == 0;
    if (entry == Entry::Calibrate || lazy)
        && calibration(rom, word(ram, b + 0x38), word(ram, b + 0x36), word(ram, b + 0x30), word(ram, b + 0x34))
            .is_none()
    {
        return Ok(None);
    }
    if matches!(entry, Entry::State0 | Entry::State1) || entry == Entry::Main && !lazy {
        let selector = if entry == Entry::State0 {
            0
        } else if entry == Entry::State1 {
            1
        } else {
            word(ram, x + 2)
        };
        if selector > 1 {
            return Ok(None);
        }
        let index = word(ram, b + 0x38).wrapping_add(u16::from(selector == 0));
        if product(rom, index, word(ram, b + 0x30)).is_none() {
            return Ok(None);
        }
    }
    if matches!(entry, Entry::CloseX | Entry::InstructionActivate) || entry == Entry::Main && !lazy {
        if !services.available(entry) {
            return Ok(None);
        }
    }
    match entry {
        Entry::Init => {
            put(ram, b + 0x16, 0x804d);
            animation(ram, b, 0x86a7);
            put(ram, x + 2, 1);
            let p = word(ram, b + 0x3c);
            put(ram, b + 0x30, word(rom, INIT_A + usize::from(p as u8) * 2));
            put(ram, b + 0x34, word(rom, INIT_C + usize::from(p >> 8) * 2));
            put(ram, b + 0x36, 0);
            put(ram, b + 0x38, 0);
            put(ram, b + 0x3a, 0);
        }
        Entry::Calibrate => calibrate(ram, rom, b, x)?,
        Entry::Main if lazy => {
            calibrate(ram, rom, b, x)?;
            put(ram, x + 10, 1);
        }
        Entry::Main => {
            put(ram, x + 6, word(ram, b + 0x30));
            let close = services.close_x(ram, cur, word(ram, b + 0x3e))?;
            if close != 0 {
                put(ram, x + 4, 0);
                put(ram, x + 8, 0);
                if word(ram, b + 0x3a) == 0 {
                    put(ram, b + 0x3a, 1);
                    let (_, active, _) = current(ram)?;
                    animation(ram, active, 0x86bf);
                }
            }
            if close != 0 || word(ram, x + 4) == 0 {
                put(ram, b + 0x30, word(ram, x + 6));
                let (_, active, ext) = current(ram)?;
                let selector = word(ram, x + 2);
                if selector > 1 {
                    return Err("Bouncing Goofball service produced invalid state selector".into());
                }
                state(ram, rom, active, ext, selector == 0)?;
            } else if word(ram, x + 8) == 0 {
                put(ram, x + 8, 1);
                let (_, active, _) = current(ram)?;
                animation(ram, active, 0x86a7);
            }
        }
        Entry::State0 => state(ram, rom, b, x, true)?,
        Entry::State1 => state(ram, rom, b, x, false)?,
        Entry::CloseX => return Ok(Some(Output::Proximity(services.close_x(ram, cur, word(ram, b + 0x3e))?))),
        Entry::IdleAnimation => animation(ram, b, 0x86a7),
        Entry::ActiveAnimation => animation(ram, b, 0x86bf),
        Entry::InstructionActivate => {
            put(ram, x + 4, 0);
            services.sfx2_max6(ram, 0xe)?;
            return Ok(Some(Output::Continue(jp)));
        }
        Entry::InstructionNoop => unreachable!(),
    }
    Ok(Some(Output::Done))
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Native {
        close: u16,
        failed: bool,
    }
    impl Services for Native {
        fn available(&self, _: Entry) -> bool {
            true
        }
        fn close_x(&mut self, _: &mut [u8], k: u16, distance: u16) -> Result<u16, String> {
            assert_eq!((k, distance), (0, 7));
            Ok(self.close)
        }
        fn sfx2_max6(&mut self, ram: &mut [u8], sound: u16) -> Result<(), String> {
            assert_eq!(sound, 14);
            assert_eq!(word(ram, EXTENDED + 4), 0);
            if self.failed { Err("sound terminal".into()) } else { Ok(()) }
        }
    }
    #[test]
    fn actual_a2_rom_offsets_distinguish_adjacent_bank() {
        // Literal LoROM addresses independently transcribed from A2:86DF,
        // A2:86EF and A2:8701, not derived from candidate constants.
        let mut rom = vec![0; 0x110702];
        put(&mut rom, 0x1106df, 2);
        put(&mut rom, 0x1106ef, 6);
        rom[0x110701] = 3;
        put(&mut rom, 0x1086df, 9);
        put(&mut rom, 0x1086ef, 99);
        rom[0x108701] = 17;
        let mut ram = vec![0; SIZE];
        let mut native = Native { close: 0, failed: false };
        execute(Entry::Init, 0, 0, &mut ram, &rom, &mut native).unwrap();
        assert_eq!(word(&ram, ENEMIES + 0x30), 2);
        assert_eq!(word(&ram, ENEMIES + 0x34), 6);
        execute(Entry::Calibrate, 0, 0, &mut ram, &rom, &mut native).unwrap();
        assert_eq!(word(&ram, ENEMIES + 0x32), 6);
        assert_eq!(word(&ram, ENEMIES + 0x38), 1);
    }
    #[test]
    fn main_close_runs_active_animation_and_upward_underflow_transition() {
        let mut ram = vec![0; SIZE];
        let mut rom = vec![0; CURVE + 1];
        rom[CURVE] = 255;
        put(&mut ram, EXTENDED + 10, 1);
        put(&mut ram, EXTENDED + 2, 1);
        put(&mut ram, EXTENDED + 4, 1);
        put(&mut ram, ENEMIES + 0x30, 2);
        put(&mut ram, ENEMIES + 0x3e, 7);
        put(&mut ram, ENEMIES + 6, 0);
        let mut native = Native { close: 1, failed: false };
        execute(Entry::Main, 0, 0, &mut ram, &rom, &mut native).unwrap();
        assert_eq!(word(&ram, ENEMIES + 0x1a), 0x86bf);
        assert_eq!(word(&ram, ENEMIES + 6), 0xffff);
        assert_eq!(word(&ram, ENEMIES + 0x38), 0xffff);
        assert_eq!(word(&ram, EXTENDED + 2), 0);
        assert_eq!((word(&ram, EXTENDED + 4), word(&ram, EXTENDED + 8)), (0, 0));
    }
    #[test]
    fn calibration_uses_low_multiplicand_and_exact_signed_stop() {
        let mut ram = vec![0; SIZE];
        let mut rom = vec![0; CURVE + 24];
        rom[CURVE] = 3;
        rom[CURVE + 1] = 4;
        put(&mut ram, ENEMIES + 0x30, 0xab02);
        put(&mut ram, ENEMIES + 0x34, 14);
        let mut native = Native { close: 0, failed: false };
        execute(Entry::Calibrate, 0, 0, &mut ram, &rom, &mut native).unwrap();
        assert_eq!((word(&ram, ENEMIES + 0x32), word(&ram, ENEMIES + 0x36), word(&ram, ENEMIES + 0x38)), (8, 14, 2));
        assert_eq!((word(&ram, EXTENDED), word(&ram, EXTENDED + 4), word(&ram, EXTENDED + 8)), (2, 1, 1));
    }
    #[test]
    fn states_preserve_fraction_and_read_before_curve_substitution() {
        let mut ram = vec![0; SIZE];
        let mut rom = vec![0; CURVE + 24];
        put(&mut ram, ENEMIES + 0x30, 2);
        put(&mut ram, ENEMIES + 0x38, 22);
        put(&mut ram, EXTENDED, 23);
        put(&mut ram, ENEMIES + 6, 0xffff);
        put(&mut ram, ENEMIES + 8, 0x1234);
        let mut native = Native { close: 0, failed: false };
        execute(Entry::State0, 0, 0, &mut ram, &rom, &mut native).unwrap();
        assert_eq!(word(&ram, ENEMIES + 6), 0);
        assert_eq!(word(&ram, ENEMIES + 8), 0x1234);
        assert_eq!(word(&ram, EXTENDED + 2), 1);
        rom.truncate(CURVE + 23);
        put(&mut ram, ENEMIES + 0x38, 23);
        let before = ram.clone();
        assert_eq!(execute(Entry::State1, 0, 0, &mut ram, &rom, &mut native).unwrap(), None);
        assert_eq!(ram, before);
    }
    #[test]
    fn activate_ignores_k_preserves_jp_and_keeps_terminal_write() {
        let mut ram = vec![0; SIZE];
        put(&mut ram, EXTENDED + 4, 1);
        let mut native = Native { close: 0, failed: true };
        assert_eq!(
            execute(Entry::InstructionActivate, 1, 0x88c9, &mut ram, &[], &mut native),
            Err("sound terminal".into())
        );
        assert_eq!(word(&ram, EXTENDED + 4), 0);
        put(&mut ram, CURRENT, 1);
        assert_eq!(
            execute(Entry::InstructionNoop, 1, 0xffff, &mut ram, &[], &mut native),
            Ok(Some(Output::Continue(0xffff)))
        );
    }
}
