use crate::camera_types::*;
use crate::transaction::{Commit, Transaction};
use crate::{Bytes, Counts, HookOutcome};
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};

type Callback = unsafe extern "C" fn(*mut c_void) -> i32;
type SetHook = unsafe extern "C" fn(Option<Callback>, *mut c_void);
type Observe = unsafe extern "C" fn(u32, u32, u16) -> i32;
type Hook = dyn FnMut(&mut [u8], &mut CameraObserver) -> Result<HookOutcome, String>;

/// Passive ordered observations only; no state access or synchronization.
pub struct CameraObserver {
    observe: Observe,
    observed: bool,
}
impl CameraObserver {
    pub fn observe(&mut self, observation: CameraObservation) -> Result<(), String> {
        self.observed = true;
        let (event, routine, operand) = match observation {
            CameraObservation::Enter { routine, operand } => (0, routine, operand),
            CameraObservation::Return { routine } => (1, routine, 0),
        };
        // SAFETY: bounded passive observation without a gameplay-state pointer.
        let status = unsafe { (self.observe)(event, routine as u32, operand) };
        if status != 0 {
            return Err(format!("invalid camera observation: {status}"));
        }
        Ok(())
    }
}
#[derive(Clone, Copy)]
pub(crate) struct HookFunctions {
    pub set: SetHook,
    pub commit: Commit,
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
        callback: impl FnMut(&mut [u8], &mut CameraObserver) -> Result<HookOutcome, String> + 'static,
        ram: Bytes,
        functions: HookFunctions,
    ) -> Box<Self> {
        Box::new(Self { callback: Box::new(callback), transaction: Transaction::new(ram, functions.commit), functions })
    }
}
pub(crate) unsafe extern "C" fn trampoline(context: *mut c_void) -> i32 {
    // SAFETY: stable Game-owned registration. All recursive/cross-consumer roots
    // are rejected by native dispatch before entering this callback again.
    let registration = unsafe { &mut *context.cast::<Registration>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<HookOutcome, String> {
        registration.transaction.begin();
        let mut observer = CameraObserver { observe: registration.functions.observe, observed: false };
        let outcome = (registration.callback)(&mut registration.transaction.scratch, &mut observer)?;
        if matches!(outcome, HookOutcome::Refused) && observer.observed {
            return Err("camera refusal after observation".into());
        }
        registration.transaction.finish(outcome, false, "camera")?;
        Ok(outcome)
    }));
    match result {
        Ok(Ok(HookOutcome::Handled)) => 1,
        Ok(Ok(HookOutcome::Refused)) => 0,
        Ok(Err(error)) => {
            eprintln!("Rust camera failure: {error}");
            -1
        }
        Err(_) => {
            eprintln!("Rust camera panic: C retry prohibited");
            -1
        }
    }
}
pub(crate) fn counters(function: Counts) -> CameraCounters {
    let mut values = [0; 56];
    // SAFETY: authenticated native adapter writes exactly56 u64 counters.
    unsafe { function(values.as_mut_ptr()) };
    CameraCounters {
        c_entries: values[0],
        rust_entries: values[1],
        rust_completed: values[2],
        rust_refused: values[3],
        routine_c_entries: values[4..17].try_into().unwrap(),
        routine_c_completed: values[17..30].try_into().unwrap(),
        routine_rust_entries: values[30..43].try_into().unwrap(),
        routine_rust_completed: values[43..56].try_into().unwrap(),
    }
}
