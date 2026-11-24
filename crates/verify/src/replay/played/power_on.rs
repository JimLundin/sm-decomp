//! A run from the console's power-on: the front end's boot, compared at each cut until the title.
//!
//! The run admits the power-on State itself and reads nothing of cut 0: its work RAM is the
//! emulator's power-on fill, and its save memory is the movie's starting save memory, which the
//! save files' owner will read as the game settles. Each cut borrows the inputs [`BORROWED`] names.
//!
//! Compared at each cut: while the logo is still to come, only whether the screen is blank, as
//! the level under the blank first comes from the platform's power-on value; after that the
//! screen, blank or lit, and its level. The count of finished frames at finished cuts and while
//! the sound loads, and once the game settles from the third cut after the upload's end: the
//! settle clears the count's memory through the upload's cut and the two after it. The hold's
//! frames left while the logo is held before and after the frame.

use super::{Report, Stop, rows::Rows, word};
use crate::replay::compare::{CompareFault, Observation, compare};
use crate::replay::{data_failure, result::Outcome};
use crate::source;
use game::production::played::front_end;
use oracle_record::{ApuAccess, ApuAccessKind, FrameRecord};
use screens_boot::{
    DriverUpload, FrameEnd, FrameWork, PowerOnStage, PowerOnState, Rules, ScreenLight,
};

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
];

/// Play the power-on row `row` from power-on through `end`. A run from a later cut has no State
/// to admit: nothing reads the power-on's State from a record.
pub(super) fn play(
    rows: &mut Rows<'_>,
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
    };
    if start != 0 {
        let later = Outcome::Unsupported {
            cut: Some(start),
            dependency: "a power-on run from a later cut".into(),
        };
        return Ok(report(0, Stop::At(later)));
    }
    let mut run = Run {
        state: front_end::State::AT_POWER_ON,
        ended: None,
    };
    let mut held = 0;
    let mut previous: Option<u32> = None;
    for cut in 1..=end {
        let current = rows.record(cut)?;
        if previous.is_some_and(|frame| current.video_frame != frame.wrapping_add(1)) {
            return Err(data_failure(Some(cut), "played rows join"));
        }
        if let Err(outcome) = run.step(&current, cut) {
            return Ok(report(held, Stop::At(outcome)));
        }
        held = held.saturating_add(1);
        previous = Some(current.video_frame);
    }
    Ok(report(held, Stop::End))
}

/// The front end's State and the cut the upload ended at.
struct Run {
    state: front_end::State,
    ended: Option<u64>,
}

impl Run {
    /// Play the interval ending at `cut` and compare the boot with the record at `cut`.
    ///
    /// An upload's end outside the sound's load is bad data, and the game settles only on one,
    /// so a run that reaches the title found exactly one.
    fn step(&mut self, record: &FrameRecord, cut: u64) -> Result<(), Outcome> {
        let before = self.state.power_on();
        let ended = upload_ended(&record.apu_log);
        if ended && before.stage != PowerOnStage::SoundLoading {
            return Err(data_failure(
                Some(cut),
                "power-on upload end outside the load",
            ));
        }
        let finished = super::finished(record);
        let frame = FrameEnd {
            work: if finished {
                FrameWork::Finished
            } else {
                FrameWork::Unfinished
            },
            upload: if ended {
                DriverUpload::Ended
            } else {
                DriverUpload::Continuing
            },
        };
        self.state
            .step(frame, RULES)
            .map_err(|fault| Outcome::Unsupported {
                cut: Some(cut),
                dependency: format!("played frame {fault:?}"),
            })?;
        if ended {
            self.ended = Some(cut);
        }
        let after = self.state.power_on();
        let compared = Compared::at(before, after, finished, self.ended, cut);
        let expected = compared.observations(&Fields::recorded(record)?);
        let actual = compared.observations(&Fields::played(after));
        compare(&expected, &actual).map_err(|fault| match fault {
            CompareFault::Difference(field) => Outcome::Difference { cut, field },
            CompareFault::EmptyExpected
            | CompareFault::DuplicateField(_)
            | CompareFault::MissingActual(_) => data_failure(Some(cut), "played comparison schema"),
        })
    }
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

    /// The any% movie from power-on: every cut of the boot holds, and the title's first frame,
    /// cut 271, is not owned.
    #[test]
    #[ignore = "pinned record cache"]
    fn power_on_played_alone() {
        use std::io::Write as _;
        let path = |name| std::env::var_os(name).unwrap().into();
        let sources = crate::replay::Sources {
            cache: path("SM_CACHE"),
            lock: path("SM_LOCK"),
            ..Default::default()
        };
        let report = super::super::play(&sources, "any-power-on", 0, 567).unwrap();
        writeln!(
            std::io::stdout().lock(),
            "PLAYED_ALONE row={} start={} held={} stop={:?}",
            report.row,
            report.start,
            report.held,
            report.stop
        )
        .unwrap();
        assert_eq!(report.held, 270);
        assert!(matches!(
            report.stop,
            super::Stop::At(crate::replay::result::Outcome::Unsupported { cut: Some(271), .. })
        ));
    }
}
