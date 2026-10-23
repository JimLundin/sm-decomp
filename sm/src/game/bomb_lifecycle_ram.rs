//! Checked physical projectile fields and explicit native bomb effects.
use crate::game::bomb_lifecycle::{self as lifecycle, Context, Field, Fields, Kind};
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, word_arrays as arrays};
pub struct Runtime;
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
impl<'g> Fields<&'g mut Game> for Runtime {
    fn selected(&self, g: &&'g mut Game) -> u16 {
        word!(g.projectile_index)
    }
    fn read(&self, g: &&'g mut Game, slot: u16, field: Field) -> u16 {
        let at = offset(field) + usize::from(slot & !1);
        u16::from_ne_bytes([g.g_ram[at], g.g_ram[at + 1]])
    }
    fn write(&mut self, g: &mut &'g mut Game, slot: u16, field: Field, value: u16) {
        let at = offset(field) + usize::from(slot & !1);
        g.g_ram[at..at + 2].copy_from_slice(&value.to_ne_bytes());
    }
    fn invalid(&mut self, _: &'g mut Game, index: usize, length: usize) -> ! {
        panic!("index out of bounds: the len is {length} but the index is {index}")
    }
}
impl<'g> Context<&'g mut Game> for Runtime {
    fn power_active(&self, g: &&'g mut Game) -> bool {
        word!(g.power_bomb_flag) != 0
    }
    fn gravity(&self, g: &&'g mut Game) -> u32 {
        (u32::from(word!(g.samus_y_accel)) << 16) | u32::from(word!(g.samus_y_subaccel))
    }
    fn clear(&mut self, g: &'g mut Game, slot: u16) -> &'g mut Game {
        unsafe { crate::game::sm_90::ClearProjectile(slot, g) };
        g
    }
    fn origin(&mut self, g: &'g mut Game, slot: u16) -> &'g mut Game {
        word!(g.power_bomb_explosion_x_pos = word!(g.projectile_x_pos[slot >> 1]));
        word!(g.power_bomb_explosion_y_pos = word!(g.projectile_y_pos[slot >> 1]));
        g
    }
    fn enable_hdma(&mut self, g: &'g mut Game) -> &'g mut Game {
        unsafe { crate::game::sm_88::EnableHdmaObjects(g) };
        g
    }
    fn spawn_power_bomb(&mut self, g: &'g mut Game) -> &'g mut Game {
        unsafe { crate::game::explosion_window::spawn_power_bomb(g) };
        g
    }
    fn sound(&mut self, g: &'g mut Game) -> &'g mut Game {
        crate::game::sm_80::QueueSfx2_Max6(8, g);
        g
    }
    fn flash_animation(&mut self, g: &'g mut Game, slot: u16) -> &'g mut Game {
        word!(g.projectile_bomb_instruction_ptr[slot >> 1] += 28);
        g
    }
    fn explosion_animation(&mut self, g: &'g mut Game, slot: u16) -> &'g mut Game {
        crate::game::projectile_animation::start_bomb_explosion(slot, g);
        g
    }
    fn contact(&mut self, g: &'g mut Game, slot: u16, spread: bool) -> (&'g mut Game, u8) {
        if spread {
            crate::game::bomb_terrain::spread(g, slot, self)
        } else {
            (crate::game::bomb_terrain::explode(g, slot, self), 0)
        }
    }
}
/// Native effects require initialized runtime state.
pub unsafe fn advance_fuse(power: bool, game: &mut Game) {
    lifecycle::advance_fuse(game, power, &mut Runtime);
}
/// Native effects require initialized runtime state.
pub unsafe fn update(selected: u16, kind: Kind, game: &mut Game) {
    lifecycle::update(game, selected, kind, &mut Runtime);
}
