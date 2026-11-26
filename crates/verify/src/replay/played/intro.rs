//! The intro's production replay and the recorded State and frame publications it compares.
//!
//! The opening starts at the options handoff. The last pages and flashback scopes admit the
//! intro's State once from a finished cut. Later records supply the movie pad and the platform's
//! work completion and audio queue reply. The indexed flashback also borrows the encounter's
//! explosion hold completion. The power-on run selects integrated production.
//!
//! Finished cuts compare stage, brightness, scripts, glows, click turn and the colours owned by
//! the intro. The scoped frame report adds the caret, clicks, ordered music requests and the
//! flashback's encounter, layout, text and palette publications. An interrupted cut compares
//! brightness without advancing the intro. The last-pages scope also compares equipment restoration.

pub(in crate::replay) mod delivery;
pub(in crate::replay) mod indexed;

use super::platform::{Controller, finished, polled};
use super::{Report, Stop, rows::Rows, word};
use crate::replay::compare::Observation;
use crate::replay::readers::ExpectedCut;
use crate::replay::{data_failure, result::Outcome};
use crate::source::{self, WramByte, WramWord};
use core::num::Wrapping;
use data::rom::Rom;
use game::production::played::intro::{self, FightInput, FrameEnd};
use import::intro::{ListPlaces, Places};
use oracle_record::FrameRecord;
use screens_intro::{
    Equipment, FightReply, FlashbackRequest, IntroContent, IntroEvent, Language, MusicRequest,
    PictureLayout, Report as Frame, Rules, Shows, Step, TextTiles, TrackReply,
};
use std::path::Path;
use types::intro::{
    Baby, BabyScript, Blanking, Brightness, BrightnessLevel, Caret, CaretScript, ClickTurn,
    ColourSet, EyesScript, FadeFrames, FadingColour, FadingColours, FightVisibility, Glow, Glows,
    IntroStage, IntroState, PageScript, Pan, Progress, ScriptPlace, StageWait, StepAt, TextPlace,
};
use types::{Colour, Pair, Palette};

/// The streams of the intro's opening and page 1 no owner here produces, each with its writer.
pub const OPENING_MASKED: &[&str] = &[
    "VRAM: the intro's tiles, the opening's words, page 1's letters and the eyes (E2)",
    "CGRAM: the opening's and page 1's colours, the intro's palettes as its setup loads them (content through T1's load)",
    "OAM: the caret (E1)",
    "the PPU mirrors the stages write: the layers and scrolls (E3, E4)",
    "what the intro's first frame sets outside it: her State and her drawing, the equipment lent and the button bindings, the area, the room's size, the door, layer 1's place, the line program and the colour effects (their owners', none in the loop before the battle on Zebes)",
];

/// The streams of the intro's last pages no owner here produces, each with its writer.
pub const MASKED: &[&str] = &[
    "VRAM: the scene's tiles, page 5's, and the eyes' and the letters' tile words (E2)",
    "OAM: the baby and the caret (E1)",
    "the colour blend's bands while a dissolve runs, the HDMA object (E3)",
    "the PPU mirrors the stages write: the layers and colour math (E3)",
    "during flight: cycling colours, mode 7 art, sprite pictures, label tile attributes and the random draw during lag",
    "during arrival: the cartridge write and intermediate room setup before HUD publication",
];

/// What an intro run holds from its seed: the intro's State, the language and the pad's latch.
const SEED_HELD: &[&str] = &[
    "from inside the last pages, the intro's State as the seed's work RAM holds it: the stage, the dissolve's count, the colours' levels and steps and the colours shown, the picture's height, the baby, the page, the eyes, the caret, the glows, the click turn and the brightness; as the intro begins, only that it begins",
    "the language, from the options' setting",
    "the pad as the game last latched it, which the movie's presses follow",
];

const LAST_PAGES_HELD: &[&str] = &[
    "the last pages' stage, scripts, colours, brightness, and options language",
    "the pad latch, both frame counts and the random sequence",
    "Samus's health, the player's saved item bindings, shot charge, sound occupancy and room quake",
];

/// What an intro run borrows from the record (decision 35).
const BORROWED: &[&str] = &[
    "lag: whether each cut's frame of work finished",
    "audio: whether a track waits in the queue of tracks as the frame of work begins",
];

const FLASHBACK_BORROWED: &[&str] = &[
    "lag: whether each cut's frame of work finished",
    "audio: whether a track waits in the queue of tracks as the frame of work begins",
    "Zebes encounter: the independently located expiry of its explosion hold",
];

const FLASHBACK_MASKED: &[&str] = &[
    "Zebes room play: demo, collision, movement, equipment, weapons and encounter",
    "the encounter's shake, palette effects, renderers' VRAM, OAM and CGRAM and layer mirrors",
];

/// The boss reaches the end of its explosion hold and switches to its return callback.
fn fight_reply(began: &FrameRecord, current: &FrameRecord) -> Result<FightReply, Outcome> {
    const CALLBACK: WramWord = wram("ZEBES_BOSS_CALLBACK", 0x1b3d);
    const ELAPSED: WramWord = wram("ZEBES_BOSS_EXPLOSION_HOLD", 0x1b7d);
    let ready = word(&began.wram, CALLBACK)? == 0xb80f
        && word(&began.wram, ELAPSED)? == 127
        && word(&current.wram, CALLBACK)? == 0xb82e
        && word(&current.wram, ELAPSED)? == 128;
    Ok(if ready {
        FightReply::ReadyToReturn
    } else {
        FightReply::Fighting
    })
}

/// The rules the run plays: the game as it ships.
const RULES: Rules = Rules::REFERENCE;

/// The game state of the intro, and the cinematic function of each of its stages.
const INTRO: u16 = 0x1e;
const BEGINS: u16 = 0xa395;
const TO_SCENE: u16 = 0xb2d2;
/// The function of the picture shown and of a page typed: which is told by the part the run
/// plays and whether page 5 has begun.
const SHOWN: u16 = 0xa391;
const STAGES: [(u16, IntroStage); 23] = [
    (BEGINS, IntroStage::Begins),
    (0xa5a7, IntroStage::OpeningLoads),
    (0xa5bd, IntroStage::OpeningBrightens),
    (0xa5f8, IntroStage::OpeningPause),
    (0xa613, IntroStage::FirstTune),
    (0xa639, IntroStage::SecondTuneLoads),
    (0xa64c, IntroStage::SecondTune),
    (0xa663, IntroStage::OpeningDarkens),
    (0xa66f, IntroStage::PageOneMadeReady),
    (0xa82b, IntroStage::PageOneLoads),
    (0xa84a, IntroStage::PageOneBrightens),
    (0xaeb8, IntroStage::PageOneWaits),
    (0xb250, IntroStage::IntoFight),
    (0xb35f, IntroStage::PageTwoPending),
    (0xb3f4, IntroStage::OutOfFight),
    (0xaf6c, IntroStage::PageTwoWaits),
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
/// The boss flash can overwrite sprite palette 7 after the intro publishes it.
const BOSS_FLASH_TIMER: WramWord = wram("ZEBES_BOSS_FLASH_TIMER", 0x1add);
/// Whether the fight is drawn in front of the cinematic sprites.
const FIGHT_VISIBILITY: WramWord = wram("INTRO_FIGHT_VISIBILITY", 0x1a57);
/// The picture's height in the last pages.
const HEIGHT: WramWord = wram("CINEMATIC_BG1_Y", 0x1997);
/// The baby, the sprite object in slot 14: its y, list, timer and loop count.
const BABY_X: WramWord = wram("CINEMATIC_SPRITE_X", 0x1a99);
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
/// The slow fades' count, a byte, which they reload from the fade's count.
const SLOW_FADE_COUNT: WramByte = WramByte {
    name: "SCREEN_FADE_DELAY",
    offset: 0x0723,
};
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

/// Which part of the intro a run plays: the opening and page 1, or the last pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Part {
    Opening,
    Flashback,
    Delivery,
    LastPages,
}

/// The part of the intro a seed admits a run to: the intro's first frame still to come, or the
/// dissolve from page 4's text to the scene, before page 5 has begun, so the record's stages read
/// right as the run plays on. None for any other seed.
pub(super) fn starts(wram: &[u8]) -> Result<Option<Part>, Outcome> {
    if word(wram, source::GAME_STATE)? != INTRO {
        return Ok(None);
    }
    Ok(match word(wram, source::CINEMATIC_ACTIVE)? {
        BEGINS => Some(Part::Opening),
        TO_SCENE => Some(Part::LastPages),
        _ => None,
    })
}

/// What a run reads the intro from: the scenes' content and the record's reader.
#[derive(Clone)]
pub(super) struct Scenes {
    pub(super) content: IntroContent,
    reading: Reading,
}

impl Scenes {
    /// The scenes imported from the player's ROM at `rom`, read for `part`.
    pub(super) fn import(rom: &Path, part: Part) -> Result<Self, Outcome> {
        let bad = || data_failure(None, "played ROM image");
        let image = Rom::new(std::fs::read(rom).map_err(|_| bad())?).map_err(|_| bad())?;
        let scene = import::intro::import(image.lorom())
            .map_err(|_| data_failure(None, "played intro scene"))?;
        let places = import::intro::list_places(image.lorom())
            .map_err(|_| data_failure(None, "played intro lists"))?;
        let content = intro::content(&scene);
        Ok(Self {
            reading: Reading {
                places,
                repeat_times: repeat_times(&content),
                part,
                page_five_begun: false,
                fight_returned: false,
                previous_flashback: None,
            },
            content,
        })
    }

    #[cfg(test)]
    pub(super) fn page_two_seed(&mut self, record: &FrameRecord) -> Result<intro::State, Outcome> {
        let seed = self.reading.state(&record.wram)?;
        if seed.stage != IntroStage::PageTwoWaits || !finished(record) {
            return Err(data_failure(None, "discovery seed after page two"));
        }
        intro::admit(Language::English, seed, shown(&record.wram)?)
            .map_err(|_| data_failure(None, "discovery intro seed"))
    }

    /// The intro's State fields at `current`, recorded and played, for a run that plays on into
    /// the intro from power-on: those of the State alone, as the front end reports no frame of
    /// the intro's.
    pub(super) fn compared(
        &mut self,
        current: &FrameRecord,
        state: &intro::State,
    ) -> Result<(Vec<Observation>, Vec<Observation>), Outcome> {
        if !finished(current) || state.intro.stage == IntroStage::Begins {
            return not_begun(current, state);
        }
        self.reading.part = delivery::next_part(self.reading.part, &current.wram)?;
        let recorded = self.reading.state(&current.wram)?;
        let expected = observations(&View {
            part: self.reading.part,
            state: &recorded,
            colours: &shown(&current.wram)?,
        });
        let actual = observations(&View {
            part: self.reading.part,
            state: &state.intro,
            colours: &state.colours,
        });
        Ok((expected, actual))
    }
}

/// The fields of a cut whose record is inside a frame's work, or where the intro has not yet
/// begun: the brightness, and the stage at a finished cut.
fn not_begun(
    current: &FrameRecord,
    state: &intro::State,
) -> Result<(Vec<Observation>, Vec<Observation>), Outcome> {
    let mut expected = brightness(brightness_of(&current.wram)?);
    let mut actual = brightness(state.intro.brightness);
    if finished(current) {
        let function = word(&current.wram, source::CINEMATIC_ACTIVE)?;
        let recorded = STAGES
            .iter()
            .find(|(at, _)| *at == function)
            .map_or(-1, |&(_, stage)| stage_code(stage));
        expected.push(Observation::quantity("intro.stage", recorded));
        actual.push(Observation::quantity(
            "intro.stage",
            stage_code(state.intro.stage),
        ));
    }
    Ok((expected, actual))
}

/// Play `part` of the intro from the seed at `start` through `end`.
pub(super) fn play(
    rows: &mut Rows<'_>,
    rom: &Path,
    seed: FrameRecord,
    part: Part,
    row: &str,
    start: u64,
    end: u64,
) -> Result<Report, Outcome> {
    let Scenes {
        content,
        mut reading,
    } = Scenes::import(rom, part)?;
    let report = |held, stop| Report {
        row: row.to_owned(),
        start,
        held,
        stop,
        seed_held: if part == Part::LastPages {
            LAST_PAGES_HELD
        } else {
            SEED_HELD
        },
        borrowed: match part {
            Part::Flashback => FLASHBACK_BORROWED,
            Part::LastPages => super::flight::BORROWED,
            Part::Opening | Part::Delivery => BORROWED,
        },
        masked: match part {
            Part::Opening => OPENING_MASKED,
            Part::Flashback | Part::Delivery => FLASHBACK_MASKED,
            Part::LastPages => MASKED,
        },
    };
    let admitted = word(&seed.wram, LANGUAGE).and_then(|language| {
        let language = match language {
            0 => Language::English,
            _ => Language::Japanese,
        };
        let japanese = |_| Outcome::Unsupported {
            cut: None,
            dependency: "the intro's Japanese pages".into(),
        };
        match part {
            Part::Opening => intro::begins(language).map_err(japanese),
            Part::Flashback | Part::Delivery | Part::LastPages => {
                let state = reading.state(&seed.wram)?;
                intro::admit(language, state, shown(&seed.wram)?).map_err(japanese)
            }
        }
    });
    let state = match admitted {
        Ok(state) => state,
        Err(outcome) => return Ok(report(0, Stop::At(super::at(start, outcome)))),
    };
    let controller = match Controller::seeded(&seed.wram) {
        Ok(controller) => controller,
        Err(outcome) => return Ok(report(0, Stop::At(super::at(start, outcome)))),
    };
    let run = Run {
        state,
        content,
        reading,
    };
    let (held, stop) = run.through(
        rows,
        rom,
        row,
        seed,
        start.saturating_add(1)..=end,
        (part, controller),
    )?;
    Ok(report(held, stop))
}

impl Run {
    fn through(
        mut self,
        rows: &mut Rows<'_>,
        rom: &Path,
        row: &str,
        seed: FrameRecord,
        cuts: core::ops::RangeInclusive<u64>,
        (part, mut controller): (Part, Controller),
    ) -> Result<(u64, Stop), Outcome> {
        let mut held = 0;
        let mut last_pages = if part == Part::LastPages {
            Some((
                super::frame::seed(&seed.wram)?,
                super::start::Carried::last_pages(&seed.wram)?,
            ))
        } else {
            None
        };
        let mut previous = seed.clone();
        let mut began = seed;
        let end = *cuts.end();
        for cut in cuts {
            let current = rows.record(cut)?;
            if current.video_frame != previous.video_frame.wrapping_add(1) {
                return Err(data_failure(Some(cut), "played rows join"));
            }
            if let Err(outcome) = self.step(controller, &began, &previous, &current, cut) {
                return Ok((held, Stop::At(super::at(cut, outcome))));
            }
            controller = controller.read(current.pad.0, polled(&current));
            if let Some((frame, carry)) = &mut last_pages {
                if super::flight::starts(&previous.wram)? {
                    return Ok((
                        held,
                        Stop::At(Outcome::Difference {
                            cut,
                            field: "flight.active".into(),
                        }),
                    ));
                }
                *frame = frame.after_intro_frame(finished(&current));
                let mut expected = super::frame::expected(&current.wram)?.to_vec();
                let mut actual = super::frame::actual(*frame).to_vec();
                if finished(&current) {
                    expected.push(super::frame::random_observation(super::frame::random(
                        &current.wram,
                    )?));
                    actual.push(super::frame::random_observation(frame.random));
                }
                if let Err(outcome) = crate::replay::compared(cut, &expected, &actual) {
                    return Ok((held, Stop::At(outcome)));
                }
                if cut < end
                    && let Some(journey) =
                        game::production::played::flight::Journey::after_intro(&self.state, *frame)
                {
                    let checkpoint = Departure {
                        intro: self.state,
                        frame: *frame,
                        controller,
                        carry: *carry,
                    };
                    let next = checkpoint.continue_flight(journey, cut)?;
                    let (more, stop) =
                        next.through(rows, rom, row, current, cut.saturating_add(1)..=end, None)?;
                    return Ok((held.saturating_add(1).saturating_add(more), stop));
                }
            }
            held = held.saturating_add(1);
            if finished(&current) {
                began = current.clone();
            }
            previous = current;
        }
        Ok((held, Stop::End))
    }
}

/// All the owners of the scoped run at the last intro cut. The next work starts the flight.
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Departure {
    pub(super) intro: intro::State,
    pub(super) frame: game::production::played::frame::State,
    pub(super) controller: Controller,
    pub(super) carry: super::start::Carried,
}

impl Departure {
    pub(super) fn continue_flight(
        self,
        journey: game::production::played::flight::Journey,
        cut: u64,
    ) -> Result<super::flight::Continuation, Outcome> {
        let restored = super::flight::restored(&self, cut, "intro departure checkpoint")?;
        Ok(super::flight::Continuation {
            journey: game::production::played::flight::Journey::after_intro(
                &restored.intro,
                restored.frame,
            )
            .filter(|after| *after == journey)
            .ok_or(Outcome::CheckpointFailure { cut })?,
            controller: restored.controller,
            carry: restored.carry,
            video: None,
        })
    }
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
        let part = self.reading.part;
        let frame = frame_input(part, controller, began, previous, current)?;
        let report =
            intro::played(&mut self.state, frame, &self.content, RULES).map_err(|fault| {
                Outcome::Unsupported {
                    cut: Some(cut),
                    dependency: format!("played frame {fault:?}"),
                }
            })?;
        let expected = self.reading.expected(began, current, &self.content)?;
        let (mut actual, outputs) = actual(part, &self.state, report.as_ref());
        actual.extend(outputs);
        crate::replay::compared(cut, &expected.fields, &actual)
    }
}

/// Decode only the inputs the intro reads during this interval.
fn frame_input(
    part: Part,
    controller: Controller,
    began: &FrameRecord,
    previous: &FrameRecord,
    current: &FrameRecord,
) -> Result<FrameEnd, Outcome> {
    // The movie's pad is the input; the game's own latch only checks its alignment.
    if polled(previous) && word(&previous.wram, source::JOYPAD1_LASTKEYS)? != controller.held {
        return Err(data_failure(None, "played pad alignment"));
    }
    if !finished(current) {
        return Ok(FrameEnd::Lagged);
    }
    Ok(FrameEnd::Finished {
        pressed: controller.pressed != 0,
        fight: if part == Part::Flashback {
            FightInput::Scoped(fight_reply(began, current)?)
        } else {
            FightInput::Integrated
        },
        track: if super::music_ready(&began.wram)? {
            TrackReply::NoneWaits
        } else {
            TrackReply::TrackWaits
        },
    })
}

fn actual(
    part: Part,
    state: &intro::State,
    report: Option<&Frame>,
) -> (Vec<Observation>, Vec<Observation>) {
    let Some(report) = report else {
        let semantics = brightness(state.intro.brightness);
        let outputs = if part == Part::Flashback {
            vec![Observation::quantity("intro.frame-finished", 0)]
        } else {
            Vec::new()
        };
        return (semantics, outputs);
    };
    let semantics = observations(&View {
        part,
        state: &state.intro,
        colours: &state.colours,
    });
    let outputs = Reported {
        caret_shown: report.caret.is_some(),
        clicks: count(report, IntroEvent::Click),
        music: report.music.iter().copied().map(music_code).collect(),
        returned: count(report, IntroEvent::EquipmentSet(Equipment::RETURNED)) > 0,
    };
    let mut outputs = outputs.observations();
    if part == Part::Flashback {
        outputs.extend([
            Observation::quantity("intro.frame-finished", 1),
            event_observation(&report.events),
            Observation::quantity(
                "intro.palette-published",
                i64::from(report.colours.is_some()),
            ),
        ]);
        if let Some(palette) = &report.colours {
            outputs.extend([
                published_palette(palette, false),
                published_palette(palette, true),
            ]);
        }
    }
    (semantics, outputs)
}

fn event_observation(events: &[IntroEvent]) -> Observation {
    use FlashbackRequest::{HideFightSamusAndShots, StartZebesFight};
    use PictureLayout::{PlainFight, PlainText, SplitBlend};
    Observation {
        field: "intro.events".into(),
        bytes: events
            .iter()
            .filter_map(|event| {
                Some(match event {
                    IntroEvent::Flashback(StartZebesFight) => 1,
                    IntroEvent::Layout(SplitBlend) => 2,
                    IntroEvent::TextCleared => 3,
                    IntroEvent::Layout(PlainFight) => 4,
                    IntroEvent::FlashingLightsBegin => 5,
                    IntroEvent::TextUpdates(TextTiles::Paused) => 6,
                    IntroEvent::TextUpdates(TextTiles::Resumed) => 7,
                    IntroEvent::Flashback(HideFightSamusAndShots) => 8,
                    IntroEvent::Layout(PlainText) => 9,
                    IntroEvent::Discovery(screens_intro::DiscoveryRequest::StartDiscovery) => 10,
                    IntroEvent::Discovery(screens_intro::DiscoveryRequest::FinishDiscovery) => 11,
                    IntroEvent::PictureSelected(screens_intro::ScenePicture::Discovery) => 12,
                    IntroEvent::PictureSelected(screens_intro::ScenePicture::Delivery) => 13,
                    IntroEvent::PictureSelected(screens_intro::ScenePicture::Examination) => 14,
                    IntroEvent::Layout(PictureLayout::PlainScene) => 15,
                    IntroEvent::Click | IntroEvent::EquipmentSet(_) => return None,
                })
            })
            .collect(),
    }
}

/// The palette mirror has a separate owner for the last 16 entries when the boss flashes.
fn published_palette(palette: &Palette, sprite_seven: bool) -> Observation {
    let (field, range) = if sprite_seven {
        ("intro.palette-sprite7", 240..256)
    } else {
        ("intro.palette", 0..240)
    };
    Observation {
        field: field.into(),
        bytes: palette
            .0
            .get(range)
            .unwrap_or_default()
            .iter()
            .flat_map(|colour| colour.bits().to_le_bytes())
            .collect(),
    }
}

/// How many of the frame's events are this one.
fn count(report: &Frame, event: IntroEvent) -> i64 {
    i64::try_from(report.events.iter().filter(|&&of| of == event).count()).unwrap_or(i64::MAX)
}

/// The baby's repeat: the times its stretch plays in all, which the game's count starts at.
fn repeat_times(content: &IntroContent) -> Option<u16> {
    content.baby[BabyScript::Examination]
        .iter()
        .find_map(|step| match step {
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

/// The record's reader: where the lists stand, the baby's repeat, and the part the run plays
/// and whether page 5 has begun, which tell the picture shown from the page typed.
#[derive(Clone)]
struct Reading {
    places: Places,
    repeat_times: Option<u16>,
    part: Part,
    page_five_begun: bool,
    fight_returned: bool,
    previous_flashback: Option<(IntroStage, u16)>,
}

impl Reading {
    fn expected(
        &mut self,
        began: &FrameRecord,
        current: &FrameRecord,
        content: &IntroContent,
    ) -> Result<ExpectedCut, Outcome> {
        let (semantics, outputs) = if finished(current) {
            let prior = self.previous_flashback;
            let state = self.state(&current.wram)?;
            let colours = shown(&current.wram)?;
            let semantics = observations(&View {
                part: self.part,
                state: &state,
                colours: &colours,
            });
            let mut outputs = Reported {
                caret_shown: caret_shown(&state, content),
                clicks: i64::from(clicks(began, current)?),
                music: music_requests(began, current)?,
                returned: returned(began, current)?,
            }
            .observations();
            if self.part == Part::Flashback {
                outputs.extend(flashback_outputs(prior, &state, &colours, began)?);
            }
            (semantics, outputs)
        } else {
            let semantics = brightness(brightness_of(&current.wram)?);
            let outputs = if self.part == Part::Flashback {
                vec![Observation::quantity("intro.frame-finished", 0)]
            } else {
                Vec::new()
            };
            (semantics, outputs)
        };
        Ok(ExpectedCut {
            fields: semantics.iter().chain(&outputs).cloned().collect(),
            semantic_count: semantics.len(),
            output_count: outputs.len(),
            semantics,
            replies: 0,
        })
    }

    /// The intro's State as the record's work RAM holds it.
    fn state(&mut self, wram: &[u8]) -> Result<IntroState, Outcome> {
        let stage = if self.part == Part::Delivery {
            delivery::stage(wram)?
        } else {
            self.stage(wram)?
        };
        self.state_at(wram, stage)
    }

    fn stage(&self, wram: &[u8]) -> Result<IntroStage, Outcome> {
        let function = word(wram, source::CINEMATIC_ACTIVE)?;
        let stage = if function == SHOWN {
            if self.part == Part::Flashback && !self.fight_returned {
                IntroStage::Fight
            } else if self.part != Part::LastPages || self.page_five_begun {
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
        Ok(stage)
    }

    fn state_at(&mut self, wram: &[u8], stage: IntroStage) -> Result<IntroState, Outcome> {
        if matches!(stage, IntroStage::ToPageFive | IntroStage::ToText) {
            self.page_five_begun = true;
        }
        self.fight_returned |= matches!(
            stage,
            IntroStage::PageTwoPending | IntroStage::OutOfFight | IntroStage::PageTwoWaits
        );
        let dissolve = word(
            wram,
            if (self.part == Part::Delivery && delivery::return_count(stage))
                || matches!(
                    stage,
                    IntroStage::ToText | IntroStage::OutOfFight | IntroStage::PageTwoWaits
                )
                || stage == IntroStage::Text && self.fight_returned
            {
                TO_TEXT_COUNT
            } else {
                TO_SCENE_COUNT
            },
        )?;
        if matches!(self.part, Part::Flashback | Part::Delivery) {
            self.previous_flashback = Some((stage, dissolve));
        }
        // The stage's wait is the dissolve's count's word, read only while the opening holds.
        let wait = if holding(stage) {
            StageWait::new(dissolve).ok_or_else(|| data_failure(None, "played intro wait"))?
        } else {
            StageWait::ZERO
        };
        let fade = if slow_fading(stage) {
            u16::from(
                SLOW_FADE_COUNT
                    .read(wram)
                    .map_err(|_| data_failure(None, SLOW_FADE_COUNT.name))?,
            )
        } else {
            word(wram, FADE_COUNT)?
        };
        Ok(IntroState {
            stage,
            wait,
            colours: colours(wram)?,
            dissolve,
            pan: if self.part == Part::Delivery {
                delivery::pan(wram)?
            } else {
                Pan::AT_REST
            },
            height: if self.part == Part::Flashback {
                0
            } else {
                word(wram, HEIGHT)?.cast_signed()
            },
            fight: if self.part == Part::Flashback && word(wram, FIGHT_VISIBILITY)? != 0 {
                FightVisibility::InFront
            } else {
                FightVisibility::Hidden
            },
            baby: match self.part {
                Part::Flashback => None,
                Part::Delivery => delivery::baby(self, wram, stage)?,
                Part::Opening | Part::LastPages => self.baby(wram)?,
            },
            page: script(
                wram,
                [PAGE_LIST, PAGE_TIMER],
                &self.places.pages,
                [
                    PageScript::One,
                    PageScript::Two,
                    PageScript::Three,
                    PageScript::Four,
                    PageScript::Five,
                    PageScript::Six,
                ],
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
            fade: FadeFrames::new(fade).ok_or_else(|| data_failure(None, "played intro fade"))?,
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
            place: Pair {
                x: Wrapping(word(wram, BABY_X)?),
                y: Wrapping(word(wram, BABY_Y)?),
            },
            script: ScriptPlace {
                script: BabyScript::Examination,
                progress: Progress {
                    step: step_at(&self.places.baby, list)?,
                    hold: word(wram, BABY_TIMER)?,
                    repeat_left: (count != 0 && Some(count) != self.repeat_times).then_some(count),
                },
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
fn script<S: Copy, const N: usize>(
    wram: &[u8],
    [list, timer]: [WramWord; 2],
    places: &[ListPlaces; N],
    scripts: [S; N],
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

/// The music's requests the frame asked for: the entries the music queue took since it began.
fn music_requests(began: &FrameRecord, current: &FrameRecord) -> Result<Vec<u16>, Outcome> {
    const QUEUE: WramWord = wram("MUSIC_QUEUE", 0x0619);
    let first = word(&began.wram, MUSIC_QUEUE_NEXT)? / 2;
    let last = word(&current.wram, MUSIC_QUEUE_NEXT)? / 2;
    let length = last.wrapping_sub(first) & MUSIC_ENTRIES;
    (0..length)
        .map(|at| table(&current.wram, QUEUE, first.wrapping_add(at) & MUSIC_ENTRIES))
        .collect()
}

fn music_code(request: MusicRequest) -> u16 {
    use game::production::played::front_end::Sound;
    super::screens::sound_code(Sound::IntroMusic(request))
        .map_or(0, |[_, low, high]| u16::from_le_bytes([low, high]))
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
    part: Part,
    state: &'a IntroState,
    colours: &'a Palette,
}

/// What a frame of the intro reported: whether the caret shows, the clicks, the music's
/// requests and whether the equipment was returned.
struct Reported {
    caret_shown: bool,
    clicks: i64,
    music: Vec<u16>,
    returned: bool,
}

impl Reported {
    fn observations(self) -> Vec<Observation> {
        vec![
            Observation::quantity("intro.caret.shown", i64::from(self.caret_shown)),
            Observation::quantity("intro.clicks", self.clicks),
            Observation {
                field: "intro.music-requests".into(),
                bytes: self.music.into_iter().flat_map(u16::to_le_bytes).collect(),
            },
            Observation::quantity("intro.equipment-returned", i64::from(self.returned)),
        ]
    }
}

/// The documented scene transitions select these requests. Palette ownership is taken
/// from the independently observed boss flash timer at the beginning of work.
fn flashback_outputs(
    prior: Option<(IntroStage, u16)>,
    state: &IntroState,
    colours: &Palette,
    began: &FrameRecord,
) -> Result<Vec<Observation>, Outcome> {
    use IntroStage::{Fight, IntoFight, OutOfFight, PageOneWaits, PageTwoPending, Text};
    use PictureLayout::{PlainFight, PlainText, SplitBlend};
    let Some((before, count)) = prior else {
        return Err(data_failure(None, "flashback preceding finished stage"));
    };
    let events: &[IntroEvent] = match (before, state.stage) {
        (PageOneWaits, IntoFight) => &[
            IntroEvent::Flashback(FlashbackRequest::StartZebesFight),
            IntroEvent::Layout(SplitBlend),
        ],
        (IntoFight, Fight) => &[
            IntroEvent::TextCleared,
            IntroEvent::Layout(PlainFight),
            IntroEvent::FlashingLightsBegin,
        ],
        (Fight, PageTwoPending) => &[IntroEvent::TextUpdates(TextTiles::Paused)],
        (PageTwoPending, OutOfFight) => &[
            IntroEvent::Layout(SplitBlend),
            IntroEvent::TextUpdates(TextTiles::Resumed),
        ],
        (OutOfFight, OutOfFight) if count == 1 => &[IntroEvent::Flashback(
            FlashbackRequest::HideFightSamusAndShots,
        )],
        (OutOfFight, Text) => &[IntroEvent::Layout(PlainText)],
        _ => &[],
    };
    let publishes = matches!(
        (before, state.stage),
        (PageOneWaits, IntoFight) | (PageTwoPending, OutOfFight)
    ) || matches!(before, IntoFight | OutOfFight) && count % 4 == 0;
    let mut outputs = vec![
        Observation::quantity("intro.frame-finished", 1),
        event_observation(events),
        Observation::quantity("intro.palette-published", i64::from(publishes)),
    ];
    if publishes {
        outputs.push(published_palette(colours, false));
        if before != IntoFight || word(&began.wram, BOSS_FLASH_TIMER)? == 0 {
            outputs.push(published_palette(colours, true));
        }
    }
    Ok(outputs)
}

/// The stages the opening holds in, counting the stage's wait down.
const fn holding(stage: IntroStage) -> bool {
    matches!(
        stage,
        IntroStage::OpeningPause | IntroStage::FirstTune | IntroStage::SecondTune
    )
}

/// The stages of the slow fades.
const fn slow_fading(stage: IntroStage) -> bool {
    matches!(
        stage,
        IntroStage::OpeningBrightens | IntroStage::OpeningDarkens | IntroStage::PageOneBrightens
    )
}

/// Whether the dissolve's count is compared: while a dissolve runs, and from page 1's making
/// ready on, where it holds the count the next part's dissolve starts from.
const fn dissolve_compared(part: Part, stage: IntroStage) -> bool {
    match stage {
        IntroStage::ToScene
        | IntroStage::ToText
        | IntroStage::PageOneLoads
        | IntroStage::PageOneBrightens
        | IntroStage::PageOneWaits
        | IntroStage::IntoFight
        | IntroStage::Fight
        | IntroStage::PageTwoPending
        | IntroStage::OutOfFight
        | IntroStage::PageTwoWaits
        | IntroStage::IntoDiscovery
        | IntroStage::Discovery
        | IntroStage::PageThreePending
        | IntroStage::OutOfDiscovery
        | IntroStage::PageThreeWaits
        | IntroStage::IntoDelivery
        | IntroStage::Delivery
        | IntroStage::PageFourPending
        | IntroStage::OutOfDelivery
        | IntroStage::PageFourWaits => true,
        IntroStage::Text => !matches!(part, Part::LastPages),
        IntroStage::Begins
        | IntroStage::OpeningLoads
        | IntroStage::OpeningBrightens
        | IntroStage::OpeningPause
        | IntroStage::FirstTune
        | IntroStage::SecondTuneLoads
        | IntroStage::SecondTune
        | IntroStage::OpeningDarkens
        | IntroStage::PageOneMadeReady
        | IntroStage::Scene
        | IntroStage::ToPageFive
        | IntroStage::WaitingForPress
        | IntroStage::FadingOut
        | IntroStage::Done => false,
    }
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
        IntroStage::Begins => 10,
        IntroStage::OpeningLoads => 11,
        IntroStage::OpeningBrightens => 12,
        IntroStage::OpeningPause => 13,
        IntroStage::FirstTune => 14,
        IntroStage::SecondTuneLoads => 15,
        IntroStage::SecondTune => 16,
        IntroStage::OpeningDarkens => 17,
        IntroStage::PageOneMadeReady => 18,
        IntroStage::PageOneLoads => 19,
        IntroStage::PageOneBrightens => 20,
        IntroStage::PageOneWaits => 21,
        IntroStage::ToScene => 0,
        IntroStage::Scene => 1,
        IntroStage::ToPageFive => 2,
        IntroStage::ToText => 3,
        IntroStage::Text => 4,
        IntroStage::WaitingForPress => 5,
        IntroStage::FadingOut => 6,
        IntroStage::Done => 7,
        IntroStage::IntoFight => 22,
        IntroStage::Fight => 23,
        IntroStage::PageTwoPending => 24,
        IntroStage::OutOfFight => 25,
        IntroStage::PageTwoWaits => 26,
        IntroStage::IntoDiscovery => 27,
        IntroStage::Discovery => 28,
        IntroStage::PageThreePending => 29,
        IntroStage::OutOfDiscovery => 30,
        IntroStage::PageThreeWaits => 31,
        IntroStage::IntoDelivery => 32,
        IntroStage::Delivery => 33,
        IntroStage::PageFourPending => 34,
        IntroStage::OutOfDelivery => 35,
        IntroStage::PageFourWaits => 36,
    }
}

/// Each script's place: the baby's, the page's, the eyes' and the caret's. Which script runs is
/// compared as a number, a label for the comparison alone.
fn scripts(state: &IntroState) -> Vec<Observation> {
    let mut fields = Vec::new();
    fields.extend(place(
        "baby",
        state.baby.map(|baby| {
            (
                i64::from(baby.script.script == BabyScript::Delivery),
                baby.script.progress,
            )
        }),
    ));
    fields.extend(place(
        "page",
        state.page.map(|page| {
            let which = match page.script {
                PageScript::One => 1,
                PageScript::Two => 2,
                PageScript::Three => 3,
                PageScript::Four => 4,
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
    let mut fields = vec![
        quantity("intro.stage", stage_code(state.stage)),
        quantity(
            "intro.wait",
            if holding(state.stage) {
                i64::from(state.wait.frames())
            } else {
                -1
            },
        ),
        quantity(
            "intro.dissolve",
            if dissolve_compared(view.part, state.stage) {
                i64::from(state.dissolve)
            } else {
                -1
            },
        ),
    ];
    // The opening's and page 1's colours are the setup's load, which no owner here produces.
    if view.part != Part::Opening
        && !(view.part == Part::Flashback && state.stage == IntroStage::PageOneWaits)
    {
        fields.push(Observation {
            field: "intro.colours".to_owned(),
            bytes: state
                .colours
                .0
                .iter()
                .flat_map(|colour| colour.levels.into_iter().chain(colour.steps))
                .flat_map(u16::to_le_bytes)
                .collect(),
        });
    }
    if view.part == Part::LastPages {
        fields.push(Observation {
            field: "intro.colours-shown".to_owned(),
            bytes: view
                .colours
                .0
                .iter()
                .flat_map(|colour| colour.bits().to_le_bytes())
                .collect(),
        });
    }
    if view.part == Part::Flashback {
        fields.push(quantity(
            "intro.fight-visible",
            i64::from(state.fight == FightVisibility::InFront),
        ));
    } else {
        fields.push(quantity("intro.height", i64::from(state.height)));
    }
    fields.push(quantity(
        "intro.baby.y",
        state.baby.map_or(-1, |baby| i64::from(baby.place.y.0)),
    ));
    if view.part == Part::Delivery {
        fields.extend([
            quantity("intro.pan", i64::from(u16::from(state.pan))),
            quantity(
                "intro.baby.x",
                state.baby.map_or(-1, |baby| i64::from(baby.place.x.0)),
            ),
        ]);
    }
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
    ]);
    fields.push(glows_observed(&state.glows));
    fields.push(quantity(
        "intro.click-turn",
        i64::from(state.click == ClickTurn::On),
    ));
    fields.extend(brightness(state.brightness));
    fields.push(quantity(
        "intro.fade-count",
        if state.stage == IntroStage::FadingOut || slow_fading(state.stage) {
            i64::from(state.fade.frames())
        } else {
            -1
        },
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

    /// The intro's opening and page 1 from the intro's first frame to page 1's wait: every cut
    /// holds, as the reviewed run in scenarios/replay/played.toml says.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn the_intros_opening_played_alone() {
        let path = |name| std::env::var_os(name).unwrap().into();
        let sources = crate::replay::Sources {
            cache: path("SM_CACHE"),
            rom: path("SM_ROM"),
            lock: path("SM_LOCK"),
            ..Default::default()
        };
        let report = super::super::play(&sources, "any-intro-0568", 0, 1650).unwrap();
        diagnostic(format_args!(
            "PLAYED_ALONE row={} start={} held={} stop={:?}",
            report.row, report.start, report.held, report.stop
        ));
        assert_eq!((report.held, report.stop), (1650, super::Stop::End));
    }
}
