//! Concrete Rio FFI registration. Native excludes all callback reentry.
use crate::{
    Bytes, Size,
    rio_hooks::{RioTransport, execute_rio},
};
use sm_rio::{Entry, NativeService, Output, WRAM_SIZE};
use std::{
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
};
type Callback = unsafe extern "C" fn(*mut c_void, u32, u16, u32, *mut u16) -> i32;
type Set = unsafe extern "C" fn(Option<Callback>, *mut c_void);
type Generation = unsafe extern "C" fn() -> u64;
type Available = unsafe extern "C" fn() -> i32;
type Bind = unsafe extern "C" fn(*const u8, usize, u64) -> i32;
type Service = unsafe extern "C" fn(*mut u8, usize, u32, u16, i32, *mut u16, u64) -> i32;
type Commit = unsafe extern "C" fn(*const u8, usize, u64) -> i32;
#[derive(Clone, Copy)]
pub(crate) struct Functions {
    pub counters: crate::Counts,
    pub set: Set,
    pub generation: Generation,
    pub available: Available,
    pub bind: Bind,
    pub service: Service,
    pub commit: Commit,
}
pub(crate) struct Registration {
    pub functions: Functions,
    pub ram: Bytes,
    pub rom: Bytes,
    pub rom_size: Size,
}
struct Transport {
    functions: Functions,
    generation: u64,
    admitted: bool,
}
impl Transport {
    fn current(&self) -> Result<(), String> {
        // SAFETY: authenticated process-lifetime library and exclusive invocation.
        if unsafe { (self.functions.generation)() } != self.generation {
            return Err("Rio generation changed".into());
        }
        Ok(())
    }
}
impl RioTransport for Transport {
    fn bind(&mut self, scratch: &mut [u8]) -> Result<(), String> {
        self.current()?;
        // SAFETY: buffer lives through this synchronous callback; C binds exact size/token.
        if unsafe { (self.functions.bind)(scratch.as_ptr(), scratch.len(), self.generation) } != 0 {
            return Err("Rio native binding rejected".into());
        }
        Ok(())
    }
    fn available(&self) -> bool {
        self.admitted
    }
    fn service(&mut self, service: NativeService, k: u16, amount: i32, scratch: &mut [u8]) -> Result<u16, String> {
        self.current()?;
        let selector = match service {
            NativeService::Horizontal => 0,
            NativeService::Vertical => 1,
            NativeService::Angle => 2,
        };
        let mut result = 0;
        // SAFETY: bound buffer and exclusive callback; C authenticates token before access.
        let status = unsafe {
            (self.functions.service)(
                scratch.as_mut_ptr(),
                scratch.len(),
                selector,
                k,
                amount,
                &mut result,
                self.generation,
            )
        };
        if status != 0 {
            return Err(format!("Rio native service rejected: {status}"));
        }
        self.current()?;
        Ok(result)
    }
    fn commit(&mut self, scratch: &[u8]) -> Result<(), String> {
        self.current()?;
        // SAFETY: final WRAM already validated; C checks exact binding/token and sequence.
        if unsafe { (self.functions.commit)(scratch.as_ptr(), scratch.len(), self.generation) } != 0 {
            return Err("Rio native commit rejected".into());
        }
        Ok(())
    }
}
pub(crate) unsafe extern "C" fn trampoline(
    context: *mut c_void,
    entry: u32,
    k: u16,
    arg: u32,
    output: *mut u16,
) -> i32 {
    let outcome = catch_unwind(AssertUnwindSafe(|| -> Result<Option<Output>, String> {
        if context.is_null() || output.is_null() {
            return Err("Rio missing callback pointer".into());
        }
        // SAFETY: C excludes sibling/recursive access to Game's stable registration.
        let registration = unsafe { &*context.cast::<Registration>() };
        let entry = Entry::try_from(entry)?;
        // SAFETY: canonical WRAM is stable/exclusive. Own snapshot before native calls.
        let canonical = unsafe { std::slice::from_raw_parts((registration.ram)(), WRAM_SIZE) }.to_vec();
        // SAFETY: ROM cannot change within this synchronous invocation.
        let rom = unsafe { std::slice::from_raw_parts((registration.rom)(), (registration.rom_size)()) };
        let mut transport = Transport {
            functions: registration.functions,
            generation: unsafe { (registration.functions.generation)() },
            admitted: unsafe { (registration.functions.available)() } == 1,
        };
        let result = execute_rio(entry, k, arg, &canonical, rom, &mut transport)?;
        let encoded = match result {
            Some(Output::Continue(j)) => j,
            _ => 0,
        };
        // SAFETY: C owns the non-null output for the duration of this callback.
        unsafe { *output = encoded };
        Ok(result)
    }));
    match outcome {
        Ok(Ok(Some(_))) => 1,
        Ok(Ok(None)) => 0,
        Ok(Err(e)) => {
            eprintln!("Rust Rio terminal failure: {e}");
            -1
        }
        Err(_) => {
            eprintln!("Rust Rio panic; C retry prohibited");
            -1
        }
    }
}
