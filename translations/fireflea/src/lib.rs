//! Fireflea's complete recovered actor boundary; native services remain external.
const WRAM: usize = 0x20000;
const CURRENT: usize = 0xe54;
const ENEMIES: usize = 0xf78;
const EXTENDED: usize = 0x7800;
const SPEEDS: usize = 0x110187;
const AMPLITUDES: usize = 0x118d1d;
const DARKNESS: usize = 0x177e;

/// The six helpers retain their explicit k; Touch uses k only for death animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Init,
    Origin,
    Phase,
    Speed,
    Amplitude,
    Position,
    Bounds,
    Main,
    Touch,
    Powerbomb,
    Shot,
    Common,
}

/// Implementations publish and copy back full canonical WRAM synchronously.
/// Availability must establish the entire entry's native capability before any
/// actor write. Accepted service failure is terminal, never permission to retry C.
pub trait Services {
    fn available(&self, entry: Entry) -> bool;
    fn cosine(&mut self, ram: &mut [u8], angle: u16, amplitude: u16) -> Result<u16, String>;
    fn sine(&mut self, ram: &mut [u8], angle: u16, amplitude: u16) -> Result<u16, String>;
    fn touch(&mut self, ram: &mut [u8]) -> Result<(), String>;
    fn powerbomb(&mut self, ram: &mut [u8]) -> Result<(), String>;
    fn shot(&mut self, ram: &mut [u8]) -> Result<(), String>;
    fn death(&mut self, ram: &mut [u8], k: u16, animation: u16) -> Result<(), String>;
}

// Same canonical little-endian word/slot representation as restart/waver and
// restart/ripper. These small private helpers introduce no public memory protocol.
fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn slot(k: u16) -> bool {
    k < 0x800 && k & 63 == 0
}
fn row(value: u16) -> bool {
    value <= 516 && value & 3 == 0
}
fn speed(ram: &mut [u8], rom: &[u8], b: usize, x: usize) {
    let mut selector = 8 * (word(ram, b + 0x3e) & 0xff);
    if word(ram, b + 0x3c) & 1 == 0 {
        selector += 4;
    }
    put(ram, x + 4, selector);
    put(ram, b + 0x32, word(rom, SPEEDS + usize::from(selector)));
    put(ram, b + 0x30, word(rom, SPEEDS + usize::from(selector) + 2));
}
fn amplitude(ram: &mut [u8], rom: &[u8], b: usize) {
    put(ram, b + 0x34, word(rom, AMPLITUDES + usize::from(word(ram, b + 0x3e) >> 8) * 2) & 0xff);
}
fn bounds(ram: &mut [u8], b: usize, x: usize) {
    put(ram, x, word(ram, b + 6).wrapping_sub(word(ram, b + 0x34)));
    put(ram, x + 2, word(ram, b + 0x34).wrapping_add(word(ram, b + 6)));
}
fn darkness(ram: &mut [u8]) {
    let level = word(ram, DARKNESS);
    if (level.wrapping_sub(12) as i16) < 0 {
        put(ram, DARKNESS, level.wrapping_add(2));
    }
}
fn common(ram: &mut [u8]) -> Result<(), String> {
    let current = word(ram, CURRENT);
    if !slot(current) {
        return Err("Fireflea service changed current slot outside canonical domain".into());
    }
    if word(ram, ENEMIES + usize::from(current) + 0x14) == 0 {
        darkness(ram);
    }
    Ok(())
}
fn position(ram: &mut [u8], b: usize, services: &mut impl Services) -> Result<(), String> {
    let cosine = services.cosine(ram, word(ram, b + 0x36), word(ram, b + 0x34))?;
    // C retains its E pointer and rereads origin after synchronous foreign effects.
    put(ram, b + 2, word(ram, b + 0x38).wrapping_add(cosine));
    let sine = services.sine(ram, word(ram, b + 0x36), word(ram, b + 0x34))?;
    put(ram, b + 6, word(ram, b + 0x3a).wrapping_add(sine));
    Ok(())
}

/// Ok(false) refuses unchanged before any write/request. Err after admission
/// is terminal to the invocation and may retain ordered writes/service effects.
/// The caller authenticates this immutable runtime ROM against the native image.
pub fn execute(entry: Entry, k: u16, ram: &mut [u8], rom: &[u8], services: &mut impl Services) -> Result<bool, String> {
    if ram.len() != WRAM {
        return Ok(false);
    }
    let current = word(ram, CURRENT);
    if !slot(current) {
        return Ok(false);
    }
    let helper = matches!(
        entry,
        Entry::Origin | Entry::Phase | Entry::Speed | Entry::Amplitude | Entry::Position | Entry::Bounds
    );
    if (helper || entry == Entry::Touch) && !slot(k) {
        return Ok(false);
    }
    let selected = if helper { k } else { current };
    let b = ENEMIES + usize::from(selected);
    let x = EXTENDED + usize::from(selected);
    let circular = word(ram, b + 0x3c) & 2 != 0;
    let needs_speed = matches!(entry, Entry::Init | Entry::Speed) || (entry == Entry::Main && !circular);
    let needs_amplitude = matches!(entry, Entry::Init | Entry::Amplitude);
    if needs_speed && rom.get(SPEEDS..SPEEDS + 520).is_none() {
        return Ok(false);
    }
    if matches!(entry, Entry::Init | Entry::Speed) && word(ram, b + 0x3e) & 0xff > 64 {
        return Ok(false);
    }
    if entry == Entry::Main && !circular && !row(word(ram, x + 4)) {
        return Ok(false);
    }
    if needs_amplitude && (word(ram, b + 0x3e) >> 8 >= 8 || rom.get(AMPLITUDES..AMPLITUDES + 16).is_none()) {
        return Ok(false);
    }
    let foreign = matches!(entry, Entry::Position | Entry::Touch | Entry::Powerbomb | Entry::Shot)
        || (matches!(entry, Entry::Init | Entry::Main) && circular);
    if foreign && !services.available(entry) {
        return Ok(false);
    }
    match entry {
        Entry::Init => {
            put(ram, b + 0x1a, 0x8c2f);
            if circular {
                put(ram, b + 0x38, word(ram, b + 2));
                put(ram, b + 0x3a, word(ram, b + 6));
                put(ram, b + 0x36, word(ram, b + 0x3c) & 0xff00);
            }
            speed(ram, rom, b, x);
            amplitude(ram, rom, b);
            if circular {
                position(ram, b, services)?;
            } else {
                bounds(ram, b, x);
            }
        }
        Entry::Origin => {
            put(ram, b + 0x38, word(ram, b + 2));
            put(ram, b + 0x3a, word(ram, b + 6));
        }
        Entry::Phase => put(ram, b + 0x36, word(ram, b + 0x3c) & 0xff00),
        Entry::Speed => speed(ram, rom, b, x),
        Entry::Amplitude => amplitude(ram, rom, b),
        Entry::Position => position(ram, b, services)?,
        Entry::Bounds => bounds(ram, b, x),
        Entry::Main if circular => {
            // Each C call rereads D; a foreign cosine can change D before sine.
            let cosine = services.cosine(ram, word(ram, b + 0x36) >> 8, word(ram, b + 0x34))?;
            put(ram, b + 2, word(ram, b + 0x38).wrapping_add(cosine));
            let sine = services.sine(ram, word(ram, b + 0x36) >> 8, word(ram, b + 0x34))?;
            put(ram, b + 6, word(ram, b + 0x3a).wrapping_add(sine));
            // Unaligned LE16 at A+1 spans high(A),low(B); do not use D:C.
            let delta = u16::from_le_bytes([ram[b + 0x31], ram[b + 0x32]]);
            put(ram, b + 0x36, word(ram, b + 0x36).wrapping_add(delta));
        }
        Entry::Main => {
            let selector = usize::from(word(ram, x + 4));
            let delta = (u32::from(word(rom, SPEEDS + selector)) << 16) | u32::from(word(rom, SPEEDS + selector + 2));
            let old = (u32::from(word(ram, b + 6)) << 16) | u32::from(word(ram, b + 8));
            let next = old.wrapping_add(delta);
            put(ram, b + 6, (next >> 16) as u16);
            put(ram, b + 8, next as u16);
            let y = word(ram, b + 6);
            if (y.wrapping_sub(word(ram, x)) as i16) < 0 || y.wrapping_sub(word(ram, x + 2)) as i16 >= 0 {
                put(ram, x + 4, word(ram, x + 4) ^ 4);
            }
        }
        Entry::Touch => {
            services.touch(ram)?;
            services.death(ram, k, 0)?;
            darkness(ram);
        }
        Entry::Powerbomb => {
            services.powerbomb(ram)?;
            common(ram)?;
        }
        Entry::Shot => {
            services.shot(ram)?;
            common(ram)?;
        }
        Entry::Common => common(ram)?,
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Native {
        calls: Vec<&'static str>,
    }
    impl Services for Native {
        fn available(&self, _: Entry) -> bool {
            true
        }
        fn cosine(&mut self, ram: &mut [u8], angle: u16, amplitude: u16) -> Result<u16, String> {
            assert_eq!((angle, amplitude), (0x12, 7));
            self.calls.push("cosine");
            put(ram, ENEMIES + 0x36, 0x3400);
            Ok(0xfffe)
        }
        fn sine(&mut self, _: &mut [u8], angle: u16, amplitude: u16) -> Result<u16, String> {
            assert_eq!((angle, amplitude), (0x34, 7));
            self.calls.push("sine");
            Ok(3)
        }
        fn touch(&mut self, _: &mut [u8]) -> Result<(), String> {
            self.calls.push("touch");
            Ok(())
        }
        fn powerbomb(&mut self, ram: &mut [u8]) -> Result<(), String> {
            self.calls.push("powerbomb");
            put(ram, ENEMIES + 0x14, 0);
            Ok(())
        }
        fn shot(&mut self, _: &mut [u8]) -> Result<(), String> {
            self.calls.push("shot");
            Err("shot terminal".into())
        }
        fn death(&mut self, _: &mut [u8], k: u16, animation: u16) -> Result<(), String> {
            assert_eq!((k, animation), (64, 0));
            self.calls.push("death");
            Ok(())
        }
    }
    #[test]
    fn circular_calls_reread_phase_and_unaligned_speed_bytes() {
        let mut ram = vec![0; WRAM];
        for (offset, value) in
            [(0x3c, 2), (0x36, 0x1200), (0x34, 7), (0x38, 1), (0x3a, 0xfffe), (0x30, 0xab00), (0x32, 0x00cd)]
        {
            put(&mut ram, ENEMIES + offset, value);
        }
        let mut native = Native::default();
        assert_eq!(execute(Entry::Main, 0xffff, &mut ram, &[], &mut native), Ok(true));
        assert_eq!(native.calls, ["cosine", "sine"]);
        assert_eq!(word(&ram, ENEMIES + 2), 0xffff);
        assert_eq!(word(&ram, ENEMIES + 6), 1);
        assert_eq!(word(&ram, ENEMIES + 0x36), 0x01ab);
    }
    #[test]
    fn vertical_carry_and_signed_boundary_flip() {
        let mut ram = vec![0; WRAM];
        let mut rom = vec![0; SPEEDS + 520];
        put(&mut rom, SPEEDS, 0);
        put(&mut rom, SPEEDS + 2, 0x8000);
        put(&mut ram, ENEMIES + 6, 0x7fff);
        put(&mut ram, ENEMIES + 8, 0xffff);
        put(&mut ram, EXTENDED, 0x7fff);
        put(&mut ram, EXTENDED + 2, 0x8000);
        assert_eq!(execute(Entry::Main, 0, &mut ram, &rom, &mut Native::default()), Ok(true));
        assert_eq!(word(&ram, ENEMIES + 6), 0x8000);
        assert_eq!(word(&ram, ENEMIES + 8), 0x7fff);
        assert_eq!(word(&ram, EXTENDED + 4), 4);
        let before = ram.clone();
        put(&mut ram, EXTENDED + 4, 520);
        let invalid = ram.clone();
        assert_eq!(execute(Entry::Main, 0, &mut ram, &rom, &mut Native::default()), Ok(false));
        assert_eq!(ram, invalid);
        assert_ne!(ram, before);
    }
    #[test]
    fn damage_effect_order_darkness_sign_and_terminal_failure() {
        let mut ram = vec![0; WRAM];
        put(&mut ram, DARKNESS, 11);
        let mut native = Native::default();
        execute(Entry::Touch, 64, &mut ram, &[], &mut native).unwrap();
        assert_eq!(native.calls, ["touch", "death"]);
        assert_eq!(word(&ram, DARKNESS), 13);
        put(&mut ram, DARKNESS, 12);
        put(&mut ram, ENEMIES + 0x14, 1);
        execute(Entry::Powerbomb, 0, &mut ram, &[], &mut native).unwrap();
        assert_eq!(word(&ram, DARKNESS), 12);
        put(&mut ram, DARKNESS, 0x800c);
        execute(Entry::Common, 0, &mut ram, &[], &mut native).unwrap();
        assert_eq!(word(&ram, DARKNESS), 0x800e);
        assert_eq!(execute(Entry::Shot, 0, &mut ram, &[], &mut native), Err("shot terminal".into()));
        assert_eq!(word(&ram, DARKNESS), 0x800e);
    }
}
