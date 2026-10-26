//! Content for normal locomotion; compatibility locations stay in the provider.
#![forbid(unsafe_code)]
use crate::game::{
    liquid_surface::LiquidAtHeight, player_collision::effects::Content as CollisionContent,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpeedRecord {
    /// Already selected bank-local record identity, including wrapped movement offset.
    Selected(u16),
    Bomb,
    GrappleAir,
    GrappleWater,
    GrappleLava,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FootstepRegion {
    Default,
    Crateria,
    Sand,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FootstepRule {
    Default,
    Sand,
    SandWhenFxTen,
    SandAtOrBelow944,
}

pub(crate) trait Content: CollisionContent {
    /// Capture Native after the original low-window diagnostic. Read fields lazily:
    /// unused speed fields must not acquire a new checked-read failure.
    fn speed_record(&mut self, profile: SpeedRecord) -> Self::Record;
    /// None retains the current selector; it does not choose the normal table.
    fn liquid_speed_table(&self, liquid: LiquidAtHeight) -> Option<u16>;
    /// Native fixed frame-byte table; no low-window diagnostic.
    fn spin_sound_frames(&self) -> Self::Record;
    /// Preserve the original eight-entry checked area dispatch.
    fn footstep_region(&self) -> FootstepRegion;
    /// Called only for Crateria after cinematic==0. Preserve the signed room gate
    /// and the exact sixteen-entry checked room table (unlike landing's table).
    fn footstep_room_rule(&self) -> FootstepRule;
}
