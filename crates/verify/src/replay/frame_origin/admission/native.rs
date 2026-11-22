//! Bind current native publication, sprite finalization and retained frame history.

use super::super::{buffer_address, portion, sprite_history_profile};
use super::Admission;
use crate::replay::{self, data_failure, result::Outcome};
use game::production::frame;

pub(super) struct NativeReturn {
    pub(super) background: Option<frame::background_caller::Contract>,
    pub(super) return_work: Option<frame::frame_return::Contract>,
    pub(super) sender_cue: Option<frame::sender_stage::CueBinding>,
    pub(super) late_hud: Option<frame::late_hud::Contract>,
}

pub(super) struct TailWork {
    pub(super) room_tail: Option<frame::room_tail::Contract>,
    pub(super) frame_tail: Option<frame::frame_tail::Contract>,
    pub(super) play_time: Option<frame::play_time::Contract>,
    pub(super) room_quake: Option<frame::room_quake::Contract>,
}

impl Admission<'_> {
    pub(super) fn native_return(
        &self,
        appearance: &replay::appearance::Content,
    ) -> Result<NativeReturn, Outcome> {
        let Self {
            manifest,
            data: content,
            wram,
            templates,
            ..
        } = *self;
        let (background, return_work, sender_cue, late_hud) = if let Some(native) =
            &manifest.native_return
        {
            use replay::frame_suffix_owner as source;

            let rendering = appearance.rendering(content.rom())?;
            native.hud_queue_scope.read()?;
            let queue = source::initial_hud_queue(wram)
                .map_err(|_| data_failure(None, "initial live HUD queue"))?;
            if queue != frame::suffix_caller::HudQueue::Empty {
                return Err(data_failure(None, "initial empty HUD queue"));
            }
            let late_hud = frame::late_hud::Contract {
                enemy_background: portion(templates, "Enemy background request")?
                    .bind(&[])
                    .map_err(|_| data_failure(None, "quiet enemy-background profile"))?,
                status: portion(templates, "Later HUD handling/request")?
                    .bind(&[])
                    .map_err(|_| data_failure(None, "quiet late HUD profile"))?,
                queue: source::hud_transfer_template(content.rom().lorom())
                    .map_err(|_| data_failure(None, "immutable HUD descriptor"))?,
            };
            let pictures = source::nmi_pictures(&rendering, &appearance.clips, content.rom())
                .map_err(|_| data_failure(None, "admitted NMI Running picture bindings"))?;
            let recipes = source::nmi_recipes(&native.wire.text()?, &native.s6.text()?, templates)
                .map_err(|_| data_failure(None, "admitted ordered NMI recipes"))?;
            let idle = source::idle_nmi_poll_parts(&native.idle_poll.text()?, templates)
                .map_err(|_| data_failure(None, "admitted NMI idle-poll profile"))?;
            let prefix_and_poll = recipes
                .prefix_template(&idle)
                .map_err(|_| data_failure(None, "NMI prefix and poll match the whole"))?;
            let (builder_address, finalization) = self.sprite_finalization(&pictures)?;
            (
                Some(frame::background_caller::Contract {
                    inputs: source::background_scroll_inputs(wram)
                        .map_err(|_| data_failure(None, "background scroll origin"))?,
                    whole: source::background_scroll_template(templates)
                        .map_err(|_| data_failure(None, "background scroll profile"))?,
                }),
                Some(frame::frame_return::Contract {
                    initial: source::initial_borrowed_video(wram)
                        .map_err(|_| data_failure(None, "initial native video fields"))?,
                    rendering,
                    pictures,
                    finalization,
                    adjacent_finalization: None,
                    builder_address,
                    preparation: source::preparation_template(templates)
                        .map_err(|_| data_failure(None, "software frame-request profile"))?,
                    nmi: recipes.whole,
                    adjacent_nmi: None,
                    prefix_and_poll,
                    recipes: recipes.publications,
                }),
                Some(
                    replay::frame_sender_inputs::footstep(
                        content.rom(),
                        &native.footstep.text()?,
                        templates,
                    )
                    .map_err(|_| data_failure(None, "actual Sender cue binding"))?,
                ),
                Some(late_hud),
            )
        } else {
            (None, None, None, None)
        };
        Ok(NativeReturn {
            background,
            return_work,
            sender_cue,
            late_hud,
        })
    }

    fn sprite_finalization(
        &self,
        pictures: &[frame::suffix_caller::PictureNmi],
    ) -> Result<(data::dma::BusAddress, data::timing::Profile), Outcome> {
        let Self {
            manifest,
            templates,
            ..
        } = *self;
        let mut builder = templates
            .materializations
            .iter()
            .filter(|value| value.name == "SpriteDrawList");
        let builder = builder
            .next()
            .filter(|_| builder.next().is_none())
            .ok_or_else(|| data_failure(None, "unique sprite readiness gate"))?;
        if builder.initial_seed || builder.length != 544 {
            return Err(data_failure(None, "current sprite readiness gate"));
        }
        let builder_address = data::dma::OAM_BUFFER;
        let builder_wram = buffer_address(
            &manifest.buffer_regions.text()?,
            "SpriteDrawList",
            builder.length,
        )?;
        if builder_wram.offset != builder_address.offset
            || pictures.iter().any(|picture| {
                let object = &picture.immutable;
                object.object_upload_source_offset != builder_address.offset
                    || object.object_upload_source_bank != builder_address.bank
                    || usize::from(object.object_upload_length) != builder.length
            })
        {
            return Err(data_failure(
                None,
                "live OAM builder and native mirrored source",
            ));
        }
        let finalization = portion(templates, "Sprite-list finalization")?
            .bind(&[])
            .map_err(|_| data_failure(None, "sprite finalization profile"))?;
        Ok((builder_address, finalization))
    }

    pub(super) fn bind_tail(&self, state: &mut frame::State) -> Result<TailWork, Outcome> {
        let Self {
            manifest,
            case,
            checked_rom,
            wram,
            templates,
            ..
        } = *self;
        let mut play_time = None;
        let mut room_quake = None;
        let (room_tail, frame_tail) = if let Some(native) = &manifest.native_return
            && let Some(history) = &native.s6_history
        {
            let debug = native
                .final_debug_operand
                .as_ref()
                .map(|member| {
                    replay::frame_history_inputs::debug_mask(&member.text()?, checked_rom)
                })
                .transpose()?;
            let (history, room, tail) =
                replay::frame_history_inputs::origin(wram, &history.text()?, checked_rom, debug)?;
            state.frame_history = Some(history);
            state.room_tail_origin = Some(room);
            let quiet = |name| {
                portion(templates, name)?
                    .bind(&[])
                    .map_err(|_| data_failure(None, "complete quiet room and block profile"))
            };
            state.frame_tail_current = Some(tail.current);
            if case
                .rules
                .iter()
                .any(|rule| rule == replay::index::PLAY_TIME_RULE)
            {
                let time = replay::frame_history_inputs::play_time(tail.time)?;
                state.play_time = frame::play_time::Stage::from_origin(time);
                play_time = Some(
                    frame::play_time::Contract::new(&portion(templates, "Health and game time")?)
                        .map_err(|_| data_failure(None, "qualified ordinary play-time profile"))?,
                );
            }
            if case
                .rules
                .iter()
                .any(|rule| rule == replay::index::ROOM_QUAKE_RULE)
            {
                let [timer, phase] = tail.shake;
                state.room_quake = frame::room_quake::Stage::from_origin(world_quake::State {
                    remaining: world_quake::QuakeDuration(timer),
                    phase: world_quake::QuakePhase(phase),
                });
                room_quake = Some(
                    frame::room_quake::Contract::new(&portion(templates, "Room shake")?)
                        .map_err(|_| data_failure(None, "qualified quiet room quake profile"))?,
                );
            }
            let frame_tail = if let Some(diagnostics_inactive) = tail.inactive_debug {
                Some(frame::frame_tail::Contract {
                    hurt: quiet("Hurt timers, lists and return")?,
                    history: sprite_history_profile(templates)?,
                    diagnostics_inactive,
                })
            } else {
                None
            };
            (
                Some(frame::room_tail::Contract {
                    blocks: quiet("Block preparation and background upload decisions")?,
                    room: quiet("Room action")?,
                }),
                frame_tail,
            )
        } else {
            (None, None)
        };
        Ok(TailWork {
            room_tail,
            frame_tail,
            play_time,
            room_quake,
        })
    }
}
