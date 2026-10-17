//! MiniDraygon and linked Evir projectile recovered-C Translation scope.
//! Candidate only: translated source does not establish actual-C agreement.
const WRAM: usize = 0x20000;
const CURRENT: usize = 0xe54;
const BASE: usize = 0xf78;
const EXTENDED: usize = 0x7800;
const DRAW: usize = 0xe84;
const DRAW_SIZES: usize = 0xf68;
const SPEEDS: usize = 0x110187;

fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

/// `argument` is the unchanged instruction pointer for instruction entries,
/// or the full banked address for Dispatch; implicit-current entries ignore it.
/// Ok(None) refuses without writes/services. Err is terminal after ownership.
/// Runtime ROM must be authenticated against the reference by the adapter.
pub fn execute(
    entry: Entry,
    argument: u32,
    ram: &mut [u8],
    rom: &[u8],
    services: &mut impl Services,
) -> Result<Option<u32>, String> {
    if ram.len() != WRAM {
        return Ok(None);
    }
    let k = word(ram, CURRENT);
    if !slot(k) || k > 0x740 {
        return Ok(None);
    }
    if matches!(entry, Entry::Func7 | Entry::ProjectileMain) && k < 128 {
        return Ok(None);
    }
    let valid_dispatch = |ea| matches!(ea, 0xa88922 | 0xa88a34 | 0xa88a3b | 0xa88a78);
    if entry == Entry::Dispatch && !valid_dispatch(argument) {
        return Ok(None);
    }
    if matches!(entry, Entry::Main | Entry::ProjectileMain)
        && !valid_dispatch(u32::from(word(ram, BASE + usize::from(k) + 0x34)) | 0xa80000)
    {
        return Ok(None);
    }
    if entry == Entry::Init && word(ram, BASE + usize::from(k) + 0x3c) == 0 && rom.len() < SPEEDS + 2048 {
        return Ok(None);
    }
    if !services.available(entry) {
        return Ok(None);
    }
    let mut a = Actor { ram, rom, services };
    match entry {
        Entry::Instruction1 => {
            let b = a.b();
            let death = word(a.ram, 0x7000 + usize::from(a.k()) + 30 * 64 + 2);
            put(a.ram, b + 0x3a, if death == 0x870b { 0xfff8 } else { 8 });
        }
        Entry::Instruction2 => a.services.sound2_max6(a.ram, 0x5e)?,
        Entry::Instruction3 => {
            let b = a.b();
            let death = word(a.ram, 0x7000 + usize::from(a.k()) + 30 * 64 + 2);
            put(a.ram, b + 0x3a, word(a.ram, b + 0x3a).wrapping_add(if death == 0x870b { 1 } else { 0xffff }));
        }
        Entry::Instruction4 => {
            let b = a.b();
            let x = a.x();
            put(a.ram, x + 24, 0);
            put(a.ram, x + 22, 0);
            put(a.ram, b + 0x34, 0x8a34);
        }
        Entry::Init => {
            // Init captures E before Func1's synchronous delta service.
            let b = a.b();
            let x = a.x();
            if word(a.ram, b + 0x3c) != 0 {
                a.func2();
                put(a.ram, b + 0x22, 4);
            } else {
                a.func1()?;
                let selector = usize::from(word(a.ram, b + 0x3e) & 0xff) * 8;
                if a.rom.len() < SPEEDS + selector + 8 {
                    return Err("MiniDraygon Init runtime ROM speed row unavailable after service".into());
                }
                put(a.ram, x + 8, word(a.rom, SPEEDS + selector));
                put(a.ram, x + 6, word(a.rom, SPEEDS + selector + 2));
                put(a.ram, x + 12, word(a.rom, SPEEDS + selector + 4));
                put(a.ram, x + 10, word(a.rom, SPEEDS + selector + 6));
                // Recovered C explicitly reads Get_MiniDraygon(8*LOBYTE(parameter2)),
                // not the current E. Byte offsets need not be 64-byte aligned.
                put(a.ram, b + 0x38, (word(a.ram, BASE + selector + 0x3e) >> 8) >> 1);
            }
            put(a.ram, x, 0);
            put(a.ram, x + 2, 0);
            put(a.ram, b + 0x34, 0x8922);
        }
        Entry::Func1 => a.func1()?,
        Entry::Func2 | Entry::Func6 => a.func2(),
        Entry::ProjectileInit => {
            let b = a.b();
            let x = a.x();
            put(a.ram, x + 4, 0x876f);
            a.func12();
            let palette = a.draw(73);
            put(a.ram, b + 0x1e, palette);
            let tiles = a.draw(74);
            put(a.ram, b + 0x20, tiles);
            a.func3();
            put(a.ram, x + 2, 0);
            put(a.ram, x + 24, 0);
            put(a.ram, x + 22, 0);
            put(a.ram, b + 0x3a, 0);
            put(a.ram, b + 0x34, 0x8a34);
        }
        Entry::Func3 | Entry::Func8 => a.func3(),
        Entry::Dispatch => a.dispatch(argument)?,
        Entry::Main => {
            let state = u32::from(word(a.ram, a.b() + 0x34)) | 0xa80000;
            a.dispatch(state)?;
        }
        Entry::Func4 => a.func4()?,
        Entry::Func5 => a.func5()?,
        Entry::ProjectileMain => {
            // Captured E is retained across Func7 and its shared services.
            let b = a.b();
            let x = a.x();
            if word(a.ram, b + 0x26) == 0 {
                if word(a.ram, x + 22) != 0 {
                    put(a.ram, x + 4, 0x876f);
                    a.func12();
                } else if word(a.ram, x + 24) != 0 {
                    put(a.ram, x + 4, 0x8775);
                    a.func12();
                } else {
                    a.func7()?;
                }
            }
            let state = u32::from(word(a.ram, b + 0x34)) | 0xa80000;
            a.dispatch(state)?;
        }
        Entry::Func7 => a.func7()?,
        Entry::Func9 => a.func9()?,
        Entry::Func10 => a.func10(),
        Entry::Func11 => a.func11()?,
        Entry::Func12 => a.func12(),
        Entry::Touch => {
            a.services.touch(a.ram)?;
            a.valid_current()?;
            a.func13();
        }
        Entry::Powerbomb => {
            a.services.powerbomb(a.ram)?;
            a.valid_current()?;
            a.func13();
        }
        Entry::Shot => {
            a.services.shot(a.ram)?;
            a.valid_current()?;
            a.func13();
        }
        Entry::Func13 => a.func13(),
    }
    Ok(Some(
        if matches!(entry, Entry::Instruction1 | Entry::Instruction2 | Entry::Instruction3 | Entry::Instruction4) {
            argument
        } else {
            0
        },
    ))
}
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn slot(k: u16) -> bool {
    k < 0x800 && k & 63 == 0
}
fn add_hi_lo(ram: &mut [u8], high: usize, low: usize, delta: u32) {
    let value = (u32::from(word(ram, high)) << 16) | u32::from(word(ram, low));
    let next = value.wrapping_add(delta);
    put(ram, high, (next >> 16) as u16);
    put(ram, low, next as u16);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Instruction1,
    Instruction2,
    Instruction3,
    Instruction4,
    Init,
    Func1,
    Func2,
    ProjectileInit,
    Func3,
    Dispatch,
    Main,
    Func4,
    Func5,
    Func6,
    ProjectileMain,
    Func7,
    Func8,
    Func9,
    Func10,
    Func11,
    Func12,
    Touch,
    Powerbomb,
    Shot,
    Func13,
}

/// Synchronous shared enemy services, with full canonical WRAM copyback.
/// `available` must preflight every service the admitted entry can reach.
/// Errors after admission are terminal; callers must never retry reference C.
/// Native adapters must preserve the full unsigned 32-bit trigonometric result:
/// recovered sine negates high and low halves independently.
pub trait Services {
    fn available(&self, entry: Entry) -> bool;
    fn delta_x(&mut self, ram: &mut [u8], k: u16) -> Result<u16, String>;
    fn within_x(&mut self, ram: &mut [u8], k: u16, distance: u16) -> Result<u16, String>;
    fn angle(&mut self, ram: &mut [u8], k: u16) -> Result<u16, String>;
    fn cosine_full(&mut self, ram: &mut [u8], angle: u16, amplitude: u16) -> Result<u32, String>;
    fn sine_full(&mut self, ram: &mut [u8], angle: u16, amplitude: u16) -> Result<u32, String>;
    fn offscreen(&mut self, ram: &mut [u8], distance: u16) -> Result<u16, String>;
    fn sound2_max6(&mut self, ram: &mut [u8], sound: u16) -> Result<(), String>;
    fn touch(&mut self, ram: &mut [u8]) -> Result<(), String>;
    fn powerbomb(&mut self, ram: &mut [u8]) -> Result<(), String>;
    fn shot(&mut self, ram: &mut [u8]) -> Result<(), String>;
}

struct Actor<'a, S> {
    ram: &'a mut [u8],
    rom: &'a [u8],
    services: &'a mut S,
}
impl<S: Services> Actor<'_, S> {
    fn k(&self) -> u16 {
        word(self.ram, CURRENT)
    }
    fn b(&self) -> usize {
        BASE + usize::from(self.k())
    }
    fn x(&self) -> usize {
        EXTENDED + usize::from(self.k())
    }
    fn draw(&self, n: usize) -> u16 {
        word(self.ram, DRAW + usize::from(self.k()) + n * 2)
    }
    fn valid_current(&self) -> Result<(), String> {
        if slot(self.k()) && self.k() <= 0x740 {
            Ok(())
        } else {
            Err("MiniDraygon service changed current slot outside admitted linked-slot domain".into())
        }
    }
    fn func1(&mut self) -> Result<(), String> {
        let k = self.k();
        let direction = self.services.delta_x(self.ram, k)?;
        self.valid_current()?;
        let b = self.b();
        let x = self.x();
        put(self.ram, b + 0x32, u16::from(direction & 0x8000 == 0));
        put(self.ram, x + 4, if word(self.ram, b + 0x32) != 0 { 0x870b } else { 0x86a7 });
        self.func12();
        Ok(())
    }
    fn func2(&mut self) {
        let direction = word(self.ram, DRAW_SIZES + usize::from(self.k()) + 2);
        let b = self.b();
        let x = self.x();
        put(self.ram, b + 0x32, direction);
        let px = self.draw(91).wrapping_add(if direction != 0 { 4 } else { 0xfffc });
        put(self.ram, b + 2, px);
        let py = self.draw(93).wrapping_add(10);
        put(self.ram, b + 6, py);
        put(self.ram, x + 4, if direction != 0 { 0x8727 } else { 0x86c3 });
        self.func12();
    }
    fn func3(&mut self) {
        let direction = self.draw(83);
        let b = self.b();
        put(self.ram, b + 0x32, direction);
        let px = self.draw(59).wrapping_add(if direction != 0 { 4 } else { 0xfffc });
        put(self.ram, b + 2, px);
        let py = self.draw(61).wrapping_add(18);
        put(self.ram, b + 6, py);
    }
    fn func4(&mut self) -> Result<(), String> {
        if word(self.ram, self.b() + 0x3c) != 0 {
            self.func2();
            Ok(())
        } else {
            self.func5()
        }
    }
    fn func5(&mut self) -> Result<(), String> {
        if word(self.ram, self.x() + 128 + 0x18) == 0 {
            self.func1()?;
        }
        let b = self.b();
        let x = self.x();
        let (hi, lo) = if word(self.ram, x) != 0 { (8, 6) } else { (12, 10) };
        let velocity = (u32::from(word(self.ram, x + hi)) << 16) | u32::from(word(self.ram, x + lo));
        add_hi_lo(self.ram, b + 6, b + 8, velocity);
        let timer = word(self.ram, b + 0x38).wrapping_sub(1);
        put(self.ram, b + 0x38, timer);
        if timer & 0x8000 != 0 {
            put(self.ram, b + 0x38, word(self.ram, b + 0x3e) >> 8);
            put(self.ram, x, word(self.ram, x) ^ 1);
        }
        Ok(())
    }
    fn func7(&mut self) -> Result<(), String> {
        let k = self.k();
        if k < 128 {
            return Err("MiniDraygon projectile lacks preceding root slots".into());
        }
        let near = self.services.within_x(self.ram, k - 128, 0x80)?;
        self.valid_current()?;
        if near != 0 {
            let k = self.k();
            let angle = self.services.angle(self.ram, k)?;
            self.valid_current()?;
            let angle = 0u16.wrapping_sub(u16::from(angle.wrapping_sub(64) as u8));
            // C captures E after angle, retaining that pointer across both trig services.
            let x = self.x();
            let b = self.b();
            let cosine = self.services.cosine_full(self.ram, angle, 4)?;
            self.valid_current()?;
            put(self.ram, x + 14, (cosine >> 16) as u16);
            put(self.ram, x + 16, cosine as u16);
            let sine = self.services.sine_full(self.ram, angle, 4)?;
            self.valid_current()?;
            put(self.ram, x + 18, (sine >> 16) as u16);
            put(self.ram, x + 20, sine as u16);
            put(self.ram, x + 4, 0x876f);
            self.func12();
            put(self.ram, x + 22, 1);
            put(self.ram, b + 0x34, 0x8a3b);
        }
        Ok(())
    }
    fn func9(&mut self) -> Result<(), String> {
        self.func11()?;
        let b = self.b();
        let x = self.x();
        let dx = (u32::from(word(self.ram, x + 14)) << 16) | u32::from(word(self.ram, x + 16));
        let dy = (u32::from(word(self.ram, x + 18)) << 16) | u32::from(word(self.ram, x + 20));
        add_hi_lo(self.ram, b + 2, b + 4, dx);
        add_hi_lo(self.ram, b + 6, b + 8, dy);
        Ok(())
    }
    fn func10(&mut self) {
        if self.draw(77) == 0 {
            let b = self.b();
            let x = self.x();
            if word(self.ram, x + 24) != 0 {
                self.func3();
                put(self.ram, b + 2, word(self.ram, b + 2).wrapping_add(word(self.ram, b + 0x3a)));
            } else {
                put(self.ram, x + 4, 0x876f);
                self.func12();
                put(self.ram, x + 24, 0);
                put(self.ram, x + 22, 0);
                put(self.ram, b + 0x34, 0x8a34);
            }
        }
    }
    fn func11(&mut self) -> Result<(), String> {
        let off = self.services.offscreen(self.ram, 0x100)?;
        self.valid_current()?;
        if off != 0 && self.draw(77) == 0 {
            let b = self.b();
            let x = self.x();
            put(self.ram, x + 22, 0);
            put(self.ram, x + 24, 1);
            put(self.ram, b + 0x34, 0x8a78);
            // Retain the repeated store in the recovered source.
            put(self.ram, x + 24, 1);
            put(self.ram, x + 4, 0x8775);
            self.func12();
        }
        Ok(())
    }
    fn func12(&mut self) {
        let b = self.b();
        let x = self.x();
        let next = word(self.ram, x + 4);
        if next != word(self.ram, x + 2) {
            put(self.ram, b + 0x1a, next);
            put(self.ram, x + 2, next);
            put(self.ram, b + 0x1c, 1);
            put(self.ram, b + 0x18, 0);
        }
    }
    fn func13(&mut self) {
        let b = self.b();
        let child = b + 64;
        let projectile = b + 128;
        if word(self.ram, b + 0x14) == 0 {
            put(self.ram, child + 0xe, word(self.ram, child + 0xe) | 0x200);
            put(self.ram, projectile + 0xe, word(self.ram, projectile + 0xe) | 0x200);
        }
        let frozen = word(self.ram, b + 0x26);
        if frozen != 0 {
            put(self.ram, child + 0x26, frozen);
            put(self.ram, child + 0x12, word(self.ram, child + 0x12) | 4);
            if word(self.ram, projectile + 0x34) != 0x8a3b {
                put(self.ram, projectile + 0x12, word(self.ram, projectile + 0x12) | 4);
                put(self.ram, projectile + 0x26, word(self.ram, b + 0x26));
            }
        }
    }
    fn dispatch(&mut self, ea: u32) -> Result<(), String> {
        match ea {
            0xa88922 => self.func4(),
            0xa88a34 => {
                self.func3();
                Ok(())
            }
            0xa88a3b => self.func9(),
            0xa88a78 => {
                self.func10();
                Ok(())
            }
            _ => Err(format!("unsupported MiniDraygon dispatch address {ea:#08x} after admission")),
        }
    }
}
