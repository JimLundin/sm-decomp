//! What a frame reports (spec §1.4) beyond the state's screens.

use clean_types::Cell;

use crate::cells::Answer;
use crate::reactions::SoundRequest;

/// The frame's outputs, each in order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// `AnswersToHerMotion`: one per touch, in the order of the touches.
    pub answers: Vec<Answer>,
    /// `WhatTheScreenRedraws`: the cells whose look a step changed.
    pub redrawn: Vec<Cell>,
    /// `Sounds`: the sounds the scripts asked for, each with its queue limit.
    pub sounds: Vec<SoundRequest>,
}
