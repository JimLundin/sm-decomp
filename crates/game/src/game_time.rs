//! The file's play time, counted once per gameplay frame.

use crate::state::GameTime;

/// Frames in a second, seconds in a minute and minutes in an hour.
const PER_UNIT: u16 = 60;
/// The first hour the clock can't show.
const HOURS_SHOWN: u16 = 100;

/// The time the clock stops at: 99:59:59 and 59 frames.
pub const CAP: GameTime = GameTime {
    hours: HOURS_SHOWN.wrapping_sub(1),
    minutes: PER_UNIT.wrapping_sub(1),
    seconds: PER_UNIT.wrapping_sub(1),
    frames: PER_UNIT.wrapping_sub(1),
};

impl GameTime {
    /// One frame of play.
    ///
    /// Bank `$82` log, `$82:DB69`: frames, then seconds, then minutes count up and roll to
    /// zero at 60, each carrying into the next; from 100 hours the clock holds at [`CAP`].
    /// The original compares signed, so a unit at 8000h or above never rolls over.
    pub const fn tick(&mut self) {
        let carries = roll(&mut self.frames) && roll(&mut self.seconds) && roll(&mut self.minutes);
        if carries {
            self.hours = self.hours.wrapping_add(1);
        }
        if self.hours.wrapping_sub(HOURS_SHOWN).cast_signed() >= 0 {
            *self = CAP;
        }
    }
}

/// Count `unit` up, rolling it to zero and carrying when it reaches a full unit.
const fn roll(unit: &mut u16) -> bool {
    *unit = unit.wrapping_add(1);
    let carries = unit.wrapping_sub(PER_UNIT).cast_signed() >= 0;
    if carries {
        *unit = 0;
    }
    carries
}

#[cfg(test)]
mod tests {
    use super::CAP;
    use crate::state::GameTime;

    const fn time(hours: u16, minutes: u16, seconds: u16, frames: u16) -> GameTime {
        GameTime {
            hours,
            minutes,
            seconds,
            frames,
        }
    }

    fn ticked(mut at: GameTime) -> GameTime {
        at.tick();
        at
    }

    #[test]
    fn frames_count_and_carry_through_every_unit() {
        assert_eq!(ticked(time(0, 0, 22, 33)), time(0, 0, 22, 34));
        assert_eq!(ticked(time(0, 0, 22, 59)), time(0, 0, 23, 0));
        assert_eq!(ticked(time(0, 12, 59, 59)), time(0, 13, 0, 0));
        assert_eq!(ticked(time(3, 59, 59, 59)), time(4, 0, 0, 0));
    }

    #[test]
    fn the_clock_stops_at_the_cap() {
        assert_eq!(ticked(time(99, 59, 59, 58)), CAP);
        assert_eq!(ticked(CAP), CAP);
        assert_eq!(ticked(time(150, 0, 0, 0)), CAP);
    }

    #[test]
    fn a_unit_whose_difference_is_negative_never_rolls() {
        // `CMP #60; BMI` tests the sign of the unit minus 60, not the unit: from 803Ch up the
        // difference is negative, so the unit counts on, but 8001h minus 60 is positive and rolls.
        assert_eq!(ticked(time(0, 0, 0, 0xFFF0)), time(0, 0, 0, 0xFFF1));
        assert_eq!(ticked(time(0, 0, 0, 0x8000)), time(0, 0, 1, 0));
        // The hours cap is the same signed test against 100.
        assert_eq!(ticked(time(0xFFF0, 0, 0, 0)), time(0xFFF0, 0, 0, 1));
        assert_eq!(ticked(time(0x8000, 0, 0, 0)), CAP);
    }

    #[test]
    fn a_unit_already_past_sixty_rolls_on_the_next_frame() {
        assert_eq!(ticked(time(0, 0, 0, 70)), time(0, 0, 1, 0));
    }
}
