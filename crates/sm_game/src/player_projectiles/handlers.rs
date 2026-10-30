//! What a player projectile runs each frame: its bank $90 handler address,
//! stored in the shot's `motion` word and dispatched by `motion::run`.

/// Does nothing; an empty slot or a shot waiting to be retired.
pub const IDLE: u16 = 0xb169;

/// Uncharged and charged beams without wave, updated every 4 frames.
pub const BEAM: u16 = 0xaef3;
/// Wave and plasma-wave beams, updated every 4 frames.
pub const WAVE_BEAM: u16 = 0xb0c3;
/// Ice-wave beams, updated every 3 frames.
pub const ICE_WAVE_BEAM: u16 = 0xb0e4;
/// The hyper beam, updated every frame.
pub const HYPER_BEAM: u16 = 0xb159;
/// A wave beam's oscillation step.
pub const WAVE_ADVANCE: u16 = 0xb103;

pub const MISSILE: u16 = 0xaf68;
pub const SUPER_MISSILE: u16 = 0xafe5;
/// The trailing half of a super missile.
pub const SUPER_MISSILE_EXTENSION: u16 = 0xb075;

pub const BOMB: u16 = 0xb099;
pub const POWER_BOMB: u16 = 0xb0ae;
/// One of the spread bombs from a charged bomb release.
pub const SPREAD_BOMB: u16 = 0xd8f7;

/// Charged special beams orbiting or released (`specials::Kind`).
pub const WAVE_SPECIAL: u16 = 0xda08;
pub const ICE_SPECIAL: u16 = 0xcf09;
pub const ICE_SPECIAL_RELEASED: u16 = 0xcf7a;
pub const PLASMA_SPECIAL: u16 = 0xd793;
pub const SPAZER_SPECIAL: u16 = 0xdb06;
pub const SPAZER_SPECIAL_RELEASED: u16 = 0xdc9c;
/// The plasma special's radius growing, then contracting, and its departure.
pub const PLASMA_RADIUS_GROWING: u16 = 0xd7e1;
pub const PLASMA_RADIUS_CONTRACTING: u16 = 0xd7fa;
pub const PLASMA_DEPARTING: u16 = 0xd813;

/// A speed-booster echo following Samus.
pub const SPEED_ECHO: u16 = 0xd4d2;
/// The shot fired while the suit pickup sequence runs.
pub const SUIT_PICKUP_SHOT: u16 = 0xefd3;

/// Collision-only handlers for each flight kind and the axes they test.
pub mod collide {
    pub const BEAM_X: u16 = 0xaf60;
    pub const BEAM_Y: u16 = 0xaf4a;
    pub const BEAM_XY: u16 = 0xaf52;
    pub const MISSILE_X: u16 = 0xafdd;
    pub const MISSILE_Y: u16 = 0xafc7;
    pub const MISSILE_XY: u16 = 0xafcf;
    pub const SUPER_MISSILE_X: u16 = 0xb06a;
    pub const SUPER_MISSILE_Y: u16 = 0xb047;
    pub const SUPER_MISSILE_XY: u16 = 0xb052;
    pub const WAVE_BEAM_X: u16 = 0xb151;
    pub const WAVE_BEAM_Y: u16 = 0xb13b;
    pub const WAVE_BEAM_XY: u16 = 0xb143;
}

/// The `axes` bits `motion::collide` tests.
pub mod axes {
    pub const X: u8 = 1;
    pub const Y: u8 = 2;
    pub const BOTH: u8 = X | Y;
}
