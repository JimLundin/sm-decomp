use crate::transaction::{Commit, Transaction, WRAM_SIZE};
use crate::waver_types::*;
use crate::{Bytes, Counts, HookOutcome, Size};
use std::{
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
};
type Callback = unsafe extern "C" fn(*mut c_void, u32, u16, u16, *mut u16) -> i32;
type SetHook = unsafe extern "C" fn(Option<Callback>, *mut c_void);
type ObserverCallback = unsafe extern "C" fn(*mut c_void, u32, u32, u16, u16, u32, i32, u16, u16, *const u8) -> i32;
type SetObserver = unsafe extern "C" fn(Option<ObserverCallback>, *mut c_void);
type Generation = unsafe extern "C" fn() -> u64;
type Available = unsafe extern "C" fn() -> i32;
type Bind = unsafe extern "C" fn(*const u8, usize, u64) -> i32;
type Service = unsafe extern "C" fn(*mut u8, usize, u32, u16, i32, u16, *mut u16, u64) -> i32;
type Hook =
    dyn FnMut(WaverEntry, u16, u16, &mut [u8], &[u8], &mut WaverServices) -> Result<Option<WaverOutput>, String>;
pub struct WaverServices {
    service: Service,
    available: bool,
    generation: u64,
    requested: bool,
    failed: bool,
}
impl WaverServices {
    pub fn available(&self) -> bool {
        self.available
    }
    fn request(
        &mut self,
        ram: &mut [u8],
        service: WaverService,
        k: u16,
        argument: i32,
        auxiliary: u16,
    ) -> Result<u16, String> {
        self.requested = true;
        let mut result = 0;
        // SAFETY: native validates the exact bound allocation, size, generation,
        // selector, arguments and call order before dereferencing any buffer.
        let status = unsafe {
            (self.service)(
                ram.as_mut_ptr(),
                ram.len(),
                service as u32,
                k,
                argument,
                auxiliary,
                &mut result,
                self.generation,
            )
        };
        if status != 0 {
            self.failed = true;
            return Err(format!("Waver service rejected: {status}"));
        }
        if service != WaverService::Sine && result > 1 {
            self.failed = true;
            return Err("invalid Waver collision result".into());
        }
        Ok(result)
    }
    pub fn move_horizontal(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
        self.request(ram, WaverService::MoveHorizontal, k, amount, 0).map(|v| v as u8)
    }
    pub fn sine(&mut self, ram: &mut [u8], angle: u16, amplitude: u16) -> Result<u16, String> {
        self.request(ram, WaverService::Sine, 0, i32::from(angle), amplitude)
    }
    pub fn move_vertical(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
        self.request(ram, WaverService::MoveVertical, k, amount, 0).map(|v| v as u8)
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
        callback: impl FnMut(
            WaverEntry,
            u16,
            u16,
            &mut [u8],
            &[u8],
            &mut WaverServices,
        ) -> Result<Option<WaverOutput>, String>
        + 'static,
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
pub(crate) unsafe extern "C" fn trampoline(context: *mut c_void, entry: u32, k: u16, jp: u16, output: *mut u16) -> i32 {
    // SAFETY: C excludes observer, service and sibling reentry before invoking
    // this stable Game-owned registration and its private Transaction.
    let registration = unsafe { &mut *context.cast::<Registration>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<Option<WaverOutput>, String> {
        let entry = *WaverEntry::ALL.get(entry as usize).ok_or("invalid Waver entry")?;
        if output.is_null() {
            return Err("missing Waver result pointer".into());
        }
        registration.transaction.begin();
        let generation = unsafe { (registration.functions.generation)() };
        let available = unsafe { (registration.functions.available)() } == 1;
        if unsafe { (registration.functions.bind)(registration.transaction.scratch.as_ptr(), WRAM_SIZE, generation) }
            != 0
        {
            return Err("Waver transaction binding failed".into());
        }
        let mut services = WaverServices {
            service: registration.functions.service,
            available,
            generation,
            requested: false,
            failed: false,
        };
        // SAFETY: Game owns immutable ROM for the entire registration lifetime;
        // the three admitted native services never write it.
        let rom = unsafe { std::slice::from_raw_parts((registration.rom)(), (registration.rom_size)()) };
        let outcome = (registration.callback)(entry, k, jp, &mut registration.transaction.scratch, rom, &mut services)?;
        if services.failed || unsafe { (registration.functions.generation)() } != generation {
            return Err("Waver invocation poisoned by failed service or generation".into());
        }
        let encoded = match (entry, outcome) {
            (_, None) => 0,
            (WaverEntry::Instruction, Some(WaverOutput::Continue(value))) if value == jp => value,
            (WaverEntry::Init | WaverEntry::Main | WaverEntry::SelectAnimation, Some(WaverOutput::Done)) => 0,
            _ => return Err("invalid Waver typed result or continuation".into()),
        };
        registration.transaction.finish(
            if outcome.is_some() { HookOutcome::Handled } else { HookOutcome::Refused },
            services.requested,
            "Waver",
        )?;
        unsafe { *output = encoded };
        Ok(outcome)
    }));
    match result {
        Ok(Ok(Some(_))) => 1,
        Ok(Ok(None)) => 0,
        Ok(Err(error)) => {
            eprintln!("Rust Waver failure: {error}");
            -1
        }
        Err(_) => {
            eprintln!("Rust Waver panic: C retry prohibited");
            -1
        }
    }
}
type Observer = dyn FnMut(WaverObservation, &[u8]) -> Result<(), String>;
pub(crate) struct ObserverRegistration {
    callback: Box<Observer>,
}
impl ObserverRegistration {
    pub fn new(callback: impl FnMut(WaverObservation, &[u8]) -> Result<(), String> + 'static) -> Box<Self> {
        Box::new(Self { callback: Box::new(callback) })
    }
}
pub(crate) unsafe extern "C" fn observer_trampoline(
    context: *mut c_void,
    phase: u32,
    entry: u32,
    k: u16,
    jp: u16,
    service: u32,
    argument: i32,
    auxiliary: u16,
    result: u16,
    ram: *const u8,
) -> i32 {
    // SAFETY: distinct observer registration and stable canonical WRAM; native
    // excludes every synchronous writer and recursive registration entry.
    let registration = unsafe { &mut *context.cast::<ObserverRegistration>() };
    let outcome = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let phase = *[WaverPhase::RootBegin, WaverPhase::RootEnd, WaverPhase::ServiceBegin, WaverPhase::ServiceEnd]
            .get(phase as usize)
            .ok_or("invalid Waver phase")?;
        let entry = *WaverEntry::ALL.get(entry as usize).ok_or("invalid observed Waver entry")?;
        let service = if service == 3 {
            None
        } else {
            Some(*WaverService::ALL.get(service as usize).ok_or("invalid observed Waver service")?)
        };
        if ram.is_null() {
            return Err("missing Waver WRAM observation".into());
        }
        (registration.callback)(
            WaverObservation { phase, entry, k, jp, service, argument, auxiliary, result },
            unsafe { std::slice::from_raw_parts(ram, WRAM_SIZE) },
        )
    }));
    match outcome {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => {
            eprintln!("Rust Waver observer failure: {error}");
            -1
        }
        Err(_) => {
            eprintln!("Rust Waver observer panic: evidence incomplete");
            -1
        }
    }
}
pub(crate) fn counters(function: Counts) -> WaverCounters {
    let mut v = [0; 31];
    // SAFETY: dedicated native transport writes exactly 31 u64 values.
    unsafe { function(v.as_mut_ptr()) };
    WaverCounters {
        entries: v[..4].try_into().unwrap(),
        c_entries: v[4..8].try_into().unwrap(),
        rust_entries: v[8..12].try_into().unwrap(),
        rust_refused: v[12..16].try_into().unwrap(),
        rust_completed: v[16..20].try_into().unwrap(),
        terrain_unavailable: v[20..24].try_into().unwrap(),
        service_requests: v[24..27].try_into().unwrap(),
        service_completions: v[27..30].try_into().unwrap(),
        invalid_requests: v[30],
    }
}
