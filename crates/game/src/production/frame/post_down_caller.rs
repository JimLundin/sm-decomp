//! Distinct actual Contact, transport and poststep completion boundaries.

use super::{Caller, Content, Dependency, State, appearance_caller, contact_contract};
use crate::production::{ProductionFault, Unsupported, timing::WorkId};
use canonical::{Canonical, model::Model};

/// Actual post-Down owners, retaining readonly producer snapshots while charging.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// Actual Down has not yet supplied the Contact opportunity.
    #[default]
    Idle,
    /// The whole Contact owner is charging; neither consumer is published early.
    Contact {
        /// Actual retained Appearance choice.
        appearance: appearance_caller::Stage,
        /// Actual completed ordinary Movement at this opportunity.
        movement: super::movement::Movement,
        /// Actual completed Down producer.
        down: super::movement_down_contract::Completion,
        /// Fresh source-qualified current owners retained through billing.
        current: Model<contact_contract::CurrentPin>,
        /// Actual graphical consumer before the physical owner.
        graphics_before: super::rendering_stage::Contacts,
        /// Actual Part2 choice from this Contact's completed result.
        graphics_chosen: super::rendering_stage::Contacts,
    },
    /// Actual Contact and Part2 completed; transport has not enqueued the sound.
    ContactComplete(ContactCompleted),
    /// Actual transport enqueue is chosen privately while its whole envelope charges.
    SoundOffer {
        /// Actual completed Contact supplying this sole request.
        contact: ContactCompleted,
        /// Current appearance output stream before this consumer.
        appearance: crate::production::appearance::Work,
        /// Actual transport owners before enqueue.
        senders_before: super::Senders,
        /// Actual output consumer before enqueue.
        offers_before: super::sender_stage::Offers,
        /// Actual physical ring metadata before enqueue.
        queues_before: super::sender_contract::Queues,
        /// Actual privately chosen transport result.
        senders_chosen: super::Senders,
        /// Actual privately chosen consumer result.
        offers_chosen: super::sender_stage::Offers,
        /// Actual privately chosen ring advancement.
        queues_chosen: super::sender_contract::Queues,
        /// Actual enabled/demo/power-bomb owners at this opportunity.
        current: super::post_down_contract::OfferCurrent,
    },
    /// Actual transport enqueue returned before Poststep begins.
    SoundComplete(SoundCompleted),
    /// Actual ordinary travel and Stage0 echo disposition await Poststep return.
    Poststep {
        /// Actual preceding completed producers.
        sound: SoundCompleted,
        /// Actual borrowed Stage0 appearance owner at this opportunity.
        appearance: crate::production::appearance::Work,
    },
    /// Actual Poststep returned, before the `HUDMap` consumer.
    Complete(PoststepCompleted),
}

/// Actual completed Contact and its ordinary producer provenance.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct ContactCompleted {
    /// Matching returned Contact producer and actual result.
    completion: appearance_caller::Completion,
    /// Actual completed ordinary travel and centre, before any Aim work.
    movement: super::movement::Movement,
    /// Actual completed Down provenance.
    down: super::movement_down_contract::Completion,
    /// Current Contact owners retained for the next actual consumer.
    current: Model<contact_contract::CurrentPin>,
}

/// Actual distinct `SoundOffer` return and its once-consumed transport output.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct SoundCompleted {
    /// Actual matching physical identity returned by transport enqueue.
    work: WorkId,
    /// Actual Contact supplying the request.
    contact: ContactCompleted,
    /// Actual completed transport queues and progress.
    senders: super::Senders,
    /// Actual completed sound consumer position.
    offers: super::sender_stage::Offers,
    /// Actual completed physical ring advancement.
    queues: super::sender_contract::Queues,
}

/// Actual distinct Poststep return, before `HUDMap` uses ordinary travel/centre.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct PoststepCompleted {
    /// Actual matching physical identity returned by Poststep.
    work: WorkId,
    /// Actual completed upstream producer chain.
    sound: SoundCompleted,
    /// Actual borrowed echo disposition consumed by this Stage0 return.
    appearance: crate::production::appearance::Work,
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(
        Dependency::PlayerContinuationPreparation,
    ))
}

impl Stage {
    pub(super) fn owners_hold(&self, state: &State) -> bool {
        match self {
            Self::Idle => true,
            Self::Contact {
                appearance,
                movement,
                down,
                current,
                graphics_before,
                graphics_chosen,
            } => {
                appearance.validate(state).is_ok()
                    && state.movement == *movement
                    && state.contact_current.as_ref() == Some(current)
                    && state.graphical_contacts == *graphics_before
                    && matches!(&state.caller, Caller::PlayerContinuation {
                        down: super::movement_down_contract::Stage::Complete(actual), ..
                    } if actual == down)
                    && appearance
                        .receipt()
                        .is_some_and(|receipt| graphics_chosen.validate(&receipt.result).is_ok())
            }
            Self::ContactComplete(contact) => contact.holds(state).is_ok(),
            Self::SoundOffer {
                contact,
                appearance,
                senders_before,
                offers_before,
                queues_before,
                current,
                ..
            } => {
                contact.holds(state).is_ok()
                    && state.appearance == *appearance
                    && state.senders == *senders_before
                    && state.sound_offers == *offers_before
                    && state.sender_queues.as_ref() == Some(queues_before)
                    && state.sound_offer_current == Some(*current)
            }
            Self::SoundComplete(sound) => sound.holds(state).is_ok(),
            Self::Poststep { sound, appearance } => {
                sound.holds(state).is_ok() && state.appearance == *appearance
            }
            Self::Complete(poststep) => {
                poststep.work == WorkId(1106)
                    && poststep.sound.holds(state).is_ok()
                    && poststep.appearance == state.appearance
                    && poststep.appearance.model.0.state.stage == types::BoostStage::Stage0
            }
        }
    }

    /// Actual distinct identities already returned in causal producer order.
    #[must_use]
    pub fn completed_identities(&self) -> Vec<WorkId> {
        match self {
            Self::Idle | Self::Contact { .. } => Vec::new(),
            Self::ContactComplete(contact) | Self::SoundOffer { contact, .. } => {
                vec![contact.completion.work]
            }
            Self::SoundComplete(sound) | Self::Poststep { sound, .. } => {
                vec![sound.contact.completion.work, sound.work]
            }
            Self::Complete(poststep) => vec![
                poststep.sound.contact.completion.work,
                poststep.sound.work,
                poststep.work,
            ],
        }
    }

    /// Actual current physical identity, absent between completed owners.
    #[must_use]
    pub const fn charging_identity(&self) -> Option<WorkId> {
        match self {
            Self::Contact { .. } => Some(appearance_caller::Kind::Contact.identity()),
            Self::SoundOffer { .. } => Some(WorkId(1105)),
            Self::Poststep { .. } => Some(WorkId(1106)),
            Self::Idle | Self::ContactComplete(_) | Self::SoundComplete(_) | Self::Complete(_) => {
                None
            }
        }
    }
}

impl ContactCompleted {
    fn holds(&self, state: &State) -> Result<(), ProductionFault> {
        self.completion.receipt.validate().map_err(|_| missing())?;
        let Caller::PlayerContinuation {
            down: super::movement_down_contract::Stage::Complete(down),
            ..
        } = &state.caller
        else {
            return Err(missing());
        };
        if self.completion.work != appearance_caller::Kind::Contact.identity()
            || self.down != *down
            || self.movement != state.movement
            || self.completion.receipt.result.model != state.appearance.model
            || state.graphical_contacts.consumed()
                != u64::try_from(state.appearance.model.0.graphics.len()).map_err(|_| missing())?
        {
            return Err(missing());
        }
        Ok(())
    }
}

impl SoundCompleted {
    fn holds(&self, state: &State) -> Result<(), ProductionFault> {
        self.contact.holds(state)?;
        if self.work != WorkId(1105)
            || self.senders != state.senders
            || self.offers != state.sound_offers
            || state.sender_queues.as_ref() != Some(&self.queues)
        {
            return Err(missing());
        }
        Ok(())
    }
}

impl State {
    pub(super) fn begin_footstep_offer(
        &mut self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let Caller::PlayerContinuation {
            post_down: Stage::ContactComplete(contact),
            charging: false,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        contact.holds(self)?;
        let current = self.sound_offer_current.ok_or_else(missing)?;
        let permission = current.permission().ok_or_else(missing)?;
        let result = contact
            .completion
            .receipt
            .result
            .model
            .0
            .completed
            .last()
            .ok_or_else(missing)?;
        let samus_appearance::Receipt::Appearance { output, .. } = result else {
            return Err(missing());
        };
        let sound = output.sound.ok_or_else(missing)?;
        if sound != samus_appearance::SoundRequest::Footstep
            || sound.queue() != samus_appearance::SoundQueue::Footsteps
            || sound.max_waiting() != 6
        {
            return Err(missing());
        }
        let profile = content
            .post_down
            .as_ref()
            .ok_or_else(missing)?
            .sound_offer(current)
            .map_err(|_| missing())?;
        let mut candidate = self.clone();
        super::sender_caller::deliver_appearance(&mut candidate, permission)
            .map_err(|_| missing())?;
        let retained = Stage::SoundOffer {
            contact: contact.clone(),
            appearance: self.appearance.clone(),
            current,
            senders_before: self.senders.clone(),
            offers_before: self.sound_offers.clone(),
            queues_before: self.sender_queues.clone().ok_or_else(missing)?,
            senders_chosen: candidate.senders.clone(),
            offers_chosen: candidate.sound_offers.clone(),
            queues_chosen: candidate.sender_queues.clone().ok_or_else(missing)?,
        };
        self.timing
            .begin(WorkId(1105), profile)
            .map_err(ProductionFault::FrameHardware)?;
        let Caller::PlayerContinuation {
            post_down,
            charging,
            ..
        } = &mut self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        *post_down = retained;
        *charging = true;
        Ok(())
    }

    pub(super) fn validate_footstep_offer(&self, content: &Content) -> Result<(), ProductionFault> {
        let Caller::PlayerContinuation {
            post_down:
                Stage::SoundOffer {
                    contact,
                    appearance,
                    senders_before,
                    offers_before,
                    queues_before,
                    senders_chosen,
                    offers_chosen,
                    queues_chosen,
                    current,
                },
            charging: true,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        contact.holds(self)?;
        if self.appearance != *appearance
            || self.senders != *senders_before
            || self.sound_offers != *offers_before
            || self.sender_queues.as_ref() != Some(queues_before)
            || self.sound_offer_current != Some(*current)
        {
            return Err(missing());
        }
        let first = usize::try_from(offers_before.consumed()).map_err(|_| missing())?;
        if appearance.model.0.sounds.get(first..)
            != Some(&[samus_appearance::PublishedSound::Appearance(
                samus_appearance::SoundRequest::Footstep,
            )])
            || offers_chosen.consumed()
                != offers_before
                    .consumed()
                    .checked_add(1)
                    .ok_or_else(missing)?
            || senders_chosen.action != senders_before.action
            || senders_chosen.ambient != senders_before.ambient
        {
            return Err(missing());
        }
        offers_chosen.validate(appearance).map_err(|_| missing())?;
        let mut queues = queues_before.clone();
        queues
            .footstep(
                &senders_before.contact.0,
                &senders_chosen.contact.0,
                current.permission().ok_or_else(missing)?,
            )
            .map_err(|_| missing())?;
        if queues != *queues_chosen {
            return Err(missing());
        }
        content
            .post_down
            .as_ref()
            .ok_or_else(missing)?
            .sound_offer(*current)
            .map_err(|_| missing())?;
        Ok(())
    }

    pub(super) fn complete_footstep_offer(
        &mut self,
        work: WorkId,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        self.validate_footstep_offer(content)?;
        if work != WorkId(1105) {
            return Err(missing());
        }
        let Caller::PlayerContinuation {
            post_down:
                Stage::SoundOffer {
                    contact,
                    senders_chosen,
                    offers_chosen,
                    queues_chosen,
                    ..
                },
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        let mut candidate = self.clone();
        candidate.senders = senders_chosen.clone();
        candidate.sound_offers = offers_chosen.clone();
        candidate.sender_queues = Some(queues_chosen.clone());
        let retained = Stage::SoundComplete(SoundCompleted {
            work,
            contact: contact.clone(),
            senders: senders_chosen.clone(),
            offers: offers_chosen.clone(),
            queues: queues_chosen.clone(),
        });
        let Caller::PlayerContinuation {
            post_down,
            charging,
            ..
        } = &mut candidate.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        *post_down = retained;
        *charging = false;
        *self = candidate;
        Ok(())
    }

    pub(super) fn begin_poststep(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let Caller::PlayerContinuation {
            post_down: Stage::SoundComplete(sound),
            charging: false,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        sound.holds(self)?;
        // The retained actual Movement contains both travels and centre; later Aim never supplies either.
        self.movement.outputs().ok_or_else(missing)?;
        let profile = content
            .post_down
            .as_ref()
            .ok_or_else(missing)?
            .poststep(self.appearance.model.0.state.stage)
            .map_err(|_| missing())?;
        let retained = Stage::Poststep {
            sound: sound.clone(),
            appearance: self.appearance.clone(),
        };
        self.timing
            .begin(WorkId(1106), profile)
            .map_err(ProductionFault::FrameHardware)?;
        let Caller::PlayerContinuation {
            post_down,
            charging,
            ..
        } = &mut self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        *post_down = retained;
        *charging = true;
        Ok(())
    }

    pub(super) fn validate_poststep(&self, content: &Content) -> Result<(), ProductionFault> {
        let Caller::PlayerContinuation {
            post_down: Stage::Poststep { sound, appearance },
            charging: true,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        sound.holds(self)?;
        if self.appearance != *appearance {
            return Err(missing());
        }
        content
            .post_down
            .as_ref()
            .ok_or_else(missing)?
            .poststep(self.appearance.model.0.state.stage)
            .map_err(|_| missing())?;
        Ok(())
    }

    pub(super) fn complete_poststep(
        &mut self,
        work: WorkId,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        self.validate_poststep(content)?;
        if work != WorkId(1106) {
            return Err(missing());
        }
        let Caller::PlayerContinuation {
            post_down: Stage::Poststep { sound, appearance },
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        let retained = Stage::Complete(PoststepCompleted {
            work,
            sound: sound.clone(),
            appearance: appearance.clone(),
        });
        let Caller::PlayerContinuation {
            post_down,
            charging,
            ..
        } = &mut self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        *post_down = retained;
        *charging = false;
        Ok(())
    }

    pub(super) fn begin_stride_contact(
        &mut self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let room = self.validate_room_owner(content)?;
        let Caller::PlayerContinuation {
            down: super::movement_down_contract::Stage::Complete(down),
            post_down: Stage::Idle,
            charging: false,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        let down = down.clone();
        let current_model = *self.contact_current.as_ref().ok_or_else(missing)?;
        let current = current_model.0;
        if current.context.room != room {
            return Err(missing());
        }
        let mut candidate = self.clone();
        let mut appearance = appearance_caller::Stage::Idle;
        appearance
            .contact(
                &mut candidate,
                &content.appearance,
                self.playing_figure.0,
                current.suppressors,
            )
            .map_err(|_| missing())?;
        let qualified = content
            .contact
            .as_ref()
            .ok_or_else(missing)?
            .qualify(&appearance, &candidate, &content.appearance, current)
            .map_err(|_| missing())?;
        let work = appearance_caller::Kind::Contact.identity();
        let mut graphics_chosen = self.graphical_contacts.clone();
        super::rendering_stage::choose_contacts(
            &mut graphics_chosen,
            &qualified.receipt.result,
            super::rendering_stage::Part2Premises {
                dry_ceres: current.context.area == types::Area::Ceres
                    && current.context.water == contact_contract::Surface::Absent
                    && current.context.lava == contact_contract::Surface::Absent,
            },
            work,
        )
        .map_err(|_| missing())?;
        candidate
            .timing
            .begin(work, qualified.profile)
            .map_err(ProductionFault::FrameHardware)?;
        appearance.bind(work).map_err(|_| missing())?;
        let retained = Stage::Contact {
            appearance,
            movement: self.movement.clone(),
            down,
            current: current_model,
            graphics_before: self.graphical_contacts.clone(),
            graphics_chosen,
        };
        let Caller::PlayerContinuation {
            post_down,
            charging,
            ..
        } = &mut candidate.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        *post_down = retained;
        *charging = true;
        *self = candidate;
        Ok(())
    }

    pub(super) fn validate_stride_contact(&self, content: &Content) -> Result<(), ProductionFault> {
        let room = self.validate_room_owner(content)?;
        let Caller::PlayerContinuation {
            down: super::movement_down_contract::Stage::Complete(actual_down),
            post_down:
                Stage::Contact {
                    appearance,
                    movement,
                    down,
                    current,
                    graphics_before,
                    graphics_chosen,
                },
            charging: true,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        if self.movement != *movement
            || actual_down != down
            || self.contact_current.as_ref() != Some(current)
            || self.graphical_contacts != *graphics_before
        {
            return Err(missing());
        }
        if current.0.context.room != room {
            return Err(missing());
        }
        let qualified = content
            .contact
            .as_ref()
            .ok_or_else(missing)?
            .qualify(appearance, self, &content.appearance, current.0)
            .map_err(|_| missing())?;
        let mut graphics = graphics_before.clone();
        super::rendering_stage::choose_contacts(
            &mut graphics,
            &qualified.receipt.result,
            super::rendering_stage::Part2Premises {
                dry_ceres: current.0.context.area == types::Area::Ceres
                    && current.0.context.water == contact_contract::Surface::Absent
                    && current.0.context.lava == contact_contract::Surface::Absent,
            },
            appearance_caller::Kind::Contact.identity(),
        )
        .map_err(|_| missing())?;
        if graphics != *graphics_chosen {
            return Err(missing());
        }
        Ok(())
    }

    pub(super) fn complete_stride_contact(
        &mut self,
        work: WorkId,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        self.validate_stride_contact(content)?;
        let Caller::PlayerContinuation {
            post_down:
                Stage::Contact {
                    appearance,
                    movement,
                    down,
                    current,
                    graphics_chosen,
                    ..
                },
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        let qualified = content
            .contact
            .as_ref()
            .ok_or_else(missing)?
            .qualify(appearance, self, &content.appearance, current.0)
            .map_err(|_| missing())?;
        let mut candidate = self.clone();
        let mut appearance = appearance.clone();
        let (completion, outputs) = appearance
            .complete(&mut candidate, work)
            .map_err(|_| missing())?;
        qualified
            .completed(&completion, &outputs)
            .map_err(|_| missing())?;
        qualified
            .graphical(
                qualified.graphic,
                current.0.context,
                candidate.appearance.model.0.state.stage,
            )
            .map_err(|_| missing())?;
        let mut graphics = graphics_chosen.clone();
        super::rendering_stage::consume_contacts(&mut graphics, work).map_err(|_| missing())?;
        candidate.graphical_contacts = graphics;
        let retained = Stage::ContactComplete(ContactCompleted {
            completion,
            movement: movement.clone(),
            down: down.clone(),
            current: *current,
        });
        let Caller::PlayerContinuation {
            post_down,
            charging,
            ..
        } = &mut candidate.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        *post_down = retained;
        *charging = false;
        *self = candidate;
        Ok(())
    }
}
