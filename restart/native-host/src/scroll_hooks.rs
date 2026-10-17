use crate::hooks::SetHook;
use crate::transaction::{Commit, Transaction, WRAM_SIZE};
use crate::{Bytes, Counts, HookOutcome};
#[cfg(feature = "legacy-evidence")]
use serde::{Deserialize, Serialize};
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum ScrollEntry {
    DisplayViewablePartOfRoom = 0,
    UpdateScrollVarsUpdateMap = 1,
    CalculateLayer2PosAndScrollsWhenScrolling = 2,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct ScrollCounters {
    pub c_entries: [u64; 3],
    pub rust_entries: [u64; 3],
    pub rust_completed: [u64; 3],
    pub rust_refused: [u64; 3],
    pub foreign_requests: u64,
    pub foreign_completions: u64,
    /// Actual flushes executed synchronously inside each scheduled entry.
    pub flush_requests: [u64; 3],
    pub flush_completions: [u64; 3],
    /// Consumed BG1/BG2 enables inside each entry's synchronous flushes.
    pub columns_flushed: [[u64; 2]; 3],
    pub rows_flushed: [[u64; 2]; 3],
    pub entry_vram_dma_bytes: [u64; 3],
    /// All actual queue flushes, including later NMI execution outside entries.
    pub total_flush_requests: u64,
    pub total_flush_completions: u64,
    pub consumed_columns: [u64; 2],
    pub consumed_rows: [u64; 2],
    /// Bytes actually written by DMA to VRAM ports during any queue flush.
    pub vram_dma_bytes: u64,
}

pub(crate) type FlushVram = unsafe extern "C" fn(*mut u8) -> i32;
type Hook = dyn FnMut(ScrollEntry, &mut [u8], &mut ScrollServices) -> Result<HookOutcome, String>;

/// Synchronous hardware queue service; WRAM changes are copied back before
/// returning, alongside real native PPU/DMA and ordered register effects.
pub struct ScrollServices {
    flush: FlushVram,
    expected_ram: *const u8,
    requested: bool,
}

impl ScrollServices {
    pub fn flush_vram(&mut self, ram: &mut [u8]) -> Result<(), String> {
        self.requested = true;
        if ram.len() != WRAM_SIZE || ram.as_ptr() != self.expected_ram {
            return Err("VRAM flush requires this invocation's complete WRAM copy".into());
        }
        // SAFETY: native service copies full WRAM, runs the real queue flush
        // synchronously, and copies its resulting WRAM back to this allocation.
        let status = unsafe { (self.flush)(ram.as_mut_ptr()) };
        if status != 0 {
            return Err(format!("VRAM flush unsupported invocation context: {status}"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct HookFunctions {
    pub set: SetHook,
    pub commit: Commit,
    pub flush: FlushVram,
    pub counters: Counts,
}

pub(crate) struct Registration {
    callback: Box<Hook>,
    transaction: Transaction,
    flush: FlushVram,
}

impl Registration {
    pub fn new(
        callback: impl FnMut(ScrollEntry, &mut [u8], &mut ScrollServices) -> Result<HookOutcome, String> + 'static,
        ram: Bytes,
        functions: HookFunctions,
    ) -> Box<Self> {
        Box::new(Self {
            callback: Box::new(callback),
            transaction: Transaction::new(ram, functions.commit),
            flush: functions.flush,
        })
    }
}

pub(crate) unsafe extern "C" fn trampoline(context: *mut c_void, entry: u32) -> i32 {
    // SAFETY: Game owns the stable boxed registration; C prohibits nested
    // consumer callbacks, including cross-consumer service reentry.
    let registration = unsafe { &mut *context.cast::<Registration>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<HookOutcome, String> {
        let entry = match entry {
            0 => ScrollEntry::DisplayViewablePartOfRoom,
            1 => ScrollEntry::UpdateScrollVarsUpdateMap,
            2 => ScrollEntry::CalculateLayer2PosAndScrollsWhenScrolling,
            _ => return Err("invalid native scroll entry".into()),
        };
        registration.transaction.begin();
        let mut services = ScrollServices {
            flush: registration.flush,
            expected_ram: registration.transaction.scratch.as_ptr(),
            requested: false,
        };
        let outcome = (registration.callback)(entry, &mut registration.transaction.scratch, &mut services)?;
        registration.transaction.finish(outcome, services.requested, "scroll")?;
        Ok(outcome)
    }));
    match result {
        Ok(Ok(HookOutcome::Handled)) => 1,
        Ok(Ok(HookOutcome::Refused)) => 0,
        Ok(Err(error)) => {
            eprintln!("Rust scroll failure: {error}");
            -1
        }
        Err(_) => {
            eprintln!("Rust scroll panic: C retry prohibited");
            -1
        }
    }
}

pub(crate) fn counters(function: Counts) -> ScrollCounters {
    let mut values = [0; 42];
    // SAFETY: the authenticated native shim writes exactly 42 u64 counters.
    unsafe { function(values.as_mut_ptr()) };
    let pairs = |at: usize| std::array::from_fn(|i| [values[at + i * 2], values[at + i * 2 + 1]]);
    ScrollCounters {
        c_entries: values[0..3].try_into().unwrap(),
        rust_entries: values[3..6].try_into().unwrap(),
        rust_completed: values[6..9].try_into().unwrap(),
        rust_refused: values[9..12].try_into().unwrap(),
        foreign_requests: values[12],
        foreign_completions: values[13],
        flush_requests: values[14..17].try_into().unwrap(),
        flush_completions: values[17..20].try_into().unwrap(),
        columns_flushed: pairs(20),
        rows_flushed: pairs(26),
        entry_vram_dma_bytes: values[32..35].try_into().unwrap(),
        total_flush_requests: values[35],
        total_flush_completions: values[36],
        consumed_columns: values[37..39].try_into().unwrap(),
        consumed_rows: values[39..41].try_into().unwrap(),
        vram_dma_bytes: values[41],
    }
}
