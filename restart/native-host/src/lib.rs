//! One process-lifetime recovered-C game behind an authenticated native library.
//! Native failures terminate the caller; use the CLI's supervised child process.
#[cfg(feature = "native-preparation")]
pub mod camera_control;
mod camera_hooks;
mod camera_types;
mod ceres_door_hooks;
#[cfg(feature = "native-preparation")]
mod control_fixture;
#[cfg(feature = "native-preparation")]
pub mod door_scrolling_control;
mod door_scrolling_hooks;
mod door_scrolling_types;
mod evidence;
mod hooks;
mod item_instruction_hooks;
mod item_instruction_types;
mod persistence_hooks;
mod persistence_types;
mod player_motion_hooks;
mod player_motion_types;
#[cfg(feature = "native-preparation")]
pub mod preparation_control;
mod prepare;
mod rio_hooks;
mod rio_native;
mod ripper_hooks;
mod ripper_types;
#[cfg(feature = "native-preparation")]
pub mod room_transition_control;
mod room_transition_hooks;
mod room_transition_types;
mod scroll_hooks;
#[cfg(feature = "native-preparation")]
pub mod smoke_control;
#[cfg(all(feature = "native-preparation", feature = "private-checks"))]
pub fn producer_input_assertion() -> fn() {
    preparation_control::producer_inputs_are_bounded_before_preparation
}
mod transaction;
mod waver_hooks;
mod waver_types;

pub use camera_hooks::CameraObserver;
pub use camera_types::{CameraCounters, CameraObservation, CameraRoutine};
pub use ceres_door_hooks::{CeresDoorCounters, CeresDoorEntry, CeresDoorOutcome, CeresDoorOutput, CeresDoorServices};
pub use door_scrolling_hooks::DoorScrollingServices;
pub use door_scrolling_types::{
    DoorScrollingCounters, DoorScrollingEntry, DoorScrollingObservation, DoorScrollingRoutine,
};
pub use hooks::{HookOutcome, ProjectileCounters, ProjectileEntry, ProjectileServices};
pub use item_instruction_hooks::ItemInstructionServices;
pub use item_instruction_types::{
    ItemInstructionCounters, ItemInstructionEntry, ItemInstructionEntryCounters, ItemInstructionObservation,
    ItemInstructionOutcome, ItemInstructionOutput, ItemInstructionPhase, ItemInstructionService,
};
use libloading::Library;
pub use persistence_hooks::PersistenceServices;
pub use persistence_types::{
    PersistenceCounters, PersistenceEntry, PersistenceObservation, PersistenceOutcome, PersistenceOutput,
    PersistencePhase,
};
pub use player_motion_hooks::PlayerMotionServices;
pub use player_motion_types::{PlayerMotionCounters, PlayerMotionEntry, PlayerMotionObservation, PlayerMotionPhase};
#[cfg(feature = "private-checks")]
pub fn private_assertions() -> Vec<(&'static str, fn())> {
    let mut assertions = prepare::private_assertions();
    assertions.push((
        "counter_transport_retains_last_service_and_child",
        evidence::counter_transport_retains_last_service_and_child as fn(),
    ));
    assertions.push((
        "all_native_counter_fields_preserve_distinct_sentinels",
        evidence::all_native_counter_fields_preserve_distinct_sentinels as fn(),
    ));
    assertions
}
pub use prepare::{ARCHIVE_SHA256, Prepared, SOURCE_REVISION, prepare};
pub use rio_hooks::{RioTransport, execute_rio};
pub use ripper_hooks::RipperServices;
pub use ripper_types::{RipperCounters, RipperEntry, RipperObservation, RipperPhase};
pub use room_transition_hooks::RoomTransitionServices;
pub use room_transition_types::{
    RoomTransitionCoroutineService, RoomTransitionCounters, RoomTransitionEntry, RoomTransitionObservation,
    RoomTransitionOutcome, RoomTransitionRoutine, RoomTransitionSyncService,
};
pub use scroll_hooks::{ScrollCounters, ScrollEntry, ScrollServices};
#[cfg(feature = "legacy-evidence")]
use serde::{Deserialize, Serialize};
use std::ffi::{CString, c_char};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
pub use waver_hooks::WaverServices;
pub use waver_types::{WaverCounters, WaverEntry, WaverObservation, WaverOutput, WaverPhase, WaverService};

pub const ROM_SHA256: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";
pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;
pub const AUDIO_RATE: u32 = 32000;
pub const AUDIO_FRAMES_PER_STEP: usize = 534;
static BOOTED: AtomicBool = AtomicBool::new(false);

/// Authenticate a prepared native library without building or booting a game.
/// Candidate gameplay is intentionally absent from this reference identity.
pub fn authenticate_library(library: &Path) -> Result<String, String> {
    prepare::verify_library(library)?;
    prepare::hash_file(library)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct State {
    pub host_steps: u64,
    pub polls: u64,
    pub updates: u64,
    pub nmis: u64,
    pub waits: u64,
    /// Actual native scheduler entries; internal C functions are not counted.
    pub c_gameplay_calls: u64,
    pub cpu_fallback_calls: u64,
    pub cpu_instructions: u64,
    pub game_state: u16,
    pub loading_game_state: u16,
    pub area: u16,
    pub room: u16,
    pub x: u16,
    pub y: u16,
    pub timer: u16,
    pub requested_buttons: u16,
    pub effective_buttons: u16,
    pub joy1_current: u16,
    pub joy1_new: u16,
}

type Boot = unsafe extern "C" fn(*const c_char) -> i32;
type Step = unsafe extern "C" fn(u16, *mut i16) -> i32;
type Counts = unsafe extern "C" fn(*mut u64);
type Buttons = unsafe extern "C" fn() -> u16;
type Bytes = unsafe extern "C" fn() -> *const u8;
type Pixels = unsafe extern "C" fn() -> *const u32;
type Size = unsafe extern "C" fn() -> usize;

pub struct Game {
    // Deliberately retained until process exit: the C runtime has process global
    // state and provides no complete teardown/reinitialization contract.
    _library: ManuallyDrop<Library>,
    step: Step,
    counts: Counts,
    buttons: Buttons,
    ram: Bytes,
    pixels: Pixels,
    rom: Bytes,
    rom_size: Size,
    hook_functions: hooks::HookFunctions,
    hook: Option<Box<hooks::Registration>>,
    scroll_functions: scroll_hooks::HookFunctions,
    scroll_hook: Option<Box<scroll_hooks::Registration>>,
    ceres_door_functions: ceres_door_hooks::HookFunctions,
    ceres_door_hook: Option<Box<ceres_door_hooks::Registration>>,
    room_transition_functions: room_transition_hooks::HookFunctions,
    room_transition_hook: Option<Box<room_transition_hooks::Registration>>,
    door_scrolling_functions: door_scrolling_hooks::HookFunctions,
    door_scrolling_hook: Option<Box<door_scrolling_hooks::Registration>>,
    camera_functions: camera_hooks::HookFunctions,
    camera_hook: Option<Box<camera_hooks::Registration>>,
    item_instruction_functions: item_instruction_hooks::HookFunctions,
    item_instruction_hook: Option<Box<item_instruction_hooks::Registration>>,
    item_instruction_observer: Option<Box<item_instruction_hooks::ObserverRegistration>>,
    persistence_functions: persistence_hooks::HookFunctions,
    persistence_hook: Option<Box<persistence_hooks::Registration>>,
    persistence_observer: Option<Box<persistence_hooks::ObserverRegistration>>,
    player_motion_functions: player_motion_hooks::HookFunctions,
    player_motion_hook: Option<Box<player_motion_hooks::Registration>>,
    player_motion_observer: Option<Box<player_motion_hooks::ObserverRegistration>>,
    waver_functions: waver_hooks::HookFunctions,
    waver_hook: Option<Box<waver_hooks::Registration>>,
    waver_observer: Option<Box<waver_hooks::ObserverRegistration>>,
    ripper_functions: ripper_hooks::HookFunctions,
    rio_functions: rio_native::Functions,
    rio_hook: Option<Box<rio_native::Registration>>,
    ripper_hook: Option<Box<ripper_hooks::Registration>>,
    ripper_observer: Option<Box<ripper_hooks::ObserverRegistration>>,
    effects: Bytes,
    effects_size: Size,
    observation: Bytes,
    observation_size: Size,
    audio: [i16; AUDIO_FRAMES_PER_STEP * 2],
    patched_rom_sha256: String,
    requested_buttons: u16,
    // Prevent moving this concrete singleton to another thread or sharing it.
    _single_thread: PhantomData<Rc<()>>,
}

impl Game {
    pub fn boot(library: &Path, rom: &Path) -> Result<Self, String> {
        if BOOTED.load(Ordering::SeqCst) {
            return Err("only one native Game may boot per process".into());
        }
        if std::fs::metadata(rom).map_err(|e| e.to_string())?.len() != 3_145_728
            || prepare::hash_file(rom)? != ROM_SHA256
        {
            return Err("pristine ROM size/SHA-256 mismatch".into());
        }
        prepare::verify_library(library)?;
        let rom = rom.canonicalize().map_err(|e| e.to_string())?;
        let path = CString::new(rom.as_os_str().as_encoded_bytes()).map_err(|e| e.to_string())?;
        // SAFETY: The binary and complete source manifest were authenticated
        // against this recipe. All symbol signatures match the bundled shim.
        unsafe {
            let library = Library::new(library).map_err(|e| e.to_string())?;
            let boot: Boot = *library.get(b"sm_native_boot\0").map_err(|e| e.to_string())?;
            let step: Step = *library.get(b"sm_native_step\0").map_err(|e| e.to_string())?;
            let counts: Counts = *library.get(b"sm_native_counts\0").map_err(|e| e.to_string())?;
            let buttons: Buttons = *library.get(b"sm_native_buttons\0").map_err(|e| e.to_string())?;
            let ram: Bytes = *library.get(b"sm_native_wram\0").map_err(|e| e.to_string())?;
            let pixels: Pixels = *library.get(b"sm_native_pixels\0").map_err(|e| e.to_string())?;
            let rom: Bytes = *library.get(b"sm_native_rom\0").map_err(|e| e.to_string())?;
            let rom_size: Size = *library.get(b"sm_native_rom_size\0").map_err(|e| e.to_string())?;
            let hook_functions = hooks::HookFunctions {
                set: *library.get(b"sm_native_set_projectile_hook\0").map_err(|e| e.to_string())?,
                commit: *library.get(b"sm_native_commit_wram\0").map_err(|e| e.to_string())?,
                generation: *library.get(b"sm_native_rom_generation\0").map_err(|e| e.to_string())?,
                spawn: *library.get(b"sm_native_spawn_plm\0").map_err(|e| e.to_string())?,
                counters: *library.get(b"sm_native_projectile_counts\0").map_err(|e| e.to_string())?,
            };
            let effects: Bytes = *library.get(b"sm_native_effects\0").map_err(|e| e.to_string())?;
            let scroll_functions = scroll_hooks::HookFunctions {
                set: *library.get(b"sm_native_set_scroll_hook\0").map_err(|e| e.to_string())?,
                commit: hook_functions.commit,
                flush: *library.get(b"sm_native_flush_vram\0").map_err(|e| e.to_string())?,
                counters: *library.get(b"sm_native_scroll_counts\0").map_err(|e| e.to_string())?,
            };
            let effects_size: Size = *library.get(b"sm_native_effects_size\0").map_err(|e| e.to_string())?;
            let ceres_door_functions = ceres_door_hooks::HookFunctions {
                set: *library.get(b"sm_native_set_ceres_door_hook\0").map_err(|e| e.to_string())?,
                commit: hook_functions.commit,
                generation: hook_functions.generation,
                spawn: *library.get(b"sm_native_ceres_door_spawn_eproj\0").map_err(|e| e.to_string())?,
                counters: *library.get(b"sm_native_ceres_door_counts\0").map_err(|e| e.to_string())?,
            };
            let observation: Bytes = *library.get(b"sm_native_observation\0").map_err(|e| e.to_string())?;
            let room_transition_functions = room_transition_hooks::HookFunctions {
                set: *library.get(b"sm_native_set_room_transition_hook\0").map_err(|e| e.to_string())?,
                commit: hook_functions.commit,
                generation: hook_functions.generation,
                service: *library.get(b"sm_native_room_transition_service\0").map_err(|e| e.to_string())?,
                observe: *library.get(b"sm_native_room_transition_observe\0").map_err(|e| e.to_string())?,
                counters: *library.get(b"sm_native_room_transition_counts\0").map_err(|e| e.to_string())?,
            };
            let observation_size: Size = *library.get(b"sm_native_observation_size\0").map_err(|e| e.to_string())?;
            let door_scrolling_functions = door_scrolling_hooks::HookFunctions {
                set: *library.get(b"sm_native_set_door_scrolling_hook\0").map_err(|e| e.to_string())?,
                commit: hook_functions.commit,
                service: *library.get(b"sm_native_door_scrolling_service\0").map_err(|e| e.to_string())?,
                observe: *library.get(b"sm_native_door_scrolling_observe\0").map_err(|e| e.to_string())?,
                counters: *library.get(b"sm_native_door_scrolling_counts\0").map_err(|e| e.to_string())?,
            };
            let camera_functions = camera_hooks::HookFunctions {
                set: *library.get(b"sm_native_set_camera_hook\0").map_err(|e| e.to_string())?,
                commit: hook_functions.commit,
                observe: *library.get(b"sm_native_camera_observe\0").map_err(|e| e.to_string())?,
                counters: *library.get(b"sm_native_camera_counts\0").map_err(|e| e.to_string())?,
            };
            let item_instruction_functions = item_instruction_hooks::HookFunctions {
                set: *library.get(b"sm_native_set_item_instruction_hook\0").map_err(|e| e.to_string())?,
                observer_set: *library.get(b"sm_native_set_item_instruction_observer\0").map_err(|e| e.to_string())?,
                commit: hook_functions.commit,
                generation: hook_functions.generation,
                service: *library.get(b"sm_native_item_instruction_service\0").map_err(|e| e.to_string())?,
                counters: *library.get(b"sm_native_item_instruction_counts\0").map_err(|e| e.to_string())?,
            };
            let persistence_functions = persistence_hooks::HookFunctions {
                set: *library.get(b"sm_native_set_persistence_hook\0").map_err(|e| e.to_string())?,
                observer_set: *library.get(b"sm_native_set_persistence_observer\0").map_err(|e| e.to_string())?,
                sram: *library.get(b"sm_native_sram\0").map_err(|e| e.to_string())?,
                generation: *library.get(b"sm_native_persistence_generation\0").map_err(|e| e.to_string())?,
                service: *library.get(b"sm_native_persistence_service\0").map_err(|e| e.to_string())?,
                counters: *library.get(b"sm_native_persistence_counts\0").map_err(|e| e.to_string())?,
            };
            let player_motion_functions = player_motion_hooks::HookFunctions {
                set: *library.get(b"sm_native_set_player_motion_hook\0").map_err(|e| e.to_string())?,
                observer_set: *library.get(b"sm_native_set_player_motion_observer\0").map_err(|e| e.to_string())?,
                commit: *library.get(b"sm_native_player_motion_commit\0").map_err(|e| e.to_string())?,
                generation: *library.get(b"sm_native_player_motion_generation\0").map_err(|e| e.to_string())?,
                available: *library.get(b"sm_native_player_motion_available\0").map_err(|e| e.to_string())?,
                bind: *library.get(b"sm_native_player_motion_bind\0").map_err(|e| e.to_string())?,
                service: *library.get(b"sm_native_player_motion_terrain\0").map_err(|e| e.to_string())?,
                counters: *library.get(b"sm_native_player_motion_counts\0").map_err(|e| e.to_string())?,
            };
            let waver_functions = waver_hooks::HookFunctions {
                set: *library.get(b"sm_native_set_waver_hook\0").map_err(|e| e.to_string())?,
                observer_set: *library.get(b"sm_native_set_waver_observer\0").map_err(|e| e.to_string())?,
                commit: *library.get(b"sm_native_waver_commit\0").map_err(|e| e.to_string())?,
                generation: *library.get(b"sm_native_waver_generation\0").map_err(|e| e.to_string())?,
                available: *library.get(b"sm_native_waver_available\0").map_err(|e| e.to_string())?,
                bind: *library.get(b"sm_native_waver_bind\0").map_err(|e| e.to_string())?,
                service: *library.get(b"sm_native_waver_service\0").map_err(|e| e.to_string())?,
                counters: *library.get(b"sm_native_waver_counts\0").map_err(|e| e.to_string())?,
            };
            let ripper_functions = ripper_hooks::HookFunctions {
                set: *library.get(b"sm_native_set_ripper_hook\0").map_err(|e| e.to_string())?,
                observer_set: *library.get(b"sm_native_set_ripper_observer\0").map_err(|e| e.to_string())?,
                commit: *library.get(b"sm_native_ripper_commit\0").map_err(|e| e.to_string())?,
                generation: *library.get(b"sm_native_ripper_generation\0").map_err(|e| e.to_string())?,
                available: *library.get(b"sm_native_ripper_available\0").map_err(|e| e.to_string())?,
                bind: *library.get(b"sm_native_ripper_bind\0").map_err(|e| e.to_string())?,
                service: *library.get(b"sm_native_ripper_service\0").map_err(|e| e.to_string())?,
                counters: *library.get(b"sm_native_ripper_counts\0").map_err(|e| e.to_string())?,
            };
            let rio_functions = rio_native::Functions {
                counters: *library.get(b"sm_native_rio_counts\0").map_err(|e| e.to_string())?,
                set: *library.get(b"sm_native_set_rio_hook\0").map_err(|e| e.to_string())?,
                generation: *library.get(b"sm_native_rio_generation\0").map_err(|e| e.to_string())?,
                available: *library.get(b"sm_native_rio_available\0").map_err(|e| e.to_string())?,
                bind: *library.get(b"sm_native_rio_bind\0").map_err(|e| e.to_string())?,
                service: *library.get(b"sm_native_rio_service\0").map_err(|e| e.to_string())?,
                commit: *library.get(b"sm_native_rio_commit\0").map_err(|e| e.to_string())?,
            };
            BOOTED
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .map_err(|_| "only one native Game may boot per process")?;
            let library = ManuallyDrop::new(library);
            let status = boot(path.as_ptr());
            if status != 0 {
                return Err(format!("native cold boot failed: {status}"));
            }
            let patched_rom_sha256 = prepare::hash_bytes(std::slice::from_raw_parts(rom(), rom_size()));
            Ok(Self {
                _library: library,
                step,
                counts,
                buttons,
                ram,
                pixels,
                rom,
                rom_size,
                hook_functions,
                hook: None,
                scroll_functions,
                scroll_hook: None,
                ceres_door_functions,
                ceres_door_hook: None,
                room_transition_functions,
                room_transition_hook: None,
                door_scrolling_functions,
                door_scrolling_hook: None,
                camera_functions,
                camera_hook: None,
                item_instruction_functions,
                item_instruction_hook: None,
                item_instruction_observer: None,
                persistence_functions,
                persistence_hook: None,
                persistence_observer: None,
                player_motion_functions,
                player_motion_hook: None,
                player_motion_observer: None,
                waver_functions,
                waver_hook: None,
                waver_observer: None,
                ripper_functions,
                rio_functions,
                rio_hook: None,
                ripper_hook: None,
                ripper_observer: None,
                effects,
                effects_size,
                observation,
                observation_size,
                audio: [0; AUDIO_FRAMES_PER_STEP * 2],
                patched_rom_sha256,
                requested_buttons: 0,
                _single_thread: PhantomData,
            })
        }
    }

    pub fn host_step(&mut self, buttons: u16) -> Result<State, String> {
        if buttons & !0x0fff != 0 {
            return Err("native buttons use only bits 0..11".into());
        }
        self.requested_buttons = buttons;
        // SAFETY: unique mutable access excludes output borrows; the native shim
        // writes exactly 534 stereo samples into this fixed buffer.
        let status = unsafe { (self.step)(buttons, self.audio.as_mut_ptr()) };
        if status != 0 {
            return Err(format!("native host step failed: {status}"));
        }
        Ok(self.state())
    }

    pub fn state(&self) -> State {
        let mut counts = [0; 8];
        // SAFETY: this C function writes precisely eight u64 counters.
        unsafe {
            (self.counts)(counts.as_mut_ptr());
        }
        let ram = self.wram();
        let word = |at: usize| u16::from_le_bytes([ram[at], ram[at + 1]]);
        State {
            host_steps: counts[0],
            polls: counts[1],
            updates: counts[2],
            nmis: counts[3],
            waits: counts[4],
            c_gameplay_calls: counts[5],
            cpu_fallback_calls: counts[6],
            cpu_instructions: counts[7],
            game_state: word(0x998),
            loading_game_state: word(0xd914),
            area: word(0x79f),
            room: word(0x79b),
            x: word(0xaf6),
            y: word(0xafa),
            timer: word(0x943),
            requested_buttons: self.requested_buttons,
            // SAFETY: boot initialized the input object for this live singleton.
            effective_buttons: unsafe { (self.buttons)() },
            joy1_current: word(0x8b),
            joy1_new: word(0x8f),
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (WIDTH, HEIGHT)
    }
    pub fn pixels(&self) -> &[u32] {
        // SAFETY: the retained C library owns this fixed buffer. A borrow of self
        // prevents host_step from mutating the buffer until the borrow ends.
        unsafe { std::slice::from_raw_parts((self.pixels)(), WIDTH * HEIGHT) }
    }
    pub fn audio(&self) -> &[i16] {
        &self.audio
    }
    pub fn wram(&self) -> &[u8] {
        // SAFETY: canonical WRAM is a fixed process-lifetime 128 KiB array.
        unsafe { std::slice::from_raw_parts((self.ram)(), 0x20000) }
    }
    pub fn rom(&self) -> &[u8] {
        // SAFETY: stable cartridge storage; borrowing Game prevents stepping.
        unsafe { std::slice::from_raw_parts((self.rom)(), (self.rom_size)()) }
    }
    pub fn effects(&self) -> &[u8] {
        // SAFETY: private native trace storage is stable until host_step.
        unsafe { std::slice::from_raw_parts((self.effects)(), (self.effects_size)()) }
    }
    pub fn observation_bytes(&self) -> &[u8] {
        // SAFETY: the passive capture is complete when host_step returns.
        unsafe { std::slice::from_raw_parts((self.observation)(), (self.observation_size)()) }
    }
    pub fn projectile_counters(&self) -> ProjectileCounters {
        hooks::counters(self.hook_functions.counters)
    }
    pub fn set_projectile_hook(
        &mut self,
        callback: impl FnMut(ProjectileEntry, &mut [u8], &[u8], &mut ProjectileServices) -> Result<HookOutcome, String>
        + 'static,
    ) -> Result<(), String> {
        // Unregister before dropping/replacing the old stable context.
        self.clear_projectile_hook();
        let mut registration =
            hooks::Registration::new(callback, self.ram, self.rom, self.rom_size, self.hook_functions);
        let context = (&mut *registration as *mut hooks::Registration).cast();
        // SAFETY: the box stays owned by Game; no call is active under &mut self.
        unsafe {
            (self.hook_functions.set)(Some(hooks::trampoline), context);
        }
        self.hook = Some(registration);
        Ok(())
    }
    pub fn clear_projectile_hook(&mut self) {
        // SAFETY: Game owns the registration and native dispatch is synchronous.
        unsafe {
            (self.hook_functions.set)(None, std::ptr::null_mut());
        }
        self.hook = None;
    }
    pub fn scroll_counters(&self) -> ScrollCounters {
        scroll_hooks::counters(self.scroll_functions.counters)
    }
    pub fn set_scroll_hook(
        &mut self,
        callback: impl FnMut(ScrollEntry, &mut [u8], &mut ScrollServices) -> Result<HookOutcome, String> + 'static,
    ) -> Result<(), String> {
        self.clear_scroll_hook();
        let mut registration = scroll_hooks::Registration::new(callback, self.ram, self.scroll_functions);
        let context = (&mut *registration as *mut scroll_hooks::Registration).cast();
        // SAFETY: native dispatch is synchronous; the stable box remains owned
        // by Game until unregistered under exclusive access.
        unsafe { (self.scroll_functions.set)(Some(scroll_hooks::trampoline), context) };
        self.scroll_hook = Some(registration);
        Ok(())
    }
    pub fn clear_scroll_hook(&mut self) {
        // SAFETY: Game owns this context and cannot step while exclusively held.
        unsafe { (self.scroll_functions.set)(None, std::ptr::null_mut()) };
        self.scroll_hook = None;
    }
    pub fn ceres_door_counters(&self) -> CeresDoorCounters {
        ceres_door_hooks::counters(self.ceres_door_functions.counters)
    }
    pub fn set_ceres_door_hook(
        &mut self,
        callback: impl FnMut(
            CeresDoorEntry,
            u16,
            u16,
            &mut [u8],
            &[u8],
            &mut CeresDoorServices,
        ) -> Result<CeresDoorOutcome, String>
        + 'static,
    ) -> Result<(), String> {
        self.clear_ceres_door_hook();
        let mut registration =
            ceres_door_hooks::Registration::new(callback, self.ram, self.rom, self.rom_size, self.ceres_door_functions);
        let context = (&mut *registration as *mut ceres_door_hooks::Registration).cast();
        // SAFETY: stable registration is owned by Game until native unregister.
        unsafe { (self.ceres_door_functions.set)(Some(ceres_door_hooks::trampoline), context) };
        self.ceres_door_hook = Some(registration);
        Ok(())
    }
    pub fn clear_ceres_door_hook(&mut self) {
        // SAFETY: exclusive Game access excludes an active native invocation.
        unsafe { (self.ceres_door_functions.set)(None, std::ptr::null_mut()) };
        self.ceres_door_hook = None;
    }
    pub fn room_transition_counters(&self) -> RoomTransitionCounters {
        room_transition_hooks::counters(self.room_transition_functions.counters)
    }
    pub fn set_room_transition_hook(
        &mut self,
        callback: impl FnMut(
            RoomTransitionEntry,
            &mut [u8],
            &[u8],
            &mut RoomTransitionServices,
        ) -> Result<RoomTransitionOutcome, String>
        + 'static,
    ) -> Result<(), String> {
        self.clear_room_transition_hook();
        let mut registration = room_transition_hooks::Registration::new(
            callback,
            self.ram,
            self.rom,
            self.rom_size,
            self.room_transition_functions,
        );
        let context = (&mut *registration as *mut room_transition_hooks::Registration).cast();
        // SAFETY: Game retains this stable box through native synchronous calls.
        unsafe { (self.room_transition_functions.set)(Some(room_transition_hooks::trampoline), context) };
        self.room_transition_hook = Some(registration);
        Ok(())
    }
    pub fn clear_room_transition_hook(&mut self) {
        // SAFETY: exclusive Game access excludes an active invocation.
        unsafe { (self.room_transition_functions.set)(None, std::ptr::null_mut()) };
        self.room_transition_hook = None;
    }
    pub fn door_scrolling_counters(&self) -> DoorScrollingCounters {
        door_scrolling_hooks::counters(self.door_scrolling_functions.counters)
    }
    pub fn set_door_scrolling_hook(
        &mut self,
        callback: impl FnMut(DoorScrollingEntry, &mut [u8], &mut DoorScrollingServices) -> Result<HookOutcome, String>
        + 'static,
    ) -> Result<(), String> {
        self.clear_door_scrolling_hook();
        let mut registration =
            door_scrolling_hooks::Registration::new(callback, self.ram, self.door_scrolling_functions);
        let context = (&mut *registration as *mut door_scrolling_hooks::Registration).cast();
        // SAFETY: the boxed callback stays alive until the native hook is cleared.
        unsafe { (self.door_scrolling_functions.set)(Some(door_scrolling_hooks::trampoline), context) };
        self.door_scrolling_hook = Some(registration);
        Ok(())
    }
    pub fn clear_door_scrolling_hook(&mut self) {
        // SAFETY: no synchronous callback is live when Game is mutably borrowed.
        unsafe { (self.door_scrolling_functions.set)(None, std::ptr::null_mut()) };
        self.door_scrolling_hook = None;
    }
    pub fn camera_counters(&self) -> CameraCounters {
        camera_hooks::counters(self.camera_functions.counters)
    }
    pub fn set_camera_hook(
        &mut self,
        callback: impl FnMut(&mut [u8], &mut CameraObserver) -> Result<HookOutcome, String> + 'static,
    ) -> Result<(), String> {
        self.clear_camera_hook();
        let mut registration = camera_hooks::Registration::new(callback, self.ram, self.camera_functions);
        let context = (&mut *registration as *mut camera_hooks::Registration).cast();
        // SAFETY: the boxed registration stays alive until native hook removal.
        unsafe { (self.camera_functions.set)(Some(camera_hooks::trampoline), context) };
        self.camera_hook = Some(registration);
        Ok(())
    }
    pub fn clear_camera_hook(&mut self) {
        // SAFETY: exclusive Game borrow excludes an active synchronous callback.
        unsafe { (self.camera_functions.set)(None, std::ptr::null_mut()) };
        self.camera_hook = None;
    }
    pub fn item_instruction_counters(&self) -> ItemInstructionCounters {
        item_instruction_hooks::counters(self.item_instruction_functions.counters)
    }
    pub fn set_item_instruction_hook(
        &mut self,
        callback: impl FnMut(
            ItemInstructionEntry,
            u16,
            Option<u16>,
            &mut [u8],
            &[u8],
            &mut ItemInstructionServices,
        ) -> Result<ItemInstructionOutcome, String>
        + 'static,
    ) -> Result<(), String> {
        self.clear_item_instruction_hook();
        let mut registration = item_instruction_hooks::Registration::new(
            callback,
            self.ram,
            self.rom,
            self.rom_size,
            self.item_instruction_functions,
        );
        let context = (&mut *registration as *mut item_instruction_hooks::Registration).cast();
        // SAFETY: this distinct boxed registration remains live until cleared.
        unsafe { (self.item_instruction_functions.set)(Some(item_instruction_hooks::trampoline), context) };
        self.item_instruction_hook = Some(registration);
        Ok(())
    }
    pub fn clear_item_instruction_hook(&mut self) {
        // SAFETY: exclusive Game access excludes synchronous callback execution.
        unsafe { (self.item_instruction_functions.set)(None, std::ptr::null_mut()) };
        self.item_instruction_hook = None;
    }
    pub fn set_item_instruction_observer(
        &mut self,
        callback: impl FnMut(ItemInstructionObservation, &[u8]) -> Result<(), String> + 'static,
    ) -> Result<(), String> {
        self.clear_item_instruction_observer();
        let mut registration = item_instruction_hooks::ObserverRegistration::new(callback);
        let context = (&mut *registration as *mut item_instruction_hooks::ObserverRegistration).cast();
        // SAFETY: observer and gameplay callback use separately owned boxes.
        unsafe {
            (self.item_instruction_functions.observer_set)(Some(item_instruction_hooks::observer_trampoline), context)
        };
        self.item_instruction_observer = Some(registration);
        Ok(())
    }
    pub fn clear_item_instruction_observer(&mut self) {
        // SAFETY: no callback can be active through this exclusive Game borrow.
        unsafe { (self.item_instruction_functions.observer_set)(None, std::ptr::null_mut()) };
        self.item_instruction_observer = None;
    }
    pub fn sram(&self) -> &[u8] {
        // SAFETY: canonical SRAM is stable until the next exclusive Game call.
        unsafe { std::slice::from_raw_parts((self.persistence_functions.sram)(), persistence_hooks::SRAM_SIZE) }
    }
    pub fn persistence_counters(&self) -> PersistenceCounters {
        persistence_hooks::counters(self.persistence_functions.counters)
    }
    pub fn set_persistence_hook(
        &mut self,
        callback: impl FnMut(
            PersistenceEntry,
            u16,
            &mut [u8],
            &mut [u8],
            &[u8],
            &mut PersistenceServices,
        ) -> Result<PersistenceOutcome, String>
        + 'static,
    ) -> Result<(), String> {
        self.clear_persistence_hook();
        let mut registration =
            persistence_hooks::Registration::new(callback, self.rom, self.rom_size, self.persistence_functions);
        let context = (&mut *registration as *mut persistence_hooks::Registration).cast();
        // SAFETY: the distinct boxed registration remains live until cleared.
        unsafe { (self.persistence_functions.set)(Some(persistence_hooks::trampoline), context) };
        self.persistence_hook = Some(registration);
        Ok(())
    }
    pub fn clear_persistence_hook(&mut self) {
        // SAFETY: exclusive Game access excludes synchronous callback execution.
        unsafe { (self.persistence_functions.set)(None, std::ptr::null_mut()) };
        self.persistence_hook = None;
    }
    pub fn set_persistence_observer(
        &mut self,
        callback: impl FnMut(PersistenceObservation, &[u8], &[u8]) -> Result<(), String> + 'static,
    ) -> Result<(), String> {
        self.clear_persistence_observer();
        let mut registration = persistence_hooks::ObserverRegistration::new(callback);
        let context = (&mut *registration as *mut persistence_hooks::ObserverRegistration).cast();
        // SAFETY: observer and gameplay registrations have separate stable boxes.
        unsafe { (self.persistence_functions.observer_set)(Some(persistence_hooks::observer_trampoline), context) };
        self.persistence_observer = Some(registration);
        Ok(())
    }
    pub fn clear_persistence_observer(&mut self) {
        // SAFETY: no callback can be active through this exclusive Game borrow.
        unsafe { (self.persistence_functions.observer_set)(None, std::ptr::null_mut()) };
        self.persistence_observer = None;
    }
    pub fn player_motion_counters(&self) -> PlayerMotionCounters {
        player_motion_hooks::counters(self.player_motion_functions.counters)
    }
    pub fn set_player_motion_hook(
        &mut self,
        callback: impl FnMut(PlayerMotionEntry, i32, &mut [u8], &mut PlayerMotionServices) -> Result<HookOutcome, String>
        + 'static,
    ) -> Result<(), String> {
        self.clear_player_motion_hook();
        let mut registration = player_motion_hooks::Registration::new(callback, self.ram, self.player_motion_functions);
        let context = (&mut *registration as *mut player_motion_hooks::Registration).cast();
        // SAFETY: the distinct boxed registration remains live until cleared.
        unsafe { (self.player_motion_functions.set)(Some(player_motion_hooks::trampoline), context) };
        self.player_motion_hook = Some(registration);
        Ok(())
    }
    pub fn clear_player_motion_hook(&mut self) {
        // SAFETY: exclusive Game access excludes synchronous callback execution.
        unsafe { (self.player_motion_functions.set)(None, std::ptr::null_mut()) };
        self.player_motion_hook = None;
    }
    pub fn set_player_motion_observer(
        &mut self,
        callback: impl FnMut(PlayerMotionObservation, &[u8]) -> Result<(), String> + 'static,
    ) -> Result<(), String> {
        self.clear_player_motion_observer();
        let mut registration = player_motion_hooks::ObserverRegistration::new(callback);
        let context = (&mut *registration as *mut player_motion_hooks::ObserverRegistration).cast();
        // SAFETY: observer and gameplay registrations have separate stable boxes.
        unsafe { (self.player_motion_functions.observer_set)(Some(player_motion_hooks::observer_trampoline), context) };
        self.player_motion_observer = Some(registration);
        Ok(())
    }
    pub fn clear_player_motion_observer(&mut self) {
        // SAFETY: no callback can be active through this exclusive Game borrow.
        unsafe { (self.player_motion_functions.observer_set)(None, std::ptr::null_mut()) };
        self.player_motion_observer = None;
    }
    pub fn waver_counters(&self) -> WaverCounters {
        waver_hooks::counters(self.waver_functions.counters)
    }
    pub fn set_waver_hook(
        &mut self,
        callback: impl FnMut(
            WaverEntry,
            u16,
            u16,
            &mut [u8],
            &[u8],
            &mut WaverServices,
        ) -> Result<Option<WaverOutput>, String>
        + 'static,
    ) -> Result<(), String> {
        self.clear_waver_hook();
        let mut registration =
            waver_hooks::Registration::new(callback, self.ram, self.rom, self.rom_size, self.waver_functions);
        let context = (&mut *registration as *mut waver_hooks::Registration).cast();
        // SAFETY: Game retains the stable registration until the hook is cleared.
        unsafe { (self.waver_functions.set)(Some(waver_hooks::trampoline), context) };
        self.waver_hook = Some(registration);
        Ok(())
    }
    pub fn clear_waver_hook(&mut self) {
        // SAFETY: exclusive Game access excludes synchronous callback execution.
        unsafe { (self.waver_functions.set)(None, std::ptr::null_mut()) };
        self.waver_hook = None;
    }
    pub fn set_waver_observer(
        &mut self,
        callback: impl FnMut(WaverObservation, &[u8]) -> Result<(), String> + 'static,
    ) -> Result<(), String> {
        self.clear_waver_observer();
        let mut registration = waver_hooks::ObserverRegistration::new(callback);
        let context = (&mut *registration as *mut waver_hooks::ObserverRegistration).cast();
        // SAFETY: observer owns a distinct stable box and native excludes reentry.
        unsafe { (self.waver_functions.observer_set)(Some(waver_hooks::observer_trampoline), context) };
        self.waver_observer = Some(registration);
        Ok(())
    }
    pub fn clear_waver_observer(&mut self) {
        // SAFETY: exclusive Game access excludes synchronous callback execution.
        unsafe { (self.waver_functions.observer_set)(None, std::ptr::null_mut()) };
        self.waver_observer = None;
    }
    pub fn ripper_counters(&self) -> RipperCounters {
        ripper_hooks::counters(self.ripper_functions.counters)
    }
    /// Install ordinary Rio ownership at native AI/instruction scheduler roots.
    pub fn install_rio(&mut self) {
        self.clear_rio();
        let mut registration = Box::new(rio_native::Registration {
            functions: self.rio_functions,
            ram: self.ram,
            rom: self.rom,
            rom_size: self.rom_size,
        });
        let context = (&mut *registration as *mut rio_native::Registration).cast();
        // SAFETY: exclusive Game borrow and retained stable registration.
        unsafe { (self.rio_functions.set)(Some(rio_native::trampoline), context) };
        self.rio_hook = Some(registration);
    }
    /// Counts: roots, native, attempted Rust, refused Rust, completed Rust and
    /// unavailable terrain (ten each), followed by three request/completion
    /// counters each and one invalid-request count.
    pub fn rio_counters(&self) -> [u64; 67] {
        let mut counts = [0; 67];
        // SAFETY: authenticated native function writes exactly 67 u64 values.
        unsafe { (self.rio_functions.counters)(counts.as_mut_ptr()) };
        counts
    }
    pub fn clear_rio(&mut self) {
        // SAFETY: exclusive Game access excludes active callbacks.
        unsafe { (self.rio_functions.set)(None, std::ptr::null_mut()) };
        self.rio_hook = None;
    }
    pub fn set_ripper_hook(
        &mut self,
        callback: impl FnMut(RipperEntry, &mut [u8], &[u8], &mut RipperServices) -> Result<Option<()>, String> + 'static,
    ) -> Result<(), String> {
        self.clear_ripper_hook();
        let mut registration =
            ripper_hooks::Registration::new(callback, self.ram, self.rom, self.rom_size, self.ripper_functions);
        let context = (&mut *registration as *mut ripper_hooks::Registration).cast();
        // SAFETY: Game retains the stable registration until the hook is cleared.
        unsafe { (self.ripper_functions.set)(Some(ripper_hooks::trampoline), context) };
        self.ripper_hook = Some(registration);
        Ok(())
    }
    pub fn clear_ripper_hook(&mut self) {
        // SAFETY: exclusive Game access excludes synchronous callback execution.
        unsafe { (self.ripper_functions.set)(None, std::ptr::null_mut()) };
        self.ripper_hook = None;
    }
    pub fn set_ripper_observer(
        &mut self,
        callback: impl FnMut(RipperObservation, &[u8]) -> Result<(), String> + 'static,
    ) -> Result<(), String> {
        self.clear_ripper_observer();
        let mut registration = ripper_hooks::ObserverRegistration::new(callback);
        let context = (&mut *registration as *mut ripper_hooks::ObserverRegistration).cast();
        // SAFETY: observer owns a distinct stable box and native excludes reentry.
        unsafe { (self.ripper_functions.observer_set)(Some(ripper_hooks::observer_trampoline), context) };
        self.ripper_observer = Some(registration);
        Ok(())
    }
    pub fn clear_ripper_observer(&mut self) {
        // SAFETY: exclusive Game access excludes synchronous callback execution.
        unsafe { (self.ripper_functions.observer_set)(None, std::ptr::null_mut()) };
        self.ripper_observer = None;
    }
    /// ROM patches may change during execution when upstream bug-fix state is
    /// installed. Recompute on demand to report the current ROM view exactly.
    pub fn patched_rom_sha256(&mut self) -> &str {
        // SAFETY: retained cartridge storage is stable for this singleton.
        self.patched_rom_sha256 =
            unsafe { prepare::hash_bytes(std::slice::from_raw_parts((self.rom)(), (self.rom_size)())) };
        &self.patched_rom_sha256
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        self.clear_rio();
        self.clear_projectile_hook();
        self.clear_scroll_hook();
        self.clear_ceres_door_hook();
        self.clear_room_transition_hook();
        self.clear_door_scrolling_hook();
        self.clear_camera_hook();
        self.clear_item_instruction_hook();
        self.clear_item_instruction_observer();
        self.clear_persistence_hook();
        self.clear_persistence_observer();
        self.clear_player_motion_hook();
        self.clear_player_motion_observer();
        self.clear_waver_hook();
        self.clear_waver_observer();
        self.clear_ripper_hook();
        self.clear_ripper_observer();
    }
}
