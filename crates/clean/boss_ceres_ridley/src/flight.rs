//! His flight: once he has taken off he rises fast, slows, and hovers near the top right of the
//! room, his speed drawn to a point of his own, until she is nearly out of energy and he gives
//! up or he turns to an attack; every run he moves by his speed, kept within the room's bounds,
//! and meeting its left side fast he shakes the room.
//!
//! His position is whole px with a fraction in 1/256 px on each axis; his speed is in 1/256 px a
//! run, signed, x rightward and y downward.

use core::num::Wrapping;

use types::{Pair, Px};

use crate::ridley_phase::{Attack, FlightPhase, GiveUp, RidleyPhase};
use crate::station::{Shake, ShakeLayers, ShakeMotion, ShakeStart};

/// A speed along one axis, in 1/256 px a run, signed 16-bit: its arithmetic wraps mod 2^16.
pub type Speed = Wrapping<i16>;

/// No speed on either axis.
pub const STILL: Pair<Speed> = Pair {
    x: Wrapping(0),
    y: Wrapping(0),
};

/// `Rises`: the rise's pull, 16/256 px a run more upward each run.
const RISE_PULL: Speed = Wrapping(16);
/// `Rises`: once above this height, y 112, he starts to slow.
const SLOWS_ABOVE: Px = Wrapping(112);
/// `SlowsDown`: the slowing, 20/256 px a run more downward each run.
const SLOWING: Speed = Wrapping(20);
/// `SlowsDown`: once above this height, y 80, he hovers from the next run.
const HOVERS_ABOVE: Px = Wrapping(80);

/// `Hovers`: his point, the place he hovers at and is drawn to.
pub const HIS_POINT: Pair<Px> = Pair {
    x: Wrapping(192),
    y: Wrapping(100),
};
/// `Hovers`: he is near his point within 16 px of it on both axes.
const NEAR_HIS_POINT: u16 = 16;
/// `Hovers`: her energy's limit: under 30 he gives up.
const HER_ENERGY_LIMIT: u16 = 30;
/// `Hovers`: his hits' limit: from her 100th hit on him he gives up in his fake retreat.
const HIS_HITS_LIMIT: u16 = 100;
/// `Hovers`: his patience: the runs he hovers without reaching his point before he attacks.
const PATIENCE: u8 = 124;
/// `Hovers`: the attack is chosen by the random number mod 16.
const ATTACK_ROLLS: u16 = 16;
/// `Hovers`: the first roll that lunges.
const FIRST_LUNGE: u16 = 4;
/// `Hovers`: the last roll that lunges.
const LAST_LUNGE: u16 = 8;
/// `Hovers`: the first roll that swoops.
const FIRST_SWOOP: u16 = 10;
/// `Hovers`: the last roll that swoops.
const LAST_SWOOP: u16 = 14;

/// `PulledToHisPoint`: the pull's divisor: the pull is the gap to his point over 16.
const PULL_DIVISOR: u16 = 16;
/// `PulledToHisPoint`: the top speed his pull leaves him, 1,280/256 px a run each way.
const TOP_SPEED: Speed = Wrapping(1280);
/// `PulledToHisPoint`: while he is moving off the place he hovers at, he is pulled three times
/// as hard.
const PULLS_MOVING_AWAY: Speed = Wrapping(3);
/// `PulledToHisPoint`: he turns back on this run when two pulls and the odd unit would have
/// turned him; then he gains one more.
const PULLS_TO_TURN: Speed = Wrapping(2);

/// `Flies`: his speed, on either axis, that shakes the room as he meets its left side.
const SHAKING_SPEED: u16 = 640;
/// `Flies`: the shake as he meets the room's left side fast: the back layers, sideways, 3 px for
/// 12 frames.
const WALL_SHAKE: ShakeStart = ShakeStart {
    shake: Shake {
        strength: 3,
        length: 12,
    },
    layers: ShakeLayers::Back,
    motion: ShakeMotion::Sideways,
};

/// A side of the room that stops his move. Listed alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    /// The room's bottom, y 176.
    Bottom,
    /// Its left side, x 40.
    Left,
    /// Its right side, x 224.
    Right,
    /// Its top, y −32.
    Top,
}

/// The room's bounds along one axis: the least and the most whole px he may be at, signed, and
/// the bound he meets at each.
struct Span {
    /// The least.
    least: i16,
    /// The bound he meets short of the least.
    short: Bound,
    /// The most.
    most: i16,
    /// The bound he meets at the most or past it.
    past: Bound,
}

/// `Flies`: the room's bounds: x 40 to 224 and y −32 to 176.
const ROOM: Pair<Span> = Pair {
    x: Span {
        least: 40,
        short: Bound::Left,
        most: 224,
        past: Bound::Right,
    },
    y: Span {
        least: -32,
        short: Bound::Top,
        most: 176,
        past: Bound::Bottom,
    },
};

/// The runs he has hovered without reaching his point, 0 to 123: 0 from his placing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoverRuns(u8);

impl HoverRuns {
    /// No runs: as he is placed, and as he turns to an attack.
    pub const NONE: Self = Self(0);

    /// This many runs, or none from his patience, 124, on.
    #[must_use]
    pub const fn new(runs: u8) -> Option<Self> {
        if runs < PATIENCE {
            Some(Self(runs))
        } else {
            None
        }
    }

    /// How many runs.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// `Hovers`: his hover's runs one more; none once that reaches his patience, when he
    /// attacks.
    const fn one_more(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(runs) => Self::new(runs),
            None => None,
        }
    }
}

/// How the fight stands as he runs: what his hover reads to give up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fight {
    /// Her energy as it stands when he runs, any hit of an earlier frame taken off
    /// (samus-health).
    pub her_energy: u16,
    /// How many of her shots have hit him since he was placed, a shot of the same frame included
    /// (a later part counts them).
    pub hits_taken: u16,
}

impl Fight {
    /// `Hovers`: whether the fight ends his hover: from her 100th hit on him his fake retreat,
    /// whatever her energy; else with her energy under 30 his retreat.
    const fn gives_up(self) -> Option<GiveUp> {
        if self.hits_taken >= HIS_HITS_LIMIT {
            Some(GiveUp::FakeRetreat)
        } else if self.her_energy < HER_ENERGY_LIMIT {
            Some(GiveUp::Retreating)
        } else {
            None
        }
    }
}

/// What his phase's rule leaves for his move: his phase, his speed and his hover's runs.
pub struct Steered {
    /// His phase as the rule leaves it.
    pub phase: RidleyPhase,
    /// His speed, which he then moves by.
    pub speed: Pair<Speed>,
    /// His hover's runs.
    pub hover_runs: HoverRuns,
}

impl Steered {
    /// The rule leaves him in this flight phase, at this speed, his hover's runs as they are.
    const fn flying(phase: FlightPhase, speed: Pair<Speed>, hover_runs: HoverRuns) -> Self {
        Self {
            phase: RidleyPhase::InFlight(phase),
            speed,
            hover_runs,
        }
    }
}

/// What his phase's rule decides on a run of his flight.
pub enum Ruled {
    /// He flies on: he moves by his speed, and his take-off's figure steps on.
    FliesOn(Steered),
    /// He gives up: all he does on this run is a later part's.
    GivesUp(GiveUp),
}

impl FlightPhase {
    /// His phase's rule on a run of his flight, reading his whole px and his speed as the run
    /// starts, his hover's runs, the fight and the random number.
    ///
    /// `TakesOff`: taking off, his speed is 0 before he moves. `Rises`: rising, his y speed
    /// gains 16 upward; once above y 112 he slows, and `SlowsDown` runs on this run too.
    /// `Hovers`: hovering, as `hover` says.
    pub(crate) fn rule(
        self,
        position: Pair<Px>,
        speed: Pair<Speed>,
        hover_runs: HoverRuns,
        fight: Fight,
        random: u16,
    ) -> Ruled {
        match self {
            Self::TakingOff => Ruled::FliesOn(Steered::flying(Self::TakingOff, STILL, hover_runs)),
            Self::Rising => {
                let speed = Pair {
                    y: speed.y - RISE_PULL,
                    ..speed
                };
                Ruled::FliesOn(if short_of(position.y, SLOWS_ABOVE) {
                    slowed(position.y, speed, hover_runs)
                } else {
                    Steered::flying(Self::Rising, speed, hover_runs)
                })
            }
            Self::Slowing => Ruled::FliesOn(slowed(position.y, speed, hover_runs)),
            Self::Hovering => fight.gives_up().map_or_else(
                || Ruled::FliesOn(hover(position, speed, hover_runs, random)),
                Ruled::GivesUp,
            ),
        }
    }
}

/// `SlowsDown`: slowing, his y speed gains 20 downward; once above y 80, his y `y` as the run
/// starts, he hovers from the next run.
fn slowed(y: Px, speed: Pair<Speed>, hover_runs: HoverRuns) -> Steered {
    let speed = Pair {
        y: speed.y + SLOWING,
        ..speed
    };
    let phase = if short_of(y, HOVERS_ABOVE) {
        FlightPhase::Hovering
    } else {
        FlightPhase::Slowing
    };
    Steered::flying(phase, speed, hover_runs)
}

/// `Hovers`: hovering, her energy and his hits not ending it, he is pulled to his point
/// (`PulledToHisPoint`); then near his point, judged where the run started, he turns to an
/// attack, his hover's runs 0; not near, he hovers on a run more, until his hover's runs reach
/// his patience, when he attacks too.
fn hover(position: Pair<Px>, speed: Pair<Speed>, hover_runs: HoverRuns, random: u16) -> Steered {
    let speed = Pair {
        x: pulled(position.x, HIS_POINT.x, speed.x),
        y: pulled(position.y, HIS_POINT.y, speed.y),
    };
    let hovers_on = if near_his_point(position) {
        None
    } else {
        hover_runs.one_more()
    };
    hovers_on.map_or_else(
        || Steered {
            phase: RidleyPhase::Attacking(attack(random)),
            speed,
            hover_runs: HoverRuns::NONE,
        },
        |hover_runs| Steered::flying(FlightPhase::Hovering, speed, hover_runs),
    )
}

/// `Hovers`: whether he is within 16 px of his point on both axes, each difference taken mod
/// 2^16 as a signed number.
fn near_his_point(position: Pair<Px>) -> bool {
    let near = |at: Px, point: Px| (at - point).0.cast_signed().unsigned_abs() <= NEAR_HIS_POINT;
    near(position.x, HIS_POINT.x) && near(position.y, HIS_POINT.y)
}

/// `Hovers`: the attack he turns to, by the random number mod 16: 4 to 8 `Lunge`, 10 to 14
/// `Swoop`, and 0 to 3, 9 and 15 `Fireballs`.
const fn attack(random: u16) -> Attack {
    match random % ATTACK_ROLLS {
        FIRST_LUNGE..=LAST_LUNGE => Attack::Lunge,
        FIRST_SWOOP..=LAST_SWOOP => Attack::Swoop,
        _ => Attack::Fireballs,
    }
}

/// Where his point lies from him on one axis.
#[derive(Clone, Copy)]
enum PointLies {
    /// Right of him, or below.
    Ahead,
    /// Left of him, or above.
    Behind,
}

impl PointLies {
    /// `PulledToHisPoint`: where his point `q` lies from his whole px `p`: with `D` =
    /// `(p − q) mod 2^16`, ahead when `D` is 32,768 or more, behind when 1 to 32,767; none when
    /// he is at it.
    const fn of(p: Px, q: Px) -> Option<Self> {
        match p.0.wrapping_sub(q.0).cast_signed() {
            0 => None,
            gap if gap < 0 => Some(Self::Ahead),
            _ => Some(Self::Behind),
        }
    }

    /// `PulledToHisPoint`: the pull `d`: `max(1, floor(g / 16))`, with `g` the gap to his point
    /// the way it lies, `65,536 − D` ahead and `D` behind.
    fn pull(self, p: Px, q: Px) -> Speed {
        let gap = match self {
            Self::Ahead => q - p,
            Self::Behind => p - q,
        };
        Wrapping((gap.0 / PULL_DIVISOR).max(1).cast_signed())
    }

    /// `PulledToHisPoint`: the odd unit added to each pull: `c` ahead and `1 − c` behind, with
    /// `c` 1 when `p` ≥ `q` as unsigned 16-bit numbers. So it is 0 ahead and 1 behind, except
    /// far apart as unsigned numbers (as when he is above the room's top).
    fn odd_unit(self, p: Px, q: Px) -> Speed {
        let c = p.0 >= q.0;
        let odd = match self {
            Self::Ahead => c,
            Self::Behind => !c,
        };
        Wrapping(i16::from(odd))
    }

    /// Whether a speed heads to his point, or is at the point's side of 0: not negative ahead,
    /// negative behind.
    const fn heads_to(self, speed: Speed) -> bool {
        match self {
            Self::Ahead => speed.0 >= 0,
            Self::Behind => speed.0 < 0,
        }
    }

    /// `speed` with `units` added on the side that closes on the place he hovers at, mod 2^16.
    fn toward(self, speed: Speed, units: Speed) -> Speed {
        match self {
            Self::Ahead => speed + units,
            Self::Behind => speed - units,
        }
    }
}

/// `PulledToHisPoint`: his speed `v` on one axis pulled to his point `q` from his whole px `p`.
///
/// At his point his speed stays. Heading to it, it gains `d` and the odd unit toward it; moving
/// away, three times `d` and the odd unit, and one more if two pulls and the odd unit would
/// have turned him. Each new speed is taken mod 2^16 as a signed number; then a speed past
/// the top speed while closing on it, the difference read signed, is the top speed. So any speed
/// from −1,280 to 1,280 stays within it.
fn pulled(p: Px, q: Px, v: Speed) -> Speed {
    let Some(lies) = PointLies::of(p, q) else {
        return v;
    };
    let d = lies.pull(p, q);
    let odd = lies.odd_unit(p, q);
    let speed = if lies.heads_to(v) {
        lies.toward(v, d + odd)
    } else {
        let turns = lies.heads_to(lies.toward(v, d * PULLS_TO_TURN + odd));
        lies.toward(v, d * PULLS_MOVING_AWAY + odd + Wrapping(i16::from(turns)))
    };
    if lies.heads_to(lies.toward(speed, -TOP_SPEED)) {
        lies.toward(Wrapping(0), TOP_SPEED)
    } else {
        speed
    }
}

/// Whether `at` is short of `edge`: `(at − edge) mod 2^16`, read signed, is negative. Left of
/// it on x, above it on y.
const fn short_of(at: Px, edge: Px) -> bool {
    at.0.wrapping_sub(edge.0).cast_signed() < 0
}

/// One axis of his move: where it leaves him, his speed, and the bound he met on it, if any.
struct AxisMove {
    /// His whole px.
    whole: Px,
    /// His fraction, in 1/256 px.
    fraction: u8,
    /// His speed.
    speed: Speed,
    /// The bound he met.
    bound: Option<Bound>,
}

impl Span {
    /// `Flies`: his move along this axis: his place in 1/256 px (his whole px × 256 + his
    /// fraction) gains his speed, mod 2^24; then short of the least he is stopped at it, and
    /// at the most or past it at the most: his fraction kept, his speed 0, the bound met.
    fn moved(&self, whole: Px, fraction: u8, speed: Speed) -> AxisMove {
        let [high, low] = whole.0.to_be_bytes();
        let place = u32::from_be_bytes([0, high, low, fraction]);
        let [_, high, low, fraction] = place.wrapping_add_signed(i32::from(speed.0)).to_be_bytes();
        let whole = Wrapping(u16::from_be_bytes([high, low]));
        let stopped = |at: i16, bound: Bound| AxisMove {
            whole: Wrapping(at.cast_unsigned()),
            fraction,
            speed: Wrapping(0),
            bound: Some(bound),
        };
        if short_of(whole, Wrapping(self.least.cast_unsigned())) {
            stopped(self.least, self.short)
        } else if !short_of(whole, Wrapping(self.most.cast_unsigned())) {
            stopped(self.most, self.past)
        } else {
            AxisMove {
                whole,
                fraction,
                speed,
                bound: None,
            }
        }
    }
}

/// What his move leaves: where he is, his speed, the bound he met and the shake he starts.
pub struct Moved {
    /// His whole px.
    pub position: Pair<Px>,
    /// His position's fractions, in 1/256 px.
    pub fraction: Pair<u8>,
    /// His speed.
    pub speed: Pair<Speed>,
    /// The bound he met: y's when he met one, else x's.
    pub bound: Option<Bound>,
    /// The shake, as he meets the room's left side fast.
    pub shake: Option<ShakeStart>,
}

/// `Flies`: his move by his speed, after his phase's rule: x, then y, each kept within the
/// room's bounds. Meeting the left side with the larger of his two speeds' sizes, taken before
/// he is stopped, 640 or more, he shakes the room.
pub fn flown(position: Pair<Px>, fraction: Pair<u8>, speed: Pair<Speed>) -> Moved {
    let x = ROOM.x.moved(position.x, fraction.x, speed.x);
    let y = ROOM.y.moved(position.y, fraction.y, speed.y);
    let fast = speed.x.0.unsigned_abs().max(speed.y.0.unsigned_abs()) >= SHAKING_SPEED;
    Moved {
        position: Pair {
            x: x.whole,
            y: y.whole,
        },
        fraction: Pair {
            x: x.fraction,
            y: y.fraction,
        },
        speed: Pair {
            x: x.speed,
            y: y.speed,
        },
        bound: y.bound.or(x.bound),
        shake: (fast && x.bound == Some(Bound::Left)).then_some(WALL_SHAKE),
    }
}
