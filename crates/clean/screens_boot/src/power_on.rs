//! One frame end of the power-on sequence: black, the logo brightening, held and dimming, black
//! while the sound driver loads and the game settles, then the title.

use types::power_on::{FinishedFrames, HoldFrames, Level, PowerOnStage, PowerOnState, ScreenLight};

use crate::fade::Fade;
use crate::frame::{DriverUpload, FrameEnd, FrameWork, Outcome};
use crate::rules::Rules;

/// The power-on sequence at one frame end.
///
/// The count of finished frames moves first (`FramesCounted`), then the stage's rule reads it;
/// an upload that ended in the frame sets the count back to 0 after both (`SoundLoads`). A frame
/// whose work did not finish changes nothing while the logo shows or the game settles.
#[must_use]
pub const fn step(rules: Rules, state: PowerOnState, frame: FrameEnd) -> Outcome {
    let frames = frames_counted(state.frames, frame.work);
    let level = state.screen.level();
    let next = match (state.stage, frame.work) {
        (PowerOnStage::Settling, FrameWork::Finished) => return Outcome::TitleBegins,
        (
            PowerOnStage::Brightening
            | PowerOnStage::Held(_)
            | PowerOnStage::Dimming
            | PowerOnStage::Settling,
            FrameWork::Unfinished,
        ) => state,
        (PowerOnStage::BeforeLogo, FrameWork::Unfinished) => quiet_until_the_logo(frames),
        (PowerOnStage::BeforeLogo, FrameWork::Finished) => {
            Fade::Brighten.step(rules, Level::DARK, frames)
        }
        (PowerOnStage::Brightening, FrameWork::Finished) => {
            Fade::Brighten.step(rules, level, frames)
        }
        (PowerOnStage::Held(left), FrameWork::Finished) => logo_held(rules, left, frames),
        (PowerOnStage::Dimming, FrameWork::Finished) => Fade::Dim.step(rules, level, frames),
        (PowerOnStage::SoundLoading, FrameWork::Finished | FrameWork::Unfinished) => {
            sound_loads(frame.upload, frames)
        }
    };
    Outcome::PowerOn(next)
}

/// Spec: `FramesCounted`. A finished frame adds one to the count, wrapping from 65,535 to 0; a
/// frame whose work did not finish adds none. (Once the game settles, its finished frame is the
/// title's, and the title counts it.)
const fn frames_counted(frames: FinishedFrames, work: FrameWork) -> FinishedFrames {
    match work {
        FrameWork::Finished => FinishedFrames(frames.0.wrapping_add(1)),
        FrameWork::Unfinished => frames,
    }
}

/// Spec: `QuietUntilTheLogo`. Until the logo's first frame finishes, the screen stays blank at
/// full level.
const fn quiet_until_the_logo(frames: FinishedFrames) -> PowerOnState {
    PowerOnState {
        stage: PowerOnStage::BeforeLogo,
        screen: ScreenLight::Blank(Level::FULL),
        frames,
    }
}

/// Spec: `LogoHeld`. At full, each finished frame takes one from the hold left; a finished frame
/// with none left starts the dimming from full (`LogoDims`).
const fn logo_held(rules: Rules, left: HoldFrames, frames: FinishedFrames) -> PowerOnState {
    match left.0.checked_sub(1) {
        Some(still_left) => PowerOnState {
            stage: PowerOnStage::Held(HoldFrames(still_left)),
            screen: ScreenLight::Lit(Level::FULL),
            frames,
        },
        None => Fade::Dim.step(rules, Level::FULL, frames),
    }
}

/// Spec: `SoundLoads`. Black at full level while the sound driver loads, whether or not the
/// frame finished; once its upload ends within a frame, the game settles and the count of
/// finished frames starts again from 0.
const fn sound_loads(upload: DriverUpload, frames: FinishedFrames) -> PowerOnState {
    match upload {
        DriverUpload::Continuing => PowerOnState {
            stage: PowerOnStage::SoundLoading,
            screen: ScreenLight::Blank(Level::FULL),
            frames,
        },
        DriverUpload::Ended => PowerOnState {
            stage: PowerOnStage::Settling,
            screen: ScreenLight::Blank(Level::FULL),
            frames: FinishedFrames::ZERO,
        },
    }
}
