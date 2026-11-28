#![cfg(not(doctest))]
//! The screens-options model, built from its behaviour spec alone: the options menu from the
//! dark frame that ends file select until the menu hands on, one frame end at a time.
//!
//! The State is `types::options::OptionsState`. Each frame end, `step` advances it and returns
//! what the frame asks to play; once the menu has handed on, a frame end belongs to the piece
//! that follows, and `step` says so.

mod catalogue;
mod choosing;
mod cursor;
mod dissolve;
mod fade;
mod frame;
mod leave;
mod load;
mod special;

pub use catalogue::{Behaviour, BehaviourId, behaviour};
pub use frame::{ChosenFile, FrameEnd, MenuAction, MenuSound, NextPiece, Pressed};
use types::options::{OptionsStage, OptionsState, WayOut};

/// One frame end of the options menu: the State advanced, and the sounds asked for, oldest first.
///
/// A frame end with no frame finished changes nothing. `file` is how the chosen file begins,
/// which start game reads.
///
/// # Errors
///
/// After a way out, the frame end belongs to the piece that follows: `NextPiece` names it, and
/// the State is left as it is.
pub fn step(
    state: &mut OptionsState,
    frame_end: FrameEnd,
    file: ChosenFile,
) -> Result<Vec<MenuSound>, NextPiece> {
    if let OptionsStage::HandedOn(way_out) = state.stage {
        return Err(NextPiece(way_out));
    }
    let mut sounds = Vec::new();
    if let FrameEnd::Finished(pressed) = frame_end {
        state.stage = finished_frame(state, pressed, file, &mut sounds);
    }
    Ok(sounds)
}

/// The stage after a finished frame, each stage by its own entry.
fn finished_frame(
    state: &mut OptionsState,
    pressed: Pressed,
    file: ChosenFile,
    sounds: &mut Vec<MenuSound>,
) -> OptionsStage {
    let OptionsState {
        stage,
        screen,
        settings,
    } = state;
    match *stage {
        OptionsStage::Loading(load_step) => load::load(load_step),
        OptionsStage::Brightening(cursor) => load::brighten(screen, cursor),
        OptionsStage::Choosing(cursor) => choosing::choose(cursor, pressed, file, settings, sounds),
        OptionsStage::Special(cursor) => {
            special::special_settings(cursor, pressed, settings, sounds)
        }
        OptionsStage::DissolvingOut(dissolving) => dissolve::darken(screen, dissolving),
        OptionsStage::DissolvingIn(dissolving) => dissolve::brighten(screen, dissolving),
        OptionsStage::Leaving { begins, .. } => {
            leave::darken(screen, *stage, leave::into_game(begins))
        }
        OptionsStage::Returning(_) => leave::darken(screen, *stage, WayOut::FileSelectAgain),
        OptionsStage::HandedOn(way_out) => OptionsStage::HandedOn(way_out),
    }
}
