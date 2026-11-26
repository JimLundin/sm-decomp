//! A run from the console's power-on: the front end's boot, then its screens from the title on
//! (`screens`), compared at each cut through the options' hand-on to the intro and on through
//! the intro's opening and page 1, whose own fields are the intro run's (`intro`).
//!
//! The run admits the power-on State itself: of cut 0 it reads only where the next cut's
//! interval begins, as its work RAM is the emulator's power-on fill. Each cut borrows the inputs
//! [`BORROWED`] names. The movie's pad is read from the title on: the settle clears the game's
//! own latch, so the boot's presses reach no screen.
//!
//! Compared at each cut: while the logo is still to come, only whether the screen is blank, as
//! the level under the blank first comes from the platform's power-on value; after that the
//! screen, blank or lit, and its level. The count of finished frames at finished cuts and while
//! the sound loads, and once the game settles from the third cut after the upload's end: the
//! settle clears the count's memory through the upload's cut and the two after it. The hold's
//! frames left while the logo is held before and after the frame.

use super::intro::{Part, Scenes};
use super::platform::{Controller, polled};
use super::screens::{self, Asked, Borrowed, Queues};
use super::{Report, Stop, rows::Rows, word};
use crate::replay::compare::{CompareFault, Observation, compare};
use crate::replay::{data_failure, result::Outcome};
use crate::source;
use data::rom::Rom;
use front_end::{Cartridge, CartridgeFrame, Content, FrameEnd, Rules, Work};
use game::exact::save_files::{Format, profile_bytes};
use game::production::played::front_end;
use oracle_record::{ApuAccess, ApuAccessKind, FrameRecord};
use screens_boot::{DriverUpload, PowerOnStage, PowerOnState, ScreenLight};
use std::path::Path;
use types::save_files::BatteryImage;

/// The logo's timer: the hold's frames left.
const HOLD_LEFT: source::WramWord = source::WramWord {
    name: "LOGO_TIMER",
    offset: 0x0DE2,
};

/// The display register's forced blank bit and its brightness.
const BLANK: u8 = 0x80;
const LEVEL: u8 = 0x0F;

/// The upload's cut and the two after it, where the settle's clear is under way.
const CLEARING: u64 = 3;

/// The APU ports the upload's end is written to: the driver's entry on ports 2 and 3, then 0 on
/// port 1, where a block of data has a size above 0.
const ENTRY_PORTS: [u8; 2] = [2, 3];
const SIZE_PORT: u8 = 1;

/// The rules the run plays: the game as it ships.
const RULES: Rules = Rules::REFERENCE;

/// Inputs a power-on run reads from the record at each cut, because their owner is outside the
/// rungs (decisions 35 and 37). A power-on run holds nothing from a seed.
pub(super) const BORROWED: &[&str] = &[
    "lag: whether each cut's frame of work finished, a vblank without NMI as not finished",
    "the boot's driver upload ended in the frame (CPU and audio timing until T1), from the interval's APU port writes: looked for at every cut, and an end outside the sound's load is bad data",
    "lag from the title on: how many frames of work finished in the interval, from the cut kind, and 2 where the game's own count steps by 2 at a requested NMI (an NMI inside the interval that the record's cut list doesn't show); any other disagreement is bad data",
    "the random number as the game settles (game-random's seeding of the boot), read from the record as the title begins",
    "audio from the intro on: whether a track waits and the sound count read by Samus at the start of her playable turn",
    "the initial platform battery and the cut at which the startup diagnostic and scan have finished",
];

/// Play the power-on row `row` from power-on through `end`, the intro's scenes imported from
/// the player's ROM at `rom`. A run from a later cut has no State to admit: nothing reads the
/// power-on's State from a record.
pub(super) fn play(
    rows: &mut Rows<'_>,
    rom: &Path,
    row: &str,
    start: u64,
    end: u64,
) -> Result<Report, Outcome> {
    let report = |held, stop| Report {
        row: row.to_owned(),
        start,
        held,
        stop,
        seed_held: &[],
        borrowed: BORROWED,
        masked: &[
            "the displayed room tiles, sprite pixels and layer mirrors (E1, E2, E3 and E4)",
            "the unproduced retained tail of room memory and the opening palette transfers (T1)",
        ],
    };
    if start != 0 {
        let later = Outcome::Unsupported {
            cut: Some(start),
            dependency: "a power-on run from a later cut".into(),
        };
        return Ok(report(0, Stop::At(later)));
    }
    let mut previous = rows.record(0)?;
    let mut run = Run::begin(rom, &previous)?;
    let mut held = 0;
    let mut began = previous.clone();
    for cut in 1..=end {
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

/// The front end's State and content, the intro's reader, the cut the upload ended at, and from
/// the title on the controller, the queues at the last finished cut and the sounds played since
/// it.
#[derive(Clone)]
struct Run {
    state: front_end::State,
    cartridge: Cartridge,
    format: Format,
    content: Content,
    scenes: Scenes,
    ended: Option<u64>,
    controller: Controller,
    queues: Option<Queues>,
    sounds: Vec<[u8; 3]>,
}

impl Run {
    fn begin(rom: &Path, power_on: &FrameRecord) -> Result<Self, Outcome> {
        let scenes = Scenes::import(rom, Part::Opening)?;
        let battery = BatteryImage::new(power_on.sram.to_vec())
            .ok_or_else(|| data_failure(Some(0), "power-on platform battery"))?;
        let path = rom;
        let rom =
            Rom::new(std::fs::read(rom).map_err(|_| data_failure(None, "save format source"))?)
                .map_err(|_| data_failure(None, "checked save format source"))?;
        let format =
            Format::load(rom.lorom()).map_err(|_| data_failure(None, "save format assets"))?;
        let cartridge = Cartridge::power_on(&format, &battery);
        Ok(Self {
            state: front_end::State::AT_POWER_ON,
            cartridge,
            format,
            content: Content {
                intro: scenes.content.clone(),
                actors: Some(front_end::ActorsContent {
                    room: crate::replay::intro_rooms::room_content(path, &rom)?.0,
                    rules: game::production::played::intro_rooms::ScopedRules {
                        movement: samus_movement::Rules::default(),
                        appearance: samus_appearance::Rules::VANILLA,
                    },
                    pad: import::playback::import(rom.lorom())
                        .map_err(|_| data_failure(None, "intro recorded controls"))?,
                }),
            },
            scenes,
            ended: None,
            controller: Controller::default(),
            queues: None,
            sounds: Vec::new(),
        })
    }

    /// Play the interval from `previous` to `record`, at `cut`, whose frame of work began after
    /// `began`, and compare the part that runs with the record.
    ///
    /// An upload's end outside the sound's load is bad data, and the game settles only on one,
    /// so a run that reaches the title found exactly one.
    fn step(
        &mut self,
        began: &FrameRecord,
        previous: &FrameRecord,
        record: &FrameRecord,
        cut: u64,
    ) -> Result<(), Outcome> {
        let before = self.state.clone();
        // From the title on the uploads are the audio side's: the title's music data ends the
        // same way, at 354 in the any% movie.
        let ended = matches!(before, front_end::State::PowerOn(_)) && upload_ended(&record.apu_log);
        let loading = matches!(before, front_end::State::PowerOn(power_on) if power_on.stage == PowerOnStage::SoundLoading);
        if ended && !loading {
            return Err(data_failure(
                Some(cut),
                "power-on upload end outside the load",
            ));
        }
        let finished = super::finished(record);
        let pressed = screens::pressed(self.controller);
        let work = match before {
            front_end::State::PowerOn(_) => u16::from(finished),
            front_end::State::Screens(_) => screens::finished_frames(previous, record, finished)
                .map_err(|outcome| super::at(cut, outcome))?,
        };
        let frame = FrameEnd {
            work: match work {
                0 => Work::Unfinished,
                1 => Work::Finished(pressed),
                // The second frame read no latch of its own: the record holds none for the NMI
                // inside the interval.
                _ => Work::FinishedTwice([pressed; 2]),
            },
            upload: if ended {
                DriverUpload::Ended
            } else {
                DriverUpload::Continuing
            },
        };
        let answers = Borrowed::read(began, previous).map_err(|outcome| super::at(cut, outcome))?;
        if cut == 271 {
            let report =
                self.cartridge
                    .boot(&self.format)
                    .map_err(|fault| Outcome::Unsupported {
                        cut: Some(cut),
                        dependency: format!("cartridge startup {fault:?}"),
                    })?;
            if record.wram.get(0x1f59).copied() != Some(report.demos.sets()) {
                return Err(Outcome::Difference {
                    cut,
                    field: "save-files.demos".to_owned(),
                });
            }
        }
        let was_started = self.cartridge.started().is_some();
        let sounds = self
            .state
            .step_with_cartridge(
                &mut self.cartridge,
                &self.format,
                &CartridgeFrame {
                    end: frame,
                    in_flight: pressed,
                    rules: &RULES,
                    content: &self.content,
                    platform: &answers,
                },
            )
            .map_err(|fault| Outcome::Unsupported {
                cut: Some(cut),
                dependency: format!("played frame {fault:?}"),
            })?;
        if cut >= 271 && self.cartridge.image(&self.format).as_slice() != record.sram.as_ref() {
            return Err(Outcome::Difference {
                cut,
                field: "save-files.battery".to_owned(),
            });
        }
        if !was_started
            && let Some(run) = self.cartridge.started()
            && record.wram.get(0x09a2..0x0a02) != Some(&profile_bytes(&run.profile))
        {
            return Err(Outcome::Difference {
                cut,
                field: "save-files.working-profile".to_owned(),
            });
        }
        if ended {
            self.ended = Some(cut);
        }
        match (&before, self.state.clone()) {
            (front_end::State::PowerOn(before), front_end::State::PowerOn(after)) => {
                self.compare_power_on(*before, after, record, finished, cut)
            }
            (_, front_end::State::Screens(after)) => {
                if self.queues.is_none() {
                    self.queues =
                        Some(Queues::read(previous).map_err(|outcome| super::at(cut, outcome))?);
                }
                self.sounds
                    .extend(sounds.into_iter().filter_map(screens::sound_code));
                self.compare_screens(&after, record, finished, cut)
            }
            (front_end::State::Screens(_), front_end::State::PowerOn(_)) => {
                Err(data_failure(Some(cut), "played front end back at power-on"))
            }
        }
    }

    /// The boot's fields at `cut`.
    fn compare_power_on(
        &self,
        before: PowerOnState,
        after: PowerOnState,
        record: &FrameRecord,
        finished: bool,
        cut: u64,
    ) -> Result<(), Outcome> {
        let compared = Compared::at(before, after, finished, self.ended, cut);
        let expected = compared.observations(&Fields::recorded(record)?);
        let actual = compared.observations(&Fields::played(after));
        differs(&expected, &actual, cut)
    }

    /// The screens' fields at `cut`, the movie's pad read for the next frame, and the sounds
    /// compared at finished cuts.
    fn compare_screens(
        &mut self,
        after: &front_end::Screens,
        record: &FrameRecord,
        finished: bool,
        cut: u64,
    ) -> Result<(), Outcome> {
        self.controller = self.controller.read(record.pad.0, polled(record));
        // The movie's pad is the input; the game's own latch only checks its alignment.
        if polled(record)
            && (
                word(&record.wram, source::JOYPAD1_LASTKEYS)?,
                word(&record.wram, source::JOYPAD1_NEWKEYS)?,
            ) != (self.controller.held, self.controller.pressed)
        {
            return Err(data_failure(Some(cut), "played pad alignment"));
        }
        let asked = match (finished, &self.queues) {
            (true, Some(queues)) => {
                let mut played = std::mem::take(&mut self.sounds);
                played.sort_unstable();
                let expected = queues
                    .added(record)
                    .map_err(|outcome| super::at(cut, outcome))?;
                self.queues =
                    Some(Queues::read(record).map_err(|outcome| super::at(cut, outcome))?);
                Some(Asked { expected, played })
            }
            (false, _) | (true, None) => None,
        };
        let mut pairs = screens::compared(record, after, finished, asked)
            .map_err(|outcome| super::at(cut, outcome))?;
        if let front_end::Screen::Intro(intro) = &after.screen {
            let (expected, actual) = self
                .scenes
                .compared(record, &intro.scene)
                .map_err(|outcome| super::at(cut, outcome))?;
            pairs.expected.extend(expected);
            pairs.actual.extend(actual);
            let room = intro
                .actors
                .rooms
                .as_ref()
                .and_then(game::production::played::intro_rooms::State::active);
            let content = self
                .content
                .actors
                .as_ref()
                .ok_or_else(|| data_failure(Some(cut), "power-on intro room content"))?;
            let shown = crate::replay::intro_rooms::fight_shown(&record.wram)?;
            let (expected, actual) = crate::replay::intro_rooms::populated(
                room,
                &content.room.rooms,
                &record.wram,
                shown,
            )?;
            pairs.expected.extend(expected);
            pairs.actual.extend(actual);
            let encounter = intro.actors.battle.as_ref().map(|battle| &battle.encounter);
            let (expected, actual) =
                crate::replay::flashback::played_state(record, shown, encounter)?;
            pairs.expected.extend(expected);
            pairs.actual.extend(actual);
            let (expected, actual) =
                crate::replay::intro_rooms::opening_fields(&intro.actors, &record.wram)?;
            pairs.expected.extend(expected);
            pairs.actual.extend(actual);
        }
        differs(&pairs.expected, &pairs.actual, cut)
    }
}

/// The first difference between the record's fields and the played ones.
fn differs(expected: &[Observation], actual: &[Observation], cut: u64) -> Result<(), Outcome> {
    compare(expected, actual).map_err(|fault| match fault {
        CompareFault::Difference(field) => Outcome::Difference { cut, field },
        CompareFault::EmptyExpected
        | CompareFault::DuplicateField(_)
        | CompareFault::MissingActual(_) => data_failure(Some(cut), "played comparison schema"),
    })
}

/// Borrowed from audio: the CPU sent the upload's end in the interval, a write of 0 to the size
/// port right after the driver's entry was written to the entry ports.
fn upload_ended(log: &[ApuAccess]) -> bool {
    let writes: Vec<&ApuAccess> = log
        .iter()
        .filter(|access| {
            matches!(
                access.kind,
                ApuAccessKind::CpuWrite | ApuAccessKind::DmaWrite
            )
        })
        .collect();
    writes.windows(2).any(|pair| {
        matches!(pair, [entry, size] if ENTRY_PORTS.contains(&entry.port)
            && size.port == SIZE_PORT
            && size.value == 0)
    })
}

/// Which of the boot's fields a cut compares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Compared {
    /// The screen: only whether it is blank, or blank or lit with its level.
    screen: Screen,
    finished_frames: bool,
    hold: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Screen {
    Blank,
    Whole,
}

impl Compared {
    fn at(
        before: PowerOnState,
        after: PowerOnState,
        finished: bool,
        ended: Option<u64>,
        cut: u64,
    ) -> Self {
        let quiet =
            before.stage == PowerOnStage::BeforeLogo && after.stage == PowerOnStage::BeforeLogo;
        // Before the load the count is compared at finished cuts only, because until the clear in
        // cut 4's interval it is still the emulator's power-on fill, and no frame finishes before
        // the logo. Every cut of the fades in the record is finished, so no other cut is lost.
        let finished_frames = match after.stage {
            PowerOnStage::Settling => {
                ended.is_some_and(|ended| cut.saturating_sub(ended) >= CLEARING)
            }
            PowerOnStage::SoundLoading => true,
            PowerOnStage::BeforeLogo
            | PowerOnStage::Brightening
            | PowerOnStage::Held(_)
            | PowerOnStage::Dimming => finished,
        };
        Self {
            screen: if quiet { Screen::Blank } else { Screen::Whole },
            finished_frames,
            hold: matches!(
                (before.stage, after.stage),
                (PowerOnStage::Held(_), PowerOnStage::Held(_))
            ),
        }
    }

    fn observations(self, fields: &Fields) -> Vec<Observation> {
        let mut observed = vec![match self.screen {
            Screen::Blank => {
                Observation::quantity("power-on.blank", i64::from(fields.screen & BLANK))
            }
            Screen::Whole => Observation::quantity(
                "power-on.screen",
                i64::from(fields.screen & (BLANK | LEVEL)),
            ),
        }];
        if self.finished_frames {
            observed.push(Observation::quantity(
                "power-on.finished-frames",
                i64::from(fields.finished_frames),
            ));
        }
        // A State outside the hold has no frames left to compare: its actual is missing, a schema
        // fault rather than a made-up value.
        if let (true, Some(hold)) = (self.hold, fields.hold) {
            observed.push(Observation::quantity("power-on.hold", i64::from(hold)));
        }
        observed
    }
}

/// The boot's fields, the screen as the display register holds it.
struct Fields {
    screen: u8,
    finished_frames: u16,
    /// The hold's frames left; none outside the hold.
    hold: Option<u16>,
}

impl Fields {
    fn recorded(record: &FrameRecord) -> Result<Self, Outcome> {
        Ok(Self {
            screen: record.ppu.inidisp,
            finished_frames: word(&record.wram, source::NMI_FRAME_COUNTER_WORD)?,
            hold: Some(word(&record.wram, HOLD_LEFT)?),
        })
    }

    const fn played(state: PowerOnState) -> Self {
        let screen = match state.screen {
            ScreenLight::Blank(level) => BLANK | level.get(),
            ScreenLight::Lit(level) => level.get(),
        };
        Self {
            screen,
            finished_frames: state.frames.0,
            hold: match state.stage {
                PowerOnStage::Held(left) => Some(left.0),
                PowerOnStage::BeforeLogo
                | PowerOnStage::Brightening
                | PowerOnStage::Dimming
                | PowerOnStage::SoundLoading
                | PowerOnStage::Settling => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Compared, Screen, upload_ended};
    use oracle_record::{ApuAccess, ApuAccessKind};
    use screens_boot::{
        FinishedFrames, HoldFrames, Level, PowerOnStage, PowerOnState, ScreenLight,
    };

    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn power_on_resumes_after_opening_and_battle_setup() {
        const DEMO_NEW_KEYS: crate::source::WramWord = crate::source::WramWord {
            name: "DEMO_NEW_KEYS",
            offset: 0x0a86,
        };
        const SHOOT: u16 = 0x0040;
        const FIRST_MISSILE: u64 = 2310;
        let sources = crate::replay::played::tests::sources();
        let lock = super::super::read_lock(&sources).unwrap();
        let mut rows = super::super::open_rows(&sources, &lock, "any-power-on").unwrap();
        let mut previous = rows.record(0).unwrap();
        let mut began = previous.clone();
        let mut uninterrupted = super::Run::begin(&sources.rom, &previous).unwrap();
        let mut resumed = Vec::<super::Run>::new();
        for cut in 1..FIRST_MISSILE {
            let frame = rows.record(cut).unwrap();
            uninterrupted.step(&began, &previous, &frame, cut).unwrap();
            for fork in &mut resumed {
                fork.step(&began, &previous, &frame, cut).unwrap();
                assert_eq!(fork.state, uninterrupted.state, "front end at {cut}");
                assert_eq!(
                    fork.cartridge, uninterrupted.cartridge,
                    "cartridge at {cut}"
                );
            }
            if matches!(cut, 569 | 2219) {
                assert!(!super::super::finished(&frame));
                resumed.push(uninterrupted.clone());
            }
            if super::super::finished(&frame) {
                began = frame.clone();
            }
            previous = frame;
        }
        assert_eq!(resumed.len(), 2);
        let first_missile = rows.record(FIRST_MISSILE).unwrap();
        assert!(super::super::finished(&first_missile));
        assert_ne!(
            super::word(&first_missile.wram, DEMO_NEW_KEYS).unwrap() & SHOOT,
            0
        );
        for run in std::iter::once(&mut uninterrupted).chain(&mut resumed) {
            assert!(matches!(
                run.step(&began, &previous, &first_missile, FIRST_MISSILE),
                Err(super::Outcome::Unsupported {
                    cut: Some(FIRST_MISSILE),
                    ..
                })
            ));
        }
    }

    fn write(port: u8, value: u8) -> ApuAccess {
        ApuAccess {
            kind: ApuAccessKind::CpuWrite,
            port,
            value,
            apu_cycle: 0,
        }
    }

    #[test]
    fn the_upload_ends_with_a_zero_size_after_the_entry() {
        let read = ApuAccess {
            kind: ApuAccessKind::CpuRead,
            ..write(0, 0)
        };
        let end = [
            write(2, 0x00),
            write(3, 0x15),
            read,
            write(1, 0),
            write(0, 0xCD),
        ];
        assert!(upload_ended(&end));
        // A data byte of 0 follows the counter on port 0; a block's start has a size above 0.
        assert!(!upload_ended(&[write(0, 7), write(1, 0)]));
        assert!(!upload_ended(&[write(2, 0), write(3, 0x15), write(1, 1)]));
        assert!(!upload_ended(&[]));
    }

    fn state(stage: PowerOnStage) -> PowerOnState {
        PowerOnState {
            stage,
            screen: ScreenLight::Blank(Level::FULL),
            frames: FinishedFrames::ZERO,
        }
    }

    #[test]
    fn the_count_is_compared_from_the_third_cut_after_the_upload_ends() {
        let settling = state(PowerOnStage::Settling);
        let counted = |cut| Compared::at(settling, settling, false, Some(253), cut).finished_frames;
        assert_eq!(
            [253, 254, 255, 256].map(counted),
            [false, false, false, true]
        );
        let loading = state(PowerOnStage::SoundLoading);
        assert!(Compared::at(loading, loading, false, None, 200).finished_frames);
        let quiet = state(PowerOnStage::BeforeLogo);
        let compared = Compared::at(quiet, quiet, false, None, 3);
        assert_eq!(
            (compared.screen, compared.finished_frames, compared.hold),
            (Screen::Blank, false, false)
        );
        let held = state(PowerOnStage::Held(HoldFrames(5)));
        assert!(Compared::at(held, held, true, None, 40).hold);
        let full = state(PowerOnStage::Brightening);
        assert!(!Compared::at(full, held, true, None, 32).hold);
    }
}
