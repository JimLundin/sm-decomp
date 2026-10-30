//! Phantoon's random timers and flame-rain positions through the imported `sm_data` tables.
#![forbid(unsafe_code)]
use crate::imported::Imported;
use crate::records::Records;
use sm_data::cartridge::{RomImage, lorom_offset};
use sm_data::phantoon::{
    FLAME_RAIN_ADDRESS, FLAME_RAIN_BYTES, FlameRainPositions, TIMERS_ADDRESS, TIMERS_BYTES, Timers,
    import_flame_rain,
};
pub use sm_data::phantoon::{FlameRainPosition, Timer};

pub type ImportedTimers = Imported<Timers>;

pub fn import_timers(image: &RomImage) -> Option<ImportedTimers> {
    Imported::import(image, TIMERS_ADDRESS, TIMERS_BYTES, Timers::import)
}

/// Entry `index` of `timer` in the loaded image. The table serves the eight
/// entries of the imported allocation; otherwise the original indexed load runs.
pub fn timer(game: &(impl Records + ?Sized), timer: Timer, index: usize) -> u16 {
    let rom = game.loaded_image();
    Imported::from(&game.records().phantoon_timers, rom)
        .and_then(|timers| timers.frames(timer, index))
        .unwrap_or_else(|| {
            let base = lorom_offset(TIMERS_ADDRESS) as usize + timer.offset();
            rom.table_word(base, index as isize)
        })
}

pub type ImportedFlameRain = Imported<FlameRainPositions>;

pub fn import(image: &RomImage) -> Option<ImportedFlameRain> {
    Imported::import(
        image,
        FLAME_RAIN_ADDRESS,
        FLAME_RAIN_BYTES,
        import_flame_rain,
    )
}

/// Position `row` at $A7:CDAD in the loaded image. The table serves it while
/// that image is the imported allocation; otherwise, and past the eight rows,
/// the original three indexed loads run with their failures.
pub fn flame_rain(game: &(impl Records + ?Sized), row: usize) -> FlameRainPosition {
    let rom = game.loaded_image();
    Imported::from(&game.records().flame_rain, rom)
        .and_then(|table| table.row(row))
        .unwrap_or_else(|| {
            let load = |at: usize| {
                rom.table_word(
                    lorom_offset(FLAME_RAIN_ADDRESS) as usize,
                    (4 * row + at) as isize,
                )
            };
            FlameRainPosition {
                figure_eight_index: load(0),
                x_position: load(1),
                y_position: load(2),
            }
        })
}
