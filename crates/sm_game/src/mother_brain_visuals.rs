//! Mother Brain encounter palettes, fades, wall sealing, body flicker and HDMA spawns.
use crate::actors::boss_lease::BossState;
use crate::actors::mother_brain::{BODY_INDEX, BRAIN_INDEX, Body, BodyEdit, Brain};
use crate::actors::mother_brain_fight::{Fade, Lease};
use crate::actors::slot_view::SlotState;
use crate::constants::bank_86::ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION;
use crate::constants::enemy_properties::K_ENEMY_PROPS_INVISIBLE;
use crate::display::hdma_spawn::HdmaObjectDefinition;
use crate::imported::{Imported, word};
use crate::palette_copies::copy_rom_colors_to_palette;
use crate::runtime_access::SessionRam;
use sm_data::mother_brain::{
    BACK_LEG_PALETTES_ADDRESS, BODY_PALETTES_ADDRESS, HealthPalettes, PALETTES_BYTES,
    import_health_palettes,
};

/// Mother Brain's palette lines in the palette buffer, by byte offset, and
/// the bank every colour source lives in.
const BRAIN_LINE: u16 = 0x62;
const LEGS_LINE: u16 = 0xa2;
const BODY_LINE: u16 = 0x82;
const BODY_ALIAS_LINE: u16 = 0x122;
const BACK_LEG_LINE: u16 = 0x162;
const PALETTE_BANK: u8 = 0xad;
/// The lights-off colours ($AD:F22F) for the brain and leg lines.
const LIGHTS_OFF_COLORS: u16 = 0xf22f;
/// The main-screen designation mirror the body flicker toggles BG2 in.
/// The encounter's health palette tables, imported from the loaded image.
#[derive(Default)]
pub struct ImportedTables {
    body_palettes: Option<Imported<HealthPalettes>>,
    back_leg_palettes: Option<Imported<HealthPalettes>>,
}

pub fn import(image: &sm_data::cartridge::RomImage) -> ImportedTables {
    let tables = |address| Imported::import(image, address, PALETTES_BYTES, import_health_palettes);
    ImportedTables {
        body_palettes: tables(BODY_PALETTES_ADDRESS),
        back_leg_palettes: tables(BACK_LEG_PALETTES_ADDRESS),
    }
}

/// The palette and layer steps' host: a boss state's slot state, palette
/// copies and cartridge, the projected RAM words, and the encounter's tables.
pub trait PaletteHost:
    BossState + SessionRam + SlotState + crate::display::preparation::Access
{
    fn mother_brain_visuals(&self) -> &ImportedTables;
}

/// $A9:881D: Mother Brain body function - fake death - descent - initial pause.
const FAKE_DEATH_DESCENT_INITIAL_PAUSE: u16 = 0x881d;

/// The queue byte the drained fades' stray accent word lands on ($7E:017C).
const STRAY_ACCENT: usize = 0x17c - 0xd0;

/// Step fade `kind` to `frame`; returns 1 once its colour list ends.
pub fn fade<H: PaletteHost + ?Sized>(kind: Fade, frame: u16, game: &mut H) -> u8 {
    let (table, colors) = match kind {
        Fade::ToBlack => (0xad_e9e8, 14),
        Fade::FakeDeathToGray => (0xad_ed8a, 3),
        Fade::FakeDeathFromGray => (0xad_ed9c, 3),
        Fade::DrainedFromGray => (0xad_ed9c, 13),
        Fade::DrainedToGray => (0xad_ef87, 15),
        Fade::RealDeathToGray => (0xad_f107, 15),
        Fade::LightsOn => (0xad_f273, 14),
    };
    let base = sm_data::cartridge::lorom_offset(table) as usize;
    let source = game.native_image().table_word(base, frame as isize);
    if source == 0 {
        return 1;
    }
    let mut copy =
        |line, source, count| copy_rom_colors_to_palette(line, PALETTE_BANK, source, count, game);
    match kind {
        Fade::FakeDeathToGray | Fade::FakeDeathFromGray => copy(BODY_ALIAS_LINE, source, colors),
        Fade::RealDeathToGray => copy(0x1e2, source, colors),
        Fade::LightsOn => {
            copy(BRAIN_LINE, source, colors);
            copy(LEGS_LINE, source.wrapping_add(28), colors);
        }
        Fade::ToBlack | Fade::DrainedFromGray | Fade::DrainedToGray => {
            copy(BODY_LINE, source, colors);
            copy(BODY_ALIAS_LINE, source, colors);
            let tail = source.wrapping_add(colors * 2);
            if matches!(kind, Fade::ToBlack) {
                copy(BACK_LEG_LINE, tail, colors);
            } else {
                copy(0x168, tail, 5);
                // The original accent write targets $017c, outside the palette
                // buffer: bytes $AC..$AD of the queued tilemaps' queue ($7E:00D0).
                let address = (u32::from(PALETTE_BANK) << 16) | u32::from(tail.wrapping_add(10));
                let accent = crate::menus::menu_tilemap::native_record(address, game).word(0);
                let queue = &mut game.session_mut().queued_tilemaps.queue;
                queue[STRAY_ACCENT..STRAY_ACCENT + 2].copy_from_slice(&accent.to_ne_bytes());
                game.words_stored();
            }
        }
    }
    0
}

/// Mother Brain's body and back-leg palettes for her current health band.
pub fn update_health_palette<H: PaletteHost + ?Sized>(game: &mut H) {
    if Body::of(game, BODY_INDEX).corpse_state() < 2 {
        return;
    }
    let band = match Brain::of(game, BRAIN_INDEX).actor().health {
        9000.. => 0,
        5400.. => 1,
        1800.. => 2,
        _ => 3,
    };
    for line in [BODY_LINE, BODY_ALIAS_LINE] {
        let tables = &game.mother_brain_visuals().body_palettes;
        let source = word(tables, game, BODY_PALETTES_ADDRESS, band);
        copy_rom_colors_to_palette(line, PALETTE_BANK, source, 15, game);
    }
    let tables = &game.mother_brain_visuals().back_leg_palettes;
    let source = word(tables, game, BACK_LEG_PALETTES_ADDRESS, band);
    copy_rom_colors_to_palette(BACK_LEG_LINE, PALETTE_BANK, source, 15, game);
}

/// The lights-off colours on the brain and leg lines.
pub fn lights_off<H: PaletteHost + ?Sized>(game: &mut H) {
    for line in [BRAIN_LINE, LEGS_LINE] {
        copy_rom_colors_to_palette(line, PALETTE_BANK, LIGHTS_OFF_COLORS, 14, game);
    }
}

/// Seal the wall behind Mother Brain: two dust clouds and two wall blocks,
/// then her next function.
pub fn seal_wall(game: &mut impl Lease) {
    game.owned_step(spawn_wall_seal);
    *BodyEdit::of(game, BODY_INDEX).function() = FAKE_DEATH_DESCENT_INITIAL_PAUSE;
}

/// The wall seal's two dust clouds and two wall blocks.
fn spawn_wall_seal(game: &mut impl Lease) {
    for y in [72, 152] {
        game.place_projectile_spawn(248, y);
        crate::enemy_projectiles::lifecycle::spawn_room(
            game,
            ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16,
            9,
        );
    }
    for y in [4, 9] {
        crate::room_mechanisms::creation::spawn_hardcoded_in(game, 0xf, y, 0xb673);
    }
}

/// Flicker Mother Brain's body layer and visibility on alternate frames.
pub fn flicker_body<H: PaletteHost + ?Sized>(game: &mut H) {
    let invisible = K_ENEMY_PROPS_INVISIBLE as u16;
    // The main screen layers and the previous main screen layers after them.
    let screen = crate::frame_display_word!(game.MainLayersPair);
    if Body::of(game, BODY_INDEX).actor().frame & 1 != 0 {
        crate::frame_display_word!(game.MainLayersPair = screen | 2);
        BodyEdit::of(game, BODY_INDEX).actor().properties &= !invisible;
    } else {
        crate::frame_display_word!(game.MainLayersPair = screen & !2);
        BodyEdit::of(game, BODY_INDEX).actor().properties |= invisible;
    }
}

/// The rising and rainbow beams' HDMA objects; each returns its slot.
pub fn spawn_rising_hdma<H: crate::hdma_objects::Allocator + ?Sized>(game: &mut H) -> u16 {
    crate::hdma_objects::spawn_hdma_owned(0x88, &HdmaObjectDefinition::new(0, 0x2c, 0xe727), game)
}
pub fn spawn_rainbow_hdma<H: crate::hdma_objects::Allocator + ?Sized>(game: &mut H) -> u16 {
    crate::hdma_objects::spawn_hdma_owned(
        0x88,
        &HdmaObjectDefinition::new(0x41, 0x26, 0xe751),
        game,
    )
}
