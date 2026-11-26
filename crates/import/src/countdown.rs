//! The countdown word and its ten digit drawings from bank 80.
use data::assets::countdown::{Figures, IndirectHdmaStart};
use data::rom::{AddressError, Lorom, RomAddress};
use data::spritemap::Spritemap;

/// Load the drawings used by the countdown picture.
///
/// # Errors
/// A map or pointer outside the player image is refused.
pub fn import(rom: Lorom<'_>) -> Result<Figures, AddressError> {
    let frame = Spritemap::load(rom, RomAddress::new(0x80, 0xa060))?;
    let pointers = rom.read(RomAddress::new(0x80, 0x9fd4), 20)?;
    let mut digits = std::array::from_fn(|_| Spritemap::default());
    for (digit, pointer) in digits.iter_mut().zip(pointers.as_chunks::<2>().0) {
        *digit = Spritemap::load(rom, RomAddress::new(0x80, u16::from_le_bytes(*pointer)))?;
    }
    Ok(Figures { frame, digits })
}

/// Resolve a queued shaft platform map written to the low lane of Mode 7 VRAM.
///
/// # Errors
/// A source crossing the supplied player ROM bounds is rejected.
pub fn platform_tiles(rom: Lorom<'_>, source: RomAddress) -> Result<[u8; 4], AddressError> {
    let mut tiles = [0; 4];
    tiles.copy_from_slice(rom.read(source, 4)?);
    Ok(tiles)
}

/// Read the first indirect HDMA entry for a shaft channel.
///
/// # Errors
/// A source crossing the supplied player ROM bounds is rejected.
pub fn indirect_hdma_start(
    rom: Lorom<'_>,
    source: RomAddress,
) -> Result<IndirectHdmaStart, AddressError> {
    let mut header = [0; 3];
    header.copy_from_slice(rom.read(source, 3)?);
    let [lines, low, high] = header;
    Ok(IndirectHdmaStart {
        source,
        lines,
        indirect_address: u16::from_le_bytes([low, high]),
    })
}

/// Why the shaft's indirect raster content could not be imported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RasterError {
    /// An address in a spawn descriptor or a table was outside the ROM.
    Address(AddressError),
    /// A table was outside the supported full-frame indirect programs.
    Table,
}

impl From<AddressError> for RasterError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// Import the haze, elevator mode split and HUD scroll programs of the return shaft.
///
/// # Errors
/// A spawn descriptor or table is invalid, or a table does not terminate in the visible frame.
pub fn shaft_raster(
    rom: Lorom<'_>,
) -> Result<[data::assets::countdown::IndirectHdmaProgram; 3], RasterError> {
    const HAZE: RomAddress = RomAddress::new(0x88, 0xdddd);
    const ELEVATOR_MODE: RomAddress = RomAddress::new(0x88, 0xd92c);
    const HUD_SCROLL: RomAddress = RomAddress::new(0x88, 0xd869);
    Ok([
        raster_program(rom, HAZE, 17)?,
        raster_program(rom, ELEVATOR_MODE, 10)?,
        raster_program(rom, HUD_SCROLL, 8)?,
    ])
}

/// Import the direct main-screen layer table used in Ridley's room.
///
/// # Errors
/// The table is invalid, names an unsupported transfer, or runs past the visible frame.
pub fn ridley_main_screen(rom: Lorom<'_>) -> Result<data::dma::DirectHdmaProgram, RasterError> {
    use data::dma::{BusAddress, DirectHdmaProgram};

    let spawn = RomAddress::new(0x88, 0xd8ea);
    let [control, destination, low, high] = rom
        .read(spawn, 4)?
        .try_into()
        .map_err(|_| RasterError::Table)?;
    if control != 0 || destination != 0x2c {
        return Err(RasterError::Table);
    }
    let script = RomAddress::new(spawn.bank, u16::from_le_bytes([low, high]));
    let address = u16::from_le_bytes([read_byte(rom, script, 5)?, read_byte(rom, script, 6)?]);
    let table = RomAddress::new(read_byte(rom, script, 2)?, address);
    let mut cursor = table;
    let mut visible = 0u16;
    while visible < 224 {
        let lines = read_byte(rom, cursor, 0)?;
        if lines == 0 && visible > 0 {
            return Ok(DirectHdmaProgram {
                control,
                destination,
                source: BusAddress {
                    bank: table.bank,
                    offset: table.offset,
                },
                terminal_cursor: cursor.advance(1).ok_or(RasterError::Table)?.offset,
            });
        }
        if lines == 0 || lines > 127 {
            return Err(RasterError::Table);
        }
        visible = visible.saturating_add(u16::from(lines));
        cursor = cursor.advance(2).ok_or(RasterError::Table)?;
    }
    Err(RasterError::Table)
}

fn raster_program(
    rom: Lorom<'_>,
    spawn: RomAddress,
    table_pointer_offset: usize,
) -> Result<data::assets::countdown::IndirectHdmaProgram, RasterError> {
    use data::assets::countdown::IndirectHdmaProgram;

    let mut descriptor = [0; 4];
    descriptor.copy_from_slice(rom.read(spawn, 4)?);
    let [control, destination, script_low, script_high] = descriptor;
    if control & 0x40 == 0 {
        return Err(RasterError::Table);
    }
    let script = RomAddress::new(spawn.bank, u16::from_le_bytes([script_low, script_high]));
    let table_bank = read_byte(rom, script, 2)?;
    let data_bank = read_byte(rom, script, 5)?;
    let table_register = script
        .advance(table_pointer_offset)
        .ok_or(RasterError::Table)?;
    let mut address = [0; 2];
    address.copy_from_slice(rom.read(table_register, 2)?);
    let table = RomAddress::new(table_bank, u16::from_le_bytes(address));
    let opening = indirect_hdma_start(rom, table)?;
    if opening.lines == 0 {
        return Err(RasterError::Table);
    }

    let mut cursor = table;
    let mut lines = 0u16;
    // Fixed Ceres is a 224-line mode and these three tables all reach a terminator.
    while lines < 224 {
        let header = read_byte(rom, cursor, 0)?;
        if header == 0 {
            let terminal_address = u16::from_le_bytes([header, read_byte(rom, cursor, 1)?]);
            return Ok(IndirectHdmaProgram {
                control,
                destination,
                data_bank,
                opening,
                terminal_address,
                terminal_cursor: cursor.advance(2).ok_or(RasterError::Table)?.offset,
            });
        }
        let line_count = header & 0x7f;
        lines = lines.saturating_add(if line_count == 0 {
            128
        } else {
            u16::from(line_count)
        });
        cursor = cursor.advance(3).ok_or(RasterError::Table)?;
    }
    Err(RasterError::Table)
}

fn read_byte(rom: Lorom<'_>, start: RomAddress, offset: usize) -> Result<u8, RasterError> {
    let address = start.advance(offset).ok_or(RasterError::Table)?;
    rom.read(address, 1)?
        .first()
        .copied()
        .ok_or(RasterError::Table)
}

/// The tilemap selected by the escape shaft's room effects.
///
/// # Errors
/// Returns an address error if the selected graphics are outside the ROM.
pub fn shaft_effects(rom: Lorom<'_>) -> Result<Vec<u16>, AddressError> {
    Ok(rom
        .read(RomAddress::new(0x8a, 0xb279), 0x840)?
        .as_chunks::<2>()
        .0
        .iter()
        .copied()
        .map(u16::from_le_bytes)
        .collect())
}
