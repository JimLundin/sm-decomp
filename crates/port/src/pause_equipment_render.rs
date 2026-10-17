//! Pause equipment output, translated from pinned snesrev/sm 81891F,
//! 82A881 and 82B1E0..82B3F9. Each public call preflights its selected
//! ROM records and canonical output bounds before changing WRAM.
use crate::Wram;

const OAM: usize = 0x370;
const OAM_EXT: usize = 0x570;
const OAM_NEXT: usize = 0x590;
const RESERVE_TIMER: usize = 0x72f;
const RESERVE_FRAME: usize = 0x743;
const CATEGORY_ITEM: usize = 0x755;
const RESERVE_MAX: usize = 0x9d4;
const RESERVE_HEALTH: usize = 0x9d6;
const TILEMAP: usize = 0x3800;
const QUEUE: usize = 0xd0;
const QUEUE_TAIL: usize = 0x330;

fn fixed(address: u16) -> usize {
    0x10000 + usize::from(address & 0x7fff)
}

fn rom_word(rom: &[u8], offset: usize) -> Result<u16, &'static str> {
    let bytes = rom.get(offset..offset + 2).ok_or("pause equipment ROM word extent")?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn table_word(rom: &[u8], table: u16, index: usize) -> Result<u16, &'static str> {
    rom_word(rom, fixed(table) + 2 * index)
}

fn banked(pointer: u16) -> Result<usize, &'static str> {
    if pointer < 0x8000 {
        return Err("pause equipment pointer below banked ROM");
    }
    Ok(fixed(pointer))
}

fn oam_cursor(r: &Wram<'_>) -> Result<(), &'static str> {
    if r.word(OAM_NEXT) >= 0x200 {
        return Err("pause equipment OAM cursor outside wrapping buffer");
    }
    Ok(())
}

struct Spritemap<'a> {
    entries: &'a [u8],
    x: u16,
    y: u16,
    attributes: u16,
}

fn spritemap(rom: &[u8], id: u16, x: u16, y: u16, attributes: u16) -> Result<Spritemap<'_>, &'static str> {
    let pointer = table_word(rom, 0xc569, usize::from(id))?;
    let offset = banked(pointer)?;
    let count = usize::from(rom_word(rom, offset)?);
    let entries = rom.get(offset + 2..offset + 2 + count * 5).ok_or("pause equipment spritemap extent")?;
    Ok(Spritemap { entries, x, y, attributes })
}

fn emit(r: &mut Wram<'_>, map: &Spritemap<'_>) {
    let mut index = usize::from(r.word(OAM_NEXT));
    for entry in map.entries.chunks_exact(5) {
        let dx = u16::from_le_bytes([entry[0], entry[1]]);
        let x = map.x.wrapping_add(dx);
        // Preserve C's write order, including unaligned cursor aliases into
        // the extension words. Menu sprites have no clipping step.
        r.set_byte(OAM + index, x as u8);
        let extension = ((x & 0x100) >> 8) | (u16::from(dx & 0x8000 != 0) * 2);
        let address = OAM_EXT + (index >> 5) * 2;
        let shift = 2 * ((index >> 2) & 7);
        r.set_word(address, r.word(address) | (extension << shift));
        r.set_byte(OAM + index + 1, map.y.wrapping_add(u16::from(entry[2])) as u8);
        let tile = u16::from_le_bytes([entry[3], entry[4]]);
        r.set_word(OAM + index + 2, map.attributes | (tile & 0xf1ff));
        index = (index + 4) & 0x1ff;
    }
    r.set_word(OAM_NEXT, index as u16);
}

/// Emit the selected bank-82 menu spritemap, wrapping after every OAM entry.
pub fn draw_menu_spritemap(
    r: &mut Wram<'_>,
    rom: &[u8],
    id: u16,
    x: u16,
    y: u16,
    attributes: u16,
) -> Result<(), &'static str> {
    oam_cursor(r)?;
    let map = spritemap(rom, id, x, y, attributes)?;
    emit(r, &map);
    Ok(())
}

/// Read a delay/metadata stream through its 0xff terminator. The terminator
/// is not an admissible retained frame; it resets an advancing animation.
fn animation(rom: &[u8], pointer: u16, stride: usize) -> Result<&[u8], &'static str> {
    let offset = banked(pointer)?;
    let bank_end = fixed(0x8000) + 0x8000;
    let bytes = rom.get(offset..bank_end).ok_or("pause equipment animation extent")?;
    for (index, frame) in bytes.chunks_exact(stride).enumerate() {
        if frame[0] == 0xff {
            if index == 0 {
                return Err("pause equipment empty animation");
            }
            return Ok(&bytes[..index * stride]);
        }
    }
    Err("pause equipment unterminated animation")
}

struct SpriteAnimation<'a> {
    timer_address: usize,
    frame_address: usize,
    timer: u16,
    frame: u16,
    map: Spritemap<'a>,
}

fn sprite_animation<'a>(
    r: &Wram<'_>,
    rom: &'a [u8],
    kind: u16,
    x: u16,
    y: u16,
) -> Result<SpriteAnimation<'a>, &'static str> {
    let variants = match kind {
        2 => 3,     // L/R labels
        3 => 4,     // Equipment categories
        6..=9 => 1, // Map arrows
        _ => return Err("pause equipment unused sprite animation kind"),
    };
    let index = usize::from(kind - 1);
    let timer_address = usize::from(table_word(rom, 0xc0b2, index)?);
    let frame_address = usize::from(table_word(rom, 0xc0c4, index)?);
    let variant_address = usize::from(table_word(rom, 0xc0d6, index)?);
    if timer_address + 2 > 0x2000 || frame_address + 2 > 0x2000 || variant_address >= 0x2000 {
        return Err("pause equipment animation WRAM pointer extent");
    }
    let variant = usize::from(r.byte(variant_address));
    if variant >= variants {
        return Err("pause equipment sprite animation variant");
    }
    let frames = animation(rom, table_word(rom, 0xc0e8, index)?, 3)?;
    let mut frame = r.word(frame_address);
    if usize::from(frame) >= frames.len() / 3 {
        return Err("pause equipment sprite animation frame");
    }
    let mut timer = r.word(timer_address).wrapping_sub(1);
    if timer as i16 <= 0 {
        frame += 1;
        if usize::from(frame) == frames.len() / 3 {
            frame = 0;
        }
        timer = u16::from(frames[3 * usize::from(frame)]);
    }
    // The C dereference of RomPtr_RAM here is a BYTE. The high equipment
    // item row must not affect the category's spritemap base selection.
    let base_pointer = table_word(rom, 0xc1e4, index)?.wrapping_add((variant * 2) as u16);
    let base_id = rom_word(rom, banked(base_pointer)?)?;
    let id = base_id.wrapping_add(u16::from(frames[3 * usize::from(frame) + 2]));
    let attributes = table_word(rom, 0xc0fa, 3)?;
    oam_cursor(r)?;
    let map = spritemap(rom, id, x, y.wrapping_sub(1), attributes)?;
    Ok(SpriteAnimation { timer_address, frame_address, timer, frame, map })
}

fn emit_animation(r: &mut Wram<'_>, plan: &SpriteAnimation<'_>) {
    r.set_word(plan.timer_address, plan.timer);
    r.set_word(plan.frame_address, plan.frame);
    emit(r, &plan.map);
}

/// Advance and draw one of the used pause label/selector/arrow animations.
pub fn draw_pause_screen_sprite_anim(
    r: &mut Wram<'_>,
    rom: &[u8],
    kind: u16,
    x: u16,
    y: u16,
) -> Result<(), &'static str> {
    let plan = sprite_animation(r, rom, kind, x, y)?;
    emit_animation(r, &plan);
    Ok(())
}

/// Draw the selector when any reserve capacity, collected item or beam exists.
pub fn equipment_screen_draw_item_selector(r: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    if r.word(RESERVE_MAX) | r.word(0x9a4) | r.word(0x9a8) == 0 {
        return Ok(());
    }
    let selection = r.word(CATEGORY_ITEM);
    let category = usize::from(selection as u8);
    let row = usize::from(selection >> 8);
    let rows = [2, 5, 6, 3];
    if category >= rows.len() || row >= rows[category] {
        return Err("pause equipment selector category/item");
    }
    let pointer = table_word(rom, 0xc18e, category)?.wrapping_add((row * 4) as u16);
    let offset = banked(pointer)?;
    let x = rom_word(rom, offset)?.wrapping_sub(1);
    let y = rom_word(rom, offset + 2)?;
    draw_pause_screen_sprite_anim(r, rom, 3, x, y)
}

struct ReservePalette {
    timer: u16,
    frame: u16,
    attributes: u16,
    animated: bool,
}

fn reserve_palette(r: &Wram<'_>, rom: &[u8]) -> Result<ReservePalette, &'static str> {
    if r.word(RESERVE_HEALTH) == 0 {
        return Ok(ReservePalette { timer: 0, frame: 0, attributes: 0x600, animated: false });
    }
    let frames = animation(rom, 0xc165, 2)?;
    let mut frame = r.word(RESERVE_FRAME);
    if usize::from(frame) >= frames.len() / 2 {
        return Err("pause equipment reserve palette frame");
    }
    let mut timer = r.word(RESERVE_TIMER).wrapping_sub(1);
    if timer == 0 || timer & 0x8000 != 0 {
        frame += 1;
        if usize::from(frame) == frames.len() / 2 {
            frame = 0;
        }
        timer = u16::from(frames[2 * usize::from(frame)]);
    }
    Ok(ReservePalette { timer, frame, attributes: table_word(rom, 0xc0fa, 3)?, animated: true })
}

fn emit_palette(r: &mut Wram<'_>, palette: &ReservePalette) {
    if palette.animated {
        r.set_word(RESERVE_TIMER, palette.timer);
        r.set_word(RESERVE_FRAME, palette.frame);
    }
}

/// Advance the reserve animation when nonempty and return its sprite palette.
pub fn equipment_screen_display_reserves_palette_setup(r: &mut Wram<'_>, rom: &[u8]) -> Result<u16, &'static str> {
    let palette = reserve_palette(r, rom)?;
    emit_palette(r, &palette);
    Ok(palette.attributes)
}

/// Draw whole, partial and empty tanks, the end cap, and the three reserve
/// digits. The six X positions admit at most five tanks plus their end cap.
pub fn equipment_screen_display_reserve_tank_amount_(r: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    let maximum = r.word(RESERVE_MAX);
    let health = r.word(RESERVE_HEALTH);
    if maximum > 500 || health > maximum {
        return Err("pause equipment reserve capacity/amount");
    }
    let palette = reserve_palette(r, rom)?;
    if maximum == 0 {
        emit_palette(r, &palette);
        return Ok(());
    }
    oam_cursor(r)?;
    let full = health / 100;
    let remainder = health % 100;
    let capacity = maximum / 100;
    let mut maps = Vec::with_capacity(6);
    let x = |index: usize| 0x18 + index as u16 * 8;
    for index in 0..usize::from(full) {
        maps.push(spritemap(rom, 0x1b, x(index), 0x5f, palette.attributes)?);
    }
    if remainder != 0 {
        let mut step: u16 = 2 * (remainder / 14);
        if (step.wrapping_sub(7) as i16) < 0 && remainder % 14 != 0 && r.byte(0x5b5) & 4 == 0 {
            step += 2;
        }
        if (health.wrapping_sub(100) as i16) >= 0 {
            step += 16;
        }
        let ids = [0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27];
        maps.push(spritemap(rom, ids[usize::from(step / 2)], x(maps.len()), 0x5f, palette.attributes)?);
    }
    while maps.len() < usize::from(capacity) {
        maps.push(spritemap(rom, 0x20, x(maps.len()), 0x5f, palette.attributes)?);
    }
    maps.push(spritemap(rom, 0x1f, x(maps.len()), 0x5f, palette.attributes)?);
    emit_palette(r, &palette);
    for map in &maps {
        emit(r, map);
    }
    r.set_word(TILEMAP + 394 * 2, remainder % 10 + 2052);
    r.set_word(TILEMAP + 393 * 2, remainder / 10 + 2052);
    r.set_word(TILEMAP + 392 * 2, full + 2052);
    Ok(())
}

/// The bank-82 public reserve amount entry delegates to the same renderer.
pub fn equipment_screen_display_reserve_tank_amount(r: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    equipment_screen_display_reserve_tank_amount_(r, rom)
}

fn wireframe<'a>(r: &Wram<'_>, rom: &'a [u8]) -> Result<&'a [u8], &'static str> {
    let equipped = r.word(0x9a2) & 0x101;
    for index in 0..4 {
        if table_word(rom, 0xb257, index)? == equipped {
            let pointer = table_word(rom, 0xb25f, index)?;
            let offset = banked(pointer)?;
            return rom.get(offset..offset + 17 * 8 * 2).ok_or("pause equipment wireframe extent");
        }
    }
    Err("pause equipment wireframe equipment selection")
}

fn emit_wireframe(r: &mut Wram<'_>, tiles: &[u8]) {
    for (row, words) in tiles.chunks_exact(16).enumerate() {
        for (column, word) in words.chunks_exact(2).enumerate() {
            r.set_word(TILEMAP + 472 + row * 64 + column * 2, u16::from_le_bytes([word[0], word[1]]));
        }
    }
}

/// Select the suit/wireframe using only equipped item bits 0 and 8.
pub fn write_samus_wireframe_tilemap(r: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    let tiles = wireframe(r, rom)?;
    emit_wireframe(r, tiles);
    Ok(())
}

/// Append the exact packed seven-byte bank-7E VRAM write after drawing.
pub fn write_samus_wireframe_tilemap_and_queue(r: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    let tail = usize::from(r.word(QUEUE_TAIL));
    if tail + 7 > QUEUE_TAIL - QUEUE {
        return Err("pause equipment VRAM queue extent");
    }
    let tiles = wireframe(r, rom)?;
    emit_wireframe(r, tiles);
    r.set_word(QUEUE + tail, 1280);
    r.set_word(QUEUE + tail + 2, 0x3900);
    r.set_byte(QUEUE + tail + 4, 0x7e);
    r.set_word(QUEUE + tail + 5, 0x3080);
    r.set_word(QUEUE_TAIL, (tail + 7) as u16);
    Ok(())
}

/// Family ABI packs DrawMenuSpritemap's four words into a/b. The direct
/// three-word animation API packs kind/X into a and Y into b; its C oracle
/// gate must use the real three-argument prototype.
pub fn dispatch_rom(r: &mut Wram<'_>, rom: &[u8], symbol: &str, a: i32, b: i32) -> Result<u32, &'static str> {
    match symbol {
        "DrawMenuSpritemap" => {
            draw_menu_spritemap(r, rom, a as u16, (a as u32 >> 16) as u16, b as u16, (b as u32 >> 16) as u16)?
        }
        "DrawPauseScreenSpriteAnim" => {
            draw_pause_screen_sprite_anim(r, rom, a as u16, (a as u32 >> 16) as u16, b as u16)?
        }
        "EquipmentScreenDrawItemSelector" => equipment_screen_draw_item_selector(r, rom)?,
        "EquipmentScreenDisplayReserveTankAmount" => equipment_screen_display_reserve_tank_amount(r, rom)?,
        "EquipmentScreenDisplayReserveTankAmount_" => equipment_screen_display_reserve_tank_amount_(r, rom)?,
        "EquipmentScreenDisplayReserves_PaletteSetup" => {
            return equipment_screen_display_reserves_palette_setup(r, rom).map(u32::from);
        }
        "WriteSamusWireframeTilemap" => write_samus_wireframe_tilemap(r, rom)?,
        "WriteSamusWireframeTilemapAndQueue" => write_samus_wireframe_tilemap_and_queue(r, rom)?,
        _ => return Err("unknown pause equipment rendering routine"),
    }
    Ok(0)
}
