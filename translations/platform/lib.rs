//! Complete Rising/Falling Platform actor; common checks and shutter dispatch stay native.
#![forbid(unsafe_code)]

const WRAM: usize = 0x20000;
const CURRENT: usize = 0xe54;
const SPEEDS: usize = 0x110187;
const FUNC2: u16 = 0xef09;
const FUNC4: u16 = 0xef28;
const UP: u16 = 0xef68;
const DOWN: u16 = 0xefd4;
const WAIT_UP: u16 = 0xf040;
const WAIT_DOWN: u16 = 0xf072;
const STOP: u16 = 0xf099;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Init,
    Main,
    Touch,
    Shot,
    Powerbomb,
    Function { number: u8, slot: u16 },
    Dispatch { address: u32 },
}
/// Exact neighboring cases in the original shared dispatcher. These definitions
/// are external dependencies, not claimed as translated platform routines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum HorizontalShutter {
    Function2 = 0xa2f224,
    Function4 = 0xa2f243,
    Function8 = 0xa2f272,
    Function9 = 0xa2f2e4,
    Function12 = 0xa2f38c,
    Function13 = 0xa2f3b0,
    Function14 = 0xa2f3d4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    PlayerWithinX { slot: u16, distance: u16 },
    PlayerTouches { slot: u16 },
    EnemyScreenCheck,
    QueueSfx3Max6 { sound: u16 },
    HorizontalShutter(HorizontalShutter),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response {
    Done,
    Word(u16),
}
pub trait Services {
    /// Immutable whole-entry admission before writes/requests, including all
    /// future copybacks and Main's postdispatch powerbomb path. Bind ROM, speed
    /// row, slot/allocation and stable CURRENT, generation and sibling fences.
    /// Preserve Func2 index<5 after callbacks and exact typed shutter selector
    /// semantics; no native effect/observation may occur in this method.
    fn admitted(&self, entry: Entry, ram: &[u8], rom: &[u8]) -> bool;
    /// Synchronous maintained common function with full WRAM copyback. Accepted
    /// failure/wrong result must terminate the root while preserving effects.
    fn request(&mut self, ram: &mut [u8], request: Request) -> Result<Response, String>;
}

/// None is unchanged no-request refusal. The adapter must prohibit retry on Err.
pub fn execute(entry: Entry, ram: &mut [u8], rom: &[u8], services: &mut impl Services) -> Result<Option<()>, String> {
    if ram.len() != WRAM {
        return Ok(None);
    }
    let current = word(ram, CURRENT);
    let slot = match entry {
        Entry::Function { slot, .. } => slot,
        _ => current,
    };
    if !valid_slot(slot) || !valid_slot(current) {
        return Ok(None);
    }
    if let Entry::Function { number, .. } = entry {
        if !(1..=13).contains(&number) || (!matches!(number, 1 | 7) && slot != current) {
            return Ok(None);
        }
    }
    let selected = match entry {
        Entry::Main => match selector(0xa20000 | u32::from(word(ram, base(slot) + 0x30))) {
            Some(v) => Some(v),
            None => return Ok(None),
        },
        Entry::Dispatch { address } => match selector(address) {
            Some(v) => Some(v),
            None => return Ok(None),
        },
        _ => None,
    };
    let initializing = matches!(entry, Entry::Init | Entry::Function { number: 1, .. });
    if initializing {
        let row = usize::from(word(ram, base(slot) + 0x1a) & 0xff);
        if rom.get(SPEEDS + row * 8..SPEEDS + row * 8 + 8).is_none() || word(ram, base(slot) + 0x3c) & 0xff > 4 {
            return Ok(None);
        }
    } else if matches!(entry, Entry::Function { number: 2, .. }) || matches!(selected, Some(Selector::Local(2))) {
        if ext(ram, slot, 7) >> 1 >= 5 {
            return Ok(None);
        }
    }
    if !services.admitted(entry, ram, rom) {
        return Ok(None);
    }
    let mut actor = Actor { ram, rom, services, slot, current };
    match entry {
        Entry::Init => {
            actor.initialize();
            actor.set(0x1a, 0xede7);
        }
        Entry::Main => {
            actor.dispatch(selected.expect("preflight Main selector"))?;
            let state = actor.get(0x30);
            let mut contact = 0xffff;
            for i in 0..4 {
                contact &= word(actor.ram, 0x182c + 2 * i);
            }
            if state != UP
                && state != DOWN
                && contact != 0xffff
                && contact == actor.current
                && word(actor.ram, 0xa6e) != 0
            {
                actor.powerbomb()?;
            }
        }
        Entry::Dispatch { .. } => actor.dispatch(selected.expect("preflight dispatch selector"))?,
        Entry::Function { number, .. } => actor.function(number)?,
        Entry::Touch | Entry::Shot | Entry::Powerbomb => actor.powerbomb()?,
    }
    Ok(Some(()))
}
fn valid_slot(slot: u16) -> bool {
    slot <= 0x7c0 && slot & 0x3f == 0
}
fn base(slot: u16) -> usize {
    0xf78 + usize::from(slot)
}
fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn ext_at(slot: u16, n: usize) -> usize {
    if n < 0x20 { 0x7800 + usize::from(slot) + 2 * n } else { 0x8000 + usize::from(slot) + 2 * (n - 0x20) }
}
fn ext(ram: &[u8], slot: u16, n: usize) -> u16 {
    word(ram, ext_at(slot, n))
}
#[derive(Clone, Copy)]
enum Selector {
    Local(u8),
    Foreign(HorizontalShutter),
}
fn selector(address: u32) -> Option<Selector> {
    Some(match address {
        0xa2ef09 => Selector::Local(2),
        0xa2ef28 => Selector::Local(4),
        0xa2ef68 => Selector::Local(9),
        0xa2efd4 => Selector::Local(10),
        0xa2f040 => Selector::Local(11),
        0xa2f072 => Selector::Local(12),
        0xa2f099 => Selector::Local(13),
        0xa2f224 => Selector::Foreign(HorizontalShutter::Function2),
        0xa2f243 => Selector::Foreign(HorizontalShutter::Function4),
        0xa2f272 => Selector::Foreign(HorizontalShutter::Function8),
        0xa2f2e4 => Selector::Foreign(HorizontalShutter::Function9),
        0xa2f38c => Selector::Foreign(HorizontalShutter::Function12),
        0xa2f3b0 => Selector::Foreign(HorizontalShutter::Function13),
        0xa2f3d4 => Selector::Foreign(HorizontalShutter::Function14),
        _ => return None,
    })
}
struct Actor<'a, S> {
    ram: &'a mut [u8],
    rom: &'a [u8],
    services: &'a mut S,
    slot: u16,
    current: u16,
}
impl<S: Services> Actor<'_, S> {
    fn get(&self, n: usize) -> u16 {
        word(self.ram, base(self.slot) + n)
    }
    fn set(&mut self, n: usize, v: u16) {
        put(self.ram, base(self.slot) + n, v);
    }
    fn ext(&self, n: usize) -> u16 {
        ext(self.ram, self.slot, n)
    }
    fn set_ext(&mut self, n: usize, v: u16) {
        put(self.ram, ext_at(self.slot, n), v);
    }
    fn value(&mut self, request: Request) -> Result<u16, String> {
        match self.services.request(self.ram, request)? {
            Response::Word(v) => Ok(v),
            _ => Err("platform service returned wrong word result".into()),
        }
    }
    fn done(&mut self, request: Request) -> Result<(), String> {
        match self.services.request(self.ram, request)? {
            Response::Done => Ok(()),
            _ => Err("platform service returned wrong void result".into()),
        }
    }
    fn dispatch(&mut self, selected: Selector) -> Result<(), String> {
        match selected {
            Selector::Local(n) => self.function(n),
            Selector::Foreign(n) => self.done(Request::HorizontalShutter(n)),
        }
    }
    fn initialize(&mut self) {
        let instruction = self.get(0x1a);
        let low = instruction & 0xff;
        let high = instruction >> 8;
        self.set_ext(0, low);
        let at = SPEEDS + usize::from(low) * 8;
        self.set(0x36, word(self.rom, at));
        self.set(0x34, word(self.rom, at + 2));
        self.set(0x3a, word(self.rom, at + 4));
        self.set(0x38, word(self.rom, at + 6));
        self.set_ext(1, high);
        self.set_ext(0x20, high);
        let properties = self.get(0x10);
        let a = properties & 0xff;
        let b = properties >> 8;
        self.set_ext(2, a);
        self.set_ext(8, a * 16);
        self.set_ext(3, b);
        self.set_ext(9, b * 16);
        let parameter = self.get(0x3c);
        let mode = parameter & 0xff;
        let range = parameter >> 8;
        self.set_ext(4, mode);
        self.set_ext(7, mode * 2);
        self.set_ext(5, range);
        let timer = self.get(0x3e);
        self.set_ext(6, timer);
        self.set(0x32, timer);
        let y = self.get(6);
        self.set_ext(0xf, y);
        self.set_ext(0x10, y.wrapping_add(range));
        if high == 0 {
            self.set_ext(0x10, y);
            self.set_ext(0xf, y.wrapping_sub(range));
        }
        self.set(0x30, FUNC2);
        self.set(0x10, 0);
        self.set_ext(0xa, 0);
    }
    fn sound(&mut self) -> Result<(), String> {
        // Recovered CheckIfEnemyIsOnScreen returns nonzero for off-screen.
        if self.value(Request::EnemyScreenCheck)? == 0 {
            self.done(Request::QueueSfx3Max6 { sound: 0xe })?;
        }
        Ok(())
    }
    fn start(&mut self) -> Result<(), String> {
        self.set(0x30, if self.ext(1) != 0 { DOWN } else { UP });
        self.sound()
    }
    fn move_y(&mut self, down: bool) -> Result<(), String> {
        self.set_ext(0xe, self.get(6));
        self.set_ext(0xa, 0);
        if self.value(Request::PlayerTouches { slot: self.slot })? != 0 {
            self.set_ext(0xa, 1);
        }
        let amount = if down {
            u32::from(self.get(0x36)) << 16 | u32::from(self.get(0x34))
        } else {
            u32::from(self.get(0x3a)) << 16 | u32::from(self.get(0x38))
        };
        let position = (u32::from(self.get(6)) << 16 | u32::from(self.get(8))).wrapping_add(amount);
        self.set(6, (position >> 16) as u16);
        self.set(8, position as u16);
        if self.ext(0xa) != 0 {
            let displacement = self.get(6).wrapping_sub(self.ext(0xe));
            put(self.ram, 0xb5c, displacement);
        }
        let reached = if down {
            (self.get(6).wrapping_sub(self.ext(0x10)) as i16) >= 0
        } else {
            (self.ext(0xf).wrapping_sub(self.get(6)) as i16) >= 0
        };
        if reached {
            let delay = self.ext(if down { 9 } else { 8 });
            if delay == 4080 {
                self.set(0x30, STOP);
            } else {
                self.set(0x32, delay);
                self.set(0x30, if down { WAIT_DOWN } else { WAIT_UP });
            }
        }
        Ok(())
    }
    fn function(&mut self, n: u8) -> Result<(), String> {
        match n {
            1 => self.initialize(),
            2 => {
                let index = self.ext(7) >> 1;
                if index >= 5 {
                    return Err("platform trigger index changed outside admission".into());
                }
                self.function([3, 4, 5, 6, 6][usize::from(index)])?;
            }
            3 => {
                let old = self.get(0x32);
                self.set(0x32, old.wrapping_sub(1));
                if old == 1 {
                    self.set(0x32, self.ext(6));
                    self.start()?;
                }
            }
            4 => {
                if self.value(Request::PlayerWithinX { slot: self.slot, distance: self.ext(6) })? != 0 {
                    self.start()?;
                }
            }
            5 | 7 => self.start()?,
            6 | 13 => {}
            8 => self.sound()?,
            9 | 10 => self.move_y(n == 10)?,
            11 => {
                let timer = self.get(0x32).wrapping_sub(1);
                self.set(0x32, timer);
                if timer & 0x8000 != 0 {
                    self.set(0x30, DOWN);
                    if self.ext(4) == 1 && self.ext(1) != 0 {
                        self.set(0x30, FUNC4);
                    }
                    if self.get(0) != 0xd83f {
                        self.sound()?;
                    }
                }
            }
            12 => {
                let timer = self.get(0x32).wrapping_sub(1);
                self.set(0x32, timer);
                if timer & 0x8000 != 0 {
                    self.sound()?;
                    self.set(0x30, UP);
                    if self.ext(4) == 1 && self.ext(1) == 0 {
                        self.set(0x30, FUNC4);
                    }
                }
            }
            _ => return Err("unknown platform function after admission".into()),
        }
        Ok(())
    }
    fn powerbomb(&mut self) -> Result<(), String> {
        let mode = self.ext(7);
        if (mode.wrapping_sub(6) as i16) < 0 {
            return self.sound();
        }
        if mode != 8 {
            if self.ext(0xc) != 0 {
                return Ok(());
            }
            self.set_ext(0xc, 1);
        }
        if !matches!(self.get(0x30), UP | DOWN) {
            self.set(0x30, if self.ext(0x20) != 0 { DOWN } else { UP });
            self.set_ext(0x20, self.ext(0x20) ^ 1);
            self.sound()?;
        }
        Ok(())
    }
}
