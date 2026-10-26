use crate::snapshot_io::{Block, SnapshotIo, TransferError};
// Split-word assembly evaluates high then low. The i32 form retains signed
// casts, including sign extension of the low operand. Repeated receivers and
// operands are evaluated separately. Set/add keep pointer-before-value order
// and the helpers' low-before-high writes, including unaligned/overlapping words.
macro_rules! words {
    ($game:ident.$hi:ident, $lo:ident += $value:expr $(,)?) => {
        $crate::rtl::AddToHiLo(
            $crate::ram_map::ram_ptr!($game, u16; $crate::ram_map::words::$hi as isize),
            $crate::ram_map::ram_ptr!($game, u16; $crate::ram_map::words::$lo as isize),
            $value,
        )
    };
    ($game:ident.$hi:ident, $lo:ident = $value:expr $(,)?) => {
        $crate::rtl::SetHiLo(
            $crate::ram_map::ram_ptr!($game, u16; $crate::ram_map::words::$hi as isize),
            $crate::ram_map::ram_ptr!($game, u16; $crate::ram_map::words::$lo as isize),
            $value,
        )
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
    __assert_fail, fclose, fopen, fprintf, fread, fseek, fwrite, memcpy, printf, rename, sprintf,
    stderr,
};
use crate::cpu_infra::RtlUpdateSnesPatchForBugfix;
use crate::game_state::Game;
use crate::host::{Die, RtlApuLock, RtlApuUnlock};
use crate::initial_state::InitialState;
use crate::ram_map::ram_ptr;
use crate::ram_map::word;
use crate::recording::RecordingBytes;
pub use crate::recording::StateRecorder;
use crate::snes::apu::apu_cycle;
use crate::snes::ppu::ppu_copy;
pub use crate::snes::registers::*;
use crate::snes::snes::{snes_catchupApu, snes_read, snes_reset, snes_saveload, snes_write};
use crate::spc_player::{
    SpcPlayer_CopyVariablesFromRam, SpcPlayer_CopyVariablesToRam, SpcPlayer_GenerateSamples,
    SpcPlayer_Initialize, SpcPlayer_Upload,
};
pub use crate::types::Dsp;
pub use crate::types::{FILE, LongPtr, PairU16, VoidP, size_t};
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
        if game.g_rtl_syncall.is_some() {
            game.g_rtl_syncall.expect("non-null function pointer")(game);
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
        crate::snes::ppu::PpuRestoreRenderRouting(&mut *(*game.g_snes).ppu);
        (*(*game.g_snes).cpu).e = false_0 != 0;
        let next: u32 = ((*(*game.g_snes)
            .ram
            .offset(((*(*game.g_snes).cpu).sp as i32 + 3) as isize)
            as i32
            | (*(*game.g_snes)
                .ram
                .offset(((*(*game.g_snes).cpu).sp as i32 + 4) as isize) as i32)
                << 8
            | (*(*game.g_snes)
                .ram
                .offset(((*(*game.g_snes).cpu).sp as i32 + 5) as isize) as i32)
                << 16)
            + 1) as u32;
        if next == 0x82897e {
            *(*game.g_snes)
                .ram
                .offset(((*(*game.g_snes).cpu).sp as i32 + 3) as isize) =
                (0xf71b - 1i32 & 0xff) as u8;
            *(*game.g_snes)
                .ram
                .offset(((*(*game.g_snes).cpu).sp as i32 + 4) as isize) =
                (0xf71b - 1i32 >> 8) as u8;
        }
        RtlSynchronizeWholeState(game);
    }
}

pub unsafe fn ReadFromFile(f: *mut FILE, data: *mut c_void, n: size_t) {
    unsafe {
        if fread(data, 1 as size_t, n, f) as size_t != n {
            Die(c"fread failed\n".as_ptr());
        }
    }
}
pub unsafe fn RtlReset(mode: i32, game: &mut Game) {
    unsafe {
        let actor_transition = game.prepare_actor_canonical();
        game.snes_frame_counter = 0;
        snes_reset(game.g_snes, true_0 != 0, game);
        if mode & 1 == 0 {
            game.g_sram.fill(0..0x2000, 0);
        }
        *ram_ptr!(game; 0x611) = 1;
        RtlApuLock();
        RtlRestoreMusicAfterLoad_Locked(true_0 != 0, game);
        RtlApuUnlock();
        RtlSynchronizeWholeState(game);
        game.commit_actor_canonical(actor_transition);
        if mode & 2 == 0 {
            game.state_recorder = StateRecorder::default();
        }
    }
}
pub unsafe fn StateRecorder_Load(f: *mut FILE, replay_mode: bool, game: &mut Game) {
    unsafe {
        if !game.supports_legacy_persistence() {
            eprintln!("StateRecorder_Load: legacy persistence is unavailable for a prepared world");
            return;
        }
        let mut hdr: [u32; 16] = [0i32 as u32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let is_old: bool = false_0 != 0;
        let mut is_reset: bool = false_0 != 0;
        ReadFromFile(
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
            ReadFromFile(
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
        game.state_recorder.total_frames = hdr[1i32 as usize];
        recording_allocation(game.state_recorder.log.resize(hdr[2] as usize));
        ReadFromFile(
            f,
            game.state_recorder.log.as_mut_ptr() as *mut c_void,
            game.state_recorder.log.size,
        );
        game.state_recorder.last_inputs = hdr[3i32 as usize] as u16;
        game.state_recorder.frames_since_last = hdr[4i32 as usize];
        recording_allocation(game.state_recorder.base_snapshot.resize(hdr[5] as usize));
        ReadFromFile(
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
            if game.state_recorder.base_snapshot.size > 8192 as size_t {
                let mut state = SnapshotIo::Load(game.state_recorder.base_snapshot.as_slice());
                snes_saveload(game.g_snes, &mut state);
                let complete = state.remaining() == 0;
                FinishLoadSnesState(game);
                crate::native_assertion! {
                    '_c2rust_label_0: complete;
                    b"state.p == state.pend\0", b"src/sm_rtl.c\0", 219,
                    b"void StateRecorder_Load(StateRecorder *, FILE *, _Bool)\0"
                };
            } else {
                drop(actor_transition.take());
                RtlReset(2, game);
                if game.state_recorder.base_snapshot.size == 8192 as size_t {
                    game.g_sram
                        .write_bytes(0, &game.state_recorder.base_snapshot.as_slice()[..8192]);
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
            ReadFromFile(f, arr.as_mut_ptr() as *mut c_void, arr.size);
            let mut state = SnapshotIo::Load(arr.as_slice());
            snes_saveload(game.g_snes, &mut state);
            let complete = state.remaining() == 0;
            FinishLoadSnesState(game);
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
            ReadFromFile(f, bytes.as_mut_ptr().cast(), bytes.len());
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
            ReadFromFile(
                f,
                game.state_recorder.base_audio_queue.as_mut_ptr().cast(),
                71,
            );
            game.state_recorder.has_base_audio_queue =
                game.state_recorder.base_snapshot.size > 8192;
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
    }
}
pub unsafe fn StateRecorder_Save(f: *mut FILE, saving_with_bug: bool, game: &mut Game) {
    unsafe {
        if !game.supports_legacy_persistence() {
            eprintln!("StateRecorder_Save: legacy persistence is unavailable for a prepared world");
            return;
        }
        let mut hdr: [u32; 16] = [0i32 as u32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut arr = RecordingBytes::default();
        snes_saveload(game.g_snes, &mut SnapshotIo::Save(&mut arr));
        let sr = &game.state_recorder;
        crate::native_assertion! {
            '_c2rust_label: sr.base_snapshot.size == 0 as size_t
                || sr.base_snapshot.size == arr.size
                || sr.base_snapshot.size == 8192 as size_t;
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
    }
}
pub unsafe fn StateRecorder_ClearKeyLog(game: &mut Game) {
    unsafe {
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
            snapshots.1.capture(game.g_spc_player);
            // Replay restores the queue separately, so do not deliver it early.
            RtlSaveMusicStateToRam_Locked(game, false);
            Some(snapshots)
        } else {
            None
        };
        snes_saveload(
            game.g_snes,
            &mut SnapshotIo::Save(&mut game.state_recorder.base_snapshot),
        );
        if let Some(snapshots) = live_audio {
            snapshots.0.restore(game.g_snes);
            snapshots.1.restore(game.g_spc_player);
        }
        recording_allocation(game.state_recorder.rebase());
    }
}
pub unsafe fn StateRecorder_ReadNextReplayState(game: &mut Game) -> u16 {
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
                if (game.state_recorder.replay_cmd as i32) < 0xc0 {
                    game.state_recorder.last_inputs = (game.state_recorder.last_inputs as i32
                        ^ (1i32) << (game.state_recorder.replay_cmd as i32 >> 4))
                        as u16;
                } else if (game.state_recorder.replay_cmd as i32) < 0xd0 {
                    let mut nb: i32 = 1 + (game.state_recorder.replay_cmd as i32 >> 2 & 3);
                    if nb == 4 {
                        do_until! { t as i32 != 255 ;
                            crate::update_place! { previous fresh3 = replay_pos => replay_pos + 1; }
                            let mut t: u8 = game.state_recorder.log.byte(fresh3 as usize);
                            nb += t as i32;
                        }
                    }
                    let mut addr: u32 =
                        ((game.state_recorder.replay_cmd as i32 >> 1 & 1) << 16) as u32;
                    crate::update_place! { previous fresh4 = replay_pos => replay_pos + 1; }
                    addr |= ((game.state_recorder.log.byte(fresh4 as usize) as i32) << 8) as u32;
                    crate::update_place! { previous fresh5 = replay_pos => replay_pos + 1; }
                    addr |= game.state_recorder.log.byte(fresh5 as usize) as u32;
                    do_until! { nb == 0 ;
                        crate::update_place! { previous fresh6 = replay_pos => replay_pos + 1; }
                        game.g_ram[(addr & 0x1ffff) as usize] = game.state_recorder.log.byte(fresh6 as usize);
                        if !game.g_rtl_memory_ptr.is_null() {
                            let offset = (addr & 0x1ffff) as usize;
                            memcpy(game.g_rtl_memory_ptr.add(offset).cast(), (&raw const game.g_ram[offset]).cast(), 1);
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
                let mask: i32 = if (cmd as i32) < 0xc0 { 0xf } else { 0x1 };
                let mut frames: i32 = cmd as i32 & mask;
                if frames == mask {
                    do_until! { t_0 as i32 != 255 ;
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
pub unsafe fn RtlRunFrame(mut inputs: i32, game: &mut Game) -> bool {
    unsafe {
        if !game.supports_legacy_persistence()
            && u32::from(game.g_runmode) != crate::cpu_infra::RM_MINE
        {
            eprintln!("RtlRunFrame: CPU authority is unavailable for a prepared world");
            return false;
        }
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
                    recording_allocation(
                        game.state_recorder
                            .record_patch(0xc2, &[game.g_ram[0xc2], game.g_ram[0xc2 + 1]]),
                    );
                }
            }
            recording_allocation(game.state_recorder.record(inputs as u16));
        }
        if word!(game.bug_fix_counter) as i32 != game.currently_installed_bug_fix_counter as i32 {
            RtlUpdateSnesPatchForBugfix(game);
        }
        game.g_rtl_runframe.expect("non-null function pointer")(inputs as u16, 0, game);
        game.snes_frame_counter += 1;
        RtlPushApuState(game);
        is_replay
    }
}
pub unsafe fn RtlSaveSnapshot(filename: *const c_char, saving_with_bug: bool, game: &mut Game) {
    unsafe {
        if !game.supports_legacy_persistence() {
            eprintln!("RtlSaveSnapshot: legacy persistence is unavailable for a prepared world");
            return;
        }
        let f = fopen(filename, c"wb".as_ptr()) as *mut FILE;
        RtlApuLock();
        RtlSaveMusicStateToRam_Locked(game, true);
        StateRecorder_Save(f, saving_with_bug, game);
        RtlApuUnlock();
        fclose(f);
    }
}
static kBugSaves: crate::types::ImmutableCStringPointers<7> = crate::types::immutable_c_strings! {
    b"Before Kraid\0", b"Before Golden Torizo\0", b"After Crocomire\0", b"Baby Metroid\0",
    b"Tourian Statue\0", b"Before Ridley\0", b"Enter Mother Brain\0",
};
pub unsafe fn RtlSaveLoad(cmd: i32, slot: i32, game: &mut Game) {
    unsafe {
        if !game.supports_legacy_persistence() {
            eprintln!("RtlSaveLoad: legacy persistence is unavailable for a prepared world");
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
            RtlApuLock();
            StateRecorder_Load(f, cmd == kSaveLoad_Replay as i32, game);
            ppu_copy((*game.g_snes).my_ppu, (*game.g_snes).ppu);
            RtlApuUnlock();
            RtlSynchronizeWholeState(game);
            fclose(f);
            if *ram_ptr!(game; 0x611)
                | *ram_ptr!(game; 0x612)
                | *ram_ptr!(game; 0x613)
                | *ram_ptr!(game; 0x614)
                | *ram_ptr!(game; 0x615)
                != 0
            {
                printf(
                    c"Coroutine state: %d, %d, %d, %d, %d\n".as_ptr(),
                    *ram_ptr!(game; 0x611) as i32,
                    *ram_ptr!(game; 0x612) as i32,
                    *ram_ptr!(game; 0x613) as i32,
                    *ram_ptr!(game; 0x614) as i32,
                    *ram_ptr!(game; 0x615) as i32,
                );
            }
            if *ram_ptr!(game; 0x611) as i32 == 4 {
                *ram_ptr!(game; 0x611) = (10 + word!(game.game_state) as i32) as u8;
            }
            if word!(game.bug_fix_counter) as i32 == 0 {
                word!(game.bug_fix_counter = word!(game.bug_fix_counter_BAD));
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
pub fn invalid_interrupt(game: &mut Game) {
    unreachable_failure(true, game);
    loop {}
}
pub unsafe fn Unreachable(game: &mut Game) -> bool {
    unreachable_failure(false, game)
}
// Only these fixed static diagnostics cross the native assertion boundary.
// No caller-provided pointers or backend allocations are accessed.
fn unreachable_failure(invalid_interrupt: bool, game: &mut Game) -> bool {
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
        };
        game.g_ram[0x1ffffi32 as usize] = 1;
        false_0 != 0
    }
}
pub use crate::rom::lorom_offset;

/// Capture a checked record while preserving the legacy low-window diagnostic.
pub(crate) unsafe fn rom_record(bank: u8, addr: u16, game: &mut Game) -> crate::rom::RomRecord {
    unsafe {
        let address = (u32::from(bank) << 16) | u32::from(addr);
        if address & 0x8000 == 0 {
            printf(c"RomPtr - Invalid access 0x%x!\n".as_ptr(), address);
            game.g_fail = true;
            #[cfg(sm_verify_dll)]
            crate::sprite_effect_observer::diagnostic(address, game);
            #[cfg(sm_verify_dll)]
            crate::room_reference_observer::diagnostic(address, game);
            #[cfg(sm_verify_dll)]
            crate::room_background_observer::diagnostic(address, game);
            #[cfg(sm_verify_dll)]
            crate::actor_population_animation_observer::diagnostic(address, game);
        }
        crate::rom::RomRecord {
            image: game
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
pub(crate) unsafe fn rom_ptr_with_bank(bank: u8, addr: u16, game: &mut Game) -> *const u8 {
    unsafe { RomPtr((u32::from(bank) << 16) | u32::from(addr), game) }
}

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
    unsafe {
        if addr & 0x8000 == 0 {
            printf(c"RomPtr - Invalid access 0x%x!\n".as_ptr(), addr);
            game.g_fail = true_0 != 0;
            #[cfg(sm_verify_dll)]
            crate::room_contact_start_observer::diagnostic(addr, game);
        }
        game.legacy_native_rom_ptr()
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
            game.g_ram[mirror] = value;
            WriteReg(register, value, game);
        } else {
            WriteReg(register, value, game);
            game.g_ram[mirror] = value;
        }
    }
}

pub unsafe fn WriteReg(reg: u16, value: u8, game: &mut Game) {
    unsafe {
        snes_write(game.g_snes, reg as u32, value, game);
        #[cfg(sm_verify_dll)]
        crate::room_background_observer::written(reg, value, game);
    }
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
    unsafe { snes_read(game.g_snes, reg as u32, game) }
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
        crate::native_assertion! {
            '_c2rust_label: adr >= APUI00 as i32 as u32 && adr <= APUI03 as i32 as u32;
            b"adr >= APUI00 && adr <= APUI03\0", b"src/sm_rtl.c\0", 571,
            b"void RtlApuWrite(uint32, uint8)\0"
        };
        if game.uploading_apu() {
            snes_catchupApu(game.g_snes, game);
            (*(*game.g_snes).apu).registers.inPorts[(adr & 0x3) as usize] = val;
            return;
        }
        game.apu_queue.write((adr & 3) as usize, val);
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
            let ports: &mut [u8] = if game.g_use_my_apu_code {
                &mut (*game.g_spc_player).state.input_ports
            } else {
                &mut (*(*game.g_snes).apu).registers.inPorts
            };
            crate::apu_queue::ApuQueue::apply_ports(writes, ports);
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
            SpcPlayer_CopyVariablesFromRam(game.g_spc_player);
        }
        if is_reset {
            SpcPlayer_Initialize(game.g_spc_player);
        }
        game.apu_queue.reset();
    }
}
unsafe fn RtlSaveMusicStateToRam_Locked(game: &mut Game, include_queued_commands: bool) {
    unsafe {
        if game.g_use_my_apu_code {
            if include_queued_commands {
                game.apu_queue
                    .apply_queued(&mut (*game.g_spc_player).state.input_ports);
            }
            SpcPlayer_CopyVariablesToRam(game.g_spc_player);
            (*(*(*game.g_snes).apu).dsp).copy_state_from(&*(*game.g_spc_player).state.dsp);
            (*(*game.g_snes).apu)
                .ram
                .copy_from_ram(&(*game.g_spc_player).ram);
        }
    }
}
pub unsafe fn RtlRenderAudio(audio_buffer: &mut [[i16; 2]], game: &mut Game) {
    unsafe {
        RtlApuLock();
        RtlPopApuState_Locked(game);
        if !game.g_use_my_apu_code {
            if !game.uploading_apu() {
                while ((*(*(*game.g_snes).apu).dsp).state.sampleOffset as i32) < 534 {
                    apu_cycle((*game.g_snes).apu, game);
                }
                (*(*(*game.g_snes).apu).dsp)
                    .state
                    .write_samples(audio_buffer);
            }
        } else {
            SpcPlayer_GenerateSamples(game.g_spc_player);
            (*(*game.g_spc_player).state.dsp)
                .state
                .write_samples(audio_buffer);
        }
        RtlApuUnlock();
    }
}
pub unsafe fn RtlCheat(c: c_char, game: &mut Game) {
    unsafe {
        if c as i32 == 'w' as i32 {
            word!(game.samus_health = word!(game.samus_max_health));
            recording_allocation(
                game.state_recorder
                    .record_patch(0x9c2, &[game.g_ram[0x9c2], game.g_ram[0x9c2 + 1]]),
            );
        } else if c as i32 == 'q' as i32 {
            word!(checked_i32, game.samus_y_pos -= 4i32);
            word!(game.samus_y_speed = -8i32 as u16);
            recording_allocation(
                game.state_recorder
                    .record_patch(0xafa, &[game.g_ram[0xafa], game.g_ram[0xafa + 1]]),
            );
            recording_allocation(
                game.state_recorder
                    .record_patch(0xb2e, &[game.g_ram[0xb2e], game.g_ram[0xb2e + 1]]),
            );
            word!(game.menu_index = 0);
        } else if c as i32 == 'q' as i32 {
            word!(game.japanese_text_flag = (word!(game.japanese_text_flag) == 0) as i32 as u16);
            recording_allocation(
                game.state_recorder
                    .record_patch(0x9e2, &[game.g_ram[0x9e2], game.g_ram[0x9e2 + 1]]),
            );
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
            let mut bytes = game.g_sram.bytes(0..8192);
            let count = fread(bytes.as_mut_ptr().cast(), 1, 8192, f);
            game.g_sram.write_bytes(0, &bytes);
            if count != 8192 {
                fprintf(stderr, c"Error reading saves/sm.srm\n".as_ptr());
            }
            fclose(f);
            RtlSynchronizeWholeState(game);
            recording_allocation(game.state_recorder.base_snapshot.resize(8192));
            game.state_recorder.has_base_audio_queue = false;
            game.g_sram
                .read_bytes(0, game.state_recorder.base_snapshot.as_mut_slice());
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
            let bytes = game.g_sram.bytes(0..8192);
            fwrite(bytes.as_ptr().cast(), 1, 8192, f);
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
