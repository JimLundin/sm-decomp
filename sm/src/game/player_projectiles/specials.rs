//! Direct special-attack creation/orbit/departure binding for all selectors.
use super::{Context, Transaction};
use crate::game::projectile_initialization::Initialization;
use crate::game::special_attacks::{self as shared, Attack, Control, Field, Input, Kind};
use crate::ram_map::{word_arrays as arrays, words};
fn offset(field: Field) -> usize {
    use Field::*;
    match field {
        Type => arrays::projectile_type,
        Direction => arrays::projectile_dir,
        X => arrays::projectile_x_pos,
        Y => arrays::projectile_y_pos,
        XFraction => arrays::projectile_bomb_x_subpos,
        YFraction => arrays::projectile_bomb_y_subpos,
        XSpeed => arrays::projectile_bomb_x_speed,
        YSpeed => arrays::projectile_bomb_y_speed,
        Variable => arrays::projectile_variables,
        Timer => arrays::projectile_timers,
        Auxiliary => arrays::projectile_unk_A,
    }
}
fn control_offset(field: Control) -> usize {
    use Control::*;
    match field {
        Selected => words::projectile_index,
        Hud => words::hud_item_index,
        AutoCancel => words::samus_auto_cancel_hud_item_index,
        Count => words::projectile_counter,
        Cooldown => words::cooldown_timer,
        Charge => words::flare_counter,
        OrbitStep => words::used_for_sba_attacksB60,
    }
}
fn instruction(kind: Kind) -> u16 {
    match kind {
        Kind::Wave => 0xda08,
        Kind::Ice => 0xcf09,
        Kind::IceReleased => 0xcf7a,
        Kind::Plasma => 0xd793,
        Kind::Spazer => 0xdb06,
        Kind::SpazerReleased => 0xdc9c,
    }
}
impl<'g> shared::Context<Transaction<'g>> for Context {
    fn native_word(&self, g: &Transaction<'g>, slot: u16, field: Field) -> u16 {
        let at = offset(field) + usize::from(slot & !1);
        g.read_native(at)
    }
    fn write_native(&mut self, g: &mut Transaction<'g>, slot: u16, field: Field, value: u16) {
        let at = offset(field) + usize::from(slot & !1);
        g.write_slice_native(at, value);
    }
    fn read(&self, g: &Transaction<'g>, slot: u16, field: Field) -> u16 {
        let at = offset(field) + usize::from(slot & !1);
        g.read_le(at)
    }
    fn low_byte(&self, g: &Transaction<'g>, slot: u16, field: Field) -> u8 {
        g.byte(offset(field) + usize::from(slot & !1))
    }
    fn write(&mut self, g: &mut Transaction<'g>, slot: u16, field: Field, value: u16) {
        let at = offset(field) + usize::from(slot & !1);
        g.write_slice_le(at, value);
    }
    fn motion_is(&self, g: &Transaction<'g>, slot: u16, kind: Kind) -> bool {
        word!(g.projectile_bomb_pre_instructions[slot >> 1]) == instruction(kind)
    }
    fn motion(&mut self, g: &mut Transaction<'g>, slot: u16, kind: Kind) {
        word!(g.projectile_bomb_pre_instructions[slot >> 1] = instruction(kind));
    }
    fn control(&self, g: &Transaction<'g>, field: Control) -> u16 {
        word!(g,control_offset(field);)
    }
    fn set_control(&mut self, g: &mut Transaction<'g>, field: Control, value: u16) {
        word!(g,control_offset(field);=value);
    }
    fn input(&self, g: &Transaction<'g>, field: Input) -> u16 {
        use Input::*;
        match field {
            SamusX => word!(g.samus_x_pos),
            SamusY => word!(g.samus_y_pos),
            CameraX => word!(g.layer1_x_pos),
            CameraY => word!(g.layer1_y_pos),
            Beams => word!(g.equipped_beams),
            Facing => u16::from(g.byte(0xa1e)),
            PowerBombs => word!(g.samus_power_bombs),
        }
    }
    fn initialize(
        &mut self,
        mut g: Transaction<'g>,
        slot: u16,
        kind: Initialization,
    ) -> Transaction<'g> {
        super::animation::initialize(slot, kind, g)
    }
    fn trail(&mut self, mut g: Transaction<'g>, slot: u16) -> Transaction<'g> {
        super::presentation::emit(slot, g)
    }
    fn clear(&mut self, mut g: Transaction<'g>, slot: u16) -> Transaction<'g> {
        super::motion::clear(slot, g)
    }
    fn sound(&mut self, mut g: Transaction<'g>, sound: u16) -> Transaction<'g> {
        g.foreign(|g| crate::game::sm_80::QueueSfx1_Max6(sound, g))
            .0
    }
    fn power_bombs(&mut self, mut g: Transaction<'g>, value: u16) -> Transaction<'g> {
        word!(g.samus_power_bombs = value);
        g
    }
    fn cooldown(&mut self, mut g: Transaction<'g>, index: usize) -> (Transaction<'g>, u16) {
        (
            g,
            u16::from(crate::game::beam_definitions::MANUAL_COOLDOWN[index]),
        )
    }
    fn point(
        &mut self,
        mut g: Transaction<'g>,
        angle: u16,
        radius: u16,
    ) -> (Transaction<'g>, (u16, u16)) {
        let point = super::motion::point(angle, radius);
        (g, point)
    }
    fn invalid(&mut self, _: Transaction<'g>, index: usize, length: usize) -> ! {
        panic!("index out of bounds: the len is {length} but the index is {index}")
    }
    fn invalid_plasma_phase(&mut self, mut g: Transaction<'g>) -> ! {
        let g = g.release();
        unsafe { crate::rtl::Unreachable(g) };
        loop {
            core::hint::spin_loop();
        }
    }
}
pub(super) fn fire(game: Transaction<'_>) -> (Transaction<'_>, u8) {
    shared::fire(game, &mut Context::default())
}
pub(super) fn create(attack: Attack, game: Transaction<'_>) -> (Transaction<'_>, u8) {
    shared::create(game, attack, &mut Context::default())
}
pub(super) fn update(selected: u16, kind: Kind, game: Transaction<'_>) -> Transaction<'_> {
    shared::update(game, selected, kind, &mut Context::default())
}
pub(super) fn plasma_radius(
    selected: u16,
    contracting: bool,
    game: Transaction<'_>,
) -> Transaction<'_> {
    shared::plasma_radius(game, selected, contracting, &mut Context::default())
}
pub(super) fn depart_plasma(selected: u16, game: Transaction<'_>) -> Transaction<'_> {
    shared::depart_plasma(game, selected, &mut Context::default())
}
