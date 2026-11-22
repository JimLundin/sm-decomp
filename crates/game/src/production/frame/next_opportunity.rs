//! Open the next software opportunity after the actual native return.

use super::{
    Caller, Content, Dependency, PreludeSource, State,
    borrowed_receipts::{FrameContinuationAfterNmi, PublicationOrigin, SoftwareRequest},
    frame_return::Stage,
    suffix_caller::Suffix,
};
use crate::production::{ProductionFault, Unsupported};
use canonical::model::Model;

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(
        Dependency::FramePublicationAcknowledgment,
    ))
}

impl State {
    pub(super) fn begin_next_opportunity(
        &mut self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let Stage::Ready {
            builder,
            suffix: Suffix::Acknowledged { controller },
            ..
        } = &self.return_progress
        else {
            return Ok(());
        };
        // The first interval qualifies the hurt-flash store only through native return.
        // Its later reader and visual effects need their own Drawing owner.
        if self.s6_drawing.deferred_hurt_flash() {
            return Err(ProductionFault::Unsupported(Unsupported::Frame(
                Dependency::DrawingPreparation,
            )));
        }
        let (_, native_admitted) = self.gameplay_publications()?;
        self.validate_cannon_history()?;
        let controller = controller.word().ok_or_else(missing)?;
        let history = self.frame_history.as_ref().ok_or_else(missing)?;
        let play_time = self.play_time.current().ok_or_else(missing)?;
        let room_quake = self.room_quake.current().ok_or_else(missing)?;
        if self.movement.finished().is_none()
            || !matches!(self.late_hud, super::late_hud::Stage::Retired)
            || history
                .native_controller()
                .is_none_or(|[held, _]| held != controller)
            || self
                .health
                .as_ref()
                .is_some_and(|health| health.validate_current().is_err())
        {
            return Err(missing());
        }
        self.validate_carried_health(content).map_err(|_| {
            ProductionFault::Unsupported(Unsupported::Frame(Dependency::HealthCurrentInputs))
        })?;
        let scene = self.scene_after_native_return(content, native_admitted)?;
        self.publications
            .begin_interval()
            .map_err(ProductionFault::FramePublications)?;
        self.frame_history
            .as_mut()
            .ok_or_else(missing)?
            .begin_opportunity();
        self.sprite_builder = Some(builder.clone());
        let carry_tail = content
            .interruption
            .as_ref()
            .is_some_and(super::interruption_caller::Contract::admits_adjacent);
        self.retire_work(carry_tail)?;
        self.scene_countdown =
            scene.map(|owner| Model(owner.begin_adjacent(self.publications.interval_start())));
        self.play_time = super::play_time::Stage::from_origin(play_time);
        self.room_quake = super::room_quake::Stage::from_origin(room_quake);
        self.caller = Caller::AwaitingAcknowledgment {
            prelude: PreludeSource::Recurring,
            receipt: Some(FrameContinuationAfterNmi {
                publication: PublicationOrigin {
                    prior_request: SoftwareRequest::Acknowledged,
                },
            }),
        };
        Ok(())
    }

    fn retire_work(&mut self, carry_tail: bool) -> Result<(), ProductionFault> {
        self.appearance.retire_frame().map_err(|_| missing())?;
        self.movement = super::movement::Movement::new(*self.movement.state());
        self.hud = super::hud::Hud::new(self.hud.state().clone());
        if let Some(health) = &mut self.health {
            health.retire_opportunity();
        }
        if let Some(liquid) = &mut self.liquid_history {
            liquid.retire_opportunity();
        }
        if let Some(palette) = &mut self.samus_palette {
            palette.retire_opportunity();
        }
        if let Some(drawing) = &mut self.drawing {
            drawing.begin_opportunity();
        }
        if let Some(owners) = &mut self.recurring_player {
            owners.completed_door_input = None;
            owners
                .doors
                .current_eligibility(self.actor_eligibility.as_mut().ok_or_else(missing)?)
                .ok_or_else(missing)?;
        }
        self.movement_stage = super::movement_stage::Stage::default();
        self.inside_inputs = None;
        self.inside_pending = None;
        self.inside_completed = None;
        self.graphical_contacts = super::rendering_stage::Contacts::default();
        self.rendering = super::rendering_stage::Stage::default();
        if !carry_tail {
            self.drawing_origins = None;
            self.room_tail_origin = None;
        } else if !matches!(self.room_tail, super::room_tail::Stage::Completed(_))
            && let Some(room) = self.room_tail_origin.as_mut()
        {
            room.departure_inactive = None;
        }
        self.sound_offer_current = None;
        self.movement_inputs = None;
        self.world = None;
        self.prepared_action = None;
        self.hud_display_current = None;
        self.hud_map = super::hud_map_caller::Stage::default();
        self.maintenance = super::maintenance::Stage::default();
        self.s6_camera = super::s6_camera::Stage::default();
        self.s6_drawing = super::s6_drawing::Stage::default();
        self.background = super::suffix_caller::BackgroundScroll::default();
        self.return_progress = Stage::Pending;
        self.sender_progress = super::sender_caller::Cursor::default();
        self.sender_history = None;
        self.late_hud = super::late_hud::Stage::default();
        self.room_tail = super::room_tail::Stage::default();
        self.frame_tail = super::frame_tail::Stage::default();
        self.animation = super::animation_caller::Stage::default();
        self.interruption = super::interruption_caller::Stage::default();
        self.late_movement = super::late_movement::Stage::default();
        self.countdown = super::countdown_caller::Stage::default();
        self.countdown_delivered.clear();
        self.sender_stage = super::sender_stage::Stage::default();
        self.completed_contact_read = None;
        self.sound_offers = super::sender_stage::Offers::default();
        self.content.begin_opportunity();
        Ok(())
    }
}
