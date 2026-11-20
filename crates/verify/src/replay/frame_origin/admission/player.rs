//! Load selected player motion, contact and pose contracts from current owners.

use super::super::{portion, validate_haze_ready};
use super::Admission;
use crate::lock::Sha256;
use crate::replay::{self, data_failure, result::Outcome};
use canonical::model::Model;
use game::production::frame;

pub(super) struct RequiredProfiles {
    pub(super) acknowledgment: data::timing::Template,
    pub(super) music: data::timing::Template,
    pub(super) haze: data::timing::Template,
    pub(super) haze_tail: data::timing::Template,
    pub(super) layer_blend: data::timing::Template,
    pub(super) preparation_content: frame::PreparationContent,
    pub(super) player_entry: frame::PlayerEntryContent,
}

pub(super) struct MovementGeometry {
    pub(super) movement_room: Option<frame::movement_caller::Room>,
    pub(super) ordinary_right: Option<frame::movement_pair_contract::PhysicalContract>,
    pub(super) compound_support: Option<frame::movement_support_contract::PhysicalContract>,
}

impl Admission<'_> {
    pub(super) fn required_profiles(
        &self,
        reference_acceleration: types::Fixed,
    ) -> Result<RequiredProfiles, Outcome> {
        let Self { templates, .. } = *self;
        validate_haze_ready(templates)?;
        let acknowledgment = portion(templates, "FrameContinuationAfterNmi")?;
        let music = portion(templates, "Music")?;
        let haze = portion(templates, "Haze")?;
        let haze_tail = portion(templates, "HazeBackdropBandsReady")?;
        let layer_blend = portion(templates, "LayerBlend")?;
        let preparation_content = frame::PreparationContent {
            builder: portion(templates, "BeginSpriteList")?,
            gameplay: portion(templates, "GameplayDispatch")?,
            actors: portion(templates, "ActorEligibility")?,
            debug: portion(templates, "Debug")?,
            palette: portion(templates, "PaletteObjects")?,
        };
        let player_entry = frame::PlayerEntryContent {
            pose_input: portion(templates, "S2.PoseInput")?,
            suit_palette: portion(templates, "S2.SuitPalette")?,
            environment: portion(templates, "S2.Environment")?,
            reference_acceleration,
        };
        Ok(RequiredProfiles {
            acknowledgment,
            music,
            haze,
            haze_tail,
            layer_blend,
            preparation_content,
            player_entry,
        })
    }

    pub(super) fn player_continuation(
        &self,
    ) -> Result<Option<frame::player_continuation::Content>, Outcome> {
        let Self {
            manifest,
            templates,
            ..
        } = *self;
        let player_continuation = if manifest.player_continuation_current.is_some() {
            Some(frame::player_continuation::Content {
                projectiles: portion(templates, "S2.Projectiles")?,
                interaction: portion(templates, "S2.SamusInteraction")?,
                actors: portion(templates, "S2.ActorUpdate")?,
                sprites: portion(templates, "S2.SpriteObjects")?,
                contact: portion(templates, "S2.ContactReset")?,
            })
        } else {
            None
        };
        Ok(player_continuation)
    }

    pub(super) fn motion_reset(
        &self,
        entry_labels: &str,
    ) -> Result<Option<frame::MotionResetContent>, Outcome> {
        let Self {
            manifest,
            wram,
            templates,
            ..
        } = *self;
        let motion_reset = if let Some(member) = &manifest.motion_reset_current {
            let accepted = Sha256::try_from(
                "cb8c715ada04990e6bae72b4e63e7ea88e2ca02ac46d160b07451b75953e7254",
            )
            .map_err(|_| data_failure(None, "frame full MotionReset source identity"))?;
            if member.pin()? != accepted {
                return Err(data_failure(None, "frame full MotionReset source identity"));
            }
            Some(frame::MotionResetContent {
                facing: replay::frame_motion_reset_contract::contract(
                    wram,
                    &member.text()?,
                    entry_labels,
                )
                .map_err(|_| {
                    data_failure(None, "frame full MotionReset current and selected bindings")
                })?,
                whole: replay::frame_motion_reset_contract::whole_template(templates)
                    .map_err(|_| data_failure(None, "frame whole MotionReset physical owner"))?,
            })
        } else {
            None
        };
        Ok(motion_reset)
    }

    pub(super) fn run(&self) -> Result<Option<frame::movement_caller::RunContract>, Outcome> {
        let Self {
            manifest,
            data: content,
            templates,
            ..
        } = *self;
        let run = if let Some(member) = &manifest.selected_stage_predicates {
            let accepted = Sha256::try_from(
                "0b57c1ef79d6497f2a8abe2e9aed13029f59bdd6e16dd657e06e1ee43e62f251",
            )
            .map_err(|_| data_failure(None, "frame selected stage source identity"))?;
            if member.pin()? != accepted {
                return Err(data_failure(None, "frame selected stage source identity"));
            }
            Some(
                replay::frame_movement_owner::contract(content, templates, &member.text()?)
                    .map_err(|_| data_failure(None, "frame complete local Run predicates"))?,
            )
        } else {
            None
        };
        Ok(run)
    }

    pub(super) fn movement_geometry(&self) -> Result<MovementGeometry, Outcome> {
        let Self {
            manifest,
            wram,
            data: content,
            templates,
            ..
        } = *self;
        let (movement_room, ordinary_right, compound_support) = if let Some(member) =
            &manifest.movement_source_admission
        {
            let accepted = Sha256::try_from(
                "95f5550d884a19f507328c5c27cd7d211f8672903ece0db7a27d234f4558ac2c",
            )
            .map_err(|_| data_failure(None, "frame movement source admission identity"))?;
            if member.pin()? != accepted {
                return Err(data_failure(
                    None,
                    "frame movement source admission identity",
                ));
            }
            let (room, owner) = replay::frame_movement_pair_owner::ordinary_right(
                wram,
                content,
                templates,
                &member.text()?,
            )
            .map_err(|_| data_failure(None, "frame whole ordinary current query and arithmetic"))?;
            let (support_room, support) = replay::frame_movement_support_owner::contract(
                wram,
                content,
                templates,
                &member.text()?,
            )
            .map_err(|_| {
                data_failure(
                    None,
                    "frame compound support current queries and arithmetic",
                )
            })?;
            if room != support_room {
                return Err(data_failure(None, "frame shared movement room"));
            }
            (Some(room), Some(owner), Some(support))
        } else {
            (None, None, None)
        };
        Ok(MovementGeometry {
            movement_room,
            ordinary_right,
            compound_support,
        })
    }

    pub(super) fn bind_inside(
        &self,
        state: &mut frame::State,
        appearance: &replay::appearance::Content,
    ) -> Result<Option<frame::inside_contract::PhysicalContract>, Outcome> {
        let Self {
            manifest,
            wram,
            data: content,
            templates,
            ..
        } = *self;
        let inside_physical = if let Some(member) = &manifest.inside_current_physical {
            let player = manifest.player_current.as_ref().ok_or_else(|| {
                data_failure(None, "frame complete current Inside external owners")
            })?;
            let external =
                replay::frame_player_inputs::qualified(wram, &player.text()?).map_err(|_| {
                    data_failure(None, "frame independently current Inside external owners")
                })?;
            let (room, contract) =
                replay::frame_inside_contract::contract(wram, content, templates, &member.text()?)
                    .map_err(|_| {
                        data_failure(None, "frame complete current Inside physical contract")
                    })?;
            let ahead = state
                .current_animation(&appearance.appearance)
                .map_err(|_| data_failure(None, "frame actual pure current Playing bookkeeping"))?;
            state.inside_inputs = Some(Model(
                external.movement(state.playing_figure.0.sprite, ahead),
            ));
            state.inside_world = Some(Model(room));
            Some(contract)
        } else {
            None
        };
        Ok(inside_physical)
    }

    pub(super) fn early_hud_profile(
        &self,
        has_controls: bool,
    ) -> Result<Option<data::timing::Template>, Outcome> {
        let Self { templates, .. } = *self;
        let early_hud_owner = if has_controls {
            Some(
                replay::frame_hud_controls_owner::whole_template(templates)
                    .map_err(|_| data_failure(None, "frame complete early HUD controls owner"))?,
            )
        } else {
            None
        };
        Ok(early_hud_owner)
    }

    pub(super) fn ground_following_down(
        &self,
    ) -> Result<Option<frame::movement_down_contract::PhysicalContract>, Outcome> {
        let Self {
            manifest,
            wram,
            data: content,
            templates,
            ..
        } = *self;
        let ground_following_down = match (
            &manifest.movement_down_current,
            &manifest.movement_source_admission,
        ) {
            (Some(down), Some(room)) => Some(
                replay::frame_movement_down_owner::contract(
                    wram,
                    content,
                    templates,
                    &down.text()?,
                    &room.text()?,
                )
                .map_err(|_| {
                    data_failure(
                        None,
                        "frame whole downward current query and live arithmetic",
                    )
                })?,
            ),
            (None, _) => None,
            (Some(_), None) => {
                return Err(data_failure(None, "frame downward current room contract"));
            }
        };
        Ok(ground_following_down)
    }

    pub(super) fn bind_contact(
        &self,
        state: &mut frame::State,
    ) -> Result<Option<frame::contact_contract::PhysicalContract>, Outcome> {
        let Self {
            manifest,
            wram,
            templates,
            ..
        } = *self;
        let contact = match (&manifest.contact_current, &manifest.appearance_neighbors) {
            (Some(contact), Some(neighbors)) => {
                let accepted = Sha256::try_from(
                    "ee0f4c5816570843d001dde8103d97aada9b2a225f59ce889d811dd72e09be7b",
                )
                .map_err(|_| data_failure(None, "frame appearance neighbour source identity"))?;
                if neighbors.pin()? != accepted {
                    return Err(data_failure(
                        None,
                        "frame appearance neighbour source identity",
                    ));
                }
                let suppressors =
                    replay::frame_appearance_inputs::qualified_contact(wram, &neighbors.text()?)
                        .map_err(|_| {
                            data_failure(None, "frame current Contact suppressor origins")
                        })?;
                let key = data::room::RoomKey::from_room_pointer(
                    crate::source::ROOM_PTR
                        .read(wram)
                        .map_err(|_| data_failure(None, "frame current Contact room origin"))?,
                );
                let room = import::world::id(key)
                    .ok_or_else(|| data_failure(None, "frame checked Contact room binding"))?;
                state.contact_current = Some(Model(
                    replay::frame_contact_owner::current_origin(
                        wram,
                        &contact.text()?,
                        suppressors,
                        room,
                        state
                            .sender_current
                            .ok_or_else(|| data_failure(None, "frame current Contact bus owner"))?
                            .bus,
                    )
                    .map_err(|_| {
                        data_failure(None, "frame current Contact room and liquid owners")
                    })?,
                ));
                Some(
                    replay::frame_contact_owner::physical_contract(templates)
                        .map_err(|_| data_failure(None, "frame whole Contact physical owner"))?,
                )
            }
            (None, None) => None,
            _ => return Err(data_failure(None, "frame paired Contact source members")),
        };
        Ok(contact)
    }

    pub(super) fn bind_post_down(
        &self,
        state: &mut frame::State,
        has_contact: bool,
    ) -> Result<Option<frame::post_down_contract::Physical>, Outcome> {
        let Self {
            manifest,
            wram,
            image,
            templates,
            ..
        } = *self;
        let post_down = if has_contact {
            let proof = manifest
                .sender_current_profile
                .as_ref()
                .ok_or_else(|| data_failure(None, "frame current SoundOffer source"))?;
            state.sound_offer_current = Some(
                replay::frame_sender_contract::offer_current(&proof.text()?, wram, image).map_err(
                    |_| {
                        data_failure(
                            None,
                            "frame current SoundOffer enabled scene and power-bomb origins",
                        )
                    },
                )?,
            );
            Some(
                replay::frame_post_down_owner::physical(templates).map_err(|_| {
                    data_failure(None, "frame distinct SoundOffer and Poststep whole owners")
                })?,
            )
        } else {
            None
        };
        Ok(post_down)
    }

    pub(super) fn aim_right(
        &self,
    ) -> Result<Option<frame::movement_pair_contract::PhysicalContract>, Outcome> {
        let Self {
            manifest,
            wram,
            data: content,
            templates,
            ..
        } = *self;
        let aim_right = if manifest.late_movement {
            let source = manifest
                .movement_source_admission
                .as_ref()
                .ok_or_else(|| data_failure(None, "actual aim room and traversal"))?;
            let admission = replay::frame_movement_pair_owner::source_admission(
                wram,
                content,
                templates,
                &source.text()?,
            )
            .map_err(|_| data_failure(None, "actual aim room and traversal"))?;
            let template =
                replay::frame_appearance_owner::pre_wait_template(templates, "Movement.AimRight")
                    .map_err(|_| data_failure(None, "whole aim query envelope"))?;
            Some(frame::movement_pair_contract::PhysicalContract::new(
                frame::movement_caller::PairOwner::AimRight,
                admission,
                template,
            ))
        } else {
            None
        };
        Ok(aim_right)
    }

    pub(super) fn late_movement(&self) -> Result<Option<frame::late_movement::Contract>, Outcome> {
        let Self {
            manifest,
            templates,
            ..
        } = *self;
        Ok(if manifest.late_movement {
            let template = |name| {
                replay::frame_appearance_owner::pre_wait_template(templates, name)
                    .map_err(|_| data_failure(None, "whole late movement envelope"))
            };
            Some(
                frame::late_movement::Contract::new(
                    &template("S3.AimCaller")?,
                    &template("S3.AimAdmission")?,
                )
                .map_err(|_| data_failure(None, "whole late movement envelope"))?,
            )
        } else {
            None
        })
    }

    pub(super) fn pose_fit(
        &self,
    ) -> Result<Option<frame::appearance_caller::PhysicalContract>, Outcome> {
        let Self {
            manifest,
            templates,
            ..
        } = *self;
        Ok(if manifest.late_movement {
            let member = manifest
                .selected_stage_predicates
                .as_ref()
                .ok_or_else(|| data_failure(None, "whole shared PoseFit envelope"))?;
            Some(
                replay::frame_appearance_owner::physical_owners(templates, &member.text()?)
                    .map_err(|_| data_failure(None, "whole shared PoseFit envelope"))?
                    .selected_clip,
            )
        } else {
            None
        })
    }

    pub(super) fn animation(
        &self,
    ) -> Result<Option<frame::appearance_caller::PhysicalContract>, Outcome> {
        let Self {
            manifest,
            templates,
            ..
        } = *self;
        Ok(if manifest.animation_liquid_origin.is_some() {
            let member = manifest
                .selected_stage_predicates
                .as_ref()
                .ok_or_else(|| data_failure(None, "animation complete physical owner"))?;
            Some(
                replay::frame_appearance_owner::physical_owners(templates, &member.text()?)
                    .map_err(|_| data_failure(None, "animation complete physical owner"))?
                    .animation,
            )
        } else {
            None
        })
    }

    pub(super) fn ground_pull(
        &self,
    ) -> Result<Option<frame::movement_ground_contract::PhysicalContract>, Outcome> {
        let Self {
            manifest,
            templates,
            ..
        } = *self;
        Ok(
            match (
                &manifest.selected_stage_predicates,
                &manifest.selected_stage_grouping,
            ) {
                (Some(predicates), Some(grouping)) => Some(
                    replay::frame_movement_ground_owner::contract(
                        templates,
                        &predicates.text()?,
                        &grouping.text()?,
                    )
                    .map_err(|_| {
                        data_failure(None, "frame whole ground pull current local prefix")
                    })?,
                ),
                (_, None) => None,
                (None, Some(_)) => {
                    return Err(data_failure(
                        None,
                        "frame ground pull grouping without selected predicates",
                    ));
                }
            },
        )
    }
}
