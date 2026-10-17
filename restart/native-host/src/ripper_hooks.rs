use crate::ripper_types::*;
use crate::transaction::{Commit, Transaction, WRAM_SIZE};
use crate::{Bytes, Counts, HookOutcome, Size};
use std::{
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
};
type Callback = unsafe extern "C" fn(*mut c_void, u32) -> i32;
type SetHook = unsafe extern "C" fn(Option<Callback>, *mut c_void);
type ObserverCallback = unsafe extern "C" fn(*mut c_void, u32, u32, u16, i32, u8, *const u8) -> i32;
type SetObserver = unsafe extern "C" fn(Option<ObserverCallback>, *mut c_void);
type Generation = unsafe extern "C" fn() -> u64;
type Available = unsafe extern "C" fn() -> i32;
type Bind = unsafe extern "C" fn(*const u8, usize, u64) -> i32;
type Service = unsafe extern "C" fn(*mut u8, usize, u32, u16, i32, *mut u8, u64) -> i32;
type Hook = dyn FnMut(RipperEntry, &mut [u8], &[u8], &mut RipperServices) -> Result<Option<()>, String>;
pub struct RipperServices {
    service: Service,
    available: bool,
    generation: u64,
    requested: bool,
    failed: bool,
}
impl RipperServices {
    pub fn available(&self) -> bool {
        self.available
    }
    pub fn move_horizontal(&mut self, ram: &mut [u8], slot: u16, amount: i32) -> Result<u8, String> {
        self.requested = true;
        let mut result = 0;
        // SAFETY: C authenticates exact allocation, size, generation and arguments
        // before reading the buffer. Every failed request poisons the invocation.
        let status =
            unsafe { (self.service)(ram.as_mut_ptr(), ram.len(), 0, slot, amount, &mut result, self.generation) };
        if status != 0 || result > 1 {
            self.failed = true;
            return Err(format!("Ripper service rejected: status={status} result={result}"));
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
    rom: Bytes,
    rom_size: Size,
    functions: HookFunctions,
}
impl Registration {
    pub fn new(
        callback: impl FnMut(RipperEntry, &mut [u8], &[u8], &mut RipperServices) -> Result<Option<()>, String> + 'static,
        ram: Bytes,
        rom: Bytes,
        rom_size: Size,
        functions: HookFunctions,
    ) -> Box<Self> {
        Box::new(Self {
            callback: Box::new(callback),
            transaction: Transaction::new(ram, functions.commit),
            rom,
            rom_size,
            functions,
        })
    }
}
pub(crate) unsafe extern "C" fn trampoline(context: *mut c_void, entry: u32) -> i32 {
    // SAFETY: native excludes sibling/observer/reentrant access to this stable
    // Game-owned registration and its private Transaction allocation.
    let registration = unsafe { &mut *context.cast::<Registration>() };
    let outcome = catch_unwind(AssertUnwindSafe(|| -> Result<Option<()>, String> {
        let entry = *RipperEntry::ALL.get(entry as usize).ok_or("invalid Ripper entry")?;
        registration.transaction.begin();
        let generation = unsafe { (registration.functions.generation)() };
        let available = unsafe { (registration.functions.available)() } == 1;
        if unsafe { (registration.functions.bind)(registration.transaction.scratch.as_ptr(), WRAM_SIZE, generation) }
            != 0
        {
            return Err("Ripper transaction binding failed".into());
        }
        let mut services = RipperServices {
            service: registration.functions.service,
            available,
            generation,
            requested: false,
            failed: false,
        };
        // SAFETY: Game-bound ROM is immutable; native horizontal movement never writes it.
        let rom = unsafe { std::slice::from_raw_parts((registration.rom)(), (registration.rom_size)()) };
        let outcome = (registration.callback)(entry, &mut registration.transaction.scratch, rom, &mut services)?;
        if services.failed || unsafe { (registration.functions.generation)() } != generation {
            return Err("Ripper invocation poisoned by failed service or generation".into());
        }
        registration.transaction.finish(
            if outcome.is_some() { HookOutcome::Handled } else { HookOutcome::Refused },
            services.requested,
            "Ripper",
        )?;
        Ok(outcome)
    }));
    match outcome {
        Ok(Ok(Some(()))) => 1,
        Ok(Ok(None)) => 0,
        Ok(Err(e)) => {
            eprintln!("Rust Ripper failure: {e}");
            -1
        }
        Err(_) => {
            eprintln!("Rust Ripper panic: C retry prohibited");
            -1
        }
    }
}
type Observer = dyn FnMut(RipperObservation, &[u8]) -> Result<(), String>;
pub(crate) struct ObserverRegistration {
    callback: Box<Observer>,
}
impl ObserverRegistration {
    pub fn new(callback: impl FnMut(RipperObservation, &[u8]) -> Result<(), String> + 'static) -> Box<Self> {
        Box::new(Self { callback: Box::new(callback) })
    }
}
pub(crate) unsafe extern "C" fn observer_trampoline(
    context: *mut c_void,
    phase: u32,
    entry: u32,
    slot: u16,
    amount: i32,
    result: u8,
    ram: *const u8,
) -> i32 {
    // SAFETY: distinct observer registration, stable canonical WRAM and native
    // guards excluding every callback-driven writer for the duration of this borrow.
    let registration = unsafe { &mut *context.cast::<ObserverRegistration>() };
    let outcome = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let phase = *[RipperPhase::RootBegin, RipperPhase::RootEnd, RipperPhase::ServiceBegin, RipperPhase::ServiceEnd]
            .get(phase as usize)
            .ok_or("invalid Ripper phase")?;
        let entry = *RipperEntry::ALL.get(entry as usize).ok_or("invalid observed Ripper entry")?;
        if ram.is_null() {
            return Err("missing Ripper WRAM observation".into());
        }
        (registration.callback)(RipperObservation { phase, entry, slot, amount, result }, unsafe {
            std::slice::from_raw_parts(ram, WRAM_SIZE)
        })
    }));
    match outcome {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => {
            eprintln!("Rust Ripper observer failure: {e}");
            -1
        }
        Err(_) => {
            eprintln!("Rust Ripper observer panic: evidence incomplete");
            -1
        }
    }
}
pub(crate) fn counters(function: Counts) -> RipperCounters {
    let mut v = [0; 27];
    // SAFETY: dedicated C transport writes exactly27 u64 counters.
    unsafe { function(v.as_mut_ptr()) };
    RipperCounters {
        entries: v[..4].try_into().unwrap(),
        c_entries: v[4..8].try_into().unwrap(),
        rust_entries: v[8..12].try_into().unwrap(),
        rust_refused: v[12..16].try_into().unwrap(),
        rust_completed: v[16..20].try_into().unwrap(),
        terrain_unavailable: v[20..24].try_into().unwrap(),
        service_requests: v[24],
        service_completions: v[25],
        invalid_requests: v[26],
    }
}
