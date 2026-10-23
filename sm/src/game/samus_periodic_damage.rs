//! Periodic health loss, including the broken-turret grapple contact.
#![forbid(unsafe_code)]

pub trait Context {
    fn frozen(&self) -> bool;
    fn pending_damage(&self) -> u32;
    fn pending_whole(&self) -> u16;
    fn set_pending_whole(&mut self, value: u16);
    fn equipment(&self) -> u16;
    fn health(&self) -> u32;
    fn set_health(&mut self, value: u32);
    fn health_is_negative(&self) -> bool;
    fn clear_health(&mut self);
    fn clear_damage(&mut self);
}

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidDamage;

pub fn grapple_contact(c: &mut impl Context) -> u8 {
    c.set_pending_whole(c.pending_whole().wrapping_add(1));
    0x41 // Collision and grapple connection.
}

pub fn apply(c: &mut impl Context) -> Result<(), InvalidDamage> {
    if c.frozen() {
        c.clear_damage();
        return Ok(());
    }
    let mut damage = c.pending_damage();
    if damage & 0x8000_0000 != 0 {
        return Err(InvalidDamage);
    }
    if c.equipment() & 0x20 != 0 {
        damage = (damage >> 2) & 0x00ff_ff00;
    } else if c.equipment() & 1 != 0 {
        damage = (damage >> 1) & 0x00ff_ff00;
    }
    c.set_health(c.health().wrapping_sub(damage));
    if c.health_is_negative() {
        c.clear_health();
    }
    c.clear_damage();
    Ok(())
}
