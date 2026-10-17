use crate::hooks::Generation;
use crate::room_transition_types::*;
use crate::transaction::{Commit, Transaction, WRAM_SIZE};
use crate::{Bytes, Counts, HookOutcome, Size};
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};

type Callback = unsafe extern "C" fn(*mut c_void, u32, *mut u8) -> i32;
type SetHook = unsafe extern "C" fn(Option<Callback>, *mut c_void);
type Service = unsafe extern "C" fn(*mut u8, u32, u32, u32, u32, *mut u16) -> i32;
type Observe = unsafe extern "C" fn(u32, u32, u16) -> i32;
type Hook = dyn FnMut(
    RoomTransitionEntry,
    &mut [u8],
    &[u8],
    &mut RoomTransitionServices,
) -> Result<RoomTransitionOutcome, String>;

/// Fixed named synchronous operations; no state or borrow survives a root return.
pub struct RoomTransitionServices {
    service: Service,
    observe: Observe,
    expected_ram: *const u8,
    requested: bool,
    observed: bool,
}
impl RoomTransitionServices {
    fn call(&mut self, ram: &mut [u8], id: u32, args: [u32; 3]) -> Result<u16, String> {
        self.requested = true;
        if ram.len() != WRAM_SIZE || ram.as_ptr() != self.expected_ram {
            return Err("room-transition service requires this invocation's complete WRAM copy".into());
        }
        let mut result = 0;
        // SAFETY: the native adapter validates scope/arguments, synchronizes
        // canonical state, runs admitted child hooks and copies back before return.
        let status = unsafe { (self.service)(ram.as_mut_ptr(), id, args[0], args[1], args[2], &mut result) };
        if status != 0 {
            return Err(format!("room-transition service {id} failed: {status}"));
        }
        Ok(result)
    }
    pub fn sync(&mut self, ram: &mut [u8], service: RoomTransitionSyncService) -> Result<(), String> {
        let (id, args) = service.encode();
        if self.call(ram, id, args)? != 0 {
            return Err("nonzero synchronous transition service result".into());
        }
        Ok(())
    }
    pub fn palette_fade(&mut self, ram: &mut [u8]) -> Result<u8, String> {
        u8::try_from(self.call(ram, 48, [0; 3])?).map_err(|_| "palette fade result exceeds u8".into())
    }
    pub fn has_queued_music(&mut self, ram: &mut [u8]) -> Result<u8, String> {
        u8::try_from(self.call(ram, 49, [0; 3])?).map_err(|_| "music query result exceeds u8".into())
    }
    pub fn call_samus_code(&mut self, ram: &mut [u8], code: u16) -> Result<u16, String> {
        self.call(ram, 50, [code.into(), 0, 0])
    }
    pub fn coroutine(&mut self, ram: &mut [u8], service: RoomTransitionCoroutineService) -> Result<u8, String> {
        let id = match service {
            RoomTransitionCoroutineService::WaitForNmi => 51,
            RoomTransitionCoroutineService::PlmHandler => 52,
        };
        u8::try_from(self.call(ram, id, [0; 3])?).map_err(|_| "coroutine result exceeds u8".into())
    }
    pub fn observe(&mut self, observation: RoomTransitionObservation) -> Result<(), String> {
        self.observed = true;
        let (event, routine, value) = match observation {
            RoomTransitionObservation::Enter { routine, operand } => (0, routine, operand),
            RoomTransitionObservation::Return { routine, result } => (1, routine, result),
        };
        // SAFETY: passive fixed-catalog observation; no state pointer is exposed.
        let status = unsafe { (self.observe)(event, routine as u32, value) };
        if status != 0 {
            return Err(format!("invalid room-transition routine observation: {status}"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct HookFunctions {
    pub set: SetHook,
    pub commit: Commit,
    pub generation: Generation,
    pub service: Service,
    pub observe: Observe,
    pub counters: Counts,
}
pub(crate) struct Registration {
    callback: Box<Hook>,
    transaction: Transaction,
    rom: Bytes,
    rom_size: Size,
    rom_copy: Vec<u8>,
    rom_generation: Option<u64>,
    functions: HookFunctions,
}
impl Registration {
    pub fn new(
        callback: impl FnMut(
            RoomTransitionEntry,
            &mut [u8],
            &[u8],
            &mut RoomTransitionServices,
        ) -> Result<RoomTransitionOutcome, String>
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
            rom_copy: Vec::new(),
            rom_generation: None,
            functions,
        })
    }
}
pub(crate) unsafe extern "C" fn trampoline(context: *mut c_void, entry: u32, output: *mut u8) -> i32 {
    // SAFETY: this stable registration belongs to Game. Only a distinct existing
    // child registration may execute while the parent is inside its native service.
    let registration = unsafe { &mut *context.cast::<Registration>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<RoomTransitionOutcome, String> {
        let entry = match entry {
            0 => RoomTransitionEntry::HitDoorBlock,
            1 => RoomTransitionEntry::LoadingNextRoom,
            2 => RoomTransitionEntry::LoadingNextRoomAsync,
            _ => return Err("invalid native room-transition entry".into()),
        };
        registration.transaction.begin();
        // SAFETY: retained native cartridge and generation are stable during this callback.
        unsafe {
            let generation = (registration.functions.generation)();
            if registration.rom_generation != Some(generation) {
                registration.rom_copy =
                    std::slice::from_raw_parts((registration.rom)(), (registration.rom_size)()).to_vec();
                registration.rom_generation = Some(generation);
            }
        }
        let mut services = RoomTransitionServices {
            service: registration.functions.service,
            observe: registration.functions.observe,
            expected_ram: registration.transaction.scratch.as_ptr(),
            requested: false,
            observed: false,
        };
        let outcome = (registration.callback)(
            entry,
            &mut registration.transaction.scratch,
            &registration.rom_copy,
            &mut services,
        )?;
        let ownership = match outcome {
            RoomTransitionOutcome::Handled(_) => HookOutcome::Handled,
            RoomTransitionOutcome::Refused => {
                if services.observed {
                    return Err("room-transition refusal after observation".into());
                }
                HookOutcome::Refused
            }
        };
        registration.transaction.finish(ownership, services.requested, "room transition")?;
        if let RoomTransitionOutcome::Handled(value) = outcome {
            // SAFETY: native supplies this live byte for the synchronous callback.
            unsafe { *output = value };
        }
        Ok(outcome)
    }));
    match result {
        Ok(Ok(RoomTransitionOutcome::Handled(_))) => 1,
        Ok(Ok(RoomTransitionOutcome::Refused)) => 0,
        Ok(Err(error)) => {
            eprintln!("Rust room-transition failure: {error}");
            -1
        }
        Err(_) => {
            eprintln!("Rust room-transition panic: C retry prohibited");
            -1
        }
    }
}
pub(crate) fn counters(function: Counts) -> RoomTransitionCounters {
    let mut values = [0; 709];
    // SAFETY: authenticated native adapter writes exactly 709 u64 counters.
    unsafe { function(values.as_mut_ptr()) };
    RoomTransitionCounters {
        c_entries: values[0..3].try_into().unwrap(),
        rust_entries: values[3..6].try_into().unwrap(),
        rust_completed: values[6..9].try_into().unwrap(),
        rust_refused: values[9..12].try_into().unwrap(),
        root_pending: values[12..15].try_into().unwrap(),
        root_resumed: values[15..18].try_into().unwrap(),
        routine_c_entries: values[18..45].try_into().unwrap(),
        routine_c_completed: values[45..72].try_into().unwrap(),
        routine_rust_entries: values[72..99].try_into().unwrap(),
        routine_rust_completed: values[99..126].try_into().unwrap(),
        service_requests: values[126..179].try_into().unwrap(),
        service_completions: values[179..232].try_into().unwrap(),
        foreign_requests: values[232..285].try_into().unwrap(),
        foreign_completions: values[285..338].try_into().unwrap(),
        service_pending: values[338..391].try_into().unwrap(),
        child_entries: std::array::from_fn(|i| values[391 + i * 3..391 + (i + 1) * 3].try_into().unwrap()),
        child_completions: std::array::from_fn(|i| values[550 + i * 3..550 + (i + 1) * 3].try_into().unwrap()),
    }
}
