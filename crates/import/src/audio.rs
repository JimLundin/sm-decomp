//! Imports the opening Ceres arrangement and the cues admitted by resident audio.

mod effect;
mod music;
mod source;

use data::assets::audio::{Content, ResidentAudio, ResidentMusicReply};
use data::assets::world::rooms::{
    CERES_ELEVATOR_ROOM, DEAD_SCIENTIST_ROOM, FALLING_TILE_ROOM, MAGNET_STAIRS_ROOM,
};
use data::id::RoomId;
use data::rom::{AddressError, Lorom, Rom, RomAddress};
use data::room::{self, Area, Music, RoomError};

use source::{Resources, Upload};

/// Why the cartridge's resident audio cannot be admitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    /// No opening Ceres arrangement is admitted for this room.
    RoomSelection(RoomId),
    /// The room header or its states could not be read.
    Room(RoomError),
    /// The ROM does not contain a source pointer.
    Address(AddressError),
    /// A source upload is truncated, out of bounds or overlaps itself.
    Upload,
    /// A pointer leaves the designated asset region.
    Pointer,
    /// The arrangement is outside the supported two-track, repeating form.
    Arrangement,
    /// A selected track uses an unsupported command, parameter or phrase form.
    Music,
    /// A selected cue source is replaced, aliased to another cue or has invalid part arity.
    Cue,
    /// A selected effect uses an unsupported command or span form.
    Effect,
    /// An instrument or its sample is unavailable or outside the supported subset.
    Resource,
}

const OPENING_ROUTE: [RoomId; 4] = [
    CERES_ELEVATOR_ROOM,
    FALLING_TILE_ROOM,
    MAGNET_STAIRS_ROOM,
    DEAD_SCIENTIST_ROOM,
];

/// Imports the resident arrangement for a room reached from the Ceres elevator in the opening
/// route. The elevator's initial state selects the song. Each subsequent room must retain it.
///
/// # Errors
///
/// Returns an error if the room or one of the required audio assets cannot be admitted.
pub fn resident_ceres(rom: &Rom, selected: RoomId) -> Result<ResidentAudio, ImportError> {
    let image = rom.lorom();
    let selection = opening_selection(image, selected)?;
    decode(image, selection)
}

fn opening_selection(rom: Lorom<'_>, selected: RoomId) -> Result<Music, ImportError> {
    let end = OPENING_ROUTE
        .iter()
        .position(|&candidate| candidate == selected)
        .ok_or(ImportError::RoomSelection(selected))?;
    let decode_room = |id| {
        let key = crate::world::key(id).ok_or(ImportError::RoomSelection(id))?;
        let decoded = room::decode(rom, key).map_err(ImportError::Room)?;
        if decoded.area != Area::Ceres {
            return Err(ImportError::RoomSelection(id));
        }
        Ok(decoded)
    };
    let entrance = decode_room(CERES_ELEVATOR_ROOM)?;
    let selection = entrance.default.music;
    if selection.data == 0 || selection.data % 3 != 0 || selection.track < 5 {
        return Err(ImportError::RoomSelection(CERES_ELEVATOR_ROOM));
    }
    for id in OPENING_ROUTE
        .into_iter()
        .take(end.saturating_add(1))
        .skip(1)
    {
        if decode_room(id)?
            .states()
            .any(|(_, state)| state.music != (Music { data: 0, track: 0 }))
        {
            return Err(ImportError::RoomSelection(id));
        }
    }
    Ok(selection)
}

// The data selector is already a byte offset in the table of long pointers.
const MUSIC_UPLOADS: RomAddress = RomAddress::new(0x8f, 0xe7e1);

fn decode(rom: Lorom<'_>, selection: Music) -> Result<ResidentAudio, ImportError> {
    let common = Upload::read(rom, MUSIC_UPLOADS)?;
    let song_pointer = MUSIC_UPLOADS
        .advance(usize::from(selection.data))
        .ok_or(ImportError::Upload)?;
    let area = Upload::read(rom, song_pointer)?;
    let mut resources = Resources::new(&common, &area);
    let (tracks, echo_delay) = music::decode(&area, selection.track, &mut resources)?;
    let (effects, aliases) = effect::decode(&common, &area, &mut resources)?;
    Ok(ResidentAudio {
        content: Content { tracks, effects },
        music_reply: ResidentMusicReply {
            identity: "resident-opening-forever".into(),
            original_byte: selection.track,
        },
        echo_delay,
        instruments: resources.instruments,
        samples: resources.samples,
        aliases,
    })
}

#[cfg(test)]
mod tests;
