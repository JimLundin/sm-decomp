//! HUD icons, resource counters and their ordered tilemap upload.
use crate::display::registers::{MDMAEN, VMADDL, VMAIN};
use crate::display::transfer::setup_transfer;
use crate::menus::menu_tilemap::native_record;

/// The Session-owned player and inventory values the HUD shows. `sm` reads the
/// compat words for the game; a Session lease reads its owned fields.
pub trait HudPlayer {
    /// $09C2 and $09C4.
    fn health(&self) -> u16;
    fn max_health(&self) -> u16;
    /// $09C6/$09CA/$09CE and $09C8/$09CC/$09D0: missiles, supers, power bombs.
    fn ammunition(&self, index: usize) -> u16;
    fn max_ammunition(&self, index: usize) -> u16;
    /// $09D6 and $09C0.
    fn reserve_health(&self) -> u16;
    fn reserve_mode(&self) -> u16;
    /// $09A2.
    fn equipped_items(&self) -> u16;
    /// $09D2 and $0A04: the selected and automatically cancelled HUD items.
    fn selected_item(&self) -> u16;
    fn automatic_item(&self) -> u16;
    /// $0D32.
    fn grapple_function(&self) -> u16;
}

/// What the HUD's frame step needs: RAM words, its player values, sound
/// effects and the native cartridge. A Session lease can provide it.
pub trait HudFrameHost:
    HudPlayer
    + crate::ram::CompatibilityRam
    + crate::player_position::PlayerFacts
    + crate::effects::SoundEffects
    + crate::cartridge::NativeDiagnostics
{
    /// The raw work-RAM NMI frame counter byte, as the item blink reads it.
    fn raw_frame_counter_byte(&self) -> u8;
}

/// What the whole HUD reads and writes: the frame step's host, plus raw RAM
/// and the registers its initial upload uses.
pub trait HudHost:
    HudFrameHost + crate::ram::RamHost + crate::display::sequence::Registers
{
}
impl<T> HudHost for T where
    T: HudFrameHost + crate::ram::RamHost + crate::display::sequence::Registers + ?Sized
{
}
use crate::ram::{checked_word as at, word, words as fields};

/// $80:9DD3: the ammunition digit tiles, 0 to 9.
const WEAPON_DIGITS: u32 = 0x80_9dd3;
/// $80:9DBF: the energy digit tiles, 0 to 9.
const HEALTH_DIGITS: u32 = 0x80_9dbf;
/// $80:998B: the AUTO reserve tiles; the second set, six words on, is empty.
const RESERVE_TILES: u32 = 0x80_998b;
/// $80:988B: the HUD's top-row tiles, uploaded once at initialisation.
const TOP_ROW_TILES: u32 = 0x80_988b;
const TOP_ROW_BYTES: u16 = 0x40;
/// Item icon tilemap offsets (bytes), from missiles to X-ray.
const ITEM_OFFSETS: [u16; 5] = [0x14, 0x1c, 0x22, 0x28, 0x2e];

/// The HUD tilemap in work RAM ($7E:C608), its VRAM home and size.
const TILEMAP_RAM: usize = 0xc608;
const TILEMAP_VRAM: u16 = 0x5800;
const TILEMAP_BYTES: u16 = 192;
/// The VRAM word address of the HUD's second row, where the queued copy lands.
const TILEMAP_QUEUED_VRAM: u16 = TILEMAP_VRAM + 0x20;
const WORK_RAM_BANK: u16 = 0x7e;
/// VMAIN: increment after writing the high byte.
const VRAM_INCREMENT_AFTER_HIGH: u16 = 0x80;

/// BG3 tilemap entry bits.
const PRIORITY: u16 = 0x2000;
const H_FLIP: u16 = 0x4000;
const V_FLIP: u16 = 0x8000;
const PALETTE_BITS: u16 = 7 << 10;

/// A high-priority HUD tilemap entry for `number` in `palette`.
const fn hud_tile(number: u16, palette: u16) -> u16 {
    PRIORITY | palette << 10 | number
}
const fn palette(palette: u16) -> u16 {
    palette << 10
}

const BLANK_TILE: u16 = hud_tile(0x0f, 3);
const FULL_TANK_TILE: u16 = hud_tile(0x31, 2);
const EMPTY_TANK_TILE: u16 = hud_tile(0x30, 5);
/// Palette bits of the selected and unselected item icons.
const SELECTED_PALETTE: u16 = palette(4);
const UNSELECTED_PALETTE: u16 = palette(5);
/// The missile icon when no native definition is read: two rows of three.
const MISSILE_ICON: [u16; 6] = [
    hud_tile(0x4b, 5),
    hud_tile(0x49, 5),
    H_FLIP | hud_tile(0x4b, 5),
    hud_tile(0x4c, 5),
    hud_tile(0x4a, 5),
    H_FLIP | hud_tile(0x4c, 5),
];
/// Equipped item bits that install an icon at initialisation.
const XRAY_SCOPE: u16 = 0x8000;
const GRAPPLING_BEAM: u16 = 0x4000;
/// Samus movement types during which switching items stays silent.
const SPIN_JUMPING: u8 = 3;
const WALL_JUMPING: u8 = 20;
/// The grapple beam function while the beam is inactive.
const GRAPPLE_INACTIVE: u16 = 0xc4f0;

const _: () = {
    assert!(BLANK_TILE == 0x2c0f);
    assert!(FULL_TANK_TILE == 10289 && EMPTY_TANK_TILE == 13360);
    assert!(SELECTED_PALETTE == 0x1000 && UNSELECTED_PALETTE == 0x1400);
    assert!(!PALETTE_BITS == 0xe3ff);
    let original = [0x344b, 0x3449, 0x744b, 0x344c, 0x344a, 0x744c];
    let mut i = 0;
    while i < 6 {
        assert!(MISSILE_ICON[i] == original[i]);
        i += 1;
    }
    assert!(WEAPON_DIGITS == 8429011 && HEALTH_DIGITS == 8428991);
    assert!(TILEMAP_QUEUED_VRAM == 0x5820);
};

#[derive(Clone, Copy)]
pub enum Item {
    Missiles,
    SuperMissiles,
    PowerBombs,
    Grapple,
    Xray,
}

/// The HUD tilemap at initialisation: blank, apart from the three rows of
/// fixed frame and label tiles.
static INITIAL_TILES: [u16; 96] = initial_tiles();

const fn initial_tiles() -> [u16; 96] {
    let mut tiles = [BLANK_TILE; 96];
    tiles[26] = hud_tile(0x12, 3);
    tiles[27] = hud_tile(0x12, 3);
    tiles[28] = hud_tile(0x23, 3);
    tiles[29] = hud_tile(0x12, 3);
    tiles[30] = hud_tile(0x12, 3);
    tiles[31] = hud_tile(0x1e, 3);
    tiles[58] = hud_tile(0x22, 2);
    tiles[59] = hud_tile(0x22, 2);
    tiles[60] = hud_tile(0x23, 2);
    tiles[61] = hud_tile(0x13, 2);
    tiles[62] = hud_tile(0x14, 3);
    tiles[63] = hud_tile(0x1e, 3);
    tiles[65] = hud_tile(0x0b, 3);
    tiles[66] = hud_tile(0x0c, 3);
    tiles[67] = hud_tile(0x0d, 3);
    tiles[68] = hud_tile(0x32, 3);
    tiles[70] = hud_tile(0x09, 3);
    tiles[90] = hud_tile(0x12, 3);
    tiles[91] = hud_tile(0x12, 3);
    tiles[92] = V_FLIP | hud_tile(0x24, 2);
    tiles[93] = hud_tile(0x15, 2);
    tiles[94] = hud_tile(0x16, 3);
    tiles[95] = hud_tile(0x1e, 3);
    tiles
}

fn stamp<H: crate::ram::CompatibilityRam + ?Sized>(
    base: usize,
    columns: usize,
    count: usize,
    tile: impl Fn(usize) -> u16,
    game: &mut H,
) {
    for index in 0..count {
        word!(game.HUD_TILEMAP[base + 32 * (index / columns) + index % columns] = tile(index));
    }
}

/// Native icon reads retain their address-formation contract, even if already installed.
pub fn install<H: HudHost + ?Sized>(item: Item, game: &mut H) {
    let index = item as usize;
    let base = usize::from(ITEM_OFFSETS[index] >> 1);
    let native = (index != 0).then(|| native_record(0x8099a3 + 4 + 8 * index as u32, game));
    if word!(game.HUD_TILEMAP[base]) & 0x3ff == BLANK_TILE & 0x3ff {
        let columns = if index == 0 { 3 } else { 2 };
        stamp(
            base,
            columns,
            2 * columns,
            |i| match &native {
                Some(record) => record.word(2 * i),
                None => MISSILE_ICON[i],
            },
            game,
        );
    }
}

/// Requires initialized display/DMA hardware and the native HUD definitions.
pub fn initialize<H: HudHost + ?Sized>(game: &mut H) {
    crate::display::sequence::register_writes! { game;
        word(VMADDL as u16, TILEMAP_VRAM); word(VMAIN as u16, VRAM_INCREMENT_AFTER_HIGH);
    }
    setup_transfer(
        u16::from_ne_bytes([1, 0x18]),
        TOP_ROW_TILES,
        TOP_ROW_BYTES,
        game,
    );
    game.write_register(MDMAEN as u16, 2);
    crate::word_transfers!(game, ascending, 96; le(HUD_TILEMAP, 0) <- table(INITIAL_TILES, 0));
    for (item, mask) in [(Item::Xray, XRAY_SCOPE), (Item::Grapple, GRAPPLING_BEAM)] {
        if game.equipped_items() & mask != 0 {
            install(item, game);
        }
    }
    for (index, item) in [Item::Missiles, Item::SuperMissiles, Item::PowerBombs]
        .into_iter()
        .enumerate()
    {
        if game.max_ammunition(index) != 0 {
            install(item, game);
        }
    }
    word! { game, 0; SAMUS_PREV_HEALTH, SAMUS_PREV_MISSILES, SAMUS_PREV_SUPER_MISSILES,
    SAMUS_PREV_POWER_BOMBS, SAMUS_PREV_HUD_ITEM_INDEX, }
    ammunition(true, game);
    highlight(game.selected_item(), SELECTED_PALETTE, game);
    highlight(
        word!(game.SAMUS_PREV_HUD_ITEM_INDEX),
        UNSELECTED_PALETTE,
        game,
    );
    advance(game);
}

fn ammunition<H: HudFrameHost + ?Sized>(initial: bool, game: &mut H) {
    {
        const ROWS: [(usize, u16); 3] = [
            (fields::SAMUS_PREV_MISSILES, 0x94),
            (fields::SAMUS_PREV_SUPER_MISSILES, 0x9c),
            (fields::SAMUS_PREV_POWER_BOMBS, 0xa2),
        ];
        for (index, &(previous, offset)) in ROWS.iter().enumerate() {
            if game.max_ammunition(index) != 0
                && (initial || game.ammunition(index) != at!(game, previous;))
            {
                if !initial {
                    at!(game, previous; = game.ammunition(index));
                }
                let three = index == 0
                    || (!initial && index == 1 && word!(game.JOYPAD_DBG_FLAGS) & 0x1f40 != 0);
                let count = if !initial && index == 1 {
                    at!(game, previous;)
                } else {
                    game.ammunition(index)
                };
                number(WEAPON_DIGITS, count, offset, three, game);
            }
        }
    }
}

/// Native table capture retains its original diagnostic and allocation contract.
pub fn advance<H: HudFrameHost + ?Sized>(game: &mut H) {
    {
        if game.reserve_mode() == 1 {
            let reserve = native_record(RESERVE_TILES, game);
            let start = if game.reserve_health() == 0 { 12 } else { 0 };
            stamp(8, 2, 6, |i| reserve.word(start + 2 * i), game);
        }
        if game.health() != word!(game.SAMUS_PREV_HEALTH) {
            word!(game.SAMUS_PREV_HEALTH = game.health());
            let mut filled = game.health() / 100;
            let remainder = game.health() % 100;
            let count = usize::from(game.max_health() / 100);
            const TANKS: [u16; 14] = [
                0x42, 0x44, 0x46, 0x48, 0x4a, 0x4c, 0x4e, 2, 4, 6, 8, 0xa, 0xc, 0xe,
            ];
            for &offset in TANKS.iter().take(count) {
                let tile = if filled != 0 {
                    filled -= 1;
                    FULL_TANK_TILE
                } else {
                    EMPTY_TANK_TILE
                };
                crate::word_transfers!(game, ascending, 1; native(TILEMAP_RAM + usize::from(offset)) <- value(tile));
            }
            number(HEALTH_DIGITS, remainder, 0x8c, false, game);
        }
        ammunition(false, game);
        if game.selected_item() != word!(game.SAMUS_PREV_HUD_ITEM_INDEX) {
            highlight(game.selected_item(), SELECTED_PALETTE, game);
            highlight(
                word!(game.SAMUS_PREV_HUD_ITEM_INDEX),
                UNSELECTED_PALETTE,
                game,
            );
            word!(game.SAMUS_PREV_HUD_ITEM_INDEX = game.selected_item());
            if game.movement_type() != SPIN_JUMPING
                && game.movement_type() != WALL_JUMPING
                && game.grapple_function() == GRAPPLE_INACTIVE
                && word!(game.TIME_IS_FROZEN_FLAG) == 0
            {
                game.queue_sound(0, 0x39_u8, 6);
            }
        }
        let palette = if game.raw_frame_counter_byte() & 0x10 != 0 {
            SELECTED_PALETTE
        } else {
            UNSELECTED_PALETTE
        };
        highlight(game.automatic_item(), palette, game);
        let tail = word!(game.VRAM_WRITE_QUEUE_TAIL);
        // Wrap each field cursor before widening; the native bank word overlaps destination.
        for (offset, value) in [
            (0, TILEMAP_BYTES),
            (2, TILEMAP_RAM as u16),
            (4, WORK_RAM_BANK),
            (5, TILEMAP_QUEUED_VRAM),
        ] {
            crate::word_transfers!(game, ascending, 1;
                native(0xd0 + usize::from(tail.wrapping_add(offset))) <- value(value));
        }
        word!(game.VRAM_WRITE_QUEUE_TAIL = tail.wrapping_add(7));
    }
}

fn highlight<H: HudFrameHost + ?Sized>(selection: u16, palette: u16, game: &mut H) {
    word!(game.HUD_ITEM_TILEMAP_PALETTE_BITS = palette);
    let item = (selection as i16).wrapping_sub(1);
    if item >= 0 {
        let base = usize::from(ITEM_OFFSETS[item as usize] >> 1);
        let offsets: &[usize] = if item == 0 {
            &[0, 1, 32, 33, 2, 34]
        } else {
            &[0, 1, 32, 33]
        };
        for &offset in offsets {
            let index = base + offset;
            if word!(game.HUD_TILEMAP[index]) != BLANK_TILE {
                word!(
                    game.HUD_TILEMAP[index] = word!(game.HUD_ITEM_TILEMAP_PALETTE_BITS)
                        | word!(game.HUD_TILEMAP[index]) & !PALETTE_BITS
                );
            }
        }
    }
}

fn number<H: HudFrameHost + ?Sized>(
    address: u32,
    mut value: u16,
    mut cursor: u16,
    three: bool,
    game: &mut H,
) {
    if three {
        word!(
            game.HUD_TILEMAP[cursor >> 1] =
                native_record(address.wrapping_add(2 * u32::from(value / 100)), game).word(0)
        );
        value %= 100;
        cursor = cursor.wrapping_add(2);
    }
    let base = usize::from(cursor >> 1);
    // Units follow the widened pair index; they do not wrap the u16 byte cursor.
    for (index, digit) in [(base, value / 10), (base + 1, value % 10)] {
        word!(
            game.HUD_TILEMAP[index] =
                native_record(address.wrapping_add(2 * u32::from(digit)), game).word(0)
        );
    }
}

pub fn menu_reserve_mode<
    H: HudPlayer + crate::ram::CompatibilityRam + crate::cartridge::NativeDiagnostics + ?Sized,
>(
    automatic: bool,
    game: &mut H,
) {
    if automatic {
        // The equipment menu intentionally uses bank82, unlike the ordinary HUD.
        let cursor = if game.reserve_health() == 0 {
            0x9997
        } else {
            0x998b
        };
        let source = native_record(0x820000 | cursor, game);
        stamp(8, 2, 6, |index| source.word(2 * index), game);
    } else {
        stamp(8, 2, 6, |_| BLANK_TILE, game);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn initial_tiles_match_the_original_table() {
        const ORIGINAL: [u16; 96] = [
            0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
            0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
            0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c12, 0x2c12, 0x2c23, 0x2c12, 0x2c12, 0x2c1e, 0x2c0f,
            0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
            0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
            0x2c0f, 0x2c0f, 0x2c0f, 0x2822, 0x2822, 0x2823, 0x2813, 0x2c14, 0x2c1e, 0x2c0f, 0x2c0b,
            0x2c0c, 0x2c0d, 0x2c32, 0x2c0f, 0x2c09, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
            0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f, 0x2c0f,
            0x2c0f, 0x2c0f, 0x2c12, 0x2c12, 0xa824, 0x2815, 0x2c16, 0x2c1e,
        ];
        assert_eq!(super::INITIAL_TILES, ORIGINAL);
    }
}
