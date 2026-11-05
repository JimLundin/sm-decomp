//! What the engine's frame needs of the machine, with no CPU emulator.
use crate::game_state::Game;
pub use crate::snes::registers::*;
pub use crate::types::{BotwoonLayout, Cpu, Dma, Ppu, Snes};
#[path = "game/session_native_adoption.rs"]
pub(crate) mod session_native_adoption;
pub use crate::types::ProjectileInstr;
pub use crate::types::RunMode;
pub const RM_MINE: RunMode = 1;
pub const RM_BOTH: RunMode = 0;
pub const RM_THEIRS: RunMode = 2;
pub const NULL: *mut core::ffi::c_void = core::ptr::null_mut();

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

/// The oracle's patches are not installed here.
pub fn rtl_update_snes_patch_for_bugfix(_game: &mut Game) {}
pub(crate) use crate::lock_masks::{LOCK_SESSION_MASK, LOCK_WRAM_MASK};
crate::typed_struct! {
    initial; [#[derive(Clone)] #[repr(C)]];
    pub struct Snapshot {
        pub audio: crate::audio_snapshot::AudioSnapshot;
        pub player: crate::audio_snapshot::PlayerSnapshot;
        pub cpu: Cpu;
        pub ppu: Ppu;
        pub dma: Dma;
        pub snes: Snes;
        pub ram: [u8; 131072];
        pub(crate) source_publications: crate::game_state::SourcePublicationSnapshot;
        pub sram: [u8; 8192];
        pub apu_upload_count: u32;
        pub apu_startup_upload: bool;
    }
}
pub(crate) unsafe fn make_snapshot_with_player_owner(
    s: &mut Snapshot,
    snes: &Snes,
    c: &Cpu,
    owner: Option<&mut crate::spc_player::NativePlayer>,
    ram: &crate::game_state::WorkRam,
    backend: &crate::snes::snes::Backend,
) {
    unsafe {
        s.player.capture_from(owner.as_deref());
        s.audio.capture(snes);
        s.ppu = *snes.ppu;
        s.cpu = *c;
        s.snes = *snes;
        let dma = backend.dma_state();
        s.dma.channel = dma.channel;
        s.dma.hdmaTimer = dma.hdmaTimer;
        s.dma.dmaTimer = dma.dmaTimer;
        s.dma.dmaBusy = dma.dmaBusy;
        ram.copy_out(0, &mut s.ram);
        s.source_publications = ram.capture_source_publications();
        (*snes.cart)
            .ram
            .read_bytes(0, &mut s.sram[..(*snes.cart).ramSize as usize]);
    }
}
/// One engine frame: the only run mode this host has.
pub unsafe fn rtl_run_frame_compare(input: u16, _run_what: i32, native_game: &mut Game) {
    let game = &mut *native_game;
    unsafe {
        game.preflight_samus_runtime()
            .expect("Samus frame entry rejected");
        let mut mutation = game.samus_mutation();
        let game = mutation.game();
        (*game.g_snes).runningWhichVersion = 0xff;
        let position = game.audio_execution_position();
        game.audio_output.enter_execution(position);
        let buttons = input.reverse_bits();
        crate::game::session_runtime::with_frame_session(game, |game| {
            crate::game::gameplay::tick(crate::game::gameplay::TickInput { buttons }, game)
        });
        finish_settled_frame(game);
        (*game.g_snes).runningWhichVersion = 0;
        mutation
            .finish_current()
            .expect("Samus frame adoption rejected");
        session_native_adoption::reconcile_current(native_game)
            .expect("Session native result adoption rejected");
    }
}
