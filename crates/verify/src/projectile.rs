//! Samus's projectile slots, decoded from the original's projectile tables (bank $90 log):
//! one array of ten words per field from `$0B64` on, then the cooldown and the shot count.

use game::state::projectile::{
    Beam, BeamShot, Beams, Damage, Direction, ExplosionStep, Heading, PROJECTILE_SLOTS,
    ProjectileBody, ProjectileCommon, ProjectileType, Projectiles, Radius, Shot, ShotCount,
    SubpixelSpeed, Velocity,
};
use game::state::samus::{Fixed, Point};
use game::state::slots::{Frames, Generation, Occupancy, Slot, SlotStore};

use crate::decode::DecodeError;
use crate::source::{self, WramArray, WramWord};

/// The type word's kind, bits 8 to 11.
const KIND_MASK: u16 = 0x0f00;
/// The type word's bit 15: the projectile can't touch Samus.
const NO_SAMUS_CONTACT: u16 = 0x8000;
/// The type word's beam bits: wave, ice, spazer, plasma.
const BEAM_MASK: u16 = 0x000f;
/// The type word's charged bit.
const CHARGED: u16 = 0x0010;
/// The direction word's direction, 0 to 9.
const DIRECTION_MASK: u16 = 0x000f;
/// The direction word's deleted bit, which an enemy's contact sets.
const DELETED: u16 = 0x0010;
/// The beam explosion's instruction list (`$93:A007`).
const BEAM_EXPLOSION_LIST: u16 = 0xa007;
/// The bytes of one frame of a projectile instruction list.
const LIST_ENTRY: u16 = 8;
/// The frames the beam explosion's list shows before its delete.
const BEAM_EXPLOSION_FRAMES: u8 = 6;
/// The beams in the type word's bit order.
const BEAMS: [Beam; 4] = [Beam::Wave, Beam::Ice, Beam::Spazer, Beam::Plasma];
/// The directions in the original's order.
const DIRECTIONS: [Direction; 10] = [
    Direction::UpFacingRight,
    Direction::UpRight,
    Direction::Right,
    Direction::DownRight,
    Direction::DownFacingRight,
    Direction::DownFacingLeft,
    Direction::DownLeft,
    Direction::Left,
    Direction::UpLeft,
    Direction::UpFacingLeft,
];

/// Where the original keeps each projectile slot's fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectileSources {
    /// `.position.x.pixels`.
    pub x: WramArray,
    /// `.position.y.pixels`.
    pub y: WramArray,
    /// `.position.x.subpixels`.
    pub x_subpixels: WramArray,
    /// `.position.y.subpixels`.
    pub y_subpixels: WramArray,
    /// `.radius.x`.
    pub radius_x: WramArray,
    /// `.radius.y`.
    pub radius_y: WramArray,
    /// `.velocity.x`.
    pub speed_x: WramArray,
    /// `.velocity.y`.
    pub speed_y: WramArray,
    /// `.heading`.
    pub direction: WramArray,
    /// `.kind`.
    pub kind: WramArray,
    /// `.damage`.
    pub damage: WramArray,
    /// The occupancy, and a beam explosion's step: the instruction pointer, zero in a free
    /// slot.
    pub instruction: WramArray,
    /// `.frame_timer`.
    pub frame_timer: WramArray,
    /// `.common.trail_timer`.
    pub trail: WramArray,
    /// `projectiles.count`.
    pub count: WramWord,
    /// `projectiles.cooldown`.
    pub cooldown: WramWord,
}

impl ProjectileSources {
    /// The original's layout.
    pub const ORIGINAL: Self = Self {
        x: source::PROJECTILE_X_POS,
        y: source::PROJECTILE_Y_POS,
        x_subpixels: source::PROJECTILE_X_SUBPOS,
        y_subpixels: source::PROJECTILE_Y_SUBPOS,
        radius_x: source::PROJECTILE_X_RADIUS,
        radius_y: source::PROJECTILE_Y_RADIUS,
        speed_x: source::PROJECTILE_X_SPEED,
        speed_y: source::PROJECTILE_Y_SPEED,
        direction: source::PROJECTILE_DIR,
        kind: source::PROJECTILE_TYPE,
        damage: source::PROJECTILE_DAMAGE,
        instruction: source::PROJECTILE_INSTRUCTION_PTR,
        frame_timer: source::PROJECTILE_INSTRUCTION_TIMERS,
        trail: source::PROJECTILE_TIMERS,
        count: source::PROJECTILE_COUNTER,
        cooldown: source::COOLDOWN_TIMER,
    };

    /// Every slot in `wram`, with the cooldown and the shot count.
    ///
    /// The generation stays at its default: the original keeps no fill count. A free slot's
    /// body fields are the clear's zeroes, so only its trail timer is kept.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Unrepresentable`] for a direction past 9 or with bits besides the
    /// deleted bit, or a type word whose kind no routine writes or with stray bits;
    /// [`DecodeError::WramTooShort`] when a table lies past the record's work RAM.
    pub fn read(&self, wram: &[u8]) -> Result<Projectiles, DecodeError> {
        let mut slots = SlotStore::<ProjectileCommon, ProjectileBody, PROJECTILE_SLOTS>::default();
        for (slot, index) in slots.slots.iter_mut().zip(0_u16..) {
            *slot = self.slot(wram, index)?;
        }
        Ok(Projectiles {
            slots,
            count: ShotCount(self.count.read(wram)?),
            cooldown: Frames(self.cooldown.read(wram)?),
        })
    }

    /// Slot `index`.
    fn slot(
        &self,
        wram: &[u8],
        index: u16,
    ) -> Result<Slot<ProjectileCommon, ProjectileBody>, DecodeError> {
        let word = |array: WramArray| array.word(index)?.read(wram);
        let common = ProjectileCommon {
            trail_timer: Frames(word(self.trail)?),
        };
        let occupancy = if word(self.instruction)? == 0 {
            Occupancy::Free
        } else {
            Occupancy::Live(ProjectileBody {
                position: Point {
                    x: Fixed {
                        pixels: word(self.x)?,
                        subpixels: word(self.x_subpixels)?,
                    },
                    y: Fixed {
                        pixels: word(self.y)?,
                        subpixels: word(self.y_subpixels)?,
                    },
                },
                radius: Radius {
                    x: word(self.radius_x)?,
                    y: word(self.radius_y)?,
                },
                velocity: Velocity {
                    x: speed(word(self.speed_x)?),
                    y: speed(word(self.speed_y)?),
                },
                heading: heading(self.direction.name, word(self.direction)?)?,
                kind: self.kind(wram, index)?,
                damage: Damage(word(self.damage)?),
                frame_timer: Frames(word(self.frame_timer)?),
            })
        };
        Ok(Slot {
            generation: Generation::default(),
            common,
            occupancy,
        })
    }
}

impl ProjectileSources {
    /// The type of the projectile in slot `index`, with a beam explosion's step from its
    /// instruction pointer.
    fn kind(&self, wram: &[u8], index: u16) -> Result<ProjectileType, DecodeError> {
        let mut kind = projectile_type(self.kind.name, self.kind.word(index)?.read(wram)?)?;
        if let Shot::BeamExplosion { step, .. } = &mut kind.shot {
            let at = self.instruction.word(index)?.read(wram)?;
            *step = at
                .checked_sub(BEAM_EXPLOSION_LIST)
                .filter(|offset| offset % LIST_ENTRY == 0)
                .map(|offset| offset / LIST_ENTRY)
                .and_then(|read| u8::try_from(read).ok())
                .filter(|&read| read <= BEAM_EXPLOSION_FRAMES)
                .map(ExplosionStep)
                .ok_or(DecodeError::Unrepresentable {
                    source: self.instruction.name,
                    value: at,
                })?;
        }
        Ok(kind)
    }
}

/// A speed word, which the original reads as signed.
const fn speed(value: u16) -> SubpixelSpeed {
    SubpixelSpeed(i16::from_le_bytes(value.to_le_bytes()))
}

/// A direction word.
fn heading(source: &'static str, value: u16) -> Result<Heading, DecodeError> {
    let unrepresentable = DecodeError::Unrepresentable { source, value };
    if value & !(DIRECTION_MASK | DELETED) != 0 {
        return Err(unrepresentable);
    }
    let direction = DIRECTIONS
        .get(usize::from(value & DIRECTION_MASK))
        .copied()
        .ok_or(unrepresentable)?;
    Ok(Heading {
        direction,
        deleted: value & DELETED != 0,
    })
}

/// A type word.
fn projectile_type(source: &'static str, value: u16) -> Result<ProjectileType, DecodeError> {
    let unrepresentable = DecodeError::Unrepresentable { source, value };
    let low = value & 0x00ff;
    let beam = || {
        if low & !(BEAM_MASK | CHARGED) != 0 {
            return Err(unrepresentable);
        }
        let mut beams = Beams::default();
        for (place, beam) in BEAMS.into_iter().enumerate() {
            if low & (1 << place) != 0 {
                beams.insert(beam);
            }
        }
        Ok(BeamShot {
            beams,
            charged: low & CHARGED != 0,
        })
    };
    let plain = |shot: Shot| {
        if low == 0 {
            Ok(shot)
        } else {
            Err(unrepresentable)
        }
    };
    if value & !(NO_SAMUS_CONTACT | KIND_MASK | 0x00ff) != 0 {
        return Err(unrepresentable);
    }
    let shot = match (value & KIND_MASK) >> 8 {
        0 => Shot::Beam(beam()?),
        1 => plain(Shot::Missile)?,
        2 => plain(Shot::SuperMissile)?,
        3 => plain(Shot::PowerBomb)?,
        5 => plain(Shot::Bomb)?,
        7 => Shot::BeamExplosion {
            beam: beam()?,
            step: ExplosionStep::default(),
        },
        8 => plain(Shot::MissileExplosion)?,
        _ => return Err(unrepresentable),
    };
    Ok(ProjectileType {
        shot,
        touches_samus: value & NO_SAMUS_CONTACT == 0,
    })
}

#[cfg(test)]
mod tests {
    use super::{heading, projectile_type, speed};
    use game::state::projectile::{Beam, Direction, Shot};

    #[test]
    fn the_power_beam_and_its_explosion_decode() {
        let beam = projectile_type("t", 0x8000);
        assert!(matches!(beam, Ok(kind) if !kind.touches_samus
            && matches!(kind.shot, Shot::Beam(shot) if shot.beams.is_empty() && !shot.charged)));
        let explosion = projectile_type("t", 0x8700);
        assert!(matches!(explosion, Ok(kind) if matches!(kind.shot, Shot::BeamExplosion { .. })));
        let wave = projectile_type("t", 0x0011);
        assert!(matches!(wave, Ok(kind) if kind.touches_samus
            && matches!(kind.shot, Shot::Beam(shot) if shot.beams.contains(Beam::Wave) && shot.charged)));
        assert!(projectile_type("t", 0x0400).is_err());
        assert!(projectile_type("t", 0x0120).is_err());
    }

    #[test]
    fn a_direction_keeps_its_deleted_bit() {
        assert!(
            matches!(heading("d", 3), Ok(h) if h.direction == Direction::DownRight && !h.deleted)
        );
        assert!(matches!(heading("d", 0x17), Ok(h) if h.direction == Direction::Left && h.deleted));
        assert!(heading("d", 10).is_err());
        assert!(heading("d", 0x23).is_err());
    }

    #[test]
    fn speeds_are_signed() {
        assert_eq!(speed(0xfff0).0, -16);
        assert_eq!(speed(0x034b).0, 0x034b);
    }
}
