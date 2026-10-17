//! Entire YappingMaw actor; shared projectile initialization remains native.
const SIZE: usize = 0x20000;
const ENEMY: usize = 0xf78;
const CURRENT: usize = 0xe54;
const TABLE: usize = 0x142097; // A8:A097 LoROM
const PAIRS: usize = 0x1420a7;
const SINE: usize = 0x1031c3; // A0:B1C3
const QUAD: usize = 0x11038f; // A2:838F
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Instruction(u8),
    Init,
    Main,
    Dispatch(u32),
    Function(u8),
    Touch,
    Shot,
    Frozen,
}
/// Synchronous existing native seams; errors after admission are terminal.
/// Every capability for an entry must be available before any owned write.
pub trait Services {
    fn available(&self, entry: Entry) -> bool;
    fn spawn_body(&mut self, ram: &mut [u8], part: u16, k: u16, definition: u16) -> Result<(), String>;
    fn sprite(&mut self, ram: &mut [u8], x: u16, y: u16, kind: u16, palette: u16) -> Result<u16, String>;
    fn screen(&mut self, ram: &mut [u8]) -> Result<u16, String>;
    fn geometry(&mut self, ram: &mut [u8], from: (u16, u16), to: (u16, u16)) -> Result<(u16, u16), String>;
    fn samus_code(&mut self, ram: &mut [u8], code: u16) -> Result<(), String>;
    fn samus_adjust(&mut self, ram: &mut [u8]) -> Result<(), String>;
    fn shot(&mut self, ram: &mut [u8]) -> Result<(), String>;
    fn frozen(&mut self, ram: &mut [u8]) -> Result<(), String>;
    fn sound2(&mut self, ram: &mut [u8], sound: u16) -> Result<(), String>;
    fn write_multiplier(&mut self, value: u8) -> Result<(), String>;
    fn multiply(&mut self, a: u8, b: u8) -> Result<u16, String>;
}
fn word(a: &[u8], p: usize) -> u16 {
    u16::from_le_bytes([a[p], a[p + 1]])
}
fn put(a: &mut [u8], p: usize, v: u16) {
    a[p..p + 2].copy_from_slice(&v.to_le_bytes());
}
fn valid(k: u16) -> bool {
    k < 0x800 && k & 63 == 0
}
fn current(ram: &[u8]) -> Result<(u16, usize), String> {
    let k = word(ram, CURRENT);
    if !valid(k) {
        return Err("YappingMaw foreign current slot invalid".into());
    }
    Ok((k, ENEMY + usize::from(k)))
}
fn ext(b: usize, n: usize) -> usize {
    let k = b - ENEMY;
    match n {
        0..=0x1f => 0x7800 + k + n * 2,
        0x20..=0x3f => 0x8000 + k + (n - 0x20) * 2,
        _ => 0x8800 + k + (n - 0x40) * 2,
    }
}
fn get(ram: &[u8], b: usize, n: usize) -> u16 {
    word(ram, ext(b, n))
}
fn set(ram: &mut [u8], b: usize, n: usize, v: u16) {
    put(ram, ext(b, n), v)
}
fn signed(v: u16) -> bool {
    v & 0x8000 != 0
}
fn links(ram: &[u8], b: usize) -> bool {
    (0x40..=0x43).all(|n| {
        let j = get(ram, b, n);
        j < 36 && j & 1 == 0
    }) && get(ram, b, 0x46) < 64
        && get(ram, b, 0x46) & 1 == 0
}
fn ensure_links(ram: &[u8], b: usize) -> Result<(), String> {
    if links(ram, b) { Ok(()) } else { Err("YappingMaw foreign linked projectile/sprite slot invalid".into()) }
}
fn function_selector(address: u32) -> Option<u8> {
    match address {
        0xa8a235 => Some(1),
        0xa8a28c => Some(2),
        0xa8a445 => Some(8),
        0xa8a68a => Some(11),
        _ => None,
    }
}
/// Func16 and17, including the observable unconditional WRMPYA write.
pub fn trig(a: u16, m: u16, cosine: bool, rom: &[u8], s: &mut impl Services) -> Result<u16, String> {
    let a = if cosine { a.wrapping_sub(64) } else { a };
    let index = usize::from(a.wrapping_neg() as u8);
    let raw = rom.get(SINE + index * 2..SINE + index * 2 + 2).ok_or("YappingMaw sine ROM read unavailable")?;
    let value = i16::from_le_bytes([raw[0], raw[1]]);
    let negative = value < 0;
    let magnitude = if negative { (value as u16).wrapping_neg() } else { value as u16 };
    let high = (magnitude >> 8) as u8;
    s.write_multiplier(high)?;
    if m as u8 == 0 {
        return Ok(0);
    }
    let product = s.multiply(high, m as u8)?;
    Ok(if negative { (product.wrapping_neg() >> 8).wrapping_mul(2) | 0xff00 } else { (product >> 8).wrapping_mul(2) })
}
fn math(ram: &mut [u8], rom: &[u8], b: usize, a: u16, m: u16, cos: bool, s: &mut impl Services) -> Result<u16, String> {
    let _ = ram;
    let _ = b;
    trig(a, m, cos, rom, s)
}
fn fn_run(n: u8, k: u16, a: u16, m: u16, ram: &mut [u8], rom: &[u8], s: &mut impl Services) -> Result<u16, String> {
    let (_, b) = current(ram)?;
    match n {
        1 => {
            let (distance, angle) =
                s.geometry(ram, (word(ram, b + 2), word(ram, b + 6)), (word(ram, 0xaf6), word(ram, 0xafa)))?;
            let abs = if signed(distance) { distance.wrapping_neg() } else { distance };
            set(ram, b, 8, abs);
            if signed(abs.wrapping_sub(32)) {
                set(ram, b, 0x35, 48);
            } else if signed(abs.wrapping_sub(word(ram, b + 0x3a))) {
                if !signed(abs.wrapping_sub(64)) {
                    set(ram, b, 8, 64);
                }
                set(ram, b, 0xa, angle);
                put(ram, b + 0x30, 0xa28c);
            }
        }
        2 => {
            for p in [0x32, 0x34, 0x36] {
                put(ram, b + p, 0);
            }
            set(ram, b, 9, get(ram, b, 8) >> 1);
            let angle = 64u16.wrapping_sub(get(ram, b, 0xa)) as u8;
            set(ram, b, 0xb, u16::from(angle));
            set(ram, b, 0x2f, u16::from(angle >= 128));
            fn_run(3, k, 0, 0, ram, rom, s)?;
            let selector = 2 * u16::from(get(ram, b, 0xa).wrapping_add(16) as u8 >> 5);
            set(ram, b, 0x34, selector);
            put(ram, b + 0x1a, word(rom, TABLE + usize::from(selector)));
            put(ram, b + 0x1c, 1);
            put(ram, b + 0x18, 0);
            set(ram, b, 0x32, word(rom, PAIRS + usize::from(selector) * 2));
            set(ram, b, 0x33, word(rom, PAIRS + 2 + usize::from(selector) * 2));
            put(ram, b + 0x30, 0xa445);
        }
        3 => {
            let x = math(ram, rom, b, 128, get(ram, b, 9), true, s)?;
            set(ram, b, 0xe, x);
            let y = math(ram, rom, b, 128, get(ram, b, 9) >> 1, false, s)?;
            set(ram, b, 0xf, y);
        }
        4..=7 => {
            let distance = get(ram, b, usize::from(n - 4) * 2);
            let x = math(ram, rom, b, 0, distance, true, s)?;
            set(ram, b, 0x10, x);
            let y = math(ram, rom, b, 0, distance, false, s)?;
            set(ram, b, 0x11, y);
            let x = math(ram, rom, b, get(ram, b, 0xb), distance, true, s)?;
            set(ram, b, 0x27 + usize::from(n - 4) * 2, x.wrapping_sub(get(ram, b, 0x10)));
            let y = math(ram, rom, b, get(ram, b, 0xb), distance, false, s)?;
            set(ram, b, 0x28 + usize::from(n - 4) * 2, y.wrapping_sub(get(ram, b, 0x11)));
        }
        8 => {
            let quarter = word(ram, b + 0x32) >> 2;
            for i in 1..=4 {
                let delta = quarter.wrapping_mul(i as u16);
                set(
                    ram,
                    b,
                    0x20 + i,
                    if get(ram, b, 0x2f) != 0 { 128u16.wrapping_add(delta) } else { 128u16.wrapping_sub(delta) },
                );
            }
            let distance = get(ram, b, 9);
            for i in 0..4 {
                let v = math(ram, rom, b, get(ram, b, 0x21 + i), distance, true, s)?;
                set(ram, b, i * 2, v.wrapping_sub(get(ram, b, 0xe)));
            }
            for i in 0..4 {
                let v = math(ram, rom, b, get(ram, b, 0x21 + i), distance >> 1, false, s)?;
                set(ram, b, i * 2 + 1, v.wrapping_sub(get(ram, b, 0xf)));
            }
            for f in 4..=7 {
                fn_run(f, k, 0, 0, ram, rom, s)?;
            }
            for i in 0..8 {
                set(ram, b, i, get(ram, b, i).wrapping_add(get(ram, b, 0x27 + i)));
            }
            put(ram, b + 2, get(ram, b, 6).wrapping_add(get(ram, b, 0xc)));
            put(ram, b + 6, get(ram, b, 7).wrapping_add(get(ram, b, 0xd)));
            let current_k = current(ram)?.0;
            fn_run(9, current_k, 0, 0, ram, rom, s)?;
            if !signed(word(ram, b + 0x32)) {
                if !signed(word(ram, b + 0x32).wrapping_sub(128)) {
                    put(ram, b + 0x32, 128);
                    put(ram, b + 0x34, 0);
                    put(ram, b + 0x36, word(ram, b + 0x36).wrapping_add(4));
                }
                if get(ram, b, 0x30) != 0 {
                    fn_run(10, k, 0, 0, ram, rom, s)?;
                }
                return Ok(0);
            }
            put(ram, b + 0x30, 0xa68a);
            set(ram, b, 0x35, 48);
            let selector = get(ram, b, 0x34);
            put(
                ram,
                b + 0x1a,
                match selector {
                    4 => 0xa01f,
                    12 => 0xa03d,
                    _ => 0xa025,
                },
            );
            if word(ram, b + 0x3e) == 0 {
                put(
                    ram,
                    b + 0x1a,
                    match selector {
                        4 => 0xa05b,
                        12 => 0xa079,
                        _ => 0xa061,
                    },
                );
                if selector != 4 && selector != 12 {
                    return Ok(0);
                }
                if get(ram, b, 0x30) != 0 {
                    fn_run(10, k, 0, 0, ram, rom, s)?;
                }
            }
        }
        9 => {
            if !valid(k) {
                return Err("YappingMaw speed explicit slot invalid".into());
            }
            let b = ENEMY + usize::from(k);
            let offset = usize::from(word(ram, b + 0x36) & 0xfffe);
            let bytes =
                rom.get(QUAD + offset..QUAD + offset + 4).ok_or("YappingMaw quadratic speed ROM unavailable")?;
            let delta = (u32::from(word(bytes, 2)) << 16) | u32::from(word(bytes, 0));
            let old = (u32::from(word(ram, b + 0x32)) << 16) | u32::from(word(ram, b + 0x34));
            let value = old.wrapping_add(delta);
            put(ram, b + 0x32, (value >> 16) as u16);
            put(ram, b + 0x34, value as u16);
            put(ram, b + 0x36, word(ram, b + 0x36).wrapping_add(8));
        }
        10 => {
            s.samus_code(ram, 3)?;
            let (_, b) = current(ram)?;
            put(ram, 0xaf6, get(ram, b, 0x32).wrapping_add(word(ram, b + 2)));
            put(ram, 0xafa, get(ram, b, 0x33).wrapping_add(word(ram, b + 6)));
            s.samus_adjust(ram)?;
        }
        11 => {
            if get(ram, b, 0x30) != 0 {
                fn_run(10, k, 0, 0, ram, rom, s)?;
            }
            let count = word(ram, b + 0x38).wrapping_sub(1);
            put(ram, b + 0x38, count);
            if signed(count) && word(ram, 0xa60) != 0xe91d {
                put(ram, 0xa60, 0xe913);
                set(ram, b, 0x30, 0);
                set(ram, b, 0x35, 48);
                put(ram, b + 0x38, 64);
                put(ram, b + 0x30, 0xa235);
            }
        }
        12..=15 => {
            ensure_links(ram, b)?;
            let part = usize::from(n - 12);
            let j = usize::from(get(ram, b, 0x40 + part));
            let (dx, dy) =
                if part == 0 { (0, 0) } else { (get(ram, b, (part - 1) * 2), get(ram, b, (part - 1) * 2 + 1)) };
            put(ram, 0x1a4b + j, get(ram, b, 0xc).wrapping_add(dx));
            put(ram, 0x1a93 + j, get(ram, b, 0xd).wrapping_add(dy));
        }
        16 => return trig(a, m, true, rom, s),
        17 => return trig(a, m, false, rom, s),
        18 => {
            if k >= 36 || k & 1 != 0 {
                return Err("YappingMaw projectile palette slot invalid".into());
            }
            let at = 0x19bb + usize::from(k);
            put(ram, at, palette(word(ram, at), get(ram, b, 0x45), word(ram, b + 0x26)));
        }
        19 => {
            ensure_links(ram, b)?;
            let at = 0xf078 + usize::from(get(ram, b, 0x46));
            put(ram, at, palette(word(ram, at), get(ram, b, 0x45), word(ram, b + 0x26)));
        }
        _ => return Err("YappingMaw function outside complete owned scope".into()),
    }
    Ok(0)
}
fn palette(old: u16, saved: u16, timer: u16) -> u16 {
    let normal = saved | (old & 0xf1ff);
    if timer != 0 && (!signed(timer.wrapping_sub(90)) || timer & 2 != 0) { (old & 0xf1ff) | 0xc00 } else { normal }
}

/// Refusal occurs before canonical mutation. Accepted error retains ordered
/// owned/foreign effects and is terminal; no C fallback is permitted.
/// jp is byte-addressed bankA8 continuation; k serves Func9/18 only; a/m serve
/// pure Func16/17. Host authenticates immutable ROM and all native capabilities.
pub fn execute(
    entry: Entry,
    k: u16,
    jp: u16,
    a: u16,
    m: u16,
    ram: &mut [u8],
    rom: &[u8],
    s: &mut impl Services,
) -> Result<Option<u16>, String> {
    if ram.len() != SIZE {
        return Ok(None);
    }
    let (_, b) = match current(ram) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    if matches!(entry,Entry::Instruction(n) if !(1..=7).contains(&n))
        || matches!(entry,Entry::Function(n) if !(1..=19).contains(&n))
    {
        return Ok(None);
    }
    if let Entry::Dispatch(address) = entry {
        if function_selector(address).is_none() {
            return Ok(None);
        }
    }
    if entry == Entry::Main && function_selector(0xa80000 | u32::from(word(ram, b + 0x30))).is_none() {
        return Ok(None);
    }
    if matches!(entry, Entry::Function(9)) && !valid(k) {
        return Ok(None);
    }
    if matches!(entry, Entry::Function(18)) && (k >= 36 || k & 1 != 0) {
        return Ok(None);
    }
    let rom_needed = matches!(
        entry,
        Entry::Instruction(2..=7)
            | Entry::Init
            | Entry::Main
            | Entry::Dispatch(_)
            | Entry::Function(2..=9)
            | Entry::Function(16..=17)
    );
    if rom_needed && (rom.get(TABLE..PAIRS + 32).is_none() || rom.get(SINE..SINE + 512).is_none()) {
        return Ok(None);
    }
    let speed_needed = matches!(entry, Entry::Main | Entry::Dispatch(0xa8a445) | Entry::Function(8..=9));
    if speed_needed {
        let base = if entry == Entry::Function(9) { ENEMY + usize::from(k) } else { b };
        let offset = usize::from(word(ram, base + 0x36) & 0xfffe);
        if rom.get(QUAD + offset..QUAD + offset + 4).is_none() {
            return Ok(None);
        }
    }
    if matches!(entry, Entry::Main | Entry::Shot | Entry::Frozen | Entry::Function(12..=15) | Entry::Function(19))
        && !links(ram, b)
    {
        return Ok(None);
    }
    let foreign = matches!(
        entry,
        Entry::Init
            | Entry::Main
            | Entry::Shot
            | Entry::Frozen
            | Entry::Dispatch(_)
            | Entry::Instruction(1)
            | Entry::Function(1..=8)
            | Entry::Function(10..=11)
            | Entry::Function(16..=17)
    );
    if foreign && !s.available(entry) {
        return Ok(None);
    }
    match entry {
        Entry::Instruction(n) => {
            if n == 1 {
                if get(ram, b, 0x36) == 0 {
                    s.sound2(ram, 0x2f)?;
                }
            } else {
                let offset = match n {
                    2 => 4,
                    3 => 0,
                    4 => 28,
                    5 => 12,
                    6 => 16,
                    7 => 20,
                    _ => unreachable!(),
                };
                set(ram, b, 0x32, word(rom, PAIRS + offset));
                set(ram, b, 0x33, word(rom, PAIRS + offset + 2));
            }
            return Ok(Some(jp));
        }
        Entry::Init => {
            set(ram, b, 0xc, word(ram, b + 2));
            set(ram, b, 0xd, word(ram, b + 6));
            for n in 0..8 {
                set(ram, b, n, 0);
            }
            set(ram, b, 0x30, 0);
            put(ram, b + 0x3a, word(ram, b + 0x3c));
            put(ram, b + 0x38, 64);
            let normal = word(ram, b + 0x3e) != 0;
            put(ram, b + 0x1a, if normal { 0x9f6f } else { 0x9fc7 });
            put(ram, b + 0x1c, 1);
            put(ram, b + 0x18, 0);
            put(ram, b + 0x30, 0xa235);
            set(ram, b, 0x45, word(ram, b + 0x1e) & 0xe00);
            set(ram, b, 0x44, 3);
            for part in (0u16..=3).rev() {
                let current_k = current(ram)?.0;
                s.spawn_body(ram, part, current_k, 0xec95)?;
                // Preserve feedback copies; admitted native init does not alter
                // actor44, whose decrement controls the original four-spawn loop.
                if get(ram, b, 0x44) != part {
                    return Err("YappingMaw spawn altered loop controller outside admitted four-part domain".into());
                }
                set(ram, b, 0x44, part.wrapping_sub(1));
            }
            let tiles = word(ram, b + 0x20) | word(ram, b + 0x1e);
            set(ram, b, 0x47, tiles);
            let sprite = s.sprite(
                ram,
                word(ram, b + 2),
                word(ram, b + 6).wrapping_add(if normal { 8 } else { 0xfff8 }),
                if normal { 57 } else { 56 },
                tiles,
            )?;
            set(ram, b, 0x46, sprite);
            ensure_links(ram, b)?;
        }
        Entry::Main => {
            set(ram, b, 0x35, get(ram, b, 0x35).wrapping_sub(1));
            let offscreen = s.screen(ram)?;
            set(ram, b, 0x36, offscreen);
            let f = function_selector(0xa80000 | u32::from(word(ram, b + 0x30)))
                .ok_or("YappingMaw screen service changed dispatch outside owned scope")?;
            fn_run(f, k, a, m, ram, rom, s)?;
            for f in [15, 14, 13, 12] {
                fn_run(f, k, 0, 0, ram, rom, s)?;
            }
        }
        Entry::Dispatch(address) => {
            fn_run(function_selector(address).unwrap(), k, a, m, ram, rom, s)?;
        }
        Entry::Function(n) => return Ok(Some(fn_run(n, k, a, m, ram, rom, s)?)),
        Entry::Touch => {
            if signed(get(ram, b, 0x35)) && get(ram, b, 0x30) == 0 {
                set(ram, b, 0x35, 0);
                set(ram, b, 0x30, 1);
                put(ram, 0xa60, 0xe90e);
            }
        }
        Entry::Shot => {
            s.shot(ram)?;
            let (_, b) = current(ram)?;
            if word(ram, b + 0x14) != 0 {
                if word(ram, b + 0x26) != 0 {
                    if word(ram, 0xa60) != 0xe91d {
                        put(ram, 0xa60, 0xe913);
                    }
                    set(ram, b, 0x30, 0);
                }
            } else {
                ensure_links(ram, b)?;
                for n in 0x40..=0x43 {
                    put(ram, 0x1997 + usize::from(get(ram, b, n)), 0);
                }
                put(ram, 0xef78 + usize::from(get(ram, b, 0x46)), 0);
                if word(ram, 0xa60) != 0xe91d {
                    put(ram, 0xa60, 0xe913);
                }
                let (_, b) = current(ram)?;
                set(ram, b, 0x30, 0);
            }
        }
        Entry::Frozen => {
            s.frozen(ram)?;
            let (_, b) = current(ram)?;
            for n in 0x40..=0x43 {
                fn_run(18, get(ram, b, n), 0, 0, ram, rom, s)?;
            }
            fn_run(19, k, 0, 0, ram, rom, s)?;
        }
    }
    Ok(Some(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Native {
        events: Vec<(u8, u16)>,
    }
    impl Services for Native {
        fn available(&self, _: Entry) -> bool {
            true
        }
        fn spawn_body(&mut self, ram: &mut [u8], part: u16, _: u16, definition: u16) -> Result<(), String> {
            assert_eq!(definition, 0xec95);
            self.events.push((1, part));
            let (_, b) = current(ram)?;
            set(ram, b, 0x40 + usize::from(part), part * 2);
            Ok(())
        }
        fn sprite(&mut self, _: &mut [u8], _: u16, _: u16, kind: u16, palette: u16) -> Result<u16, String> {
            self.events.push((2, kind));
            assert_eq!(palette, 0x1200);
            Ok(6)
        }
        fn screen(&mut self, _: &mut [u8]) -> Result<u16, String> {
            Ok(0)
        }
        fn geometry(&mut self, _: &mut [u8], _: (u16, u16), _: (u16, u16)) -> Result<(u16, u16), String> {
            Ok((40, 0))
        }
        fn samus_code(&mut self, _: &mut [u8], code: u16) -> Result<(), String> {
            self.events.push((3, code));
            Ok(())
        }
        fn samus_adjust(&mut self, _: &mut [u8]) -> Result<(), String> {
            Ok(())
        }
        fn shot(&mut self, _: &mut [u8]) -> Result<(), String> {
            Ok(())
        }
        fn frozen(&mut self, _: &mut [u8]) -> Result<(), String> {
            Ok(())
        }
        fn sound2(&mut self, _: &mut [u8], sound: u16) -> Result<(), String> {
            self.events.push((4, sound));
            Ok(())
        }
        fn write_multiplier(&mut self, value: u8) -> Result<(), String> {
            self.events.push((5, u16::from(value)));
            Ok(())
        }
        fn multiply(&mut self, a: u8, b: u8) -> Result<u16, String> {
            self.events.push((6, u16::from(b)));
            Ok(u16::from(a) * u16::from(b))
        }
    }
    fn rom() -> Vec<u8> {
        vec![0; PAIRS + 32]
    }
    #[test]
    fn trig_preserves_zero_mmio_and_negative_high_product_formula() {
        let mut rom = rom();
        put(&mut rom, 0x1031c3 + 2 * 255, 0xff00);
        let mut s = Native::default();
        assert_eq!(trig(1, 0, false, &rom, &mut s).unwrap(), 0);
        assert_eq!(s.events, [(5, 1)]);
        s.events.clear();
        assert_eq!(trig(1, 2, false, &rom, &mut s).unwrap(), 0xfffe);
        assert_eq!(s.events, [(5, 1), (6, 2)]);
    }
    #[test]
    fn instruction_pair_uses_literal_a8_bank_and_retains_jp() {
        let mut rom = rom();
        put(&mut rom, 0x1420ab, 0x1234);
        put(&mut rom, 0x1420ad, 0xfff8);
        put(&mut rom, 0x13a0ab, 0x5678);
        let mut ram = vec![0; SIZE];
        let mut s = Native::default();
        assert_eq!(execute(Entry::Instruction(2), 1, 0xa0c9, 0, 0, &mut ram, &rom, &mut s).unwrap(), Some(0xa0c9));
        assert_eq!(get(&ram, ENEMY, 0x32), 0x1234);
        assert_eq!(get(&ram, ENEMY, 0x33), 0xfff8);
        assert!(s.events.is_empty());
    }
    #[test]
    fn init_orders_four_linked_parts_then_mouth_and_preserves_tiles() {
        let mut ram = vec![0; SIZE];
        put(&mut ram, ENEMY + 0x20, 0x1000);
        put(&mut ram, ENEMY + 0x22, 0x4000);
        put(&mut ram, ENEMY + 0x1e, 0x200);
        let mut s = Native::default();
        execute(Entry::Init, 0, 0, 0, 0, &mut ram, &rom(), &mut s).unwrap();
        assert_eq!(s.events, [(1, 3), (1, 2), (1, 1), (1, 0), (2, 56)]);
        assert_eq!(get(&ram, ENEMY, 0x44), 0xffff);
        assert_eq!(get(&ram, ENEMY, 0x46), 6);
        assert_eq!(get(&ram, ENEMY, 0x47), 0x1200);
    }
    #[test]
    fn palette_threshold_and_touch_capture_release_exception() {
        assert_eq!(palette(0x3123, 0x200, 90), 0x3d23);
        assert_eq!(palette(0x3123, 0x200, 88), 0x3323);
        assert_eq!(palette(0x3123, 0x200, 89), 0x3323);
        assert_eq!(palette(0x3123, 0x200, 86), 0x3d23);
        let mut ram = vec![0; SIZE];
        set(&mut ram, ENEMY, 0x35, 0xffff);
        let mut s = Native::default();
        execute(Entry::Touch, 0, 0, 0, 0, &mut ram, &[], &mut s).unwrap();
        assert_eq!(word(&ram, 0xa60), 0xe90e);
        assert_eq!(get(&ram, ENEMY, 0x30), 1);
        put(&mut ram, 0xa60, 0xe91d);
        put(&mut ram, ENEMY + 0x38, 0);
        execute(Entry::Function(11), 0, 0, 0, 0, &mut ram, &[], &mut s).unwrap();
        assert_eq!(word(&ram, 0xa60), 0xe91d);
        assert_eq!(get(&ram, ENEMY, 0x30), 1);
    }
    #[test]
    fn frozen_timer_uses_literal_base26_independent_of_vram20() {
        let mut ram = vec![0; SIZE];
        let mut s = Native::default();
        put(&mut ram, ENEMY + 0x14, 1);
        put(&mut ram, ENEMY + 0x20, 0);
        put(&mut ram, ENEMY + 0x26, 90);
        set(&mut ram, ENEMY, 0x30, 1);
        put(&mut ram, 0xa60, 0xe90e);
        execute(Entry::Shot, 0, 0, 0, 0, &mut ram, &[], &mut s).unwrap();
        assert_eq!(word(&ram, 0xa60), 0xe913);
        assert_eq!(get(&ram, ENEMY, 0x30), 0);
        set(&mut ram, ENEMY, 0x45, 0x200);
        put(&mut ram, 0x19bb, 0x3123);
        put(&mut ram, 0xf078, 0x3123);
        execute(Entry::Function(18), 0, 0, 0, 0, &mut ram, &[], &mut s).unwrap();
        execute(Entry::Function(19), 0, 0, 0, 0, &mut ram, &[], &mut s).unwrap();
        assert_eq!(word(&ram, 0x19bb), 0x3d23);
        assert_eq!(word(&ram, 0xf078), 0x3d23);
        put(&mut ram, ENEMY + 0x20, 0x1000);
        put(&mut ram, ENEMY + 0x26, 0);
        set(&mut ram, ENEMY, 0x30, 1);
        put(&mut ram, 0xa60, 0xe90e);
        execute(Entry::Shot, 0, 0, 0, 0, &mut ram, &[], &mut s).unwrap();
        assert_eq!(word(&ram, 0xa60), 0xe90e);
        assert_eq!(get(&ram, ENEMY, 0x30), 1);
        execute(Entry::Function(18), 0, 0, 0, 0, &mut ram, &[], &mut s).unwrap();
        execute(Entry::Function(19), 0, 0, 0, 0, &mut ram, &[], &mut s).unwrap();
        assert_eq!(word(&ram, 0x19bb), 0x3323);
        assert_eq!(word(&ram, 0xf078), 0x3323);
    }
}
