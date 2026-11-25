//! The importer of the intro's scenes: the only code that knows where the original keeps the
//! pages, Samus's eyes, the baby, the caret and the last scene's colours.
//!
//! Each picture script is read from the original's instruction list for it, which the importer
//! turns into steps: a list holding an instruction the intro doesn't use is refused rather than
//! read. Apart from the scene, `list_places` gives where each step's list stands in the original
//! once the step has begun, which only the verifier reads.

use data::assets::intro::{
    Colour, Cue, DissolveSets, Palette, Picture, Scene, Shows, Step, StepAt, TextPlace,
};
use data::rom::{AddressError, Lorom, RomAddress};

/// Why the intro's scene cannot be imported. Errors carry no content bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    /// A list, a drawing or a colour is outside the image.
    Address(AddressError),
    /// A list holds an instruction the intro doesn't use.
    Instruction(RomAddress),
    /// A step draws something the intro doesn't show.
    Drawing(RomAddress),
    /// A list goes to a step it hasn't reached, or repeats without a count set ahead of it.
    Loop(RomAddress),
    /// A list runs on past any script's length.
    Unended,
    /// A colour has its sixteenth bit set.
    Colour,
}

impl From<AddressError> for ImportError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// The banks of the sprite objects (the baby, the caret) and of the text objects (the pages,
/// the eyes), each holding its objects' lists and drawings.
const SPRITES: u8 = 0x8b;
const TEXT: u8 = 0x8c;

/// The objects whose headers name their lists, `LIST_AT` bytes in.
const PAGE_ONE: u16 = 0xcf3f;
const PAGE_FIVE: u16 = 0xcf57;
const PAGE_SIX: u16 = 0xcf5d;
const EYES: u16 = 0xcf63;
const BABY: u16 = 0xce67;
const CARET: u16 = 0xce6d;
const LIST_AT: usize = 4;
/// The lists the scene switches to: the eyes' from page 6, the caret's while a page waits.
const EYES_STARING: u16 = 0xd613;
const CARET_BLINKING: u16 = 0xcc03;

/// The scene's colours at their full, and the caret's, with the index of its first.
const FULL_COLOURS: u16 = 0xe3e9;
const CARET_COLOURS: u16 = 0xe5a9;
const CARET_COLOUR_COUNT: u8 = 7;
const FIRST_CARET_COLOUR: u8 = 0xe0;
/// The caret's home, px.
const CARET_HOME: [u16; 2] = [8, 0x18];

/// The dissolves' colours by index range: to the scene, the text's colours dim and the
/// picture's brighten; to the text the other way, with BG palette 1's first four left lit.
const TEXT_COLOURS: [core::ops::Range<u8>; 3] = [0..16, 48..64, 233..239];
const TEXT_AND_FRAME: [core::ops::Range<u8>; 3] = [0..20, 48..64, 233..239];
const PICTURE_COLOURS: [core::ops::Range<u8>; 2] = [32..48, 224..233];

/// The most steps a script may have before its list counts as unended.
const MAX_STEPS: usize = 256;
/// An instruction's mark: its word has the top bit set.
const INSTRUCTION: u16 = 0x8000;

/// The text objects' instructions.
const PAGE_ONE_BEGINS: u16 = 0xae43;
const PAGE_ONE_ENDS: u16 = 0xae5b;
const JAPANESE_PAGE_FIVE: u16 = 0xb19b;
const JAPANESE_PAGE_SIX: u16 = 0xb228;
const CARET_TO_BLINK: u16 = 0xadd4;
const WAIT_FOR_PRESS: u16 = 0xb1b3;
const FINISH: u16 = 0xb240;
const DELETE: u16 = 0x9698;
const TEXT_GO_TO: u16 = 0x971e;
/// A text step's drawings: nothing, a letter, or a block of tiles (the eyes).
const DRAW_NOTHING: u16 = 0x8849;
const DRAW_LETTER: u16 = 0x884d;
const DRAW_BLOCK: u16 = 0x88fd;
const EYES_DRAWINGS: [(u16, Picture); 4] = [
    (0xd781, Picture::EyesOpen),
    (0xd791, Picture::EyesHalfOpen),
    (0xd7a1, Picture::EyesClosed),
    (0xd7b1, Picture::EyesStaring),
];

/// The sprite objects' instructions.
const SET_COUNT: u16 = 0x94d6;
const COUNT_DOWN_AND_GO_TO: u16 = 0x94c3;
const START_PAGE_FIVE: u16 = 0xb34e;
const SPRITE_GO_TO: u16 = 0x94bc;
/// A sprite step's pictures; a step drawing none shows nothing.
const NO_SPRITE: u16 = 0;
const SPRITE_PICTURES: [(u16, Picture); 4] = [
    (0x8d53, Picture::BabyFirst),
    (0x8d5a, Picture::BabySecond),
    (0x8d61, Picture::BabyThird),
    (0x8d68, Picture::Caret),
];

/// The letters' font: tile `FONT_FIRST + n` is the `n`th of `FONT`, and `SPACE_TILE` a space.
const FONT_FIRST: u16 = 0x2000;
const FONT: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789.,.':";
const EXCLAMATION_TILE: u16 = 0x202a;
const SPACE_TILE: u16 = 0x002f;

/// Where a script's list stands in the original: at its start, and after each step that shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListPlaces {
    start: u16,
    after: Vec<(u16, usize)>,
}

impl ListPlaces {
    /// The step a list standing at `pointer` is at, if it is one of this script's.
    #[must_use]
    pub fn step_at(&self, pointer: u16) -> Option<StepAt> {
        if pointer == self.start {
            return Some(StepAt::Start);
        }
        self.after
            .iter()
            .find(|(after, _)| *after == pointer)
            .map(|&(_, step)| StepAt::Step(step))
    }
}

/// Where each of the scene's scripts' lists stands in the original.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Places {
    /// Page 1's, page 5's and page 6's.
    pub pages: [ListPlaces; 3],
    /// The eyes' blinking and staring.
    pub eyes: [ListPlaces; 2],
    /// The baby's.
    pub baby: ListPlaces,
    /// The caret's steady and blinking.
    pub caret: [ListPlaces; 2],
}

/// The scenes' eight scripts' lists, in the order `import` and `list_places` name them: page 1,
/// page 5, page 6, the eyes blinking and staring, the baby, the caret steady and blinking.
fn lists(rom: Lorom<'_>) -> Result<[(Vec<Step>, ListPlaces); 8], ImportError> {
    let header = |bank, object| -> Result<RomAddress, ImportError> {
        Ok(at(bank, word(rom, at(SPRITES, object), LIST_AT)?))
    };
    Ok([
        script(rom, header(TEXT, PAGE_ONE)?, List::Text)?,
        script(rom, header(TEXT, PAGE_FIVE)?, List::Text)?,
        script(rom, header(TEXT, PAGE_SIX)?, List::Text)?,
        script(rom, header(TEXT, EYES)?, List::Text)?,
        script(rom, at(TEXT, EYES_STARING), List::Text)?,
        script(rom, header(SPRITES, BABY)?, List::Sprite)?,
        script(rom, header(SPRITES, CARET)?, List::Sprite)?,
        script(rom, at(SPRITES, CARET_BLINKING), List::Sprite)?,
    ])
}

/// Where the scene's scripts' lists stand in the original, for the verifier to read a record's
/// script steps by.
///
/// # Errors
/// As [`import`]'s, for the lists.
pub fn list_places(rom: Lorom<'_>) -> Result<Places, ImportError> {
    let [
        page_one,
        page_five,
        page_six,
        blinking,
        staring,
        baby,
        steady,
        caret_blinking,
    ] = lists(rom)?.map(|(_, places)| places);
    Ok(Places {
        pages: [page_one, page_five, page_six],
        eyes: [blinking, staring],
        baby,
        caret: [steady, caret_blinking],
    })
}

/// The intro's scenes, with the pages in English.
///
/// # Errors
/// A list, drawing or colour outside the image, an instruction or a drawing the intro doesn't
/// use, a list that loops forward or runs on, and a colour over 15 bits.
pub fn import(rom: Lorom<'_>) -> Result<Scene, ImportError> {
    let [
        page_one,
        page_five,
        page_six,
        eyes_blinking,
        eyes_staring,
        baby,
        caret_steady,
        caret_blinking,
    ] = lists(rom)?.map(|(steps, _)| steps);
    let mut full = Palette([Colour::BLACK; 256]);
    for (colour, index) in full.0.iter_mut().zip(0_usize..) {
        *colour = colour_at(rom, FULL_COLOURS, index)?;
    }
    let mut caret_colours = Vec::new();
    for n in 0..CARET_COLOUR_COUNT {
        caret_colours.push((
            FIRST_CARET_COLOUR.wrapping_add(n),
            colour_at(rom, CARET_COLOURS, usize::from(n))?,
        ));
    }
    let indices = |ranges: &[core::ops::Range<u8>]| -> Vec<u8> {
        ranges.iter().flat_map(Clone::clone).collect()
    };
    let scene = Scene {
        page_one,
        page_five,
        page_six,
        eyes_blinking,
        eyes_staring,
        baby,
        caret_steady,
        caret_blinking,
        full,
        to_scene: DissolveSets {
            dims: indices(&TEXT_AND_FRAME),
            brightens: indices(&PICTURE_COLOURS),
        },
        to_text: DissolveSets {
            dims: indices(&PICTURE_COLOURS),
            brightens: indices(&TEXT_COLOURS),
        },
        text: indices(&TEXT_COLOURS),
        caret_colours,
        caret_home: CARET_HOME,
    };
    Ok(scene)
}

/// The two kinds of list: a text object's, whose steps carry a place and a drawing, and a sprite
/// object's, whose steps carry a picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum List {
    Text,
    Sprite,
}

/// A list's steps from `start` to its end or its last go to, and where it stands after each
/// step that shows.
fn script(
    rom: Lorom<'_>,
    start: RomAddress,
    list: List,
) -> Result<(Vec<Step>, ListPlaces), ImportError> {
    let mut steps = Vec::new();
    let mut addresses = Vec::new();
    let mut after = Vec::new();
    let mut count: Option<(u16, u16)> = None;
    let mut here = start;
    let index_of = |addresses: &[u16], target: u16| -> Result<usize, ImportError> {
        addresses
            .iter()
            .position(|&address| address == target)
            .ok_or_else(|| ImportError::Loop(at(start.bank, target)))
    };
    while steps.len() < MAX_STEPS {
        let first = word(rom, here, 0)?;
        let (step, length) = if first & INSTRUCTION == 0 {
            let length = match list {
                List::Text => 6,
                List::Sprite => 4,
            };
            let shows = match list {
                List::Text => text_drawing(rom, here)?,
                List::Sprite => sprite_picture(rom, here)?,
            };
            let step = Step::Show { shows, hold: first };
            after.push((offset_after(here, length)?, steps.len()));
            (Some(step), length)
        } else {
            match (list, first) {
                // Page 1's first instruction turns the click off, which placing a page does.
                (List::Text, PAGE_ONE_BEGINS | JAPANESE_PAGE_FIVE | JAPANESE_PAGE_SIX) => (None, 2),
                (List::Text, PAGE_ONE_ENDS) => (Some(Step::Cue(Cue::WaitForPressThenFight)), 2),
                (List::Text, CARET_TO_BLINK) => (Some(Step::Cue(Cue::CaretBlinks)), 2),
                (List::Text, WAIT_FOR_PRESS) => (Some(Step::Cue(Cue::WaitForPress)), 2),
                (List::Text, FINISH) => (Some(Step::Cue(Cue::Finish)), 2),
                (List::Text, DELETE) => (Some(Step::End), 2),
                (List::Text, TEXT_GO_TO) | (List::Sprite, SPRITE_GO_TO) => {
                    let to = index_of(&addresses, word(rom, here, 2)?)?;
                    (Some(Step::Back(to)), 4)
                }
                (List::Sprite, SET_COUNT) => {
                    count = Some((here.offset, word(rom, here, 2)?));
                    (None, 4)
                }
                (List::Sprite, COUNT_DOWN_AND_GO_TO) => {
                    let target = word(rom, here, 2)?;
                    // The count must be set ahead of the stretch it counts, once.
                    let times = match count {
                        Some((set_at, times)) if set_at < target => times,
                        _ => return Err(ImportError::Loop(here)),
                    };
                    let from = index_of(&addresses, target)?;
                    (Some(Step::Repeat { from, times }), 4)
                }
                (List::Sprite, START_PAGE_FIVE) => (Some(Step::Cue(Cue::PageFive)), 2),
                _ => return Err(ImportError::Instruction(here)),
            }
        };
        let ends = matches!(step, Some(Step::End | Step::Back(_)));
        if let Some(step) = step {
            addresses.push(here.offset);
            steps.push(step);
        }
        if ends {
            let places = ListPlaces {
                start: start.offset,
                after,
            };
            return Ok((steps, places));
        }
        here = on(here, length)?;
    }
    Err(ImportError::Unended)
}

/// What a text step at `step` shows: nothing, a letter at the step's place, or the eyes.
fn text_drawing(rom: Lorom<'_>, step: RomAddress) -> Result<Shows, ImportError> {
    let [column, row] = word(rom, step, 2)?.to_le_bytes();
    let drawing = at(TEXT, word(rom, step, 4)?);
    match word(rom, drawing, 0)? {
        DRAW_NOTHING => Ok(Shows::Nothing),
        DRAW_LETTER => Ok(Shows::Letter {
            glyph: glyph(word(rom, drawing, 4)?).ok_or(ImportError::Drawing(drawing))?,
            at: TextPlace {
                column: column.into(),
                row: row.into(),
            },
        }),
        DRAW_BLOCK => EYES_DRAWINGS
            .iter()
            .find(|(of, _)| *of == drawing.offset)
            .map(|&(_, picture)| Shows::Picture(picture))
            .ok_or(ImportError::Drawing(drawing)),
        _ => Err(ImportError::Drawing(drawing)),
    }
}

/// What a sprite step at `step` shows: nothing, or one of the scene's sprite pictures.
fn sprite_picture(rom: Lorom<'_>, step: RomAddress) -> Result<Shows, ImportError> {
    let picture = word(rom, step, 2)?;
    if picture == NO_SPRITE {
        return Ok(Shows::Nothing);
    }
    SPRITE_PICTURES
        .iter()
        .find(|(of, _)| *of == picture)
        .map(|&(_, picture)| Shows::Picture(picture))
        .ok_or_else(|| ImportError::Drawing(at(SPRITES, picture)))
}

/// The letter a tile of the font shows.
fn glyph(tile: u16) -> Option<char> {
    match tile {
        SPACE_TILE => Some(' '),
        EXCLAMATION_TILE => Some('!'),
        _ => FONT.chars().nth(usize::from(tile.checked_sub(FONT_FIRST)?)),
    }
}

/// The `index`th colour of the table at `table` in the text bank.
fn colour_at(rom: Lorom<'_>, table: u16, index: usize) -> Result<Colour, ImportError> {
    Colour::new(word(rom, at(TEXT, table), index.saturating_mul(2))?).ok_or(ImportError::Colour)
}

const fn at(bank: u8, offset: u16) -> RomAddress {
    RomAddress::new(bank, offset)
}

fn on(place: RomAddress, count: usize) -> Result<RomAddress, ImportError> {
    place.advance(count).ok_or(ImportError::Unended)
}

/// The offset in its bank just after the `length` bytes at `place`.
fn offset_after(place: RomAddress, length: usize) -> Result<u16, ImportError> {
    Ok(on(place, length)?.offset)
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
// bank $8B log. The text objects `$8B:CF3F` (page 1, whose list begins with `$AE43`, the click
// turn off and the Japanese page, and ends with `$AE5B`, the caret blinking and the wait for the
// fight's press), `$8B:CF57` (page 5), `$CF5D` (page 6) and `$CF63` (Samus
// blinking), whose lists are in bank 8C; `$8B:B4BC` switches the eyes to `$8C:D613` on page 6.
// The sprite objects `$8B:CE67` (the baby being examined) and `$CE6D` (the caret); `$8B:ADD4`
// sets the caret to `$CC03`, and `$8B:ADEE` places it at (8, 18h). A text step is (timer,
// column, row, drawing); a drawing is `$8B:8849` (nothing), `$884D` with a font tile, or
// `$88FD` with a block of tiles (the eyes, `$8C:D781`..`$D7B1`). A sprite step is (timer,
// spritemap), `$8B:94D6` setting the count that `$94C3` counts down and goes to, `$B34E`
// starting page 5 and `$94BC` going to. The colours: `$8B:B3A1` loads `$8C:E3E9` and clears
// BG palettes 0, 3 and sprite palette 6 colours 9-Eh; `$8B:B2D2` dims BG palettes 0, 1 (0-3),
// 3 and sprite palette 6 (9-Eh) and brightens BG palette 2 and sprite palette 6 (0-8);
// `$8B:B458` the reverse, BG palette 0 without BG palette 1's, and then sets sprite palette 6
// colours 0-6 to `$8C:E5A9`.

#[cfg(test)]
mod tests {
    use super::{ImportError, import};
    use data::assets::intro::{Cue, Picture, Shows, Step, StepAt, TextPlace};
    use data::rom::{Lorom, RomAddress};

    /// An image holding an invented scene where the importer reads one.
    struct Image(Vec<u8>);

    impl Image {
        fn put(&mut self, bank: u8, offset: u16, words: &[u16]) {
            let at = Lorom::new(&self.0)
                .file_offset(RomAddress::new(bank, offset))
                .unwrap();
            let bytes = words.iter().flat_map(|word| word.to_le_bytes());
            self.0[at..]
                .iter_mut()
                .zip(bytes)
                .for_each(|(to, from)| *to = from);
        }
    }

    /// Every list one picture held and gone back to, but page 5's and the baby's, which the
    /// tests read; the letters' and the pictures' drawings.
    fn scene() -> Image {
        let mut image = Image(vec![0; 0x68000]);
        // Each object's header names its list four bytes in.
        for (header, list) in [
            (0xcf43, 0x9200),
            (0xcf5b, 0x9000),
            (0xcf61, 0x9100),
            (0xcf67, 0x9100),
            (0xce6b, 0x9000),
            (0xce71, 0x9100),
        ] {
            image.put(0x8b, header, &[list]);
        }
        // Page 5: the Japanese page skipped, nothing, "B" at (3, 4), the wait, the end.
        image.put(
            0x8c,
            0x9000,
            &[0xb19b, 1, 0x0101, 0xd683, 5, 0x0403, 0xd68b, 0xb1b3, 0x9698],
        );
        image.put(0x8c, 0xd683, &[0x8849]);
        // Page 1: its first instruction, nothing, the wait for the fight, the end.
        image.put(0x8c, 0x9200, &[0xae43, 1, 0x0101, 0xd683, 0xae5b, 0x9698]);
        image.put(0x8c, 0xd68b, &[0x884d, 0x0101, 0x2001]);
        image.put(0x8c, 0x9100, &[0x80, 0x0d11, 0xd781, 0x971e, 0x9100]);
        image.put(0x8c, 0xd613, &[0x40, 0x0d11, 0xd781, 0x971e, 0xd613]);
        image.put(0x8c, 0xd781, &[0x88fd]);
        // The baby: a count of 3, two pictures repeated, page 5's cue, one picture forever.
        image.put(
            0x8b,
            0x9000,
            &[
                0x94d6, 3, 10, 0x8d53, 10, 0x8d5a, 0x94c3, 0x9004, 0xb34e, 10, 0x8d61, 0x94bc,
                0x9012,
            ],
        );
        image.put(0x8b, 0x9100, &[5, 0x8d68, 5, 0, 0x94bc, 0x9100]);
        image.put(0x8b, 0xcc03, &[5, 0x8d68, 0x94bc, 0xcc03]);
        image
    }

    #[test]
    fn the_scripts_are_read_from_their_lists() {
        let image = scene();
        let scene = import(Lorom::new(&image.0)).unwrap();
        let places = super::list_places(Lorom::new(&image.0)).unwrap();
        assert_eq!(
            scene.page_five,
            [
                Step::Show {
                    shows: Shows::Nothing,
                    hold: 1
                },
                Step::Show {
                    shows: Shows::Letter {
                        glyph: 'B',
                        at: TextPlace { column: 3, row: 4 }
                    },
                    hold: 5
                },
                Step::Cue(Cue::WaitForPress),
                Step::End,
            ]
        );
        assert_eq!(scene.baby.get(2), Some(&Step::Repeat { from: 0, times: 3 }));
        assert_eq!(scene.baby.get(3), Some(&Step::Cue(Cue::PageFive)));
        assert_eq!(scene.baby.get(5), Some(&Step::Back(4)));
        assert_eq!(
            scene.caret_steady.get(1),
            Some(&Step::Show {
                shows: Shows::Nothing,
                hold: 5
            })
        );
        assert_eq!(
            scene.eyes_staring.first(),
            Some(&Step::Show {
                shows: Shows::Picture(Picture::EyesOpen),
                hold: 0x40
            })
        );
        assert_eq!(
            scene.page_one,
            [
                Step::Show {
                    shows: Shows::Nothing,
                    hold: 1
                },
                Step::Cue(Cue::WaitForPressThenFight),
                Step::End,
            ]
        );
        let [page_one, page_five, _] = &places.pages;
        assert_eq!(page_one.step_at(0x9208), Some(StepAt::Step(0)));
        assert_eq!(page_five.step_at(0x9000), Some(StepAt::Start));
        assert_eq!(page_five.step_at(0x9008), Some(StepAt::Step(0)));
        assert_eq!(page_five.step_at(0x900e), Some(StepAt::Step(1)));
        assert_eq!(places.baby.step_at(0x9016), Some(StepAt::Step(4)));
        assert_eq!(scene.text.len(), 38);
    }

    #[test]
    fn an_instruction_the_intro_does_not_use_is_refused() {
        let mut image = scene();
        image.put(0x8c, 0x9000, &[0xb19c]);
        assert!(matches!(
            import(Lorom::new(&image.0)),
            Err(ImportError::Instruction(_))
        ));
    }

    #[test]
    fn a_repeat_with_no_count_set_ahead_is_refused() {
        let mut image = scene();
        image.put(0x8b, 0x9000, &[0x94c3]);
        assert!(matches!(
            import(Lorom::new(&image.0)),
            Err(ImportError::Loop(_))
        ));
    }
}
