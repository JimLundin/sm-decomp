//! Direct bomb fuse/contact/update binding; HDMA creation is an actual foreign boundary.
use super::{Context, Transaction};
use crate::game::bomb_lifecycle::{self as lifecycle, Field, Fields, Kind};
use crate::ram_map::word_arrays as arrays;
fn offset(field: Field) -> usize {
    use Field::*;
    match field {
        Type => arrays::projectile_type,
        Direction => arrays::projectile_dir,
        Variable => arrays::projectile_variables,
        X => arrays::projectile_x_pos,
        Y => arrays::projectile_y_pos,
        XFraction => arrays::projectile_bomb_x_subpos,
        YFraction => arrays::projectile_bomb_y_subpos,
        XSpeed => arrays::projectile_bomb_x_speed,
        YSpeed => arrays::projectile_bomb_y_speed,
        SpeedFraction => arrays::projectile_timers,
        YRadius => arrays::projectile_y_radius,
        Auxiliary => arrays::projectile_unk_A,
    }
}
impl<'g> Fields<Transaction<'g>> for Context {
    fn selected(&self, g: &Transaction<'g>) -> u16 {
        word!(g.projectile_index)
    }
    fn read(&self, g: &Transaction<'g>, slot: u16, field: Field) -> u16 {
        let at = offset(field) + usize::from(slot & !1);
        g.read_native(at)
    }
    fn write(&mut self, g: &mut Transaction<'g>, slot: u16, field: Field, value: u16) {
        let at = offset(field) + usize::from(slot & !1);
        g.write_slice_native(at, value);
    }
    fn invalid(&mut self, _: Transaction<'g>, index: usize, length: usize) -> ! {
        panic!("index out of bounds: the len is {length} but the index is {index}")
    }
}
impl<'g> lifecycle::Context<Transaction<'g>> for Context {
    fn power_active(&self, g: &Transaction<'g>) -> bool {
        word!(g.power_bomb_flag) != 0
    }
    fn gravity(&self, g: &Transaction<'g>) -> u32 {
        (u32::from(word!(g.samus_y_accel)) << 16) | u32::from(word!(g.samus_y_subaccel))
    }
    fn clear(&mut self, mut g: Transaction<'g>, slot: u16) -> Transaction<'g> {
        super::motion::clear(slot, g)
    }
    fn origin(&mut self, mut g: Transaction<'g>, slot: u16) -> Transaction<'g> {
        word!(g.power_bomb_explosion_x_pos = word!(g.projectile_x_pos[slot >> 1]));
        word!(g.power_bomb_explosion_y_pos = word!(g.projectile_y_pos[slot >> 1]));
        g
    }
    fn enable_hdma(&mut self, mut g: Transaction<'g>) -> Transaction<'g> {
        g.foreign(|g| unsafe { crate::game::sm_88::EnableHdmaObjects(g) })
            .0
    }
    fn spawn_power_bomb(&mut self, mut g: Transaction<'g>) -> Transaction<'g> {
        g.foreign(|g| unsafe { crate::game::explosion_window::spawn_power_bomb(g) })
            .0
    }
    fn sound(&mut self, mut g: Transaction<'g>) -> Transaction<'g> {
        g.foreign(|g| crate::game::sm_80::QueueSfx2_Max6(8, g)).0
    }
    fn flash_animation(&mut self, mut g: Transaction<'g>, slot: u16) -> Transaction<'g> {
        word!(g.projectile_bomb_instruction_ptr[slot >> 1] += 28);
        g
    }
    fn explosion_animation(&mut self, mut g: Transaction<'g>, slot: u16) -> Transaction<'g> {
        {
            let pointer = g
                .rom()
                .table_word(crate::rtl::lorom_offset(0x938683) as usize, 0);
            word!(g.projectile_bomb_instruction_ptr[slot >> 1] = pointer);
            word!(g.projectile_bomb_instruction_timers[slot >> 1] = 1);
            g
        }
    }
    fn contact(
        &mut self,
        mut g: Transaction<'g>,
        slot: u16,
        spread: bool,
    ) -> (Transaction<'g>, u8) {
        if spread {
            crate::game::bomb_terrain::spread(g, slot, self)
        } else {
            (crate::game::bomb_terrain::explode(g, slot, self), 0)
        }
    }
}
pub(super) fn advance_fuse(power: bool, game: Transaction<'_>) -> Transaction<'_> {
    lifecycle::advance_fuse(game, power, &mut Context::default())
}
pub(super) fn bomb_update(selected: u16, kind: Kind, game: Transaction<'_>) -> Transaction<'_> {
    lifecycle::update(game, selected, kind, &mut Context::default())
}
