use crate::player_motion_types::*;
use crate::transaction::{Commit, Transaction, WRAM_SIZE};
use crate::{Bytes, Counts, HookOutcome};
use std::{
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
};

type Callback = unsafe extern "C" fn(*mut c_void, u32, i32) -> i32;
type SetHook = unsafe extern "C" fn(Option<Callback>, *mut c_void);
type ObserverCallback = unsafe extern "C" fn(*mut c_void, u32, u32, i32, i32, *const u8) -> i32;
type SetObserver = unsafe extern "C" fn(Option<ObserverCallback>, *mut c_void);
type Generation = unsafe extern "C" fn() -> u64;
type Available = unsafe extern "C" fn() -> i32;
type Bind = unsafe extern "C" fn(*const u8, usize, u64) -> i32;
type Service = unsafe extern "C" fn(*mut u8, usize, u32, i32, *mut i32, u64) -> i32;
type Hook = dyn FnMut(PlayerMotionEntry, i32, &mut [u8], &mut PlayerMotionServices) -> Result<HookOutcome, String>;

pub struct PlayerMotionServices {
    service: Service,
    available: bool,
    generation: u64,
    requested: bool,
    failed: bool,
}
impl PlayerMotionServices {
    pub fn available(&self) -> bool {
        self.available
    }
    pub fn move_vertical(&mut self, ram: &mut [u8], amount: i32) -> Result<i32, String> {
        self.requested = true;
        let mut result = 0;
        // SAFETY: native counts every request and validates size, exact issued
        // allocation, generation and context before reading or copying bytes.
        let status = unsafe { (self.service)(ram.as_mut_ptr(), ram.len(), 0, amount, &mut result, self.generation) };
        if status != 0 {
            self.failed = true;
            return Err(format!("player-motion terrain service rejected: {status}"));
        }
        Ok(result)
    }
}
#[derive(Clone, Copy)]
pub(crate) struct HookFunctions {
    pub set: SetHook,
    pub observer_set: SetObserver,
    pub commit: Commit,
    pub generation: Generation,
    pub available: Available,
    pub bind: Bind,
    pub service: Service,
    pub counters: Counts,
}
pub(crate) struct Registration {
    callback: Box<Hook>,
    transaction: Transaction,
    functions: HookFunctions,
}
impl Registration {
    pub fn new(
        callback: impl FnMut(PlayerMotionEntry, i32, &mut [u8], &mut PlayerMotionServices) -> Result<HookOutcome, String>
        + 'static,
        ram: Bytes,
        functions: HookFunctions,
    ) -> Box<Self> {
        Box::new(Self { callback: Box::new(callback), transaction: Transaction::new(ram, functions.commit), functions })
    }
}
pub(crate) unsafe extern "C" fn trampoline(context: *mut c_void, entry: u32, amount: i32) -> i32 {
    // SAFETY: native excludes same-root/sibling/observer reentry before touching
    // this stable Game-owned registration or the Transaction allocation.
    let registration = unsafe { &mut *context.cast::<Registration>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<HookOutcome, String> {
        let entry = *PlayerMotionEntry::ALL.get(entry as usize).ok_or("invalid player-motion selector")?;
        registration.transaction.begin();
        let generation = unsafe { (registration.functions.generation)() };
        let available = unsafe { (registration.functions.available)() } == 1;
        let bound =
            unsafe { (registration.functions.bind)(registration.transaction.scratch.as_ptr(), WRAM_SIZE, generation) };
        if bound != 0 {
            return Err("player-motion transaction binding failed".into());
        }
        let mut services = PlayerMotionServices {
            service: registration.functions.service,
            available,
            generation,
            requested: false,
            failed: false,
        };
        let outcome = (registration.callback)(entry, amount, &mut registration.transaction.scratch, &mut services)?;
        if services.failed || unsafe { (registration.functions.generation)() } != generation {
            return Err("player-motion invocation poisoned by failed service or generation".into());
        }
        registration.transaction.finish(outcome, services.requested, "player-motion")?;
        Ok(outcome)
    }));
    match result {
        Ok(Ok(HookOutcome::Handled)) => 1,
        Ok(Ok(HookOutcome::Refused)) => 0,
        Ok(Err(error)) => {
            eprintln!("Rust player-motion failure: {error}");
            -1
        }
        Err(_) => {
            eprintln!("Rust player-motion panic: C retry prohibited");
            -1
        }
    }
}
type Observer = dyn FnMut(PlayerMotionObservation, &[u8]) -> Result<(), String>;
pub(crate) struct ObserverRegistration {
    callback: Box<Observer>,
}
impl ObserverRegistration {
    pub fn new(callback: impl FnMut(PlayerMotionObservation, &[u8]) -> Result<(), String> + 'static) -> Box<Self> {
        Box::new(Self { callback: Box::new(callback) })
    }
}
pub(crate) unsafe extern "C" fn observer_trampoline(
    context: *mut c_void,
    phase: u32,
    entry: u32,
    amount: i32,
    result: i32,
    ram: *const u8,
) -> i32 {
    // SAFETY: distinct observer registration; canonical WRAM is stable and
    // native excludes any callback-driven state writer during this borrow.
    let registration = unsafe { &mut *context.cast::<ObserverRegistration>() };
    let outcome = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let phase = *[
            PlayerMotionPhase::RootBegin,
            PlayerMotionPhase::RootEnd,
            PlayerMotionPhase::TerrainBegin,
            PlayerMotionPhase::TerrainEnd,
        ]
        .get(phase as usize)
        .ok_or("invalid player-motion phase")?;
        let entry = *PlayerMotionEntry::ALL.get(entry as usize).ok_or("invalid observed player-motion selector")?;
        if ram.is_null() {
            return Err("missing player-motion WRAM observation".into());
        }
        (registration.callback)(PlayerMotionObservation { phase, entry, amount, result }, unsafe {
            std::slice::from_raw_parts(ram, WRAM_SIZE)
        })
    }));
    match outcome {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => {
            eprintln!("Rust player-motion observer failure: {error}");
            -1
        }
        Err(_) => {
            eprintln!("Rust player-motion observer panic: evidence incomplete");
            -1
        }
    }
}
pub(crate) fn counters(function: Counts) -> PlayerMotionCounters {
    let mut values = [0; 23];
    // SAFETY: dedicated native transport writes exactly 23 u64 counters.
    unsafe { function(values.as_mut_ptr()) };
    PlayerMotionCounters {
        entries: values[..2].try_into().unwrap(),
        c_entries: values[2..4].try_into().unwrap(),
        rust_entries: values[4..6].try_into().unwrap(),
        rust_refused: values[6..8].try_into().unwrap(),
        rust_completed: values[8..10].try_into().unwrap(),
        terrain_unavailable: values[10..12].try_into().unwrap(),
        terrain_requests: values[12],
        terrain_completions: values[13],
        terrain_invalid_requests: values[14],
        enemy_hit_returns: values[15..17].try_into().unwrap(),
        terrain_returns: values[17..19].try_into().unwrap(),
        feedback_returns: values[19..21].try_into().unwrap(),
        no_collision_move_returns: values[21..23].try_into().unwrap(),
    }
}
