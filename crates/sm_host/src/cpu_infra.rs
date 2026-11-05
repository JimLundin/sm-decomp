//! What the engine's frame needs of the machine, with no CPU emulator.
use crate::game_state::Game;
pub use crate::snes::registers::*;
pub use crate::types::{BotwoonLayout, Cpu, Dma, Ppu, Snes};
#[path = "../../../sm/src/game/session_native_adoption.rs"]
pub(crate) mod session_native_adoption;
pub use crate::types::ProjectileInstr;
pub use crate::types::RunMode;
pub const RM_MINE: RunMode = 1;
pub const RM_BOTH: RunMode = 0;
pub const RM_THEIRS: RunMode = 2;

pub(crate) mod beam {
    use crate::game_state::Game;

    /// # Safety
    /// The backend must be installed and exclusively accessed, as for every
    /// function here.
    pub(crate) unsafe fn reset(game: &mut Game) {
        unsafe {
            (*game.g_snes).vPos = 0;
            (*game.g_snes).hPos = (*game.g_snes).vPos;
        }
    }
    pub(crate) unsafe fn nmi_wanted(game: &Game) -> bool {
        unsafe { (*(*game.g_snes).cpu).nmiWanted }
    }
    /// One beam event at `position` on the current line: the raster, HDMA
    /// reading its tables from work RAM, or the line's wrap.
    pub(crate) unsafe fn advance_at(game: &mut Game, position: u16) {
        unsafe {
            (*game.g_snes).hPos = position;
            game.execution().advance_beam();
        }
    }
    pub(crate) unsafe fn vertical_irq_enabled(game: &Game) -> bool {
        unsafe { (*game.g_snes).vIrqEnabled }
    }
    pub(crate) unsafe fn vertical_position(game: &Game) -> u16 {
        unsafe { (*game.g_snes).vPos }
    }
    pub(crate) unsafe fn vertical_timer(game: &Game) -> u16 {
        unsafe { (*game.g_snes).vTimer }
    }
    pub(crate) unsafe fn clear_nmi(game: &mut Game) {
        unsafe { (*(*game.g_snes).cpu).nmiWanted = false };
    }
}

pub unsafe fn debug_game_over_menu() {
    unsafe {
        crate::native_assertion! {
            '_c2rust_label;
            b"0\0", b"src/sm_cpu_infra.c\0", 914, b"void DebugGameOverMenu(void)\0"
        };
    }
}

pub unsafe fn finish_settled_frame(native_game: &mut Game) {
    unsafe { adopt_settlement(native_game, |_| ()) }
}
unsafe fn adopt_settlement(native_game: &mut Game, settle: impl FnOnce(&mut Game)) {
    let game = &mut *native_game;
    game.preflight_samus_runtime()
        .expect("Samus settlement entry rejected");
    let mut mutation = game.samus_mutation();
    settle(mutation.game());
    mutation
        .finish_current()
        .expect("Samus settlement adoption rejected");
    session_native_adoption::reconcile_current(native_game)
        .expect("Session native result adoption rejected");
}
