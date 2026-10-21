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
        $game.g_rom.offset($crate::rtl::lorom_offset($address))
    };
    ($game:ident, $target:ty; $address:literal) => {
        $game.g_rom.offset($crate::rtl::lorom_offset($address)) as *mut $target
    };
}
pub(crate) use rom_ptr;

/// Read a word from a fixed LoROM table, retaining pointer-before-index order.
macro_rules! rom_table_word {
    ($game:ident, $address:literal, $index:expr $(,)?) => {
        $crate::rtl::read_rom_table_word(
            $game
                .g_rom
                .offset($crate::rtl::lorom_offset($address))
                .cast::<u16>(),
            $index,
        )
    };
}
pub(crate) use rom_table_word;

use crate::ram_map::ram_ptr;
use ::core::ffi::{c_char, c_long, c_ulong, c_void};
use ::core::mem::size_of;
use ::core::ptr::null_mut;
#[inline]
pub(crate) unsafe fn read_rom_table_word(table: *const u16, index: isize) -> u16 {
    table.offset(index).read_unaligned()
}

use crate::c_library::{
    __assert_fail, fclose, fopen, fprintf, fread, fseek, ftell, fwrite, memcpy, memset, printf,
    rename, sprintf, stderr,
};
use crate::cpu_infra::RtlUpdateSnesPatchForBugfix;
use crate::game_state::Game;
use crate::host::{Die, RtlApuLock, RtlApuUnlock};
use crate::ram_map::word;
use crate::snes::apu::apu_cycle;
use crate::snes::dsp::dsp_getSamples;
use crate::snes::ppu::ppu_copy;
pub use crate::snes::registers::*;
use crate::snes::snes::{snes_catchupApu, snes_read, snes_reset, snes_saveload, snes_write};
use crate::spc_player::{
    SpcPlayer_CopyVariablesFromRam, SpcPlayer_CopyVariablesToRam, SpcPlayer_GenerateSamples,
    SpcPlayer_Initialize, SpcPlayer_Upload,
};
pub use crate::types::{_IO_codecvt, _IO_marker, _IO_wide_data};
pub use crate::types::{
    _IO_lock_t, __off64_t, __off_t, size_t, LongPtr, PairU16, VoidP, _IO_FILE, FILE,
};
use crate::util::{
    ByteArray_AppendByte, ByteArray_AppendData, ByteArray_Destroy, ByteArray_Resize,
};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ApuWriteEnt {
    pub ports: [u8; 4],
}
pub use crate::types::{
    Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel,
    DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, Snes, Spc, Timer,
    WindowLayer,
};
pub use crate::types::{RunFrameFunc, SyncAllFunc};
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct StateRecorder {
        pub last_inputs: u16;
        pub frames_since_last, total_frames, replay_pos, replay_pos_last_complete,
            replay_frame_counter, replay_next_cmd_at, snapshot_flags: u32;
        pub replay_cmd: u8;
        pub replay_mode: bool;
        pub log, base_snapshot: ByteArray;
        pub has_base_audio_queue: bool;
        pub base_audio_queue: [u8; 71];
    }
}
pub use crate::types::{
    ByteArray, Channel, SaveLoadFunc, SaveLoadOperation as C2RustUnnamed_0, SfxChan, SfxRegs,
    SpcPlayer,
};
pub const kSaveLoad_Replay: C2RustUnnamed_0 = 3;
pub const kSaveLoad_Load: C2RustUnnamed_0 = 2;
pub const kSaveLoad_Save: C2RustUnnamed_0 = 1;
pub const kApuMaxQueueSize: C2RustUnnamed_2 = 16;
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct LoadFuncState {
        pub p, pend: *mut u8;
    }
}
pub const kCurrentBugFixCounter: C2RustUnnamed_1 = 1;
pub use crate::types::SnesRegs;
pub type C2RustUnnamed_1 = u32;
pub type C2RustUnnamed_2 = u32;
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
    if game.g_rtl_syncall.is_some() {
        game.g_rtl_syncall.expect("non-null function pointer")(game);
    }
}
unsafe fn RtlSyncMemoryRegion(ptr: *mut c_void, n: size_t, game: &mut Game) {
    let data = ptr as *mut u8;
    crate::native_assertion! {
        '_c2rust_label: data >= &raw mut game.g_ram as *mut u8 && data < ram_ptr!(game; 0x20000);
        b"data >= g_ram && data < g_ram + 0x20000\0", b"src/sm_rtl.c\0", 38,
        b"void RtlSyncMemoryRegion(void *, size_t)\0"
    };
    if !game.g_rtl_memory_ptr.is_null() {
        memcpy(
            game.g_rtl_memory_ptr
                .offset(data.offset_from(&raw mut game.g_ram as *mut u8) as c_long as isize)
                as *mut c_void,
            data as *const c_void,
            n,
        );
    }
}
pub unsafe fn ByteArray_AppendVl(arr: *mut ByteArray, mut v: u32) {
    while v >= 255 {
        ByteArray_AppendByte(arr, 255);
        v = v.wrapping_sub(255);
    }
    ByteArray_AppendByte(arr, v as u8);
}
pub unsafe fn saveFunc(ctx_in: *mut c_void, data: *mut c_void, data_size: size_t, game: &mut Game) {
    ByteArray_AppendData(ctx_in as *mut ByteArray, data as *mut u8, data_size);
}
pub unsafe fn loadFunc(ctx: *mut c_void, data: *mut c_void, data_size: size_t, game: &mut Game) {
    let st = ctx as *mut LoadFuncState;
    crate::native_assertion! {
        '_c2rust_label: (*st).pend.offset_from((*st).p) as c_long as size_t >= data_size;
        b"(size_t)(st->pend - st->p) >= data_size\0", b"src/sm_rtl.c\0", 59,
        b"void loadFunc(void *, void *, size_t)\0"
    };
    memcpy(data, (*st).p as *const c_void, data_size);
    (*st).p = (*st).p.offset(data_size as isize);
}
unsafe fn LoadSnesState(func: Option<SaveLoadFunc>, ctx: *mut c_void, game: &mut Game) {
    snes_saveload(game.g_snes, func, ctx, game);
    crate::snes::ppu::PpuRestoreRenderRouting(&mut *(*game.g_snes).ppu);
    (*(*game.g_snes).cpu).e = false_0 != 0;
    let next: u32 = ((*(*game.g_snes)
        .ram
        .offset(((*(*game.g_snes).cpu).sp as i32 + 3) as isize) as i32
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
            .offset(((*(*game.g_snes).cpu).sp as i32 + 3) as isize) = (0xf71b - 1i32 & 0xff) as u8;
        *(*game.g_snes)
            .ram
            .offset(((*(*game.g_snes).cpu).sp as i32 + 4) as isize) = (0xf71b - 1i32 >> 8) as u8;
    }
    RtlSynchronizeWholeState(game);
}
unsafe fn SaveSnesState(func: Option<SaveLoadFunc>, ctx: *mut c_void, game: &mut Game) {
    snes_saveload(game.g_snes, func, ctx, game);
}

pub unsafe fn StateRecorder_Init(sr: *mut StateRecorder) {
    ByteArray_Destroy(&raw mut (*sr).log);
    ByteArray_Destroy(&raw mut (*sr).base_snapshot);
    memset(sr as *mut c_void, 0, size_of::<StateRecorder>() as size_t);
}
pub unsafe fn StateRecorder_RecordCmd(sr: *mut StateRecorder, cmd: u8) {
    let frames: i32 = (*sr).frames_since_last as i32;
    (*sr).frames_since_last = 0;
    let x: i32 = if (cmd as i32) < 0xc0 { 0xf } else { 0x1 };
    ByteArray_AppendByte(
        &raw mut (*sr).log,
        (cmd as i32 | (if frames < x { frames } else { x })) as u8,
    );
    if frames >= x {
        ByteArray_AppendVl(&raw mut (*sr).log, (frames - x) as u32);
    }
}
pub unsafe fn StateRecorder_Record(sr: *mut StateRecorder, inputs: u16) {
    let diff: u16 = (inputs ^ (*sr).last_inputs);
    if diff as i32 != 0 {
        (*sr).last_inputs = inputs;
        for i in 0i32..12 {
            if diff as i32 >> i & 1 != 0 {
                StateRecorder_RecordCmd(sr, (i << 4) as u8);
            }
        }
    }
    (*sr).frames_since_last = (*sr).frames_since_last.wrapping_add(1);
    (*sr).total_frames = (*sr).total_frames.wrapping_add(1);
}
pub unsafe fn StateRecorder_RecordPatchByte(
    sr: *mut StateRecorder,
    addr: u32,
    value: *const u8,
    num: i32,
) {
    crate::native_assertion! {
        '_c2rust_label: addr < 0x20000;
        b"addr < 0x20000\0", b"src/sm_rtl.c\0", 133,
        b"void StateRecorder_RecordPatchByte(StateRecorder *, uint32, const uint8 *, int)\0"
    };
    let lb: size_t = (*sr).log.size;
    let lq: i32 = if num - 1i32 <= 3 { num - 1i32 } else { 3 };
    StateRecorder_RecordCmd(
        sr,
        (0xc0 | (if addr & 0x10000 != 0 { 2 } else { 0 }) | lq << 2) as u8,
    );
    if lq == 3 {
        ByteArray_AppendVl(&raw mut (*sr).log, (num - 1i32 - 3i32) as u32);
    }
    ByteArray_AppendByte(&raw mut (*sr).log, (addr >> 8) as u8);
    ByteArray_AppendByte(&raw mut (*sr).log, addr as u8);
    let mut i: i32 = 0;
    while i < num {
        ByteArray_AppendByte(&raw mut (*sr).log, *value.offset(i as isize));
        i += 1;
    }
}
pub unsafe fn ReadFromFile(f: *mut FILE, data: *mut c_void, n: size_t) {
    if fread(data, 1 as size_t, n, f) as size_t != n {
        Die(b"fread failed\n\0" as *const u8 as *const c_char);
    }
}
pub unsafe fn RtlReset(mode: i32, game: &mut Game) {
    game.snes_frame_counter = 0;
    snes_reset(game.g_snes, true_0 != 0, game);
    if mode & 1 == 0 {
        memset(game.g_sram as *mut c_void, 0, 0x2000 as size_t);
    }
    *ram_ptr!(game; 0x611) = 1;
    RtlApuLock();
    RtlRestoreMusicAfterLoad_Locked(true_0 != 0, game);
    RtlApuUnlock();
    RtlSynchronizeWholeState(game);
    if mode & 2 == 0 {
        StateRecorder_Init(&raw mut game.state_recorder);
    }
}
pub unsafe fn StateRecorder_Load(
    sr: *mut StateRecorder,
    f: *mut FILE,
    replay_mode: bool,
    game: &mut Game,
) {
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
    (*sr).total_frames = hdr[1i32 as usize];
    ByteArray_Resize(&raw mut (*sr).log, hdr[2i32 as usize] as size_t);
    ReadFromFile(f, (*sr).log.data as *mut c_void, (*sr).log.size);
    (*sr).last_inputs = hdr[3i32 as usize] as u16;
    (*sr).frames_since_last = hdr[4i32 as usize];
    ByteArray_Resize(&raw mut (*sr).base_snapshot, hdr[5i32 as usize] as size_t);
    ReadFromFile(
        f,
        (*sr).base_snapshot.data as *mut c_void,
        (*sr).base_snapshot.size,
    );
    (*sr).has_base_audio_queue = false;
    (*sr).snapshot_flags = hdr[9i32 as usize];
    (*sr).replay_next_cmd_at = 0;
    (*sr).replay_mode = replay_mode;
    if replay_mode {
        crate::set_fields! {[(*sr)]; frames_since_last = 0, last_inputs = 0,
        replay_pos_last_complete = 0, replay_pos = (*sr).replay_pos_last_complete,
        replay_frame_counter = 0, };
        if (*sr).base_snapshot.size > 8192 as size_t {
            let mut state: LoadFuncState = LoadFuncState {
                p: (*sr).base_snapshot.data,
                pend: (*sr)
                    .base_snapshot
                    .data
                    .offset((*sr).base_snapshot.size as isize),
            };
            LoadSnesState(
                Some(loadFunc as unsafe fn(*mut c_void, *mut c_void, size_t, &mut Game) -> ()),
                &raw mut state as *mut c_void,
                game,
            );
            crate::native_assertion! {
                '_c2rust_label_0: state.p == state.pend;
                b"state.p == state.pend\0", b"src/sm_rtl.c\0", 219,
                b"void StateRecorder_Load(StateRecorder *, FILE *, _Bool)\0"
            };
        } else {
            RtlReset(2, game);
            if (*sr).base_snapshot.size == 8192 as size_t {
                memcpy(
                    game.g_sram as *mut c_void,
                    (*sr).base_snapshot.data as *const c_void,
                    8192 as size_t,
                );
            }
            is_reset = true_0 != 0;
        }
    } else {
        (*sr).replay_pos_last_complete = hdr[7i32 as usize];
        (*sr).replay_pos = (*sr).replay_pos_last_complete;
        (*sr).replay_frame_counter = hdr[8i32 as usize];
        (*sr).replay_mode = (*sr).replay_frame_counter != 0;
        crate::native_assertion! {
            '_c2rust_label_1: hdr[6i32 as usize] == 275493;
            b"hdr[6] == 275493\0", b"src/sm_rtl.c\0", 232,
            b"void StateRecorder_Load(StateRecorder *, FILE *, _Bool)\0"
        };
        let mut arr: ByteArray = ByteArray {
            data: null_mut::<u8>(),
            size: 0,
            capacity: 0,
        };
        ByteArray_Resize(&raw mut arr, hdr[6i32 as usize] as size_t);
        ReadFromFile(f, arr.data as *mut c_void, arr.size);
        let mut state_0: LoadFuncState = LoadFuncState {
            p: arr.data,
            pend: arr.data.offset(arr.size as isize),
        };
        LoadSnesState(
            Some(loadFunc as unsafe fn(*mut c_void, *mut c_void, size_t, &mut Game) -> ()),
            &raw mut state_0 as *mut c_void,
            game,
        );
        ByteArray_Destroy(&raw mut arr);
        crate::native_assertion! {
            '_c2rust_label_2: state_0.p == state_0.pend;
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
        ReadFromFile(f, (*sr).base_audio_queue.as_mut_ptr().cast(), 71);
        (*sr).has_base_audio_queue = (*sr).base_snapshot.size > 8192;
        if replay_mode && !is_reset {
            crate::audio_snapshot::ApuQueueSnapshot::from_bytes(&(*sr).base_audio_queue)
                .restore(game);
        }
    }
    RtlUpdateSnesPatchForBugfix(game);
}
pub unsafe fn StateRecorder_Save(
    sr: *mut StateRecorder,
    f: *mut FILE,
    saving_with_bug: bool,
    game: &mut Game,
) {
    let mut hdr: [u32; 16] = [0i32 as u32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut arr: ByteArray = ByteArray {
        data: null_mut::<u8>(),
        size: 0,
        capacity: 0,
    };
    SaveSnesState(
        Some(saveFunc as unsafe fn(*mut c_void, *mut c_void, size_t, &mut Game) -> ()),
        &raw mut arr as *mut c_void,
        game,
    );
    crate::native_assertion! {
        '_c2rust_label: (*sr).base_snapshot.size == 0 as size_t
            || (*sr).base_snapshot.size == arr.size
            || (*sr).base_snapshot.size == 8192 as size_t;
        b"sr->base_snapshot.size == 0 || sr->base_snapshot.size == arr.size || sr->base_snapshot.size == 8192\0",
        b"src/sm_rtl.c\0", 260, b"void StateRecorder_Save(StateRecorder *, FILE *, _Bool)\0"
    };
    hdr[0i32 as usize] = 2;
    hdr[1i32 as usize] = (*sr).total_frames;
    hdr[2i32 as usize] = (*sr).log.size as u32;
    hdr[3i32 as usize] = (*sr).last_inputs as u32;
    hdr[4i32 as usize] = (*sr).frames_since_last;
    hdr[5i32 as usize] = (*sr).base_snapshot.size as u32;
    hdr[6i32 as usize] = arr.size as u32;
    if (*sr).replay_mode {
        hdr[7i32 as usize] = (*sr).replay_pos_last_complete;
        hdr[8i32 as usize] = (*sr).replay_frame_counter;
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
    if (*sr).has_base_audio_queue {
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
        (*sr).log.data as *const c_void,
        1 as size_t,
        (*sr).log.size,
        f,
    );
    fwrite(
        (*sr).base_snapshot.data as *const c_void,
        1 as size_t,
        (*sr).base_snapshot.size,
        f,
    );
    fwrite(arr.data as *const c_void, 1 as size_t, arr.size, f);
    if let Some(bytes) = audio_queue {
        fwrite(bytes.as_ptr().cast(), 1, bytes.len(), f);
    }
    if (*sr).has_base_audio_queue {
        fwrite((*sr).base_audio_queue.as_ptr().cast(), 1, 71, f);
    }
    ByteArray_Destroy(&raw mut arr);
}
pub unsafe fn StateRecorder_ClearKeyLog(sr: *mut StateRecorder, game: &mut Game) {
    printf(b"Clearing key log!\n\0" as *const u8 as *const c_char);
    (*sr).base_snapshot.size = 0 as size_t;
    (*sr).has_base_audio_queue = true;
    (*sr).base_audio_queue = crate::audio_snapshot::ApuQueueSnapshot::capture(game).to_bytes();
    let live_audio = if game.g_use_my_apu_code {
        // These snapshots contain only zero-valid scalars, arrays and pointers.
        // Initialize on the heap to avoid large stack temporaries.
        let mut storage = Box::<(
            crate::audio_snapshot::AudioSnapshot,
            crate::audio_snapshot::PlayerSnapshot,
        )>::new_uninit();
        core::ptr::write_bytes(storage.as_mut_ptr(), 0, 1);
        let mut snapshots = storage.assume_init();
        snapshots.0.capture(game.g_snes);
        snapshots.1.capture(game.g_spc_player);
        // Replay restores the queue separately, so do not deliver it early.
        RtlSaveMusicStateToRam_Locked(game, false);
        Some(snapshots)
    } else {
        None
    };
    SaveSnesState(
        Some(saveFunc as unsafe fn(*mut c_void, *mut c_void, size_t, &mut Game) -> ()),
        &raw mut (*sr).base_snapshot as *mut c_void,
        game,
    );
    if let Some(snapshots) = live_audio {
        snapshots.0.restore(game.g_snes);
        snapshots.1.restore(game.g_spc_player);
    }
    let mut old_log: ByteArray = (*sr).log;
    let old_frames_since_last: i32 = (*sr).frames_since_last as i32;
    memset(
        &raw mut (*sr).log as *mut c_void,
        0,
        size_of::<ByteArray>() as size_t,
    );
    (*sr).frames_since_last = 0;
    if (*sr).last_inputs != 0 {
        for i in 0i32..12 {
            if (*sr).last_inputs as i32 >> i & 1 != 0 {
                StateRecorder_RecordCmd(sr, (i << 4) as u8);
            }
        }
    }
    if (*sr).replay_mode {
        if (*sr).replay_next_cmd_at != 0xffffffff {
            (*sr).replay_next_cmd_at = (*sr)
                .replay_next_cmd_at
                .wrapping_sub(old_frames_since_last as u32);
            (*sr).frames_since_last = (*sr).replay_next_cmd_at;
            (*sr).replay_pos_last_complete = (*sr).log.size as u32;
            StateRecorder_RecordCmd(sr, (*sr).replay_cmd);
            let old_replay_pos: i32 = (*sr).replay_pos as i32;
            (*sr).replay_pos = (*sr).log.size as u32;
            ByteArray_AppendData(
                &raw mut (*sr).log,
                old_log.data.offset(old_replay_pos as isize),
                old_log.size.wrapping_sub(old_replay_pos as size_t),
            );
        }
        (*sr).total_frames = (*sr).total_frames.wrapping_sub((*sr).replay_frame_counter);
        (*sr).replay_frame_counter = 0;
    } else {
        (*sr).total_frames = 0;
    }
    ByteArray_Destroy(&raw mut old_log);
    (*sr).frames_since_last = 0;
}
pub unsafe fn StateRecorder_ReadNextReplayState(sr: *mut StateRecorder, game: &mut Game) -> u16 {
    crate::native_assertion! {
        '_c2rust_label: (*sr).replay_mode;
        b"sr->replay_mode\0", b"src/sm_rtl.c\0", 322,
        b"uint16 StateRecorder_ReadNextReplayState(StateRecorder *)\0"
    };
    while (*sr).frames_since_last >= (*sr).replay_next_cmd_at {
        let mut replay_pos: i32 = (*sr).replay_pos as i32;
        if replay_pos as u32 != (*sr).replay_pos_last_complete {
            (*sr).frames_since_last = 0;
            if ((*sr).replay_cmd as i32) < 0xc0 {
                (*sr).last_inputs =
                    ((*sr).last_inputs as i32 ^ (1i32) << ((*sr).replay_cmd as i32 >> 4)) as u16;
            } else if ((*sr).replay_cmd as i32) < 0xd0 {
                let mut nb: i32 = 1 + ((*sr).replay_cmd as i32 >> 2 & 3);
                if nb == 4 {
                    do_until! { (t as i32 != 255);
                        crate::update_place! { previous fresh3 = replay_pos => replay_pos + 1; }
                        let mut t: u8 = *(*sr).log.data.offset(fresh3 as isize);
                        nb += t as i32;
                    }
                }
                let mut addr: u32 = (((*sr).replay_cmd as i32 >> 1 & 1) << 16) as u32;
                crate::update_place! { previous fresh4 = replay_pos => replay_pos + 1; }
                addr |= ((*(*sr).log.data.offset(fresh4 as isize) as i32) << 8) as u32;
                crate::update_place! { previous fresh5 = replay_pos => replay_pos + 1; }
                addr |= *(*sr).log.data.offset(fresh5 as isize) as u32;
                do_until! { (nb == 0);
                    crate::update_place! { previous fresh6 = replay_pos => replay_pos + 1; }
                    game.g_ram[(addr & 0x1ffff) as usize] = *(*sr).log.data.offset(fresh6 as isize);
                    RtlSyncMemoryRegion(
                        ram_ptr!(game, c_void; (addr & 0x1ffff) as isize),
                        1 as size_t,
                        game,
                    );
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
        (*sr).replay_pos_last_complete = replay_pos as u32;
        if replay_pos as size_t >= (*sr).log.size {
            (*sr).replay_pos = replay_pos as u32;
            (*sr).replay_next_cmd_at = 0xffffffff as u32;
            break;
        } else {
            crate::update_place! { previous fresh7 = replay_pos => replay_pos + 1; }
            let cmd: u8 = *(*sr).log.data.offset(fresh7 as isize);
            let mask: i32 = if (cmd as i32) < 0xc0 { 0xf } else { 0x1 };
            let mut frames: i32 = cmd as i32 & mask;
            if frames == mask {
                do_until! { (t_0 as i32 != 255);
                    crate::update_place! { previous fresh8 = replay_pos => replay_pos + 1; }
                    let mut t_0: u8 = *(*sr).log.data.offset(fresh8 as isize);
                    frames += t_0 as i32;
                }
            }
            (*sr).replay_next_cmd_at = frames as u32;
            (*sr).replay_cmd = cmd;
            (*sr).replay_pos = replay_pos as u32;
        }
    }
    (*sr).frames_since_last = (*sr).frames_since_last.wrapping_add(1);
    (*sr).replay_frame_counter = (*sr).replay_frame_counter.wrapping_add(1);
    if (*sr).replay_frame_counter >= (*sr).total_frames {
        (*sr).replay_mode = false_0 != 0;
    }
    (*sr).last_inputs
}
pub unsafe fn StateRecorder_StopReplay(sr: *mut StateRecorder) {
    if !(*sr).replay_mode {
        return;
    }
    (*sr).replay_mode = false_0 != 0;
    (*sr).total_frames = (*sr).replay_frame_counter;
    (*sr).log.size = (*sr).replay_pos_last_complete as size_t;
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    RtlClearKeyLog => StateRecorder_ClearKeyLog(&raw mut game.state_recorder, game);
    RtlStopReplay => StateRecorder_StopReplay(&raw mut game.state_recorder);
}
pub unsafe fn RtlRunFrame(mut inputs: i32, game: &mut Game) -> bool {
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
        inputs = StateRecorder_ReadNextReplayState(&raw mut game.state_recorder, game) as i32;
    } else {
        if game.state_recorder.snapshot_flags & 1 != 0 {
            game.state_recorder.snapshot_flags &= !(1i32) as u32;
            inputs = game.state_recorder.last_inputs as i32;
        } else if word!(game.bug_fix_counter) as i32 != kCurrentBugFixCounter as i32 {
            if game.g_debug_flag {
                printf(
                    b"bug_fix_counter %d => %d\n\0" as *const u8 as *const c_char,
                    word!(game.bug_fix_counter) as i32,
                    kCurrentBugFixCounter as i32,
                );
            }
            if (word!(game.bug_fix_counter) as i32) < kCurrentBugFixCounter as i32 {
                word!(game.bug_fix_counter = kCurrentBugFixCounter as i32 as u16);
                StateRecorder_RecordPatchByte(
                    &raw mut game.state_recorder,
                    (ram_ptr!(game, u16; 0xc2) as *mut u8)
                        .offset_from(&raw mut game.g_ram as *mut u8) as c_long
                        as u32,
                    ram_ptr!(game, u16; 0xc2) as *mut u8,
                    2,
                );
            }
        }
        StateRecorder_Record(&raw mut game.state_recorder, inputs as u16);
    }
    if word!(game.bug_fix_counter) as i32 != game.currently_installed_bug_fix_counter as i32 {
        RtlUpdateSnesPatchForBugfix(game);
    }
    game.g_rtl_runframe.expect("non-null function pointer")(inputs as u16, 0, game);
    game.snes_frame_counter += 1;
    RtlPushApuState(game);
    is_replay
}
pub unsafe fn RtlSaveSnapshot(filename: *const c_char, saving_with_bug: bool, game: &mut Game) {
    let f = fopen(filename, b"wb\0" as *const u8 as *const c_char) as *mut FILE;
    RtlApuLock();
    RtlSaveMusicStateToRam_Locked(game, true);
    StateRecorder_Save(&raw mut game.state_recorder, f, saving_with_bug, game);
    RtlApuUnlock();
    fclose(f);
}
static kBugSaves: crate::types::ImmutableCStringPointers<7> = crate::types::immutable_c_strings! {
    b"Before Kraid\0", b"Before Golden Torizo\0", b"After Crocomire\0", b"Baby Metroid\0",
    b"Tourian Statue\0", b"Before Ridley\0", b"Enter Mother Brain\0",
};
pub unsafe fn RtlSaveLoad(cmd: i32, slot: i32, game: &mut Game) {
    let mut name: [c_char; 128] = [0; 128];
    if slot >= 256 {
        let i: i32 = slot - 256i32;
        if cmd == kSaveLoad_Save as i32
            || i as usize
                >= (size_of::<[*const c_char; 7]>() as usize)
                    .wrapping_div(size_of::<*const c_char>() as usize)
        {
            return;
        }
        sprintf(
            &raw mut name as *mut c_char,
            b"saves/%s.sav\0" as *const u8 as *const c_char,
            kBugSaves[i as usize],
        );
    } else {
        sprintf(
            &raw mut name as *mut c_char,
            b"saves/save%d.sav\0" as *const u8 as *const c_char,
            slot,
        );
    }
    printf(
        b"*** %s slot %d\n\0" as *const u8 as *const c_char,
        if cmd == kSaveLoad_Save as i32 {
            b"Saving\0" as *const u8 as *const c_char
        } else if cmd == kSaveLoad_Load as i32 {
            b"Loading\0" as *const u8 as *const c_char
        } else {
            b"Replaying\0" as *const u8 as *const c_char
        },
        slot,
    );
    if cmd != kSaveLoad_Save as i32 {
        let f = fopen(
            &raw mut name as *mut c_char,
            b"rb\0" as *const u8 as *const c_char,
        ) as *mut FILE;
        if f.is_null() {
            printf(
                b"Failed fopen: %s\n\0" as *const u8 as *const c_char,
                &raw mut name as *mut c_char,
            );
            return;
        }
        RtlApuLock();
        StateRecorder_Load(
            &raw mut game.state_recorder,
            f,
            cmd == kSaveLoad_Replay as i32,
            game,
        );
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
                b"Coroutine state: %d, %d, %d, %d, %d\n\0" as *const u8 as *const c_char,
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
pub unsafe fn MemCpy(dst: *mut c_void, src: *const c_void, size: i32) {
    memcpy(dst, src, size as size_t);
}
pub unsafe fn MakePairU16(k: u16, j: u16) -> PairU16 {
    let r: PairU16 = PairU16 { k: k, j: j };
    r
}
pub unsafe fn mov24(a: *mut LongPtr, d: u32) {
    (*a).addr = (d & 0xffff) as VoidP;
    (*a).bank = (d >> 16) as u8;
}
pub unsafe fn Load24(src: *const LongPtr) -> u32 {
    *(src as *mut u32) & 0xffffff
}
pub unsafe fn Unreachable(game: &mut Game) -> bool {
    printf(b"Unreachable!\n\0" as *const u8 as *const c_char);
    crate::native_assertion! {
        '_c2rust_label;
        b"0\0", b"src/sm_rtl.c\0", 512, b"_Bool Unreachable(void)\0"
    };
    game.g_ram[0x1ffffi32 as usize] = 1;
    false_0 != 0
}
/// Map a banked LoROM address to an offset in the ROM allocation.
/// Callers keep their window/bounds checks and add linear table offsets afterward.
#[inline]
pub const fn lorom_offset(address: u32) -> isize {
    (((address >> 16) << 15 | address & 0x7fff) & 0x3fffff) as isize
}

/// Resolve a word address in a fixed ROM bank through the maintained validator.
#[inline]
pub(crate) unsafe fn rom_bank_ptr<const BANK: u8>(addr: u16, game: &mut Game) -> *const u8 {
    RomPtr((u32::from(BANK) << 16) | u32::from(addr), game)
}

/// Resolve an explicit bank and word address, retaining RomPtr diagnostics.
#[inline]
pub(crate) unsafe fn rom_ptr_with_bank(bank: u8, addr: u16, game: &mut Game) -> *const u8 {
    RomPtr((u32::from(bank) << 16) | u32::from(addr), game)
}

pub unsafe fn RomPtr(addr: u32, game: &mut Game) -> *const u8 {
    if addr & 0x8000 == 0 {
        printf(
            b"RomPtr - Invalid access 0x%x!\n\0" as *const u8 as *const c_char,
            addr,
        );
        game.g_fail = true_0 != 0;
    }
    game.g_rom.offset(crate::rtl::lorom_offset(addr))
}
/// Preserve the setup sequence's explicit RAM mirror and bus-write order.
pub(crate) unsafe fn write_register_and_mirror(
    register: u16,
    value: u8,
    mirror: usize,
    mirror_first: bool,
    game: &mut Game,
) {
    if mirror_first {
        game.g_ram[mirror] = value;
        WriteReg(register, value, game);
    } else {
        WriteReg(register, value, game);
        game.g_ram[mirror] = value;
    }
}

pub unsafe fn WriteReg(reg: u16, value: u8, game: &mut Game) {
    snes_write(game.g_snes, reg as u32, value, game);
}
/// Magnitude of a signed SNES word; 0x8000 remains the unsigned value 0x8000.
#[inline]
pub(crate) fn abs16(value: u16) -> u16 {
    (value as i16).unsigned_abs()
}

/// Add to a split word pair, reading low then high and writing low then high.
///
/// # Safety
/// Both pointers must allow initialized word reads and word writes. They may
/// be unaligned or overlap; neither pointee may be concurrently accessed.
#[inline]
pub(crate) unsafe fn AddToHiLo(hi: *mut u16, lo: *mut u16, value: u32) {
    let low = u32::from(lo.read_unaligned());
    let high = u32::from(hi.read_unaligned());
    let sum = (low | (high << 16)).wrapping_add(value);
    lo.write_unaligned(sum as u16);
    hi.write_unaligned((sum >> 16) as u16);
}

/// Set a split word pair, writing low before high even when the words overlap.
///
/// # Safety
/// Both pointers must allow word writes. They may be unaligned or overlap;
/// neither pointee may be concurrently accessed.
#[inline]
pub(crate) unsafe fn SetHiLo(hi: *mut u16, lo: *mut u16, value: u32) {
    lo.write_unaligned(value as u16);
    hi.write_unaligned((value >> 16) as u16);
}

pub unsafe fn Mult8x8(a: u8, b: u8) -> u16 {
    (a as i32 * b as i32) as u16
}
pub unsafe fn SnesDivide(a: u16, b: u8) -> u16 {
    ((if b as i32 == 0 {
        0xffff
    } else {
        a as i32 / b as i32
    }) as u16)
}
pub unsafe fn SnesModulus(a: u16, b: u8) -> u16 {
    ((if b as i32 == 0 {
        a as i32
    } else {
        a as i32 % b as i32
    }) as u16)
}
pub unsafe fn ReadReg(reg: u16, game: &mut Game) -> u8 {
    snes_read(game.g_snes, reg as u32, game)
}
pub unsafe fn ReadRegWord(reg: u16, game: &mut Game) -> u16 {
    let mut rv: u16 = ReadReg(reg, game) as u16;
    rv = (rv as i32 | (ReadReg((reg as i32 + 1) as u16, game) as i32) << 8) as u16;
    rv
}
pub unsafe fn WriteRegWord(reg: u16, value: u16, game: &mut Game) {
    WriteReg(reg, value as u8, game);
    WriteReg((reg as i32 + 1) as u16, (value >> 8) as u8, game);
}

pub unsafe fn RtlApuWrite(adr: u32, val: u8, game: &mut Game) {
    crate::native_assertion! {
        '_c2rust_label: adr >= APUI00 as i32 as u32 && adr <= APUI03 as i32 as u32;
        b"adr >= APUI00 && adr <= APUI03\0", b"src/sm_rtl.c\0", 571,
        b"void RtlApuWrite(uint32, uint8)\0"
    };
    if word!(game.is_uploading_apu) != 0 {
        snes_catchupApu(game.g_snes, game);
        (*(*game.g_snes).apu).inPorts[(adr & 0x3) as usize] = val;
        return;
    }
    game.g_apu_write.ports[(adr & 0x3) as usize] = val;
}
unsafe fn IsFrameEmpty(w: *mut ApuWriteEnt) -> bool {
    (*w).ports[0i32 as usize] as i32 == 255
        && (*w).ports[1i32 as usize] as i32 == 255
        && (*w).ports[2i32 as usize] as i32 == 255
        && (*w).ports[3i32 as usize] as i32 == 255
}
pub unsafe fn RtlPushApuState(game: &mut Game) {
    RtlApuLock();
    if word!(game.is_uploading_apu) == 0 {
        if game.g_apu_queue_size as i32 == 0 {
            game.g_apu_time_since_empty = 0;
        } else {
            if game.g_apu_time_since_empty as i32 >= 32
                && IsFrameEmpty(&raw mut game.g_apu_write) as i32 != 0
            {
                game.g_apu_time_since_empty = (game.g_apu_time_since_empty as i32 - 4i32) as u8;
                RtlApuUnlock();
                return;
            }
            game.g_apu_time_since_empty = game.g_apu_time_since_empty.wrapping_add(1);
        }
        crate::update_place! { previous fresh2 = game.g_apu_write_ent_pos => game.g_apu_write_ent_pos.wrapping_add(1); }
        let w0 = (&raw mut game.g_apu_write_ents as *mut ApuWriteEnt)
            .offset((fresh2 as i32 & kApuMaxQueueSize as i32 - 1i32) as isize);
        if game.g_apu_queue_size as i32 == kApuMaxQueueSize as i32 {
            let w1 = (&raw mut game.g_apu_write_ents as *mut ApuWriteEnt).offset(
                (game.g_apu_write_ent_pos as i32 & kApuMaxQueueSize as i32 - 1i32) as isize,
            );
            for i in 0i32..4 {
                if (*w1).ports[i as usize] as i32 == 255 {
                    (*w1).ports[i as usize] = (*w0).ports[i as usize];
                }
            }
        } else {
            game.g_apu_queue_size = game.g_apu_queue_size.wrapping_add(1);
        }
        *w0 = game.g_apu_write as ApuWriteEnt;
        memset(
            &raw mut game.g_apu_write as *mut c_void,
            0xff,
            size_of::<ApuWriteEnt>() as size_t,
        );
    } else {
        game.g_apu_queue_size = 0;
    }
    RtlApuUnlock();
}
unsafe fn RtlPopApuState_Locked(game: &mut Game) {
    if word!(game.is_uploading_apu) != 0 {
        return;
    }
    let input_ports = if game.g_use_my_apu_code as i32 != 0 {
        &raw mut (*game.g_spc_player).input_ports as *mut u8
    } else {
        &raw mut (*(*game.g_snes).apu).inPorts as *mut u8
    };
    if game.g_apu_queue_size as i32 != 0 {
        crate::update_place! { previous fresh1 = game.g_apu_queue_size => game.g_apu_queue_size.wrapping_sub(1); }
        let w = (&raw mut game.g_apu_write_ents as *mut ApuWriteEnt).offset(
            (game.g_apu_write_ent_pos as i32 - fresh1 as i32 & kApuMaxQueueSize as i32 - 1i32)
                as isize,
        );
        for i in 0i32..4 {
            if (*w).ports[i as usize] as i32 != 255 {
                *input_ports.offset(i as isize) = (*w).ports[i as usize];
            }
        }
    }
}
unsafe fn RtlResetApuQueue(game: &mut Game) {
    game.g_apu_queue_size = 0;
    game.g_apu_time_since_empty = game.g_apu_queue_size;
    game.g_apu_write_ent_pos = game.g_apu_time_since_empty;
    memset(
        &raw mut game.g_apu_write as *mut c_void,
        0xff,
        size_of::<ApuWriteEnt>() as size_t,
    );
}
pub unsafe fn RtlApuUpload(p: *const u8, game: &mut Game) {
    let version = if game.g_snes.is_null() {
        0
    } else {
        (*game.g_snes).runningWhichVersion
    };
    if version == 1 || version == 2 {
        let startup = p == RomPtr(0xcf8000, game);
        let snapshot = if version == 1 {
            &mut game.g_snapshot_theirs
        } else {
            &mut game.g_snapshot_mine
        };
        snapshot.apu_upload_count = snapshot.apu_upload_count.saturating_add(1);
        snapshot.apu_startup_upload = startup;
    }
    RtlApuLock();
    RtlResetApuQueue(game);
    SpcPlayer_Upload(game.g_spc_player, p);
    RtlApuUnlock();
}
unsafe fn RtlRestoreMusicAfterLoad_Locked(is_reset: bool, game: &mut Game) {
    if game.g_use_my_apu_code {
        memcpy(
            &raw mut (*game.g_spc_player).ram as *mut u8 as *mut c_void,
            &raw mut (*(*game.g_snes).apu).ram as *mut u8 as *const c_void,
            65536i32 as size_t,
        );
        memcpy(
            &raw mut (*(*game.g_spc_player).dsp).ram as *mut u8 as *mut c_void,
            &raw mut (*(*(*game.g_snes).apu).dsp).ram as *mut u8 as *const c_void,
            (size_of::<Dsp>() as size_t).wrapping_sub(8 as size_t),
        );
        SpcPlayer_CopyVariablesFromRam(game.g_spc_player);
    }
    if is_reset {
        SpcPlayer_Initialize(game.g_spc_player);
    }
    RtlResetApuQueue(game);
}
unsafe fn RtlSaveMusicStateToRam_Locked(game: &mut Game, include_queued_commands: bool) {
    if game.g_use_my_apu_code {
        let spc_player = game.g_spc_player;
        let mut i: i32 = if include_queued_commands {
            game.g_apu_queue_size as i32
        } else {
            0
        };
        while i != 0 {
            let we = (&raw mut game.g_apu_write_ents as *mut ApuWriteEnt).offset(
                (game.g_apu_write_ent_pos as i32 - i & kApuMaxQueueSize as i32 - 1i32) as isize,
            );
            for j in 0i32..4 {
                if (*we).ports[j as usize] as i32 != 255 {
                    (*spc_player).input_ports[j as usize] = (*we).ports[j as usize];
                }
            }
            i -= 1;
        }
        SpcPlayer_CopyVariablesToRam(game.g_spc_player);
        memcpy(
            &raw mut (*(*(*game.g_snes).apu).dsp).ram as *mut u8 as *mut c_void,
            &raw mut (*(*game.g_spc_player).dsp).ram as *mut u8 as *const c_void,
            (size_of::<Dsp>() as size_t).wrapping_sub(8 as size_t),
        );
        memcpy(
            &raw mut (*(*game.g_snes).apu).ram as *mut u8 as *mut c_void,
            &raw mut (*game.g_spc_player).ram as *mut u8 as *const c_void,
            65536i32 as size_t,
        );
    }
}
pub unsafe fn RtlRenderAudio(audio_buffer: *mut i16, samples: i32, channels: i32, game: &mut Game) {
    crate::native_assertion! {
        '_c2rust_label: channels == 2;
        b"channels == 2\0", b"src/sm_rtl.c\0", 679, b"void RtlRenderAudio(int16 *, int, int)\0"
    };
    RtlApuLock();
    RtlPopApuState_Locked(game);
    if !game.g_use_my_apu_code {
        if word!(game.is_uploading_apu) == 0 {
            while ((*(*(*game.g_snes).apu).dsp).sampleOffset as i32) < 534 {
                apu_cycle((*game.g_snes).apu, game);
            }
            dsp_getSamples((*(*game.g_snes).apu).dsp, audio_buffer, samples);
        }
    } else {
        SpcPlayer_GenerateSamples(game.g_spc_player);
        dsp_getSamples((*game.g_spc_player).dsp, audio_buffer, samples);
    }
    RtlApuUnlock();
}
pub unsafe fn RtlCheat(c: c_char, game: &mut Game) {
    if c as i32 == 'w' as i32 {
        word!(game.samus_health = word!(game.samus_max_health));
        StateRecorder_RecordPatchByte(
            &raw mut game.state_recorder,
            0x9c2,
            ram_ptr!(game, u16; 0x9c2) as *mut u8,
            2,
        );
    } else if c as i32 == 'q' as i32 {
        word!(checked_i32, game.samus_y_pos -= 4i32);
        word!(game.samus_y_speed = -8i32 as u16);
        StateRecorder_RecordPatchByte(
            &raw mut game.state_recorder,
            0xafa,
            ram_ptr!(game, u16; 0xafa) as *mut u8,
            2,
        );
        StateRecorder_RecordPatchByte(
            &raw mut game.state_recorder,
            0xb2e,
            ram_ptr!(game, u16; 0xb2e) as *mut u8,
            2,
        );
        word!(game.menu_index = 0);
    } else if c as i32 == 'q' as i32 {
        word!(game.japanese_text_flag = (word!(game.japanese_text_flag) == 0) as i32 as u16);
        StateRecorder_RecordPatchByte(
            &raw mut game.state_recorder,
            0x9e2,
            ram_ptr!(game, u16; 0x9e2) as *mut u8,
            2,
        );
    }
}
pub unsafe fn RtlReadSram(game: &mut Game) {
    let f = fopen(
        b"saves/sm.srm\0" as *const u8 as *const c_char,
        b"rb\0" as *const u8 as *const c_char,
    ) as *mut FILE;
    if !f.is_null() {
        if fread(game.g_sram as *mut c_void, 1 as size_t, 8192 as size_t, f) != 8192 as c_ulong {
            fprintf(
                stderr,
                b"Error reading saves/sm.srm\n\0" as *const u8 as *const c_char,
            );
        }
        fclose(f);
        RtlSynchronizeWholeState(game);
        ByteArray_Resize(&raw mut game.state_recorder.base_snapshot, 8192 as size_t);
        game.state_recorder.has_base_audio_queue = false;
        memcpy(
            game.state_recorder.base_snapshot.data as *mut c_void,
            game.g_sram as *const c_void,
            8192 as size_t,
        );
    }
}
pub unsafe fn RtlWriteSram(game: &mut Game) {
    if game.VERIFY_ACTIVE {
        return;
    }
    rename(
        b"saves/sm.srm\0" as *const u8 as *const c_char,
        b"saves/sm.srm.bak\0" as *const u8 as *const c_char,
    );
    let f = fopen(
        b"saves/sm.srm\0" as *const u8 as *const c_char,
        b"wb\0" as *const u8 as *const c_char,
    ) as *mut FILE;
    if !f.is_null() {
        fwrite(game.g_sram as *const c_void, 1 as size_t, 8192 as size_t, f);
        fclose(f);
    } else {
        fprintf(
            stderr,
            b"Unable to write saves/sm.srm\n\0" as *const u8 as *const c_char,
        );
    };
}
