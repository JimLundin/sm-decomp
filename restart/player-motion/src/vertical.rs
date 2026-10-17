use crate::{Terrain, feedback, memory::*, solid_enemy::Contact};

pub(crate) fn no_collision(ram: &mut [u8], amount: i32, upward: bool) {
    // Both upward entry points exclude i32::MIN before any mutation.
    let applied = if upward { -amount } else { amount };
    add_y(ram, applied);
    set_hilo(ram, if upward { UP } else { DOWN }, applied);
}

pub(crate) fn move_root(
    ram: &mut [u8],
    contact: Contact,
    upward: bool,
    terrain: &mut impl Terrain,
) -> Result<(), String> {
    put(ram, DIRECTION, if upward { 2 } else { 3 });
    contact.apply(ram);
    put(ram, COLLISION, contact.collision);
    if contact.collision != 0 {
        if upward {
            feedback::up(ram);
        } else {
            feedback::down(ram)?;
        }
        no_collision(ram, contact.amount, upward);
    } else {
        let amount = if upward { -contact.amount } else { contact.amount };
        let applied = terrain.move_vertical(ram, amount)?;
        // The synchronous service already changed Y. Publish its returned
        // amount before feedback, using its full current copyback state.
        set_hilo(ram, if upward { UP } else { DOWN }, applied);
        if upward {
            feedback::up(ram);
        } else {
            feedback::down(ram)?;
        }
    }
    Ok(())
}
