//! Borrowed native-issued buffer pairs, protected by the dedicated C dispatcher.
use crate::persistence_types::*;
use crate::{Bytes, Counts, Size};
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};

pub(crate) const WRAM_SIZE: usize = 0x20000;
pub(crate) const SRAM_SIZE: usize = 0x2000;
type Callback = unsafe extern "C" fn(*mut c_void, u32, u16, *mut u8, *mut u8, u64, *mut u32) -> i32;
type SetHook = unsafe extern "C" fn(Option<Callback>, *mut c_void);
type ObserverCallback = unsafe extern "C" fn(*mut c_void, u32, u32, u16, u32, *const u8, *const u8) -> i32;
type SetObserver = unsafe extern "C" fn(Option<ObserverCallback>, *mut c_void);
type Service = unsafe extern "C" fn(*mut u8, *mut u8, u64) -> i32;
type Generation = unsafe extern "C" fn() -> u64;
type Hook = dyn FnMut(
    PersistenceEntry,
    u16,
    &mut [u8],
    &mut [u8],
    &[u8],
    &mut PersistenceServices,
) -> Result<PersistenceOutcome, String>;

pub struct PersistenceServices {
    service: Service,
    expected_ram: *const u8,
    expected_sram: *const u8,
    generation: u64,
    entry: PersistenceEntry,
    failed: bool,
}
impl PersistenceServices {
    pub fn available(&self) -> bool {
        self.entry == PersistenceEntry::Save && !self.failed
    }
    pub fn write_sram(&mut self, ram: &mut [u8], sram: &mut [u8]) -> Result<(), String> {
        if !self.available()
            || ram.len() != WRAM_SIZE
            || sram.len() != SRAM_SIZE
            || ram.as_ptr() != self.expected_ram
            || sram.as_ptr() != self.expected_sram
        {
            self.failed = true;
            return Err("persistence service requires this Save invocation's complete issued WRAM/SRAM pair".into());
        }
        // SAFETY: both are this callback's issued mutable buffers. C validates
        // their identity and generation before committing or reading either.
        let status = unsafe { (self.service)(ram.as_mut_ptr(), sram.as_mut_ptr(), self.generation) };
        if status != 0 {
            self.failed = true;
            return Err(format!("native persistence service failed: {status}"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct HookFunctions {
    pub set: SetHook,
    pub observer_set: SetObserver,
    pub sram: Bytes,
    pub generation: Generation,
    pub service: Service,
    pub counters: Counts,
}
pub(crate) struct Registration {
    callback: Box<Hook>,
    rom: Bytes,
    rom_size: Size,
    functions: HookFunctions,
}
impl Registration {
    pub fn new(
        callback: impl FnMut(
            PersistenceEntry,
            u16,
            &mut [u8],
            &mut [u8],
            &[u8],
            &mut PersistenceServices,
        ) -> Result<PersistenceOutcome, String>
        + 'static,
        rom: Bytes,
        rom_size: Size,
        functions: HookFunctions,
    ) -> Box<Self> {
        Box::new(Self { callback: Box::new(callback), rom, rom_size, functions })
    }
}

pub(crate) fn decode(entry: PersistenceEntry, value: u32) -> Result<PersistenceOutput, String> {
    match (entry, value) {
        (PersistenceEntry::Load, 0x10000 | 0x10001) => Ok(PersistenceOutput::Loaded(value as u8)),
        (PersistenceEntry::Save | PersistenceEntry::Pack | PersistenceEntry::Unpack, 0) => Ok(PersistenceOutput::Done),
        _ => Err("invalid typed persistence output".into()),
    }
}
fn encode(entry: PersistenceEntry, output: PersistenceOutput) -> Result<u32, String> {
    let value = match output {
        PersistenceOutput::Done => 0,
        PersistenceOutput::Loaded(value @ 0..=1) => 0x10000 | u32::from(value),
        PersistenceOutput::Loaded(_) => return Err("invalid persistence Load byte".into()),
    };
    decode(entry, value)?;
    Ok(value)
}

pub(crate) unsafe extern "C" fn trampoline(
    context: *mut c_void,
    entry: u32,
    argument: u16,
    ram: *mut u8,
    sram: *mut u8,
    generation: u64,
    output: *mut u32,
) -> i32 {
    // SAFETY: native excludes recursive and sibling entry before entering the
    // stable Game-owned registration and issues distinct complete scratch arrays.
    let registration = unsafe { &mut *context.cast::<Registration>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<PersistenceOutcome, String> {
        let entry = *PersistenceEntry::ALL.get(entry as usize).ok_or("invalid persistence entry")?;
        if ram.is_null()
            || sram.is_null()
            || output.is_null()
            || unsafe { (registration.functions.generation)() } != generation
        {
            return Err("invalid persistence buffer generation".into());
        }
        let mut services = PersistenceServices {
            service: registration.functions.service,
            expected_ram: ram,
            expected_sram: sram,
            generation,
            entry,
            failed: false,
        };
        // Copy the ROM so the user callback never holds a canonical native borrow.
        let rom = unsafe { std::slice::from_raw_parts((registration.rom)(), (registration.rom_size)()) }.to_vec();
        let outcome = (registration.callback)(
            entry,
            argument,
            unsafe { std::slice::from_raw_parts_mut(ram, WRAM_SIZE) },
            unsafe { std::slice::from_raw_parts_mut(sram, SRAM_SIZE) },
            &rom,
            &mut services,
        )?;
        if services.failed || unsafe { (registration.functions.generation)() } != generation {
            return Err("persistence invocation poisoned by service failure or changed generation".into());
        }
        if let PersistenceOutcome::Handled(value) = outcome {
            unsafe { *output = encode(entry, value)? };
        }
        // C checks dirty/refused buffers and service use before any fallback.
        Ok(outcome)
    }));
    match result {
        Ok(Ok(PersistenceOutcome::Handled(_))) => 1,
        Ok(Ok(PersistenceOutcome::Refused)) => 0,
        Ok(Err(error)) => {
            eprintln!("Rust persistence failure: {error}");
            -1
        }
        Err(_) => {
            eprintln!("Rust persistence panic: C retry prohibited");
            -1
        }
    }
}

type Observer = dyn FnMut(PersistenceObservation, &[u8], &[u8]) -> Result<(), String>;
pub(crate) struct ObserverRegistration {
    callback: Box<Observer>,
}
impl ObserverRegistration {
    pub fn new(
        callback: impl FnMut(PersistenceObservation, &[u8], &[u8]) -> Result<(), String> + 'static,
    ) -> Box<Self> {
        Box::new(Self { callback: Box::new(callback) })
    }
}
pub(crate) unsafe extern "C" fn observer_trampoline(
    context: *mut c_void,
    phase: u32,
    entry: u32,
    argument: u16,
    output: u32,
    ram: *const u8,
    sram: *const u8,
) -> i32 {
    // SAFETY: observer and gameplay registrations are distinct boxes. Native
    // keeps canonical images stable and forbids observer reentry/mutation.
    let registration = unsafe { &mut *context.cast::<ObserverRegistration>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let phase =
            *[PersistencePhase::Begin, PersistencePhase::End, PersistencePhase::DiskBegin, PersistencePhase::DiskEnd]
                .get(phase as usize)
                .ok_or("invalid persistence observation phase")?;
        let entry = *PersistenceEntry::ALL.get(entry as usize).ok_or("invalid observed persistence entry")?;
        if ram.is_null() || sram.is_null() {
            return Err("missing persistence observation images".into());
        }
        let output = if phase == PersistencePhase::End {
            Some(decode(entry, output)?)
        } else if output == u32::MAX {
            None
        } else {
            return Err("unexpected persistence phase output".into());
        };
        (registration.callback)(
            PersistenceObservation { phase, entry, argument, output },
            unsafe { std::slice::from_raw_parts(ram, WRAM_SIZE) },
            unsafe { std::slice::from_raw_parts(sram, SRAM_SIZE) },
        )
    }));
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => {
            eprintln!("Rust persistence observer failure: {error}");
            -1
        }
        Err(_) => {
            eprintln!("Rust persistence observer panic: evidence incomplete");
            -1
        }
    }
}
pub(crate) fn counters(function: Counts) -> PersistenceCounters {
    let mut values = [0; 22];
    // SAFETY: dedicated frozen C ABI writes exactly 22 u64 values.
    unsafe { function(values.as_mut_ptr()) };
    PersistenceCounters {
        entries: values[..4].try_into().unwrap(),
        c_entries: values[4..8].try_into().unwrap(),
        rust_entries: values[8..12].try_into().unwrap(),
        rust_refused: values[12..16].try_into().unwrap(),
        rust_completed: values[16..20].try_into().unwrap(),
        write_requests: values[20],
        write_completions: values[21],
    }
}
