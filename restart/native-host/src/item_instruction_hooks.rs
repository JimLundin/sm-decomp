use crate::hooks::Generation;
use crate::item_instruction_types::*;
use crate::transaction::{Commit, Transaction, WRAM_SIZE};
use crate::{Bytes, Counts, HookOutcome, Size};
use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};

type Callback = unsafe extern "C" fn(*mut c_void, u32, u16, u16, *mut u32) -> i32;
type SetHook = unsafe extern "C" fn(Option<Callback>, *mut c_void);
type ObserverCallback = unsafe extern "C" fn(*mut c_void, u32, u32, u16, u32, u32, u32, *const u8) -> i32;
type SetObserver = unsafe extern "C" fn(Option<ObserverCallback>, *mut c_void);
type Service = unsafe extern "C" fn(*mut u8, u32, u16) -> i32;
type Hook = dyn FnMut(
    ItemInstructionEntry,
    u16,
    Option<u16>,
    &mut [u8],
    &[u8],
    &mut ItemInstructionServices,
) -> Result<ItemInstructionOutcome, String>;

pub struct ItemInstructionServices {
    service: Service,
    expected_ram: *const u8,
    entry: ItemInstructionEntry,
    requested: bool,
}

impl ItemInstructionServices {
    pub fn available(&self, service: ItemInstructionService) -> bool {
        matches!(
            (self.entry, service),
            (
                ItemInstructionEntry::CollectAmmoMissileTank,
                ItemInstructionService::AddMissilesToHud | ItemInstructionService::PlayRoomMusicAfter
            ) | (ItemInstructionEntry::ClearMusicQueueAndQueueTrack, ItemInstructionService::QueueMusicDelayed8)
                | (ItemInstructionEntry::PickupEquipmentAndShowMessage, ItemInstructionService::PlayRoomMusicAfter)
                | (
                    ItemInstructionEntry::CollectHealthEnergyTank
                        | ItemInstructionEntry::CollectHealthReserveTank
                        | ItemInstructionEntry::CollectAmmoSuperMissileTank
                        | ItemInstructionEntry::CollectAmmoPowerBombTank,
                    ItemInstructionService::PlayRoomMusicAfter
                )
                | (ItemInstructionEntry::CollectAmmoSuperMissileTank, ItemInstructionService::AddSuperMissilesToHud)
                | (ItemInstructionEntry::CollectAmmoPowerBombTank, ItemInstructionService::AddPowerBombsToHud)
                | (
                    ItemInstructionEntry::DrawItemFrame0 | ItemInstructionEntry::DrawItemFrame1,
                    ItemInstructionService::ProcessPlmDrawInstruction
                        | ItemInstructionService::CalculatePlmBlockCoords
                        | ItemInstructionService::DrawPlm
                )
        )
    }
    fn call(&mut self, ram: &mut [u8], service: ItemInstructionService, argument: u16) -> Result<(), String> {
        self.requested = true;
        if ram.len() != WRAM_SIZE || ram.as_ptr() != self.expected_ram {
            return Err("item service requires this invocation's complete private WRAM".into());
        }
        if !self.available(service) {
            return Err("item service is unavailable for this entry".into());
        }
        // SAFETY: this is the live private transaction allocation; the native
        // service validates its active entry and arguments before copying it.
        let status = unsafe { (self.service)(ram.as_mut_ptr(), service as u32, argument) };
        if status != 0 {
            return Err(format!("item native service failed: {status}"));
        }
        Ok(())
    }
    pub fn add_missiles_to_hud(&mut self, ram: &mut [u8]) -> Result<(), String> {
        self.call(ram, ItemInstructionService::AddMissilesToHud, 0)
    }
    pub fn play_room_music_after(&mut self, ram: &mut [u8], frames: u16) -> Result<(), String> {
        self.call(ram, ItemInstructionService::PlayRoomMusicAfter, frames)
    }
    pub fn queue_music_delayed8(&mut self, ram: &mut [u8], track: u16) -> Result<(), String> {
        self.call(ram, ItemInstructionService::QueueMusicDelayed8, track)
    }
    pub fn process_plm_draw_instruction(&mut self, ram: &mut [u8], slot: u16) -> Result<(), String> {
        self.call(ram, ItemInstructionService::ProcessPlmDrawInstruction, slot)
    }
    pub fn calculate_plm_block_coords(&mut self, ram: &mut [u8], slot: u16) -> Result<(), String> {
        self.call(ram, ItemInstructionService::CalculatePlmBlockCoords, slot)
    }
    pub fn draw_plm(&mut self, ram: &mut [u8], slot: u16) -> Result<(), String> {
        self.call(ram, ItemInstructionService::DrawPlm, slot)
    }
    pub fn add_super_missiles_to_hud(&mut self, ram: &mut [u8]) -> Result<(), String> {
        self.call(ram, ItemInstructionService::AddSuperMissilesToHud, 0)
    }
    pub fn add_power_bombs_to_hud(&mut self, ram: &mut [u8]) -> Result<(), String> {
        self.call(ram, ItemInstructionService::AddPowerBombsToHud, 0)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct HookFunctions {
    pub set: SetHook,
    pub observer_set: SetObserver,
    pub commit: Commit,
    pub generation: Generation,
    pub service: Service,
    pub counters: Counts,
}

type Observer = dyn FnMut(ItemInstructionObservation, &[u8]) -> Result<(), String>;
pub(crate) struct ObserverRegistration {
    callback: Box<Observer>,
}
impl ObserverRegistration {
    pub fn new(callback: impl FnMut(ItemInstructionObservation, &[u8]) -> Result<(), String> + 'static) -> Box<Self> {
        Box::new(Self { callback: Box::new(callback) })
    }
}

pub(crate) unsafe extern "C" fn observer_trampoline(
    context: *mut c_void,
    phase: u32,
    entry: u32,
    slot: u16,
    value: u32,
    parent: u32,
    scope: u32,
    ram: *const u8,
) -> i32 {
    // SAFETY: this is a separately owned observer box, never the gameplay box.
    // Native excludes observer recursion and keeps canonical WRAM stable here.
    let registration = unsafe { &mut *context.cast::<ObserverRegistration>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), String> {
        let phase = [
            ItemInstructionPhase::InstructionBegin,
            ItemInstructionPhase::InstructionEnd,
            ItemInstructionPhase::ServiceBegin,
            ItemInstructionPhase::ServiceEnd,
        ]
        .get(phase as usize)
        .copied()
        .ok_or("invalid item observation phase")?;
        let entry = if entry == u32::MAX {
            None
        } else {
            Some(*ItemInstructionEntry::ALL.get(entry as usize).ok_or("invalid observed item entry")?)
        };
        if parent > 1 || scope > 3 || ram.is_null() {
            return Err("invalid item observation context".into());
        }
        let observation = ItemInstructionObservation {
            phase,
            entry,
            slot,
            value,
            context: parent,
            scope_identity: scope != 0,
            scope_id: scope,
        };
        (registration.callback)(observation, unsafe { std::slice::from_raw_parts(ram, WRAM_SIZE) })
    }));
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => {
            eprintln!("Rust item observer failure: {error}");
            -1
        }
        Err(_) => {
            eprintln!("Rust item observer panic: evidence incomplete");
            -1
        }
    }
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
            ItemInstructionEntry,
            u16,
            Option<u16>,
            &mut [u8],
            &[u8],
            &mut ItemInstructionServices,
        ) -> Result<ItemInstructionOutcome, String>
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

fn encode(entry: ItemInstructionEntry, output: ItemInstructionOutput) -> Result<u32, String> {
    match (entry == ItemInstructionEntry::GotoLinkIfTriggered, output) {
        (true, ItemInstructionOutput::PreinstructionDone) => Ok(0),
        (false, ItemInstructionOutput::Stop) => Ok(1 << 16),
        (false, ItemInstructionOutput::Continue(offset)) if offset >= 0x8000 => Ok(2 << 16 | u32::from(offset)),
        _ => Err("item output tag or bank84 offset does not match entry".into()),
    }
}

pub(crate) unsafe extern "C" fn trampoline(
    context: *mut c_void,
    entry: u32,
    slot: u16,
    operand: u16,
    output: *mut u32,
) -> i32 {
    // SAFETY: native guards exclude overlapping gameplay registrations before
    // entering this callback; Game owns the stable box for its whole lifetime.
    let registration = unsafe { &mut *context.cast::<Registration>() };
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<ItemInstructionOutcome, String> {
        let entry = ItemInstructionEntry::ALL.get(entry as usize).copied().ok_or("invalid item entry")?;
        registration.transaction.begin();
        // SAFETY: native cartridge storage remains valid during the callback.
        unsafe {
            let generation = (registration.functions.generation)();
            if registration.rom_generation != Some(generation) {
                registration.rom_copy =
                    std::slice::from_raw_parts((registration.rom)(), (registration.rom_size)()).to_vec();
                registration.rom_generation = Some(generation);
            }
        }
        let mut services = ItemInstructionServices {
            service: registration.functions.service,
            expected_ram: registration.transaction.scratch.as_ptr(),
            entry,
            requested: false,
        };
        let operand = (entry != ItemInstructionEntry::GotoLinkIfTriggered).then_some(operand);
        let outcome = (registration.callback)(
            entry,
            slot,
            operand,
            &mut registration.transaction.scratch,
            &registration.rom_copy,
            &mut services,
        )?;
        let (ownership, encoded) = match outcome {
            ItemInstructionOutcome::Handled(value) => (HookOutcome::Handled, Some(encode(entry, value)?)),
            ItemInstructionOutcome::Refused => (HookOutcome::Refused, None),
        };
        // Encode before committing: malformed handled outputs never commit.
        registration.transaction.finish(ownership, services.requested, "item instruction")?;
        if let Some(value) = encoded {
            unsafe { *output = value };
        }
        Ok(outcome)
    }));
    match result {
        Ok(Ok(ItemInstructionOutcome::Handled(_))) => 1,
        Ok(Ok(ItemInstructionOutcome::Refused)) => 0,
        Ok(Err(error)) => {
            eprintln!("Rust item instruction failure: {error}");
            -1
        }
        Err(_) => {
            eprintln!("Rust item instruction panic: C retry prohibited");
            -1
        }
    }
}

pub(crate) fn counters(function: Counts) -> ItemInstructionCounters {
    let mut values = [0; 961];
    // SAFETY: the frozen C transport writes exactly 961 counters.
    unsafe { function(values.as_mut_ptr()) };
    let group = |at| ItemInstructionEntryCounters {
        entries: values[at..at + 21].try_into().unwrap(),
        c_entries: values[at + 21..at + 42].try_into().unwrap(),
        rust_entries: values[at + 42..at + 63].try_into().unwrap(),
        rust_completed: values[at + 63..at + 84].try_into().unwrap(),
        rust_refused: values[at + 84..at + 105].try_into().unwrap(),
    };
    ItemInstructionCounters {
        total: group(0),
        scope_standalone: group(105),
        scope_service52: group(210),
        other_standalone: group(315),
        other_service52: group(420),
        morph_standalone: group(525),
        morph_service52: group(630),
        tank_standalone: group(735),
        tank_service52: group(840),
        service_requests: values[945..953].try_into().unwrap(),
        service_completions: values[953..961].try_into().unwrap(),
    }
}
