//! Separately borrowed player preparation effects at their actual completion boundary.

use super::borrowed_gameplay::{GameplayDispatchReceipt, GameplayRoute, SceneController};
use super::borrowed_preparation::{ActorEligibilityReceipt, ActorSlotIdentity};
use crate::production::MovementContent;
use canonical::{Canonical, model::Pinned};
use serde::{Deserialize, Serialize};
use types::{Action, Buttons, Fixed, Item, Pose, Radius, Set};

/// Actual physical pad bits, before configured action translation.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicalButtons(pub u16);

/// Actual configured physical bindings read by ordinary pose input.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputBindings {
    /// Shoot binding.
    pub shoot: PhysicalButtons,
    /// Jump binding.
    pub jump: PhysicalButtons,
    /// Run binding.
    pub run: PhysicalButtons,
    /// Item cancellation binding.
    pub cancel: PhysicalButtons,
    /// Aim-up binding.
    pub aim_up: PhysicalButtons,
    /// Aim-down binding.
    pub aim_down: PhysicalButtons,
}

/// Complete causal current input owner; overwritten pending commands are not inputs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoseInputOrigin {
    /// Actual starting pose, independently decoded through checked pose content.
    pub pose: Pose,
    /// Actual current action owner.
    pub action: Action,
    /// Actual committed held physical buttons.
    pub held: PhysicalButtons,
    /// Actual committed newly pressed physical buttons.
    pub pressed: PhysicalButtons,
    /// Current configured bindings.
    pub bindings: InputBindings,
}

/// A cleared pose command; other command arms require their own qualification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PoseCommand {
    /// No pending command.
    None,
}

/// A cleared bomb-jump direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BombJumpDirection {
    /// No bomb-jump direction.
    None,
}

/// Actual ordinary pose-input completion, separate from the playing pose.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoseInputTerminal {
    /// Actual translated newly pressed actions, not physical pad bits.
    pub pressed_actions: Buttons,
    /// Actual translated held actions.
    pub held_actions: Buttons,
    /// Deferred prospective selection; never substituted for the starting pose.
    pub prospective: Pose,
    /// Actual special pending selection after clearing.
    pub special: Option<Pose>,
    /// Actual super-special pending selection after clearing.
    pub super_special: Option<Pose>,
    /// Actual ordinary, special and super-special commands after clearing.
    pub commands: [PoseCommand; 3],
    /// Checked imported radius of the starting pose.
    pub radius: Radius,
    /// Actual completed bomb-jump direction.
    pub bomb_jump: BombJumpDirection,
}

/// Completed actual pose-input result, leaving prospective placement deferred.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoseInputCompletion {
    /// Fresh actual current dependency consumed by this operation.
    pub dependency: PoseInputOrigin,
    /// Actual produced completion.
    pub terminal: PoseInputTerminal,
}

/// Actual equipment owner consulted by suit-palette preparation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuitPaletteOrigin {
    /// Actual currently equipped items.
    pub equipment: Set<Item>,
}

/// Separately checked suit-palette meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuitPaletteDisposition {
    /// The ordinary power suit palette.
    PowerSuit,
}

/// External suit-palette owner completion.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeferredSuitPaletteReceipt {
    /// Complete actual current equipment dependency.
    pub dependency: SuitPaletteOrigin,
    /// Independently checked terminal palette meaning.
    pub terminal: SuitPaletteDisposition,
}

/// Actual completed suit-palette preparation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuitPaletteCompletion {
    /// Fresh current equipment consumed at completion.
    pub dependency: SuitPaletteOrigin,
    /// Actual independently borrowed result.
    pub terminal: SuitPaletteDisposition,
}

/// Liquid-surface owner arm qualified for this bounded operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiquidSurface {
    /// No active surface.
    Inactive,
}

/// Complete current environment owner consulted before the Inside queries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentOrigin {
    /// Actual equipment owner.
    pub equipment: Set<Item>,
    /// Actual starting pose.
    pub pose: Pose,
    /// Actual whole-pixel vertical centre.
    pub centre_y: u16,
    /// Actual ordinary liquid surface disposition.
    pub water: LiquidSurface,
    /// Actual lava/acid surface disposition.
    pub lava_acid: LiquidSurface,
}

/// External prepared environment; this prepares Rules, not Movement's terminal state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedEnvironment {
    /// Ordinary acceleration imported from checked reference content.
    pub acceleration: Fixed,
}

/// External environment result retained until its full physical operation completes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeferredEnvironmentReceipt {
    /// Complete actual current environment dependency.
    pub dependency: EnvironmentOrigin,
    /// Separately borrowed prepared result.
    pub terminal: PreparedEnvironment,
}

/// Actual completed environment preparation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentCompletion {
    /// Fresh actual dependency consumed at completion.
    pub dependency: EnvironmentOrigin,
    /// Actual independently borrowed result.
    pub terminal: PreparedEnvironment,
}

/// First three current owner slots; each is refreshed at its own consumption edge.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerEntryOrigins {
    /// Ordinary pose-input dependency.
    pub pose_input: PoseInputOrigin,
    /// Suit-palette dependency.
    pub suit_palette: SuitPaletteOrigin,
    /// Environment dependency.
    pub environment: EnvironmentOrigin,
}

/// Independently admitted suit-palette and environment offers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerEntryOffers {
    /// Deferred suit-palette disposition.
    pub suit_palette: DeferredSuitPaletteReceipt,
    /// Deferred environment preparation.
    pub environment: DeferredEnvironmentReceipt,
}

/// Canonical current-entry owner schema, traced from the actual serde aggregate.
pub struct PlayerEntryOriginsPin;
impl Pinned for PlayerEntryOriginsPin {
    type State = Box<PlayerEntryOrigins>;
    const NAME: &'static str = "production-frame-player-entry-origins";
    const SHAPE: u64 = 0x5DDB_743F_1513_91A9;
}

/// Canonical deferred-entry offer schema, traced from the actual serde aggregate.
pub struct PlayerEntryOffersPin;
impl Pinned for PlayerEntryOffersPin {
    type State = Box<PlayerEntryOffers>;
    const NAME: &'static str = "production-frame-player-entry-offers";
    const SHAPE: u64 = 0x6150_0C16_05B5_E193;
}

/// Actual completed entry owners retained across pauses and fresh restoration.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerEntryCompletions {
    /// Completed pose input, with its prospective pose still deferred.
    pub pose_input: Option<PoseInputCompletion>,
    /// Completed suit palette.
    pub suit_palette: Option<SuitPaletteCompletion>,
    /// Completed environment preparation.
    pub environment: Option<EnvironmentCompletion>,
}

/// Canonical completed-entry owner schema, traced from the actual serde aggregate.
pub struct PlayerEntryCompletionsPin;
impl Pinned for PlayerEntryCompletionsPin {
    type State = Box<PlayerEntryCompletions>;
    const NAME: &'static str = "production-frame-player-entry-completions";
    const SHAPE: u64 = 0x67B1_B4E9_17BA_50E2;
}

/// Current simulation-clock disposition, independent of instruction ownership.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClockDisposition {
    /// Simulation is running.
    Running,
    /// Unsupported stopped-clock arm.
    Stopped,
}

/// Both current clocks consulted by the bounded instruction collections.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionClocks {
    /// Current simulation clock.
    pub simulation: ClockDisposition,
    /// Current enemy clock.
    pub enemy: ClockDisposition,
}

/// Actual physical instruction-slot identity, not a source cursor or selected count.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionSlotIdentity(pub u8);

/// Actual current instruction-owner disposition.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstructionDisposition {
    /// No instruction body is active.
    Empty,
    /// An active body requires separate qualification.
    Active,
}

/// One actual ordered projectile instruction owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectileInstruction {
    /// Actual physical owner identity.
    pub identity: InstructionSlotIdentity,
    /// Actual instruction-owner disposition.
    pub instruction: InstructionDisposition,
}

/// Complete current projectile collection owner.
#[derive(Canonical, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectilesOrigin {
    /// All actual physical slot owners in traversal order.
    pub slots: [ProjectileInstruction; 10],
    /// Actual current simulation and enemy clocks.
    pub clocks: CollectionClocks,
}

/// External completed projectile collection, with no active instruction body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectilesTerminal {
    /// Actual unchanged ordered slot owners.
    pub slots: [ProjectileInstruction; 10],
}

/// Deferred complete projectile owner, requiring actual completed Inside/HUD dependencies.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeferredProjectilesReceipt {
    /// Complete actual current collection dependency.
    pub dependency: ProjectilesOrigin,
    /// Actual separately qualified unchanged collection result.
    pub terminal: ProjectilesTerminal,
}

/// One actual ordered sprite-object instruction owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteInstruction {
    /// Actual physical owner identity.
    pub identity: InstructionSlotIdentity,
    /// Actual instruction-owner disposition.
    pub instruction: InstructionDisposition,
}

/// Complete current sprite-object collection owner.
#[derive(Canonical, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteObjectsOrigin {
    /// All actual physical slot owners in traversal order.
    pub slots: [SpriteInstruction; 32],
    /// Actual current simulation and enemy clocks.
    pub clocks: CollectionClocks,
}

/// External completed sprite-object collection, with no active instruction body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteObjectsTerminal {
    /// Actual unchanged ordered slot owners.
    pub slots: [SpriteInstruction; 32],
}

/// Deferred complete sprite owner, requiring actual completed actor preparation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeferredSpriteObjectsReceipt {
    /// Complete actual current collection dependency.
    pub dependency: SpriteObjectsOrigin,
    /// Actual separately qualified unchanged collection result.
    pub terminal: SpriteObjectsTerminal,
}

/// Actual retained administrative counter; its value never selects a gameplay arm.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdministrativeCounter(pub u16);

/// Explicit separately qualified administrative counter effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CounterEffect {
    /// Advance the actual retained counter by one, wrapping at the word boundary.
    WrappingIncrementOne,
}

/// Current contact-reset owner; overwritten identities/distances are not input predicates.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactResetOrigin {
    /// Actual retained execution counter.
    pub execution_counter: AdministrativeCounter,
}

/// Actual sprite-interaction owner disposition.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionDisposition {
    /// Interaction is enabled.
    Enabled,
    /// Unsupported disabled-interaction arm.
    Disabled,
}

/// Actual Samus projectile and bomb activity owners.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SamusActivity {
    /// Actual projectile count.
    pub projectiles: u16,
    /// Actual bomb count.
    pub bombs: u16,
}

/// Complete current interaction admission, independent of selected Movement output.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SamusInteractionOrigin {
    /// Actual sprite-interaction owner.
    pub sprite_interaction: InteractionDisposition,
    /// Actual projectile-interaction owner.
    pub projectile_interaction: InteractionDisposition,
    /// Actual projectile/bomb activity owners.
    pub activity: SamusActivity,
}

/// Qualified no-activity interaction result; source bookkeeping is excluded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SamusInteractionTerminal {
    /// Actual admitted interaction disposition.
    pub sprite_interaction: InteractionDisposition,
    /// Actual admitted projectile-interaction disposition.
    pub projectile_interaction: InteractionDisposition,
    /// Actual unchanged projectile/bomb activity.
    pub activity: SamusActivity,
}

/// Deferred actual interaction result, requiring completed selected preparation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeferredSamusInteractionReceipt {
    /// Complete fresh current interaction dependency.
    pub dependency: SamusInteractionOrigin,
    /// Separately qualified no-activity result.
    pub terminal: SamusInteractionTerminal,
}

/// Actor solidity admitted by the checked current properties.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorSolidity {
    /// Solid-marked actor.
    Solid,
}

/// Current actor touch disposition, separate from its solidity.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorTouch {
    /// No Samus touch body is admitted for this actor.
    Intangible,
    /// The ordinary touch body is admitted; its current geometry misses.
    Tangible,
}

/// Actual qualified padded-view result, with its complete current geometry/view premises.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorViewDisposition {
    /// Within the padded view, but suppressed by the current invisible property.
    WithinPaddedViewButInvisible,
    /// Below the padded view.
    BelowPaddedView,
}

/// Checked ordinary Ceres-door AI/content meaning; native pointers remain importer-only.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorContentDisposition {
    /// Current Ceres-door content and ordinary AI with basic enabled instructions.
    OrdinaryCeresDoor,
}

/// Current scene admitted by the Ceres-door update.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorScene {
    /// Escape has not started.
    PreEscape,
}

/// Current graphics-update disposition, preserving the checked default lower properties.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphicsUpdated {
    /// Graphics-updated flag is set.
    Set,
    /// Graphics-updated flag is clear.
    Clear,
}

/// Current actor geometry, retained through this qualified no-writer update.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorGeometry {
    /// Actual centre, including its current fractional quantities.
    pub position: types::Position,
    /// Actual current half-sizes.
    pub radius: Radius,
}

/// Current unselected actor owner in qualified ordinal order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorSeed {
    /// Current checked ordinary content and AI.
    pub content: ActorContentDisposition,
    /// Current touch disposition.
    pub touch: ActorTouch,
    /// Current solid-marked disposition.
    pub solidity: ActorSolidity,
    /// Complete actual current geometry.
    pub geometry: ActorGeometry,
    /// Actual retained frame counter; never an admission selector.
    pub frame_counter: AdministrativeCounter,
    /// Actual current nonexpiring instruction timer.
    pub instruction_timer: u16,
    /// Actual graphics-update disposition.
    pub graphics_updated: GraphicsUpdated,
    /// Actual no-flash arm's current timer.
    pub flash_timer: u16,
}

/// Complete meaningful current actor owner in actual completed S1 active order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentActor {
    /// Actual physical actor slot from completed membership.
    pub identity: ActorSlotIdentity,
    /// Current checked ordinary content and AI.
    pub content: ActorContentDisposition,
    /// Current touch disposition.
    pub touch: ActorTouch,
    /// Current solid-marked disposition.
    pub solidity: ActorSolidity,
    /// Complete actual current geometry.
    pub geometry: ActorGeometry,
    /// Actual retained frame counter; never an admission selector.
    pub frame_counter: AdministrativeCounter,
    /// Actual current nonexpiring instruction timer.
    pub instruction_timer: u16,
    /// Actual graphics-update disposition.
    pub graphics_updated: GraphicsUpdated,
    /// Actual no-flash arm's current timer.
    pub flash_timer: u16,
    /// Actual qualified drawing disposition.
    pub view: ActorViewDisposition,
}

/// Actual Samus contact-damage mode.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactDamageDisposition {
    /// Ordinary actor contact.
    Ordinary,
    /// Special contact mode requires separate qualification.
    Special,
}

/// Actual additional ordinary-contact gate.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdditionalContactGate {
    /// No additional contact gate.
    None,
}

/// Current ordinary tangible-actor contact premises.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrdinaryActorContact {
    /// Actual actor invincibility timer.
    pub actor_invincibility: u16,
    /// Actual Samus invincibility timer.
    pub samus_invincibility: u16,
    /// Actual additional contact gate.
    pub additional_gate: AdditionalContactGate,
    /// Actual ordinary-contact damage disposition.
    pub damage_mode: ContactDamageDisposition,
    /// Actual sprite-interaction admission.
    pub sprite_interaction: InteractionDisposition,
    /// Actual current projectile/bomb activity.
    pub activity: SamusActivity,
}

/// Actual selected Samus quantities consumed by the bounded horizontal contact miss.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SamusContactGeometry {
    /// Actual whole-pixel horizontal centre.
    pub centre_x: types::Px,
    /// Actual checked current body's horizontal half-size.
    pub radius_x: types::Px,
}

/// Complete current actor-update owner; source allocator/AI cursors are excluded.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorUnselectedOrigin {
    /// Current owners in qualified ordinal order; no selected roster is installed.
    pub actors: [ActorSeed; 2],
    /// Actual current clocks.
    pub clocks: CollectionClocks,
    /// Actual pre-escape scene.
    pub scene: ActorScene,
    /// Actual no-shake request owner.
    pub shake_requested: bool,
    /// Actual tangible actor's ordinary contact premises.
    pub contact: OrdinaryActorContact,
}

/// Validation-only predicates for future actual selected dependencies.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorRequiredDependencies {
    /// Required contact geometry, checked against actual Samus and checked body content.
    pub samus: SamusContactGeometry,
    /// Required viewport, checked against the actual current Camera owner.
    pub camera_view: types::Pair<u16>,
    /// Required completed active order; never used to initialize or assign actor identities.
    pub active_order: [ActorSlotIdentity; 2],
    /// Required completed interactive order, distinct from active geometry order.
    pub interactive_order: Vec<ActorSlotIdentity>,
    /// Required drawing arms, validated before any physical actor work.
    pub views: [ActorViewDisposition; 2],
}

/// Separately qualified effects for one actual actor owner.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorEffects {
    /// Qualified unchanged actual geometry, never an observed future box.
    pub geometry: ActorGeometry,
    /// Qualified mechanical effect on the retained administrative frame counter.
    pub frame_counter: CounterEffect,
    /// Separately borrowed actual terminal instruction timer.
    pub instruction_timer: u16,
    /// Separately borrowed actual terminal graphics-update disposition.
    pub graphics_updated: GraphicsUpdated,
}

/// Complete actual borrowed actor update result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorUpdateTerminal {
    /// Actual owner effects in completed S1 active order.
    pub actors: [ActorEffects; 2],
}

/// Deferred complete actor owner effects with separately retained selected dependencies.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeferredActorUpdateReceipt {
    /// Complete current unselected actor and contact dependency.
    pub dependency: ActorUnselectedOrigin,
    /// Validation-only required selected dependencies; never model initializers.
    pub required: ActorRequiredDependencies,
    /// Separately qualified actual owner result.
    pub terminal: ActorUpdateTerminal,
}

/// One cleared solid-contact direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolidContact {
    /// Actual contact identity, absent after the reset.
    pub actor: Option<ActorSlotIdentity>,
    /// Actual contact distance, absent after the reset.
    pub distance: Option<types::Px>,
}

/// Complete separately qualified solid-contact reset.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactResetTerminal {
    /// Qualified mechanical effect on the actual retained execution counter.
    pub execution_counter: CounterEffect,
    /// Actual cleared left, right, up and down contact owners.
    pub contacts: [SolidContact; 4],
}

/// Deferred contact-reset owner result, requiring completed actor/sprite preparation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeferredContactResetReceipt {
    /// Actual retained administrative counter dependency.
    pub dependency: ContactResetOrigin,
    /// Separately qualified actual reset effects.
    pub terminal: ContactResetTerminal,
}

/// Actual completed selected prerequisites, qualified and completed by their sole owners.
pub struct CompletedInsideAndHudWeapons<'a> {
    /// Actual full Inside operation after its final retained answer was consumed.
    pub inside: &'a super::movement_stage::Completion,
    /// Actual early HUD controls from the completed observation in the sole Hud owner.
    pub controls: &'a super::hud_stage::ControlsCompletion,
}

/// Actual selected preparation retained before actor contact/update.
pub struct CompletedPlayerSelection<'a> {
    /// Actual completed Inside and HUD dependencies.
    pub inside_hud: CompletedInsideAndHudWeapons<'a>,
    /// Actual completed `MotionReset` and immutable previous movement snapshot.
    pub motion_reset: &'a super::MotionResetCompletion,
}

/// Completed actual projectile collection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectilesCompletion {
    /// Fresh current dependency consumed at completion.
    pub dependency: ProjectilesOrigin,
    /// Actual separately qualified result.
    pub terminal: ProjectilesTerminal,
}

/// Completed actual interaction admission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SamusInteractionCompletion {
    /// Fresh current dependency consumed at completion.
    pub dependency: SamusInteractionOrigin,
    /// Actual separately qualified result.
    pub terminal: SamusInteractionTerminal,
}

/// Actual completed actor owners, retaining completed interactive identity/order separately.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorUpdateCompletion {
    /// Fresh complete actor dependency consumed at completion.
    pub dependency: ActorUnselectedOrigin,
    /// Actual terminal actor owners in completed S1 active order.
    pub actors: [CurrentActor; 2],
    /// Actual completed S1 interactive identities; this is the World roster authority.
    pub interactive: Vec<ActorSlotIdentity>,
}

/// Completed actual sprite-object collection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteObjectsCompletion {
    /// Fresh current dependency consumed at completion.
    pub dependency: SpriteObjectsOrigin,
    /// Actual separately qualified result.
    pub terminal: SpriteObjectsTerminal,
}

/// Completed actual contact reset.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactResetCompletion {
    /// Actual retained counter dependency consumed at completion.
    pub dependency: ContactResetOrigin,
    /// Actual counter after the qualified mechanical administrative effect.
    pub execution_counter: AdministrativeCounter,
    /// Actual cleared contacts in left/right/up/down order.
    pub contacts: [SolidContact; 4],
}

/// Changed dependency, unavailable content or unsupported preparation arm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerPreparationFault {
    /// Complete fresh current dependency differs from the retained predicate.
    Dependency,
    /// Required checked imported content differs or is unavailable.
    Content,
    /// Actual gameplay route or owner arm is unsupported.
    Unsupported,
}

fn running(clocks: CollectionClocks) -> Result<(), PlayerPreparationFault> {
    if clocks.simulation != ClockDisposition::Running || clocks.enemy != ClockDisposition::Running {
        return Err(PlayerPreparationFault::Unsupported);
    }
    Ok(())
}

const fn no_activity(activity: SamusActivity) -> Result<(), PlayerPreparationFault> {
    if activity.projectiles != 0 || activity.bombs != 0 {
        return Err(PlayerPreparationFault::Unsupported);
    }
    Ok(())
}

impl CompletedInsideAndHudWeapons<'_> {
    /// Check causal completion shape; the caller independently qualifies each full source envelope.
    /// This is never an admission check by movement family or HUD selection alone.
    fn validate(&self) -> Result<(), PlayerPreparationFault> {
        use samus_movement::suspended::MovementDecision;
        let inside = &self.inside.receipt;
        if inside.group != super::movement_stage::Group::Inside
            || !matches!(
                inside.decisions.0.as_slice(),
                [
                    MovementDecision::InsideRequest(_),
                    MovementDecision::InsideAnswer(_)
                ]
            )
            || self.controls.answer.0.selection != interface_hud::Selection::None
            || self.controls.answer.0.switched.is_some()
        {
            return Err(PlayerPreparationFault::Dependency);
        }
        Ok(())
    }
}

impl DeferredProjectilesReceipt {
    /// Consume once after the actual full projectile opportunity and completed selected inputs.
    /// # Errors
    /// Rejects changed owners, active instructions, stopped clocks or missing actual prerequisites.
    pub fn complete(
        self,
        current: &ProjectilesOrigin,
        selected: &CompletedInsideAndHudWeapons<'_>,
    ) -> Result<ProjectilesCompletion, PlayerPreparationFault> {
        if current != &self.dependency || self.terminal.slots != current.slots {
            return Err(PlayerPreparationFault::Dependency);
        }
        current.complete_quiet(selected)
    }
}

impl ProjectilesOrigin {
    pub(super) fn complete_quiet(
        &self,
        selected: &CompletedInsideAndHudWeapons<'_>,
    ) -> Result<ProjectilesCompletion, PlayerPreparationFault> {
        selected.validate()?;
        running(self.clocks)?;
        for (index, slot) in self.slots.iter().enumerate() {
            if usize::from(slot.identity.0) != index
                || slot.instruction != InstructionDisposition::Empty
            {
                return Err(PlayerPreparationFault::Unsupported);
            }
        }
        Ok(ProjectilesCompletion {
            dependency: self.clone(),
            terminal: ProjectilesTerminal { slots: self.slots },
        })
    }
}

impl DeferredSamusInteractionReceipt {
    /// Consume once with actual completed selected preparation and projectile owners.
    /// # Errors
    /// Rejects changed current owners, activity, disabled modes or inconsistent projectile results.
    pub fn complete(
        self,
        current: &SamusInteractionOrigin,
        selected: &CompletedPlayerSelection<'_>,
        projectiles: &ProjectilesCompletion,
    ) -> Result<SamusInteractionCompletion, PlayerPreparationFault> {
        if current != &self.dependency
            || self.terminal.sprite_interaction != current.sprite_interaction
            || self.terminal.projectile_interaction != current.projectile_interaction
            || self.terminal.activity != current.activity
        {
            return Err(PlayerPreparationFault::Dependency);
        }
        current.complete_quiet(selected, projectiles)
    }
}

impl SamusInteractionOrigin {
    pub(super) fn complete_quiet(
        self,
        selected: &CompletedPlayerSelection<'_>,
        projectiles: &ProjectilesCompletion,
    ) -> Result<SamusInteractionCompletion, PlayerPreparationFault> {
        selected.inside_hud.validate()?;
        if projectiles.terminal.slots != projectiles.dependency.slots {
            return Err(PlayerPreparationFault::Dependency);
        }
        running(projectiles.dependency.clocks)?;
        if projectiles
            .terminal
            .slots
            .iter()
            .any(|slot| slot.instruction != InstructionDisposition::Empty)
            || self.sprite_interaction != InteractionDisposition::Enabled
            || self.projectile_interaction != InteractionDisposition::Enabled
        {
            return Err(PlayerPreparationFault::Unsupported);
        }
        no_activity(self.activity)?;
        Ok(SamusInteractionCompletion {
            dependency: self,
            terminal: SamusInteractionTerminal {
                sprite_interaction: self.sprite_interaction,
                projectile_interaction: self.projectile_interaction,
                activity: self.activity,
            },
        })
    }
}

/// Live body and camera used to qualify an actor's geometry at the current opportunity.
#[derive(Clone, Copy)]
pub struct ActorView<'a> {
    /// Current selected movement state.
    pub samus: &'a samus_movement::Samus,
    /// Pose dimensions from the admitted movement content.
    pub content: &'a MovementContent,
    /// Current camera position in pixels.
    pub camera: types::Pair<u16>,
}

impl ActorUnselectedOrigin {
    pub(super) fn validate_quiet_update(&self) -> Result<(), PlayerPreparationFault> {
        running(self.clocks)?;
        no_activity(self.contact.activity)?;
        if self.shake_requested
            || self.contact.actor_invincibility != 0
            || self.contact.samus_invincibility != 0
            || self.contact.damage_mode != ContactDamageDisposition::Ordinary
            || self.contact.sprite_interaction != InteractionDisposition::Enabled
        {
            return Err(PlayerPreparationFault::Unsupported);
        }
        Ok(())
    }
}

impl DeferredActorUpdateReceipt {
    /// Validate actual selected dependencies before accepting physical Actor work.
    /// # Errors
    /// Rejects changed current owners, actual selected body/view, memberships or Inside receipt.
    pub fn validate_dependencies(
        &self,
        current: &ActorUnselectedOrigin,
        selected: &CompletedPlayerSelection<'_>,
        view: ActorView<'_>,
        membership: &ActorEligibilityReceipt,
        interaction: &SamusInteractionCompletion,
    ) -> Result<(), PlayerPreparationFault> {
        use samus_movement::suspended::MovementDecision;
        use types::Parts;
        let ActorView {
            samus: actual_samus,
            content,
            camera: camera_view,
        } = view;
        selected.inside_hud.validate()?;
        if current != &self.dependency
            || !actual_samus.eq(&selected.motion_reset.previous.0)
            || self.required.camera_view != camera_view
            || membership.active.as_slice() != self.required.active_order
            || membership.interactive != self.required.interactive_order
            || membership.active.len() != current.actors.len()
            || membership.interactive.as_slice()
                != membership
                    .active
                    .get(1..)
                    .ok_or(PlayerPreparationFault::Dependency)?
            || current.contact.activity != interaction.terminal.activity
            || current.contact.sprite_interaction != interaction.terminal.sprite_interaction
        {
            return Err(PlayerPreparationFault::Dependency);
        }
        let radius = content
            .radius(actual_samus.pose)
            .ok_or(PlayerPreparationFault::Content)?;
        if self.required.samus.centre_x != actual_samus.position.x.pixel()
            || self.required.samus.radius_x != radius.x
        {
            return Err(PlayerPreparationFault::Dependency);
        }
        let Some(MovementDecision::InsideRequest(bounds)) =
            selected.inside_hud.inside.receipt.decisions.0.first()
        else {
            return Err(PlayerPreparationFault::Dependency);
        };
        if bounds.at != actual_samus.position || bounds.r != radius {
            return Err(PlayerPreparationFault::Dependency);
        }
        if self.required.views
            != [
                ActorViewDisposition::WithinPaddedViewButInvisible,
                ActorViewDisposition::BelowPaddedView,
            ]
            || membership.active.iter().any(|identity| identity.0 >= 32)
            || membership.active.first() == membership.active.last()
        {
            return Err(PlayerPreparationFault::Dependency);
        }
        Ok(())
    }

    /// Consume once after repeating validation against actual completed selected owners.
    /// # Errors
    /// Rejects changed dependencies, effects or bounded AI arms.
    pub fn complete(
        self,
        current: &ActorUnselectedOrigin,
        selected: &CompletedPlayerSelection<'_>,
        view: ActorView<'_>,
        membership: &ActorEligibilityReceipt,
        interaction: &SamusInteractionCompletion,
    ) -> Result<ActorUpdateCompletion, PlayerPreparationFault> {
        self.validate_dependencies(current, selected, view, membership, interaction)?;
        current.validate_quiet_update()?;
        let mut actors = Vec::with_capacity(2);
        for (index, (actor, effect)) in current.actors.iter().zip(&self.terminal.actors).enumerate()
        {
            if effect.geometry != actor.geometry
                || effect.instruction_timer != actor.instruction_timer.wrapping_sub(1)
            {
                return Err(PlayerPreparationFault::Dependency);
            }
            let touch = if index == 0 {
                ActorTouch::Intangible
            } else {
                ActorTouch::Tangible
            };
            let view = if index == 0 {
                ActorViewDisposition::WithinPaddedViewButInvisible
            } else {
                ActorViewDisposition::BelowPaddedView
            };
            if actor.touch != touch
                || self.required.views.get(index) != Some(&view)
                || actor.flash_timer != 0
                || actor.instruction_timer == 1
                || actor.graphics_updated != GraphicsUpdated::Set
                || effect.graphics_updated != GraphicsUpdated::Clear
            {
                return Err(PlayerPreparationFault::Unsupported);
            }
            let mut terminal = CurrentActor {
                identity: *membership
                    .active
                    .get(index)
                    .ok_or(PlayerPreparationFault::Dependency)?,
                content: actor.content,
                touch: actor.touch,
                solidity: actor.solidity,
                geometry: actor.geometry,
                frame_counter: actor.frame_counter,
                instruction_timer: actor.instruction_timer,
                graphics_updated: actor.graphics_updated,
                flash_timer: actor.flash_timer,
                view,
            };
            terminal.frame_counter = match effect.frame_counter {
                CounterEffect::WrappingIncrementOne => {
                    AdministrativeCounter(actor.frame_counter.0.wrapping_add(1))
                }
            };
            terminal.instruction_timer = effect.instruction_timer;
            terminal.graphics_updated = effect.graphics_updated;
            terminal.geometry = effect.geometry;
            actors.push(terminal);
        }
        Ok(ActorUpdateCompletion {
            dependency: self.dependency,
            actors: actors
                .try_into()
                .map_err(|_| PlayerPreparationFault::Dependency)?,
            interactive: membership.interactive.clone(),
        })
    }
}

impl ActorUpdateCompletion {
    /// Join completed interactive identities to actual completed marked-solid actor geometry.
    /// Both bounded owners are marked solid; no unqualified frozen state is invented.
    /// # Errors
    /// Rejects missing/duplicate identities or a mismatched actual completed actor owner.
    pub fn solid_boxes(&self) -> Result<Vec<types::Hitbox>, PlayerPreparationFault> {
        use types::Parts;
        let mut boxes = Vec::with_capacity(self.interactive.len());
        for (index, identity) in self.interactive.iter().enumerate() {
            if self
                .interactive
                .iter()
                .take(index)
                .any(|earlier| earlier == identity)
            {
                return Err(PlayerPreparationFault::Dependency);
            }
            let mut actors = self
                .actors
                .iter()
                .filter(|actor| actor.identity == *identity);
            let actor = actors.next().ok_or(PlayerPreparationFault::Dependency)?;
            if actors.next().is_some() {
                return Err(PlayerPreparationFault::Dependency);
            }
            match actor.solidity {
                ActorSolidity::Solid => boxes.push(types::Hitbox {
                    centre: types::Pair {
                        x: actor.geometry.position.x.pixel(),
                        y: actor.geometry.position.y.pixel(),
                    },
                    radius: actor.geometry.radius,
                }),
            }
        }
        Ok(boxes)
    }
}

impl DeferredSpriteObjectsReceipt {
    /// Consume once after actual actor completion, with fresh current clocks and slot owners.
    /// # Errors
    /// Rejects changed owners, active instructions or invalid completed actor identity joins.
    pub fn complete(
        self,
        current: &SpriteObjectsOrigin,
        actors: &ActorUpdateCompletion,
    ) -> Result<SpriteObjectsCompletion, PlayerPreparationFault> {
        if current != &self.dependency || self.terminal.slots != current.slots {
            return Err(PlayerPreparationFault::Dependency);
        }
        current.complete_quiet(actors)
    }
}

impl SpriteObjectsOrigin {
    pub(super) fn complete_quiet(
        &self,
        actors: &ActorUpdateCompletion,
    ) -> Result<SpriteObjectsCompletion, PlayerPreparationFault> {
        let _ = actors.solid_boxes()?;
        running(self.clocks)?;
        for (index, slot) in self.slots.iter().enumerate() {
            if usize::from(slot.identity.0) != index
                || slot.instruction != InstructionDisposition::Empty
            {
                return Err(PlayerPreparationFault::Unsupported);
            }
        }
        Ok(SpriteObjectsCompletion {
            dependency: self.clone(),
            terminal: SpriteObjectsTerminal { slots: self.slots },
        })
    }
}

impl DeferredContactResetReceipt {
    /// Consume once after actual actor/sprite completion and apply the qualified counter effect.
    /// # Errors
    /// Rejects changed counters, contacts or missing consistent completed producer results.
    pub fn complete(
        self,
        current: ContactResetOrigin,
        actors: &ActorUpdateCompletion,
        sprites: &SpriteObjectsCompletion,
    ) -> Result<ContactResetCompletion, PlayerPreparationFault> {
        if current != self.dependency {
            return Err(PlayerPreparationFault::Dependency);
        }
        if self
            .terminal
            .contacts
            .iter()
            .any(|contact| contact.actor.is_some() || contact.distance.is_some())
        {
            return Err(PlayerPreparationFault::Unsupported);
        }
        current.complete_reset(actors, sprites)
    }
}

impl ContactResetOrigin {
    pub(super) fn complete_reset(
        self,
        actors: &ActorUpdateCompletion,
        sprites: &SpriteObjectsCompletion,
    ) -> Result<ContactResetCompletion, PlayerPreparationFault> {
        if sprites != &sprites.dependency.complete_quiet(actors)? {
            return Err(PlayerPreparationFault::Dependency);
        }
        Ok(ContactResetCompletion {
            dependency: self,
            execution_counter: AdministrativeCounter(self.execution_counter.0.wrapping_add(1)),
            contacts: [SolidContact {
                actor: None,
                distance: None,
            }; 4],
        })
    }
}

pub(super) fn gameplay(receipt: &GameplayDispatchReceipt) -> Result<(), PlayerPreparationFault> {
    if receipt.scene != SceneController::MainGameplay
        || receipt.route != GameplayRoute::MainGameplay
    {
        return Err(PlayerPreparationFault::Unsupported);
    }
    Ok(())
}

pub(super) fn ordinary_equipment(equipment: &Set<Item>) -> Result<(), PlayerPreparationFault> {
    if equipment[Item::GravitySuit] || equipment[Item::VariaSuit] {
        return Err(PlayerPreparationFault::Unsupported);
    }
    Ok(())
}

impl DeferredSuitPaletteReceipt {
    /// Consume the borrowed palette meaning once after the full actual operation.
    /// # Errors
    /// Rejects changed equipment, unsupported suits or unavailable actual gameplay dispatch.
    pub fn complete(
        self,
        current: &SuitPaletteOrigin,
        route: &GameplayDispatchReceipt,
    ) -> Result<SuitPaletteCompletion, PlayerPreparationFault> {
        gameplay(route)?;
        if current != &self.dependency {
            return Err(PlayerPreparationFault::Dependency);
        }
        ordinary_equipment(&current.equipment)?;
        Ok(SuitPaletteCompletion {
            dependency: self.dependency,
            terminal: self.terminal,
        })
    }
}

impl DeferredEnvironmentReceipt {
    /// Consume once against fresh current environment and independently imported acceleration.
    /// # Errors
    /// Rejects changed current owners, unsupported suits/route or changed reference content.
    pub fn complete(
        self,
        current: &EnvironmentOrigin,
        imported_acceleration: Fixed,
        route: &GameplayDispatchReceipt,
    ) -> Result<EnvironmentCompletion, PlayerPreparationFault> {
        gameplay(route)?;
        if current != &self.dependency {
            return Err(PlayerPreparationFault::Dependency);
        }
        ordinary_equipment(&current.equipment)?;
        if self.terminal.acceleration != imported_acceleration {
            return Err(PlayerPreparationFault::Content);
        }
        Ok(EnvironmentCompletion {
            dependency: self.dependency,
            terminal: self.terminal,
        })
    }
}

/// Remaining five player preparation origins, separate from entry owners.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerContinuationOrigins {
    /// Projectiles owner.
    pub projectiles: ProjectilesOrigin,
    /// Interaction owner.
    pub interaction: SamusInteractionOrigin,
    /// Actors owner.
    pub actors: ActorUnselectedOrigin,
    /// Sprites owner.
    pub sprites: SpriteObjectsOrigin,
    /// Contact owner.
    pub contact: ContactResetOrigin,
}

/// Canonical remaining-five current origins.
pub struct PlayerContinuationOriginsPin;
impl Pinned for PlayerContinuationOriginsPin {
    type State = Box<PlayerContinuationOrigins>;
    const NAME: &'static str = "production-frame-player-continuation-origins";
    const SHAPE: u64 = 0x0802_52EE_E8CF_306B;
}

/// Remaining five player preparation offers, separate from entry owners.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerContinuationOffers {
    /// Projectiles owner.
    pub projectiles: DeferredProjectilesReceipt,
    /// Interaction owner.
    pub interaction: DeferredSamusInteractionReceipt,
    /// Actors owner.
    pub actors: DeferredActorUpdateReceipt,
    /// Sprites owner.
    pub sprites: DeferredSpriteObjectsReceipt,
    /// Contact owner.
    pub contact: DeferredContactResetReceipt,
}

/// Canonical remaining-five separately deferred offers.
pub struct PlayerContinuationOffersPin;
impl Pinned for PlayerContinuationOffersPin {
    type State = Box<PlayerContinuationOffers>;
    const NAME: &'static str = "production-frame-player-continuation-offers";
    const SHAPE: u64 = 0xCB4B_2F27_1F8A_B2F6;
}

/// Remaining five player preparation completions, separate from entry owners.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerContinuationCompletions {
    /// Projectiles owner.
    pub projectiles: Option<ProjectilesCompletion>,
    /// Interaction owner.
    pub interaction: Option<SamusInteractionCompletion>,
    /// Actors owner.
    pub actors: Option<ActorUpdateCompletion>,
    /// Sprites owner.
    pub sprites: Option<SpriteObjectsCompletion>,
    /// Contact owner.
    pub contact: Option<ContactResetCompletion>,
}

/// Canonical remaining-five actual completions.
pub struct PlayerContinuationCompletionsPin;
impl Pinned for PlayerContinuationCompletionsPin {
    type State = Box<PlayerContinuationCompletions>;
    const NAME: &'static str = "production-frame-player-continuation-completions";
    const SHAPE: u64 = 0xB010_9A1E_C067_AD26;
}

#[cfg(test)]
#[test]
fn player_continuation_pins_hold() {
    assert_eq!(
        canonical::model::pin_holds::<PlayerContinuationOriginsPin>(),
        Ok(())
    );
    assert_eq!(
        canonical::model::pin_holds::<PlayerContinuationOffersPin>(),
        Ok(())
    );
    assert_eq!(
        canonical::model::pin_holds::<PlayerContinuationCompletionsPin>(),
        Ok(())
    );
}

#[cfg(test)]
#[path = "borrowed_player_preparation/actor_tests.rs"]
mod actor_tests;
