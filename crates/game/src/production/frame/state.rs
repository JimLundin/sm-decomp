//! Complete selected owners committed together at a retained frame boundary.

use super::inputs::{FigurePin, MovementInputsPin, PreparedActionPin, WorldPin};
use super::{
    ContentFault, PublicationFault, Publications, TransferContent, hud::Hud, movement::Movement,
};
use crate::production::sender::Senders;
use crate::production::{AdmissionFault, ProductionFault, SenderInput, appearance, timing};
use crate::state::owned::CameraPin;
use audio_transport::SoundGroup;
use canonical::Canonical;
use canonical::model::Model;
use data::timing::MasterTicks;

/// One authority for each selected producer and its physical consumers.
/// Empty appearance work holds the initial quantities without executing a rule.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Current software continuation and unread admitted external owner quantities.
    pub caller: super::Caller,
    /// Actual current drawing interrupt request, retained independently of its parent.
    pub drawing: Option<super::interrupts::DrawingState>,
    /// Retained hardware, charges and suspended interrupt parents.
    pub timing: timing::State,
    /// Current music countdown and queue disposition.
    pub music: Option<super::borrowed_receipts::MusicServiceReceipt>,
    /// Complete current Haze model and its physical transport.
    pub haze: Option<super::haze::Haze>,
    /// Shared sequence as of the most recent completed Random draw.
    pub random: Option<super::random::Sequence>,
    /// Current scene controller carried between returns and read at gameplay dispatch.
    pub scene: Option<super::borrowed_gameplay::SceneController>,
    /// Live actor scheduling policy and physical slot properties.
    pub actor_eligibility: Option<super::borrowed::ActorEligibilityOrigin>,
    /// Current debug mode, qualified shortcut latch and controller two edges.
    pub debug: Option<super::borrowed_preparation::DebugOrigin>,
    /// Current palette object enable and slot occupancy.
    pub palette_objects: Option<super::borrowed_preparation::PaletteOrigin>,
    /// Current equipment and room surfaces for ordinary player entry.
    pub entry_environment: Option<super::entry_environment::EntryEnvironment>,
    /// Live sprite-builder backing before the next reset consumes it.
    pub sprite_builder: Option<super::borrowed::SpriteBuilderOrigin>,
    /// Current movement, including accepted requests and answers.
    pub movement: Movement,
    /// Retained physical operation metadata; current movement remains in its sole owner.
    pub movement_stage: super::movement_stage::Stage,
    /// Sole current room-block owner available before any actor geometry exists.
    pub inside_world: Option<Model<super::inside_world::InsideWorldPin>>,
    /// Actual current generic premises captured only as early Inside bookkeeping.
    /// These are not proof that the later frame-start animation query completed.
    pub inside_inputs: Option<Model<MovementInputsPin>>,
    /// Current live stable player modes and parity across requested NMIs.
    pub inside_premises: Option<super::inputs::InsidePremises>,
    /// Live products belonging only to the actual full Inside physical owner.
    pub inside_pending: Option<super::inside_contract::Pending>,
    /// Actual completed Inside receipt, retained for subsequent player owners.
    pub inside_completed: Option<super::movement_stage::Completion>,
    /// Complete current following camera, independent of future recorded viewports.
    pub camera: Model<CameraPin>,
    /// Actual graphical outputs already consumed by their qualified rendering work.
    pub graphical_contacts: super::rendering_stage::Contacts,
    /// Current sprite production and its once-only readiness publication.
    pub rendering: super::rendering_stage::Stage,
    /// Borrowed current drawing neighbours; selected producers join at the drawing opportunity.
    pub drawing_origins: Option<super::drawing_owners::Origins>,
    /// Independently imported Contact neighbours, revalidated at their actual use.
    pub contact_current: Option<Model<super::contact_contract::CurrentPin>>,
    /// Actual source-qualified sound-enabled, demo and power-bomb owners.
    pub sound_offer_current: Option<super::post_down_contract::OfferCurrent>,
    /// Current external inputs captured at their actual opportunity; unread inputs stay absent.
    pub movement_inputs: Option<Model<MovementInputsPin>>,
    /// Geometry captured at its actual opportunity, including current reaction permissions.
    pub world: Option<Model<WorldPin>>,
    /// Past action captured by actual preparation, independent of later current action.
    pub prepared_action: Option<Model<PreparedActionPin>>,
    /// Current animation and retained appearance outputs.
    pub appearance: appearance::Work,
    /// Actual currently playing figure, never reconstructed from a future selected pose.
    pub playing_figure: Model<FigurePin>,
    /// Complete HUD grid, caches, selection and exploration.
    pub hud: Hud,
    /// Independently admitted current Display owners, revalidated at actual refresh.
    pub hud_display_current: Option<Model<super::hud_stage::DisplayReceiptPin>>,
    /// Current inventory read by item controls, independent of HUD drawing.
    pub hud_inventory: Option<super::hud_controls_inputs::CurrentInventory>,
    /// Current HUD frame byte advanced at actual requested NMI completion.
    pub hud_frame_count: Option<u8>,
    /// Actual complete tilemap readiness, publication and separate return progress.
    pub hud_map: super::hud_map_caller::Stage,
    /// Sole complete live health and actual actor/interrupt preservation history.
    pub health: Option<super::health::Health>,
    /// Actual returned Health maintenance and its post-opportunity energy.
    pub maintenance: super::maintenance::Stage,
    /// Sole shared colour banks and actual retained Appearance colour preparation.
    pub samus_palette: Option<super::samus_palette::Palette>,
    /// Actual empty neighbours and following-camera progress.
    pub s6_camera: super::s6_camera::Stage,
    /// Actual Draw, selected appearance and current sprite-builder progress.
    pub s6_drawing: super::s6_drawing::Stage,
    /// Retained actual Background arithmetic.
    pub background: super::suffix_caller::BackgroundScroll,
    /// Retained actual finalization, request and physical native return.
    pub return_progress: super::frame_return::Stage,
    /// Actual ordered Sender services during the frame tail.
    pub sender_progress: super::sender_caller::Cursor,
    /// Actual Sender admission inputs retained for validation of the earlier sound offer.
    pub sender_history: Option<super::sender_caller::History>,
    /// Actual late HUD/status and once-only queue publishing authority.
    pub late_hud: super::late_hud::Stage,
    /// Carried current and prior-frame Drawing and final controller history.
    pub frame_history: Option<super::frame_history::History>,
    /// Current player collection and interaction owners carried across native returns.
    pub recurring_player: Option<super::player_continuation::RecurringPlayer>,
    /// Current physical action bindings, independently owned across returns.
    pub pose_input_configuration: Option<super::pose_input::Configuration>,
    /// Independently admitted post-background room scope.
    pub room_tail_origin: Option<super::room_tail::Admission>,
    /// Actual block and room work before play time.
    pub room_tail: super::room_tail::Stage,
    /// Complete play time and pending physical publication.
    pub play_time: super::play_time::Stage,
    /// Complete current room quake and pending physical phase publication.
    pub room_quake: super::room_quake::Stage,
    /// Current actor cursors and three independent hurt timers.
    pub frame_tail_current: Option<super::frame_tail::Current>,
    /// Actual hurt/list and final history lifecycle.
    pub frame_tail: super::frame_tail::Stage,
    /// Independently retained original liquid physics.
    pub liquid_history: Option<super::liquid_history::History>,
    /// Actual post-countdown animation and its consumed completion.
    pub animation: super::animation_caller::Stage,
    /// Actual inactive interruption through its entire physical return.
    pub interruption: super::interruption_caller::Stage,
    /// Late movement selection and actual physical continuation.
    pub late_movement: super::late_movement::Stage,
    /// Countdown caller selection and independently retained actual completion.
    pub countdown: super::countdown_caller::Stage,
    /// Actual delivered receiver prefix; publication inspection never appends here.
    pub countdown_delivered: Vec<Model<super::countdown_caller::AnswerPin>>,
    /// Separately borrowed current scene scheduling, never inferred from HUD countdown.
    pub scene_countdown: Option<Model<super::countdown_scene::CurrentPin>>,
    /// Three independent actual sender owners.
    pub senders: Senders,
    /// Current qualified service metadata, retaining no second writable sender state.
    pub sender_stage: super::sender_stage::Stage,
    /// Exactly-once progress through actual retained appearance sound requests.
    pub sound_offers: super::sender_stage::Offers,
    /// Independently borrowed current service downtime and guarded fetch regime.
    pub sender_current: Option<super::sender_contract::Current>,
    /// Origin-derived physical queue positions checked against the sole clean FIFOs.
    pub sender_queues: Option<super::sender_contract::Queues>,
    /// Current source content; future materializations remain absent.
    pub content: TransferContent,
    /// Complete physical output order and external delivery progress.
    pub publications: Publications,
}

/// A complete frame transaction failed before commit.
#[derive(Debug, PartialEq, Eq)]
pub enum Fault {
    /// Invalid hardware or unavailable physical mechanism.
    Timing(timing::Fault),
    /// Invalid current content ownership.
    Content(ContentFault),
    /// Invalid retained output ordering or delivery progress.
    Publications(PublicationFault),
    /// Invalid retained appearance work.
    Appearance(appearance::WorkFault),
    /// Graphical consumer progress does not match retained Appearance outputs.
    Rendering(super::rendering_stage::Fault),
    /// Invalid sender ownership.
    Sender(AdmissionFault),
    /// Sender service or produced-request prefix does not match its current owner.
    SenderStage(super::sender_stage::Fault),
    /// Retained physical queue positions differ from actual sender owners.
    SenderContract(super::sender_contract::Fault),
    /// Actual sender service rejected its inputs.
    Production(ProductionFault),
    /// Decoded geometry is not a valid current provider.
    World(crate::production::world::WorldFault),
    /// Preparation has already captured this frame's action.
    AlreadyPrepared,
    /// Drawing requires the actual completed preparation snapshot.
    UnpreparedDrawing,
    /// Selection cannot consume partial movement outputs as completed publications.
    UnfinishedMovement,
    /// Entry completion history or retained physical identity is inconsistent.
    InvalidPlayerEntry,
    /// A restored Random request disagrees with its owner, seed or physical order.
    InvalidRandomWork,
    /// Current room-block capability failed restored geometry validation.
    InsideWorld(crate::production::world::WorldFault),
    /// The actual figure requires unavailable checked clip content.
    Clip(crate::production::ClipFault),
}

impl State {
    pub(super) fn has_physical_bill(
        &self,
        identity: timing::WorkId,
        profile: &data::timing::Profile,
    ) -> bool {
        let mut active = self
            .timing
            .work
            .iter()
            .filter(|work| work.kind == timing::WorkKind::Semantic(identity));
        active.next().is_some_and(|work| &work.profile == profile) && active.next().is_none()
    }

    /// Read the current shared number without advancing the sequence.
    #[must_use]
    pub fn current_random(&self) -> Option<u16> {
        self.random.as_ref().map(|sequence| sequence.current().0)
    }

    /// Capture actual pose history and prepared action at a completed motion owner.
    /// The caller supplies only the identity returned by qualified physical work.
    /// Prospective pose input does not replace either current selected model here.
    ///
    /// # Errors
    /// This frame's preparation has already consumed its actual current action.
    pub fn complete_motion_reset(
        &mut self,
        work: timing::WorkId,
    ) -> Result<super::MotionResetCompletion, Fault> {
        if work != timing::WorkId(18) {
            return Err(Fault::InvalidPlayerEntry);
        }
        self.prepare_action()?;
        Ok(super::MotionResetCompletion {
            work,
            previous: Model(*self.movement.state()),
            clip_start: super::motion_reset_contract::ClipStartPreparation::Cleared,
        })
    }

    /// Read frame-start animation feedback from the actual retained playing owner.
    /// No future movement pose or recorded playhead supplies this query.
    ///
    /// # Errors
    /// The current playing figure has no admitted immutable clip.
    pub fn current_animation(
        &self,
        content: &crate::production::AppearanceContent,
    ) -> Result<samus_appearance::Ahead, Fault> {
        Ok(appearance::ahead(
            &self.appearance.model.0.state,
            self.current_clip(content)?,
        ))
    }

    /// Resolve current-playing content without choosing a future pose or sprite.
    ///
    /// # Errors
    /// The retained actual figure has no imported clip.
    pub fn current_clip<'a>(
        &self,
        content: &'a crate::production::AppearanceContent,
    ) -> Result<&'a samus_appearance::Clip, Fault> {
        content.get(self.playing_figure.0).ok_or(Fault::Clip(
            crate::production::ClipFault::Missing(self.playing_figure.0),
        ))
    }

    /// Resolve the actual accepted movement pose and completed drawing sprite publication.
    /// This input does not publish the selected clip or replace the current playing figure.
    ///
    /// # Errors
    /// Incomplete movement or an actual figure with no imported content.
    pub fn accepted_figure(
        &self,
        content: &crate::production::AppearanceContent,
    ) -> Result<samus_appearance::Figure, Fault> {
        let (movement, outputs) = self.movement.finished().ok_or(Fault::UnfinishedMovement)?;
        let figure = samus_appearance::Figure {
            pose: movement.pose,
            sprite: outputs.sprite,
        };
        content
            .get(figure)
            .ok_or(Fault::Clip(crate::production::ClipFault::Missing(figure)))?;
        Ok(figure)
    }

    /// Capture only the actual current movement action when preparation is performed.
    ///
    /// # Errors
    /// Repeating preparation cannot replace the completed snapshot.
    pub const fn prepare_action(&mut self) -> Result<(), Fault> {
        if self.prepared_action.is_some() {
            return Err(Fault::AlreadyPrepared);
        }
        self.prepared_action = Some(Model(self.movement.state().pose.action));
        Ok(())
    }

    /// Form a fresh actual drawing read from preparation and current movement authorities.
    /// No drawing, animation or sound effect is performed by constructing the input.
    ///
    /// # Errors
    /// Drawing before completed preparation remains unsupported.
    pub fn drawing_input(&self) -> Result<samus_appearance::AppearanceRead, Fault> {
        let previous = self
            .prepared_action
            .as_ref()
            .ok_or(Fault::UnpreparedDrawing)?;
        Ok(samus_appearance::AppearanceRead::Draw {
            previous: previous.0,
            current: self.movement.state().pose.action,
        })
    }

    /// Validate the complete saved physical and consumer continuation.
    ///
    /// # Errors
    /// Any invalid owner or pending continuation rejects admission.
    pub fn validate(&self) -> Result<(), Fault> {
        let sender_started = self.sender_progress != super::sender_caller::Cursor::New;
        if self.sender_history.is_some() != sender_started {
            return Err(Fault::InvalidPlayerEntry);
        }
        self.validate_player_entry()?;
        self.validate_prelude()?;
        self.validate_random_work()?;
        if self
            .samus_palette
            .as_ref()
            .is_some_and(|palette| !palette.holds(self))
        {
            return Err(Fault::InvalidPlayerEntry);
        }
        if self
            .liquid_history
            .as_ref()
            .is_some_and(|history| history.validate_current().is_err())
        {
            return Err(Fault::InvalidPlayerEntry);
        }
        if self
            .health
            .as_ref()
            .is_some_and(|health| health.validate_current().is_err())
        {
            return Err(Fault::InvalidPlayerEntry);
        }
        if self
            .scene_countdown
            .as_ref()
            .is_some_and(|owner| owner.0.validate_preserved().is_err())
        {
            return Err(Fault::InvalidPlayerEntry);
        }
        if !self.hud_map.holds(self)
            || !self.countdown.holds(self)
            || !self.maintenance.holds(self)
            || !self.play_time.holds(self)
            || !self.room_quake.holds(self)
        {
            return Err(Fault::InvalidPlayerEntry);
        }
        if let Some(world) = &self.inside_world {
            world.0.validate().map_err(Fault::InsideWorld)?;
        }
        self.gameplay_publications()
            .map_err(|_| Fault::InvalidPlayerEntry)?;
        self.timing.validate().map_err(Fault::Timing)?;
        self.content.validate().map_err(Fault::Content)?;
        if self.samus_palette.is_some() {
            super::validate_regions(self.transfer_regions().iter()).map_err(Fault::Content)?;
        }
        self.publications.validate().map_err(Fault::Publications)?;
        self.appearance.validate().map_err(Fault::Appearance)?;
        self.graphical_contacts
            .validate(&self.appearance)
            .map_err(Fault::Rendering)?;
        if let Some(world) = &self.world {
            world.0.validate().map_err(Fault::World)?;
        }
        self.senders.validate().map_err(Fault::Sender)?;
        if self.sender_current.is_some() != self.sender_queues.is_some() {
            return Err(Fault::SenderContract(
                super::sender_contract::Fault::Current,
            ));
        }
        if let Some(queues) = &self.sender_queues {
            queues
                .validate(&self.senders)
                .map_err(Fault::SenderContract)?;
        }
        if let Some(current) = self.sender_current {
            current.validate().map_err(Fault::SenderContract)?;
        }
        self.sender_stage
            .validate(&self.senders)
            .map_err(Fault::SenderStage)?;
        self.sound_offers
            .validate(&self.appearance)
            .map_err(Fault::SenderStage)
    }

    pub(super) fn transfer_regions(&self) -> Vec<data::timing::ContentRegion> {
        let mut regions = self.content.regions();
        regions.extend(
            self.samus_palette
                .as_ref()
                .map(super::samus_palette::Palette::region),
        );
        regions
    }

    /// Advance admitted hardware and retain every actual output atomically.
    /// The budget chooses only a pause; hardware determines the resulting wall time.
    ///
    /// # Errors
    /// Invalid current owners, physical mechanism or publication order.
    pub fn step_hardware(&mut self, budget: MasterTicks) -> Result<timing::Progress, Fault> {
        let mut candidate = self.clone();
        candidate.validate()?;
        let (progress, events) = candidate
            .timing
            .step(budget, &candidate.transfer_regions())
            .map_err(Fault::Timing)?;
        candidate
            .publications
            .record(events)
            .map_err(Fault::Publications)?;
        *self = candidate;
        Ok(progress)
    }

    /// Advance one actual sender service on a complete private frame candidate.
    /// Returned semantic milestones are bound to hardware by the frame caller.
    ///
    /// # Errors
    /// Invalid ownership, reversed time or an unadmitted reply rejects all changes.
    pub fn sender(
        &mut self,
        group: SoundGroup,
        input: SenderInput<'_>,
    ) -> Result<Vec<audio_transport::sender::SenderOutput>, Fault> {
        let mut candidate = self.clone();
        candidate.validate()?;
        let outputs =
            crate::production::sender::advance(candidate.senders.current(group), group, input)
                .map_err(Fault::Production)?;
        *self = candidate;
        Ok(outputs)
    }
}
