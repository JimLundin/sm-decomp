use crate::transaction::{Commit, Transaction};
use crate::{Bytes, Counts, Size};
#[cfg(feature = "legacy-evidence")]
use serde::{Deserialize, Serialize};
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum ProjectileEntry {
    FireUnchargedBeam = 0,
    HandleProjectile = 1,
    DrawPlayerExplosions2 = 2,
    DrawBombAndProjectileExplosions = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookOutcome {
    Handled,
    Refused,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct ProjectileCounters {
    pub c_entries: [u64; 4],
    pub rust_entries: [u64; 4],
    pub rust_completed: [u64; 4],
    pub rust_refused: [u64; 4],
    pub foreign_calls: u64,
    /// Actual four-byte OAM entries emitted by the two draw operations while
    /// an ordinary projectile/explosion slot was active on entry.
    pub drawn_oam_entries: [u64; 4],
    /// Entry-time Ceres high-bit context, regardless of owner.
    pub ceres_high_bit_entries: [u64; 4],
    pub ceres_high_bit_completed: [u64; 4],
    /// Fire/draw refusals with only the retained high-bit exclusion present.
    /// All other inventory/slot guards were clear at entry.
    pub ceres_high_bit_refused: [u64; 4],
    pub other_refused: [u64; 4],
}

pub(crate) type Callback = unsafe extern "C" fn(*mut c_void, u32) -> i32;
pub(crate) type SetHook = unsafe extern "C" fn(Option<Callback>, *mut c_void);
pub(crate) type Generation = unsafe extern "C" fn() -> u64;
pub(crate) type SpawnPlm = unsafe extern "C" fn(*mut u8, u16) -> i32;
type Hook = dyn FnMut(ProjectileEntry, &mut [u8], &[u8], &mut ProjectileServices) -> Result<HookOutcome, String>;

/// The sole admitted synchronous foreign gameplay service. It exchanges copied
/// WRAM with C; C never receives references into Rust-owned gameplay state.
pub struct ProjectileServices {
    spawn: SpawnPlm,
    expected_ram: *const u8,
    requested: bool,
}

impl ProjectileServices {
    pub fn spawn_plm(&mut self, ram: &mut [u8], header: u16) -> Result<u8, String> {
        self.requested = true;
        if ram.len() != 0x20000 || ram.as_ptr() != self.expected_ram {
            return Err("SpawnPLM requires this invocation's complete WRAM copy".into());
        }
        // SAFETY: the reviewed shim copies from this private WRAM into canonical
        // C memory, executes synchronously, then copies back after C returns.
        let result = unsafe { (self.spawn)(ram.as_mut_ptr(), header) };
        if result < 0 {
            return Err(format!("SpawnPLM unsupported invocation context: {result}"));
        }
        u8::try_from(result).map_err(|_| "SpawnPLM result out of range".into())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct HookFunctions {
    pub set: SetHook,
    pub commit: Commit,
    pub generation: Generation,
    pub spawn: SpawnPlm,
    pub counters: Counts,
}

pub(crate) struct Registration {
    callback: Box<Hook>,
    rom: Bytes,
    rom_size: Size,
    functions: HookFunctions,
    transaction: Transaction,
    rom_copy: Vec<u8>,
    rom_generation: Option<u64>,
}

impl Registration {
    pub fn new(
        callback: impl FnMut(ProjectileEntry, &mut [u8], &[u8], &mut ProjectileServices) -> Result<HookOutcome, String>
        + 'static,
        ram: Bytes,
        rom: Bytes,
        rom_size: Size,
        functions: HookFunctions,
    ) -> Box<Self> {
        Box::new(Self {
            callback: Box::new(callback),
            rom,
            rom_size,
            functions,
            transaction: Transaction::new(ram, functions.commit),
            rom_copy: Vec::new(),
            rom_generation: None,
        })
    }
}

pub(crate) unsafe extern "C" fn trampoline(context: *mut c_void, entry: u32) -> i32 {
    // SAFETY: Game retains the stable boxed registration until it unregisters
    // this callback. Native dispatch is synchronous and rejects nested calls.
    let registration = unsafe { &mut *context.cast::<Registration>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<HookOutcome, String> {
        let entry = match entry {
            0 => ProjectileEntry::FireUnchargedBeam,
            1 => ProjectileEntry::HandleProjectile,
            2 => ProjectileEntry::DrawPlayerExplosions2,
            3 => ProjectileEntry::DrawBombAndProjectileExplosions,
            _ => return Err("invalid native projectile entry".into()),
        };
        // SAFETY: buffers and ROM generation belong to the retained native
        // singleton. Copy them before handing owned state to candidate Rust.
        registration.transaction.begin();
        unsafe {
            let generation = (registration.functions.generation)();
            if registration.rom_generation != Some(generation) {
                registration.rom_copy =
                    std::slice::from_raw_parts((registration.rom)(), (registration.rom_size)()).to_vec();
                registration.rom_generation = Some(generation);
            }
        }
        let mut services = ProjectileServices {
            spawn: registration.functions.spawn,
            expected_ram: registration.transaction.scratch.as_ptr(),
            requested: false,
        };
        let outcome = (registration.callback)(
            entry,
            &mut registration.transaction.scratch,
            &registration.rom_copy,
            &mut services,
        )?;
        registration.transaction.finish(outcome, services.requested, "projectile")?;
        Ok(outcome)
    }));
    match result {
        Ok(Ok(HookOutcome::Handled)) => 1,
        Ok(Ok(HookOutcome::Refused)) => 0,
        Ok(Err(error)) => {
            eprintln!("Rust projectile failure: {error}");
            -1
        }
        Err(_) => {
            eprintln!("Rust projectile panic: C retry prohibited");
            -1
        }
    }
}

pub(crate) fn counters(function: Counts) -> ProjectileCounters {
    let mut values = [0; 37];
    // SAFETY: shim writes exactly the documented 37 counters.
    unsafe {
        function(values.as_mut_ptr());
    }
    ProjectileCounters {
        c_entries: values[0..4].try_into().unwrap(),
        rust_entries: values[4..8].try_into().unwrap(),
        rust_completed: values[8..12].try_into().unwrap(),
        rust_refused: values[12..16].try_into().unwrap(),
        foreign_calls: values[16],
        drawn_oam_entries: values[17..21].try_into().unwrap(),
        ceres_high_bit_entries: values[21..25].try_into().unwrap(),
        ceres_high_bit_completed: values[25..29].try_into().unwrap(),
        ceres_high_bit_refused: values[29..33].try_into().unwrap(),
        other_refused: values[33..37].try_into().unwrap(),
    }
}
