//! A run from inside the intro's last pages: the intro alone, compared at each cut.
//!
//! The run admits the intro's State from its seed, a record of the intro's own work RAM: where
//! the scene has got to, its colours, the baby, the page, the eyes, the caret, the glows, the
//! click turn and the brightness. It reads nothing of room play. The scene's content is
//! imported from the player's ROM, and the options' language is read from the seed. Each cut
//! borrows whether its frame of work finished ([`BORROWED`]); the press is the movie's.
//!
//! At every finished cut it compares: the stage, the dissolve's count while one runs, the
//! colours' levels and steps and the colours shown, the picture's height, each script's name,
//! step, hold and repeat count (the page, the eyes, the baby, the caret), the caret's place and
//! whether it shows, the glows, the click turn, the clicks against the sound queue, the music's
//! stop against the music queue, the brightness, the fade's count while it fades, and whether
//! the equipment was returned. At a lag cut the record is inside the frame's work, so only the
//! brightness is compared. A run is admitted only at the dissolve to the scene, the stage the
//! spec admits, which tells the page typed from the picture shown as the run plays on.
//! [`MASKED`] names what no owner here produces, which is not compared.

use super::platform::{Controller, finished, polled};
use super::{Report, Stop, rows::Rows, word};
use crate::replay::compare::{CompareFault, Observation, compare};
use crate::replay::{data_failure, result::Outcome};
use crate::source::{self, WramByte, WramWord};
use core::num::Wrapping;
use data::rom::Rom;
use game::production::played::intro::{self, FrameEnd};
use import::intro::{ListPlaces, Places};
use oracle_record::FrameRecord;
use screens_intro::{IntroContent, IntroEvent, Language, Report as Frame, Rules, Shows, Step};
use std::path::Path;
use types::intro::{
    Baby, Blanking, Brightness, BrightnessLevel, Caret, CaretScript, ClickTurn, ColourSet,
    EyesScript, FadeFrames, FadingColour, FadingColours, Glow, Glows, IntroStage, IntroState,
    PageScript, Progress, ScriptPlace, StepAt, TextPlace,
};
use types::{Colour, Pair, Palette};

/// The streams of the intro's frames no owner here produces, each with its writer.
pub const MASKED: &[&str] = &[
    "VRAM: the scene's tiles, page 5's, and the eyes' and the letters' tile words (E2)",
    "OAM: the baby and the caret (E1)",
    "the colour blend's bands while a dissolve runs, the HDMA object (E3)",
    "the PPU mirrors the stages write: the layers and colour math (E3)",
];

/// What an intro run holds from its seed: the intro's State, the language and the pad's latch.
const SEED_HELD: &[&str] = &[
    "the intro's State as the seed's work RAM holds it: the stage, the dissolve's count, the colours' levels and steps and the colours shown, the picture's height, the baby, the page, the eyes, the caret, the glows, the click turn and the brightness",
    "the language, from the options' setting",
    "the pad as the game last latched it, which the movie's presses follow",
];

/// What an intro run borrows from the record (decision 35).
const BORROWED: &[&str] = &["lag: whether each cut's frame of work finished"];

/// The rules the run plays: the game as it ships.
const RULES: Rules = Rules::REFERENCE;

/// The game state of the intro, and the cinematic function of each of its last pages' stages.
const INTRO: u16 = 0x1e;
const TO_SCENE: u16 = 0xb2d2;
/// The function of both the picture shown and page 5 typed: which is told by whether page 5
/// has begun.
const SHOWN: u16 = 0xa391;
const STAGES: [(u16, IntroStage); 7] = [
    (TO_SCENE, IntroStage::ToScene),
    (0xb392, IntroStage::ToPageFive),
    (0xb458, IntroStage::ToText),
    (0xb1da, IntroStage::WaitingForPress),
    (0xa390, IntroStage::Text),
    (0xb72f, IntroStage::FadingOut),
    (0xbca0, IntroStage::Done),
];

/// The options' language: 0 for English.
const LANGUAGE: WramWord = wram("JAPANESE_TEXT", 0x09e2);
/// The dissolves' counts, to the scene and to the text.
const TO_SCENE_COUNT: WramWord = wram("CINEMATIC_FUNCTION_TIMER", 0x1a49);
const TO_TEXT_COUNT: WramWord = wram("INTRO_CROSS_FADE_TIMER", 0x1a4b);
/// The fading colours' levels and steps, red, green and blue, and the colours shown.
const LEVELS: [WramWord; 3] = [
    wram("INTRO_RED_LEVELS", 0x2400),
    wram("INTRO_GREEN_LEVELS", 0x2600),
    wram("INTRO_BLUE_LEVELS", 0x2800),
];
const STEPS: [WramWord; 3] = [
    wram("INTRO_RED_STEPS", 0x2a00),
    wram("INTRO_GREEN_STEPS", 0x2c00),
    wram("INTRO_BLUE_STEPS", 0x2e00),
];
const SHOWN_COLOURS: WramWord = wram("PALETTE_BUFFER", 0xc000);
/// The picture's height.
const HEIGHT: WramWord = wram("CINEMATIC_BG1_Y", 0x1997);
/// The baby, the sprite object in slot 14: its y, list, timer and loop count.
const BABY_Y: WramWord = wram("CINEMATIC_SPRITE_Y", 0x1ab9);
const BABY_LIST: WramWord = wram("CINEMATIC_SPRITE_LIST", 0x1b39);
const BABY_TIMER: WramWord = wram("CINEMATIC_SPRITE_TIMER", 0x1b79);
const BABY_COUNT: WramWord = wram("CINEMATIC_SPRITE_LOOP", 0x1b99);
/// The caret, the sprite object in slot 15: its x, y, list and timer.
const CARET_X: WramWord = wram("CINEMATIC_SPRITE_X", 0x1a9b);
const CARET_Y: WramWord = wram("CINEMATIC_SPRITE_Y", 0x1abb);
const CARET_LIST: WramWord = wram("CINEMATIC_SPRITE_LIST", 0x1b3b);
const CARET_TIMER: WramWord = wram("CINEMATIC_SPRITE_TIMER", 0x1b7b);
/// The page and the eyes, the BG objects in slots 0 and 1: their lists and timers.
const PAGE_LIST: WramWord = wram("CINEMATIC_BG_LIST", 0x19d1);
const PAGE_TIMER: WramWord = wram("CINEMATIC_BG_TIMER", 0x19e1);
const EYES_LIST: WramWord = wram("CINEMATIC_BG_LIST", 0x19d3);
const EYES_TIMER: WramWord = wram("CINEMATIC_BG_TIMER", 0x19e3);
/// The glows' slots: each one's pointer, timer, column, row and next colour set's palette bits.
const GLOW_SLOTS: u16 = 8;
const GLOW_POINTER: WramWord = wram("INTRO_GLOW_POINTERS", 0x19f7);
const GLOW_TIMER: WramWord = wram("INTRO_GLOW_TIMERS", 0x1a07);
const GLOW_COLUMN: WramWord = wram("INTRO_GLOW_COLUMNS", 0x1a17);
const GLOW_ROW: WramWord = wram("INTRO_GLOW_ROWS", 0x1a27);
const GLOW_PALETTE: WramWord = wram("INTRO_GLOW_PALETTES", 0x1a37);
const GLOW_SET_SHIFT: u32 = 10;
/// The click turn.
const CLICK_TURN: WramWord = wram("INTRO_CLICK_TURN", 0x1ba1);
/// The display's mirror: its forced blank and its level; and the fade's count.
const BLANK: u8 = 0x80;
const LEVEL: u8 = 0x0f;
const FADE_COUNT: WramWord = wram("SCREEN_FADE_COUNTER", 0x0725);
/// The sound queue the clicks go to: its next index, which wraps at 16 entries.
const CLICK_QUEUE: WramByte = WramByte {
    name: "SOUND_LIB3_QUEUE_NEXT",
    offset: 0x0648,
};
const QUEUE_ENTRIES: u8 = 0x0f;
/// The music queue the music's stop goes to: its next index, two bytes an entry, eight entries.
const MUSIC_QUEUE_NEXT: WramWord = wram("MUSIC_QUEUE_NEXT", 0x0639);
const MUSIC_ENTRIES: u16 = 7;
/// The player's button bindings, which the intro's end restores from the ones it saved.
const BINDINGS: WramWord = wram("BUTTON_BINDINGS", 0x09b2);
const SAVED_BINDINGS: WramWord = wram("SAVED_BUTTON_BINDINGS", 0x0d82);
const BINDING_COUNT: u16 = 7;

const fn wram(name: &'static str, offset: u16) -> WramWord {
    WramWord { name, offset }
}

/// Whether the seed is where an intro run is admitted: the dissolve from page 4's text to the
/// scene, before page 5 has begun, so the record's stages read right as the run plays on.
pub(super) fn starts(wram: &[u8]) -> Result<bool, Outcome> {
    Ok(word(wram, source::GAME_STATE)? == INTRO
        && word(wram, source::CINEMATIC_ACTIVE)? == TO_SCENE)
}

/// Play the intro from the seed at `start` through `end`.
pub(super) fn play(
    rows: &mut Rows<'_>,
    rom: &Path,
    seed: FrameRecord,
    row: &str,
    start: u64,
    end: u64,
) -> Result<Report, Outcome> {
    let bad = || data_failure(None, "played ROM image");
    let image = Rom::new(std::fs::read(rom).map_err(|_| bad())?).map_err(|_| bad())?;
    let scene = import::intro::import(image.lorom())
        .map_err(|_| data_failure(None, "played intro scene"))?;
    let places = import::intro::list_places(image.lorom())
        .map_err(|_| data_failure(None, "played intro lists"))?;
    let content = intro::content(&scene);
    let report = |held, stop| Report {
        row: row.to_owned(),
        start,
        held,
        stop,
        seed_held: SEED_HELD,
        borrowed: BORROWED,
        masked: MASKED,
    };
    let mut reading = Reading {
        places,
        repeat_times: repeat_times(&content),
        page_five_begun: false,
    };
    let admitted = reading.state(&seed.wram).and_then(|state| {
        let language = match word(&seed.wram, LANGUAGE)? {
            0 => Language::English,
            _ => Language::Japanese,
        };
        let colours = shown(&seed.wram)?;
        intro::admit(language, state, colours).map_err(|_| Outcome::Unsupported {
            cut: None,
            dependency: "the intro's Japanese pages".into(),
        })
    });
    let state = match admitted {
        Ok(state) => state,
        Err(outcome) => return Ok(report(0, Stop::At(super::at(start, outcome)))),
    };
    let mut controller = match Controller::seeded(&seed.wram) {
        Ok(controller) => controller,
        Err(outcome) => return Ok(report(0, Stop::At(super::at(start, outcome)))),
    };
    let mut run = Run {
        state,
        content,
        reading,
    };
    let mut held = 0;
    let mut previous = seed.clone();
    let mut began = seed;
    for cut in start.saturating_add(1)..=end {
        let current = rows.record(cut)?;
        if current.video_frame != previous.video_frame.wrapping_add(1) {
            return Err(data_failure(Some(cut), "played rows join"));
        }
        if let Err(outcome) = run.step(controller, &began, &previous, &current, cut) {
            return Ok(report(held, Stop::At(super::at(cut, outcome))));
        }
        held = held.saturating_add(1);
        controller = controller.read(current.pad.0, polled(&current));
        if finished(&current) {
            began = current.clone();
        }
        previous = current;
    }
    Ok(report(held, Stop::End))
}

/// The intro's owner, its content, and the record's reader.
struct Run {
    state: intro::State,
    content: IntroContent,
    reading: Reading,
}

impl Run {
    /// Play the interval ending at `current` and compare the intro with it. `began` is the
    /// record its frame of work began after.
    fn step(
        &mut self,
        controller: Controller,
        began: &FrameRecord,
        previous: &FrameRecord,
        current: &FrameRecord,
        cut: u64,
    ) -> Result<(), Outcome> {
        // The movie's pad is the input; the game's own latch only checks its alignment.
        if polled(previous) && word(&previous.wram, source::JOYPAD1_LASTKEYS)? != controller.held {
            return Err(data_failure(None, "played pad alignment"));
        }
        let frame = if finished(current) {
            FrameEnd::Finished {
                pressed: controller.pressed != 0,
            }
        } else {
            FrameEnd::Lagged
        };
        let report =
            intro::played(&mut self.state, frame, &self.content, RULES).map_err(|fault| {
                Outcome::Unsupported {
                    cut: Some(cut),
                    dependency: format!("played frame {fault:?}"),
                }
            })?;
        let (expected, actual) = if let Some(report) = report {
            let recorded = self.reading.state(&current.wram)?;
            let expected = observations(&View {
                caret_shown: caret_shown(&recorded, &self.content),
                state: &recorded,
                colours: &shown(&current.wram)?,
                clicks: i64::from(clicks(began, current)?),
                music_stops: i64::from(music_requests(began, current)?),
                returned: returned(began, current)?,
            });
            let actual = observations(&View {
                state: &self.state.intro,
                colours: &self.state.colours,
                caret_shown: report.caret.is_some(),
                clicks: count(&report, IntroEvent::Click),
                music_stops: count(&report, IntroEvent::MusicStops),
                returned: count(&report, IntroEvent::EquipmentReturned) > 0,
            });
            (expected, actual)
        } else {
            // A lag cut: the record is inside the frame's work.
            (
                brightness(brightness_of(&current.wram)?),
                brightness(self.state.intro.brightness),
            )
        };
        compare(&expected, &actual).map_err(|fault| match fault {
            CompareFault::Difference(field) => Outcome::Difference { cut, field },
            CompareFault::EmptyExpected
            | CompareFault::DuplicateField(_)
            | CompareFault::MissingActual(_) => data_failure(Some(cut), "played comparison schema"),
        })
    }
}

/// How many of the frame's events are this one.
fn count(report: &Frame, event: IntroEvent) -> i64 {
    i64::try_from(report.events.iter().filter(|&&of| of == event).count()).unwrap_or(i64::MAX)
}

/// The baby's repeat: the times its stretch plays in all, which the game's count starts at.
fn repeat_times(content: &IntroContent) -> Option<u16> {
    content.baby.iter().find_map(|step| match step {
        Step::Repeat { times, .. } => Some(*times),
        Step::Show { .. } | Step::Back(_) | Step::Hand(_) | Step::End => None,
    })
}

/// Whether the caret's script's current step shows its picture, as the content has it.
fn caret_shown(state: &IntroState, content: &IntroContent) -> bool {
    state.caret.is_some_and(|caret| {
        let StepAt::Step(at) = caret.script.progress.step else {
            return false;
        };
        matches!(
            content.caret[caret.script.script].get(at),
            Some(Step::Show {
                shows: Shows::Image(_),
                ..
            })
        )
    })
}

/// The record's reader: where the lists stand, the baby's repeat, and whether page 5 has
/// begun, which tells the picture shown from the page typed.
struct Reading {
    places: Places,
    repeat_times: Option<u16>,
    page_five_begun: bool,
}

impl Reading {
    /// The intro's State as the record's work RAM holds it.
    fn state(&mut self, wram: &[u8]) -> Result<IntroState, Outcome> {
        let function = word(wram, source::CINEMATIC_ACTIVE)?;
        let stage = if function == SHOWN {
            if self.page_five_begun {
                IntroStage::Text
            } else {
                IntroStage::Scene
            }
        } else {
            STAGES
                .iter()
                .find(|(at, _)| *at == function)
                .map(|&(_, stage)| stage)
                .ok_or_else(|| data_failure(None, "played intro stage"))?
        };
        if matches!(stage, IntroStage::ToPageFive | IntroStage::ToText) {
            self.page_five_begun = true;
        }
        let dissolve = word(
            wram,
            if stage == IntroStage::ToText {
                TO_TEXT_COUNT
            } else {
                TO_SCENE_COUNT
            },
        )?;
        Ok(IntroState {
            stage,
            colours: colours(wram)?,
            dissolve,
            height: word(wram, HEIGHT)?.cast_signed(),
            baby: self.baby(wram)?,
            page: script(
                wram,
                [PAGE_LIST, PAGE_TIMER],
                &self.places.pages,
                [PageScript::Five, PageScript::Six],
            )?,
            eyes: script(
                wram,
                [EYES_LIST, EYES_TIMER],
                &self.places.eyes,
                [EyesScript::Blinking, EyesScript::Staring],
            )?,
            caret: self.caret(wram)?,
            glows: glows(wram)?,
            click: if word(wram, CLICK_TURN)? == 0 {
                ClickTurn::Off
            } else {
                ClickTurn::On
            },
            brightness: brightness_of(wram)?,
            fade: FadeFrames::new(word(wram, FADE_COUNT)?)
                .ok_or_else(|| data_failure(None, "played intro fade"))?,
        })
    }

    /// The baby: its count reads its times before the stretch has played once, and 0 after
    /// the repeat, where the model holds none.
    fn baby(&self, wram: &[u8]) -> Result<Option<Baby>, Outcome> {
        let list = word(wram, BABY_LIST)?;
        if list == 0 {
            return Ok(None);
        }
        let count = word(wram, BABY_COUNT)?;
        Ok(Some(Baby {
            y: Wrapping(word(wram, BABY_Y)?),
            progress: Progress {
                step: step_at(&self.places.baby, list)?,
                hold: word(wram, BABY_TIMER)?,
                repeat_left: (count != 0 && Some(count) != self.repeat_times).then_some(count),
            },
        }))
    }

    fn caret(&self, wram: &[u8]) -> Result<Option<Caret>, Outcome> {
        let Some(script) = script(
            wram,
            [CARET_LIST, CARET_TIMER],
            &self.places.caret,
            [CaretScript::Steady, CaretScript::Blinking],
        )?
        else {
            return Ok(None);
        };
        Ok(Some(Caret {
            place: Pair {
                x: Wrapping(word(wram, CARET_X)?),
                y: Wrapping(word(wram, CARET_Y)?),
            },
            script,
        }))
    }
}

/// The step a list standing at `pointer` is at.
fn step_at(places: &ListPlaces, pointer: u16) -> Result<StepAt, Outcome> {
    places
        .step_at(pointer)
        .ok_or_else(|| data_failure(None, "played intro list"))
}

/// The script an object runs, from its list and timer: which of `scripts` its list is in, and
/// where it stands. None when the object is empty.
fn script<S: Copy>(
    wram: &[u8],
    [list, timer]: [WramWord; 2],
    places: &[ListPlaces; 2],
    scripts: [S; 2],
) -> Result<Option<ScriptPlace<S>>, Outcome> {
    let pointer = word(wram, list)?;
    if pointer == 0 {
        return Ok(None);
    }
    let (script, step) = places
        .iter()
        .zip(scripts)
        .find_map(|(places, script)| places.step_at(pointer).map(|step| (script, step)))
        .ok_or_else(|| data_failure(None, "played intro list"))?;
    Ok(Some(ScriptPlace {
        script,
        progress: Progress {
            step,
            hold: word(wram, timer)?,
            repeat_left: None,
        },
    }))
}

/// The word `index` of a table of words in work RAM.
fn table(wram: &[u8], table: WramWord, index: u16) -> Result<u16, Outcome> {
    word(
        wram,
        WramWord {
            offset: table.offset.wrapping_add(index.wrapping_mul(2)),
            ..table
        },
    )
}

/// The scene's colours: each one's levels and steps.
fn colours(wram: &[u8]) -> Result<FadingColours, Outcome> {
    let mut colours = FadingColours::still_black();
    for (colour, index) in colours.0.iter_mut().zip(0_u16..) {
        let mut fading = FadingColour::STILL_BLACK;
        for ((level, step), (levels, steps)) in fading
            .levels
            .iter_mut()
            .zip(&mut fading.steps)
            .zip(LEVELS.iter().zip(STEPS))
        {
            *level = table(wram, *levels, index)?;
            *step = table(wram, steps, index)?;
        }
        *colour = fading;
    }
    Ok(colours)
}

/// The colours shown.
fn shown(wram: &[u8]) -> Result<Palette, Outcome> {
    let mut palette = Palette([Colour::BLACK; types::PALETTE_SLOTS]);
    for (colour, index) in palette.0.iter_mut().zip(0_u16..) {
        *colour = Colour::new(table(wram, SHOWN_COLOURS, index)?)
            .ok_or_else(|| data_failure(None, "played intro colours"))?;
    }
    Ok(palette)
}

/// The letters glowing.
fn glows(wram: &[u8]) -> Result<Glows, Outcome> {
    let mut glows = Vec::new();
    for slot in 0..GLOW_SLOTS {
        let of = |at: WramWord| table(wram, at, slot);
        if of(GLOW_POINTER)? == 0 {
            continue;
        }
        let set = u8::try_from(of(GLOW_PALETTE)? >> GLOW_SET_SHIFT)
            .ok()
            .and_then(ColourSet::new)
            .ok_or_else(|| data_failure(None, "played intro glow"))?;
        glows.push(Glow {
            at: TextPlace {
                column: of(GLOW_COLUMN)?,
                row: of(GLOW_ROW)?,
            },
            next: set,
            left: of(GLOW_TIMER)?,
        });
    }
    Glows::new(glows).ok_or_else(|| data_failure(None, "played intro glow"))
}

/// The display's brightness.
fn brightness_of(wram: &[u8]) -> Result<Brightness, Outcome> {
    let display = source::REG_INIDISP
        .read(wram)
        .map_err(|_| data_failure(None, "played intro display"))?;
    Ok(Brightness {
        level: BrightnessLevel::new(display & LEVEL)
            .ok_or_else(|| data_failure(None, "played intro display"))?,
        blanking: if display & BLANK == 0 {
            Blanking::Shown
        } else {
            Blanking::Blanked
        },
    })
}

/// The clicks the frame asked for: the entries the sound queue took since it began.
fn clicks(began: &FrameRecord, current: &FrameRecord) -> Result<u8, Outcome> {
    let next = |record: &FrameRecord| {
        CLICK_QUEUE
            .read(&record.wram)
            .map_err(|_| data_failure(None, CLICK_QUEUE.name))
    };
    Ok(next(current)?.wrapping_sub(next(began)?) & QUEUE_ENTRIES)
}

/// The music requests the frame asked for: the entries the music queue took since it began.
fn music_requests(began: &FrameRecord, current: &FrameRecord) -> Result<u16, Outcome> {
    let next = |record: &FrameRecord| word(&record.wram, MUSIC_QUEUE_NEXT);
    Ok((next(current)?.wrapping_sub(next(began)?) / 2) & MUSIC_ENTRIES)
}

/// Whether the frame returned the equipment lent for the played scenes: her missiles and their
/// capacity back to 0 from what she held, and the player's own bindings restored.
fn returned(began: &FrameRecord, current: &FrameRecord) -> Result<bool, Outcome> {
    let missiles = |record: &FrameRecord| -> Result<[u16; 2], Outcome> {
        Ok([
            word(&record.wram, source::SAMUS_MISSILES)?,
            word(&record.wram, source::SAMUS_MAX_MISSILES)?,
        ])
    };
    let mut restored = true;
    for binding in 0..BINDING_COUNT {
        restored &= table(&current.wram, BINDINGS, binding)?
            == table(&current.wram, SAVED_BINDINGS, binding)?;
    }
    Ok(missiles(began)? != [0, 0] && missiles(current)? == [0, 0] && restored)
}

/// The intro's compared fields, from either side.
struct View<'a> {
    state: &'a IntroState,
    colours: &'a Palette,
    caret_shown: bool,
    clicks: i64,
    music_stops: i64,
    returned: bool,
}

fn brightness(brightness: Brightness) -> Vec<Observation> {
    vec![
        Observation::quantity(
            "intro.brightness.level",
            i64::from(brightness.level.number()),
        ),
        Observation::quantity(
            "intro.brightness.blanked",
            i64::from(brightness.blanking == Blanking::Blanked),
        ),
    ]
}

/// A script's place as compared: its script, step, hold and repeat count, -1 for none.
fn place(name: &str, script: Option<(i64, Progress)>) -> [Observation; 4] {
    let field =
        |part: &str, value: i64| Observation::quantity(format!("intro.{name}.{part}"), value);
    let (which, progress) = script.map_or((-1, None), |(which, progress)| (which, Some(progress)));
    let step = progress.map_or(-1, |progress| match progress.step {
        StepAt::Start => -2,
        StepAt::Step(at) => i64::try_from(at).unwrap_or(i64::MAX),
    });
    [
        field("script", which),
        field("step", step),
        field(
            "hold",
            progress.map_or(-1, |progress| i64::from(progress.hold)),
        ),
        field(
            "repeat",
            progress
                .and_then(|progress| progress.repeat_left)
                .map_or(-1, i64::from),
        ),
    ]
}

/// The stage's number, a label for the comparison alone: both sides give it from the stage.
const fn stage_code(stage: IntroStage) -> i64 {
    match stage {
        IntroStage::ToScene => 0,
        IntroStage::Scene => 1,
        IntroStage::ToPageFive => 2,
        IntroStage::ToText => 3,
        IntroStage::Text => 4,
        IntroStage::WaitingForPress => 5,
        IntroStage::FadingOut => 6,
        IntroStage::Done => 7,
    }
}

/// Each script's place: the baby's, the page's, the eyes' and the caret's. Which script runs is
/// compared as a number, a label for the comparison alone.
fn scripts(state: &IntroState) -> Vec<Observation> {
    let mut fields = Vec::new();
    fields.extend(place("baby", state.baby.map(|baby| (0, baby.progress))));
    fields.extend(place(
        "page",
        state.page.map(|page| {
            let which = match page.script {
                PageScript::Five => 5,
                PageScript::Six => 6,
            };
            (which, page.progress)
        }),
    ));
    fields.extend(place(
        "eyes",
        state.eyes.map(|eyes| {
            let which = match eyes.script {
                EyesScript::Blinking => 0,
                EyesScript::Staring => 1,
            };
            (which, eyes.progress)
        }),
    ));
    fields.extend(place(
        "caret",
        state.caret.map(|caret| {
            let which = match caret.script.script {
                CaretScript::Steady => 0,
                CaretScript::Blinking => 1,
            };
            (which, caret.script.progress)
        }),
    ));
    fields
}

/// The glowing letters, in order of place: each one's place, next set and frames left.
fn glows_observed(glows: &Glows) -> Observation {
    let mut parts: Vec<[u16; 4]> = glows
        .as_slice()
        .iter()
        .map(|glow| {
            [
                glow.at.column,
                glow.at.row,
                u16::from(glow.next.number()),
                glow.left,
            ]
        })
        .collect();
    parts.sort_unstable();
    Observation {
        field: "intro.glows".to_owned(),
        bytes: parts
            .iter()
            .flatten()
            .flat_map(|part| part.to_le_bytes())
            .collect(),
    }
}

fn observations(view: &View<'_>) -> Vec<Observation> {
    let state = view.state;
    let quantity = Observation::quantity;
    let dissolving = matches!(state.stage, IntroStage::ToScene | IntroStage::ToText);
    let mut fields = vec![
        quantity("intro.stage", stage_code(state.stage)),
        quantity(
            "intro.dissolve",
            if dissolving {
                i64::from(state.dissolve)
            } else {
                -1
            },
        ),
        Observation {
            field: "intro.colours".to_owned(),
            bytes: state
                .colours
                .0
                .iter()
                .flat_map(|colour| colour.levels.into_iter().chain(colour.steps))
                .flat_map(u16::to_le_bytes)
                .collect(),
        },
        Observation {
            field: "intro.colours-shown".to_owned(),
            bytes: view
                .colours
                .0
                .iter()
                .flat_map(|colour| colour.bits().to_le_bytes())
                .collect(),
        },
        quantity("intro.height", i64::from(state.height)),
        quantity(
            "intro.baby.y",
            state.baby.map_or(-1, |baby| i64::from(baby.y.0)),
        ),
    ];
    fields.extend(scripts(state));
    let caret_at = state.caret.map(|caret| caret.place);
    fields.extend([
        quantity(
            "intro.caret.x",
            caret_at.map_or(-1, |place| i64::from(place.x.0)),
        ),
        quantity(
            "intro.caret.y",
            caret_at.map_or(-1, |place| i64::from(place.y.0)),
        ),
        quantity("intro.caret.shown", i64::from(view.caret_shown)),
    ]);
    fields.push(glows_observed(&state.glows));
    fields.push(quantity(
        "intro.click-turn",
        i64::from(state.click == ClickTurn::On),
    ));
    fields.push(quantity("intro.clicks", view.clicks));
    fields.push(quantity("intro.music-stops", view.music_stops));
    fields.extend(brightness(state.brightness));
    fields.push(quantity(
        "intro.fade-count",
        if state.stage == IntroStage::FadingOut {
            i64::from(state.fade.frames())
        } else {
            -1
        },
    ));
    fields.push(quantity(
        "intro.equipment-returned",
        i64::from(view.returned),
    ));
    fields
}

#[cfg(test)]
mod tests {
    fn diagnostic(args: std::fmt::Arguments<'_>) {
        use std::io::Write as _;
        writeln!(std::io::stdout().lock(), "{args}").unwrap();
    }

    /// The intro's last pages from inside page 4's turn to the row's last cut: every cut holds,
    /// as the reviewed run in scenarios/replay/played.toml says.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn the_intros_last_pages_played_alone() {
        let path = |name| std::env::var_os(name).unwrap().into();
        let sources = crate::replay::Sources {
            cache: path("SM_CACHE"),
            rom: path("SM_ROM"),
            lock: path("SM_LOCK"),
            ..Default::default()
        };
        let report = super::super::play(&sources, "any-intro-6012", 1, 1499).unwrap();
        diagnostic(format_args!(
            "PLAYED_ALONE row={} start={} held={} stop={:?}",
            report.row, report.start, report.held, report.stop
        ));
        assert_eq!((report.held, report.stop), (1498, super::Stop::End));
    }
}
