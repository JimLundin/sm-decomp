//! Botwoon callback identities and retained death protocol, without execution dependencies.
#![forbid(unsafe_code)]

/// Bank $B3 Botwoon functions, stored in `function`.
pub mod function {
    pub const INITIAL: u16 = 0x9878;
    pub const GO_THROUGH_HOLE: u16 = 0x989d;
    pub const MOVING_AROUND: u16 = 0x99a4;
    pub const SPITTING: u16 = 0x99e4;
    pub const DEATH_PRE_DEATH_DELAY: u16 = 0x9a46;
    pub const DEATH_FALLING_TO_GROUND: u16 = 0x9a5e;
    pub const DEATH_WAIT_FOR_BODY_TO_FALL: u16 = 0x9aca;
    pub const DEATH_CRUMBLING_WALL: u16 = 0x9af9;
}

/// Bank $B3 Botwoon movement functions, stored in `movement_function`.
pub mod movement_function {
    pub const MOVE_TOWARD_TARGET_HOLE: u16 = 0x9bb7;
    pub const START_MOVING_BY_MOVEMENT_DATA: u16 = 0xe250;
    pub const MOVE_BY_MOVEMENT_DATA: u16 = 0xe28c;
}

/// Bank $B3 Botwoon head functions, stored in `head_function`.
pub mod head_function {
    pub const MOVING_AROUND: u16 = 0x9dc0;
    pub const SPITTING_SET_ANGLE_AND_SHOW: u16 = 0x9e7d;
    pub const SPITTING_SPAWN_5: u16 = 0x9ee0;
    pub const SPITTING_SPAWN_3: u16 = 0x9f34;
    pub const SPITTING_COOLDOWN: u16 = 0x9f7a;
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct DeathProtocol {
    pub pending_hit: u16,
    pub active: u16,
    pub tail_ready: u16,
    pub previous_health: u16,
    pub body_landed: u16,
}

#[derive(Clone, Copy)]
pub enum Signal {
    PendingHit,
    Active,
    TailReady,
    PreviousHealth,
    BodyLanded,
}

impl Signal {
    /// Work-RAM address of this signal word for the actor at slot offset `enemy`.
    pub fn address(self, enemy: u16) -> usize {
        let base = match self {
            Signal::PendingHit => 0x801e,
            Signal::Active => 0x8020,
            Signal::TailReady => 0x803c,
            Signal::PreviousHealth => 0x8818,
            Signal::BodyLanded => 0x883e,
        };
        base + usize::from(enemy)
    }
}
