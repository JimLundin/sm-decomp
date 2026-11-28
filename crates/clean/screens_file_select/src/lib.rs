#![cfg(not(doctest))]
//! The screens-file-select model, built from its behaviour spec alone.
//!
//! File select's main menu, one frame end at a time: black while it loads, the menu fading in,
//! the cursor moved with Up and Down, a file chosen with A or Start and its helmet turning, and
//! the menu fading out as the options menu begins. Its State is `types::file_select`'s
//! `FileSelectState`; `frame_end` steps it, reading the slots from the caller's `SaveMemory` and
//! reporting the sounds asked for and whether the frame of work goes on.

mod catalogue;
mod choosing;
mod confirming;
mod cursor;
mod fade;
mod frame;
mod input;
mod load;
mod report;
mod rules;

pub use catalogue::{Behaviour, BehaviourId, behaviour, catalogue};
pub use frame::frame_end;
pub use input::{FrameEnd, MenuAction, SaveMemory};
pub use report::{FrameOfWork, FrameReport, MenuSound, Unsupported};
pub use rules::Rules;
