//! Public initialization identities used by actual projectile callers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Initialization {
    Projectile,
    SuperMissile,
    Bomb,
    SpecialAttack,
    Echo,
}
