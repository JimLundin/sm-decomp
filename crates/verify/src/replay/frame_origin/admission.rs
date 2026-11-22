//! Admit current source owners before opening any expected observations.

mod adjacent_tail;
pub(in crate::replay) mod completed_start;
mod native;
mod opening_clock;
mod player;
pub(in crate::replay) mod quiet_profile;
mod returns;
mod seed;

use super::{
    Manifest, buffer_address, consecutive_pre_wait, hud_display, interrupt_inputs,
    interruption_origin, portion, profiles, validate_countdown_scene_scope, validate_health_scope,
};
use crate::lock::Sha256;
use crate::replay::{
    self, Sources, data_failure,
    index::Case,
    movement,
    readers::{ActualReader, AdmittedCase, ExpectedReader, InputReader},
    result::Outcome,
};
use canonical::model::Model;
use game::{
    production::{Production, ProductionContent, ProductionRules, ProductionSeed, frame},
    samus_appearance_bridge::Clips,
};
use seed::{EntrySeed, entry_seed};

struct DrawingWork {
    haze_bands_address: data::dma::BusAddress,
    drawing: Option<frame::interrupts::DrawingProfiles>,
}

struct PostPaletteWork {
    maintenance: Option<frame::maintenance::Contract>,
    s6_camera: Option<frame::s6_camera::Contract>,
    s6_drawing: Option<frame::s6_drawing::Contract>,
}

struct Admission<'a> {
    case: &'a Case,
    manifest: &'a Manifest,
    wram: &'a [u8],
    image: &'a [u8],
    checked_rom: &'a data::rom::Rom,
    data: &'a movement::Content,
    templates: &'a profiles::Templates,
    health_hud_profile: Option<quiet_profile::HealthHud>,
    native_root: &'a std::path::Path,
}

impl Admission<'_> {
    fn bind_drawing_origins(&self, state: &mut frame::State) -> Result<(), Outcome> {
        let Self {
            manifest,
            wram,
            checked_rom,
            ..
        } = *self;
        state.drawing_origins = match (
            &manifest.drawing_current,
            &manifest.drawing_owner_requirements,
        ) {
            (Some(proof), Some(requirements)) => Some(
                replay::frame_drawing_owners::origin(wram, &proof.text()?, &requirements.text()?)
                    .map_err(|_| data_failure(None, "frame current drawing owner origins"))?,
            ),
            (None, None) => None,
            _ => {
                return Err(data_failure(
                    None,
                    "frame complete drawing owner origin contracts",
                ));
            }
        };
        if let Some(hook) = manifest
            .native_return
            .as_ref()
            .and_then(|native| native.drawing_hook.as_ref())
        {
            let fault = || data_failure(None, "actual current no-action graphics hook");
            let origin = state.drawing_origins.as_mut().ok_or_else(fault)?;
            replay::frame_drawing_owners::no_action_graphics_hook(wram, &hook.text()?, checked_rom)
                .map_err(|_| fault())?;
            origin.actors.background_hook = false;
        }
        Ok(())
    }

    fn bind_sender(
        &self,
        state: &mut frame::State,
        hardware: &[u8],
    ) -> Result<Option<frame::sender_contract::PhysicalContent>, Outcome> {
        let Self {
            manifest,
            templates,
            wram,
            ..
        } = *self;
        let sender_physical = match (
            &manifest.sender_current_profile,
            &manifest.sender_bus_origin,
        ) {
            (Some(profile), Some(bus)) => {
                let (origin, physical) = replay::frame_sender_contract::admit(
                    &profile.text()?,
                    &bus.text()?,
                    templates,
                    wram,
                    hardware,
                    &state.senders,
                )
                .map_err(|_| {
                    data_failure(
                        None,
                        "frame retained Sender current owners and full physical contracts",
                    )
                })?;
                state.sender_current = Some(origin.current);
                state.sender_queues = Some(origin.queues);
                Some(physical)
            }
            (None, None) => None,
            _ => {
                return Err(data_failure(
                    None,
                    "frame complete paired Sender source contracts",
                ));
            }
        };
        Ok(sender_physical)
    }

    fn drawing(&self) -> Result<DrawingWork, Outcome> {
        let Self {
            manifest,
            image,
            templates,
            ..
        } = *self;
        let haze_bands_address =
            buffer_address(&manifest.buffer_regions.text()?, "HazeBackdropBands", 16)?;
        let drawing_content = interrupt_inputs::content(image, &manifest.irq_recipe.text()?)
            .map_err(|_| data_failure(None, "frame immutable drawing content"))?;
        let drawing = Some(
            interrupt_inputs::profiles(templates, drawing_content)
                .map_err(|_| data_failure(None, "frame complete drawing profiles"))?,
        );
        Ok(DrawingWork {
            haze_bands_address,
            drawing,
        })
    }

    fn bind_hud_refresh(
        &self,
        state: &mut frame::State,
        hud: &replay::hud::Content,
    ) -> Result<Option<frame::hud_map_caller::Content>, Outcome> {
        let Self {
            manifest,
            wram,
            templates,
            ..
        } = *self;
        let hud_refresh = if let Some(proof) = &manifest.hud_current_controls {
            let (display, current) = hud_display::origin(
                wram,
                proof,
                hud,
                templates,
                &manifest.buffer_regions.text()?,
            )?;
            state.hud_frame_count = Some(current.0.premises.frame.0);
            state.hud_display_current = Some(current);
            Some(display)
        } else {
            None
        };
        Ok(hud_refresh)
    }

    fn post_palette(
        &self,
        appearance: &replay::appearance::Content,
        origins: Option<&frame::drawing_owners::Origins>,
    ) -> Result<PostPaletteWork, Outcome> {
        let Self {
            manifest,
            wram,
            data: content,
            templates,
            ..
        } = *self;
        let (maintenance, s6_camera, s6_drawing) = if let Some(post_palette) =
            &manifest.post_palette
        {
            let maintenance =
                replay::frame_appearance_owner::pre_wait_template(templates, "S5.Maintenance")
                    .map_err(|_| data_failure(None, "Health maintenance envelope"))?;
            let maintenance = frame::maintenance::Contract::new(&maintenance)
                .map_err(|_| data_failure(None, "Health maintenance envelope"))?;
            let camera = replay::frame_camera_inputs::Content::import(
                wram,
                content.rom(),
                &post_palette.current.text()?,
                &post_palette.room.text()?,
            )
            .map_err(|_| data_failure(None, "actual current Camera context"))?;
            let envelope = camera
                .contract(templates, &post_palette.envelope.text()?)
                .map_err(|_| data_failure(None, "actual Camera physical envelope"))?;
            let before = consecutive_pre_wait(
                templates,
                &[
                    "Enemy projectile slots",
                    "PLM slots",
                    "Animated tile slots",
                    "Interactivity admission and projectile interactions",
                    "Power-bomb interaction",
                ],
            )?;
            let camera = frame::s6_camera::Contract::new(&before, camera.inputs(), envelope)
                .map_err(|_| data_failure(None, "actual empty pre-Camera neighbours"))?;
            let drawing = frame::s6_drawing::Contract::new(
                &portion(templates, "Union.DebugDrawingBridge")?,
                &portion(
                    templates,
                    "Drawing, objects, cannon, sprites, tiles, trails, history/events",
                )?,
                appearance.rendering(content.rom())?,
            )
            .map_err(|_| data_failure(None, "actual live Drawing envelope"))?;
            let drawing = if let Some(profile) = &self.health_hud_profile {
                drawing
                    .with_decreased_energy(
                        &profile.drawing,
                        origins
                            .ok_or_else(|| data_failure(None, "current Drawing history origin"))?
                            .previous_health,
                    )
                    .map_err(|_| data_failure(None, "reached decreased-energy Drawing envelope"))?
            } else {
                drawing
            };
            (Some(maintenance), Some(camera), Some(drawing))
        } else {
            (None, None, None)
        };
        Ok(PostPaletteWork {
            maintenance,
            s6_camera,
            s6_drawing,
        })
    }

    fn bind_liquid_history(
        &self,
        state: &mut frame::State,
    ) -> Result<Option<frame::liquid_history::Admission>, Outcome> {
        let Self {
            case,
            manifest,
            wram,
            ..
        } = *self;
        Ok(
            if let (Some(member), Some(scope)) = (
                &manifest.animation_liquid_origin,
                &manifest.animation_liquid_scope,
            ) {
                let admission = replay::frame_liquid_inputs::admission(
                    wram,
                    &member.text()?,
                    &scope.text()?,
                    case,
                )
                .map_err(|_| data_failure(None, "original retained liquid and explicit scope"))?;
                state.liquid_history = Some(
                    frame::liquid_history::History::from_origin(admission.clone())
                        .map_err(|_| data_failure(None, "retained liquid owner"))?,
                );
                Some(admission)
            } else if manifest.animation_liquid_origin.is_none()
                && manifest.animation_liquid_scope.is_none()
            {
                None
            } else {
                return Err(data_failure(None, "paired liquid scope"));
            },
        )
    }

    fn bind_interruption(
        &self,
        state: &mut frame::State,
    ) -> Result<Option<frame::interruption_caller::Contract>, Outcome> {
        let Self {
            manifest,
            wram,
            templates,
            ..
        } = *self;
        Ok(if let Some(member) = &manifest.interruption_origin {
            let fault = || data_failure(None, "original inactive interruption authority");
            let admission = interruption_origin(wram, &member.text()?).ok_or_else(fault)?;
            state
                .liquid_history
                .as_mut()
                .ok_or_else(fault)?
                .admit_interruption(admission.clone())
                .map_err(|_| fault())?;
            let physical =
                replay::frame_appearance_owner::pre_wait_template(templates, "S5.Interruption")
                    .map_err(|_| data_failure(None, "whole inactive interruption envelope"))?;
            Some(
                frame::interruption_caller::Contract::new(admission, &physical)
                    .map_err(|_| data_failure(None, "whole inactive interruption envelope"))?,
            )
        } else {
            None
        })
    }

    fn bind_health(
        &self,
        state: &mut frame::State,
    ) -> Result<Option<frame::health::Admission>, Outcome> {
        let Self {
            manifest,
            case,
            wram,
            ..
        } = *self;
        Ok(
            if let (Some(member), Some(scope)) = (&manifest.health_origin, &manifest.health_scope) {
                validate_health_scope(case, &scope.text()?)?;
                let mut admission = replay::frame_health_inputs::origin(wram, &member.text()?)
                    .map_err(|_| {
                        data_failure(None, "complete health origin and current context")
                    })?;
                if let Some(arm) = replay::frame::health_hud::Arm::from_case(case) {
                    arm.configure(self.native_root, wram, &case.row, &mut admission)?;
                }
                state.health = Some(
                    frame::health::Health::from_origin(admission.clone()).map_err(|_| {
                        data_failure(None, "live health origin without maintenance")
                    })?,
                );
                Some(admission)
            } else if manifest.health_origin.is_none() && manifest.health_scope.is_none() {
                None
            } else {
                return Err(data_failure(
                    None,
                    "paired explicit health origin and scope",
                ));
            },
        )
    }

    fn countdown(&self) -> Result<Option<frame::countdown_caller::Physical>, Outcome> {
        let Self {
            manifest,
            case,
            wram,
            templates,
            ..
        } = *self;
        Ok(if let Some(member) = &manifest.countdown_origin {
            validate_countdown_scene_scope(case)?;
            let expected = Sha256::try_from(
                "1669869c7b23b99a6b56e0886edcf7627f1832066858cf077c0130c2e5a6672b",
            )
            .map_err(|_| data_failure(None, "countdown origin projection identity"))?;
            if member.pin()? != expected {
                return Err(data_failure(None, "countdown origin source identity"));
            }
            Some(
                replay::frame_countdown_owner::physical(wram, &member.text()?, templates).map_err(
                    |_| data_failure(None, "countdown complete origin and whole physical owner"),
                )?,
            )
        } else {
            None
        })
    }
}

fn health_hud_profiles(
    case: &Case,
    sources: &Sources,
) -> Result<Option<quiet_profile::HealthHud>, Outcome> {
    replay::frame::health_hud::Arm::from_case(case)
        .map(|_| {
            let text = replay::frame::health_hud::Arm::profiles(&sources.native)?;
            quiet_profile::health_hud(&text)
                .map_err(|_| data_failure(None, "retained Health and HUD physical profiles"))
        })
        .transpose()
}

fn assemble(
    case: &Case,
    sources: &Sources,
    manifest: &Manifest,
    wram: &[u8],
    content: movement::Content,
    seed: EntrySeed,
    settling_haze: Option<data::timing::Template>,
) -> Result<(frame::State, frame::Content, Clips), Outcome> {
    let mut state = seed.state;
    let admit = Admission {
        case,
        manifest,
        wram,
        image: &seed.image,
        checked_rom: &seed.checked_rom,
        data: &content,
        templates: &seed.templates,
        health_hud_profile: health_hud_profiles(case, sources)?,
        native_root: &sources.native,
    };
    admit.bind_drawing_origins(&mut state)?;
    let sender_physical = admit.bind_sender(&mut state, &seed.hardware)?;
    let required = admit.required_profiles(seed.reference_acceleration)?;
    let geometry = admit.movement_geometry()?;
    let drawing = admit.drawing()?;
    let appearance = replay::appearance::Content::load(&sources.rom)?;
    let inside_physical = admit.bind_inside(&mut state, &appearance)?;
    let hud = replay::hud::Content::load(&sources.rom)?;
    let early_hud_owner = admit.early_hud_profile(seed.early_hud_controls.is_some())?;
    let ground_following_down = admit.ground_following_down()?;
    let contact = admit.bind_contact(&mut state)?;
    let post_down = admit.bind_post_down(&mut state, contact.is_some())?;
    let hud_refresh = admit.bind_hud_refresh(&mut state, &hud)?;
    state.samus_palette = seed.palette_state;
    state.scene_countdown = seed.scene_countdown.map(|(_, current)| Model(current));
    let drawing_stage = admit.post_palette(&appearance, state.drawing_origins.as_ref())?;
    let native = admit.native_return(&appearance)?;
    let tail = admit.bind_tail(&mut state)?;
    let liquid_history = admit.bind_liquid_history(&mut state)?;
    let interruption = admit.bind_interruption(&mut state)?;
    let health = admit.bind_health(&mut state)?;
    Ok((
        state,
        frame::Content {
            ground_pull: admit.ground_pull()?,
            acknowledgment: required.acknowledgment,
            music: required.music,
            haze: required.haze,
            settling_haze,
            steady_haze: None,
            haze_tail: required.haze_tail,
            haze_bands_address: drawing.haze_bands_address,
            drawing: drawing.drawing,
            layer_blend: required.layer_blend,
            layer_defaults: seed.layer_defaults,
            random: seed.random,
            random_requests: seed.random_requests,
            preparation: required.preparation_content,
            player_entry: required.player_entry,
            animation: admit.animation()?,
            countdown: admit.countdown()?,
            appearance: appearance.appearance,
            hud: Box::new(hud.pictures),
            sender_physical,
            early_hud_controls: seed.early_hud_controls,
            early_hud_owner,
            inside_physical,
            player_continuation: admit.player_continuation()?,
            motion_reset: admit.motion_reset(&seed.entry_labels)?,
            run: admit.run()?,
            movement_room: geometry.movement_room,
            ordinary_right: geometry.ordinary_right,
            compound_support: geometry.compound_support,
            ground_following_down,
            contact,
            post_down,
            hud_refresh,
            liquid_history,
            interruption,
            late_movement: admit.late_movement()?,
            aim_right: admit.aim_right()?,
            maintenance: drawing_stage.maintenance,
            samus_palette: seed.samus_palette,
            s6_camera: drawing_stage.s6_camera,
            s6_drawing: drawing_stage.s6_drawing,
            background: native.background,
            return_work: native.return_work,
            sender_cue: native.sender_cue,
            late_hud: native.late_hud,
            room_tail: tail.room_tail,
            play_time: tail.play_time,
            room_quake: tail.room_quake,
            frame_tail: tail.frame_tail,
            pose_fit: admit.pose_fit()?,
            adjacent_animation: None,
            recurring_animation_advance: None,
            health,
            adjacent_health_context: None,
            scene_countdown: seed.scene_countdown.map(|(admission, _)| admission),
            adjacent_scene_countdown: None,
            recurring_scene_countdown: false,
            movement: content.movement,
        },
        appearance.clips,
    ))
}

#[cfg(test)]
pub(super) fn observed_layer_blend(case: &Case, sources: &Sources) -> Result<[u8; 9], Outcome> {
    let manifest = seed::read_manifest(case, sources)?;
    super::layer_blend_receipt::observed(&manifest.layer_blend.text()?)
        .map_err(|_| data_failure(None, "recorded LayerBlend settings"))
}

pub(in crate::replay) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let (root, text) = seed::read_start(case, sources)?;
    if completed_start::recognizes(&text)? {
        return completed_start::admit(case, sources, &root, &text);
    }
    let manifest = seed::parse_manifest(case, &root, &text)?;
    admit_full(case, sources, &manifest)
}

fn admit_full(
    case: &Case,
    sources: &Sources,
    manifest: &Manifest,
) -> Result<AdmittedCase, Outcome> {
    let content = movement::Content::load(&sources.rom)?;
    let wram = manifest.wram.read()?;
    let mut seed = entry_seed(case, sources, manifest, &content, &wram)?;
    let first_cut = case
        .first_cut()
        .ok_or_else(|| data_failure(None, "first native return"))?;
    let mut returns = returns::prepare(case, sources, manifest, first_cut, seed.next_pad)?;
    if let Some(pad) = &returns.pad_origin {
        let controller = &mut seed.state.timing.hardware.controller;
        if controller.words.first().copied() != Some(pad.previous_sample.0) {
            return Err(data_failure(None, "native previous controller sample"));
        }
        // A completed poll remains readable after the host has presented the next pad.
        controller.committed_pad = pad.presented;
    }
    let (mut state, mut content, clips) = assemble(
        case,
        sources,
        manifest,
        &wram,
        content,
        seed,
        returns.settling_haze.take(),
    )?;
    returns.bind_animation(case, sources, &wram, &mut content, clips.clone())?;
    content.player_entry.recurring_pose_input = returns.adjacent_pose_input;
    if let Some([down, contact]) = returns.adjacent_down_contact {
        content
            .ground_following_down
            .as_mut()
            .ok_or_else(|| data_failure(None, "native Down owner"))?
            .set_odd_two_column(down);
        content
            .contact
            .as_mut()
            .ok_or_else(|| data_failure(None, "native FootContact owner"))?
            .set_quiet(contact);
    }
    if let Some(shape) = returns.adjacent_hud_map {
        let display = content
            .hud_refresh
            .as_mut()
            .ok_or_else(|| data_failure(None, "native HUD display owner"))?;
        display.recurring = Some(shape.qualify(&display.source)?);
    }
    if let Some(actors) = returns.adjacent_actor_update {
        let continuation = content
            .player_continuation
            .as_mut()
            .ok_or_else(|| data_failure(None, "native ActorUpdate continuation"))?;
        continuation.recurring_actors = Some(actors);
    }
    bind_adjacent_countdown(returns.adjacent_countdown_scene, &wram, &mut content)?;
    content.adjacent_health_context = replay::frame_health_inputs::adjacent_context(
        case,
        returns.pad_origin.as_ref().map(|pad| &pad.schedule_sha256),
        content.health.as_ref(),
    )?;
    adjacent_tail::bind(case, sources, &wram, &clips, &mut state, &mut content)?;
    if case.opening_clock() {
        opening_clock::bind(sources, &mut state, &mut content)?;
    }
    let frame_assets = ProductionContent::Frame(Box::new(content));
    let rules = ProductionRules::Frame(Box::new(frame::Rules {
        movement: samus_movement::Rules::default(),
        appearance: samus_appearance::Rules::VANILLA,
        hud: game::production::HudRules::new(interface_hud::Rules::reference()),
    }));
    let mut production = Production::admit(
        ProductionSeed::Frame(Box::new(state)),
        frame_assets.clone(),
        rules.clone(),
    )
    .map_err(|fault| {
        data_failure(
            None,
            &format!("frame complete production admission {fault:?}"),
        )
    })?;
    production
        .schedule_frame_pad_changes(&returns.pads)
        .map_err(|fault| replay::production_failure(first_cut, fault))?;
    Ok(AdmittedCase {
        production,
        content: frame_assets,
        rules,
        inputs: InputReader::Frame(replay::frame::Inputs {
            budget: data::timing::MasterTicks(manifest.host_budget),
            observe_health_hud: replay::frame::health_hud::Arm::from_case(case).is_some(),
            opening_preview: case
                .opening_clock()
                .then(|| opening_clock::preview(&sources.native))
                .transpose()?,
        }),
        expected: ExpectedReader::Frame(Box::new(returns.expected)),
        actual: ActualReader::Frame(replay::frame::Actual::default()),
        schedule: returns.schedule,
    })
}

fn bind_adjacent_countdown(
    scene: Option<crate::replay::frame_countdown_scene::AdjacentProjection>,
    wram: &[u8],
    content: &mut frame::Content,
) -> Result<(), Outcome> {
    if let Some(scene) = scene {
        let fault = || data_failure(None, "adjacent scene continuity and whole Timer work");
        let admission = content.scene_countdown.ok_or_else(fault)?;
        if !scene.holds_origin(wram)
            || admission.choice != frame::countdown_scene::Choice::MaintainCountdown
            || content.countdown.is_none()
        {
            return Err(fault());
        }
        content.adjacent_scene_countdown = Some(admission);
    }
    Ok(())
}
