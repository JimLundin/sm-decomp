//! Complete Beetom actor. Common enemy movement, random and damage remain services.
#![forbid(unsafe_code)]

const WRAM: usize = 0x20000;
const CURRENT: usize = 0xe54;
const QUADRATIC16: usize = 0x11038f;
const QUADRATIC32: usize = 0x104bc7;
const RANDOM_STATES: usize = 0x14374e;
const MAX_ROW: u16 = 128;
const ADDRESSES: [u16; 37] = [
    0xb762, 0xb7ef, 0xb814, 0xb82f, 0xb84f, 0xb85f, 0xb873, 0xb887, 0xb8a9, 0xb8cb, 0xb8ed, 0xb90f, 0xb952, 0xb966,
    0xb97a, 0xb9a2, 0xb9b2, 0xb9c1, 0xba24, 0xba84, 0xbab7, 0xbae7, 0xbb20, 0xbb55, 0xbb88, 0xbbb8, 0xbbf1, 0xbc26,
    0xbc5a, 0xbc8a, 0xbcc3, 0xbcf8, 0xbd42, 0xbd8c, 0xbd9d, 0xbdc5, 0xbdcc,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Instruction { jp: u16 },
    Init,
    Main,
    Touch,
    Shot,
    Function { number: u8, slot: u16 },
    Search { target: u16, step: u16 },
    Dispatch { address: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Done,
    Word(u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    MoveDown { slot: u16, amount: i32 },
    MoveHorizontal { slot: u16, amount: i32 },
    NextRandom,
    PlayerDeltaX { slot: u16 },
    PlayerWithinX { slot: u16, distance: u16 },
    QueueSfx3Max6 { sound: u16 },
    NormalTouchA8,
    NormalShotA8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response {
    Done,
    Collision(bool),
    Word(u16),
}

pub trait Services {
    /// Immutable, no-call admission of this complete entry and every reachable
    /// request/copyback/table read. Bind ROM, slot, allocation, generation and
    /// sibling/reentry fences. Prove MAX_ROW for all later speed-table states,
    /// preserve the selected slot, and validate all terrain/damage/SFX domains.
    /// This policy must account for local +/-8 X probes and subsequent services.
    fn admitted(&self, entry: Entry, ram: &[u8], rom: &[u8]) -> bool;
    /// One synchronous common-service request with full WRAM copyback. Error or
    /// wrong response after acceptance is terminal: keep effects, never retry C.
    fn request(&mut self, ram: &mut [u8], request: Request) -> Result<Response, String>;
}

/// None refuses before canonical writes or service calls. The owning adapter
/// must terminate an accepted invocation on Err, including panics/reentry.
pub fn execute(
    entry: Entry,
    ram: &mut [u8],
    rom: &[u8],
    services: &mut impl Services,
) -> Result<Option<Output>, String> {
    if ram.len() != WRAM {
        return Ok(None);
    }
    if let Entry::Instruction { jp } = entry {
        return Ok(Some(Output::Word(jp)));
    }
    if let Entry::Search { target, step } = entry {
        return Ok(search(rom, target, step).map(Output::Word));
    }
    let slot = match entry {
        Entry::Function { slot, .. } => slot,
        _ => word(ram, CURRENT),
    };
    if slot > 0x7c0 || slot & 0x3f != 0 {
        return Ok(None);
    }
    if let Entry::Function { number, .. } = entry {
        // Only these seven recovered helpers take k. Every other helper reads
        // cur_enemy_index; do not silently reinterpret it as an explicit slot.
        if !matches!(number, 22 | 23 | 26 | 27 | 30 | 31 | 37) && slot != word(ram, CURRENT) {
            return Ok(None);
        }
    }
    let function = match entry {
        Entry::Function { number, .. } if (1..=37).contains(&number) && number != 2 => Some(number),
        Entry::Function { .. } => return Ok(None),
        Entry::Dispatch { address } => match dispatch_number(address) {
            Some(n) => Some(n),
            None => return Ok(None),
        },
        Entry::Main => match dispatch_number(0xa80000 | u32::from(word(ram, base(slot) + 0x34))) {
            Some(n) => Some(n),
            None => return Ok(None),
        },
        _ => None,
    };
    if rom.get(QUADRATIC16..QUADRATIC16 + 8 * usize::from(MAX_ROW) + 3).is_none()
        || rom.get(QUADRATIC32..QUADRATIC32 + 8 * usize::from(MAX_ROW) + 8).is_none()
        || rom.get(RANDOM_STATES..RANDOM_STATES + 16).is_none()
    {
        return Ok(None);
    }
    let init_rows = if entry == Entry::Init {
        let Some(a) = search(rom, 12288, 4) else { return Ok(None) };
        let Some(b) = search(rom, 16384, 5) else { return Ok(None) };
        let Some(c) = search(rom, 12288, 3) else { return Ok(None) };
        Some([a, b, c])
    } else {
        None
    };
    if entry != Entry::Init && word(ram, base(slot) + 0x32) > MAX_ROW {
        return Ok(None);
    }
    // A random service must not publish an unreviewed dynamic selector.
    for i in 0..8 {
        if dispatch_number(0xa80000 | u32::from(word(rom, RANDOM_STATES + 2 * i))).is_none() {
            return Ok(None);
        }
    }
    if !services.admitted(entry, ram, rom) {
        return Ok(None);
    }
    let mut actor = Actor { ram, rom, services, slot };
    if let Some(n) = function {
        actor.function(n)?;
    } else {
        match entry {
            Entry::Init => actor.init(init_rows.expect("preflight initialized rows"))?,
            Entry::Touch => actor.touch()?,
            Entry::Shot => actor.shot()?,
            _ => unreachable!("all other entries returned or selected a function"),
        }
    }
    Ok(Some(Output::Done))
}

fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn put(bytes: &mut [u8], at: usize, v: u16) {
    bytes[at..at + 2].copy_from_slice(&v.to_le_bytes());
}
fn base(slot: u16) -> usize {
    0xf78 + usize::from(slot)
}
fn search(rom: &[u8], target: u16, step: u16) -> Option<u16> {
    if step == 0 {
        return None;
    }
    let mut row = 0u16;
    let mut sum = 0u16;
    loop {
        row = row.wrapping_add(step);
        // Conservative bounded subset: no wrap or unbounded recovered do-loop.
        if row == 0 || row > MAX_ROW {
            return None;
        }
        let at = QUADRATIC16 + 8 * usize::from(row) + 1;
        let data = rom.get(at..at + 2)?;
        sum = sum.wrapping_add(u16::from_le_bytes([data[0], data[1]]));
        if (sum.wrapping_sub(target) as i16) >= 0 {
            return Some(row);
        }
    }
}
fn dispatch_number(address: u32) -> Option<u8> {
    if address & 0xff0000 != 0xa80000 || address > 0xffffff {
        return None;
    }
    let n = ADDRESSES.iter().position(|&a| u32::from(a) == address & 0xffff)? as u8 + 1;
    if matches!(n, 3..=21 | 24 | 25 | 28 | 29 | 32 | 33 | 35 | 36) { Some(n) } else { None }
}
struct Actor<'a, S> {
    ram: &'a mut [u8],
    rom: &'a [u8],
    services: &'a mut S,
    slot: u16,
}
impl<S: Services> Actor<'_, S> {
    fn get(&self, offset: usize) -> u16 {
        word(self.ram, base(self.slot) + offset)
    }
    fn set(&mut self, offset: usize, v: u16) {
        put(self.ram, base(self.slot) + offset, v);
    }
    fn ext(&self, n: usize) -> u16 {
        word(self.ram, 0x7800 + usize::from(self.slot) + 2 * n)
    }
    fn set_ext(&mut self, n: usize, v: u16) {
        put(self.ram, 0x7800 + usize::from(self.slot) + 2 * n, v);
    }
    fn state(&mut self, n: u8) {
        self.set(0x34, ADDRESSES[usize::from(n - 1)]);
    }
    fn instruction(&mut self, address: u16) {
        self.set_ext(0, address);
        self.function1();
    }
    fn function1(&mut self) {
        self.set(0x1a, self.ext(0));
        self.set(0x1c, 1);
        self.set(0x18, 0);
    }
    fn done(&mut self, request: Request) -> Result<(), String> {
        match self.services.request(self.ram, request)? {
            Response::Done => Ok(()),
            _ => Err("Beetom service returned wrong void result".into()),
        }
    }
    fn value(&mut self, request: Request) -> Result<u16, String> {
        match self.services.request(self.ram, request)? {
            Response::Word(v) => Ok(v),
            _ => Err("Beetom service returned wrong word result".into()),
        }
    }
    fn movement(&mut self, amount: i32, horizontal: bool) -> Result<bool, String> {
        let request = if horizontal {
            Request::MoveHorizontal { slot: self.slot, amount }
        } else {
            Request::MoveDown { slot: self.slot, amount }
        };
        match self.services.request(self.ram, request)? {
            Response::Collision(v) => Ok(v),
            _ => Err("Beetom movement returned wrong collision result".into()),
        }
    }
    fn init(&mut self, rows: [u16; 3]) -> Result<(), String> {
        for at in [0x36, 0x34, 0x30] {
            self.set(at, 0);
        }
        self.set_ext(5, 0);
        self.set_ext(8, 0);
        self.set(0x38, 64);
        self.set(0x3a, word(self.ram, 0x8b));
        put(self.ram, 0x5e5, 23);
        for (i, row) in rows.into_iter().enumerate() {
            self.set_ext(i + 2, row);
        }
        self.set_ext(0, 0xb6f2);
        let delta = self.value(Request::PlayerDeltaX { slot: self.slot })?;
        if delta & 0x8000 != 0 {
            self.set_ext(0, 0xb696);
        }
        self.function1();
        self.state(3);
        Ok(())
    }
    fn jump_vertical(&mut self, ascending: bool, step: u16) -> Result<(), String> {
        let row = self.get(0x32);
        if row > MAX_ROW {
            return Err("Beetom speed row changed outside admitted domain".into());
        }
        let at = QUADRATIC32 + 8 * usize::from(row) + if ascending { 4 } else { 0 };
        let amount = i32::from_le_bytes(self.rom[at..at + 4].try_into().expect("preflight speed bytes"));
        if self.movement(amount, false)? {
            self.state(if ascending { 15 } else { 3 });
        } else if ascending {
            let next = self.get(0x32).wrapping_sub(step);
            self.set(0x32, next);
            if (next as i16) < 0 {
                self.set(0x32, 0);
                self.set_ext(5, 1);
            }
        } else {
            let next = self.get(0x32).wrapping_add(step);
            self.set(0x32, next);
            if (next.wrapping_sub(64) as i16) >= 0 {
                self.set(0x32, 64);
            }
        }
        Ok(())
    }
    fn jump(&mut self, number: u8) -> Result<(), String> {
        let up = self.ext(5) == 0;
        let step = match number {
            20 | 21 => 4,
            24 | 25 => 5,
            28 | 29 => {
                if up {
                    5
                } else {
                    3
                }
            }
            _ => unreachable!(),
        };
        self.jump_vertical(up, step)?;
        let amount = match number {
            20 | 24 => -16384,
            21 | 25 => 16384,
            28 => -196608,
            29 => 196608,
            _ => unreachable!(),
        };
        if self.movement(amount, true)? {
            self.set_ext(9, self.ext(9) ^ 1);
            self.state(15);
        }
        Ok(())
    }
    fn crawl(&mut self, left: bool) -> Result<(), String> {
        let timer = self.get(0x36).wrapping_sub(1);
        self.set(0x36, timer);
        if timer & 0x8000 != 0 {
            self.set(0x36, 64);
            self.state(3);
            return Ok(());
        }
        self.set(2, self.get(2).wrapping_add(if left { 0xfff8 } else { 8 }));
        if self.movement(65536, false)? {
            self.set(2, self.get(2).wrapping_add(if left { 8 } else { 0xfff8 }));
            if self.movement(if left { -16384 } else { 16384 }, true)? {
                self.state(if left { 7 } else { 6 });
            }
        } else {
            self.state(if left { 7 } else { 6 });
            self.set(6, self.get(6).wrapping_sub(1));
            self.set(2, self.get(2).wrapping_add(if left { 8 } else { 0xfff8 }));
        }
        Ok(())
    }
    fn attached(&mut self, left: bool) -> Result<(), String> {
        if self.get(0x38) != 0 {
            self.set(2, word(self.ram, 0xaf6));
            self.set(6, word(self.ram, 0xafa).wrapping_sub(4));
            self.function(34)?;
        } else {
            if self.movement(if left { 1048576 } else { -1048576 }, true)? {
                self.set_ext(9, u16::from(left));
                self.movement(if left { -2097152 } else { 2097152 }, true)?;
            }
            self.set_ext(8, 0);
            self.state(16);
        }
        Ok(())
    }
    fn function(&mut self, n: u8) -> Result<(), String> {
        match n {
            1 => self.function1(),
            3 => {
                let within = self.value(Request::PlayerWithinX { slot: self.slot, distance: 0x60 })?;
                self.state(if within != 0 { 12 } else { 4 });
            }
            4 => {
                self.done(Request::NextRandom)?;
                let random = word(self.ram, 0x5e5);
                self.set(0x34, word(self.rom, RANDOM_STATES + 2 * usize::from(random & 7)));
                self.set_ext(9, random & 1);
            }
            5 => {
                self.set(0x36, 32);
                self.state(17);
            }
            6 | 7 => {
                self.state(if n == 6 { 18 } else { 19 });
                self.instruction(if n == 6 { 0xb696 } else { 0xb6f2 });
            }
            8..=11 => {
                self.set(0x32, self.ext(if n <= 9 { 2 } else { 3 }));
                self.state(match n {
                    8 => 20,
                    9 => 21,
                    10 => 24,
                    _ => 25,
                });
                self.set_ext(5, 0);
                self.instruction(if n % 2 == 0 { 0xb6ac } else { 0xb708 });
            }
            12 => {
                self.set(0x32, self.ext(4));
                self.set_ext(0, 0xb708);
                self.state(29);
                self.set_ext(9, 1);
                if self.value(Request::PlayerDeltaX { slot: self.slot })? & 0x8000 != 0 {
                    self.set_ext(0, 0xb6ac);
                    self.state(28);
                    self.set_ext(9, 0);
                }
                self.function1();
                self.set_ext(5, 0);
            }
            13 | 14 => {
                self.instruction(if n == 13 { 0xb6cc } else { 0xb728 });
                self.state(if n == 13 { 32 } else { 33 });
            }
            15 => {
                self.instruction(if self.ext(9) != 0 { 0xb6f2 } else { 0xb696 });
                self.set_ext(5, 0);
                self.state(35);
            }
            16 => {
                self.set(0x32, 0);
                self.state(36);
            }
            17 => {
                let timer = self.get(0x36).wrapping_sub(1);
                self.set(0x36, timer);
                if timer & 0x8000 != 0 {
                    self.state(3);
                }
            }
            18 | 19 => self.crawl(n == 18)?,
            20 | 21 | 24 | 25 | 28 | 29 => self.jump(n)?,
            22 | 26 | 30 => self.jump_vertical(true, if n == 22 { 4 } else { 5 })?,
            23 | 27 | 31 => self.jump_vertical(
                false,
                match n {
                    23 => 4,
                    27 => 5,
                    _ => 3,
                },
            )?,
            32 | 33 => self.attached(n == 32)?,
            34 => {
                let keys = word(self.ram, 0x8b);
                if keys != self.get(0x3a) {
                    self.set(0x3a, keys);
                    self.set(0x38, self.get(0x38).wrapping_sub(1));
                }
            }
            35 => {
                if self.movement(196608, false)? {
                    self.state(if self.ext(9) != 0 { 7 } else { 6 });
                }
            }
            36 | 37 => {
                self.jump_vertical(false, 1)?;
                if self.movement(if self.ext(9) != 0 { -131072 } else { 131072 }, true)? {
                    self.set_ext(9, self.ext(9) ^ 1);
                    self.state(15);
                }
            }
            _ => return Err("unreachable Beetom function selector after admission".into()),
        }
        Ok(())
    }
    fn touch(&mut self) -> Result<(), String> {
        if self.ext(8) == 0 {
            self.state(if self.ext(9) != 0 { 14 } else { 13 });
            self.set_ext(8, 1);
            self.set(0x38, 64);
            self.set(0x22, 2);
            self.set_ext(6, word(self.ram, 0xaf6).wrapping_sub(self.get(2)));
            self.set_ext(7, word(self.ram, 0xafa).wrapping_sub(self.get(6)));
        }
        let contact = word(self.ram, 0xa6e) != 0;
        if !contact && word(self.ram, 0xe44) & 7 == 7 && (word(self.ram, 0x9c2).wrapping_sub(30) as i16) >= 0 {
            self.done(Request::QueueSfx3Max6 { sound: 0x2d })?;
        }
        if contact || self.get(0x2c) & 0x3f == 63 {
            self.done(Request::NormalTouchA8)?;
            put(self.ram, 0x18a8, 0);
            put(self.ram, 0x18aa, 0);
        }
        Ok(())
    }
    fn shot(&mut self) -> Result<(), String> {
        self.done(Request::NormalShotA8)?;
        if self.get(0x26) != 0 && matches!(self.get(0x34), 0xbd42 | 0xbcf8) {
            self.state(15);
        }
        self.set_ext(8, 0);
        Ok(())
    }
}
