#[cfg(feature = "legacy-evidence")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum ItemInstructionEntry {
    Delete = 0,
    PreInstr = 1,
    ClearPreInstr = 2,
    Goto = 3,
    SetTimer = 4,
    GotoIfItemBitSet = 5,
    SetItemBit = 6,
    CollectAmmoMissileTank = 7,
    SetLinkReg = 8,
    Call = 9,
    Return = 10,
    ClearMusicQueueAndQueueTrack = 11,
    GotoLinkIfTriggered = 12,
    LoadItemPlmGfx = 13,
    DrawItemFrame0 = 14,
    DrawItemFrame1 = 15,
    PickupEquipmentAndShowMessage = 16,
    CollectHealthEnergyTank = 17,
    CollectHealthReserveTank = 18,
    CollectAmmoSuperMissileTank = 19,
    CollectAmmoPowerBombTank = 20,
}

impl ItemInstructionEntry {
    pub const ALL: [Self; 21] = [
        Self::Delete,
        Self::PreInstr,
        Self::ClearPreInstr,
        Self::Goto,
        Self::SetTimer,
        Self::GotoIfItemBitSet,
        Self::SetItemBit,
        Self::CollectAmmoMissileTank,
        Self::SetLinkReg,
        Self::Call,
        Self::Return,
        Self::ClearMusicQueueAndQueueTrack,
        Self::GotoLinkIfTriggered,
        Self::LoadItemPlmGfx,
        Self::DrawItemFrame0,
        Self::DrawItemFrame1,
        Self::PickupEquipmentAndShowMessage,
        Self::CollectHealthEnergyTank,
        Self::CollectHealthReserveTank,
        Self::CollectAmmoSuperMissileTank,
        Self::CollectAmmoPowerBombTank,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemInstructionOutput {
    PreinstructionDone,
    Stop,
    Continue(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemInstructionOutcome {
    Handled(ItemInstructionOutput),
    Refused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ItemInstructionService {
    AddMissilesToHud = 0,
    PlayRoomMusicAfter = 1,
    QueueMusicDelayed8 = 2,
    ProcessPlmDrawInstruction = 3,
    CalculatePlmBlockCoords = 4,
    DrawPlm = 5,
    AddSuperMissilesToHud = 6,
    AddPowerBombsToHud = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ItemInstructionPhase {
    InstructionBegin = 0,
    InstructionEnd = 1,
    ServiceBegin = 2,
    ServiceEnd = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemInstructionObservation {
    pub phase: ItemInstructionPhase,
    pub entry: Option<ItemInstructionEntry>,
    pub slot: u16,
    /// Operand, packed output, or service ID in high16 and argument in low16.
    pub value: u32,
    /// Zero standalone; one inside the actual room-transition service52.
    pub context: u32,
    pub scope_identity: bool,
    /// Zero foreign, one Chozo missile, two Morph Ball, three exposed tanks.
    pub scope_id: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct ItemInstructionEntryCounters {
    pub entries: [u64; 21],
    pub c_entries: [u64; 21],
    pub rust_entries: [u64; 21],
    pub rust_completed: [u64; 21],
    pub rust_refused: [u64; 21],
}

/// The eight identity/context groups partition `total`. Identity is captured
/// before execution (including Delete), not recomputed from committed output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct ItemInstructionCounters {
    pub total: ItemInstructionEntryCounters,
    pub scope_standalone: ItemInstructionEntryCounters,
    pub scope_service52: ItemInstructionEntryCounters,
    pub other_standalone: ItemInstructionEntryCounters,
    pub other_service52: ItemInstructionEntryCounters,
    pub morph_standalone: ItemInstructionEntryCounters,
    pub morph_service52: ItemInstructionEntryCounters,
    pub tank_standalone: ItemInstructionEntryCounters,
    pub tank_service52: ItemInstructionEntryCounters,
    pub service_requests: [u64; 8],
    pub service_completions: [u64; 8],
}
