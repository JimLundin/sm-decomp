//! What the free-run gate reads instead of a record cache: a row's seed and pads from the
//! relock bundle's directory, and the engine's [`Content`] from the ROM.
//!
//! The bundle (`relock-v2-lsnes/`, RELOCK §7) holds `seeds/<capture>/NNNN.record`, the raw
//! lsnes record of the cut before the seed's first step, and `pads/<row>.pads`, the row's
//! pad per frame. The lock pins the pads; the record is pinned through the bundle's
//! ciphertext, whose SHA-256 the lock's `bundles` entry holds and CI checks on download.

use std::fmt;
use std::path::{Path, PathBuf};

use data::enemy::eye::EyeLists;
use data::enemy::face_block::{FaceBlockLists, GlowColours};
use data::enemy::program::{EnemyPrograms, ProgramError};
use data::level::slope::Slopes;
use data::movement::{HorizontalSpeeds, VerticalSpeeds};
use data::palette::VisorColours;
use data::pose::animation::{AnimationDelays, AnimationError};
use data::pose::spritemap::{SamusSpritemaps, SpritemapError};
use data::pose::tiles::{GRAPHICS_BANKS, Graphics, SamusTiles};
use data::pose::transition::PoseTransitions;
use data::pose::{PoseError, Poses};
use data::rom::{AddressError, Rom, RomError};
use game::Content;
use game::plm::RoomPlms;
use game::state::Buttons;
use oracle_record::lsnes::RawError;

use crate::decode::{DecodeError, RoomMap};
use crate::gate::{Seed, buttons};
use crate::lock::{Row, RowId, Sha256, Start};
use crate::lockstep::{slice_zero_plms, slice_zero_rooms};

/// Why a row's inputs can't be read from the bundle.
#[derive(Debug)]
pub enum InputError {
    /// The row starts at power-on, and the gate runs only from seeds.
    PowerOn(RowId),
    /// The row's seed has WRAM edits, which land after the record's cut, so the record
    /// isn't its start.
    SeedWords(RowId),
    /// A file doesn't read.
    Io(PathBuf, std::io::Error),
    /// The pad file isn't the one the lock pins.
    PadsPin(PathBuf),
    /// A pad file line isn't a hexadecimal word.
    Pads(String),
    /// The seed's record isn't a raw lsnes record with the memories.
    Record(PathBuf, RawError),
    /// The seed's record doesn't decode to a State.
    Seed(PathBuf, DecodeError),
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PowerOn(row) => write!(f, "{row}: starts at power-on, not from a seed"),
            Self::SeedWords(row) => write!(f, "{row}: its seed has WRAM edits"),
            Self::Io(path, error) => write!(f, "{}: {error}", path.display()),
            Self::PadsPin(path) => write!(f, "{}: not the row's pads", path.display()),
            Self::Pads(error) => f.write_str(error),
            Self::Record(path, error) => write!(f, "{}: {error}", path.display()),
            Self::Seed(path, error) => write!(f, "{}: doesn't decode: {error:?}", path.display()),
        }
    }
}

impl std::error::Error for InputError {}

/// A row's free-run inputs: its seed and its pad per frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowInputs {
    /// Where the run starts.
    pub seed: Seed,
    /// One pad per frame; frame 0's is unused.
    pub pads: Vec<Buttons>,
}

impl RowInputs {
    /// Row `row`'s inputs from the bundle directory `root`.
    ///
    /// # Errors
    ///
    /// An [`InputError`] when the row has no plain seed or a file is missing or wrong.
    pub fn read(root: &Path, row: &Row, rooms: &RoomMap) -> Result<Self, InputError> {
        let Start::Seed { capture, index } = &row.start else {
            return Err(InputError::PowerOn(row.id.clone()));
        };
        if row.seed_words.is_some() {
            return Err(InputError::SeedWords(row.id.clone()));
        }
        let pad_path = root.join("pads").join(format!("{}.pads", row.id));
        let pad_bytes =
            std::fs::read(&pad_path).map_err(|error| InputError::Io(pad_path.clone(), error))?;
        if Sha256::of(&pad_bytes) != row.pads {
            return Err(InputError::PadsPin(pad_path));
        }
        let pads = oracle_record::inputs::read(&pad_path)
            .map_err(InputError::Pads)?
            .into_iter()
            .map(buttons)
            .collect();
        let record_path = root
            .join("seeds")
            .join(capture)
            .join(format!("{index:04}.record"));
        let bytes = std::fs::read(&record_path)
            .map_err(|error| InputError::Io(record_path.clone(), error))?;
        let record = oracle_record::lsnes::read(&bytes)
            .map_err(|error| InputError::Record(record_path.clone(), error))?;
        let seed =
            Seed::decode(&record, rooms).map_err(|error| InputError::Seed(record_path, error))?;
        Ok(Self { seed, pads })
    }
}

/// Why the ROM gives no [`Content`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContentError {
    /// The image isn't the release.
    Rom(RomError),
    /// Samus's pose table doesn't decode.
    Poses(PoseError),
    /// Samus's pose transition lists aren't in the image.
    Transitions(AddressError),
    /// Samus's horizontal speed tables aren't in the image.
    Speeds(AddressError),
    /// Samus's animation delay lists don't decode.
    Delays(AnimationError),
    /// The slope shapes aren't in the image.
    Slopes(AddressError),
    /// The enemies' instruction lists don't decode.
    EnemyPrograms(ProgramError),
    /// Samus's spritemaps don't decode.
    SamusSpritemaps(SpritemapError),
    /// Samus's or the face block's palette tables are outside the image.
    Palettes(AddressError),
    /// Samus's tiles definitions or graphics are outside the image.
    SamusTiles(AddressError),
    /// Slice 0's rooms don't decode.
    Rooms(data::room::RoomError),
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rom(error) => write!(f, "not the ROM: {error:?}"),
            Self::Poses(error) => write!(f, "the pose table: {error:?}"),
            Self::Transitions(error) => write!(f, "the pose transitions: {error:?}"),
            Self::Speeds(error) => write!(f, "the speed tables: {error:?}"),
            Self::Delays(error) => write!(f, "the animation delay lists: {error:?}"),
            Self::Slopes(error) => write!(f, "the slope shapes: {error:?}"),
            Self::EnemyPrograms(error) => write!(f, "the enemy instruction lists: {error:?}"),
            Self::SamusSpritemaps(error) => write!(f, "Samus's spritemaps: {error:?}"),
            Self::Palettes(error) => write!(f, "the palette tables: {error:?}"),
            Self::SamusTiles(error) => write!(f, "Samus's tiles: {error:?}"),
            Self::Rooms(error) => write!(f, "the rooms: {error:?}"),
        }
    }
}

impl std::error::Error for ContentError {}

/// The engine's content from the ROM image's bytes.
///
/// # Errors
///
/// A [`ContentError`] when the image isn't the release or a table doesn't decode.
pub fn content(image: Vec<u8>) -> Result<Content, ContentError> {
    // Decoded before the image moves into the ROM, reported after the ROM's own check.
    let populations = crate::lockstep::slice_zero_plms(&image);
    let rom = Rom::new(image).map_err(ContentError::Rom)?;
    let room_plms = RoomPlms(populations.map_err(ContentError::Rooms)?);
    let enemy_programs =
        EnemyPrograms::slice_zero(rom.lorom()).map_err(ContentError::EnemyPrograms)?;
    let eye_lists = EyeLists::resolve(&enemy_programs).map_err(ContentError::EnemyPrograms)?;
    let face_block_lists =
        FaceBlockLists::resolve(&enemy_programs).map_err(ContentError::EnemyPrograms)?;
    let delays = AnimationDelays::load(rom.lorom()).map_err(ContentError::Delays)?;
    let samus_tiles = SamusTiles::load(rom.lorom(), &delays).map_err(ContentError::SamusTiles)?;
    Ok(Content {
        poses: Poses::load(rom.lorom()).map_err(ContentError::Poses)?,
        transitions: PoseTransitions::load(rom.lorom()).map_err(ContentError::Transitions)?,
        speeds: HorizontalSpeeds::load(rom.lorom()).map_err(ContentError::Speeds)?,
        vertical: VerticalSpeeds::load(rom.lorom()).map_err(ContentError::Speeds)?,
        delays,
        slopes: Slopes::load(rom.lorom()).map_err(ContentError::Slopes)?,
        enemy_programs,
        eye_lists,
        face_block_lists,
        samus_spritemaps: SamusSpritemaps::load(rom.lorom())
            .map_err(ContentError::SamusSpritemaps)?,
        samus_tiles,
        graphics: Graphics::load(rom.lorom(), GRAPHICS_BANKS).map_err(ContentError::SamusTiles)?,
        visor_colours: VisorColours::load(rom.lorom()).map_err(ContentError::Palettes)?,
        face_block_glow: GlowColours::load(rom.lorom()).map_err(ContentError::Palettes)?,
        room_plms,
    })
}

/// The rooms the decoder knows, with what it needs to read every slot.
///
/// That is the content's instruction lists, Samus's tiles definitions and the rooms' PLM
/// populations (read from the ROM `image`), so occupied enemy and PLM slots and Samus's tile
/// uploads decode. `tools gate` and `tools lockstep` both build their map here, so the two
/// can't drift.
///
/// # Errors
/// When a room's header or state doesn't decode from the image.
pub fn rooms(content: &Content, image: &[u8]) -> Result<RoomMap, data::room::RoomError> {
    Ok(slice_zero_rooms()
        .with_programs(content.enemy_programs.clone())
        .with_samus_tiles(content.samus_tiles.clone())
        .with_plms(slice_zero_plms(image)?))
}
