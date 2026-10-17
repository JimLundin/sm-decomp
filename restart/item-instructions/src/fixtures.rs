//! Frozen B01..B25 / R01..R14 inputs shared by unit and actual-C controls.
//! Direct writes exclude native service effects: actual-C controls compare
//! complete native/candidate WRAM and service snapshots independently.
use crate::{Entry, Output, Service, Services, execute, memory::*};
use sha2::{Digest, Sha256};

#[path = "morph_fixtures.rs"]
mod morph_fixtures;
pub use morph_fixtures::{morph_body_fixtures, morph_refusal_fixtures};

#[path = "tank_fixtures.rs"]
mod tank_fixtures;
pub use tank_fixtures::{tank_body_fixtures, tank_refusal_fixtures};

pub const SEED_SHA256: &str = "7adc5568e4239e479fcef325fef05769da8f6399e5909174d5f6f00c2f436459";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expectation {
    Handled(Output),
    Refused,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Write {
    Byte(usize, u8),
    Word(usize, u16),
}
impl Write {
    pub fn apply(self, ram: &mut [u8]) {
        match self {
            Self::Byte(at, value) => ram[at] = value,
            Self::Word(at, value) => set_word(ram, at, value),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Fixture {
    pub id: &'static str,
    /// None only for R07, whose invalid raw_entry cannot inhabit Entry.
    pub entry: Option<Entry>,
    pub raw_entry: u32,
    pub slot: u16,
    pub operand: Option<u16>,
    pub ram: Vec<u8>,
    pub rom: Vec<u8>,
    pub services_available: bool,
    pub expected: Expectation,
    pub direct_writes: Vec<Write>,
    pub service_calls: Vec<(Service, u16)>,
}
impl Fixture {
    /// Preserves the malformed-entry case without constructing an invalid enum.
    pub fn run(&mut self, services: &mut dyn Services) -> Result<Option<Output>, String> {
        let entry = Entry::try_from(self.raw_entry)?;
        if self.entry != Some(entry) {
            return Err("fixture entry identity mismatch".into());
        }
        execute(entry, self.slot, self.operand, &mut self.ram, &self.rom, services)
    }
}

fn validate_inputs(seed: &[u8], rom: &[u8]) -> Result<(), String> {
    if seed.len() != WRAM_SIZE || format!("{:x}", Sha256::digest(seed)) != SEED_SHA256 {
        return Err("item fixtures require the sealed ticket17 revealed WRAM seed".into());
    }
    validate_rom(rom)
}

fn base(seed: &[u8], slot: u16) -> Vec<u8> {
    let mut ram = seed.to_vec();
    let k = usize::from(slot);
    if k != 78 {
        // Relocate all selected-slot arrays, then retire its old header.
        for at in [HEADER, 0x1c87, PRE, IP, TIMER, ARGUMENT, 0x1e17, INSTRUCTION_TIMER, 0xde6c, LINK, 0xdf0c] {
            let value = word(&ram, at + 78);
            set_word(&mut ram, at + k, value);
        }
        set_word(&mut ram, HEADER + 78, 0);
    }
    let neighbor = if k == 0 { 2 } else { 76 };
    for at in [PRE, IP, TIMER, ARGUMENT, INSTRUCTION_TIMER, LINK] {
        set_word(&mut ram, at + neighbor, 0x5aa5);
    }
    for (at, value) in [
        (HEADER + k, 0xef2f),
        (ARGUMENT + k, 0x22),
        (PRE + k, 0xdf89),
        (IP + k, 0xdfb7),
        (LINK + k, 0xe484),
        (TIMER + k, 0),
        (INSTRUCTION_TIMER + k, 0xbeef),
        (0x1c27, slot),
        (0x998, 8),
        (0x5e7, 0xa55a),
        (0x9c6, 7),
        (0x9c8, 10),
        (0xdc8, 0x7777),
        (0x639, 0),
        (0x63b, 6),
        (0x63d, 0x1234),
        (0x63f, 0x5678),
        (0x7f5, 5),
        (0xc61c, 0x2c0f),
        (0xc61e, 0x1111),
        (0xc620, 0x2222),
        (0xc65c, 0x3333),
        (0xc65e, 0x4444),
        (0xc660, 0x5555),
    ] {
        set_word(&mut ram, at, value);
    }
    ram[0xd874] = 0xa9;
    for i in 0..8 {
        set_word(&mut ram, 0x619 + 2 * i, 0x100 + i as u16);
        set_word(&mut ram, 0x629 + 2 * i, 0x200 + i as u16);
    }
    ram[0x1ffff] = 0x6d;
    ram
}

fn case(
    seed: &[u8],
    rom: &[u8],
    id: &'static str,
    entry: Entry,
    slot: u16,
    operand: Option<u16>,
    output: Output,
) -> Fixture {
    Fixture {
        id,
        entry: Some(entry),
        raw_entry: entry as u32,
        slot,
        operand,
        ram: base(seed, slot),
        rom: rom.to_vec(),
        services_available: true,
        expected: Expectation::Handled(output),
        direct_writes: vec![],
        service_calls: vec![],
    }
}

pub fn body_fixtures(seed: &[u8], rom: &[u8]) -> Result<Vec<Fixture>, String> {
    validate_inputs(seed, rom)?;
    use Entry::*;
    use Output::*;
    use Write::*;
    let specs = [
        ("B01", Delete, 0, Some(0xe4ae), Stop),
        ("B02", PreInstr, 78, Some(0xe48e), Continue(0xe490)),
        ("B03", PreInstr, 0, Some(0xdfb1), Continue(0xdfb3)),
        ("B04", ClearPreInstr, 78, Some(0xdfc9), Continue(0xdfc9)),
        ("B05", Goto, 78, Some(0xe49d), Continue(0xe493)),
        ("B06", Goto, 78, Some(0xdfc5), Continue(0xdfb3)),
        ("B07", SetTimer, 0, Some(0xe492), Continue(0xe493)),
        ("B08", GotoIfItemBitSet, 78, Some(0xe47e), Continue(0xe480)),
        ("B09", GotoIfItemBitSet, 78, Some(0xe47e), Continue(0xe4a8)),
        ("B10", SetItemBit, 78, Some(0xe4a1), Continue(0xe4a1)),
        ("B11", SetItemBit, 78, Some(0xe4a1), Continue(0xe4a1)),
        ("B12", CollectAmmoMissileTank, 78, Some(0xe4a6), Continue(0xe4a8)),
        ("B13", CollectAmmoMissileTank, 78, Some(0xe4a6), Continue(0xe4a8)),
        ("B14", CollectAmmoMissileTank, 78, Some(0xe4a6), Continue(0xe4a8)),
        ("B15", SetLinkReg, 78, Some(0xe48a), Continue(0xe48c)),
        ("B16", Call, 78, Some(0xe482), Continue(0xdfaf)),
        ("B17", Call, 78, Some(0xe486), Continue(0xdfc7)),
        ("B18", Return, 78, Some(0xdfd7), Continue(0xe488)),
        ("B19", ClearMusicQueueAndQueueTrack, 78, Some(0xe4a3), Continue(0xe4a4)),
        ("B20", ClearMusicQueueAndQueueTrack, 78, Some(0xe4a3), Continue(0xe4a4)),
        ("B21", CollectAmmoMissileTank, 78, Some(0xe4a6), Continue(0xe4a8)),
        ("B22", GotoLinkIfTriggered, 78, None, PreinstructionDone),
        ("B23", GotoLinkIfTriggered, 78, None, PreinstructionDone),
        ("B24", GotoLinkIfTriggered, 78, None, PreinstructionDone),
        ("B25", GotoLinkIfTriggered, 78, None, PreinstructionDone),
    ];
    let mut cases = Vec::with_capacity(specs.len());
    for (id, entry, slot, operand, output) in specs {
        let mut f = case(seed, rom, id, entry, slot, operand, output);
        let k = usize::from(slot);
        match id {
            "B01" => f.direct_writes = vec![Word(HEADER + k, 0)],
            "B02" | "B03" => {
                set_word(&mut f.ram, PRE + k, 0xa55a);
                f.direct_writes = vec![Word(PRE + k, 0xdf89)];
            }
            "B04" => f.direct_writes = vec![Word(PRE + k, 0x86d0)],
            "B07" => {
                set_word(&mut f.ram, TIMER + k, 0xabcd);
                f.direct_writes = vec![Byte(TIMER + k, 0x16)];
            }
            "B08" | "B09" => {
                if id == "B09" {
                    f.ram[0xd874] = 0xad;
                }
                f.direct_writes = vec![Word(0x5e7, 4)];
            }
            "B10" | "B11" => {
                if id == "B11" {
                    f.ram[0xd874] = 0xad;
                }
                f.direct_writes = vec![Word(0x5e7, 4), Byte(0xd874, 0xad)];
            }
            "B12" | "B13" | "B14" | "B21" => {
                let (ammo, cap, new_ammo, new_cap) = if id == "B13" { (0xfffe, 0xfffc, 3, 1) } else { (0, 0, 5, 5) };
                set_word(&mut f.ram, 0x9c6, ammo);
                set_word(&mut f.ram, 0x9c8, cap);
                if id == "B13" {
                    set_word(&mut f.ram, 0xc61c, 0x344b);
                }
                if id == "B14" {
                    set_word(&mut f.ram, 0x639, 14);
                    set_word(&mut f.ram, 0x63b, 2);
                }
                if id == "B21" {
                    set_word(&mut f.ram, 0x998, 40);
                }
                f.direct_writes = vec![Word(0x9c8, new_cap), Word(0x9c6, new_ammo), Word(0xdc8, 2)];
                f.service_calls = vec![(Service::AddMissilesToHud, 0), (Service::PlayRoomMusicAfter, 0x168)];
            }
            "B15" | "B16" | "B17" => {
                set_word(&mut f.ram, LINK + k, 0xa55a);
                let value = match id {
                    "B15" => 0xe49f,
                    "B16" => 0xe484,
                    _ => 0xe488,
                };
                f.direct_writes = vec![Word(LINK + k, value)];
            }
            "B18" => set_word(&mut f.ram, LINK + k, 0xe488),
            "B19" | "B20" => {
                set_word(&mut f.ram, 0x639, 14);
                if id == "B20" {
                    set_word(&mut f.ram, 0x998, 40);
                }
                for i in (0..8).rev() {
                    f.direct_writes.push(Word(0x619 + i * 2, 0));
                    f.direct_writes.push(Word(0x629 + i * 2, 0));
                }
                f.direct_writes.extend([Word(0x63b, 14), Word(0x63f, 0), Word(0x63d, 0)]);
                f.service_calls = vec![(Service::QueueMusicDelayed8, 2)];
            }
            "B23" => {
                set_word(&mut f.ram, IP + k, 0xdfc3);
                set_word(&mut f.ram, TIMER + k, 0xffff);
                f.direct_writes = vec![Word(PRE + k, 0xdfa8), Word(IP + k, 0xe484), Word(INSTRUCTION_TIMER + k, 1)];
            }
            "B24" | "B25" => {
                set_word(&mut f.ram, LINK + k, 0xe49f);
                set_word(&mut f.ram, IP + k, if id == "B24" { 0xe49b } else { 0xe497 });
                set_word(&mut f.ram, TIMER + k, if id == "B24" { 0xabff } else { 0xff00 });
                if id == "B24" {
                    f.direct_writes = vec![Word(PRE + k, 0xdfa8), Word(IP + k, 0xe49f), Word(INSTRUCTION_TIMER + k, 1)];
                }
            }
            _ => {}
        }
        cases.push(f);
    }
    Ok(cases)
}

pub fn refusal_fixtures(seed: &[u8], rom: &[u8]) -> Result<Vec<Fixture>, String> {
    validate_inputs(seed, rom)?;
    let mut cases = Vec::with_capacity(14);
    for id in ["R01", "R02", "R03", "R04", "R05", "R06", "R07", "R08", "R09", "R10", "R11", "R12", "R13", "R14"] {
        let mut f = case(seed, rom, id, Entry::CollectAmmoMissileTank, 78, Some(0xe4a6), Output::Continue(0xe4a8));
        f.expected = Expectation::Error;
        match id {
            "R01" => f.slot = 1,
            "R02" => f.slot = 80,
            "R03" => f.ram.truncate(0x1ffff),
            "R04" => {
                set_word(&mut f.ram, HEADER + 78, 0xef2b);
                f.expected = Expectation::Refused;
            }
            "R05" => {
                set_word(&mut f.ram, ARGUMENT + 78, 0x23);
                f.expected = Expectation::Refused;
            }
            "R06" => {
                f.entry = Some(Entry::SetTimer);
                f.operand = Some(0xe493);
            }
            "R07" => {
                f.entry = None;
                f.raw_entry = 21;
            }
            "R08" => f.rom[rom_index(0xe4a6)] = 6,
            "R09" => f.rom.truncate(rom_index(0xe4ad)),
            "R10" => {
                f.entry = Some(Entry::Return);
                f.operand = Some(0xdfd7);
            }
            "R11" => {
                f.entry = Some(Entry::GotoLinkIfTriggered);
                f.operand = None;
                set_word(&mut f.ram, LINK + 78, 0xe49f);
            }
            "R12" => {
                f.entry = Some(Entry::GotoLinkIfTriggered);
                f.operand = None;
                set_word(&mut f.ram, PRE + 78, 0x86d0);
            }
            "R13" => f.services_available = false,
            "R14" => {
                set_word(&mut f.ram, 0x639, 3);
                set_word(&mut f.ram, 0x63b, 2);
            }
            _ => unreachable!(),
        }
        if let Some(entry) = f.entry {
            f.raw_entry = entry as u32;
        }
        cases.push(f);
    }
    Ok(cases)
}
