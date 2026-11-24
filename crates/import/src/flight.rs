//! The importer of the flight to Ceres's scene: the only code that knows where the original
//! keeps the flight's placements, its sprites' drifts and the station's label.
//!
//! The placements are the operands of the original's own instructions, so each is read only
//! where the instruction that carries it is the one expected; a ROM with other code there is
//! refused rather than read.

use std::num::NonZeroU16;

use data::assets::flight::{
    Angle, Drift, Drifter, Keystroke, LabelPlace, LabelStep, RushSpeed, Scene, SceneSetup,
    SceneTurn, SpritePlacement, SpriteWrap, TypedLetter, WholePlace, Zoom,
};
use data::rom::{AddressError, Lorom, RomAddress};
use enum_map::EnumMap;

/// Why the flight's scene cannot be imported. Errors carry no content bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    /// A placement, a drift or the label is outside the image.
    Address(AddressError),
    /// The instruction that carries a placement or a drift is not the one expected there.
    Code(RomAddress),
    /// The setup's zoom is over 32,767.
    Zoom,
    /// A step of the label holds for no frames.
    Hold,
    /// A step of the label draws something other than one letter of the label's font.
    Letter,
    /// The label's list holds an instruction the label doesn't use.
    Instruction,
    /// The label's list runs on past its bank, or past any label's length.
    Unended,
}

impl From<AddressError> for ImportError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// The code bank the flight runs in, and the bank of its label's list and tiles.
const CODE: u8 = 0x8b;
const LABEL: u8 = 0x8c;

/// The opcodes a placement or a drift is read from: a load, an add after a cleared carry, a
/// subtract after a set one, the mask that wraps a sprite across 512 px, and the store that ends
/// a drift that doesn't wrap.
const LOAD: u8 = 0xa9;
const ADD: [u8; 2] = [0x18, 0x69];
const SUBTRACT: [u8; 2] = [0x38, 0xe9];
const MASK: u8 = 0x29;
const WRAP_MASK: u16 = 0x01ff;
const STORE: u8 = 0x9d;

/// The loads of the setup: the plane's x and y, its angle and zoom; the rushing stars' speed,
/// x and y.
const SETUP_PLANE: [u16; 2] = [0xbd92, 0xbd98];
const SETUP_ANGLE: u16 = 0xbd9e;
const SETUP_ZOOM: u16 = 0xbda4;
const STARS_SPEED: u16 = 0xbe83;
const STARS_PLACE: [u16; 2] = [0xbe89, 0xbe8f];

/// The loads of the turn: the plane's x and y and its angle.
const TURN_PLANE: [u16; 2] = [0xbe29, 0xbe2f];
const TURN_ANGLE: u16 = 0xbe35;

/// Where each sprite the turn places is: the loads of its x and y, and the routine that drifts
/// it, whose add or subtract comes first and whose wrap, if any, comes `WRAP_AT` bytes in.
struct Placed {
    kind: Drifter,
    place: [u16; 2],
    drift: u16,
}

const PLACED: [Placed; 5] = [
    Placed {
        kind: Drifter::LargeAsteroids,
        place: [0xbf22, 0xbf28],
        drift: 0xbf38,
    },
    Placed {
        kind: Drifter::Station,
        place: [0xbf4c, 0xbf52],
        drift: 0xbf62,
    },
    Placed {
        kind: Drifter::SmallAsteroids,
        place: [0xbf76, 0xbf7c],
        drift: 0xbf8c,
    },
    Placed {
        kind: Drifter::Vortex,
        place: [0xbfb3, 0xbfb9],
        drift: 0xbfc9,
    },
    Placed {
        kind: Drifter::DistantStars,
        place: [0xbea2, 0xbea8],
        drift: 0xbfc9,
    },
];
const WRAP_AT: usize = 13;

/// The label's object, whose list pointer is `LIST_AT` bytes in.
const LABEL_OBJECT: u16 = 0xcf69;
const LIST_AT: usize = 4;
/// The list's instructions: skip the next step when the text is English, and end the label.
const SKIP_IF_ENGLISH: u16 = 0xc096;
const FINISH: u16 = 0xc0a2;
/// A step's length, and the most steps a label may have before its list counts as unended.
const STEP: usize = 6;
const MAX_STEPS: usize = 64;
/// What a step's drawing does: draw a block of tiles, or nothing.
const DRAW_TILES: u16 = 0x88b7;
const DRAW_NOTHING: u16 = 0x8849;

/// The label's font: the tile each of its letters is drawn with.
const FONT: [(u16, char); 9] = [
    (0x1ed, 'S'),
    (0x1ee, 'P'),
    (0x1ef, 'A'),
    (0x1f7, 'C'),
    (0x1f8, 'E'),
    (0x1f9, 'O'),
    (0x1fa, 'L'),
    (0x1fb, 'N'),
    (0x1b9, 'Y'),
];
/// The tile number of a tilemap word: its attributes dropped.
const TILE: u16 = 0x03ff;

/// The letter a tilemap word of the label's layer shows, if it is one of the label's font.
#[must_use]
pub fn letter(word: u16) -> Option<char> {
    FONT.iter()
        .find(|(tile, _)| *tile == word & TILE)
        .map(|&(_, letter)| letter)
}

/// The flight's scene, with the label in English.
///
/// # Errors
/// A placement, drift or label outside the image, other code where one is read, a zoom out of
/// range, and a label step that holds for no frames or draws other than one letter.
pub fn import(rom: Lorom<'_>) -> Result<Scene, ImportError> {
    let zoom = Zoom::new(load(rom, SETUP_ZOOM)?).ok_or(ImportError::Zoom)?;
    let setup = SceneSetup {
        angle: angle(rom, SETUP_ANGLE)?,
        zoom,
        plane: place(rom, SETUP_PLANE)?,
        stars: place(rom, STARS_PLACE)?,
        speed: RushSpeed(load(rom, STARS_SPEED)?.cast_signed()),
    };
    let mut turn = SceneTurn {
        angle: angle(rom, TURN_ANGLE)?,
        plane: place(rom, TURN_PLANE)?,
        sprites: EnumMap::default(),
    };
    for (kind, sprite) in &mut turn.sprites {
        if let Some(placed) = PLACED.iter().find(|placed| placed.kind == kind) {
            *sprite = Some(SpritePlacement {
                place: place(rom, placed.place)?,
                drift: drift(rom, placed.drift)?,
                wrap: wrap(rom, placed.drift)?,
            });
        }
    }
    Ok(Scene {
        setup,
        turn,
        label: label(rom)?,
    })
}

/// The steps of the label's list, the English text's: each letter at its place or a pause,
/// with its hold.
fn label(rom: Lorom<'_>) -> Result<Vec<LabelStep>, ImportError> {
    let mut at = RomAddress::new(LABEL, word(rom, at(CODE, LABEL_OBJECT), LIST_AT)?);
    let mut steps = Vec::new();
    while steps.len() < MAX_STEPS {
        match word(rom, at, 0)? {
            FINISH => return Ok(steps),
            SKIP_IF_ENGLISH => at = on(at, 2 + STEP)?,
            hold if hold & 0x8000 == 0 => {
                steps.push(LabelStep {
                    keystroke: keystroke(rom, word(rom, at, 2)?, word(rom, at, 4)?)?,
                    hold: NonZeroU16::new(hold).ok_or(ImportError::Hold)?,
                });
                at = on(at, STEP)?;
            }
            _ => return Err(ImportError::Instruction),
        }
    }
    Err(ImportError::Unended)
}

/// What a step types: nothing, or the one letter its block of tiles draws, at the step's place:
/// its column in the low byte, its row in the high.
fn keystroke(rom: Lorom<'_>, place: u16, drawing: u16) -> Result<Keystroke, ImportError> {
    let drawing = at(LABEL, drawing);
    match word(rom, drawing, 0)? {
        DRAW_NOTHING => Ok(Keystroke::Pause),
        DRAW_TILES if word(rom, drawing, 2)? == 0x0101 => {
            let letter = letter(word(rom, drawing, 4)?).ok_or(ImportError::Letter)?;
            let [column, row] = place.to_le_bytes();
            Ok(Keystroke::Letter(TypedLetter {
                letter,
                place: LabelPlace {
                    column: column.into(),
                    row: row.into(),
                },
            }))
        }
        _ => Err(ImportError::Letter),
    }
}

/// The operand of the load at `offset` in the flight's bank.
fn load(rom: Lorom<'_>, offset: u16) -> Result<u16, ImportError> {
    let place = at(CODE, offset);
    if byte(rom, place)? != LOAD {
        return Err(ImportError::Code(place));
    }
    word(rom, place, 1)
}

/// An angle, the low byte of its load.
fn angle(rom: Lorom<'_>, offset: u16) -> Result<Angle, ImportError> {
    let [low, _] = load(rom, offset)?.to_le_bytes();
    Ok(Angle(low))
}

/// A whole-pixel place, from the loads of its x and y.
fn place(rom: Lorom<'_>, [x, y]: [u16; 2]) -> Result<WholePlace, ImportError> {
    Ok(WholePlace {
        x: load(rom, x)?.cast_signed(),
        y: load(rom, y)?.cast_signed(),
    })
}

/// A drift, in 65,536ths: the fraction its routine adds or subtracts each frame.
fn drift(rom: Lorom<'_>, offset: u16) -> Result<Drift, ImportError> {
    let place = at(CODE, offset);
    let code = rom.read(place, 2)?;
    let step = i32::from(word(rom, place, 2)?);
    if code == ADD {
        Ok(Drift(step))
    } else if code == SUBTRACT {
        Ok(Drift(step.saturating_neg()))
    } else {
        Err(ImportError::Code(place))
    }
}

/// Whether the routine at `offset` masks the sprite's x to 512 px after its drift, or stores it
/// as it is; other code there is refused.
fn wrap(rom: Lorom<'_>, offset: u16) -> Result<SpriteWrap, ImportError> {
    let after = on(at(CODE, offset), WRAP_AT)?;
    match byte(rom, after)? {
        MASK if word(rom, after, 1)? == WRAP_MASK => Ok(SpriteWrap::Wraps),
        STORE => Ok(SpriteWrap::RunsOn),
        _ => Err(ImportError::Code(after)),
    }
}

const fn at(bank: u8, offset: u16) -> RomAddress {
    RomAddress::new(bank, offset)
}

fn on(place: RomAddress, count: usize) -> Result<RomAddress, ImportError> {
    place.advance(count).ok_or(ImportError::Unended)
}

fn byte(rom: Lorom<'_>, place: RomAddress) -> Result<u8, ImportError> {
    rom.read(place, 1)?
        .first()
        .copied()
        .ok_or(ImportError::Address(AddressError::PastEnd(place)))
}

/// The little-endian word `offset` bytes past `place`.
fn word(rom: Lorom<'_>, place: RomAddress, offset: usize) -> Result<u16, ImportError> {
    let place = on(place, offset)?;
    let bytes: [u8; 2] = rom
        .read(place, 2)?
        .try_into()
        .map_err(|_| ImportError::Address(AddressError::PastEnd(place)))?;
    Ok(u16::from_le_bytes(bytes))
}

// ROM sources for the data above, kept out of the public docs (the original's bank addresses):
// bank $8B log. The setup `$8B:BCA0` loads the plane's x, y, angle and zoom (`$BD92`..`$BDA4`);
// the stars' initialisation `$8B:BE7E` (parameter 0) their speed, x and y. The turn `$8B:BE09`
// loads the plane's x, y and angle (`$BE29`..`$BE35`) and spawns the large asteroids (`$BF22`,
// drift `$BF35`), the station (`$BF4C`, `$BF5F`), the small asteroids (`$BF76`, `$BF89`), the
// vortex with parameter 1 (`$BFA0`, `$BFC6`) and the stars with parameter 1 (`$BE9C`, `$BFC6`).
// Each drift routine adds or subtracts its fraction and then, to wrap, masks x with 1FFh. The
// station's view spawns the label's cinematic BG object `$8B:CF69`, whose list (bank 8C) holds
// steps of (timer, place, drawing), `$8B:C096` (skip the Japanese line) and `$8B:C0A2` (finish).
// A drawing is `$8B:88B7` with its width, height and tilemap words, or `$8B:8849`, nothing.

#[cfg(test)]
mod tests {
    use super::{ImportError, import};
    use data::assets::flight::{Drifter, Keystroke, SpriteWrap};
    use data::rom::Lorom;

    /// An image holding an invented scene where the importer reads one.
    struct Image(Vec<u8>);

    impl Image {
        fn new() -> Self {
            Self(vec![0; 0x68000])
        }

        fn put(&mut self, bank: u8, offset: u16, bytes: &[u8]) {
            let at = Lorom::new(&self.0)
                .file_offset(data::rom::RomAddress::new(bank, offset))
                .unwrap();
            self.0[at..]
                .iter_mut()
                .zip(bytes)
                .for_each(|(to, &from)| *to = from);
        }

        fn load(&mut self, offset: u16, value: u16) {
            let [low, high] = value.to_le_bytes();
            self.put(0x8b, offset, &[0xa9, low, high]);
        }
    }

    fn scene() -> Image {
        let mut image = Image::new();
        for (offset, value) in [
            (0xbd92, 0xfff0),
            (0xbd98, 0x0010),
            (0xbd9e, 0x0040),
            (0xbda4, 0x0300),
            (0xbe83, 0xff00),
            (0xbe89, 0x0020),
            (0xbe8f, 0x0030),
            (0xbe29, 0x0001),
            (0xbe2f, 0x0002),
            (0xbe35, 0x0080),
        ] {
            image.load(offset, value);
        }
        for offset in [0xbf22, 0xbf28, 0xbf4c, 0xbf52, 0xbf76, 0xbf7c] {
            image.load(offset, 0x0011);
        }
        for offset in [0xbfb3, 0xbfb9, 0xbea2, 0xbea8] {
            image.load(offset, 0x0022);
        }
        for (drift, mask) in [(0xbf38, 0xbf45), (0xbf62, 0xbf6f), (0xbf8c, 0xbf99)] {
            image.put(0x8b, drift, &[0x18, 0x69, 0x00, 0x01]);
            image.put(0x8b, mask, &[0x29, 0xff, 0x01]);
        }
        image.put(0x8b, 0xbfc9, &[0x38, 0xe9, 0x00, 0x02]);
        image.put(0x8b, 0xbfd6, &[0x9d]);
        // The label: one letter, the English skip over another, a pause, the end.
        image.put(0x8b, 0xcf69 + 4, &[0x00, 0x90]);
        image.put(
            0x8c,
            0x9000,
            &[
                0x05, 0x00, 0x03, 0x04, 0x00, 0x91, 0x96, 0xc0, 0x09, 0x00, 0x00, 0x00, 0x00, 0x91,
                0x07, 0x00, 0x00, 0x00, 0x10, 0x91, 0xa2, 0xc0,
            ],
        );
        image.put(0x8c, 0x9100, &[0xb7, 0x88, 0x01, 0x01, 0xef, 0x21]);
        image.put(0x8c, 0x9110, &[0x49, 0x88]);
        image
    }

    #[test]
    fn the_scene_is_read_where_its_code_holds_it() {
        let image = scene();
        let scene = import(Lorom::new(&image.0)).unwrap();
        assert_eq!((scene.setup.plane.x, scene.setup.plane.y), (-16, 16));
        assert_eq!((scene.setup.angle.0, scene.setup.zoom.get()), (0x40, 0x300));
        assert_eq!(scene.setup.speed.0, -256);
        assert_eq!((scene.turn.plane.x, scene.turn.angle.0), (1, 0x80));
        let station = scene.turn.sprites[Drifter::Station].unwrap();
        assert_eq!((station.place.x, station.drift.0), (0x11, 0x100));
        assert_eq!(station.wrap, SpriteWrap::Wraps);
        let vortex = scene.turn.sprites[Drifter::Vortex].unwrap();
        assert_eq!((vortex.drift.0, vortex.wrap), (-0x200, SpriteWrap::RunsOn));
        assert_eq!(scene.label.len(), 2, "the Japanese step is skipped");
        let Keystroke::Letter(letter) = scene.label[0].keystroke else {
            panic!("the first step types a letter");
        };
        assert_eq!(
            (letter.letter, letter.place.column, letter.place.row),
            ('A', 3, 4)
        );
        assert_eq!(scene.label[0].hold.get(), 5);
        assert_eq!(scene.label[1].keystroke, Keystroke::Pause);
    }

    #[test]
    fn other_code_where_a_placement_is_read_is_refused() {
        let mut image = scene();
        image.put(0x8b, 0xbe35, &[0xea]);
        assert!(matches!(
            import(Lorom::new(&image.0)),
            Err(ImportError::Code(_))
        ));
    }

    #[test]
    fn other_code_after_a_drift_is_refused() {
        let mut image = scene();
        image.put(0x8b, 0xbfd6, &[0xea]);
        assert!(matches!(
            import(Lorom::new(&image.0)),
            Err(ImportError::Code(_))
        ));
    }

    #[test]
    fn a_step_that_holds_for_no_frames_is_refused() {
        let mut image = scene();
        image.put(0x8c, 0x9000, &[0x00, 0x00]);
        assert_eq!(import(Lorom::new(&image.0)), Err(ImportError::Hold));
    }
}
