//! The front end: which part of the game runs ahead of room play, one frame end at a time.
//!
//! From the console's power-on the boot runs (screens-boot's power-on: black, the logo brightening,
//! held and dimming, then black while the sound driver loads and the game settles) until the
//! title begins. The title and every part between it and room play are later pieces, so a run
//! starts either here, from power-on, or in room play, from a room's seed; room play joins this
//! State once the parts before it hand on.

use screens_boot::{FrameEnd, Outcome, PowerOnState, Rules};

/// Which part of the game runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// The console's power-on, until the title begins.
    PowerOn(PowerOnState),
}

/// Why a frame end was not played. State is unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The frame is the title's first: the title is a later piece.
    TitleBegins,
}

impl State {
    /// The console just switched on.
    pub const AT_POWER_ON: Self = Self::PowerOn(PowerOnState::AT_POWER_ON);

    /// The power-on's State. The power-on is the only part the front end runs yet; once
    /// another joins, this is where a caller learns which part runs.
    #[must_use]
    pub const fn power_on(self) -> PowerOnState {
        let Self::PowerOn(power_on) = self;
        power_on
    }

    /// Play one frame end of the part that runs.
    ///
    /// # Errors
    /// [`Fault::TitleBegins`] on the first frame that finishes once the game has settled.
    pub const fn step(&mut self, frame: FrameEnd, rules: Rules) -> Result<(), Fault> {
        let Self::PowerOn(power_on) = self;
        match screens_boot::step(rules, *power_on, frame) {
            Outcome::PowerOn(next) => {
                *power_on = next;
                Ok(())
            }
            Outcome::TitleBegins => Err(Fault::TitleBegins),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Fault, State};
    use screens_boot::{
        DriverUpload, FinishedFrames, FrameEnd, FrameWork, Level, PowerOnStage, Rules, ScreenLight,
    };

    const FINISHED: FrameEnd = FrameEnd {
        work: FrameWork::Finished,
        upload: DriverUpload::Continuing,
    };
    const UNFINISHED: FrameEnd = FrameEnd {
        work: FrameWork::Unfinished,
        upload: DriverUpload::Continuing,
    };

    #[test]
    fn the_boot_runs_from_power_on_until_the_title_begins() {
        let mut state = State::AT_POWER_ON;
        for _ in 0..17 {
            state.step(UNFINISHED, Rules::REFERENCE).unwrap();
        }
        assert_eq!(state.power_on().stage, PowerOnStage::BeforeLogo);
        state.step(FINISHED, Rules::REFERENCE).unwrap();
        let lit = state.power_on();
        assert_eq!(lit.screen, ScreenLight::Lit(Level::new(1).unwrap()));
        assert_eq!(lit.frames, FinishedFrames(1));
        // The logo brightens, holds and dims on finished frames, then the sound loads.
        while state.power_on().stage != PowerOnStage::SoundLoading {
            state.step(FINISHED, Rules::REFERENCE).unwrap();
        }
        state.step(UNFINISHED, Rules::REFERENCE).unwrap();
        let ended = FrameEnd {
            work: FrameWork::Unfinished,
            upload: DriverUpload::Ended,
        };
        state.step(ended, Rules::REFERENCE).unwrap();
        assert_eq!(state.power_on().stage, PowerOnStage::Settling);
        assert_eq!(state.power_on().frames, FinishedFrames::ZERO);
        let settled = state;
        assert_eq!(
            state.step(FINISHED, Rules::REFERENCE),
            Err(Fault::TitleBegins)
        );
        assert_eq!(state, settled, "a refused frame changes nothing");
    }
}
