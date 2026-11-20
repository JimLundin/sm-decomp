//! Origin-only borrowed owner facts retained by the frame caller.
//! Producer effects and later results require separate actual receipts.
use canonical::Canonical;
use data::timing::ChannelSlot;

/// Single physical colour component selected by the current Haze owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColourComponent {
    /// Red selection.
    Red,
    /// Green selection.
    Green,
    /// Blue selection.
    Blue,
}
/// Actual ordered Haze slot, without executing its pre-step or transfer program.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum HazeSlot {
    /// Empty current slot.
    Empty,
    /// Current `BackdropRamp` object inputs.
    BackdropRamp {
        /// Physical channel held by this object.
        channel: ChannelSlot,
        /// Current countdown before the owner's operation.
        countdown: u16,
        /// Current maximum before band generation.
        maximum: u16,
        /// Actual single colour component.
        component: ColourComponent,
    },
    /// Current `KeepTransfer` object inputs.
    KeepTransfer {
        /// Physical channel held by this object.
        channel: ChannelSlot,
        /// Current countdown before the owner's operation.
        countdown: u16,
    },
}
/// Initial default layer-blend configuration, before the Haze owner's later result.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DefaultLayerBlend(pub u16);
/// Current Haze preparation inputs, not generated bands or prepared video outputs.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct HazeOrigin {
    /// Simulation is running at the qualified origin.
    pub running: bool,
    /// No active power bomb at the qualified origin.
    pub power_bomb_inactive: bool,
    /// HDMA owner enable at the qualified origin.
    pub enabled: bool,
    /// Default owner setting read before any current Haze result.
    pub default_blend: DefaultLayerBlend,
    /// Actual object order with current inputs, not a callback schedule.
    pub slots: [HazeSlot; 6],
}
/// Current builder backing and continuation before `BeginSpriteList` occurs.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct SpriteBuilderOrigin {
    /// All current low and high backing bytes; nothing is cleared here.
    pub backing: [u8; 544],
    /// Current byte cursor, preserved before the owner's reset operation.
    pub cursor: u16,
    /// Current prepared transfer flags, preserved before the owner's reset.
    pub prepared_flags: u16,
    /// Current top-half transfer definition.
    pub prepared_top: u16,
    /// Current bottom-half transfer definition.
    pub prepared_bottom: u16,
}
/// Actual raw slot's semantic eligibility status; no membership list is generated.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorSlot {
    /// No actor in this slot.
    Empty,
    /// Respawning placeholder, not an occupied actor.
    Placeholder,
    /// Current occupied actor properties used by eligibility.
    Occupied {
        /// Current deletion flag.
        deleted: bool,
        /// Current marked offscreen processing flag.
        process_offscreen: bool,
        /// Current noninteractive property.
        intangible: bool,
    },
}
/// Current raw eligibility inputs; positions and future boxes are separate inputs.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct ActorEligibilityOrigin {
    /// Qualified policy permits actors marked for offscreen processing.
    pub permit_marked_offscreen: bool,
    /// Physical slot identities are retained by this complete array's order.
    pub slots: [ActorSlot; 32],
}
/// Complete initial borrowed prelude facts with no generated owner results.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct PreludeSeed {
    /// Current Haze owner inputs.
    pub haze: HazeOrigin,
    /// Current complete sprite-builder owner.
    pub builder: SpriteBuilderOrigin,
    /// Current raw actor eligibility inputs.
    pub actors: ActorEligibilityOrigin,
}
