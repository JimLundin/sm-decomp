//! Decisions between combat observation points; no RAM or runtime dependencies.

pub enum ShotResponse {
    Dud,
    Freeze,
    Damage(u16),
}

pub struct Shot {
    pub kind: u16,
    pub damage: u16,
}

impl Shot {
    pub fn beam(&self) -> bool {
        self.kind & 0xf00 == 0
    }
    pub fn vulnerability_offset(&self) -> Option<(u16, usize)> {
        match self.kind & 0xf00 {
            0 => Some((self.kind & 15, 0)),
            0x100 => Some((1, 11)),
            0x200 => Some((2, 11)),
            0x300 => Some((0, 15)),
            0x500 => Some((0, 14)),
            _ => None,
        }
    }
    pub fn charged(&self, primary: u8) -> bool {
        self.beam() && primary != 255 && self.kind & 0x10 != 0
    }
    pub fn response(&self, primary: u8, charged: u8) -> ShotResponse {
        if self.beam() && primary == 255 {
            return ShotResponse::Freeze;
        }
        let multiplier = if self.charged(primary) {
            if charged == 255 { 0 } else { charged & 15 }
        } else {
            primary & 0x7f
        };
        let damage = (self.damage >> 1).wrapping_mul(u16::from(multiplier));
        if self.vulnerability_offset().is_none() || damage == 0 {
            ShotResponse::Dud
        } else {
            ShotResponse::Damage(damage)
        }
    }
}

pub fn contact_attack(mode: u16) -> (u16, u16) {
    let offset = mode.wrapping_add(15);
    match mode {
        1 => (offset, 500),
        2 => (offset, 300),
        3 => (offset, 2000),
        _ => (offset.wrapping_add(1), 200),
    }
}

pub fn contact_health(health: u16, damage: u16) -> u16 {
    let remaining = health.wrapping_sub(damage);
    if remaining & 0x8000 != 0 {
        0
    } else {
        remaining
    }
}

pub fn power_bomb_damage(vulnerability: u8) -> u16 {
    if vulnerability == 255 {
        0
    } else {
        100 * u16::from(vulnerability & 0x7f)
    }
}

pub fn hurt_duration(configured: u8, extra: u16) -> u16 {
    u16::from(if configured == 0 { 4 } else { configured }) + extra
}

pub fn freeze_duration(area: u16) -> u16 {
    if area == 2 { 300 } else { 400 }
}

pub enum HealthResponse {
    Freeze,
    Health(u16),
}

pub fn shot_health(
    health: u16,
    damage: u16,
    projectile: u16,
    last_power: u16,
    frozen: u16,
) -> HealthResponse {
    let remaining = health.saturating_sub(damage);
    if remaining == 0 && projectile & 2 != 0 && last_power & 0xf0 != 0x80 && frozen == 0 {
        HealthResponse::Freeze
    } else {
        HealthResponse::Health(remaining)
    }
}

pub fn suit_damage(damage: u16, equipment: u16) -> u16 {
    if equipment & 0x20 != 0 {
        damage >> 2
    } else if equipment & 1 != 0 {
        damage >> 1
    } else {
        damage
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum LatchMode {
    Passive,
    Active,
}

pub enum GrappleOutcome {
    Miss,
    Attach,
    Cancel,
}
