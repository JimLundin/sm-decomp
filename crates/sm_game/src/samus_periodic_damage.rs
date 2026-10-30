//! Direct and periodic health loss settle into the sole inventory owner.
#![forbid(unsafe_code)]
use crate::{combat_rules::contact_health, session::Session};

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidDamage;

/// Immediate hits preserve the sentinel amount and signed-health wraparound.
pub fn direct(session: &mut Session, frozen: u16, amount: u16) -> Result<(), InvalidDamage> {
    if amount & 0x8000 != 0 {
        return Err(InvalidDamage);
    }
    if amount != 300 && frozen == 0 {
        let health = &mut session.inventory.health.current;
        *health = contact_health(*health, amount);
    }
    Ok(())
}

pub fn apply(session: &mut Session, frozen: u16) -> Result<(), InvalidDamage> {
    if frozen != 0 {
        session.player.periodic_damage_fraction = 0;
        session.player.periodic_damage = 0;
        return Ok(());
    }
    let mut damage = (u32::from(session.player.periodic_damage) << 16)
        | u32::from(session.player.periodic_damage_fraction);
    if damage & 0x8000_0000 != 0 {
        return Err(InvalidDamage);
    }
    if session.inventory.equipment.active & 0x20 != 0 {
        damage = (damage >> 2) & 0x00ff_ff00;
    } else if session.inventory.equipment.active & 1 != 0 {
        damage = (damage >> 1) & 0x00ff_ff00;
    }
    // Pending damage uses the LE field view. The original health arithmetic
    // instead reads/writes native words, then tests health through the LE view.
    let low = u16::from_ne_bytes(session.inventory.health_fraction.to_le_bytes());
    let high = u16::from_ne_bytes(session.inventory.health.current.to_le_bytes());
    let health = (u32::from(low) | (u32::from(high) << 16)).wrapping_sub(damage);
    session.inventory.health_fraction = u16::from_le_bytes((health as u16).to_ne_bytes());
    session.inventory.health.current = u16::from_le_bytes(((health >> 16) as u16).to_ne_bytes());
    if session.inventory.health.current & 0x8000 != 0 {
        session.inventory.health_fraction = 0;
        session.inventory.health.current = 0;
    }
    session.player.periodic_damage_fraction = 0;
    session.player.periodic_damage = 0;
    Ok(())
}
