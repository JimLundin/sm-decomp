use crate::door_scrolling_types::*;
use crate::transaction::{Commit, Transaction, WRAM_SIZE};
use crate::{Bytes, Counts, HookOutcome};
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};

type Callback = unsafe extern "C" fn(*mut c_void, u32) -> i32;
type SetHook = unsafe extern "C" fn(Option<Callback>, *mut c_void);
type Service = unsafe extern "C" fn(*mut u8, u32, *mut u8) -> i32;
type Observe = unsafe extern "C" fn(u32, u32, u8, u16) -> i32;
type Hook = dyn FnMut(DoorScrollingEntry, &mut [u8], &mut DoorScrollingServices) -> Result<HookOutcome, String>;

/// Each operation synchronously copies this invocation's complete private WRAM.
pub struct DoorScrollingServices {
    service: Service,
    observe: Observe,
    expected_ram: *const u8,
    requested: bool,
    observed: bool,
}
impl DoorScrollingServices {
    fn call(&mut self, ram: &mut [u8], id: u32) -> Result<u8, String> {
        self.requested = true;
        if ram.len() != WRAM_SIZE || ram.as_ptr() != self.expected_ram {
            return Err("door-scrolling service requires this invocation's complete WRAM copy".into());
        }
        let mut result = 0;
        // SAFETY: native validates the fixed operation and synchronizes both ways.
        let status = unsafe { (self.service)(ram.as_mut_ptr(), id, &mut result) };
        if status != 0 {
            return Err(format!("door-scrolling service {id} failed: {status}"));
        }
        Ok(result)
    }
    pub fn calculate_layer2_x(&mut self, ram: &mut [u8]) -> Result<u8, String> {
        self.call(ram, 0)
    }
    pub fn calculate_layer2_y(&mut self, ram: &mut [u8]) -> Result<u8, String> {
        self.call(ram, 1)
    }
    pub fn calculate_blocks(&mut self, ram: &mut [u8]) -> Result<(), String> {
        self.unit(ram, 2)
    }
    pub fn update_scroll_vars(&mut self, ram: &mut [u8]) -> Result<(), String> {
        self.unit(ram, 3)
    }
    fn unit(&mut self, ram: &mut [u8], id: u32) -> Result<(), String> {
        if self.call(ram, id)? != 0 {
            return Err("nonzero void door-scrolling service result".into());
        }
        Ok(())
    }
    pub fn observe(&mut self, observation: DoorScrollingObservation) -> Result<(), String> {
        self.observed = true;
        let (event, routine, result, counter) = match observation {
            DoorScrollingObservation::Enter { routine, counter } => (0, routine, 0, counter),
            DoorScrollingObservation::Return { routine, result, counter } => (1, routine, result, counter),
        };
        // SAFETY: passive bounded observation, without any game-state pointer.
        let status = unsafe { (self.observe)(event, routine as u32, result, counter) };
        if status != 0 {
            return Err(format!("invalid door-scrolling observation: {status}"));
        }
        Ok(())
    }
}
#[derive(Clone, Copy)]
pub(crate) struct HookFunctions {
    pub set: SetHook,
    pub commit: Commit,
    pub service: Service,
    pub observe: Observe,
    pub counters: Counts,
}
pub(crate) struct Registration {
    callback: Box<Hook>,
    transaction: Transaction,
    functions: HookFunctions,
}
impl Registration {
    pub fn new(
        callback: impl FnMut(DoorScrollingEntry, &mut [u8], &mut DoorScrollingServices) -> Result<HookOutcome, String>
        + 'static,
        ram: Bytes,
        functions: HookFunctions,
    ) -> Box<Self> {
        Box::new(Self { callback: Box::new(callback), transaction: Transaction::new(ram, functions.commit), functions })
    }
}
pub(crate) unsafe extern "C" fn trampoline(context: *mut c_void, entry: u32) -> i32 {
    // SAFETY: stable Game-owned registration; nesting uses distinct registrations.
    let registration = unsafe { &mut *context.cast::<Registration>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<HookOutcome, String> {
        let entry = match entry {
            0 => DoorScrollingEntry::FixDoorsMovingUp,
            1 => DoorScrollingEntry::ScrollingSetup,
            2 => DoorScrollingEntry::FollowTransition,
            _ => return Err("invalid native door-scrolling entry".into()),
        };
        registration.transaction.begin();
        let mut services = DoorScrollingServices {
            service: registration.functions.service,
            observe: registration.functions.observe,
            expected_ram: registration.transaction.scratch.as_ptr(),
            requested: false,
            observed: false,
        };
        let outcome = (registration.callback)(entry, &mut registration.transaction.scratch, &mut services)?;
        if matches!(outcome, HookOutcome::Refused) && services.observed {
            return Err("door-scrolling refusal after observation".into());
        }
        registration.transaction.finish(outcome, services.requested, "door scrolling")?;
        Ok(outcome)
    }));
    match result {
        Ok(Ok(HookOutcome::Handled)) => 1,
        Ok(Ok(HookOutcome::Refused)) => 0,
        Ok(Err(error)) => {
            eprintln!("Rust door-scrolling failure: {error}");
            -1
        }
        Err(_) => {
            eprintln!("Rust door-scrolling panic: C retry prohibited");
            -1
        }
    }
}
pub(crate) fn counters(function: Counts) -> DoorScrollingCounters {
    let mut v = [0; 94];
    // SAFETY: authenticated adapter writes exactly94 u64 counters.
    unsafe { function(v.as_mut_ptr()) };
    DoorScrollingCounters {
        c_entries: v[0..3].try_into().unwrap(),
        rust_entries: v[3..6].try_into().unwrap(),
        rust_completed: v[6..9].try_into().unwrap(),
        rust_refused: v[9..12].try_into().unwrap(),
        routine_c_entries: v[12..25].try_into().unwrap(),
        routine_c_completed: v[25..38].try_into().unwrap(),
        routine_rust_entries: v[38..51].try_into().unwrap(),
        routine_rust_completed: v[51..64].try_into().unwrap(),
        service_requests: v[64..68].try_into().unwrap(),
        service_completions: v[68..72].try_into().unwrap(),
        foreign_requests: v[72..76].try_into().unwrap(),
        foreign_completions: v[76..80].try_into().unwrap(),
        child_entries: v[80..84].try_into().unwrap(),
        child_completions: v[84..88].try_into().unwrap(),
        coordinator_entries: v[88..91].try_into().unwrap(),
        standalone_entries: v[91..94].try_into().unwrap(),
    }
}
