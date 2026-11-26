//! Read the flight's art, placements and palette samples at the ROM boundary.

use std::num::NonZeroU16;

use data::assets::zebes::{
    CaptionContent, CaptionHolds, ColourEffectRef, DrawingPages, Effect, PageRefs, PaletteRef,
    PixelPoint, PlacedPicture, PlanetPictures, Sample, SpriteGraphicsRef, SpritePictureRef,
    StarField, TileGraphicsRef, TileMapRef, Zebes, ZebesScene,
};
use data::compression::{DecompressError, decompress};
use data::rom::{AddressError, Lorom, RomAddress};
use data::spritemap::Spritemap;

/// Why the flight's content could not be imported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Missing ROM data.
    Address(AddressError),
    /// Malformed compressed artwork.
    Compression(DecompressError),
    /// Malformed layout, palette sample or drawing.
    Layout,
}

impl From<AddressError> for Fault {
    fn from(value: AddressError) -> Self {
        Self::Address(value)
    }
}

const SCENE: u8 = 0x8b;
const PALETTES: u8 = 0x8d;
const DRAW_ORDER: [(u16, u16); 7] = [
    (0xc83b, 0xccad),
    (0xc943, 0xcd85),
    (0xc957, 0xcd8d),
    (0xc96b, 0xcd95),
    (0xc97f, 0xcd9d),
    (0xc992, 0xccc3),
    (0xc9e6, 0xccd7),
];

/// Import the gunship and planet art, the placed pictures, and their colour effects.
///
/// # Errors
/// Fails if the original's art and tables cannot supply the declared scene content.
pub fn import(rom: Lorom<'_>) -> Result<Zebes, Fault> {
    let placements = DRAW_ORDER
        .into_iter()
        .zip(1u16..)
        .map(|((init, list), id)| picture(rom, init, list, id))
        .collect::<Result<Vec<_>, _>>()?;
    let [planet, first, second, third, fourth, english, japanese] = placements.as_slice() else {
        return Err(Fault::Layout);
    };
    let engine = effect(rom, 1, 0xe1a8, 2, 1, true)?;
    let bright = effect(rom, 2, 0xe1b0, 8, 3, false)?;
    let dim = effect(rom, 3, 0xe1b4, 8, 3, false)?;
    let scene = ZebesScene {
        pages: DrawingPages {
            flat: PageRefs {
                map: TileMapRef(1),
                graphics: TileGraphicsRef(1),
            },
            ship: PageRefs {
                map: TileMapRef(2),
                graphics: TileGraphicsRef(2),
            },
            ship_origin: PixelPoint {
                x: operand(rom, 0xc74a)?.cast_signed(),
                y: operand(rom, 0xc752)?.cast_signed(),
            },
            sprite_graphics: SpriteGraphicsRef(1),
        },
        planet: PlanetPictures {
            planet: planet.0,
            star_fields: enum_map::enum_map! {
                StarField::First => first.0,
                StarField::Second => second.0,
                StarField::Third => third.0,
                StarField::Fourth => fourth.0,
            },
            last_field: StarField::Fourth,
        },
        caption: CaptionContent {
            english: english.0,
            japanese: Some(japanese.0),
            holds: CaptionHolds {
                hidden_lead: hold(rom, RomAddress::new(SCENE, 0xccbb))?,
                english_lead: hold(rom, RomAddress::new(SCENE, 0xccc1))?,
                full_caption: hold(rom, RomAddress::new(SCENE, 0xccc7))?,
                fading_caption: hold(rom, RomAddress::new(SCENE, 0xcccd))?,
            },
            brightening: bright.id,
            dimming: dim.id,
        },
        engine_accent: engine.id,
    };
    let ship_map = graphics(rom, 0x96, 0xfe69, 0x600)?
        .into_iter()
        .skip(0x300)
        .collect();
    let ship_tiles = graphics(rom, 0x95, 0xa82f, 0x4000)?;
    let flat_map = words(&graphics(rom, 0x97, 0x8adb, 0x800)?);
    let zebes_tiles = words(&graphics(rom, 0x96, 0xec76, 0x4000)?);
    Ok(Zebes {
        scene,
        ship_map,
        ship_tiles,
        flat_map,
        zebes_tiles,
        pictures: placements
            .into_iter()
            .map(|(placement, map)| (placement.picture, map))
            .collect(),
        effects: vec![engine, bright, dim],
    })
}

fn picture(
    rom: Lorom<'_>,
    init: u16,
    list: u16,
    id: u16,
) -> Result<(PlacedPicture, Spritemap), Fault> {
    let anchor = PixelPoint {
        x: operand(rom, init)?.cast_signed(),
        y: operand(rom, init.wrapping_add(6))?.cast_signed(),
    };
    let palette = operand(rom, init.wrapping_add(12))?;
    if palette & !0x0e00 != 0 {
        return Err(Fault::Layout);
    }
    let address = RomAddress::new(0x8c, word(rom, RomAddress::new(SCENE, list))?);
    let drawing = Spritemap::load(rom, address)?;
    if drawing.entries.is_empty() || drawing.entries.len() >= 128 {
        return Err(Fault::Layout);
    }
    Ok((
        PlacedPicture {
            picture: SpritePictureRef(id),
            palette: PaletteRef(palette >> 9),
            anchor,
        },
        drawing,
    ))
}

fn effect(
    rom: Lorom<'_>,
    id: u16,
    definition: u16,
    count: usize,
    width: usize,
    repeats: bool,
) -> Result<Effect, Fault> {
    let list = word(rom, RomAddress::new(PALETTES, definition.wrapping_add(2)))?;
    let at = RomAddress::new(PALETTES, list);
    let first_word = word(rom, next(at, 2)?)?;
    let first = usize::from(first_word / 2);
    if word(rom, at)? != 0xc655 || first_word % 2 != 0 || first.saturating_add(width) > 256 {
        return Err(Fault::Layout);
    }
    let mut cursor = next(at, 4)?;
    let mut samples = Vec::with_capacity(count);
    for _ in 0..count {
        let frames = hold(rom, cursor)?;
        let bytes = rom.read(next(cursor, 2)?, width.saturating_mul(2))?;
        let colours = words(bytes);
        cursor = next(cursor, bytes.len().saturating_add(2))?;
        if word(rom, cursor)? != 0xc595 || colours.iter().any(|&colour| colour >= 0x8000) {
            return Err(Fault::Layout);
        }
        samples.push(Sample { frames, colours });
        cursor = next(cursor, 2)?;
    }
    if repeats {
        if word(rom, cursor)? != 0xc61e || word(rom, next(cursor, 2)?)? != list.wrapping_add(4) {
            return Err(Fault::Layout);
        }
    } else if word(rom, cursor)? != 0xc5cf {
        return Err(Fault::Layout);
    }
    Ok(Effect {
        id: ColourEffectRef(id),
        first,
        samples,
        repeats,
    })
}

fn graphics(rom: Lorom<'_>, bank: u8, offset: u16, size: usize) -> Result<Vec<u8>, Fault> {
    let bytes = decompress(rom.from(RomAddress::new(bank, offset))?)
        .map_err(Fault::Compression)?
        .bytes;
    bytes.get(..size).map(<[u8]>::to_vec).ok_or(Fault::Layout)
}

fn words(bytes: &[u8]) -> Vec<u16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .copied()
        .map(u16::from_le_bytes)
        .collect()
}

fn operand(rom: Lorom<'_>, offset: u16) -> Result<u16, Fault> {
    let at = RomAddress::new(SCENE, offset);
    if rom.read(at, 1)? != [0xa9] {
        return Err(Fault::Layout);
    }
    word(rom, next(at, 1)?)
}

fn next(at: RomAddress, length: usize) -> Result<RomAddress, Fault> {
    at.advance(length).ok_or(Fault::Layout)
}

fn hold(rom: Lorom<'_>, at: RomAddress) -> Result<NonZeroU16, Fault> {
    NonZeroU16::new(word(rom, at)?).ok_or(Fault::Layout)
}

fn word(rom: Lorom<'_>, at: RomAddress) -> Result<u16, Fault> {
    Ok(u16::from_le_bytes(
        rom.read(at, 2)?.try_into().map_err(|_| Fault::Layout)?,
    ))
}
