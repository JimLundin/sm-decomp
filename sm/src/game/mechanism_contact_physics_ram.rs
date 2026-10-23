//! Scoped motion fields and lazy displacement access; fatal diagnostics remain native.
use crate::game::mechanism_displacement::Displacement;
use crate::game::mechanism_contact_physics::{self, Context, Field, Setup};
use crate::game_state::Game;

pub fn offset(field: Field) -> usize {
    use Field::*;
    match field {
        YRadius => 0xb00,
        YPosition => 0xafa,
        Deceleration => 0xb4c,
        Momentum => 0xb3c,
        BoostCounter => 0xb3e,
        EchoSound => 0xb40,
        ExtraRunFraction => 0xb44,
        ExtraRunSpeed => 0xb42,
        BaseSpeed => 0xb46,
        Equipment => 0x9a2,
        ReactionPoint => 0x1e73,
        YDirection => 0xb36,
        YSpeedFraction => 0xb2c,
        YSpeed => 0xb2e,
        YSpeedMiddle => 0xb2d,
        ExternalYFraction => 0xb5a,
        ExternalY => 0xb5c,
        ExternalYMiddle => 0xb5b,
        YAccelerationFraction => 0xb32,
        YAcceleration => 0xb34,
        AutoJump => 0xaf4,
        CollisionDirection => 0xb02,
        ContactDamage => 0xa6e,
        SandVisits => 0x1e71,
    }
}
pub fn read(game: &Game, field: Field) -> u16 {
    let at = offset(field);
    u16::from_le_bytes([game.g_ram[at], game.g_ram[at + 1]])
}
pub fn write(game: &mut Game, field: Field, value: u16) {
    let at = offset(field);
    game.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
pub fn clear_base_fraction_sign(game: &mut Game) {
    game.g_ram[0xb49] &= 0x7f;
}
pub unsafe fn missing_displacement(game: &mut Game) {
    unsafe {
        crate::c_library::printf(c"ERROR: PlmSetup_QuicksandSurfaceB is broken!!\n".as_ptr());
        crate::rtl::Unreachable(game);
    }
}
struct MotionFields<'a, 'b> {
    game: &'a mut Game,
    displacement: &'a mut Displacement<'b>,
}
impl Context for MotionFields<'_, '_> {
    fn read(&self, field: Field) -> u16 {
        read(self.game, field)
    }
    fn write(&mut self, field: Field, value: u16) {
        write(self.game, field, value);
    }
    fn clear_base_fraction_sign(&mut self) {
        clear_base_fraction_sign(self.game);
    }
    fn has_displacement(&self) -> bool {
        self.displacement.present()
    }
    fn displacement(&self) -> i32 {
        self.displacement.read(self.game)
    }
    fn set_displacement(&mut self, value: i32) {
        self.displacement.write(value, self.game);
    }
    fn missing_displacement(&mut self) {
        unsafe {
            missing_displacement(self.game);
        }
    }
}
/// # Safety
/// The missing-displacement diagnostic requires the initialized native runtime.
pub unsafe fn setup(kind: Setup, displacement: &mut Displacement<'_>, game: &mut Game) -> u8 {
    mechanism_contact_physics::setup(&mut MotionFields { game, displacement }, kind)
}
