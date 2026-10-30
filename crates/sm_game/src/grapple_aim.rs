//! Grapple beam firing and connection records through the imported `sm_data` tables.
use crate::imported::Imported;
use crate::records::Records;
use sm_data::cartridge::{RomImage, lorom_offset};
pub use sm_data::grapple_aim::ConnectionSet;
use sm_data::grapple_aim::{
    CONNECTION_ADDRESS, CONNECTION_BYTES, Connection, Connections, EXTENSION_ADDRESS,
    EXTENSION_BYTES, Extension, Extensions, POINT_ANIMATION_ADDRESS, POINT_ANIMATION_BYTES,
    PointAnimation,
};

/// Imported from the loaded image by `register_rom`.
#[derive(Default)]
pub struct ImportedAim {
    extensions: Option<Imported<Extensions>>,
    connections: Option<Imported<Connections>>,
    point_animation: Option<Imported<PointAnimation>>,
}

pub fn import(image: &RomImage) -> ImportedAim {
    ImportedAim {
        extensions: Imported::import(
            image,
            EXTENSION_ADDRESS,
            EXTENSION_BYTES,
            Extensions::import,
        ),
        connections: Imported::import(
            image,
            CONNECTION_ADDRESS,
            CONNECTION_BYTES,
            Connections::import,
        ),
        point_animation: Imported::import(
            image,
            POINT_ANIMATION_ADDRESS,
            POINT_ANIMATION_BYTES,
            PointAnimation::import,
        ),
    }
}

fn loaded(game: &(impl Records + ?Sized)) -> &RomImage {
    game.loaded_image()
}

/// The extension velocities and end angle for `direction`. Other images and
/// directions past ten keep the original three indexed loads.
pub fn extension(game: &(impl Records + ?Sized), direction: isize) -> Extension {
    let rom = loaded(game);
    usize::try_from(direction)
        .ok()
        .and_then(|direction| {
            Imported::from(&game.records().grapple_aim.extensions, rom)?.direction(direction)
        })
        .unwrap_or_else(|| {
            let column = |column| {
                let base =
                    lorom_offset(EXTENSION_ADDRESS) as usize + Extension::column_offset(column);
                rom.table_word(base, direction)
            };
            Extension {
                x_velocity: column(0),
                y_velocity: column(1),
                angle: column(2),
            }
        })
}

/// The connection at word index `word` (twice the direction) of `set`. Other
/// images and indices keep the original two indexed loads.
pub fn connection(game: &(impl Records + ?Sized), set: ConnectionSet, word: isize) -> Connection {
    let rom = loaded(game);
    usize::try_from(word)
        .ok()
        .filter(|word| word % 2 == 0)
        .and_then(|word| {
            Imported::from(&game.records().grapple_aim.connections, rom)?.direction(set, word / 2)
        })
        .unwrap_or_else(|| {
            let base = lorom_offset(set.address()) as usize;
            Connection {
                beam_function: rom.table_word(base, word),
                handler: rom.table_word(base, word + 1),
            }
        })
}

/// The grapple point animation's frame bounds. Other images keep the
/// original indexed loads.
pub fn point_animation(game: &(impl Records + ?Sized)) -> PointAnimation {
    let rom = loaded(game);
    Imported::from(&game.records().grapple_aim.point_animation, rom)
        .copied()
        .unwrap_or_else(|| {
            let base = lorom_offset(POINT_ANIMATION_ADDRESS) as usize;
            PointAnimation {
                first: rom.table_word(base, 0),
                end: rom.table_word(base, 1),
            }
        })
}
