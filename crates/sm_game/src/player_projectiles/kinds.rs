//! The player projectile kind word: bits 8-11 hold what the shot is, the low
//! bits hold beam flags, and bit 15 keeps the shot from touching Samus.
#![forbid(unsafe_code)]
use crate::constants::projectile_types::*;

/// Bits 8-11; zero for beams.
pub const TYPE_MASK: u16 = 0x0f00;
pub const MISSILE: u16 = 0x0100;
pub const SUPER_MISSILE: u16 = 0x0200;
pub const POWER_BOMB: u16 = 0x0300;
pub const BOMB: u16 = 0x0500;
pub const BEAM_EXPLOSION: u16 = 0x0700;
pub const MISSILE_EXPLOSION: u16 = 0x0800;
/// Set on shots that must not interact with Samus.
pub const NO_SAMUS_CONTACT: u16 = 0x8000;

/// The missile bit of the type field. Missile code tests these single bits to
/// tell missiles (0x100) from super missiles (0x200); power bombs (0x300) set
/// both, so these are not type tests on their own.
pub const MISSILE_BIT: u16 = 0x0100;
pub const SUPER_MISSILE_BIT: u16 = 0x0200;

const _: () = {
    assert!(TYPE_MASK as u32 == kProjectileType_TypeMask);
    assert!(MISSILE as u32 == kProjectileType_Missile);
    assert!(SUPER_MISSILE as u32 == kProjectileType_SuperMissile);
    assert!(POWER_BOMB as u32 == kProjectileType_PowerBomb);
    assert!(BOMB as u32 == kProjectileType_Bomb);
    assert!(BEAM_EXPLOSION as u32 == kProjectileType_BeamExplosion);
    assert!(MISSILE_EXPLOSION as u32 == kProjectileType_MissileExplosion);
    assert!(NO_SAMUS_CONTACT as u32 == kProjectileType_DontInteractWithSamus);
};
