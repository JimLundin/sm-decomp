use crate::hooks::Generation;
use crate::transaction::{Commit, Transaction, WRAM_SIZE};
use crate::{Bytes, Counts, HookOutcome, Size};
#[cfg(feature = "legacy-evidence")]
use serde::{Deserialize, Serialize};
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum CeresDoorEntry {
    Init = 0,
    Main = 1,
    Instr1 = 2,
    Instr2 = 3,
    Instr3 = 4,
    Instr4 = 5,
    Instr5 = 6,
    Instr6 = 7,
    Instr7 = 8,
    Instr8 = 9,
    SetTangible = 10,
    SetInvisible = 11,
    SolidSetup = 12,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeresDoorOutput {
    Done,
    InstructionOffset(u16),
    SetupStatus(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeresDoorOutcome {
    Handled(CeresDoorOutput),
    Refused,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct CeresDoorCounters {
    pub c_entries: [u64; 13],
    pub rust_entries: [u64; 13],
    pub rust_completed: [u64; 13],
    pub rust_refused: [u64; 13],
    pub foreign_requests: u64,
    pub foreign_completions: u64,
    /// Actual proximity instruction returns taking its fallthrough branch.
    pub proximity_successes: [u64; 32],
    /// Completed BA48 setup invocations with all four block types solid.
    pub solid_setups: u64,
}

type Callback = unsafe extern "C" fn(*mut c_void, u32, u16, u16, *mut u16) -> i32;
type SetHook = unsafe extern "C" fn(Option<Callback>, *mut c_void);
type SpawnEproj = unsafe extern "C" fn(*mut u8, u16, u16, u16, u16) -> i32;
type Hook =
    dyn FnMut(CeresDoorEntry, u16, u16, &mut [u8], &[u8], &mut CeresDoorServices) -> Result<CeresDoorOutcome, String>;

/// Synchronous escape-puff service. Only header E509 and parameters 3/12 are
/// admitted; the complete private WRAM allocation is copied in and back out.
pub struct CeresDoorServices {
    spawn: SpawnEproj,
    expected_ram: *const u8,
    requested: bool,
}

impl CeresDoorServices {
    pub fn spawn_eproj_with_room_gfx(
        &mut self,
        ram: &mut [u8],
        header: u16,
        parameter: u16,
        point: [u16; 2],
    ) -> Result<(), String> {
        self.requested = true;
        if ram.len() != WRAM_SIZE || ram.as_ptr() != self.expected_ram {
            return Err("Ceres puff spawn requires this invocation's complete WRAM copy".into());
        }
        // SAFETY: native validation precedes synchronous execution, and native
        // code receives only the private allocation owned by this invocation.
        let status = unsafe { (self.spawn)(ram.as_mut_ptr(), header, parameter, point[0], point[1]) };
        if status != 0 {
            return Err(format!("Ceres puff spawn unsupported invocation: {status}"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct HookFunctions {
    pub set: SetHook,
    pub commit: Commit,
    pub generation: Generation,
    pub spawn: SpawnEproj,
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
            CeresDoorEntry,
            u16,
            u16,
            &mut [u8],
            &[u8],
            &mut CeresDoorServices,
        ) -> Result<CeresDoorOutcome, String>
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

fn decode_entry(entry: u32) -> Result<CeresDoorEntry, String> {
    use CeresDoorEntry::*;
    [Init, Main, Instr1, Instr2, Instr3, Instr4, Instr5, Instr6, Instr7, Instr8, SetTangible, SetInvisible, SolidSetup]
        .get(entry as usize)
        .copied()
        .ok_or_else(|| "invalid native Ceres door entry".into())
}

fn encode_output(entry: CeresDoorEntry, output: CeresDoorOutput) -> Result<u16, String> {
    match (entry, output) {
        (CeresDoorEntry::Init | CeresDoorEntry::Main, CeresDoorOutput::Done) => Ok(0),
        (CeresDoorEntry::SolidSetup, CeresDoorOutput::SetupStatus(status)) => Ok(status.into()),
        (entry, CeresDoorOutput::InstructionOffset(offset))
            if (2..12).contains(&(entry as u32)) && offset >= 0x8000 && offset & 1 == 0 =>
        {
            Ok(offset)
        }
        _ => Err("Ceres door output does not match scheduled entry or canonical A6 instruction offset".into()),
    }
}

pub(crate) unsafe extern "C" fn trampoline(
    context: *mut c_void,
    entry: u32,
    index: u16,
    instruction: u16,
    output: *mut u16,
) -> i32 {
    // SAFETY: Game owns this stable box. Native adapters reject nested callbacks
    // across all consumers before constructing another mutable borrow.
    let registration = unsafe { &mut *context.cast::<Registration>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<CeresDoorOutcome, String> {
        let entry = decode_entry(entry)?;
        registration.transaction.begin();
        // SAFETY: authenticated native generation and cartridge storage remain
        // stable during the copied-state callback.
        unsafe {
            let generation = (registration.functions.generation)();
            if registration.rom_generation != Some(generation) {
                registration.rom_copy =
                    std::slice::from_raw_parts((registration.rom)(), (registration.rom_size)()).to_vec();
                registration.rom_generation = Some(generation);
            }
        }
        let mut services = CeresDoorServices {
            spawn: registration.functions.spawn,
            expected_ram: registration.transaction.scratch.as_ptr(),
            requested: false,
        };
        let outcome = (registration.callback)(
            entry,
            index,
            instruction,
            &mut registration.transaction.scratch,
            &registration.rom_copy,
            &mut services,
        )?;
        let (ownership, encoded) = match outcome {
            CeresDoorOutcome::Handled(value) => (HookOutcome::Handled, Some(encode_output(entry, value)?)),
            CeresDoorOutcome::Refused => (HookOutcome::Refused, None),
        };
        registration.transaction.finish(ownership, services.requested, "Ceres door")?;
        if let Some(value) = encoded {
            // SAFETY: native caller supplies a live u16 for the synchronous call.
            unsafe { *output = value };
        }
        Ok(outcome)
    }));
    match result {
        Ok(Ok(CeresDoorOutcome::Handled(_))) => 1,
        Ok(Ok(CeresDoorOutcome::Refused)) => 0,
        Ok(Err(error)) => {
            eprintln!("Rust Ceres door failure: {error}");
            -1
        }
        Err(_) => {
            eprintln!("Rust Ceres door panic: C retry prohibited");
            -1
        }
    }
}

pub(crate) fn counters(function: Counts) -> CeresDoorCounters {
    let mut values = [0; 87];
    // SAFETY: authenticated C shim writes exactly 87 u64 counters.
    unsafe { function(values.as_mut_ptr()) };
    CeresDoorCounters {
        c_entries: values[0..13].try_into().unwrap(),
        rust_entries: values[13..26].try_into().unwrap(),
        rust_completed: values[26..39].try_into().unwrap(),
        rust_refused: values[39..52].try_into().unwrap(),
        foreign_requests: values[52],
        foreign_completions: values[53],
        proximity_successes: values[54..86].try_into().unwrap(),
        solid_setups: values[86],
    }
}
