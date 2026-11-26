//! Resolve the station explosion's art, placements and finite animation tracks at the ROM edge.

use core::num::NonZeroU8;
use data::assets::explosion::{
    ArtSet, AsteroidContent, AsteroidField, BackdropContent, BackdropFrame, BurstGroup, BurstTrack,
    ExplosionScene, MapDrawing, NavigationSample, Pair, PictureColours, SampleLook, SkyContent,
    SpacePalette, SpriteDrawing, Station, StationFlashStyle, Subpixels, TrackSample,
};
use data::compression::{DecompressError, decompress};
use data::rom::{AddressError, Lorom, RomAddress};
use data::spritemap::Spritemap;
use enum_map::enum_map;

/// Why a player image cannot supply the scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// An asset's address falls outside the image.
    Address(AddressError),
    /// Compressed art is invalid.
    Compression(DecompressError),
    /// A drawing, animation, placement or palette reference is unsupported.
    Layout,
}

impl From<AddressError> for Fault {
    fn from(value: AddressError) -> Self {
        Self::Address(value)
    }
}

fn word(rom: Lorom<'_>, bank: u8, at: u16) -> Result<u16, Fault> {
    Ok(u16::from_le_bytes(
        rom.read(RomAddress::new(bank, at), 2)?
            .try_into()
            .map_err(|_| Fault::Layout)?,
    ))
}

fn next(at: u16, bytes: u16) -> Result<u16, Fault> {
    at.checked_add(bytes).ok_or(Fault::Layout)
}

fn operand(rom: Lorom<'_>, at: u16, opcode: u8) -> Result<u16, Fault> {
    if rom.read(RomAddress::new(0x8b, at), 1)? != [opcode] {
        return Err(Fault::Layout);
    }
    word(rom, 0x8b, at.checked_add(1).ok_or(Fault::Layout)?)
}

fn colour_group(value: u16) -> Result<u8, Fault> {
    if value & !0x0e00 != 0 {
        return Err(Fault::Layout);
    }
    u8::try_from(value >> 9).map_err(|_| Fault::Layout)
}

struct Drawings<'a> {
    rom: Lorom<'a>,
    pictures: Vec<(SpriteDrawing, Spritemap)>,
}

impl Drawings<'_> {
    fn drawing(&mut self, name: String, pointer: u16) -> Result<SpriteDrawing, Fault> {
        let id = SpriteDrawing(name);
        let drawing = Spritemap::load(self.rom, RomAddress::new(0x8c, pointer))?;
        if drawing.entries.is_empty() || drawing.entries.len() > 128 {
            return Err(Fault::Layout);
        }
        self.pictures.push((id.clone(), drawing));
        Ok(id)
    }

    fn asteroid(
        &mut self,
        name: &str,
        init: u16,
        speed: u16,
        list: u16,
    ) -> Result<AsteroidContent, Fault> {
        Ok(AsteroidContent {
            start: Pair {
                x: operand(self.rom, init, 0xa9)?.cast_signed(),
                y: operand(self.rom, next(init, 6)?, 0xa9)?.cast_signed(),
            },
            speed: Subpixels(i32::from(operand(self.rom, speed, 0x69)?)),
            drawing: self.drawing(name.to_owned(), word(self.rom, 0x8b, next(list, 2)?)?)?,
        })
    }

    fn track(&mut self, name: &str, list: u16) -> Result<BurstTrack, Fault> {
        let mut at = list;
        let repeats = word(self.rom, 0x8b, at)? == 0x94d6;
        let passes = if repeats {
            at = next(at, 4)?;
            positive(word(self.rom, 0x8b, next(list, 2)?)?)?
        } else {
            NonZeroU8::MIN
        };
        let mut samples = Vec::new();
        let first = at;
        let mut duration = 0u32;
        loop {
            let time = word(self.rom, 0x8b, at)?;
            if time & 0x8000 != 0 {
                break;
            }
            let frames = positive(time)?;
            duration = duration.saturating_add(u32::from(frames.get()));
            let pointer = word(self.rom, 0x8b, next(at, 2)?)?;
            let look = if pointer == 0 {
                SampleLook::Blank
            } else {
                SampleLook::Drawing(self.drawing(format!("{name}-{}", samples.len()), pointer)?)
            };
            samples.push(TrackSample { frames, look });
            at = at.checked_add(4).ok_or(Fault::Layout)?;
            if samples.len() > 128 {
                return Err(Fault::Layout);
            }
        }
        if repeats {
            if word(self.rom, 0x8b, at)? != 0x94c3 || word(self.rom, 0x8b, next(at, 2)?)? != first {
                return Err(Fault::Layout);
            }
            at = at.checked_add(4).ok_or(Fault::Layout)?;
        }
        if samples.is_empty()
            || duration.saturating_mul(u32::from(passes.get())) > 32_767
            || word(self.rom, 0x8b, at)? != 0x9438
        {
            return Err(Fault::Layout);
        }
        Ok(BurstTrack {
            samples,
            passes,
            palette: PictureColours("bursts".into()),
        })
    }
}

fn positive(word: u16) -> Result<NonZeroU8, Fault> {
    u8::try_from(word)
        .ok()
        .and_then(NonZeroU8::new)
        .ok_or(Fault::Layout)
}

fn graphics(rom: Lorom<'_>, bank: u8, at: u16) -> Result<Vec<u8>, Fault> {
    Ok(decompress(rom.from(RomAddress::new(bank, at))?)
        .map_err(Fault::Compression)?
        .bytes)
}

fn lights(rom: Lorom<'_>) -> Result<(usize, Vec<NavigationSample>), Fault> {
    let start = word(rom, 0x8d, 0xe1ba)?;
    if word(rom, 0x8d, start)? != 0xc655 {
        return Err(Fault::Layout);
    }
    let byte = word(rom, 0x8d, next(start, 2)?)?;
    if !byte.is_multiple_of(2) || byte > 508 || word(rom, 0x8d, next(start, 4)?)? != 0xc61e {
        return Err(Fault::Layout);
    }
    let cycle = word(rom, 0x8d, next(start, 6)?)?;
    let mut at = cycle;
    let mut samples = Vec::new();
    loop {
        let time = word(rom, 0x8d, at)?;
        if time == 0xc61e {
            if samples.is_empty() || word(rom, 0x8d, next(at, 2)?)? != cycle {
                return Err(Fault::Layout);
            }
            return Ok((usize::from(byte / 2), samples));
        }
        if time == 0
            || time >= 0x8000
            || samples.len() >= 128
            || word(rom, 0x8d, next(at, 6)?)? != 0xc595
        {
            return Err(Fault::Layout);
        }
        samples.push(NavigationSample {
            frames: time,
            colours: [
                word(rom, 0x8d, next(at, 2)?)?,
                word(rom, 0x8d, next(at, 4)?)?,
            ],
        });
        at = at.checked_add(8).ok_or(Fault::Layout)?;
    }
}

/// Import the settled station drawing and its animation tracks.
///
/// # Errors
/// Refuses unavailable art, unsupported lists, and values outside the model's types.
pub fn import(rom: Lorom<'_>) -> Result<Station, Fault> {
    let mut art = Drawings {
        rom,
        pictures: Vec::new(),
    };
    let backdrop = BackdropContent {
        place: Pair {
            x: operand(rom, 0xbfa5, 0xa9)?.cast_signed(),
            y: operand(rom, 0xbfb9, 0xa9)?.cast_signed(),
        },
        drawings: enum_map! {
            BackdropFrame::First => art.drawing("space-first".into(), word(rom, 0x8b, 0xcc5d)?)?,
            BackdropFrame::Second => art.drawing("space-second".into(), word(rom, 0x8b, 0xcc59)?)?,
        },
    };
    let sky = SkyContent {
        asteroids: enum_map! {AsteroidField::Large => art.asteroid("large-asteroids", 0xbf22, 0xbf39, 0xcc3f)?, AsteroidField::Small => art.asteroid("small-asteroids", 0xbf76, 0xbf8d, 0xcc4f)?},
        backdrop,
        palette: PictureColours("sky".into()),
    };
    let tracks = enum_map! {
        BurstGroup::First => art.track("first",0xccdb)?,
        BurstGroup::Middle => art.track("middle",0xccf5)?,
        BurstGroup::Last => art.track("last",0xcd1b)?,
        BurstGroup::Final => art.track("final",0xce1b)?,
    };
    let scene = ExplosionScene {
        art: ArtSet("station".into()),
        space_palette: SpacePalette("navigation".into()),
        flash_style: StationFlashStyle("station".into()),
        ship_front: MapDrawing("ship-front".into()),
        empty_drawing: MapDrawing("empty".into()),
        pivot: Pair {
            x: operand(rom, 0xc23a, 0xa9)?.cast_signed(),
            y: operand(rom, 0xc242, 0xa9)?.cast_signed(),
        },
        sky,
        tracks,
    };
    let maps = graphics(rom, 0x96, 0xfe69)?;
    let region = |range| maps.get(range).map(<[u8]>::to_vec).ok_or(Fault::Layout);
    let mut station_map = region(0x600..0xc00)?;
    let empty = rom
        .read(RomAddress::new(0x8b, 0xc1b8), 1)?
        .first()
        .copied()
        .ok_or(Fault::Layout)?;
    station_map.resize(0x4000, empty);
    let mut sprite_tiles = graphics(rom, 0x96, 0xd10a)?;
    sprite_tiles.truncate(0x4000);
    let common = rom.read(RomAddress::new(0x9a, 0xd200), 0x1a00)?;
    sprite_tiles
        .get_mut(..common.len())
        .ok_or(Fault::Layout)?
        .copy_from_slice(common);
    let plane_tiles = graphics(rom, 0x95, 0xa82f)?
        .get(..0x4000)
        .ok_or(Fault::Layout)?
        .to_vec();
    if sprite_tiles.len() != 0x4000 {
        return Err(Fault::Layout);
    }
    let mut palette = [0; 256];
    for (out, raw) in palette.iter_mut().zip(
        rom.read(RomAddress::new(0x8c, 0xe5e9), 512)?
            .as_chunks::<2>()
            .0,
    ) {
        *out = u16::from_le_bytes(*raw);
    }
    let (light_first, lights) = lights(rom)?;
    Ok(Station {
        scene,
        station_map,
        ship_front: region(0..0x300)?,
        ship_back: region(0x300..0x600)?,
        empty_map: region(0xc00..0xf00)?,
        plane_tiles,
        sprite_tiles,
        drawings: art.pictures,
        sky_palette: colour_group(operand(rom, 0xbfbf, 0xa9)?)?,
        burst_palette: colour_group(operand(rom, 0xc464, 0xa9)?)?,
        palette,
        light_first,
        lights,
    })
}
