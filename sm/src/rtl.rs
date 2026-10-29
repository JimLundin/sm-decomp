use crate::execution::{DmaRomSource, Execution, HostContext};
use crate::snapshot_io::{Block, SnapshotIo, TransferError};
// Split-word assembly evaluates high then low. The i32 form retains signed
// casts, including sign extension of the low operand. Repeated receivers and
// operands are evaluated separately. Set/add keep pointer-before-value order
// and the helpers' low-before-high writes, including unaligned/overlapping words.
macro_rules! words {
    ($game:ident.$hi:ident, $lo:ident += $value:expr $(,)?) => {
        match (
            $game.ram().share_for_backend(),
            $crate::ram_map::words::$hi,
            $crate::ram_map::words::$lo,
            $value,
        ) {
            (ram, hi, lo, value) => {
                let value: u32 = value;
                let mut bytes = [0; 2];
                ram.copy_out(lo, &mut bytes);
                let low = u32::from(u16::from_ne_bytes(bytes));
                ram.copy_out(hi, &mut bytes);
                let high = u32::from(u16::from_ne_bytes(bytes));
                let sum = (low | (high << 16)).wrapping_add(value);
                ram.copy_in(lo, &(sum as u16).to_ne_bytes());
                ram.copy_in(hi, &((sum >> 16) as u16).to_ne_bytes());
            }
        }
    };
    ($game:ident.$hi:ident, $lo:ident = $value:expr $(,)?) => {
        match (
            $game.ram().share_for_backend(),
            $crate::ram_map::words::$hi,
            $crate::ram_map::words::$lo,
            $value,
        ) {
            (ram, hi, lo, value) => {
                let value: u32 = value;
                ram.copy_in(lo, &(value as u16).to_ne_bytes());
                ram.copy_in(hi, &((value >> 16) as u16).to_ne_bytes());
            }
        }
    };
    ($base:expr, $hi:ident, $lo:ident += $value:expr $(,)?) => {
        $crate::rtl::AddToHiLo(&raw mut ($base).$hi, &raw mut ($base).$lo, $value)
    };
    ($base:expr, $hi:ident, $lo:ident = $value:expr $(,)?) => {
        $crate::rtl::SetHiLo(&raw mut ($base).$hi, &raw mut ($base).$lo, $value)
    };
    (i32, $game:ident.$hi:ident, $lo:ident $(,)?) => {
        $crate::rtl::words!(@join i32; $crate::ram_map::word!($game.$hi), $crate::ram_map::word!($game.$lo))
    };
    ($game:ident.$hi:ident, $lo:ident $(,)?) => {
        $crate::rtl::words!(@join u32; $crate::ram_map::word!($game.$hi), $crate::ram_map::word!($game.$lo))
    };
    (i32, $base:expr, $hi:ident, $lo:ident $(,)?) => {
        $crate::rtl::words!(@join i32; ($base).$hi, ($base).$lo)
    };
    ($base:expr, $hi:ident, $lo:ident $(,)?) => {
        $crate::rtl::words!(@join u32; ($base).$hi, ($base).$lo)
    };
    (i32, ($hi:expr, $lo:expr $(,)?) $(,)?) => {
        $crate::rtl::words!(@join i32; $hi, $lo)
    };
    (($hi:expr, $lo:expr $(,)?) $(,)?) => {
        $crate::rtl::words!(@join u32; $hi, $lo)
    };
    (@join $ty:ty; $hi:expr, $lo:expr) => {
        (($hi as $ty) << 16i32) | ($lo as $ty)
    };
}
pub(crate) use words;

/// Apply an ordered register sequence without pre-evaluating later values.
macro_rules! register_writes {
    ($game:ident; $($kind:ident($($argument:tt)*);)*) => {
        $($crate::rtl::register_writes!(@write $game; $kind($($argument)*));)*
    };
    (@write $game:ident; byte($address:expr, $value:expr $(, $($tail:tt)*)?)) => {
        $crate::rtl::WriteReg($address, $value, $game $(, $($tail)*)?)
    };
    (@write $game:ident; word($address:expr, $value:expr $(, $($tail:tt)*)?)) => {
        $crate::rtl::WriteRegWord($address, $value, $game $(, $($tail)*)?)
    };
    (@write $game:ident; mirror($address:expr, $value:expr, $offset:expr, $before:expr $(, $($tail:tt)*)?)) => {
        $crate::rtl::write_register_and_mirror($address, $value, $offset, $before, $game $(, $($tail)*)?)
    };
}
pub(crate) use register_writes;

/// Construct a fixed LoROM pointer at the caller, before any later offsets.
macro_rules! rom_ptr {
    ($game:ident; $address:literal) => {
        $game
            .legacy_native_rom_ptr()
            .offset($crate::rtl::lorom_offset($address))
    };
    ($game:ident, $target:ty; $address:literal) => {
        $game
            .legacy_native_rom_ptr()
            .offset($crate::rtl::lorom_offset($address)) as *mut $target
    };
}
pub(crate) use rom_ptr;

/// Capture the shared image before evaluating a table's signed word index.
/// Borrow only the ROM field: index expressions can mutate Game or retain RAM pointers.
macro_rules! rom_table_word {
    ($game:ident, $address:expr, $index:expr $(,)?) => {
        $game
            .loaded_rom
            .as_ref()
            .expect("ROM must be loaded")
            .clone()
            .table_word($crate::rtl::lorom_offset($address) as usize, $index)
    };
}
pub(crate) use rom_table_word;

use crate::c_library::{
    fclose, fopen, fprintf, fread, fseek, fwrite, memcpy, printf, rename, sprintf, stderr,
};
use crate::cpu_infra::RtlUpdateSnesPatchForBugfix;
use crate::game_state::Game;
use crate::host::{Die, RtlApuLock, RtlApuUnlock};
use crate::initial_state::InitialState;
use crate::ram_map::word;
use crate::recording::RecordingBytes;
pub use crate::recording::StateRecorder;
use crate::snes::ppu::ppu_copy;
pub use crate::snes::registers::*;
use crate::spc_player::SpcPlayer_Upload;
pub use crate::types::Dsp;
pub use crate::types::{Cpu, FILE, LongPtr, PairU16, Snes, VoidP, size_t};
pub use crate::types::{RunFrameFunc, SyncAllFunc};
use ::core::ffi::{c_char, c_long, c_void};
use ::core::mem::size_of;
use ::core::ptr::null_mut;

pub use crate::types::SaveLoadOperation as C2RustUnnamed_0;
pub const kSaveLoad_Replay: C2RustUnnamed_0 = 3;
pub const kSaveLoad_Load: C2RustUnnamed_0 = 2;
pub const kSaveLoad_Save: C2RustUnnamed_0 = 1;

pub const kCurrentBugFixCounter: C2RustUnnamed_1 = 1;
crate::type_aliases! {
    pub C2RustUnnamed_1 = u32;
}
pub const NULL: *mut c_void = null_mut::<c_void>();
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
pub const SEEK_SET: i32 = 0;
pub const SEEK_END: i32 = 2;

pub unsafe fn RtlSetupEmuCallbacks(
    emu_ram: *mut u8,
    func: Option<RunFrameFunc>,
    sync_all: Option<SyncAllFunc>,
    game: &mut Game,
) {
    game.g_rtl_memory_ptr = emu_ram;
    game.g_rtl_runframe = func;
    game.g_rtl_syncall = sync_all;
}
unsafe fn RtlSynchronizeWholeState(game: &mut Game) {
    unsafe {
        synchronize_whole_state(game, true);
    }
}
unsafe fn synchronize_whole_state(native_game: &mut Game, complete_load: bool) {
    let game = &mut *native_game;
    unsafe {
        if let Some(sync) = game.g_rtl_syncall {
            let mut mutation = game
                .begin_samus_mutation()
                .expect("Samus mutation rejected");
            sync(mutation.game());
            if complete_load {
                mutation
                    .finish_current()
                    .expect("Samus synchronization adoption rejected");
                crate::cpu_infra::session_native_adoption::reconcile_current(native_game)
                    .expect("Session native continuation adoption rejected");
            } else {
                mutation.defer_incomplete();
            }
        }
    }
}

// Preserve the existing host diagnostic if an owned buffer cannot grow.
pub unsafe fn recording_allocation(result: crate::recording::BufferResult) {
    unsafe {
        if result.is_err() {
            Die(c"memory allocation failed".as_ptr());
            panic!("allocation error callback returned");
        }
    }
}
// Native diagnostics stay at the boundary; stream bookkeeping is checked Rust.
pub(crate) unsafe fn snapshot_result(result: Result<(), TransferError>) {
    unsafe {
        match result {
            Ok(()) => {}
            Err(TransferError::Allocation(error)) => recording_allocation(Err(error)),
            Err(TransferError::Truncated) => {
                crate::native_assertion! {
                    '_c2rust_label;
                    b"(size_t)(st->pend - st->p) >= data_size\0", b"src/sm_rtl.c\0", 59,
                    b"void loadFunc(void *, void *, size_t)\0"
                };
            }
        }
    }
}

// Hardware regions include padding and retained raw aliases. Keep raw copies
// until field-based serializers replace them; never borrow all RAM mutably.
pub(crate) unsafe fn snapshot_region(io: &mut SnapshotIo<'_>, data: *mut u8, size: usize) {
    unsafe {
        let result = io.block(size, |block| {
            if size != 0 {
                let (destination, source) = match block {
                    Block::Save(bytes) => (bytes.as_mut_ptr(), data as *const u8),
                    Block::Load(bytes) => (data, bytes.as_ptr()),
                };
                memcpy(destination.cast(), source.cast(), size);
            }
        });
        snapshot_result(result);
    }
}

pub unsafe fn FinishLoadSnesState(game: &mut Game) {
    unsafe {
        finish_load_snes_state(game, true);
    }
}
unsafe fn finish_load_snes_state(native_game: &mut Game, complete_load: bool) {
    let game = &mut *native_game;
    unsafe {
        let mut mutation = game
            .begin_samus_mutation()
            .expect("Samus mutation rejected");
        let game = mutation.game();
        crate::snes::ppu::PpuRestoreRenderRouting(&mut *(*game.g_snes).ppu);
        (*(*game.g_snes).cpu).e = false_0 != 0;
        let ram = game.ram();
        let next: u32 = ((ram.read_byte(((*(*game.g_snes).cpu).sp as i32 + 3) as usize) as i32
            | (ram.read_byte(((*(*game.g_snes).cpu).sp as i32 + 4) as usize) as i32) << 8
            | (ram.read_byte(((*(*game.g_snes).cpu).sp as i32 + 5) as usize) as i32) << 16)
            + 1) as u32;
        if next == 0x82897e {
            ram.write_byte(
                ((*(*game.g_snes).cpu).sp as i32 + 3) as usize,
                (0xf71b - 1i32 & 0xff) as u8,
            );
            ram.write_byte(
                ((*(*game.g_snes).cpu).sp as i32 + 4) as usize,
                (0xf71b - 1i32 >> 8) as u8,
            );
        }
        synchronize_whole_state(game, complete_load);
        if complete_load {
            mutation
                .finish_current()
                .expect("Samus loaded-state adoption rejected");
            crate::cpu_infra::session_native_adoption::reconcile_current(native_game)
                .expect("Session native continuation adoption rejected");
        } else {
            mutation.defer_incomplete();
        }
    }
}

pub unsafe fn ReadFromFile(f: *mut FILE, data: *mut c_void, n: size_t) {
    unsafe {
        let _ = read_file_block(f, data, n);
    }
}
unsafe fn read_file_block(f: *mut FILE, data: *mut c_void, n: size_t) -> bool {
    unsafe {
        let complete = fread(data, 1 as size_t, n, f) as size_t == n;
        if !complete {
            Die(c"fread failed\n".as_ptr());
        }
        complete
    }
}
pub unsafe fn RtlReset(mode: i32, game: &mut Game) {
    unsafe {
        reset(mode, game, true);
        game.audio_output.new_epoch();
    }
}
unsafe fn reset(mode: i32, native_game: &mut Game, complete_load: bool) {
    let game = &mut *native_game;
    unsafe {
        let actor_transition = game.prepare_actor_canonical();
        let payload = game.capture_samus_snapshot();
        game.prepare_samus_adoption(&payload)
            .expect("Samus reset preflight rejected");
        let mut mutation = game
            .begin_samus_mutation()
            .expect("Samus mutation rejected");
        let game = mutation.game();
        game.snes_frame_counter = 0;
        game.execution().reset(true_0 != 0);
        if mode & 1 == 0 {
            game.g_sram.fill(0..0x2000, 0);
        }
        game.ram().write_byte(0x611, 1);
        RtlApuLock();
        RtlRestoreMusicAfterLoad_Locked(true_0 != 0, game);
        RtlApuUnlock();
        synchronize_whole_state(game, complete_load);
        game.commit_actor_canonical(actor_transition);
        if mode & 2 == 0 {
            game.state_recorder = StateRecorder::default();
        }
        if complete_load {
            mutation
                .finish_current()
                .expect("Samus reset adoption rejected");
            crate::cpu_infra::session_native_adoption::reconcile_current(native_game)
                .expect("Session native continuation adoption rejected");
        } else {
            mutation.defer_incomplete();
        }
    }
}
pub unsafe fn StateRecorder_Load(f: *mut FILE, replay_mode: bool, game: &mut Game) {
    unsafe {
        let _ = state_recorder_load(f, replay_mode, game, false);
    }
}
unsafe fn state_recorder_load(
    f: *mut FILE,
    replay_mode: bool,
    native_game: &mut Game,
    outer_load: bool,
) -> bool {
    let game = &mut *native_game;
    unsafe {
        if !game.supports_legacy_persistence() {
            eprintln!("StateRecorder_Load: legacy persistence is unavailable for a prepared world");
            return false;
        }
        if let Err(error) = crate::cpu_infra::session_native_adoption::require_legacy_binary(game) {
            eprintln!("state_recorder_load: {error}");
            return false;
        }
        let mut hdr: [u32; 16] = [0i32 as u32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let is_old: bool = false_0 != 0;
        let mut read_complete = true;
        let mut is_reset: bool = false_0 != 0;
        read_complete &= read_file_block(
            f,
            &raw mut hdr as *mut u32 as *mut c_void,
            (8 as size_t).wrapping_mul(size_of::<u32>() as size_t),
        );
        if hdr[0i32 as usize] != 2 {
            hdr[8i32 as usize] = hdr[7i32 as usize];
            hdr[7i32 as usize] = hdr[5i32 as usize] >> 1;
            hdr[5i32 as usize] = if hdr[5i32 as usize] & 1 != 0 {
                hdr[6i32 as usize]
            } else {
                0
            };
        } else if hdr[0i32 as usize] == 2 {
            read_complete &= read_file_block(
                f,
                (&raw mut hdr as *mut u32).offset(8) as *mut c_void,
                (8 as size_t).wrapping_mul(size_of::<u32>() as size_t),
            );
        } else {
            crate::native_assertion! {
                '_c2rust_label;
                b"0\0", b"src/sm_rtl.c\0", 195,
                b"void StateRecorder_Load(StateRecorder *, FILE *, _Bool)\0"
            };
        }
        let mut actor_transition = Some(game.prepare_actor_canonical());
        let payload = game.capture_samus_snapshot();
        game.prepare_samus_adoption(&payload)
            .expect("Samus load preflight rejected");
        let mut mutation = game
            .begin_samus_mutation()
            .expect("Samus mutation rejected");
        let game = mutation.game();
        game.state_recorder.total_frames = hdr[1i32 as usize];
        recording_allocation(game.state_recorder.log.resize(hdr[2] as usize));
        read_complete &= read_file_block(
            f,
            game.state_recorder.log.as_mut_ptr() as *mut c_void,
            game.state_recorder.log.size,
        );
        game.state_recorder.last_inputs = hdr[3i32 as usize] as u16;
        game.state_recorder.frames_since_last = hdr[4i32 as usize];
        recording_allocation(game.state_recorder.base_snapshot.resize(hdr[5] as usize));
        read_complete &= read_file_block(
            f,
            game.state_recorder.base_snapshot.as_mut_ptr() as *mut c_void,
            game.state_recorder.base_snapshot.size,
        );
        game.state_recorder.has_base_audio_queue = false;
        game.state_recorder.snapshot_flags = hdr[9i32 as usize];
        game.state_recorder.replay_next_cmd_at = 0;
        game.state_recorder.replay_mode = replay_mode;
        if replay_mode {
            crate::set_fields! {[game.state_recorder]; frames_since_last = 0, last_inputs = 0,
            replay_pos_last_complete = 0, replay_pos = game.state_recorder.replay_pos_last_complete,
            replay_frame_counter = 0, };
            if game.state_recorder.base_snapshot.size > crate::recording::BASE_SRAM_SIZE as size_t {
                let complete = {
                    let (mut execution, recorder) = game.execution_and_recorder();
                    let mut state = SnapshotIo::Load(recorder.base_snapshot.as_slice());
                    execution.save_load(&mut state);
                    state.remaining() == 0
                };
                finish_load_snes_state(game, read_complete);
                crate::native_assertion! {
                    '_c2rust_label_0: complete;
                    b"state.p == state.pend\0", b"src/sm_rtl.c\0", 219,
                    b"void StateRecorder_Load(StateRecorder *, FILE *, _Bool)\0"
                };
            } else {
                drop(actor_transition.take());
                reset(2, game, read_complete);
                if game.state_recorder.base_snapshot.size
                    == crate::recording::BASE_SRAM_SIZE as size_t
                {
                    game.g_sram.write_bytes(
                        0,
                        &game.state_recorder.base_snapshot.as_slice()
                            [..crate::recording::BASE_SRAM_SIZE],
                    );
                }
                is_reset = true_0 != 0;
            }
        } else {
            game.state_recorder.replay_pos_last_complete = hdr[7i32 as usize];
            game.state_recorder.replay_pos = game.state_recorder.replay_pos_last_complete;
            game.state_recorder.replay_frame_counter = hdr[8i32 as usize];
            game.state_recorder.replay_mode = game.state_recorder.replay_frame_counter != 0;
            crate::native_assertion! {
                '_c2rust_label_1: hdr[6i32 as usize] == 275493;
                b"hdr[6] == 275493\0", b"src/sm_rtl.c\0", 232,
                b"void StateRecorder_Load(StateRecorder *, FILE *, _Bool)\0"
            };
            let mut arr = RecordingBytes::default();
            recording_allocation(arr.resize(hdr[6] as usize));
            read_complete &= read_file_block(f, arr.as_mut_ptr() as *mut c_void, arr.size);
            let mut state = SnapshotIo::Load(arr.as_slice());
            game.execution().save_load(&mut state);
            let complete = state.remaining() == 0;
            finish_load_snes_state(game, read_complete);
            drop(arr);
            crate::native_assertion! {
                '_c2rust_label_2: complete;
                b"state.p == state.pend\0", b"src/sm_rtl.c\0", 240,
                b"void StateRecorder_Load(StateRecorder *, FILE *, _Bool)\0"
            };
            if is_old {
                RtlClearKeyLog(game);
            }
        }
        if !is_reset {
            RtlRestoreMusicAfterLoad_Locked(false_0 != 0, game);
        }
        // The extension describes the final snapshot, not a replay's base state.
        if !replay_mode && hdr[10] == u32::from_le_bytes(*b"APQ1") {
            assert_eq!(hdr[11], 71, "invalid saved APU queue length");
            let mut bytes = [0; 71];
            read_complete &= read_file_block(f, bytes.as_mut_ptr().cast(), bytes.len());
            if !game.g_use_my_apu_code {
                crate::audio_snapshot::ApuQueueSnapshot::from_bytes(&bytes).restore(game);
            }
        }
        if hdr[12] == u32::from_le_bytes(*b"ABQ1") {
            assert_eq!(hdr[13], 71, "invalid replay base APU queue length");
            if replay_mode {
                // Replay read the base snapshot, leaving the final snapshot unread.
                let final_queue_size = if hdr[10] == u32::from_le_bytes(*b"APQ1") {
                    assert_eq!(hdr[11], 71);
                    71
                } else {
                    0
                };
                let skip = c_long::try_from(hdr[6])
                    .unwrap()
                    .checked_add(final_queue_size)
                    .unwrap();
                assert_eq!(fseek(f, skip, 1), 0);
            }
            read_complete &= read_file_block(
                f,
                game.state_recorder.base_audio_queue.as_mut_ptr().cast(),
                71,
            );
            game.state_recorder.has_base_audio_queue =
                game.state_recorder.base_snapshot.size > crate::recording::BASE_SRAM_SIZE;
            if replay_mode && !is_reset {
                crate::audio_snapshot::ApuQueueSnapshot::from_bytes(
                    &game.state_recorder.base_audio_queue,
                )
                .restore(game);
            }
        }
        RtlUpdateSnesPatchForBugfix(game);
        if let Some(transition) = actor_transition {
            game.commit_actor_canonical(transition);
        }
        if read_complete {
            mutation
                .finish_current()
                .expect("Samus load adoption rejected");
            crate::cpu_infra::session_native_adoption::reconcile_current(native_game)
                .expect("Session native continuation adoption rejected");
            native_game.audio_output.new_epoch();
        } else if outer_load {
            mutation.defer_incomplete();
        }
        read_complete
    }
}
pub unsafe fn StateRecorder_Save(f: *mut FILE, saving_with_bug: bool, native_game: &mut Game) {
    let game = &mut *native_game;
    unsafe {
        if !game.supports_legacy_persistence() {
            eprintln!("StateRecorder_Save: legacy persistence is unavailable for a prepared world");
            return;
        }
        if let Err(error) = crate::cpu_infra::session_native_adoption::require_legacy_binary(game) {
            eprintln!("StateRecorder_Save: {error}");
            return;
        }
        game.preflight_samus_runtime()
            .expect("Samus save entry rejected");
        let mut mutation = game
            .begin_samus_mutation()
            .expect("Samus mutation rejected");
        let game = mutation.game();
        let mut hdr: [u32; 16] = [0i32 as u32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut arr = RecordingBytes::default();
        game.execution().save_load(&mut SnapshotIo::Save(&mut arr));
        let sr = &game.state_recorder;
        crate::native_assertion! {
            '_c2rust_label: sr.base_snapshot.size == 0 as size_t
                || sr.base_snapshot.size == arr.size
                || sr.base_snapshot.size == crate::recording::BASE_SRAM_SIZE as size_t;
            b"sr->base_snapshot.size == 0 || sr->base_snapshot.size == arr.size || sr->base_snapshot.size == 8192\0",
            b"src/sm_rtl.c\0", 260, b"void StateRecorder_Save(StateRecorder *, FILE *, _Bool)\0"
        };
        hdr[0i32 as usize] = 2;
        hdr[1i32 as usize] = sr.total_frames;
        hdr[2i32 as usize] = sr.log.size as u32;
        hdr[3i32 as usize] = sr.last_inputs as u32;
        hdr[4i32 as usize] = sr.frames_since_last;
        hdr[5i32 as usize] = sr.base_snapshot.size as u32;
        hdr[6i32 as usize] = arr.size as u32;
        if sr.replay_mode {
            hdr[7i32 as usize] = sr.replay_pos_last_complete;
            hdr[8i32 as usize] = sr.replay_frame_counter;
        }
        hdr[9i32 as usize] = (saving_with_bug as i32 * 1) as u32;
        // Reserved version-2 header words identify a final physical queue trailer.
        // Replay bases retain the host queue for either audio backend.
        let audio_queue = (!game.g_use_my_apu_code)
            .then(|| crate::audio_snapshot::ApuQueueSnapshot::capture(game).to_bytes());
        if audio_queue.is_some() {
            hdr[10] = u32::from_le_bytes(*b"APQ1");
            hdr[11] = 71;
        }
        if sr.has_base_audio_queue {
            hdr[12] = u32::from_le_bytes(*b"ABQ1");
            hdr[13] = 71;
        }
        fwrite(
            &raw mut hdr as *mut u32 as *const c_void,
            1 as size_t,
            size_of::<[u32; 16]>() as size_t,
            f,
        );
        fwrite(
            sr.log.as_ptr() as *const c_void,
            1 as size_t,
            sr.log.size,
            f,
        );
        fwrite(
            sr.base_snapshot.as_ptr() as *const c_void,
            1 as size_t,
            sr.base_snapshot.size,
            f,
        );
        fwrite(arr.as_ptr() as *const c_void, 1 as size_t, arr.size, f);
        if let Some(bytes) = audio_queue {
            fwrite(bytes.as_ptr().cast(), 1, bytes.len(), f);
        }
        if sr.has_base_audio_queue {
            fwrite(sr.base_audio_queue.as_ptr().cast(), 1, 71, f);
        }
        drop(arr);
        mutation
            .finish_current()
            .expect("Samus save adoption rejected");
        crate::cpu_infra::session_native_adoption::reconcile_current(native_game)
            .expect("Session native continuation adoption rejected");
    }
}
pub unsafe fn StateRecorder_ClearKeyLog(native_game: &mut Game) {
    let game = &mut *native_game;
    unsafe {
        if let Err(error) = crate::cpu_infra::session_native_adoption::require_legacy_binary(game) {
            eprintln!("StateRecorder_ClearKeyLog: {error}");
            return;
        }
        game.preflight_samus_runtime()
            .expect("Samus recorder entry rejected");
        let mut mutation = game
            .begin_samus_mutation()
            .expect("Samus mutation rejected");
        let game = mutation.game();
        printf(c"Clearing key log!\n".as_ptr());
        game.state_recorder.base_snapshot.size = 0 as size_t;
        game.state_recorder.has_base_audio_queue = true;
        game.state_recorder.base_audio_queue =
            crate::audio_snapshot::ApuQueueSnapshot::capture(game).to_bytes();
        let live_audio = if game.g_use_my_apu_code {
            let mut snapshots = Box::new((
                crate::audio_snapshot::AudioSnapshot::INITIAL,
                crate::audio_snapshot::PlayerSnapshot::INITIAL,
            ));
            snapshots.0.capture(game.g_snes);
            game.capture_native_player_snapshot(&mut snapshots.1);
            // Replay restores the queue separately, so do not deliver it early.
            RtlSaveMusicStateToRam_Locked(game, false);
            Some(snapshots)
        } else {
            None
        };
        {
            let (mut execution, recorder) = game.execution_and_recorder();
            execution.save_load(&mut SnapshotIo::Save(&mut recorder.base_snapshot));
        }
        if let Some(snapshots) = live_audio {
            snapshots.0.restore(game.g_snes);
            game.restore_native_player_snapshot(crate::game_state::PlayerSnapshotSource::Local(
                &snapshots.1,
            ));
        }
        recording_allocation(game.state_recorder.rebase());
        mutation
            .finish_current()
            .expect("Samus recorder adoption rejected");
        crate::cpu_infra::session_native_adoption::reconcile_current(native_game)
            .expect("Session native result adoption rejected");
    }
}
pub unsafe fn StateRecorder_ReadNextReplayState(game: &mut Game) -> u16 {
    use crate::recording::log_format;
    unsafe {
        crate::native_assertion! {
            '_c2rust_label: game.state_recorder.replay_mode;
            b"sr->replay_mode\0", b"src/sm_rtl.c\0", 322,
            b"uint16 StateRecorder_ReadNextReplayState(StateRecorder *)\0"
        };
        while game.state_recorder.frames_since_last >= game.state_recorder.replay_next_cmd_at {
            let mut replay_pos: i32 = game.state_recorder.replay_pos as i32;
            if replay_pos as u32 != game.state_recorder.replay_pos_last_complete {
                game.state_recorder.frames_since_last = 0;
                if (game.state_recorder.replay_cmd as i32) < log_format::INPUT_LIMIT {
                    game.state_recorder.last_inputs = (game.state_recorder.last_inputs as i32
                        ^ (1i32)
                            << (game.state_recorder.replay_cmd as i32 >> log_format::INPUT_SHIFT))
                        as u16;
                } else if (game.state_recorder.replay_cmd as i32) < log_format::PATCH_LIMIT {
                    let mut nb: i32 = log_format::PATCH_LENGTH_BASE
                        + (game.state_recorder.replay_cmd as i32 >> log_format::PATCH_LENGTH_SHIFT
                            & log_format::PATCH_LENGTH_MASK);
                    if nb == log_format::PATCH_LENGTH_ESCAPE {
                        do_until! { t as i32 != log_format::EXTENSION as i32 ;
                            crate::update_place! { previous fresh3 = replay_pos => replay_pos + 1; }
                            let mut t: u8 = game.state_recorder.log.byte(fresh3 as usize);
                            nb += t as i32;
                        }
                    }
                    let mut addr: u32 =
                        ((game.state_recorder.replay_cmd as i32 >> log_format::ADDRESS_BANK_SHIFT
                            & log_format::ADDRESS_BANK_MASK)
                            << log_format::ADDRESS_BANK_BIT) as u32;
                    crate::update_place! { previous fresh4 = replay_pos => replay_pos + 1; }
                    addr |= ((game.state_recorder.log.byte(fresh4 as usize) as i32)
                        << log_format::ADDRESS_HIGH_SHIFT) as u32;
                    crate::update_place! { previous fresh5 = replay_pos => replay_pos + 1; }
                    addr |= game.state_recorder.log.byte(fresh5 as usize) as u32;
                    do_until! { nb == 0 ;
                        crate::update_place! { previous fresh6 = replay_pos => replay_pos + 1; }
                        let byte = game.state_recorder.log.byte(fresh6 as usize);
                        game.note_samus_external_mutation();
                        game.ram().write_byte((addr & log_format::RAM_MASK) as usize, byte);
                        if !game.g_rtl_memory_ptr.is_null() {
                            let offset = (addr & log_format::RAM_MASK) as usize;
                            let byte = game.ram().read_byte(offset);
                            memcpy(game.g_rtl_memory_ptr.add(offset).cast(), (&raw const byte).cast(), 1);
                        }
                        addr = addr.wrapping_add(1);
                        nb -= 1;
                    }
                } else {
                    crate::native_assertion! {
                        '_c2rust_label_0;
                        b"0\0", b"src/sm_rtl.c\0", 344,
                        b"uint16 StateRecorder_ReadNextReplayState(StateRecorder *)\0"
                    };
                }
            }
            game.state_recorder.replay_pos_last_complete = replay_pos as u32;
            if replay_pos as size_t >= game.state_recorder.log.size {
                game.state_recorder.replay_pos = replay_pos as u32;
                game.state_recorder.replay_next_cmd_at = 0xffffffff as u32;
                break;
            } else {
                crate::update_place! { previous fresh7 = replay_pos => replay_pos + 1; }
                let cmd: u8 = game.state_recorder.log.byte(fresh7 as usize);
                let mask: i32 = if (cmd as i32) < log_format::INPUT_LIMIT {
                    log_format::INPUT_DELAY_MASK
                } else {
                    log_format::PATCH_DELAY_MASK
                };
                let mut frames: i32 = cmd as i32 & mask;
                if frames == mask {
                    do_until! { t_0 as i32 != log_format::EXTENSION as i32 ;
                        crate::update_place! { previous fresh8 = replay_pos => replay_pos + 1; }
                        let mut t_0: u8 = game.state_recorder.log.byte(fresh8 as usize);
                        frames += t_0 as i32;
                    }
                }
                game.state_recorder.replay_next_cmd_at = frames as u32;
                game.state_recorder.replay_cmd = cmd;
                game.state_recorder.replay_pos = replay_pos as u32;
            }
        }
        game.state_recorder.frames_since_last =
            game.state_recorder.frames_since_last.wrapping_add(1);
        game.state_recorder.replay_frame_counter =
            game.state_recorder.replay_frame_counter.wrapping_add(1);
        if game.state_recorder.replay_frame_counter >= game.state_recorder.total_frames {
            game.state_recorder.replay_mode = false_0 != 0;
        }
        game.state_recorder.last_inputs
    }
}

crate::forward_calls! { pub unsafe fn (game: &mut Game);
    RtlClearKeyLog => StateRecorder_ClearKeyLog(game);
}
pub fn RtlStopReplay(game: &mut Game) {
    game.state_recorder.stop();
}
pub unsafe fn RtlRunFrame(mut inputs: i32, native_game: &mut Game) -> bool {
    let game = &mut *native_game;
    unsafe {
        if !game.supports_legacy_persistence()
            && u32::from(game.g_runmode) != crate::cpu_infra::RM_MINE
        {
            eprintln!("RtlRunFrame: CPU authority is unavailable for a prepared world");
            return false;
        }
        game.preflight_samus_runtime()
            .expect("Samus frame entry rejected");
        let mut mutation = game
            .begin_samus_mutation()
            .expect("Samus mutation rejected");
        let game = mutation.game();
        // TAS verification preserves every recorded button, including opposites.
        if !game.VERIFY_ACTIVE {
            if inputs & 0x30 == 0x30 {
                inputs ^= 0x30;
            }
            if inputs & 0xc0 == 0xc0 {
                inputs ^= 0xc0;
            }
        }
        let is_replay: bool = game.state_recorder.replay_mode;
        if is_replay {
            inputs = StateRecorder_ReadNextReplayState(game) as i32;
        } else {
            if game.state_recorder.snapshot_flags & 1 != 0 {
                game.state_recorder.snapshot_flags &= !(1i32) as u32;
                inputs = game.state_recorder.last_inputs as i32;
            } else if word!(game.bug_fix_counter) as i32 != kCurrentBugFixCounter as i32 {
                if game.g_debug_flag {
                    printf(
                        c"bug_fix_counter %d => %d\n".as_ptr(),
                        word!(game.bug_fix_counter) as i32,
                        kCurrentBugFixCounter as i32,
                    );
                }
                if (word!(game.bug_fix_counter) as i32) < kCurrentBugFixCounter as i32 {
                    word!(game.bug_fix_counter = kCurrentBugFixCounter as i32 as u16);
                    {
                        let (ram, recorder) = game.ram_and_recorder();
                        recording_allocation(
                            recorder.record_patch(
                                0xc2,
                                &[ram.read_byte(0xc2), ram.read_byte(0xc2 + 1)],
                            ),
                        );
                    }
                }
            }
            recording_allocation(game.state_recorder.record(inputs as u16));
        }
        if word!(game.bug_fix_counter) as i32 != game.currently_installed_bug_fix_counter as i32 {
            RtlUpdateSnesPatchForBugfix(game);
        }
        let admitted_input = if is_replay {
            game.state_recorder.replay_frame_counter
        } else {
            game.state_recorder.total_frames
        };
        game.audio_output.begin_input(u64::from(admitted_input));
        game.g_rtl_runframe.expect("non-null function pointer")(inputs as u16, 0, game);
        game.snes_frame_counter += 1;
        RtlPushApuState(game);
        mutation
            .finish_current()
            .expect("Samus frame adoption rejected");
        crate::cpu_infra::session_native_adoption::reconcile_current(native_game)
            .expect("Session native result adoption rejected");
        native_game.audio_output.complete_input();
        is_replay
    }
}
pub unsafe fn RtlSaveSnapshot(
    filename: *const c_char,
    saving_with_bug: bool,
    native_game: &mut Game,
) {
    let game = &mut *native_game;
    unsafe {
        if !game.supports_legacy_persistence() {
            eprintln!("RtlSaveSnapshot: legacy persistence is unavailable for a prepared world");
            return;
        }
        if let Err(error) = crate::cpu_infra::session_native_adoption::require_legacy_binary(game) {
            eprintln!("RtlSaveSnapshot: {error}");
            return;
        }
        game.preflight_samus_runtime()
            .expect("Samus save entry rejected");
        let mut mutation = game
            .begin_samus_mutation()
            .expect("Samus mutation rejected");
        let game = mutation.game();
        let f = fopen(filename, c"wb".as_ptr()) as *mut FILE;
        RtlApuLock();
        RtlSaveMusicStateToRam_Locked(game, true);
        StateRecorder_Save(f, saving_with_bug, game);
        RtlApuUnlock();
        fclose(f);
        mutation
            .finish_current()
            .expect("Samus save adoption rejected");
        crate::cpu_infra::session_native_adoption::reconcile_current(native_game)
            .expect("Session native result adoption rejected");
    }
}
static kBugSaves: crate::types::ImmutableCStringPointers<7> = crate::types::immutable_c_strings! {
    b"Before Kraid\0", b"Before Golden Torizo\0", b"After Crocomire\0", b"Baby Metroid\0",
    b"Tourian Statue\0", b"Before Ridley\0", b"Enter Mother Brain\0",
};
pub unsafe fn RtlSaveLoad(cmd: i32, slot: i32, native_game: &mut Game) {
    let game = &mut *native_game;
    unsafe {
        if !game.supports_legacy_persistence() {
            eprintln!("RtlSaveLoad: legacy persistence is unavailable for a prepared world");
            return;
        }
        if let Err(error) = crate::cpu_infra::session_native_adoption::require_legacy_binary(game) {
            eprintln!("RtlSaveLoad: {error}");
            return;
        }
        let mut name: [c_char; 128] = [0; 128];
        if slot >= 256 {
            let i: i32 = slot - 256i32;
            if cmd == kSaveLoad_Save as i32
                || i as usize
                    >= (size_of::<[*const c_char; 7]>() as usize).wrapping_div(size_of::<
                        *const c_char,
                    >(
                    )
                        as usize)
            {
                return;
            }
            sprintf(
                &raw mut name as *mut c_char,
                c"saves/%s.sav".as_ptr(),
                kBugSaves[i as usize],
            );
        } else {
            sprintf(
                &raw mut name as *mut c_char,
                c"saves/save%d.sav".as_ptr(),
                slot,
            );
        }
        printf(
            c"*** %s slot %d\n".as_ptr(),
            if cmd == kSaveLoad_Save as i32 {
                c"Saving".as_ptr()
            } else if cmd == kSaveLoad_Load as i32 {
                c"Loading".as_ptr()
            } else {
                c"Replaying".as_ptr()
            },
            slot,
        );
        if cmd != kSaveLoad_Save as i32 {
            let f = fopen(&raw mut name as *mut c_char, c"rb".as_ptr()) as *mut FILE;
            if f.is_null() {
                printf(c"Failed fopen: %s\n".as_ptr(), &raw mut name as *mut c_char);
                return;
            }
            game.preflight_samus_runtime()
                .expect("Samus load entry rejected");
            let mut mutation = game
                .begin_samus_mutation()
                .expect("Samus mutation rejected");
            let game = mutation.game();
            RtlApuLock();
            let complete_load = state_recorder_load(f, cmd == kSaveLoad_Replay as i32, game, true);
            ppu_copy((*game.g_snes).my_ppu, (*game.g_snes).ppu);
            RtlApuUnlock();
            synchronize_whole_state(game, complete_load);
            fclose(f);
            if game.ram().read_byte(0x611)
                | game.ram().read_byte(0x612)
                | game.ram().read_byte(0x613)
                | game.ram().read_byte(0x614)
                | game.ram().read_byte(0x615)
                != 0
            {
                printf(
                    c"Coroutine state: %d, %d, %d, %d, %d\n".as_ptr(),
                    game.ram().read_byte(0x611) as i32,
                    game.ram().read_byte(0x612) as i32,
                    game.ram().read_byte(0x613) as i32,
                    game.ram().read_byte(0x614) as i32,
                    game.ram().read_byte(0x615) as i32,
                );
            }
            if game.ram().read_byte(0x611) as i32 == 4 {
                game.ram()
                    .write_byte(0x611, (10 + word!(game.game_state) as i32) as u8);
            }
            if word!(game.bug_fix_counter) as i32 == 0 {
                word!(game.bug_fix_counter = word!(game.bug_fix_counter_BAD));
            }
            if complete_load {
                mutation
                    .finish_current()
                    .expect("Samus loaded continuation adoption rejected");
                crate::cpu_infra::session_native_adoption::reconcile_current(native_game)
                    .expect("Session native continuation adoption rejected");
            }
        } else {
            RtlSaveSnapshot(&raw mut name as *mut c_char, false_0 != 0, game);
        };
    }
}
pub unsafe fn MemCpy(dst: *mut c_void, src: *const c_void, size: i32) {
    unsafe {
        memcpy(dst, src, size as size_t);
    }
}
pub fn MakePairU16(k: u16, j: u16) -> PairU16 {
    let r: PairU16 = PairU16 { k, j };
    r
}
pub unsafe fn mov24(a: *mut LongPtr, d: u32) {
    unsafe {
        (*a).addr = (d & 0xffff) as VoidP;
        (*a).bank = (d >> 16) as u8;
    }
}
pub unsafe fn Load24(src: *const LongPtr) -> u32 {
    unsafe { *(src as *mut u32) & 0xffffff }
}
pub fn invalid_interrupt(_game: &mut Game) {
    unreachable_failure(true)
}
pub fn Unreachable(_game: &mut Game) -> bool {
    unreachable_failure(false)
}
// Only these fixed static diagnostics cross the native assertion boundary.
// No caller-provided pointers or backend allocations are accessed.
fn unreachable_failure(invalid_interrupt: bool) -> ! {
    unsafe {
        for message in [
            invalid_interrupt.then_some(c"InvalidInterrupt_Crash\n"),
            Some(c"Unreachable!\n"),
        ]
        .into_iter()
        .flatten()
        {
            printf(message.as_ptr());
        }
        crate::native_assertion! {
            '_c2rust_label;
            b"0\0", b"src/sm_rtl.c\0", 512, b"_Bool Unreachable(void)\0"
        }
    }
}
pub use crate::rom::lorom_offset;

/// Capture a checked record while preserving the legacy low-window diagnostic.
pub(crate) unsafe fn rom_record(bank: u8, addr: u16, game: &mut Game) -> crate::rom::RomRecord {
    let views = (game.g_snes, game.g_cpu);
    let (backend, mut host) = game.execution_parts();
    unsafe {
        rom_record_parts(bank, addr, &mut host, |output| {
            observe_backend(backend.as_deref(), views, output)
        })
    }
}
fn observe_backend(
    backend: Option<&crate::snes::snes::Backend>,
    views: (*mut Snes, *mut Cpu),
    output: &mut crate::recording::RecordingBytes,
) {
    let backend = backend.expect("observation requires an owned backend");
    backend.save_observation(views.0, views.1, output);
}
unsafe fn rom_record_parts(
    bank: u8,
    addr: u16,
    host: &mut HostContext<'_>,
    mut _observe: impl FnMut(&mut crate::recording::RecordingBytes),
) -> crate::rom::RomRecord {
    unsafe {
        let address = (u32::from(bank) << 16) | u32::from(addr);
        if address & 0x8000 == 0 {
            printf(c"RomPtr - Invalid access 0x%x!\n".as_ptr(), address);
            *host.g_fail = true;
            #[cfg(sm_verify_dll)]
            crate::sprite_effect_observer::diagnostic_parts(address, host.ram, host.loaded_rom);
            #[cfg(sm_verify_dll)]
            crate::room_reference_observer::diagnostic_parts(
                address,
                host.ram,
                host.loaded_rom,
                host.g_fail,
            );
            #[cfg(sm_verify_dll)]
            crate::room_background_observer::diagnostic_parts(
                address,
                host.ram,
                host.loaded_rom,
                host.g_fail,
                host.decompress_src,
                &mut _observe,
            );
            #[cfg(sm_verify_dll)]
            crate::actor_population_animation_observer::diagnostic_parts(
                address,
                host.ram,
                host.loaded_rom,
                host.g_fail,
                host.eproj_spawn_rect,
                host.eproj_spawn_r22,
                host.eproj_spawn_pt,
                host.eproj_spawn_varE24,
            );
        }
        crate::rom::RomRecord {
            image: host
                .loaded_rom
                .as_ref()
                .expect("ROM must be loaded")
                .clone(),
            base: lorom_offset(address) as usize,
        }
    }
}

pub(crate) unsafe fn rom_bank_record<const BANK: u8>(
    addr: u16,
    game: &mut Game,
) -> crate::rom::RomRecord {
    unsafe { rom_record(BANK, addr, game) }
}

/// Resolve a word address in a fixed ROM bank through the maintained validator.
#[inline]
pub(crate) unsafe fn rom_bank_ptr<const BANK: u8>(addr: u16, game: &mut Game) -> *const u8 {
    unsafe { RomPtr((u32::from(BANK) << 16) | u32::from(addr), game) }
}

/// Resolve an explicit bank and word address, retaining RomPtr diagnostics.
#[inline]
#[forbid(unsafe_code)]
pub(crate) fn report_invalid_rom_address(address: u32, game: &mut Game) {
    use std::io::Write;
    let _ = write!(
        std::io::stdout().lock(),
        "RomPtr - Invalid access 0x{address:x}!{}",
        if cfg!(windows) { "\r\n" } else { "\n" }
    );
    game.g_fail = true;
}

pub unsafe fn RomPtr(addr: u32, game: &mut Game) -> *const u8 {
    unsafe { rom_ptr_parts(addr, &mut game.execution_parts().1) }
}
pub(crate) unsafe fn rom_ptr_parts(addr: u32, host: &mut HostContext<'_>) -> *const u8 {
    unsafe {
        if addr & 0x8000 == 0 {
            printf(c"RomPtr - Invalid access 0x%x!\n".as_ptr(), addr);
            *host.g_fail = true_0 != 0;
            #[cfg(sm_verify_dll)]
            crate::room_contact_start_observer::diagnostic_parts(
                addr,
                host.ram,
                host.loaded_rom,
                host.g_fail,
            );
        }
        host.native_rom
            .as_ref()
            .map_or(core::ptr::null(), |image| {
                crate::native_rom::NativeRomPointer::as_mut_ptr(image).cast_const()
            })
            .offset(crate::rtl::lorom_offset(addr))
    }
}
/// Preserve the setup sequence's explicit RAM mirror and bus-write order.
pub(crate) unsafe fn write_register_and_mirror(
    register: u16,
    value: u8,
    mirror: usize,
    mirror_first: bool,
    game: &mut Game,
) {
    unsafe {
        if mirror_first {
            game.ram().write_byte(mirror, value);
            WriteReg(register, value, game);
        } else {
            WriteReg(register, value, game);
            game.ram().write_byte(mirror, value);
        }
    }
}

pub unsafe fn WriteReg(reg: u16, value: u8, game: &mut Game) {
    let views = (game.g_snes, game.g_cpu);
    unsafe { write_reg_parts(reg, value, &mut game.execution(), views) }
}
/// Borrow prepared ROM bytes through both MDMAEN pre-reads and its complete drain.
/// # Safety
/// Requires WriteReg's live, exclusive backend and quiescent native borrowers.
/// Selected CPU/device links keep their existing retained safety contracts.
pub(crate) unsafe fn write_dma_start(mask: u8, source: &DmaRomSource<'_>, game: &mut Game) {
    let views = (game.g_snes, game.g_cpu);
    unsafe {
        let mut execution = game.execution();
        execution.write_with_dma_source(MDMAEN as u32, mask, Some(source));
        observe_register_write(
            MDMAEN as u16,
            mask,
            &mut execution.host,
            execution.backend,
            views,
        );
    }
}

pub(crate) unsafe fn write_reg_execution(reg: u16, value: u8, game: &mut Execution<'_>) {
    let views = game.backend.views();
    unsafe { write_reg_parts(reg, value, game, views) }
}
// Bank-0 DMA writes have an inert cartridge tail; preserve byte observations.
#[forbid(unsafe_code)]
pub(crate) fn write_dma_register(reg: u16, value: u8, game: &mut Game) {
    let views = (game.g_snes, game.g_cpu);
    let (backend, mut host) = game.execution_parts();
    let backend = backend.expect("execution requires an owned backend");
    assert!(
        (0x4300..=0x437f).contains(&reg),
        "DMA register outside bank-0 window"
    );
    backend.write_dma_register(reg, value);
    observe_register_write(reg, value, &mut host, backend, views);
}
unsafe fn write_reg_parts(
    reg: u16,
    value: u8,
    game: &mut Execution<'_>,
    views: (*mut Snes, *mut Cpu),
) {
    unsafe { game.write(reg as u32, value) };
    observe_register_write(reg, value, &mut game.host, game.backend, views);
}
#[forbid(unsafe_code)]
fn observe_register_write(
    _reg: u16,
    _value: u8,
    _host: &mut HostContext<'_>,
    _backend: &crate::snes::snes::Backend,
    _views: (*mut Snes, *mut Cpu),
) {
    #[cfg(sm_verify_dll)]
    crate::room_background_observer::written_parts(
        _reg,
        _value,
        _host.ram,
        _host.loaded_rom,
        _host.g_fail,
        _host.decompress_src,
        |output| observe_backend(Some(_backend), _views, output),
    );
}
/// Magnitude of a signed SNES word; 0x8000 remains the unsigned value 0x8000.
#[inline]
pub(crate) fn abs16(value: u16) -> u16 {
    (value as i16).unsigned_abs()
}

/// Preserve the input high byte when bit 7 is clear.
pub(crate) fn sign_extend8(value: u16) -> u16 {
    if value & 0x80 != 0 {
        value | 0xff00
    } else {
        value
    }
}

/// Add to a split word pair, reading low then high and writing low then high.
///
/// # Safety
/// Both pointers must allow initialized word reads and word writes. They may
/// be unaligned or overlap; neither pointee may be concurrently accessed.
#[inline]
pub(crate) unsafe fn AddToHiLo(hi: *mut u16, lo: *mut u16, value: u32) {
    unsafe {
        let low = u32::from(lo.read_unaligned());
        let high = u32::from(hi.read_unaligned());
        let sum = (low | (high << 16)).wrapping_add(value);
        lo.write_unaligned(sum as u16);
        hi.write_unaligned((sum >> 16) as u16);
    }
}

/// Set a split word pair, writing low before high even when the words overlap.
///
/// # Safety
/// Both pointers must allow word writes. They may be unaligned or overlap;
/// neither pointee may be concurrently accessed.
#[inline]
pub(crate) unsafe fn SetHiLo(hi: *mut u16, lo: *mut u16, value: u32) {
    unsafe {
        lo.write_unaligned(value as u16);
        hi.write_unaligned((value >> 16) as u16);
    }
}

pub fn Mult8x8(a: u8, b: u8) -> u16 {
    (a as i32 * b as i32) as u16
}
pub fn SnesDivide(a: u16, b: u8) -> u16 {
    (if b as i32 == 0 {
        0xffff
    } else {
        a as i32 / b as i32
    }) as u16
}
pub fn SnesModulus(a: u16, b: u8) -> u16 {
    (if b as i32 == 0 {
        a as i32
    } else {
        a as i32 % b as i32
    }) as u16
}
pub unsafe fn ReadReg(reg: u16, game: &mut Game) -> u8 {
    unsafe { game.execution().read(reg as u32) }
}
pub unsafe fn ReadRegWord(reg: u16, game: &mut Game) -> u16 {
    unsafe {
        let mut rv: u16 = ReadReg(reg, game) as u16;
        rv = (rv as i32 | (ReadReg((reg as i32 + 1) as u16, game) as i32) << 8) as u16;
        rv
    }
}
pub unsafe fn WriteRegWord(reg: u16, value: u16, game: &mut Game) {
    unsafe {
        WriteReg(reg, value as u8, game);
        WriteReg((reg as i32 + 1) as u16, (value >> 8) as u8, game);
    }
}

pub unsafe fn RtlApuWrite(adr: u32, val: u8, game: &mut Game) {
    unsafe {
        let (backend, mut host) = game.execution_parts();
        if apu_write_queued(adr, val, &mut host, backend.as_deref()) {
            let mut execution = Execution {
                backend: backend.expect("APU upload requires an owned backend"),
                host,
            };
            apu_write_upload(adr, val, &mut execution);
        }
    }
}
pub(crate) unsafe fn apu_write(game: &mut Execution<'_>, adr: u32, val: u8) {
    unsafe {
        if apu_write_queued(adr, val, &mut game.host, Some(&*game.backend)) {
            apu_write_upload(adr, val, game);
        }
    }
}
pub(crate) unsafe fn audio_execution_position(
    backend: Option<&crate::snes::snes::Backend>,
    native_frame: i32,
) -> crate::audio_output::ExecutionPosition {
    unsafe {
        let (branch, pc) = backend.map_or((0, None), |backend| {
            let (snes, cpu) = backend.views();
            // Only dereference the owned CPU when it is the actual selected link.
            // Foreign/unavailable selection is a provenance gap, not a guessed PC.
            // SAFETY: both views are the Backend's own live allocations. During an
            // APU bus write the interpreter holds the CPU only as `*mut Cpu` (no
            // reference spans the bus callback), so these single-threaded field
            // reads alias nothing; mid-instruction, `pc` is the post-fetch value.
            let pc =
                ((*snes).cpu == cpu).then(|| (u32::from((*cpu).k) << 16) | u32::from((*cpu).pc));
            ((*snes).runningWhichVersion, pc)
        });
        crate::audio_output::ExecutionPosition {
            branch,
            pc,
            native_frame,
        }
    }
}
// Validate and sample the upload flag before either queue or physical effects.
unsafe fn apu_write_queued(
    adr: u32,
    val: u8,
    host: &mut HostContext<'_>,
    backend: Option<&crate::snes::snes::Backend>,
) -> bool {
    unsafe {
        crate::native_assertion! {
            '_c2rust_label: adr >= APUI00 as i32 as u32 && adr <= APUI03 as i32 as u32;
            b"adr >= APUI00 && adr <= APUI03\0", b"src/sm_rtl.c\0", 571,
            b"void RtlApuWrite(uint32, uint8)\0"
        };
        if host.ram.read_byte(0x617) != 0 || host.ram.read_byte(0x618) != 0 {
            return true;
        }
        let position = audio_execution_position(backend, *host.snes_frame_counter);
        let request = host.audio_output.accept_port(
            position,
            crate::audio_output::PortTarget::Queue,
            (adr & 3) as u8,
            val,
        );
        host.apu_queue
            .write(usize::from(request.port), request.value);
        host.audio_output.complete_port(request);
        false
    }
}
unsafe fn apu_write_upload(adr: u32, val: u8, game: &mut Execution<'_>) {
    unsafe {
        game.catch_up_apu();
        let position =
            audio_execution_position(Some(&*game.backend), *game.host.snes_frame_counter);
        let request = game.host.audio_output.accept_port(
            position,
            crate::audio_output::PortTarget::Physical,
            (adr & 3) as u8,
            val,
        );
        (*game.backend.apu_view()).registers.inPorts[usize::from(request.port)] = request.value;
        game.host.audio_output.complete_port(request);
    }
}
pub unsafe fn RtlPushApuState(game: &mut Game) {
    unsafe {
        RtlApuLock();
        game.apu_queue.push(game.uploading_apu());
        RtlApuUnlock();
    }
}
unsafe fn RtlPopApuState_Locked(game: &mut Game) {
    unsafe {
        if game.uploading_apu() {
            return;
        }
        if let Some(writes) = game.apu_queue.pop() {
            if game.g_use_my_apu_code {
                game.apply_native_audio_ports(writes);
            } else {
                crate::apu_queue::ApuQueue::apply_ports(
                    writes,
                    &mut (*(*game.g_snes).apu).registers.inPorts,
                );
            }
        }
    }
}
pub unsafe fn RtlApuUpload(address: u32, game: &mut Game) {
    unsafe {
        if address & 0x8000 == 0 {
            printf(c"RomPtr - Invalid access 0x%x!\n".as_ptr(), address);
            game.g_fail = true;
        }
        let source = lorom_offset(address) as usize;
        let image = game
            .loaded_rom
            .as_ref()
            .expect("ROM must be loaded")
            .clone();
        let version = if game.g_snes.is_null() {
            0
        } else {
            (*game.g_snes).runningWhichVersion
        };
        if version == 1 || version == 2 {
            let startup = source == lorom_offset(0xcf8000) as usize;
            let snapshot = if version == 1 {
                &mut game.g_snapshot_theirs
            } else {
                &mut game.g_snapshot_mine
            };
            snapshot.apu_upload_count = snapshot.apu_upload_count.saturating_add(1);
            snapshot.apu_startup_upload = startup;
        }
        RtlApuLock();
        game.apu_queue.reset();
        SpcPlayer_Upload(game.g_spc_player, image, source);
        RtlApuUnlock();
    }
}
unsafe fn RtlRestoreMusicAfterLoad_Locked(is_reset: bool, game: &mut Game) {
    unsafe {
        if game.g_use_my_apu_code {
            (*game.g_spc_player)
                .ram
                .copy_from_ram(&(*(*game.g_snes).apu).ram);
            (*(*game.g_spc_player).state.dsp).copy_state_from(&*(*(*game.g_snes).apu).dsp);
            game.load_native_audio_variables();
        }
        if is_reset {
            game.initialize_native_player();
        }
        game.apu_queue.reset();
    }
}
unsafe fn RtlSaveMusicStateToRam_Locked(game: &mut Game, include_queued_commands: bool) {
    unsafe {
        if game.g_use_my_apu_code {
            game.save_native_audio_variables(include_queued_commands);
            (*(*(*game.g_snes).apu).dsp).copy_state_from(&*(*game.g_spc_player).state.dsp);
            (*(*game.g_snes).apu)
                .ram
                .copy_from_ram(&(*game.g_spc_player).ram);
        }
    }
}
pub unsafe fn RtlRenderAudio(audio_buffer: &mut [[i16; 2]], game: &mut Game) -> bool {
    unsafe {
        RtlApuLock();
        RtlPopApuState_Locked(game);
        let mut produced = false;
        if !game.g_use_my_apu_code {
            if !game.uploading_apu() {
                while ((*(*(*game.g_snes).apu).dsp).state.sampleOffset as i32) < 534 {
                    game.execution().cycle_apu();
                }
                (*(*(*game.g_snes).apu).dsp)
                    .state
                    .write_samples(audio_buffer);
                produced = true;
            }
        } else {
            game.render_native_audio(audio_buffer);
            produced = true;
        }
        RtlApuUnlock();
        produced
    }
}
pub unsafe fn RtlCheat(c: c_char, game: &mut Game) {
    unsafe {
        if c as i32 == 'w' as i32 {
            word!(game.samus_health = word!(game.samus_max_health));
            {
                let (ram, recorder) = game.ram_and_recorder();
                recording_allocation(
                    recorder.record_patch(0x9c2, &[ram.read_byte(0x9c2), ram.read_byte(0x9c2 + 1)]),
                );
            }
        } else if c as i32 == 'q' as i32 {
            word!(checked_i32, game.samus_y_pos -= 4i32);
            word!(game.samus_y_speed = -8i32 as u16);
            {
                let (ram, recorder) = game.ram_and_recorder();
                recording_allocation(
                    recorder.record_patch(0xafa, &[ram.read_byte(0xafa), ram.read_byte(0xafa + 1)]),
                );
            }
            {
                let (ram, recorder) = game.ram_and_recorder();
                recording_allocation(
                    recorder.record_patch(0xb2e, &[ram.read_byte(0xb2e), ram.read_byte(0xb2e + 1)]),
                );
            }
            word!(game.menu_index = 0);
        } else if c as i32 == 'q' as i32 {
            word!(game.japanese_text_flag = (word!(game.japanese_text_flag) == 0) as i32 as u16);
            {
                let (ram, recorder) = game.ram_and_recorder();
                recording_allocation(
                    recorder.record_patch(0x9e2, &[ram.read_byte(0x9e2), ram.read_byte(0x9e2 + 1)]),
                );
            }
        }
    }
}
pub unsafe fn RtlReadSram(game: &mut Game) {
    unsafe {
        if !game.supports_legacy_persistence() {
            eprintln!("RtlReadSram: legacy persistence is unavailable for a prepared world");
            return;
        }
        let f = fopen(c"saves/sm.srm".as_ptr(), c"rb".as_ptr()) as *mut FILE;
        if !f.is_null() {
            game.preflight_samus_runtime()
                .expect("Samus SRAM entry rejected");
            let mut mutation = game
                .begin_samus_mutation()
                .expect("Samus mutation rejected");
            let game = mutation.game();
            let mut bytes = game.g_sram.bytes(0..crate::recording::BASE_SRAM_SIZE);
            let count = fread(
                bytes.as_mut_ptr().cast(),
                1,
                crate::recording::BASE_SRAM_SIZE,
                f,
            );
            game.g_sram.write_bytes(0, &bytes);
            if count != crate::recording::BASE_SRAM_SIZE as u32 {
                fprintf(stderr, c"Error reading saves/sm.srm\n".as_ptr());
            }
            fclose(f);
            synchronize_whole_state(game, count == crate::recording::BASE_SRAM_SIZE as u32);
            recording_allocation(
                game.state_recorder
                    .base_snapshot
                    .resize(crate::recording::BASE_SRAM_SIZE),
            );
            game.state_recorder.has_base_audio_queue = false;
            game.g_sram
                .read_bytes(0, game.state_recorder.base_snapshot.as_mut_slice());
            if count == crate::recording::BASE_SRAM_SIZE as u32 {
                mutation
                    .finish_current()
                    .expect("Samus SRAM adoption rejected");
            }
        }
    }
}
pub unsafe fn RtlWriteSram(game: &mut Game) {
    unsafe {
        if game.VERIFY_ACTIVE {
            return;
        }
        if !game.supports_legacy_persistence() {
            eprintln!("RtlWriteSram: legacy persistence is unavailable for a prepared world");
            return;
        }
        rename(c"saves/sm.srm".as_ptr(), c"saves/sm.srm.bak".as_ptr());
        let f = fopen(c"saves/sm.srm".as_ptr(), c"wb".as_ptr()) as *mut FILE;
        if !f.is_null() {
            let bytes = game.g_sram.bytes(0..crate::recording::BASE_SRAM_SIZE);
            fwrite(
                bytes.as_ptr().cast(),
                1,
                crate::recording::BASE_SRAM_SIZE,
                f,
            );
            fclose(f);
        } else {
            fprintf(stderr, c"Unable to write saves/sm.srm\n".as_ptr());
        };
    }
}

/// Store one SNES word without imposing host alignment.
///
/// # Safety
/// `destination` must be valid for writing two bytes. It may be unaligned.
#[inline]
pub(crate) unsafe fn write_word(value: u16, destination: *mut u16) {
    unsafe {
        destination.write_unaligned(value.to_le());
    }
}
