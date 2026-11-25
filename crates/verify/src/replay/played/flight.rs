//! A run from the flight to Ceres's first frame, through the new game's dark preparation.
//!
//! The run starts the flight with the scene imported from the player's ROM. It admits the frame
//! counts and random sequence once at the flight's seed and carries both into the new game's
//! preparation. [`BORROWED`] names the platform, audio and loading answers it reads.
//!
//! The comparison checks which scene is active and both counters at every cut. In flight, the
//! random number is checked at finished cuts. The flight's brightness and its ordered music
//! requests with their initial delays are checked at every cut. Once the setup is done, it
//! also checks the stage, plane, transform, scroll, flash, fade, label, sprites and letters typed.
//! In the preparation it checks the arrival's stage, brightness and random
//! number at every cut. [`MASKED`] names the outputs outside this comparison.

use super::{Report, Stop, arrival, frame, rows::Rows, word};
use crate::replay::compare::{CompareFault, Observation, compare};
use crate::replay::{data_failure, result::Outcome};
use crate::source::{self, WramWord};
use core::num::Wrapping;
use data::rom::Rom;
use enum_map::EnumMap;
use game::production::played::flight::{self, FrameEnd};
use oracle_record::FrameRecord;
use screens_flight::{FrameReport, MusicQueue, Rules};
use std::path::Path;
use types::Fixed;
use types::flight::{
    Drifter, FlightSprite, FlightStage, MusicRequest, Scene, State, TypedLetter, Visibility,
};

/// The streams outside the comparison and their owners.
pub const MASKED: &[&str] = &[
    "CGRAM: the flight's colours as the setup loads them at cut 32 (T1), and the colours that cycle from the turn on (R4)",
    "OAM and the sprites' pictures and animation (E1)",
    "VRAM's mode 7 tilemap and tiles, with the picture's swap at the turn (E4)",
    "the BG1 tilemap but for the letters the label types: its emptying at the station, and each letter's palette, priority and flips (E2)",
    "the PPU mirrors the stages write: the layers and colour math at the turn, the layers, mode and bases at the station (E4)",
    "the random number at lag cuts during flight, where the draw depends on timing (T1)",
    "the save to cartridge memory as the new game begins (P2)",
    "the preparation's working room, palettes, actors and HUD (R2, C1, H3)",
];

/// The frame owner's initial State comes from the flight's first frame.
const SEED_HELD: &[&str] = &["the frame counts and random number at the flight's start"];

/// What a flight run borrows from the record (decision 35).
const BORROWED: &[&str] = &[
    "lag: whether each cut's frame of work finished",
    "lag: whether a frame of work begins, the last one's done",
    "the start ready: the frame a new game's start is made ready (R2 and C1's loading, T1's lag)",
    "audio: whether a track waits in the queue of tracks as the frame of work begins",
];

/// The rules the run plays: the game as it ships.
const RULES: Rules = Rules::REFERENCE;

/// The game state of the flight to Ceres, and the cinematic function of its first frame.
const FLIGHT: u16 = 0x1e;
const FIRST_FRAME: u16 = 0xbca0;
/// The cinematic function while the label is typed.
const LABEL_FUNCTION: u16 = 0xa38f;

/// Audio's queue holds eight words and its write index counts bytes.
const MUSIC_QUEUE_BYTES: u16 = 16;
const MUSIC_QUEUE_MASK: u16 = MUSIC_QUEUE_BYTES - 1;
const MUSIC_QUEUE_NEXT: WramWord = wram("MUSIC_QUEUE_NEXT", 0x0639);
const MUSIC_TRACKS: WramWord = wram("MUSIC_TRACKS", 0x0619);
const MUSIC_DELAYS: WramWord = wram("MUSIC_DELAYS", 0x0629);

/// The cinematic function of each stage.
const STAGES: [(u16, Stage); 7] = [
    (FIRST_FRAME, Stage::Starting),
    (0xbde4, Stage::WaitingForMusic),
    (0xbdf9, Stage::RushingIn),
    (0xbfda, Stage::FlyingAway),
    (LABEL_FUNCTION, Stage::Label),
    (0xc0c5, Stage::FadingOut),
    (0xc100, Stage::Done),
];

/// The plane: its angle, zoom and position (integer and fraction words).
const ANGLE: WramWord = wram("MODE7_ANGLE", 0x198d);
const ZOOM: WramWord = wram("MODE7_ZOOM", 0x198f);
const X: [WramWord; 2] = [wram("BG1_X", 0x1993), wram("BG1_X_FRACTION", 0x1991)];
const Y: [WramWord; 2] = [wram("BG1_Y", 0x1997), wram("BG1_Y_FRACTION", 0x1995)];
/// The transformation's mirrors, and the scroll's.
const TRANSFORM: [WramWord; 4] = [
    wram("M7A", 0x0078),
    wram("M7B", 0x007a),
    wram("M7C", 0x007c),
    wram("M7D", 0x007e),
];
const SCROLL: [WramWord; 2] = [wram("BG1_SCROLL_X", 0x00b1), wram("BG1_SCROLL_Y", 0x00b3)];
/// The colour math backdrop's mirrors, one per channel, and its level's bits.
const FLASH: [usize; 3] = [0x0074, 0x0075, 0x0076];
const CHANNEL: u8 = 0x1f;
/// The display's mirror: its forced blank and its level.
const BLANK: u8 = 0x80;
const LEVEL: u8 = 0x0f;
/// The screen fade's counter.
const FADE_COUNT: WramWord = wram("SCREEN_FADE_COUNTER", 0x0725);
/// The label's cinematic BG object: its list pointer and its timer.
const LABEL_LIST: WramWord = wram("CINEMATIC_BG_LIST", 0x19d3);
const LABEL_TIMER: WramWord = wram("CINEMATIC_BG_TIMER", 0x19e3);

/// The cinematic sprite objects: their slots, and each field's table.
const SLOTS: usize = 16;
const SPRITE_LIST: usize = 0x1b1d;
const SPRITE_X: usize = 0x1a7d;
const SPRITE_X_FRACTION: usize = 0x1add;
const SPRITE_Y: usize = 0x1a9d;
const SPRITE_Y_FRACTION: usize = 0x1afd;
const SPRITE_SPEED: usize = 0x1b7d;
/// The slot the rushing stars take, and each drifting sprite's: the game takes the highest free
/// slot, in the order the turn places them.
const STARS_SLOT: usize = 15;
const DRIFTING_SLOTS: [(Drifter, usize); 5] = [
    (Drifter::LargeAsteroids, 15),
    (Drifter::Station, 14),
    (Drifter::SmallAsteroids, 13),
    (Drifter::Vortex, 12),
    (Drifter::DistantStars, 11),
];

/// The label's layer: its tilemap in VRAM, 32 by 32 words.
const LABEL_LAYER: core::ops::Range<usize> = 0x5c00..0x6000;
const LAYER_WIDTH: usize = 32;

const fn wram(name: &'static str, offset: u16) -> WramWord {
    WramWord { name, offset }
}

/// Where the flight has got to, as both sides name it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Starting,
    WaitingForMusic,
    RushingIn,
    FlyingAway,
    Label,
    FadingOut,
    Done,
}

impl Stage {
    const fn of(stage: FlightStage) -> Self {
        match stage {
            FlightStage::Starting => Self::Starting,
            FlightStage::WaitingForMusic => Self::WaitingForMusic,
            FlightStage::RushingIn => Self::RushingIn,
            FlightStage::FlyingAway => Self::FlyingAway,
            FlightStage::Label(_) => Self::Label,
            FlightStage::FadingOut => Self::FadingOut,
            FlightStage::Done => Self::Done,
        }
    }

    /// Its number, as compared.
    const fn code(self) -> i64 {
        match self {
            Self::Starting => 0,
            Self::WaitingForMusic => 1,
            Self::RushingIn => 2,
            Self::FlyingAway => 3,
            Self::Label => 4,
            Self::FadingOut => 5,
            Self::Done => 6,
        }
    }

    /// Whether the view has turned: the drifting sprites hold the slots from then on.
    const fn turned(self) -> bool {
        matches!(
            self,
            Self::FlyingAway | Self::Label | Self::FadingOut | Self::Done
        )
    }
}

/// Whether the seed is the flight's first frame, which a flight run starts from.
pub(super) fn starts(wram: &[u8]) -> Result<bool, Outcome> {
    Ok(word(wram, source::GAME_STATE)? == FLIGHT
        && word(wram, source::CINEMATIC_ACTIVE)? == FIRST_FRAME)
}

/// Play the flight from the seed at `start` through `end`.
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
    let scene = import::flight::import(image.lorom())
        .map_err(|_| data_failure(None, "played flight scene"))?;
    let report = |held, stop| Report {
        row: row.to_owned(),
        start,
        held,
        stop,
        seed_held: SEED_HELD,
        borrowed: BORROWED,
        masked: MASKED,
    };
    let frame = frame::seed(&seed.wram)?;
    // Admission requires the flight's initial function. Its setup draws before its first
    // lag cut, so a lagged seed in that function already includes the draw.
    let journey = if super::finished(&seed) {
        flight::Journey::start(frame)
    } else {
        flight::Journey::resume_setup(frame)
    };
    let mut run = Run {
        journey,
        report: None,
        scene,
        label_steps: 0,
    };
    let mut held = 0;
    let mut previous = seed.clone();
    let mut began = seed;
    for cut in start.saturating_add(1)..=end {
        let current = rows.record(cut)?;
        if current.video_frame != previous.video_frame.wrapping_add(1) {
            return Err(data_failure(Some(cut), "played rows join"));
        }
        if let Err(outcome) = run.step(&began, &previous, &current, cut) {
            return Ok(report(held, Stop::At(outcome)));
        }
        held = held.saturating_add(1);
        if super::finished(&current) {
            began = current.clone();
        }
        previous = current;
    }
    Ok(report(held, Stop::End))
}

/// The flight's State, what its last frame reported, its scene, and the label's steps the
/// record has begun.
struct Run {
    journey: flight::Journey,
    report: Option<FrameReport>,
    scene: Scene,
    label_steps: u64,
}

impl Run {
    /// Play the interval ending at `current` and compare the active scene. `began` is the
    /// record its frame of work began after.
    fn step(
        &mut self,
        began: &FrameRecord,
        previous: &FrameRecord,
        current: &FrameRecord,
        cut: u64,
    ) -> Result<(), Outcome> {
        let at = |outcome| super::at(cut, outcome);
        let is_flight = matches!(self.journey.stage, flight::Stage::Flight(_));
        let end = if super::finished(current) {
            FrameEnd::Finished(
                if !is_flight || super::music_ready(&began.wram).map_err(at)? {
                    MusicQueue::Clear
                } else {
                    MusicQueue::TrackWaits
                },
            )
        } else {
            FrameEnd::Lagged
        };
        let ready = if is_flight {
            game::production::played::arrival::Readiness::NotYet
        } else {
            arrival::start(&previous.wram, &current.wram).map_err(at)?
        };
        let report = self
            .journey
            .step(
                super::finished(previous),
                end,
                ready,
                &self.scene,
                RULES,
                screens_arrival::Rules::REFERENCE,
            )
            .map_err(|fault| Outcome::Unsupported {
                cut: Some(cut),
                dependency: format!("played frame {fault:?}"),
            })?;
        let recorded_flight = word(&current.wram, source::GAME_STATE).map_err(at)? == FLIGHT;
        if recorded_flight != matches!(self.journey.stage, flight::Stage::Flight(_)) {
            return Err(Outcome::Difference {
                cut,
                field: "flight.active".into(),
            });
        }
        let (mut expected, mut actual) = match self.journey.stage {
            flight::Stage::Flight(state) => self
                .compared(&state, report, previous, current)
                .map_err(at)?,
            flight::Stage::Arrival(state) => (
                arrival::expected(&current.wram, false).map_err(at)?,
                arrival::actual(state),
            ),
        };
        expected.extend(frame::expected(&current.wram).map_err(at)?);
        actual.extend(frame::actual(self.journey.frame));
        if !recorded_flight || super::finished(current) {
            expected.push(frame::random_observation(
                frame::random(&current.wram).map_err(at)?,
            ));
            actual.push(frame::random_observation(self.journey.frame.random));
        }
        compare(&expected, &actual).map_err(|fault| match fault {
            CompareFault::Difference(field) => Outcome::Difference { cut, field },
            CompareFault::EmptyExpected
            | CompareFault::DuplicateField(_)
            | CompareFault::MissingActual(_) => data_failure(Some(cut), "played flight comparison"),
        })
    }

    fn compared(
        &mut self,
        state: &State,
        report: Option<FrameReport>,
        previous: &FrameRecord,
        current: &FrameRecord,
    ) -> Result<(Vec<Observation>, Vec<Observation>), Outcome> {
        let typed = report.as_ref().and_then(|report| report.typed);
        let music = played_music(
            report
                .as_ref()
                .map_or(&[], |report| report.music.as_slice()),
        );
        if let Some(report) = report {
            self.report = Some(report);
        }
        if step_begun(previous, current)? {
            self.label_steps = self.label_steps.saturating_add(1);
        }
        let recorded = recorded(previous, current, self.label_steps)?;
        let (mut expected, mut actual) = self.report.as_ref().map_or_else(
            // Not yet made ready: the record holds the setup's working values.
            || {
                (
                    recorded.brightness(),
                    Fields::brightness_of(state).brightness(),
                )
            },
            |report| {
                (
                    recorded.observations(),
                    played(state, report, typed).observations(),
                )
            },
        );
        expected.push(music_observation(recorded_music(previous, current)?));
        actual.push(music_observation(music));
        Ok((expected, actual))
    }
}

/// Read the entries appended between cuts, with their initial delays in write order.
fn recorded_music(previous: &FrameRecord, current: &FrameRecord) -> Result<Vec<[u16; 2]>, Outcome> {
    let next = |record: &FrameRecord| {
        let next = word(&record.wram, MUSIC_QUEUE_NEXT)?;
        if next < MUSIC_QUEUE_BYTES && next.is_multiple_of(2) {
            Ok(next)
        } else {
            Err(data_failure(None, MUSIC_QUEUE_NEXT.name))
        }
    };
    let start = next(previous)?;
    let end = next(current)?;
    let bytes = end.wrapping_sub(start) & MUSIC_QUEUE_MASK;
    (0..bytes)
        .step_by(2)
        .map(|distance| {
            let entry = start.wrapping_add(distance) & MUSIC_QUEUE_MASK;
            let slot = |table: WramWord| {
                word(
                    &current.wram,
                    wram(table.name, table.offset.wrapping_add(entry)),
                )
            };
            Ok([slot(MUSIC_TRACKS)?, slot(MUSIC_DELAYS)?])
        })
        .collect()
}

fn played_music(requests: &[MusicRequest]) -> Vec<[u16; 2]> {
    requests
        .iter()
        .map(|request| match request {
            MusicRequest::LoadCeresFlightTune(delay) => [0xff2d, delay.0],
            MusicRequest::PlayCeresFlightTune(delay) => [0x0005, delay.0],
        })
        .collect()
}

fn music_observation(requests: Vec<[u16; 2]>) -> Observation {
    Observation {
        field: "flight.music".to_owned(),
        bytes: requests
            .into_iter()
            .flatten()
            .flat_map(u16::to_le_bytes)
            .collect(),
    }
}

/// A sprite as compared: its place with fractions, and the rushing stars' speed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Sprite {
    x: i64,
    y: i64,
    speed: Option<i64>,
}

impl Sprite {
    /// Its whole-pixel place, as the sprite drawing is told it.
    fn whole(self) -> [i64; 2] {
        [self.x, self.y].map(|place| {
            let [high, low, _, _] = u32::try_from(place).unwrap_or(0).to_be_bytes();
            i64::from(i16::from_be_bytes([high, low]))
        })
    }
}

/// The flight's compared fields, from either side.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Fields {
    stage: Option<Stage>,
    angle: i64,
    zoom: i64,
    position: [i64; 2],
    transform: [i64; 4],
    scroll: [i64; 2],
    flash: [i64; 3],
    level: i64,
    blanked: bool,
    fade_count: i64,
    /// The label's step and its hold left, while it is typed.
    label: Option<(i64, i64)>,
    stars: Option<Sprite>,
    drifting: EnumMap<Drifter, Option<Sprite>>,
    /// Sprites in slots no kind of the flight holds.
    others: i64,
    /// Each letter typed this frame: its row, column and letter.
    typed: Vec<(u32, u32, char)>,
    /// The sprites the sprite drawing is told: each kind's number and whole-pixel place.
    views: Vec<[i64; 3]>,
    /// Whether the flight is done.
    done: bool,
}

impl Fields {
    /// The brightness alone, as the State holds it.
    fn brightness_of(state: &State) -> Self {
        Self {
            level: i64::from(state.brightness.level.get()),
            blanked: state.brightness.visibility == Visibility::Blanked,
            ..Self::default()
        }
    }

    fn brightness(&self) -> Vec<Observation> {
        vec![
            Observation::quantity("flight.brightness.level", self.level),
            Observation::quantity("flight.brightness.blanked", i64::from(self.blanked)),
        ]
    }

    fn observations(&self) -> Vec<Observation> {
        fn quantity(field: impl Into<String>, value: i64) -> Observation {
            Observation::quantity(field, value)
        }
        let mut fields = vec![
            quantity("flight.stage", self.stage.map_or(-1, Stage::code)),
            quantity("flight.angle", self.angle),
            quantity("flight.zoom", self.zoom),
            quantity("flight.position.x", self.position[0]),
            quantity("flight.position.y", self.position[1]),
        ];
        for (name, value) in ["a", "b", "c", "d"].iter().zip(self.transform) {
            fields.push(quantity(format!("flight.transform.{name}"), value));
        }
        fields.push(quantity("flight.scroll.x", self.scroll[0]));
        fields.push(quantity("flight.scroll.y", self.scroll[1]));
        for (name, value) in ["r", "g", "b"].iter().zip(self.flash) {
            fields.push(quantity(format!("flight.flash.{name}"), value));
        }
        fields.extend(self.brightness());
        fields.push(quantity("flight.fade-count", self.fade_count));
        let (step, hold) = self.label.unwrap_or((-1, 0));
        fields.push(quantity("flight.label.step", step));
        fields.push(quantity("flight.label.hold", hold));
        let sprites = std::iter::once(("RushingStars".to_owned(), self.stars)).chain(
            self.drifting
                .iter()
                .map(|(kind, sprite)| (format!("{kind:?}"), *sprite)),
        );
        for (kind, sprite) in sprites {
            fields.push(quantity(
                format!("flight.sprite.{kind}.placed"),
                i64::from(sprite.is_some()),
            ));
            if let Some(sprite) = sprite {
                fields.push(quantity(format!("flight.sprite.{kind}.x"), sprite.x));
                fields.push(quantity(format!("flight.sprite.{kind}.y"), sprite.y));
                if let Some(speed) = sprite.speed {
                    fields.push(quantity(format!("flight.sprite.{kind}.speed"), speed));
                }
            }
        }
        fields.push(quantity("flight.sprite.others", self.others));
        fields.push(Observation {
            field: "flight.sprite.views".to_owned(),
            bytes: self
                .views
                .iter()
                .flatten()
                .flat_map(|value| value.to_le_bytes())
                .collect(),
        });
        fields.push(quantity("flight.done", i64::from(self.done)));
        fields.push(Observation {
            field: "flight.typed".to_owned(),
            bytes: self
                .typed
                .iter()
                .flat_map(|&(row, column, letter)| {
                    [row, column, u32::from(letter)]
                        .into_iter()
                        .flat_map(u32::to_le_bytes)
                })
                .collect(),
        });
        fields
    }
}

/// A place with its fraction, as one number: the whole pixels above the fraction's 16 bits.
fn fixed(place: Fixed) -> i64 {
    i64::from(place.0)
}

/// The flight as the played State and its frame's report hold it.
fn played(state: &State, report: &FrameReport, typed: Option<TypedLetter>) -> Fields {
    let stars = state.sprites.rushing_stars.map(|stars| Sprite {
        x: fixed(stars.place.x),
        y: fixed(stars.place.y),
        speed: Some(i64::from(stars.speed.0)),
    });
    let drifting = state.sprites.drifting.map(|_, sprite| {
        sprite.map(|sprite| Sprite {
            x: fixed(sprite.place.x),
            y: fixed(sprite.place.y),
            speed: None,
        })
    });
    let label = if let FlightStage::Label(progress) = state.stage {
        Some((
            i64::try_from(progress.step).unwrap_or(i64::MAX),
            i64::from(progress.hold_left),
        ))
    } else {
        None
    };
    Fields {
        stage: Some(Stage::of(state.stage)),
        angle: i64::from(state.plane.angle.0),
        zoom: i64::from(state.plane.zoom.get()),
        position: [fixed(state.plane.position.x), fixed(state.plane.position.y)],
        transform: [
            report.transform.a,
            report.transform.b,
            report.transform.c,
            report.transform.d,
        ]
        .map(i64::from),
        scroll: [report.scroll.x, report.scroll.y].map(i64::from),
        flash: report.flash.channels().map(i64::from),
        fade_count: i64::from(state.fade_count.get()),
        label,
        stars,
        drifting,
        others: 0,
        views: report
            .sprites
            .iter()
            .map(|view| {
                [
                    kind(view.kind),
                    i64::from(view.place.x),
                    i64::from(view.place.y),
                ]
            })
            .collect(),
        done: report.done,
        typed: typed
            .map(|typed| {
                (
                    u32::from(typed.place.row),
                    u32::from(typed.place.column),
                    typed.letter,
                )
            })
            .into_iter()
            .collect(),
        ..Fields::brightness_of(state)
    }
}

/// The flight as the record at `current` holds it. The label's step is the count of steps the
/// record has begun: each moves the label's list on.
fn recorded(
    previous: &FrameRecord,
    current: &FrameRecord,
    label_steps: u64,
) -> Result<Fields, Outcome> {
    let wram = &current.wram;
    let function = word(wram, source::CINEMATIC_ACTIVE)?;
    let stage = STAGES
        .iter()
        .find(|(at, _)| *at == function)
        .map(|&(_, stage)| stage);
    let place = |[whole, fraction]: [WramWord; 2]| -> Result<i64, Outcome> {
        Ok(fixed(types::fixed(
            Wrapping(word(wram, whole)?),
            word(wram, fraction)?,
        )))
    };
    let signed =
        |at: WramWord| -> Result<i64, Outcome> { Ok(i64::from(word(wram, at)?.cast_signed())) };
    let byte = |at: usize| -> Result<u8, Outcome> {
        wram.get(at)
            .copied()
            .ok_or_else(|| data_failure(None, "played flight work RAM"))
    };
    let Sprites {
        stars,
        drifting,
        others,
    } = sprites(wram, stage.is_some_and(Stage::turned))?;
    let label = if stage == Some(Stage::Label) {
        Some((
            i64::try_from(label_steps.saturating_sub(1)).unwrap_or(i64::MAX),
            i64::from(word(wram, LABEL_TIMER)?),
        ))
    } else {
        None
    };
    let display = byte(usize::from(source::REG_INIDISP.offset))?;
    Ok(Fields {
        stage,
        angle: i64::from(word(wram, ANGLE)?),
        zoom: i64::from(word(wram, ZOOM)?),
        position: [place(X)?, place(Y)?],
        transform: [
            signed(TRANSFORM[0])?,
            signed(TRANSFORM[1])?,
            signed(TRANSFORM[2])?,
            signed(TRANSFORM[3])?,
        ],
        scroll: [signed(SCROLL[0])?, signed(SCROLL[1])?],
        flash: [
            i64::from(byte(FLASH[0])? & CHANNEL),
            i64::from(byte(FLASH[1])? & CHANNEL),
            i64::from(byte(FLASH[2])? & CHANNEL),
        ],
        level: i64::from(display & LEVEL),
        blanked: display & BLANK != 0,
        fade_count: i64::from(word(wram, FADE_COUNT)?),
        label,
        stars,
        drifting,
        others,
        views: stars
            .map(|stars| (FlightSprite::RushingStars, stars))
            .into_iter()
            .chain(drifting.iter().filter_map(|(of, sprite)| {
                sprite.map(|sprite| (FlightSprite::Drifting(of), sprite))
            }))
            .map(|(of, sprite)| {
                let [x, y] = sprite.whole();
                [kind(of), x, y]
            })
            .collect(),
        done: stage == Some(Stage::Done),
        typed: typed(previous, current)?,
    })
}

/// Whether the record at `current` began a step of the label: its first frame, or one that
/// moved the label's list on.
fn step_begun(previous: &FrameRecord, current: &FrameRecord) -> Result<bool, Outcome> {
    let in_label = |record: &FrameRecord| -> Result<bool, Outcome> {
        Ok(word(&record.wram, source::CINEMATIC_ACTIVE)? == LABEL_FUNCTION)
    };
    Ok(in_label(current)?
        && (!in_label(previous)?
            || word(&previous.wram, LABEL_LIST)? != word(&current.wram, LABEL_LIST)?))
}

/// A sprite's kind, by its number as compared.
const fn kind(sprite: FlightSprite) -> i64 {
    match sprite {
        FlightSprite::RushingStars => 0,
        FlightSprite::Drifting(Drifter::Station) => 1,
        FlightSprite::Drifting(Drifter::SmallAsteroids) => 2,
        FlightSprite::Drifting(Drifter::LargeAsteroids) => 3,
        FlightSprite::Drifting(Drifter::Vortex) => 4,
        FlightSprite::Drifting(Drifter::DistantStars) => 5,
    }
}

/// The sprites by kind, and the count of slots placed that no kind holds.
struct Sprites {
    stars: Option<Sprite>,
    drifting: EnumMap<Drifter, Option<Sprite>>,
    others: i64,
}

/// The sprites as the record's work RAM holds them, the drifting ones' slots once the view has
/// `turned`.
fn sprites(wram: &[u8], turned: bool) -> Result<Sprites, Outcome> {
    let slot = |table: usize, slot: usize| -> Result<u16, Outcome> {
        wram.get(table..)
            .and_then(|words| words.as_chunks::<2>().0.get(slot))
            .map(|bytes| u16::from_le_bytes(*bytes))
            .ok_or_else(|| data_failure(None, "played flight sprites"))
    };
    let sprite = |at: usize, speed: bool| -> Result<Option<Sprite>, Outcome> {
        if slot(SPRITE_LIST, at)? == 0 {
            return Ok(None);
        }
        let part = |whole, fraction| -> Result<i64, Outcome> {
            Ok(fixed(types::fixed(
                Wrapping(slot(whole, at)?),
                slot(fraction, at)?,
            )))
        };
        Ok(Some(Sprite {
            x: part(SPRITE_X, SPRITE_X_FRACTION)?,
            y: part(SPRITE_Y, SPRITE_Y_FRACTION)?,
            speed: if speed {
                Some(i64::from(slot(SPRITE_SPEED, at)?.cast_signed()))
            } else {
                None
            },
        }))
    };
    let mut used = vec![];
    let stars = if turned {
        None
    } else {
        used.push(STARS_SLOT);
        sprite(STARS_SLOT, true)?
    };
    let mut drifting: EnumMap<Drifter, Option<Sprite>> = EnumMap::default();
    if turned {
        for (kind, held) in &mut drifting {
            if let Some(&(_, at)) = DRIFTING_SLOTS.iter().find(|(of, _)| *of == kind) {
                used.push(at);
                *held = sprite(at, false)?;
            }
        }
    }
    let mut others: i64 = 0;
    for at in (0..SLOTS).filter(|at| !used.contains(at)) {
        if slot(SPRITE_LIST, at)? != 0 {
            others = others.saturating_add(1);
        }
    }
    Ok(Sprites {
        stars,
        drifting,
        others,
    })
}

/// The letters the record's label layer shows anew: each word that changed to a letter of the
/// label's font, by row and column. Its other changes are the layer's emptying, masked.
fn typed(previous: &FrameRecord, current: &FrameRecord) -> Result<Vec<(u32, u32, char)>, Outcome> {
    fn layer(record: &FrameRecord) -> Result<&[[u16; LAYER_WIDTH]], Outcome> {
        record
            .vram
            .get(LABEL_LAYER)
            .map(|words| words.as_chunks::<LAYER_WIDTH>().0)
            .ok_or_else(|| data_failure(None, "played flight label layer"))
    }
    let (before, after) = (layer(previous)?, layer(current)?);
    let mut letters = Vec::new();
    for ((old, new), row) in before.iter().zip(after).zip(0_u32..) {
        for ((old, &new), column) in old.iter().zip(new).zip(0_u32..) {
            if *old != new
                && let Some(letter) = import::flight::letter(new)
            {
                letters.push((row, column, letter));
            }
        }
    }
    Ok(letters)
}

#[cfg(test)]
mod tests {
    fn diagnostic(args: std::fmt::Arguments<'_>) {
        use std::io::Write as _;
        writeln!(std::io::stdout().lock(), "{args}").unwrap();
    }

    /// The flight to Ceres from its first frame to the row's last cut.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn the_flight_to_ceres_played_alone() {
        let path = |name| std::env::var_os(name).unwrap().into();
        let sources = crate::replay::Sources {
            cache: path("SM_CACHE"),
            rom: path("SM_ROM"),
            lock: path("SM_LOCK"),
            ..Default::default()
        };
        let report = super::super::play(&sources, "any-fly-to-ceres", 0, 802).unwrap();
        for start in 1..32 {
            let resumed = super::super::play(&sources, "any-fly-to-ceres", start, 32).unwrap();
            assert_eq!(resumed.held, 32 - start, "flight setup seed {start}");
            assert_eq!(resumed.stop, super::Stop::End, "flight setup seed {start}");
        }
        diagnostic(format_args!(
            "PLAYED_ALONE row={} start={} held={} stop={:?}",
            report.row, report.start, report.held, report.stop
        ));
    }
}
