//! Checked special-attack storage, content and native effect boundary.
use crate::game::special_attacks::{self as shared, Attack, Context, Control, Field, Input, Kind};
use crate::game::projectile_initialization::Initialization;
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, word_arrays as arrays, words};
pub struct Runtime;
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
impl<'g> Context<&'g mut Game> for Runtime {
    fn native_word(&self, g: &&'g mut Game, slot: u16, field: Field) -> u16 {
        let at = offset(field) + usize::from(slot & !1);
        u16::from_ne_bytes([g.g_ram[at], g.g_ram[at + 1]])
    }
    fn write_native(&mut self, g: &mut &'g mut Game, slot: u16, field: Field, value: u16) {
        let at = offset(field) + usize::from(slot & !1);
        g.g_ram[at..at + 2].copy_from_slice(&value.to_ne_bytes());
    }
    fn read(&self, g: &&'g mut Game, slot: u16, field: Field) -> u16 {
        let at = offset(field) + usize::from(slot & !1);
        u16::from_le_bytes([g.g_ram[at], g.g_ram[at + 1]])
    }
    fn low_byte(&self, g: &&'g mut Game, slot: u16, field: Field) -> u8 {
        g.g_ram[offset(field) + usize::from(slot & !1)]
    }
    fn write(&mut self, g: &mut &'g mut Game, slot: u16, field: Field, value: u16) {
        let at = offset(field) + usize::from(slot & !1);
        g.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn motion_is(&self, g: &&'g mut Game, slot: u16, kind: Kind) -> bool {
        word!(g.projectile_bomb_pre_instructions[slot >> 1]) == instruction(kind)
    }
    fn motion(&mut self, g: &mut &'g mut Game, slot: u16, kind: Kind) {
        word!(g.projectile_bomb_pre_instructions[slot >> 1] = instruction(kind));
    }
    fn control(&self, g: &&'g mut Game, field: Control) -> u16 {
        word!(g,control_offset(field);)
    }
    fn set_control(&mut self, g: &mut &'g mut Game, field: Control, value: u16) {
        word!(g,control_offset(field);=value);
    }
    fn input(&self, g: &&'g mut Game, field: Input) -> u16 {
        use Input::*;
        match field {
            SamusX => word!(g.samus_x_pos),
            SamusY => word!(g.samus_y_pos),
            CameraX => word!(g.layer1_x_pos),
            CameraY => word!(g.layer1_y_pos),
            Beams => word!(g.equipped_beams),
            Facing => u16::from(g.samus_pose_x_dir()),
            PowerBombs => word!(g.samus_power_bombs),
        }
    }
    fn initialize(&mut self, g: &'g mut Game, slot: u16, kind: Initialization) -> &'g mut Game {
        crate::game::projectile_initialization_ram::initialize(slot, kind, g);
        g
    }
    fn trail(&mut self, g: &'g mut Game, slot: u16) -> &'g mut Game {
        crate::game::projectile_trails::emit(slot, g);
        g
    }
    fn clear(&mut self, g: &'g mut Game, slot: u16) -> &'g mut Game {
        unsafe { crate::game::sm_90::ClearProjectile(slot, g) };
        g
    }
    fn sound(&mut self, g: &'g mut Game, sound: u16) -> &'g mut Game {
        unsafe { crate::game::sm_80::QueueSfx1_Max6(sound, g) };
        g
    }
    fn power_bombs(&mut self, g: &'g mut Game, value: u16) -> &'g mut Game {
        word!(g.samus_power_bombs = value);
        g
    }
    fn cooldown(&mut self, g: &'g mut Game, index: usize) -> (&'g mut Game, u16) {
        (
            g,
            u16::from(crate::game::sm_90::kProjectileCooldown_Uncharged[index]),
        )
    }
    fn point(&mut self, g: &'g mut Game, angle: u16, radius: u16) -> (&'g mut Game, (u16, u16)) {
        match crate::game::special_geometry::point(
            angle,
            radius,
            crate::game::sm_94::kSinCosTable8bit_Sext.len(),
            |index| {
                crate::game::sm_94::kSinCosTable8bit_Sext
                    .get(index)
                    .map(|&v| u16::from_le(v as u16))
            },
        ) {
            Ok(point) => (g, point),
            Err(e) => self.invalid(g, e.index, e.length),
        }
    }
    fn invalid(&mut self, _: &'g mut Game, index: usize, length: usize) -> ! {
        panic!("index out of bounds: the len is {length} but the index is {index}")
    }
    fn invalid_plasma_phase(&mut self, g: &'g mut Game) -> ! {
        unsafe { crate::rtl::Unreachable(g) };
        loop {
            core::hint::spin_loop();
        }
    }
}
/// Native effects require initialized runtime state.
pub unsafe fn fire(game: &mut Game) -> u8 {
    shared::fire(game, &mut Runtime).1
}
/// Native effects require initialized runtime state.
pub unsafe fn create(attack: Attack, game: &mut Game) -> u8 {
    shared::create(game, attack, &mut Runtime).1
}
/// Native effects require initialized runtime state.
pub unsafe fn update(selected: u16, kind: Kind, game: &mut Game) {
    shared::update(game, selected, kind, &mut Runtime);
}
/// Native effects require initialized runtime state.
pub unsafe fn resume(selected: u16, kind: Kind, original: u16, previous_x: u16, game: &mut Game) {
    shared::resume(game, selected, kind, original, previous_x, &mut Runtime);
}
pub fn plasma_radius(selected: u16, contracting: bool, game: &mut Game) {
    shared::plasma_radius(game, selected, contracting, &mut Runtime);
}
/// Native effects require initialized runtime state.
pub unsafe fn depart_plasma(selected: u16, game: &mut Game) {
    shared::depart_plasma(game, selected, &mut Runtime);
}
