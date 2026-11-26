//! `TailWhips`: aiming for the ordinary Ceres loadout, without missiles.

use types::{Pair, Px};

use crate::ridley_phase::{FlightPhase, RidleyPhase};

const TURN: u16 = 256;
const RIGHT: u16 = 64;
const UP: u16 = 128;
const LEFT: u16 = 192;
const OCTANT_DIVISOR: u16 = 8;
const DOWN_FROM_HER: u16 = 24;
const REACH: u16 = 128;
const CHANCE_FROM: u16 = 240;
const FORWARD_ARC: u16 = 16_128;
const CHANCE_ARC: u16 = 16_384;
const FORBIDDEN_START: u16 = 24;
const FORBIDDEN_END: u16 = 232;

/// The phase after steering and the random number for this run's aim.
#[derive(Clone, Copy)]
pub struct CeresAim {
    pub phase: RidleyPhase,
    pub random: u16,
}

impl CeresAim {
    /// `TailWhips`: only hovering or the first run of an attack can choose a new mark.
    /// The aim starts at the first link's position from the previous run.
    pub fn mark(self, ridley: Pair<Px>, previous_root: Pair<Px>, her: Pair<Px>) -> Option<u16> {
        if !matches!(
            self.phase,
            RidleyPhase::InFlight(FlightPhase::Hovering) | RidleyPhase::Attacking(_)
        ) {
            return None;
        }
        let arc = if self.random % TURN >= CHANCE_FROM {
            CHANCE_ARC
        } else if (her.x - ridley.x).0.cast_signed().unsigned_abs() < REACH {
            FORWARD_ARC
        } else {
            return None;
        };
        let dx = (her.x - previous_root.x).0.cast_signed();
        let dy = her
            .y
            .0
            .wrapping_add(DOWN_FROM_HER)
            .wrapping_sub(previous_root.y.0)
            .cast_signed();
        let bearing = bearing(dx, dy);
        let offset = if (FORBIDDEN_START..FORBIDDEN_END).contains(&bearing) {
            FORBIDDEN_END
        } else {
            bearing
        };
        Some(arc.wrapping_add(offset))
    }

    /// A saved mark can lie in either arc, but not inside its clamped interval.
    pub fn valid_mark(mark: u16) -> bool {
        let bearing = mark % TURN;
        let arc = mark.wrapping_sub(bearing);
        matches!(arc, FORWARD_ARC | CHANCE_ARC)
            && !(FORBIDDEN_START..FORBIDDEN_END).contains(&bearing)
    }
}

/// `TailWhips`: pick the octant using full magnitudes, then take the quotient of the low
/// bytes only. A zero divisor supplies the saturated quotient before the second division.
fn bearing(dx: i16, dy: i16) -> u16 {
    let x = dx.unsigned_abs();
    let y = dy.unsigned_abs();
    let u = (x.min(y) % TURN)
        .saturating_mul(TURN)
        .checked_div(x.max(y) % TURN)
        .unwrap_or(u16::MAX)
        / OCTANT_DIVISOR;
    let (axis, subtract): (u16, bool) = match (y >= x, dx >= 0, dy >= 0) {
        (true, true, true) => (0, false),
        (true, true, false) => (UP, true),
        (true, false, true) => (0, true),
        (true, false, false) => (UP, false),
        (false, true, true) => (RIGHT, true),
        (false, true, false) => (RIGHT, false),
        (false, false, true) => (LEFT, false),
        (false, false, false) => (LEFT, true),
    };
    let raw = if subtract {
        axis.wrapping_sub(u)
    } else {
        axis.wrapping_add(u)
    };
    raw % TURN
}
