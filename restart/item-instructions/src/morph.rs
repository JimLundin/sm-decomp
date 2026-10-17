//! Morph-only admission and the four new scheduled bodies (Common is local).
use crate::{Entry, Output, Services, memory::*};

pub(crate) const DRAW: usize = 0xde6c;
pub(crate) const GFX: usize = 0xdf0c;
pub(crate) const BLOCK: usize = 0x1c87;

pub(crate) fn admits(entry: Entry, operand: Option<u16>) -> bool {
    match entry {
        Entry::Delete => operand == Some(0xdfaf),
        Entry::PreInstr => operand == Some(0xe405),
        Entry::Goto => matches!(operand, Some(0xe40d | 0xe41b)),
        Entry::GotoIfItemBitSet => operand == Some(0xe3fd),
        Entry::SetItemBit => operand == Some(0xe411),
        Entry::SetLinkReg => operand == Some(0xe401),
        Entry::ClearMusicQueueAndQueueTrack => operand == Some(0xe413),
        Entry::GotoLinkIfTriggered => operand.is_none(),
        Entry::LoadItemPlmGfx => operand == Some(0xe3f1),
        Entry::DrawItemFrame0 => operand == Some(0xe409),
        Entry::DrawItemFrame1 => operand == Some(0xe40b),
        Entry::PickupEquipmentAndShowMessage => operand == Some(0xe416),
        _ => false,
    }
}

pub(crate) fn validate_rom(rom: &[u8]) -> Result<(), String> {
    validate_seals(rom, true)
}

pub(crate) fn validate_graphics_rom(rom: &[u8]) -> Result<(), String> {
    validate_seals(rom, false)
}

fn validate_seals(rom: &[u8], include_morph_script: bool) -> Result<(), String> {
    // All script operands, referenced frame records, and the graphics tables
    // are sealed. Artwork remains an input of the prepared native ROM.
    const SEALS: &[(u16, &[u8])] = &[
        (
            0xe3ef,
            &[
                0x64, 0x87, 0x00, 0x87, 0, 0, 0, 0, 0, 0, 0, 0, 0x7c, 0x88, 0x19, 0xe4, 0x24, 0x8a, 0x0f, 0xe4, 0xc1,
                0x86, 0x89, 0xdf, 0x4f, 0xe0, 0x67, 0xe0, 0x24, 0x87, 0x07, 0xe4, 0x99, 0x88, 0xdd, 0x8b, 0x02, 0xf3,
                0x88, 0x04, 0x00, 0x09, 0x24, 0x87, 0xa9, 0xdf,
            ],
        ),
        (0xdfa9, &[1, 0, 0xb5, 0xa2, 0xbc, 0x86]),
        (
            0x87cd,
            &[
                0x00, 0x3e, 0x80, 0x3e, 0x00, 0x3f, 0x80, 0x3f, 0x70, 0x04, 0x80, 0x04, 0x90, 0x04, 0xa0, 0x04, 0xe0,
                0x03, 0xe8, 0x03, 0xf0, 0x03, 0xf8, 0x03,
            ],
        ),
        (0xe05f, &[0x0f, 0xa3, 0x1b, 0xa3, 0x27, 0xa3, 0x33, 0xa3]),
        (0xe077, &[0x15, 0xa3, 0x21, 0xa3, 0x2d, 0xa3, 0x39, 0xa3]),
        (
            0xa30f,
            &[
                1, 0, 0x8e, 0xb0, 0, 0, 1, 0, 0x8f, 0xb0, 0, 0, 1, 0, 0x90, 0xb0, 0, 0, 1, 0, 0x91, 0xb0, 0, 0, 1, 0,
                0x92, 0xb0, 0, 0, 1, 0, 0x93, 0xb0, 0, 0, 1, 0, 0x94, 0xb0, 0, 0, 1, 0, 0x95, 0xb0, 0, 0,
            ],
        ),
        (0xa2b5, &[1, 0, 0xff, 0, 0, 0]),
    ];
    for &(offset, expected) in &SEALS[usize::from(!include_morph_script)..] {
        let start = rom_index(offset);
        if rom.get(start..start + expected.len()) != Some(expected) {
            return Err(format!("Morph bank84 seal {offset:04x} differs or is truncated"));
        }
    }
    Ok(())
}

fn valid_slot(slot: u16) -> bool {
    slot <= 78 && slot & 1 == 0
}
fn valid_ring(value: u16) -> bool {
    value <= 6 && value & 1 == 0
}
fn valid_draw(pointer: u16) -> bool {
    matches!(pointer, 0xa30f | 0xa315 | 0xa31b | 0xa321 | 0xa327 | 0xa32d | 0xa333 | 0xa339)
}

fn check_draw_state(ram: &[u8], slot: u16, pointer_will_be_assigned: bool) -> Result<(), String> {
    if !valid_slot(slot) {
        return Err("unsafe Morph draw slot".into());
    }
    let k = usize::from(slot);
    let block = word(ram, BLOCK + k);
    if block & 1 != 0 || block > 0xfffc {
        return Err("unsafe Morph level-data word".into());
    }
    if !(1..=255).contains(&word(ram, 0x7a5)) {
        return Err("unsafe Morph room-width divisor".into());
    }
    if word(ram, 0x330) > 0x260 {
        return Err("unsafe Morph draw queue tail".into());
    }
    let cursor = word(ram, 0x1c25);
    if cursor > 512 || cursor & 7 != 0 {
        return Err("unsafe Morph tilemap cursor".into());
    }
    if !pointer_will_be_assigned && !valid_draw(word(ram, DRAW + k)) {
        return Err("unsealed Morph service draw pointer".into());
    }
    Ok(())
}

pub(crate) fn preflight(entry: Entry, slot: u16, ram: &[u8]) -> Result<(), String> {
    match entry {
        Entry::LoadItemPlmGfx => {
            if !valid_ring(word(ram, 0x1c2d)) {
                return Err("unsafe Morph graphics ring".into());
            }
            // Seven packed bytes must finish before queue-tail storage at0330.
            if word(ram, 0x330) > 0x260 - 7 {
                return Err("unsafe Morph graphics queue extent".into());
            }
        }
        Entry::DrawItemFrame0 | Entry::DrawItemFrame1 => {
            if !valid_ring(word(ram, GFX + usize::from(slot))) {
                return Err("unsafe Morph frame table index".into());
            }
            check_draw_state(ram, slot, true)?;
            let saved = word(ram, 0x1c27);
            check_draw_state(ram, saved, saved == slot)?;
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn load_gfx(slot: u16, operand: u16, ram: &mut [u8], rom: &[u8]) -> Output {
    let ring = word(ram, 0x1c2d);
    let v = usize::from(ring);
    set_word(ram, GFX + usize::from(slot), ring);
    set_word(ram, 0x1c2d, (ring + 2) & 6);
    set_word(ram, 0x1c2f + v, operand);
    let tail = word(ram, 0x330);
    let q = 0xd0 + usize::from(tail);
    set_word(ram, q, 256);
    set_word(ram, q + 2, word(rom, rom_index(operand)));
    ram[q + 4] = 0x89;
    set_word(ram, q + 5, word(rom, rom_index(0x87cd) + v));
    set_word(ram, 0x330, tail + 7);
    let tiles = usize::from(word(rom, rom_index(0x87d5) + v));
    let first = word(rom, rom_index(0x87dd) + v);
    for i in 0..8 {
        let palette = u16::from(rom[rom_index(operand) + 2 + i]);
        set_word(ram, 0xa000 + tiles + i * 2, first + i as u16 + (palette << 10));
    }
    Output::Continue(operand + 10)
}

pub(crate) fn draw_frame(
    entry: Entry,
    slot: u16,
    operand: u16,
    ram: &mut [u8],
    rom: &[u8],
    services: &mut dyn Services,
) -> Result<Output, String> {
    let table = if entry == Entry::DrawItemFrame0 { 0xe05f } else { 0xe077 };
    let ring = usize::from(word(ram, GFX + usize::from(slot)));
    set_word(ram, DRAW + usize::from(slot), word(rom, rom_index(table) + ring));
    draw_common(slot, operand, ram, services)
}

fn draw_common(slot: u16, operand: u16, ram: &mut [u8], services: &mut dyn Services) -> Result<Output, String> {
    set_word(ram, INSTRUCTION_TIMER + usize::from(slot), 4);
    set_word(ram, IP + usize::from(slot), operand);
    services.process_plm_draw_instruction(ram, slot)?;
    let saved = word(ram, 0x1c27);
    check_draw_state(ram, saved, false)?;
    services.calculate_plm_block_coords(ram, saved)?;
    // A service failure or unsafe service mutation is fatal after effects.
    check_draw_state(ram, saved, false)?;
    services.draw_plm(ram, saved)?;
    Ok(Output::Stop)
}
