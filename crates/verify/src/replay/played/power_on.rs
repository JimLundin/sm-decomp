//! A run from the console's power-on: the front end's boot, then its screens from the title on
//! (`screens`), compared at each cut through the options' hand-on to the intro and on through
//! the intro, the flight and Ceres.
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
    "T1 timing of an interrupted fight frame, from its CPU program counter and calling stack",
    "the initial platform battery and the cut at which the startup diagnostic and scan have finished",
    "flight and Ceres: NMI work boundaries, audio queue replies, room setup readiness and passage fetch timing",
    "room passages: the random number at frame start; escape countdown initiation until the boss owns the start",
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
            "room tiles, sprite pixels, non-weapon sprite attributes and layer mirrors (E1, E2, E3 and E4)",
            "partly written palette, text and sprite shadow buffers inside an interrupted stage or drawing pass (T1)",
            "the unproduced retained tail of room memory and the opening palette transfers (T1)",
            "flight sprites, station lettering, layer mirrors and preparation transfers outside the current rendering checks",
            "the random number at lag cuts during flight (T1)",
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
        if super::flight::starts(&previous.wram)? {
            return Ok(report(
                held,
                Stop::At(Outcome::Difference {
                    cut,
                    field: "flight.active".into(),
                }),
            ));
        }
        if let Err(outcome) = run.step(&began, &previous, &current, cut) {
            return Ok(report(held, Stop::At(outcome)));
        }
        held = held.saturating_add(1);
        if cut < end
            && let Some(continuation) = run.flight(&current, cut)?
        {
            let (more, stop) = run.continue_flight(
                continuation,
                rows,
                rom,
                row,
                current,
                cut.saturating_add(1)..=end,
            )?;
            return Ok(report(held.saturating_add(more), stop));
        }
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
    missile_observations: crate::replay::intro_missiles::Expected,
    discovery_observations: crate::replay::discovery::Played,
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
                    encounter: import::flashback::load(rom.lorom())
                        .map_err(|_| data_failure(None, "intro encounter art"))?,
                    discovery: Some(
                        game::production::played::discovery::Discovery::new(
                            import::discovery::load(rom.lorom())
                                .map_err(|_| data_failure(None, "intro discovery art"))?,
                        )
                        .map_err(|_| data_failure(None, "intro discovery scene"))?,
                    ),
                    missiles: game::production::played::intro_missiles::Content::new(
                        import::projectile::import_intro_missile(rom.lorom())
                            .map_err(|_| data_failure(None, "intro missile assets"))?,
                        crate::replay::weapons::Content::load(path)?.weapons,
                    )
                    .map_err(|_| data_failure(None, "intro missile content"))?,
                    pad: import::playback::import(rom.lorom())
                        .map_err(|_| data_failure(None, "intro recorded controls"))?,
                }),
            },
            scenes,
            missile_observations: crate::replay::intro_missiles::Expected::load(rom.lorom())?,
            discovery_observations: crate::replay::discovery::Played::default(),
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
        let answers =
            Borrowed::read(began, previous, record).map_err(|outcome| super::at(cut, outcome))?;
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
        self.compare_battery(record, cut)?;
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
                self.compare_screens(&after, previous, record, finished, cut)
            }
            (front_end::State::Screens(_), front_end::State::PowerOn(_)) => {
                Err(data_failure(Some(cut), "played front end back at power-on"))
            }
        }
    }

    fn compare_battery(&self, record: &FrameRecord, cut: u64) -> Result<(), Outcome> {
        if cut >= 271 && self.cartridge.image(&self.format).as_slice() != record.sram.as_ref() {
            return Err(Outcome::Difference {
                cut,
                field: "save-files.battery".to_owned(),
            });
        }
        Ok(())
    }

    fn flight(
        &self,
        record: &FrameRecord,
        cut: u64,
    ) -> Result<Option<super::flight::Continuation>, Outcome> {
        let front_end::State::Screens(screens) = &self.state else {
            return Ok(None);
        };
        let front_end::Screen::Intro(intro) = &screens.screen else {
            return Ok(None);
        };
        let Some(journey) = game::production::played::flight::Journey::after_intro(
            &intro.scene,
            screens.counts.before_room(),
        ) else {
            return Ok(None);
        };
        let room = intro
            .actors
            .rooms
            .as_ref()
            .ok_or_else(|| data_failure(None, "intro actor at flight entry"))?;
        let [_, _, _, cancel, select, _, _] =
            game::exact::save_files::control_words(&intro.actors.buttons);
        let carry = super::start::Carried {
            health: room.actor.health.0,
            bindings: crate::replay::hud::Bindings::new(select, cancel),
            appearance: crate::replay::connected::AppearanceInputs {
                waiting: crate::replay::appearance::sounds_waiting(&record.wram)?,
                charge: samus_appearance::Charge::new(0)
                    .ok_or_else(|| data_failure(None, "intro actor charge"))?,
            },
            quake: world_quake::State {
                phase: world_quake::QuakePhase(0),
                shake: None,
            },
        };
        Ok(Some(
            super::intro::Departure {
                intro: intro.scene.clone(),
                frame: screens.counts.before_room(),
                controller: self.controller,
                carry,
            }
            .continue_flight(journey, cut)?,
        ))
    }

    fn continue_flight(
        &mut self,
        continuation: super::flight::Continuation,
        rows: &mut Rows<'_>,
        rom: &Path,
        row: &str,
        previous: FrameRecord,
        cuts: core::ops::RangeInclusive<u64>,
    ) -> Result<(u64, Stop), Outcome> {
        let selected = self.cartridge.state.active;
        let Some(started) = self.cartridge.started().cloned() else {
            return Err(data_failure(
                None,
                "Ceres flight requires the selected file",
            ));
        };
        continuation.through(rows, rom, row, previous, cuts, |record, cut, entered| {
            self.cartridge
                .arrival(&self.format, entered, super::finished(record))
                .map_err(|fault| Outcome::Unsupported {
                    cut: Some(cut),
                    dependency: format!("played cartridge {fault:?}"),
                })?;
            if entered {
                self.cartridge.state = super::flight::restored(
                    &self.cartridge.state,
                    cut,
                    "Ceres arrival cartridge checkpoint",
                )?;
            }
            if self.cartridge.state.active != selected || self.cartridge.started() != Some(&started)
            {
                return Err(Outcome::Difference {
                    cut,
                    field: "save-files.selected-file".into(),
                });
            }
            self.compare_battery(record, cut)
        })
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
        previous: &FrameRecord,
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
        let discovery_selected = matches!(after.screen, front_end::Screen::Intro(_))
            && super::intro::delivery::discovery_selected(&record.wram)?;
        let asked = match (finished, &self.queues) {
            (true, Some(queues)) => {
                let mut played = std::mem::take(&mut self.sounds);
                played.sort_unstable();
                let expected = queues
                    .added(record, discovery_selected)
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
            let scene = self.compare_intro(intro, after.counts, previous, record, finished, cut)?;
            pairs.expected.extend(scene.expected);
            pairs.actual.extend(scene.actual);
        }
        differs(&pairs.expected, &pairs.actual, cut)
    }
    fn compare_intro(
        &mut self,
        intro: &front_end::PlayingIntro,
        counts: game::production::played::frame::Counts,
        previous: &FrameRecord,
        record: &FrameRecord,
        finished: bool,
        cut: u64,
    ) -> Result<screens::Pairs, Outcome> {
        use game::production::played::intro_rooms as rooms;
        use types::intro::FightVisibility;

        let mut pairs = screens::Pairs::default();
        let discovery_selected = super::intro::delivery::discovery_selected(&record.wram)?;
        let (expected, actual) = self
            .scenes
            .compared(record, &intro.scene)
            .map_err(|outcome| super::at(cut, outcome))?;
        pairs.expected.extend(expected);
        pairs.actual.extend(actual);
        let fight_visible =
            intro.scene.intro.fight == FightVisibility::InFront
                || intro.actors.rooms.as_ref().is_some_and(|room| {
                    room.pending() && room.active() == Some(rooms::Start::Fight)
                });
        let room = intro
            .actors
            .rooms
            .as_ref()
            .and_then(rooms::State::active)
            .filter(|kind| match kind {
                rooms::Start::Fight => fight_visible,
                rooms::Start::Discovery => intro.actors.discovery.as_ref().is_some_and(|room| {
                    room.encounter().order != types::discovery::DiscoveryOrder::Hidden
                }),
            });
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
            discovery_selected,
        )?;
        pairs.expected.extend(expected);
        pairs.actual.extend(actual);
        let encounter = intro
            .actors
            .battle
            .as_ref()
            .filter(|_| fight_visible)
            .map(|battle| &battle.encounter);
        let (expected, actual) = crate::replay::flashback::played_state(
            record,
            shown && !discovery_selected,
            encounter,
        )?;
        pairs.expected.extend(expected);
        pairs.actual.extend(actual);
        let (expected, actual) =
            crate::replay::intro_rooms::opening_fields(&intro.actors, &record.wram)?;
        pairs.expected.extend(expected);
        pairs.actual.extend(actual);
        let (expected, actual) = self
            .discovery_observations
            .compared(
                previous,
                record,
                intro
                    .actors
                    .discovery
                    .as_ref()
                    .map(front_end::DiscoveryRoom::encounter),
                finished,
            )
            .map_err(|outcome| super::at(cut, outcome))?;
        pairs.expected.extend(expected);
        pairs.actual.extend(actual);
        if let Some(battle) = &intro.actors.battle {
            if !finished && intro_timing(record)? != front_end::IntroTiming::Unspecified {
                pairs.expected.push(Observation::quantity(
                    "front-end.random",
                    i64::from(word(&record.wram, source::RANDOM_NUMBER)?),
                ));
                pairs.actual.push(Observation::quantity(
                    "front-end.random",
                    i64::from(counts.random.number.0),
                ));
            }
            let room = intro
                .actors
                .rooms
                .as_ref()
                .ok_or_else(|| data_failure(Some(cut), "played battle room"))?;
            let (expected, actual) = self.missile_observations.played(
                record,
                &room.actor.shots.0,
                battle.supplies(&room.actor),
                finished.then_some(&battle.weapon_draws),
            )?;
            pairs.expected.extend(expected);
            pairs.actual.extend(actual);
        }
        Ok(pairs)
    }
}

/// The interrupted call and its saved return sites identify how far work ran without
/// reading any scene, actor, weapon or encounter quantities.
pub(super) fn intro_timing(record: &FrameRecord) -> Result<front_end::IntroTiming, Outcome> {
    use front_end::IntroTiming;
    use oracle_record::{Cut, Nmi, NmiKind};
    let Cut::NmiReturn(Nmi {
        kind: NmiKind::Overrun,
        interrupted_pc,
        ..
    }) = record.cut
    else {
        return Ok(IntroTiming::Unspecified);
    };
    let stack = usize::from(record.cpu.s);
    let at = |offset: usize| {
        stack
            .checked_add(offset)
            .and_then(|index| record.wram.get(index..))
    };
    let near = |offset| {
        at(offset)
            .and_then(|src| src.first_chunk::<2>())
            .map(|word| u16::from_le_bytes(*word))
    };
    let far = |offset| {
        at(offset)
            .and_then(|src| src.first_chunk::<3>())
            .map(|&[a, b, c]| u32::from_le_bytes([a, b, c, 0]))
    };
    let stage = |inner: usize, caller| {
        near(inner) == Some(caller)
            && near(inner.saturating_add(2)) == Some(0xa366)
            && far(inner.saturating_add(6)) == Some(0x0082_8b11)
            && near(inner.saturating_add(9)) == Some(0x896d)
    };
    let timing = match interrupted_pc {
        0x0080_8a37 | 0x0080_8985 if far(2) == Some(0x0082_8975) => IntroTiming::FinalisingSprites,
        0x008b_8cf1 if stage(2, 0xb072) => IntroTiming::SettingUpDiscovery,
        0x008b_8d09 if stage(2, 0xb072) => IntroTiming::SettingUpFight,
        0x008b_a876 if stage(6, 0xb2be) => IntroTiming::ClearingText,
        0x008b_8d1e if stage(2, 0xb3e5) => IntroTiming::PreparingReturn,
        _ => return Ok(IntroTiming::Unspecified),
    };
    if interrupted_pc != u32::from(record.cpu.k) << 16 | u32::from(record.cpu.pc) {
        return Err(data_failure(None, "intro interrupted instruction"));
    }
    Ok(timing)
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
        const LAST_HELD_CUT: u64 = 7512;
        let sources = crate::replay::played::tests::sources();
        let lock = super::super::read_lock(&sources).unwrap();
        let mut rows = super::super::open_rows(&sources, &lock, "any-power-on").unwrap();
        let mut previous = rows.record(0).unwrap();
        let mut began = previous.clone();
        let mut uninterrupted = super::Run::begin(&sources.rom, &previous).unwrap();
        let mut resumed = Vec::<super::Run>::new();
        let checkpoints = [569, 2219, 2320, 2325, 2350, 3517, 4323];
        for cut in 1..=LAST_HELD_CUT {
            let frame = rows.record(cut).unwrap();
            if cut == FIRST_MISSILE {
                assert!(super::super::finished(&frame));
                assert_ne!(super::word(&frame.wram, DEMO_NEW_KEYS).unwrap() & SHOOT, 0);
            }
            uninterrupted.step(&began, &previous, &frame, cut).unwrap();
            for fork in &mut resumed {
                fork.step(&began, &previous, &frame, cut).unwrap();
                assert_eq!(fork.state, uninterrupted.state, "front end at {cut}");
                assert_eq!(
                    fork.cartridge, uninterrupted.cartridge,
                    "cartridge at {cut}"
                );
            }
            if checkpoints.contains(&cut) {
                assert!(!super::super::finished(&frame));
                resumed.push(uninterrupted.clone());
            }
            if [2474, 2614, 2615].contains(&cut) {
                resumed.push(uninterrupted.clone());
            }
            if super::super::finished(&frame) {
                began = frame.clone();
            }
            previous = frame;
        }
        assert_eq!(resumed.len(), checkpoints.len() + 3);
    }

    fn discovery_seed(
        sources: &crate::replay::Sources,
        lock: &crate::lock::Lock,
        seed: &oracle_record::FrameRecord,
    ) -> (super::Run, super::Scenes) {
        use crate::replay::{intro_rooms, played::frame};
        use game::production::played::{frame::Counts, intro_rooms as rooms};

        let mut power_row = super::super::open_rows(sources, lock, "any-power-on").unwrap();
        let mut previous = power_row.record(0).unwrap();
        let mut began = previous.clone();
        let mut run = super::Run::begin(&sources.rom, &previous).unwrap();

        // Take the chosen file and its loan from their ordinary entry. The local window then
        // admits the preceding scene, actor, latch and frame counts once at the page-two cut.
        for cut in 1..=2220 {
            let current = power_row.record(cut).unwrap();
            run.step(&began, &previous, &current, cut).unwrap();
            if super::super::finished(&current) {
                began = current.clone();
            }
            previous = current;
        }

        let mut scenes = super::Scenes::import(&sources.rom, super::Part::Delivery).unwrap();
        let super::front_end::State::Screens(screens) = &mut run.state else {
            panic!("intro front end");
        };
        let super::front_end::Screen::Intro(intro) = &mut screens.screen else {
            panic!("intro playing");
        };
        let appearance = crate::replay::appearance::Content::load(&sources.rom).unwrap();
        intro.scene = scenes.page_two_seed(seed).unwrap();
        intro.actors.rooms = Some(rooms::State::new(
            intro_rooms::actor(&seed.wram, &appearance).unwrap(),
            None,
        ));
        intro.actors.battle = None;
        screens.counts = Counts {
            finished_frames: super::word(&seed.wram, super::source::NMI_FRAME_COUNTER_WORD)
                .unwrap(),
            random: frame::random(&seed.wram).unwrap(),
        };

        (run, scenes)
    }

    fn check_discovery_actor(
        cut: u64,
        finished: bool,
        intro: &super::front_end::PlayingIntro,
        current: &oracle_record::FrameRecord,
    ) {
        use crate::replay::{discovery, intro_rooms};
        use game::production::played::intro_rooms as rooms;
        use types::discovery::DiscoveryOrder;

        let discovery = intro.actors.discovery.as_ref().unwrap();
        let room = intro.actors.rooms.as_ref().unwrap();
        if cut == discovery::SEED + 1 {
            assert!(!finished);
            assert!(room.pending());
            assert_eq!(room.active(), Some(rooms::Start::Discovery));
            assert_eq!(
                intro.scene.intro.stage,
                types::intro::IntroStage::IntoDiscovery
            );
        }
        if cut == discovery::SEED + 2 {
            assert!(finished);
            assert!(!room.pending());
        }
        if discovery.encounter().order != DiscoveryOrder::Hidden || cut == discovery::END - 1 {
            let (centre, pad) = intro_rooms::discovery_actor(&current.wram).unwrap();
            assert!(
                room.actor.samus.movement.0.position == centre,
                "discovery room position at {cut}"
            );
            assert!(
                room.actor.carry.0.demo == pad,
                "discovery demo controls at {cut}"
            );
        }
    }

    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn discovery_plays_from_page_two_and_resumes_across_interruptions() {
        use crate::replay::discovery;
        use std::io::Write as _;

        let sources = crate::replay::played::tests::sources();
        let lock = super::super::read_lock(&sources).unwrap();
        let mut rows = super::super::open_rows(&sources, &lock, discovery::ROW).unwrap();
        let mut previous = rows.record(discovery::SEED).unwrap();
        let (mut run, mut scenes) = discovery_seed(&sources, &lock, &previous);
        let mut began = previous.clone();
        let mut controller = super::Controller::seeded(&previous.wram).unwrap();
        let mut witness = discovery::PlayedWitness::default();
        let mut restored = Vec::new();
        for cut in discovery::SEED + 1..=discovery::END {
            let current = rows.record(cut).unwrap();
            let finished = super::super::finished(&current);
            let pressed = super::screens::pressed(controller);
            let answers = super::Borrowed::read(&began, &previous, &current).unwrap();
            let end = super::FrameEnd {
                work: if finished {
                    super::Work::Finished(pressed)
                } else {
                    super::Work::Unfinished
                },
                upload: super::DriverUpload::Continuing,
            };
            let advance = |state: &mut super::front_end::State,
                           cartridge: &mut super::Cartridge| {
                state
                    .step_with_cartridge(
                        cartridge,
                        &run.format,
                        &super::CartridgeFrame {
                            end,
                            in_flight: pressed,
                            rules: &super::RULES,
                            content: &run.content,
                            platform: &answers,
                        },
                    )
                    .unwrap_or_else(|fault| panic!("discovery production cut {cut}: {fault:?}"))
            };
            let sound = advance(&mut run.state, &mut run.cartridge);
            for (state, cartridge) in &mut restored {
                assert!(advance(state, cartridge) == sound, "sound at {cut}");
                assert!(*state == run.state, "front end at {cut}");
                assert!(*cartridge == run.cartridge, "cartridge at {cut}");
            }
            let super::front_end::State::Screens(screens) = &run.state else {
                panic!("discovery front end at {cut}");
            };
            let super::front_end::Screen::Intro(intro) = &screens.screen else {
                panic!("discovery intro at {cut}");
            };
            let (expected, actual) = scenes.compared(&current, &intro.scene).unwrap();
            super::differs(&expected, &actual, cut).unwrap();
            let discovery = intro.actors.discovery.as_ref().unwrap();
            check_discovery_actor(cut, finished, intro, &current);
            let publication = (finished || cut == discovery::SEED + 1)
                .then(|| discovery.publication.as_ref().unwrap());
            let sent = sound.into_iter().filter_map(|offer| {
                let super::front_end::Sound::Discovery(offer) = offer else {
                    return None;
                };
                Some(offer)
            });
            let expected_offers = publication
                .into_iter()
                .flat_map(|cut| cut.sounds.iter().copied());
            assert!(sent.eq(expected_offers), "discovery sent sounds at {cut}");
            witness
                .check(&previous, &current, discovery.encounter(), publication, cut)
                .unwrap();
            if !finished {
                restored.push((run.state.clone(), run.cartridge.clone()));
            }
            controller = controller.read(current.pad.0, super::polled(&current));
            if finished {
                began = current.clone();
            }
            previous = current;
        }
        assert_eq!(restored.len(), 3);
        writeln!(
            std::io::stdout().lock(),
            "DISCOVERY_LOCAL cuts={} interrupted_checkpoints={} end={}",
            discovery::END.saturating_sub(discovery::SEED),
            restored.len(),
            discovery::END,
        )
        .unwrap();
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
