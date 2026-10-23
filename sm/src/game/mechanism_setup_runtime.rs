//! Native setup effects and diagnostics with explicit contact-displacement transport.
use crate::game::mechanism_displacement::Displacement;
use crate::game::mechanism_setup::{self, Contact, Diagnostic};
use crate::game::mechanism_setup_ram::{self, Boundary};
use crate::game_state::Game;
use crate::c_library::__assert_fail;
use core::ffi::c_char;

/// # Safety
/// Native setup callbacks and fatal diagnostics require initialized runtime state.
pub unsafe fn execute(
    original: u32,
    j: u16,
    displacement: &mut Displacement<'_>,
    accumulator: u16,
    game: &mut Game,
) -> u8 {
    match mechanism_setup_ram::execute(
        mechanism_setup::decode(original).action(),
        usize::from(j >> 1),
        accumulator,
        game,
    ) {
        Ok(value) => value,
        Err(Boundary::Contact(Contact::Physics(kind))) => unsafe {
            crate::game::mechanism_contact_physics_ram::setup(kind, displacement, game)
        },
        Err(Boundary::Contact(Contact::Hand(hand))) => unsafe {
            crate::game::mechanism_hands_ram::setup(j, hand, displacement, game)
        },
        Err(Boundary::Diagnostic(Diagnostic::UnsupportedB3E3)) => unsafe {
            crate::native_assertion! {
                '_c2rust_label;
                b"0\0", b"src/sm_84.c\0", 1370,
                b"uint8 UNUSED_sub_84B3E3(uint16)\0"
            };
            0
        },
        Err(Boundary::External(_)) => unsafe { crate::rtl::Unreachable(game) as u8 },
    }
}
