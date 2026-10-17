//! Finite ticket19 delta; uses the existing sealed seed with explicit overlays.
use super::{Expectation, Fixture, Write, base, validate_inputs};
use crate::{
    Entry, Output, Service,
    memory::*,
    morph::{BLOCK, DRAW, GFX},
};

fn case(seed: &[u8], rom: &[u8], id: &'static str, entry: Entry, operand: Option<u16>, output: Output) -> Fixture {
    let mut ram = base(seed, 68);
    for (at, value) in [
        (HEADER + 68, 0xef23),
        (ARGUMENT + 68, 0x1a),
        (LINK + 68, 0xe40f),
        (IP + 68, 0xe409),
        (BLOCK + 68, 0x298a),
        (DRAW + 68, 0xa30f),
        (GFX + 68, 0),
        (0x1c2d, 0),
        (0x330, 0),
        (0x1c25, 0),
        (0x7a5, 128),
        (0x911, 1024),
        (0x915, 640),
        (0x91d, 0),
        (0x9a2, 0x40),
        (0x9a4, 0x200),
    ] {
        set_word(&mut ram, at, value);
    }
    ram[0xd873] = 0xa9;
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

/// M01..M31 share exact inputs with native actual-C comparison. Direct writes
/// omit native renderer/music effects; full native snapshots remain the oracle.
pub fn morph_body_fixtures(seed: &[u8], rom: &[u8]) -> Result<Vec<Fixture>, String> {
    validate_inputs(seed, rom)?;
    crate::morph::validate_rom(rom)?;
    use Entry::*;
    use Output::*;
    use Write::*;
    let specs = [
        ("M01", LoadItemPlmGfx, Some(0xe3f1), Continue(0xe3fb)),
        ("M02", LoadItemPlmGfx, Some(0xe3f1), Continue(0xe3fb)),
        ("M03", LoadItemPlmGfx, Some(0xe3f1), Continue(0xe3fb)),
        ("M04", LoadItemPlmGfx, Some(0xe3f1), Continue(0xe3fb)),
        ("M05", DrawItemFrame0, Some(0xe409), Stop),
        ("M06", DrawItemFrame1, Some(0xe40b), Stop),
        ("M07", DrawItemFrame0, Some(0xe409), Stop),
        ("M08", DrawItemFrame1, Some(0xe40b), Stop),
        ("M09", DrawItemFrame0, Some(0xe409), Stop),
        ("M10", DrawItemFrame1, Some(0xe40b), Stop),
        ("M11", DrawItemFrame0, Some(0xe409), Stop),
        ("M12", DrawItemFrame1, Some(0xe40b), Stop),
        ("M13", DrawItemFrame0, Some(0xe409), Stop),
        ("M14", DrawItemFrame1, Some(0xe40b), Stop),
        ("M15", DrawItemFrame0, Some(0xe409), Stop),
        ("M16", PickupEquipmentAndShowMessage, Some(0xe416), Continue(0xe419)),
        ("M17", PickupEquipmentAndShowMessage, Some(0xe416), Continue(0xe419)),
        ("M18", PickupEquipmentAndShowMessage, Some(0xe416), Continue(0xe419)),
        ("M19", GotoIfItemBitSet, Some(0xe3fd), Continue(0xe3ff)),
        ("M20", GotoIfItemBitSet, Some(0xe3fd), Continue(0xe419)),
        ("M21", SetItemBit, Some(0xe411), Continue(0xe411)),
        ("M22", SetItemBit, Some(0xe411), Continue(0xe411)),
        ("M23", SetLinkReg, Some(0xe401), Continue(0xe403)),
        ("M24", PreInstr, Some(0xe405), Continue(0xe407)),
        ("M25", Goto, Some(0xe40d), Continue(0xe407)),
        ("M26", Goto, Some(0xe41b), Continue(0xdfa9)),
        ("M27", Delete, Some(0xdfaf), Stop),
        ("M28", ClearMusicQueueAndQueueTrack, Some(0xe413), Continue(0xe414)),
        ("M29", GotoLinkIfTriggered, None, PreinstructionDone),
        ("M30", GotoLinkIfTriggered, None, PreinstructionDone),
        ("M31", DrawItemFrame0, Some(0xe409), Stop),
    ];
    let mut cases = Vec::with_capacity(specs.len());
    for (index, (id, entry, operand, output)) in specs.into_iter().enumerate() {
        let mut f = case(seed, rom, id, entry, operand, output);
        match entry {
            LoadItemPlmGfx => {
                let ring = index as u16 * 2;
                let tail = [0, 7, 14, 601][index];
                set_word(&mut f.ram, 0x1c2d, ring);
                set_word(&mut f.ram, 0x330, tail);
                set_word(&mut f.ram, GFX + 68, 0xa55a);
                let q = 0xd0 + usize::from(tail);
                f.direct_writes = vec![
                    Word(GFX + 68, ring),
                    Word(0x1c2d, (ring + 2) & 6),
                    Word(0x1c2f + usize::from(ring), 0xe3f1),
                    Word(q, 256),
                    Word(q + 2, 0x8700),
                    Byte(q + 4, 0x89),
                    Word(q + 5, 0x3e00 + index as u16 * 128),
                    Word(0x330, tail + 7),
                ];
                for i in 0..8 {
                    f.direct_writes.push(Word(0xa470 + index * 16 + i * 2, 0x3e0 + index as u16 * 8 + i as u16));
                }
            }
            DrawItemFrame0 | DrawItemFrame1 => {
                let ring = if (4..12).contains(&index) { ((index - 4) / 2 * 2) as u16 } else { 0 };
                set_word(&mut f.ram, GFX + 68, ring);
                set_word(&mut f.ram, DRAW + 68, 0xa55a);
                let frame = usize::from(entry == DrawItemFrame1);
                let pointer = 0xa30f + ring * 6 + frame as u16 * 6;
                f.direct_writes =
                    vec![Word(DRAW + 68, pointer), Word(INSTRUCTION_TIMER + 68, 4), Word(IP + 68, operand.unwrap())];
                let saved = if id == "M31" { 70 } else { 68 };
                f.service_calls = vec![
                    (Service::ProcessPlmDrawInstruction, 68),
                    (Service::CalculatePlmBlockCoords, saved),
                    (Service::DrawPlm, saved),
                ];
                match id {
                    "M13" => set_word(&mut f.ram, 0x915, 0),
                    "M14" => set_word(&mut f.ram, 0x330, 480),
                    "M15" => set_word(&mut f.ram, 0x1c25, 512),
                    "M31" => {
                        set_word(&mut f.ram, 0x1c27, saved);
                        set_word(&mut f.ram, BLOCK + 70, 0x298a);
                        set_word(&mut f.ram, DRAW + 70, 0xa315);
                    }
                    _ => {}
                }
            }
            PickupEquipmentAndShowMessage => {
                if id == "M17" {
                    set_word(&mut f.ram, 0x9a2, 0x44);
                    set_word(&mut f.ram, 0x9a4, 0x204);
                }
                if id == "M18" {
                    set_word(&mut f.ram, 0x998, 40);
                }
                f.direct_writes = vec![Word(0x9a2, 0x44), Word(0x9a4, 0x204), Word(0xdc8, 9)];
                f.service_calls = vec![(Service::PlayRoomMusicAfter, 0x168)];
            }
            GotoIfItemBitSet | SetItemBit => {
                if matches!(id, "M20" | "M22") {
                    f.ram[0xd873] = 0xad;
                }
                f.direct_writes.push(Word(0x5e7, 4));
                if entry == SetItemBit {
                    f.direct_writes.push(Byte(0xd873, 0xad));
                }
            }
            SetLinkReg => {
                set_word(&mut f.ram, LINK + 68, 0xa55a);
                f.direct_writes.push(Word(LINK + 68, 0xe40f));
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
                set_word(&mut f.ram, TIMER + 68, if id == "M29" { 0xabff } else { 0xff00 });
                if id == "M29" {
                    f.direct_writes =
                        vec![Word(PRE + 68, 0xdfa8), Word(IP + 68, 0xe40f), Word(INSTRUCTION_TIMER + 68, 1)];
                } else {
                    set_word(&mut f.ram, IP + 68, 0xe40b);
                }
            }
            _ => {}
        }
        cases.push(f);
    }
    Ok(cases)
}

/// Changed admission/safety boundaries only. Unsafe cases never execute C.
pub fn morph_refusal_fixtures(seed: &[u8], rom: &[u8]) -> Result<Vec<Fixture>, String> {
    validate_inputs(seed, rom)?;
    crate::morph::validate_rom(rom)?;
    let mut cases = Vec::new();
    for id in [
        "MR01", "MR02", "MR03", "MR04", "MR05", "MR06", "MR07", "MR08", "MR09", "MR10", "MR11", "MR12", "MR13", "MR14",
        "MR15", "MR16", "MR17", "MR18", "MR19", "MR20",
    ] {
        let mut f = case(seed, rom, id, Entry::DrawItemFrame0, Some(0xe409), Output::Stop);
        f.expected = Expectation::Error;
        if matches!(id, "MR03" | "MR06" | "MR08") {
            f.entry = Some(Entry::LoadItemPlmGfx);
            f.operand = Some(0xe3f1);
        }
        match id {
            "MR01" => {
                set_word(&mut f.ram, HEADER + 68, 0xef24);
                f.expected = Expectation::Refused;
            }
            "MR02" => {
                set_word(&mut f.ram, ARGUMENT + 68, 0x1b);
                f.expected = Expectation::Refused;
            }
            "MR03" => f.operand = Some(0xe3f2),
            "MR04" => f.rom[rom_index(0xe416)] = 5,
            "MR05" => f.rom[rom_index(0xe05f)] ^= 1,
            "MR06" => set_word(&mut f.ram, 0x1c2d, 8),
            "MR07" => set_word(&mut f.ram, GFX + 68, 1),
            "MR08" => set_word(&mut f.ram, 0x330, 602),
            "MR09" => f.services_available = false,
            "MR10" => set_word(&mut f.ram, BLOCK + 68, 1),
            "MR11" => set_word(&mut f.ram, BLOCK + 68, 0xfffe),
            "MR12" => set_word(&mut f.ram, 0x7a5, 0),
            "MR13" => set_word(&mut f.ram, 0x1c27, 80),
            "MR14" => set_word(&mut f.ram, 0x330, 609),
            "MR15" => set_word(&mut f.ram, 0x1c25, 520),
            "MR16" => {
                set_word(&mut f.ram, 0x1c27, 70);
                set_word(&mut f.ram, BLOCK + 70, 0x298a);
                set_word(&mut f.ram, DRAW + 70, 0x1234);
            }
            "MR17" => set_word(&mut f.ram, 0x7a5, 256),
            "MR18" => set_word(&mut f.ram, 0x1c25, 4),
            "MR19" => f.rom.truncate(rom_index(0xe41c)),
            "MR20" => {
                f.entry = Some(Entry::GotoLinkIfTriggered);
                f.operand = None;
                set_word(&mut f.ram, LINK + 68, 0xe49f);
            }
            _ => unreachable!(),
        }
        f.raw_entry = f.entry.unwrap() as u32;
        cases.push(f);
    }
    Ok(cases)
}
