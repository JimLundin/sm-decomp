//! The Gripper, Ripper II and Ripper, flying back and forth (bank $A2:E17B-$A2:E57A).
#![forbid(unsafe_code)]
use crate::actors::host::{ActorLease, ActorRuntime, LeaseActors, Shot};

/// Which enemy: $D3FF, $D43F or $D47F.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Gripper,
    RipperII,
    Ripper,
}

/// The lists moving left and right, by kind; the bank log's labels for Ripper II's are swapped.
const INSTRUCTION_LISTS: [[u16; 2]; 3] = [[0xe19b, 0xe1af], [0xe2e0, 0xe2f4], [0xe48b, 0xe477]];

// Lanes 0 and 1 are the Gripper's left and right bounds.
const X_SUBVELOCITY: usize = 2;
const X_VELOCITY: usize = 3;
const SPEED_INDEX: usize = 4;

#[path = "rippers_owned.rs"]
pub mod owned;

pub fn initialize(kind: Kind, runtime: &mut impl ActorLease) {
    owned::initialize(kind, runtime);
}
pub fn main_ai(kind: Kind, runtime: &mut impl ActorLease) {
    owned::main_ai(kind, runtime);
}
/// Preserve the original external reaction, then acquire its fresh actor lease.
pub fn shot<H: LeaseActors + ActorRuntime + ?Sized>(game: &mut H) {
    game.shot_reaction(Shot::Finalize);
    owned::finish_shot(&mut game.lease());
}
