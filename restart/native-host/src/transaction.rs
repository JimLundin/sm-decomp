//! Shared copied-WRAM ownership contract for synchronous gameplay callbacks.
use crate::{Bytes, HookOutcome};

pub(crate) type Commit = unsafe extern "C" fn(*const u8);
pub(crate) const WRAM_SIZE: usize = 0x20000;

pub(crate) struct Transaction {
    ram: Bytes,
    commit: Commit,
    pub scratch: Box<[u8]>,
}

impl Transaction {
    pub fn new(ram: Bytes, commit: Commit) -> Self {
        Self { ram, commit, scratch: vec![0; WRAM_SIZE].into_boxed_slice() }
    }

    pub fn begin(&mut self) {
        // SAFETY: the retained native singleton owns complete canonical WRAM.
        // Dispatch excludes concurrent native execution. An admitted transition
        // service may synchronously enter a distinct child's registration after
        // publishing the parent scratch bytes to canonical WRAM.
        unsafe {
            self.scratch.copy_from_slice(std::slice::from_raw_parts((self.ram)(), WRAM_SIZE));
        }
    }

    pub fn finish(&self, outcome: HookOutcome, requested: bool, consumer: &str) -> Result<(), String> {
        match outcome {
            HookOutcome::Handled => {
                // SAFETY: the callback's mutable WRAM borrow has ended. Commit
                // only the complete private copy of a successful invocation.
                unsafe { (self.commit)(self.scratch.as_ptr()) };
            }
            HookOutcome::Refused => {
                if requested {
                    return Err(format!("{consumer} refusal after foreign request"));
                }
                // With no service request canonical RAM remains the input.
                let original = unsafe { std::slice::from_raw_parts((self.ram)(), WRAM_SIZE) };
                if original != self.scratch.as_ref() {
                    return Err(format!("{consumer} refusal changed private WRAM"));
                }
            }
        }
        Ok(())
    }
}
