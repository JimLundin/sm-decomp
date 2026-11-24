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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Controller {
    pub(super) held: u16,
    pub(super) pressed: u16,
}

impl Controller {
    /// The controller as the seed leaves it: the game's own last read and the buttons that
    /// went down at it. A button held through the seed went down before it, not on cut 1.
    pub(super) fn seeded(wram: &[u8]) -> Result<Self, Outcome> {
        Ok(Self {
            held: word(wram, source::JOYPAD1_LASTKEYS)?,
            pressed: word(wram, source::JOYPAD1_NEWKEYS)?,
        })
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

pub(super) const fn polled(record: &FrameRecord) -> bool {
    matches!(record.cut, Cut::NmiReturn(nmi) if nmi.polled)
}

/// The borrowed lag: the NMI found the frame's work finished, waiting for it. A lag NMI finds
/// the work part-done, whether or not it reads the pad.
pub(super) const fn finished(record: &FrameRecord) -> bool {
    matches!(record.cut, Cut::NmiReturn(nmi) if matches!(nmi.kind, NmiKind::Requested))
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
