//! Ticket20 distinguishing fixtures; the old body inventories remain unchanged.
use super::{Expectation, Fixture, Write, base, validate_inputs};
use crate::{
    Entry, Output, Service,
    memory::*,
    morph::{BLOCK, DRAW, GFX},
};

fn case(
    seed: &[u8],
    rom: &[u8],
    id: &'static str,
    header: u16,
    entry: Entry,
    operand: Option<u16>,
    output: Output,
) -> Fixture {
    let (argument, link, ip, gfx) = match header {
        0xeed7 => (0, 0xe0b1, 0xe0a9, 8),
        0xeedb => (7, 0xe0d6, 0xe0ce, 10),
        0xeedf => (8, 0xe0fb, 0xe0f3, 12),
        0xeee3 => (511, 0xe120, 0xe118, 14),
        0xef27 => (511, 0xe43d, 0xe437, 0),
        _ => unreachable!(),
    };
    let mut ram = base(seed, 68);
    for (at, value) in [
        (HEADER + 68, header),
        (ARGUMENT + 68, argument),
        (LINK + 68, link),
        (IP + 68, ip),
        (BLOCK + 68, 0x298a),
        (DRAW + 68, 0xa30f),
        (GFX + 68, gfx),
        (0x1c2d, 0),
        (0x330, 0),
        (0x1c25, 0),
        (0x7a5, 128),
        (0x911, 1024),
        (0x915, 640),
        (0x91d, 0),
        (0x9c2, 7),
        (0x9c4, 99),
        (0x9c0, 0),
        (0x9d4, 0),
        (0x9d6, 37),
        (0x9ca, 0),
        (0x9cc, 0),
        (0x9ce, 0),
        (0x9d0, 0),
        (0xc624, 0x2c0f),
        (0xc626, 0x1111),
        (0xc664, 0x2222),
        (0xc666, 0x3333),
        (0xc62a, 0x2c0f),
        (0xc62c, 0x4444),
        (0xc66a, 0x5555),
        (0xc66c, 0x6666),
    ] {
        set_word(&mut ram, at, value);
    }
    ram[0xd870..0xd8b0].fill(0x5a);
    Fixture {
        id,
        entry: Some(entry),
        raw_entry: entry as u32,
        slot: 68,
        operand,
        ram,
        rom: rom.to_vec(),
        services_available: true,
        expected: Expectation::Handled(output),
        direct_writes: vec![],
        service_calls: vec![],
    }
}

fn validate(seed: &[u8], rom: &[u8]) -> Result<(), String> {
    validate_inputs(seed, rom)?;
    for header in [0xeed7, 0xeedb, 0xeedf, 0xeee3, 0xef27] {
        crate::tanks::Domain::from_header(header).unwrap().validate_rom(rom)?;
    }
    Ok(())
}

pub fn tank_body_fixtures(seed: &[u8], rom: &[u8]) -> Result<Vec<Fixture>, String> {
    validate(seed, rom)?;
    use Entry::*;
    use Output::*;
    use Write::*;
    let specs = [
        ("T01", 0xeed7, CollectHealthEnergyTank, Some(0xe0b8), Continue(0xe0ba)),
        ("T02", 0xeed7, CollectHealthEnergyTank, Some(0xe0b8), Continue(0xe0ba)),
        ("T03", 0xeed7, CollectHealthEnergyTank, Some(0xe0b8), Continue(0xe0ba)),
        ("T04", 0xef27, CollectHealthReserveTank, Some(0xe444), Continue(0xe446)),
        ("T05", 0xef27, CollectHealthReserveTank, Some(0xe444), Continue(0xe446)),
        ("T06", 0xef27, CollectHealthReserveTank, Some(0xe444), Continue(0xe446)),
        ("T07", 0xef27, CollectHealthReserveTank, Some(0xe444), Continue(0xe446)),
        ("T08", 0xeedf, CollectAmmoSuperMissileTank, Some(0xe102), Continue(0xe104)),
        ("T09", 0xeedf, CollectAmmoSuperMissileTank, Some(0xe102), Continue(0xe104)),
        ("T10", 0xeedf, CollectAmmoSuperMissileTank, Some(0xe102), Continue(0xe104)),
        ("T11", 0xeee3, CollectAmmoPowerBombTank, Some(0xe127), Continue(0xe129)),
        ("T12", 0xeee3, CollectAmmoPowerBombTank, Some(0xe127), Continue(0xe129)),
        ("T13", 0xeee3, CollectAmmoPowerBombTank, Some(0xe127), Continue(0xe129)),
        ("T14", 0xeedb, CollectAmmoMissileTank, Some(0xe0dd), Continue(0xe0df)),
        ("T15", 0xeed7, GotoIfItemBitSet, Some(0xe09b), Continue(0xe09d)),
        ("T16", 0xeed7, SetItemBit, Some(0xe0b3), Continue(0xe0b3)),
        ("T17", 0xeedb, GotoIfItemBitSet, Some(0xe0c0), Continue(0xe0df)),
        ("T18", 0xeedb, SetItemBit, Some(0xe0d8), Continue(0xe0d8)),
        ("T19", 0xeedf, GotoIfItemBitSet, Some(0xe0e5), Continue(0xe0e7)),
        ("T20", 0xeedf, SetItemBit, Some(0xe0fd), Continue(0xe0fd)),
        ("T21", 0xeee3, GotoIfItemBitSet, Some(0xe10a), Continue(0xe129)),
        ("T22", 0xeee3, SetItemBit, Some(0xe122), Continue(0xe122)),
        ("T23", 0xeed7, SetLinkReg, Some(0xe09f), Continue(0xe0a1)),
        ("T24", 0xeedb, PreInstr, Some(0xe0c8), Continue(0xe0ca)),
        ("T25", 0xeedf, Goto, Some(0xe0f9), Continue(0xe0ef)),
        ("T26", 0xeee3, Goto, Some(0xe12b), Continue(0xdfa9)),
        ("T27", 0xef27, Delete, Some(0xdfaf), Stop),
        ("T28", 0xef27, ClearMusicQueueAndQueueTrack, Some(0xe441), Continue(0xe442)),
        ("T29", 0xeed7, GotoLinkIfTriggered, None, PreinstructionDone),
        ("T30", 0xeedb, GotoLinkIfTriggered, None, PreinstructionDone),
        ("T31", 0xeedf, GotoLinkIfTriggered, None, PreinstructionDone),
        ("T32", 0xeee3, GotoLinkIfTriggered, None, PreinstructionDone),
        ("T33", 0xef27, GotoLinkIfTriggered, None, PreinstructionDone),
        ("T34", 0xef27, LoadItemPlmGfx, Some(0xe41f), Continue(0xe429)),
        ("T35", 0xef27, DrawItemFrame0, Some(0xe437), Stop),
        ("T36", 0xef27, DrawItemFrame1, Some(0xe439), Stop),
    ];
    let mut cases = Vec::new();
    for (id, header, entry, operand, output) in specs {
        let mut f = case(seed, rom, id, header, entry, operand, output);
        match entry {
            CollectHealthEnergyTank => {
                let maximum = if id == "T02" {
                    set_word(&mut f.ram, 0x9c4, 0xffef);
                    83
                } else {
                    199
                };
                if id == "T03" {
                    set_word(&mut f.ram, 0x998, 40);
                }
                f.direct_writes = vec![Word(0x9c4, maximum), Word(0x9c2, maximum), Word(0xdc8, 1)];
                f.service_calls = vec![(Service::PlayRoomMusicAfter, 0x168)];
            }
            CollectHealthReserveTank => {
                let (mode, maximum) = match id {
                    "T05" => {
                        set_word(&mut f.ram, 0x9c0, 2);
                        set_word(&mut f.ram, 0x9d4, 100);
                        (2, 200)
                    }
                    "T06" => {
                        set_word(&mut f.ram, 0x9c0, 0xffff);
                        set_word(&mut f.ram, 0x9d4, 0xffef);
                        (0xffff, 83)
                    }
                    _ => (1, 100),
                };
                if id == "T07" {
                    set_word(&mut f.ram, 0x998, 40);
                }
                f.direct_writes = vec![Word(0x9d4, maximum), Word(0x9c0, mode), Word(0xdc8, 0x19)];
                f.service_calls = vec![(Service::PlayRoomMusicAfter, 0x168)];
            }
            CollectAmmoSuperMissileTank | CollectAmmoPowerBombTank => {
                let (current, capacity, tile, service, message) = if entry == CollectAmmoSuperMissileTank {
                    (0x9ca, 0x9cc, 0xc624, Service::AddSuperMissilesToHud, 3)
                } else {
                    (0x9ce, 0x9d0, 0xc62a, Service::AddPowerBombsToHud, 4)
                };
                let (new_current, new_capacity) = match id {
                    "T09" | "T12" => {
                        set_word(&mut f.ram, current, 7);
                        set_word(&mut f.ram, capacity, 10);
                        set_word(&mut f.ram, tile, 0x344b);
                        (12, 15)
                    }
                    "T10" | "T13" => {
                        set_word(&mut f.ram, current, 0xfffe);
                        set_word(&mut f.ram, capacity, 0xfffc);
                        (3, 1)
                    }
                    _ => (5, 5),
                };
                f.direct_writes = vec![Word(capacity, new_capacity), Word(current, new_current), Word(0xdc8, message)];
                f.service_calls = vec![(service, 0), (Service::PlayRoomMusicAfter, 0x168)];
            }
            CollectAmmoMissileTank => {
                f.direct_writes = vec![Word(0x9c8, 15), Word(0x9c6, 12), Word(0xdc8, 2)];
                f.service_calls = vec![(Service::AddMissilesToHud, 0), (Service::PlayRoomMusicAfter, 0x168)];
            }
            GotoIfItemBitSet | SetItemBit => {
                let (byte, mask) = match header {
                    0xeed7 => (0xd870, 1),
                    0xeedb => (0xd870, 128),
                    0xeedf => (0xd871, 1),
                    _ => (0xd8af, 128),
                };
                if mask == 128 {
                    f.ram[byte] |= 128;
                }
                f.direct_writes.push(Word(0x5e7, u16::from(mask)));
                if entry == SetItemBit {
                    f.direct_writes.push(Byte(byte, f.ram[byte] | mask));
                }
            }
            SetLinkReg => {
                set_word(&mut f.ram, LINK + 68, 0xa55a);
                f.direct_writes.push(Word(LINK + 68, 0xe0b1));
            }
            PreInstr => {
                set_word(&mut f.ram, PRE + 68, 0xa55a);
                f.direct_writes.push(Word(PRE + 68, 0xdf89));
            }
            Delete => f.direct_writes.push(Word(HEADER + 68, 0)),
            ClearMusicQueueAndQueueTrack => {
                set_word(&mut f.ram, 0x639, 14);
                for i in (0..8).rev() {
                    f.direct_writes.extend([Word(0x619 + i * 2, 0), Word(0x629 + i * 2, 0)]);
                }
                f.direct_writes.extend([Word(0x63b, 14), Word(0x63f, 0), Word(0x63d, 0)]);
                f.service_calls.push((Service::QueueMusicDelayed8, 2));
            }
            GotoLinkIfTriggered => {
                if id == "T30" {
                    set_word(&mut f.ram, TIMER + 68, 0xff00);
                    set_word(&mut f.ram, IP + 68, 0xe0d2);
                }
                if id == "T31" {
                    set_word(&mut f.ram, IP + 68, 0xe0f7);
                }
                if id == "T33" {
                    set_word(&mut f.ram, IP + 68, 0xe439);
                }
                if matches!(id, "T29" | "T31" | "T33") {
                    set_word(&mut f.ram, TIMER + 68, 0xabff);
                    f.direct_writes = vec![
                        Word(PRE + 68, 0xdfa8),
                        Word(IP + 68, word(&f.ram, LINK + 68)),
                        Word(INSTRUCTION_TIMER + 68, 1),
                    ];
                }
            }
            LoadItemPlmGfx => {
                set_word(&mut f.ram, 0x1c2d, 6);
                set_word(&mut f.ram, 0x330, 601);
                f.direct_writes = vec![
                    Word(GFX + 68, 6),
                    Word(0x1c2d, 0),
                    Word(0x1c35, 0xe41f),
                    Word(0x329, 256),
                    Word(0x32b, 0x9000),
                    Byte(0x32d, 0x89),
                    Word(0x32e, 0x3f80),
                    Word(0x330, 608),
                ];
                for i in 0..8 {
                    f.direct_writes.push(Word(0xa4a0 + i * 2, 0x3f8 + i as u16));
                }
            }
            DrawItemFrame0 | DrawItemFrame1 => {
                let pointer = if id == "T36" {
                    set_word(&mut f.ram, GFX + 68, 6);
                    set_word(&mut f.ram, 0x1c25, 512);
                    0xa339
                } else {
                    0xa30f
                };
                set_word(&mut f.ram, DRAW + 68, 0xa55a);
                f.direct_writes =
                    vec![Word(DRAW + 68, pointer), Word(INSTRUCTION_TIMER + 68, 4), Word(IP + 68, operand.unwrap())];
                f.service_calls = vec![
                    (Service::ProcessPlmDrawInstruction, 68),
                    (Service::CalculatePlmBlockCoords, 68),
                    (Service::DrawPlm, 68),
                ];
            }
            _ => {}
        }
        cases.push(f);
    }
    Ok(cases)
}

pub fn tank_refusal_fixtures(seed: &[u8], rom: &[u8]) -> Result<Vec<Fixture>, String> {
    validate(seed, rom)?;
    let mut cases = Vec::new();
    for id in [
        "TR01", "TR02", "TR03", "TR04", "TR05", "TR06", "TR07", "TR08", "TR09", "TR10", "TR11", "TR12", "TR13", "TR14",
        "TR15", "TR16", "TR17", "TR18", "TR19", "TR20", "TR21",
    ] {
        let mut f =
            case(seed, rom, id, 0xeedf, Entry::CollectAmmoSuperMissileTank, Some(0xe102), Output::Continue(0xe104));
        f.expected = Expectation::Error;
        if matches!(id, "TR09" | "TR10" | "TR11" | "TR12" | "TR13" | "TR18" | "TR19" | "TR20") {
            f = case(seed, rom, id, 0xef27, Entry::LoadItemPlmGfx, Some(0xe41f), Output::Continue(0xe429));
            f.expected = Expectation::Error;
        }
        match id {
            "TR01" => {
                set_word(&mut f.ram, HEADER + 68, 0xeed8);
                f.expected = Expectation::Refused;
            }
            "TR02" => set_word(&mut f.ram, ARGUMENT + 68, 0x200),
            "TR03" => set_word(&mut f.ram, ARGUMENT + 68, 0x7fff),
            "TR04" => {
                set_word(&mut f.ram, ARGUMENT + 68, 0x8000);
                f.expected = Expectation::Refused;
            }
            "TR05" => {
                set_word(&mut f.ram, ARGUMENT + 68, 0xffff);
                f.expected = Expectation::Refused;
            }
            "TR06" => {
                f.entry = Some(Entry::CollectAmmoPowerBombTank);
            }
            "TR07" => f.operand = Some(0xe127),
            "TR08" => f.rom[rom_index(0xe102)] = 6,
            "TR09" => f.rom[rom_index(0xe41f)] = 1,
            "TR10" => set_word(&mut f.ram, 0x1c2d, 8),
            "TR11" => set_word(&mut f.ram, 0x330, 602),
            "TR12" => f.rom[rom_index(0x87cd)] ^= 1,
            "TR13" => {
                f.entry = Some(Entry::GotoLinkIfTriggered);
                f.operand = None;
                set_word(&mut f.ram, LINK + 68, 0xe40f);
            }
            "TR14" => {
                f.entry = Some(Entry::GotoLinkIfTriggered);
                f.operand = None;
                set_word(&mut f.ram, IP + 68, 0xe0f4);
            }
            "TR15" => f.services_available = false,
            "TR16" => {
                set_word(&mut f.ram, HEADER + 68, 0xeee3);
                f.entry = Some(Entry::CollectAmmoPowerBombTank);
                f.operand = Some(0xe127);
                f.services_available = false;
            }
            "TR17" => set_word(&mut f.ram, 0x639, 3),
            "TR18" | "TR19" | "TR20" => {
                f.entry = Some(Entry::DrawItemFrame0);
                f.operand = Some(0xe437);
                match id {
                    "TR18" => f.services_available = false,
                    "TR19" => set_word(&mut f.ram, GFX + 68, 8),
                    _ => set_word(&mut f.ram, 0x7a5, 0),
                }
            }
            "TR21" => f.rom.truncate(rom_index(0xe107)),
            _ => unreachable!(),
        }
        f.raw_entry = f.entry.unwrap() as u32;
        cases.push(f);
    }
    Ok(cases)
}
