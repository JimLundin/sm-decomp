//! What the run borrows from the platform and audio: the pad as the game reads it, whether each
//! frame of work finished, and whether a track waits in the queue of tracks.

use super::word;
use crate::replay::{data_failure, result::Outcome};
use crate::source;
use oracle_record::{Cut, FrameRecord, NmiKind};
use types::Parity;
use world_doors::{Doors, Fetching, Passage, Phase};

/// The controller as the game sees it: the pad read by the last polling NMI and the
/// buttons that went down at that read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(in crate::replay) struct Controller {
    pub(in crate::replay) held: u16,
    pub(in crate::replay) pressed: u16,
}

impl Controller {
    /// The controller as the seed leaves it: the game's own last read and the buttons that
    /// went down at it. A button held through the seed went down before it, not on cut 1.
    pub(in crate::replay) fn seeded(wram: &[u8]) -> Result<Self, Outcome> {
        Ok(Self {
            held: word(wram, source::JOYPAD1_LASTKEYS)?,
            pressed: word(wram, source::JOYPAD1_NEWKEYS)?,
        })
    }

    /// Check the held buttons and fresh edges carried into this turn against the last polling NMI.
    pub(super) fn check_poll(self, wram: &[u8], cut: u64) -> Result<(), Outcome> {
        if word(wram, source::JOYPAD1_LASTKEYS)? != self.held {
            return Err(data_failure(None, "played pad alignment"));
        }
        if word(wram, source::JOYPAD1_NEWKEYS)? != self.pressed {
            return Err(Outcome::Difference {
                cut,
                field: "controller.pressed".to_owned(),
            });
        }
        Ok(())
    }

    /// A polling NMI latches the new pad; a lag NMI leaves the previous read in place.
    pub(super) const fn read(self, pad: u16, polled: bool) -> Self {
        if polled {
            Self {
                held: pad,
                pressed: pad & !self.held,
            }
        } else {
            self
        }
    }
}

/// Her parity: that of the count of finished frames as the frame begins.
pub(super) const fn parity(counter: u16) -> Parity {
    if counter & 1 == 0 {
        Parity::Even
    } else {
        Parity::Odd
    }
}

pub(in crate::replay) const fn polled(record: &FrameRecord) -> bool {
    matches!(record.cut, Cut::NmiReturn(nmi) if nmi.polled)
}

/// The borrowed lag: the NMI found the frame's work finished, waiting for it. A lag NMI finds
/// the work part-done, whether or not it reads the pad.
pub(in crate::replay) const fn finished(record: &FrameRecord) -> bool {
    matches!(record.cut, Cut::NmiReturn(nmi) if matches!(nmi.kind, NmiKind::Requested))
}

/// The interrupt cursor in the old population's clearing loops. Slot identities
/// are retired by their owner's original indices, independently of the observed roster.
pub(super) fn actor_fetch(record: &FrameRecord) -> game::production::played::passage::ActorFetch {
    use game::production::played::passage::ActorFetch;
    match record.cut {
        Cut::NmiReturn(nmi) if matches!(nmi.interrupted_pc, 0x00a0_8a7f..=0x00a0_8a87) => {
            // The clear writes a word, then advances its cursor twice. Count a slot
            // as retired as soon as its identity word was written.
            let ahead = match nmi.interrupted_pc {
                0x00a0_8a83 => 2,
                0x00a0_8a84 | 0x00a0_8a85 => 1,
                _ => 0,
            };
            ActorFetch::Clearing(usize::from(record.cpu.x).saturating_add(ahead).div_ceil(64))
        }
        Cut::NmiReturn(nmi) if matches!(nmi.interrupted_pc, 0x00a0_8aa9..=0x00a0_8ab0) => {
            ActorFetch::Graphics
        }
        Cut::NmiReturn(_) | Cut::VblankWithoutNmi | Cut::NmiStillRunning { .. } => {
            ActorFetch::Place
        }
    }
}

// The last tile strip of Ridley's room is already visible while the IRQ upload call
// returns to the main thread. The NMI that samples it has since reused DMA channel 1.
pub(super) fn door_enemy_tiles(
    previous: &FrameRecord,
    current: &FrameRecord,
) -> game::production::played::passage::EnemyTileTransfer {
    use game::production::played::passage::EnemyTileTransfer;
    let ready = |record: &FrameRecord| {
        matches!(record.cut, Cut::NmiReturn(nmi) if matches!(nmi.interrupted_pc, 0x0082_e02c..=0x0082_e02f))
            && word(&record.wram, source::ROOM_PTR).is_ok_and(|room| room == 0xe0b5)
            && record.wram.get(0x5be..0x5c0) == Some(&0x6d00_u16.to_le_bytes())
            && record.wram.get(0x5c3..0x5c5) == Some(&0x0600_u16.to_le_bytes())
    };
    if ready(current) && !ready(previous) {
        EnemyTileTransfer::Published
    } else {
        EnemyTileTransfer::Waiting
    }
}

/// The borrowed lag of the contents' fetch: done on its first frame. The fetch's work waits on
/// its own uploads, so every NMI through it is a lag NMI and none says when its colours were
/// set; the record shows them set within the slide, and the visor lit as the slide ends, which
/// only a fetch done before the slide's end gives. Its first frame is the earliest such.
pub(super) const fn fetching_contents(doors: Doors) -> bool {
    matches!(
        doors,
        Doors::Passing(Passage {
            phase: Phase::Arriving {
                fetching: Fetching::Contents,
                ..
            },
            ..
        })
    )
}

/// The contents' fetch under way: from its first frame until the slide ends, the frame of work
/// is held, so every NMI through it finds the fetch part-done.
pub(super) const fn fetch_under_way(doors: Doors) -> bool {
    matches!(
        doors,
        Doors::Passing(Passage {
            phase: Phase::Arriving {
                fetching: Fetching::Contents | Fetching::Done,
                ..
            },
            ..
        })
    )
}

/// The music queue's timers, its start index and the music timer.
const MUSIC_TIMERS: usize = 0x0629;
const MUSIC_QUEUE_START: source::WramWord = source::WramWord {
    name: "MUSIC_QUEUE_START",
    offset: 0x063B,
};
const MUSIC_TIMER: source::WramWord = source::WramWord {
    name: "MUSIC_TIMER",
    offset: 0x063F,
};
/// The entries the music queue holds.
const MUSIC_ENTRIES: usize = 8;

/// Audio's answer, borrowed: no track waits in the queue of tracks as the frame of work begins.
/// The main loop's start steps the queue once: with the music timer at 1 the entry at the start
/// index leaves; the music is ready when every entry's timer is then 0.
pub(super) fn music_ready(wram: &[u8]) -> Result<bool, Outcome> {
    let bad = || data_failure(None, "played music queue");
    let mut timers = wram
        .get(MUSIC_TIMERS..)
        .and_then(|queue| queue.as_chunks::<2>().0.get(..MUSIC_ENTRIES))
        .ok_or_else(bad)?
        .iter()
        .map(|word| u16::from_le_bytes(*word))
        .collect::<Vec<_>>();
    if word(wram, MUSIC_TIMER)? == 1 {
        let start = usize::from(word(wram, MUSIC_QUEUE_START)? / 2) % MUSIC_ENTRIES;
        if let Some(leaving) = timers.get_mut(start) {
            *leaving = 0;
        }
    }
    Ok(timers.iter().all(|&timer| timer == 0))
}

/// The three sound libraries' queue start indices, and after them their next indices.
const SOUND_QUEUE_STARTS: usize = 0x0643;
const SOUND_QUEUE_NEXTS: usize = 0x0646;
const SOUND_LIBRARIES: usize = 3;

/// Audio's answer, borrowed: every sound request has been sent as the frame of work begins.
/// The sounds are sent after the frame's game work, so a frame reads the queues as the last
/// one left them: each library's next index at its start index.
pub(super) fn sounds_sent(wram: &[u8]) -> Result<bool, Outcome> {
    let bad = || data_failure(None, "played sound queues");
    let starts = wram
        .get(SOUND_QUEUE_STARTS..SOUND_QUEUE_STARTS + SOUND_LIBRARIES)
        .ok_or_else(bad)?;
    let nexts = wram
        .get(SOUND_QUEUE_NEXTS..SOUND_QUEUE_NEXTS + SOUND_LIBRARIES)
        .ok_or_else(bad)?;
    Ok(starts
        .iter()
        .zip(nexts)
        .all(|(start, next)| next.wrapping_sub(*start) % 16 == 0))
}

#[cfg(test)]
mod tests {
    use super::Controller;

    #[test]
    fn a_polling_read_reports_only_buttons_that_went_down() {
        let first = Controller::default().read(0x0180, true);
        assert_eq!((first.held, first.pressed), (0x0180, 0x0180));
        let second = first.read(0x8180, true);
        assert_eq!((second.held, second.pressed), (0x8180, 0x8000));
        let released = second.read(0x0100, true);
        assert_eq!((released.held, released.pressed), (0x0100, 0));
    }

    #[test]
    fn a_button_held_through_the_seed_is_not_pressed_on_the_first_cut() {
        let mut wram = vec![0; 0x100];
        wram[0x8b..0x8d].copy_from_slice(&0x0180_u16.to_le_bytes());
        wram[0x8f..0x91].copy_from_slice(&0x0100_u16.to_le_bytes());
        let seeded = Controller::seeded(&wram).unwrap();
        assert_eq!((seeded.held, seeded.pressed), (0x0180, 0x0100));
        assert_eq!(seeded.read(0x0180, true).pressed, 0);
    }

    #[test]
    fn a_lag_nmi_keeps_the_previous_read() {
        let read = Controller::default().read(0x0080, true);
        assert_eq!(read.read(0x0100, false), read);
    }
}

/// T1 identifies the frame that begins the fade on the return to the shaft. The haze,
/// mode split and scroll have finished their first full picture by this requested NMI.
pub(super) fn shaft_raster(
    previous: &FrameRecord,
    current: &FrameRecord,
) -> Result<game::production::played::passage::ShaftRaster, Outcome> {
    use game::production::played::passage::ShaftRaster;
    const HANDLER: source::WramWord = source::WramWord {
        name: "door transition handler",
        offset: 0x099c,
    };
    const POSITION_AFTER_SCROLL: u16 = 0xe6a2;
    const FADE_IN: u16 = 0xe737;
    if finished(current)
        && word(&previous.wram, HANDLER)? == POSITION_AFTER_SCROLL
        && word(&current.wram, HANDLER)? == FADE_IN
    {
        Ok(ShaftRaster::Finished)
    } else {
        Ok(ShaftRaster::Pending)
    }
}

/// T1 marks a completed visible interval for Ridley's direct table or the room's effects clear.
pub(super) fn room_hdma(
    previous: &FrameRecord,
    current: &FrameRecord,
) -> Result<game::production::played::passage::RoomHdma, Outcome> {
    use game::production::played::passage::RoomHdma;

    const HANDLER: source::WramWord = source::WramWord {
        name: "door transition handler",
        offset: 0x099c,
    };
    const BUILD_ROOM: u16 = 0xe36e;
    const CHANNEL_FOUR_TABLE: source::WramWord = source::WramWord {
        name: "channel four HDMA object's table",
        offset: 0x18dc,
    };
    const RIDLEY_MAIN_SCREEN_TABLE: u16 = 0xd8fd;
    let level_decode = matches!(
        current.cut,
        Cut::NmiReturn(nmi) if (0x0082_ea73..=0x0082_eb0f).contains(&nmi.interrupted_pc)
    );
    if level_decode && finished(previous) && word(&previous.wram, HANDLER)? == BUILD_ROOM {
        return Ok(RoomHdma::Reset);
    }
    let armed = previous.wram.get(0x85).is_some_and(|mask| mask & 0x10 != 0);
    if finished(current)
        && armed
        && word(&previous.wram, CHANNEL_FOUR_TABLE)? == RIDLEY_MAIN_SCREEN_TABLE
    {
        return Ok(RoomHdma::RidleyMainScreenComplete);
    }
    Ok(RoomHdma::Hold)
}

/// T1 supplies the completed-work boundary during the interrupted shaft fetch.
/// The effects transfer's destination distinguishes its wait from other direct uploads.
/// The slide wait follows the completed library background transfer.
pub(super) fn shaft_background(
    record: &FrameRecord,
    doors: Doors,
) -> game::production::played::room_tiles::ShaftBackground {
    use game::production::played::room_tiles::ShaftBackground;
    if !fetch_under_way(doors) {
        return ShaftBackground::Pending;
    }
    match record.nmi().map(|nmi| nmi.interrupted_pc) {
        Some(0x0082_e06b | 0x0082_e06e)
            if record.wram.get(0x05be..0x05c0) == Some(&[0x80, 0x58]) =>
        {
            ShaftBackground::EffectsPrepared
        }
        Some(0x0082_e591) => ShaftBackground::EffectsTransferred,
        Some(0x0082_e526 | 0x0082_e529) => ShaftBackground::BackgroundTransferred,
        _ => ShaftBackground::Pending,
    }
}

/// Compare the completed shaft writes. The interrupted fill's staging bytes belong to T1.
pub(super) fn compare_shaft_background(
    trace: &super::witnesses::shaft::Background,
    state: game::production::played::room_tiles::ShaftBackground,
    video: &game::state::video::Video,
    record: &FrameRecord,
    cut: u64,
) -> Result<(), Outcome> {
    let difference = |field| Outcome::Difference {
        cut,
        field: format!("shaft-entry.{field}"),
    };
    if cut >= trace.effects_from && !trace.interrupted_shadow.contains(&cut) {
        let shadow = record
            .wram
            .get(0x4000..0x4800)
            .ok_or_else(|| data_failure(Some(cut), "shaft background shadow"))?;
        let words = shadow
            .as_chunks::<2>()
            .0
            .iter()
            .copied()
            .map(u16::from_le_bytes);
        if state
            .shadow_word()
            .is_none_or(|value| words.into_iter().any(|word| word != value))
        {
            return Err(difference("bg1-shadow"));
        }
    }
    if cut >= trace.effects_from {
        let expected = record
            .vram
            .get(0x5880..0x6000)
            .ok_or_else(|| data_failure(Some(cut), "shaft effects VRAM"))?;
        if video.memory.vram.get(0x5880..0x6000) != Some(expected) {
            return Err(difference("effects"));
        }
    }
    if cut >= trace.tiles_from {
        let expected = record
            .vram
            .get(0x4800..0x5000)
            .ok_or_else(|| data_failure(Some(cut), "shaft background VRAM"))?;
        if video.memory.vram.get(0x4800..0x5000) != Some(expected) {
            return Err(difference("background"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod shaft_background_tests {
    use super::*;
    use crate::lock::Lock;
    use std::path::Path;

    #[test]
    fn shaft_background_carries_across_native_fetch_cuts() {
        let Ok(cache) = std::env::var("SM_CACHE") else {
            return;
        };
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let lock =
            Lock::parse(&std::fs::read_to_string(root.join("scenarios/locks/lsnes.toml")).unwrap())
                .unwrap();
        let mut rows =
            super::super::rows::Rows::open(&lock, &root, Path::new(&cache), "chain-any-0010-DF8D")
                .unwrap();
        let fetching = Doors::Passing(Passage {
            phase: Phase::Arriving {
                placed: true,
                fetching: Fetching::Done,
            },
            arrival: None,
            slide_frames: 0,
        });
        let witnesses = rows.witnesses().unwrap();
        let trace = &witnesses.shaft().unwrap().background;
        let mut background = game::production::played::room_tiles::ShaftBackground::default();
        let Ok(path) = std::env::var("SM_ROM") else {
            return;
        };
        let rom = data::rom::Rom::new(std::fs::read(path).unwrap()).unwrap();
        let effects = import::countdown::shaft_effects(rom.lorom()).unwrap();
        let mut video = game::state::video::Video::default();
        let origin = rows.record(221).unwrap();
        video.memory.vram = origin.vram.to_vec();
        for cut in 222..=278 {
            let current = rows.record(cut).unwrap();
            let before = background;
            background.advance(shaft_background(&current, fetching));
            if background != before {
                background.install_background(&mut video, &effects).unwrap();
            }
            compare_shaft_background(trace, background, &video, &current, cut).unwrap();
            if (225..=227).contains(&cut) {
                assert_eq!(
                    video.memory.vram.get(0x4800..0x5000),
                    current.vram.get(0x4800..0x5000),
                    "BG2 holds at {cut}"
                );
            }
            if cut == 227 {
                let prepared = current.wram.get(0x4000..0x4800).unwrap();
                let words: Vec<_> = prepared
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .copied()
                    .map(u16::from_le_bytes)
                    .collect();
                let split = words.iter().position(|&word| word == 0x0338).unwrap();
                assert_eq!(split, 477);
                assert!(words.iter().take(split).all(|&word| word == 0x184e));
                assert!(words.iter().skip(split).all(|&word| word == 0x0338));
            }
        }
    }
}
