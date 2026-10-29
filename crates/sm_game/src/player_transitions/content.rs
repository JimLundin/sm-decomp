//! Declaration-only design; no implementation or gameplay stubs.
#![forbid(unsafe_code)]
use crate::player_collision::effects::Content as CollisionContent;

/// Fixed tables only; compatibility locations live exclusively in the adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionTable {
    InputPrograms,
    BlockFallPoses,
    BlockLandPoses,
    StandingTurn,
    CrouchingTurn,
    JumpingTurn,
    FallingTurn,
    MoonwalkJumpTurn,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationTable {
    PosePrograms,
    RunningWithoutBooster,
    RunningBoosterStages,
    BoosterStageDurations,
}
/// Imported animation program identity. Not a mutable-memory address or owner.
/// The compatibility provider resolves this bank-local identity; prepared content
/// may use the same finite identity domain without any ROM/backend module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationProgram(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandingRegion {
    Clear,
    Dust,
    WaterSpray,
    Crateria,
    Brinstar,
    Tourian,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandingRule {
    Clear,
    Dust,
    WaterSpray,
    WaterSprayWhenFxTen,
    WaterSprayAtOrBelow944,
}

/// Native pose_table is inherited from the actual collision Content trait.
/// All returned records implement the existing visual_host::Record contract.
/// Captures share allocation identity, retain live bytes, and never retain a
/// borrow of the host. No eager frame/operand/table copies are permitted.
pub trait Content: CollisionContent {
    /// Loaded pose metadata for bottom alignment and pose top/bottom queries.
    /// Capture each current/previous lookup at its original point; normalization
    /// and animation-reset radius reads still use the inherited Native table.
    fn loaded_pose_table(&mut self) -> Self::Record;
    /// Block tables use Loaded; turn tables use Native. No low-window diagnostic.
    /// Capture BEFORE evaluating any index that itself reads content or state.
    fn transition_table(&mut self, table: TransitionTable) -> Self::Record;
    /// Loaded allocation capture; caller chooses native_word for reset's old
    /// rom_table_word or whole-span bytes::<2> + LE for read_rom_word sites.
    fn animation_table(&mut self, table: AnimationTable) -> Self::Record;
    /// Apply wrapping bank-local advance BEFORE low-window diagnostic, then
    /// capture the current Native allocation AFTER that diagnostic. Advance is
    /// zero for interpreter lists; reset uses animation_skip. Reads within the
    /// returned record use LINEAR offsets and do not wrap again at bank edges.
    fn animation_program(&mut self, program: AnimationProgram, advance: u16) -> Self::Record;
    /// Resolve CURRENT installed area/content identity at the original landing
    /// dispatch point, after prior sounds. Preserve invalid-selector failure.
    fn landing_region(&self) -> LandingRegion;
    /// Resolve CURRENT room identity into authored landing policy. For Crateria,
    /// core calls this ONLY after cinematic==0, preserving skipped table reads.
    /// Compatibility provider preserves signed-room checks and table failure;
    /// future authored mappings attach rules to stable room identity.
    fn landing_room_rule(&self, region: LandingRegion) -> LandingRule;
}
