//! Room FX loading ($89:AB02 and $89:AB82): the FX entry the arriving door selects, its liquid,
//! blending and palette, and the effects its kind and bits start.
use crate::cartridge::NativeDiagnostics;
use crate::room_entry::prepared_assets::PreparedRoomAssets;
use crate::session_access::SessionAccess;
use crate::visual_host::{Content, Source};
use crate::visual_state::{PaletteBuffer, Weather};
use sm_data::cartridge::{RomRecord, lorom_offset};
use std::sync::Arc;

/// The routine an FX kind starts ($89:AC2E table): `kind >> 1` selects it.
#[derive(Clone, Copy)]
pub enum Initializer {
    Lava,
    Acid,
    Water,
    Weather(Weather),
    Land,
    BandedSky,
    FirefleaLighting,
    TourianStatue,
    CeresRidley,
    CeresElevator,
    Haze,
}
impl Initializer {
    fn from_kind(kind: u8) -> Option<Self> {
        Some(match kind >> 1 {
            0 | 7..=15 => return None,
            1 => Self::Lava,
            2 => Self::Acid,
            3 => Self::Water,
            4 => Self::Weather(Weather::Spores),
            5 => Self::Weather(Weather::Rain),
            6 => Self::Weather(Weather::Fog),
            16 => Self::Land,
            17 => Self::BandedSky,
            18 => Self::FirefleaLighting,
            19 => Self::TourianStatue,
            20 => Self::CeresRidley,
            21 => Self::CeresElevator,
            22 => Self::Haze,
            _ => panic!("room FX kind outside callback table"),
        })
    }
}

/// The owned Session, the cartridge's FX records, and the effects an FX entry starts.
pub trait FxHost: SessionAccess + Content<Record = RomRecord> + NativeDiagnostics {
    fn initialize_fx(&mut self, initializer: Initializer);
    fn spawn_fx_palette(&mut self, definition: u16);
    fn spawn_fx_animated_tiles(&mut self, definition: u16);
}

/// An FX record: the loaded cartridge's, or a prepared world's copy of the same bytes.
enum FxRecord {
    Legacy(RomRecord),
    Prepared {
        assets: Arc<PreparedRoomAssets>,
        base: usize,
    },
}
impl FxRecord {
    fn byte(&self, offset: usize) -> u8 {
        match self {
            Self::Legacy(record) => record.byte(offset),
            Self::Prepared { assets, base } => assets.rom_byte(base.checked_add(offset).unwrap()),
        }
    }
    fn word(&self, offset: usize) -> u16 {
        u16::from_ne_bytes([self.byte(offset), self.byte(offset + 1)])
    }
}
fn definition(h: &mut impl FxHost, address: u16) -> FxRecord {
    let full = 0x830000 | u32::from(address);
    match h.session().world.prepared_room_assets() {
        Some(assets) => {
            if address & 0x8000 == 0 {
                h.native_rom_diagnostic(full);
            }
            FxRecord::Prepared {
                assets,
                base: lorom_offset(full) as usize,
            }
        }
        None => FxRecord::Legacy(crate::menus::menu_tilemap::native_record(full, h)),
    }
}
fn table(h: &impl FxHost, address: u32) -> FxRecord {
    match h.session().world.prepared_room_assets() {
        Some(assets) => FxRecord::Prepared {
            assets,
            base: lorom_offset(address) as usize,
        },
        None => FxRecord::Legacy(h.table(Source::Loaded, address)),
    }
}
fn apply(h: &mut impl FxHost, definition: &FxRecord, palette: PaletteBuffer) {
    let room = &mut h.session_mut().room;
    room.liquid_height = definition.word(2);
    room.liquid_target = definition.word(4);
    room.liquid_velocity = definition.word(6);
    room.liquid_delay = u16::from(definition.byte(8));
    room.fx_blending = u16::from(definition.byte(10));
    room.fx_layer3_blending = u16::from(definition.byte(11));
    room.liquid_options = u16::from(definition.byte(12));
    let blend = definition.byte(15);
    for color in if blend == 0 { 27..28 } else { 25..28 } {
        let value = if blend == 0 {
            0
        } else {
            table(h, 0x89aa02).word(usize::from(blend & !1) + 2 * (color - 25))
        };
        let visuals = h.session_mut().presentation.visual_mut();
        match palette {
            PaletteBuffer::Current => visuals.colors[color] = value,
            PaletteBuffer::Target => visuals.targets[color] = value,
        }
    }
}

/// Load FX entry `index` of the room state's FX list into the current palette ($89:AB82).
pub fn load_entry(h: &mut impl FxHost, index: u16) {
    let address = h.session().room.entry.fx_key.wrapping_add(16 * (index & 7));
    let definition = definition(h, address);
    apply(h, &definition, PaletteBuffer::Current);
}

/// Load the room state's FX entry for the arriving door, then start its effects ($89:AB02).
pub fn load_header(h: &mut impl FxHost) {
    let mut address = h.session().room.entry.fx_key;
    if address == 0 {
        return;
    }
    loop {
        let door = definition(h, address).word(0);
        if door == 0xffff {
            return;
        }
        let session = h.session();
        if door == 0 || door == session.world.exit_key(&session.room.entry.exit) {
            break;
        }
        address = address.wrapping_add(16);
    }
    let definition = definition(h, address);
    apply(h, &definition, PaletteBuffer::Target);
    let kind = definition.byte(9);
    h.session_mut().room.liquid.kind = u16::from(kind);
    if kind != 0 {
        let tilemap = table(h, 0x83abf0).word(usize::from(kind & !1));
        h.session_mut().room.liquid.tilemap = tilemap;
        if let Some(initializer) = Initializer::from_kind(kind) {
            h.initialize_fx(initializer);
        }
    }
    for (field, table_address, palette) in [(13, 0x83ac46, true), (14, 0x83ac56, false)] {
        let bits = definition.byte(field);
        if bits == 0 {
            continue;
        }
        let address = table(h, table_address).word(2 * usize::from(h.session().room.area));
        let objects = match h.session().world.prepared_room_assets() {
            Some(assets) => {
                if address & 0x8000 == 0 {
                    h.native_rom_diagnostic(0x830000 | u32::from(address));
                }
                FxRecord::Prepared {
                    assets,
                    base: lorom_offset(0x830000 | u32::from(address)) as usize,
                }
            }
            None => FxRecord::Legacy(h.legacy_native_record(0x83, address)),
        };
        for index in (0..8).filter(|index| bits & (1 << index) != 0) {
            let object = objects.word(2 * index);
            if palette {
                h.spawn_fx_palette(object);
            } else {
                h.spawn_fx_animated_tiles(object);
            }
        }
    }
}
