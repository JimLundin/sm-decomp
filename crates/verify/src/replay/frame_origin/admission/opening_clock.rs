//! Causal work and upstream inputs for the recurring opening Frame.

use super::{adjacent_tail, quiet_profile};
use crate::lock::Sha256;
use crate::replay::frame as replay;
use crate::replay::{Sources, data_failure, frame_origin::Member, result::Outcome};
use data::timing::{MasterTicks, Template};
use game::production::frame;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct Room {
    format: u16,
    owner: String,
    initial_scene: String,
    initial_departure: String,
}

#[derive(Deserialize)]
struct Contact {
    format: u16,
    group: String,
    cue: String,
    trigger: String,
    lifetime: String,
    after_trigger_master_ticks: u64,
}

#[derive(Deserialize)]
struct Movie {
    format: u16,
    completed_opportunities: usize,
    #[serde(rename = "movie")]
    pads: PadWords,
}

#[derive(Deserialize)]
struct PadWords {
    pad_words: Vec<u16>,
}

fn member(root: &Path, directory: &str, file: &str, sha256: &str) -> Result<String, Outcome> {
    Member {
        path: root.join(directory).join(file),
        sha256: sha256.to_owned(),
    }
    .text()
}

pub(super) fn expected(native: &Path) -> Result<replay::Return, Outcome> {
    let directory = native.join("opening-clock-three-return-expected-v1");
    let pin = |sha| {
        Sha256::try_from(sha).map_err(|_| data_failure(None, "opening return observation pin"))
    };
    Ok(replay::Return {
        callbacks: directory.join("callbacks.bin"),
        callbacks_sha256: pin("772ab607b919aeff5cc37312bbf0a3d12406c02a21eb95882b63ab566b3fa71b")?,
        endpoint: directory.join("endpoint.toml"),
        endpoint_sha256: pin("6fa2dde150a9905938b9a15986de30d063664fb6451b6f67e38a1046cf459e16")?,
        final_owners: Some((
            directory.join("final-owners.toml"),
            pin("e70cc02dc55e2065128e52b730cdd03125be7b09817cf45e62aa78be9c30d142")?,
        )),
    })
}

pub(super) fn preview(native: &Path) -> Result<replay::MoviePreview, Outcome> {
    let fault = || data_failure(None, "locked opening movie pad preview");
    let source: Movie = toml::from_str(&member(
        native,
        "opening-clock-movie-input-v1",
        "opening-movie-input.toml",
        "9aacef4d9c236e03d2e6388787678db48af8ddf9f97fc0cc77f4f0fd8525637a",
    )?)
    .map_err(|_| fault())?;
    let words = source.pads.pad_words;
    if source.format != 1
        || source.completed_opportunities.checked_add(1) != Some(words.len())
        || source.completed_opportunities != 184
    {
        return Err(fault());
    }
    let [sampled, preview] = words.get(2..4).ok_or_else(fault)? else {
        return Err(fault());
    };
    Ok(replay::MoviePreview {
        sampled: game::production::timing::PhysicalPad(*sampled),
        next: game::production::timing::PhysicalPad(*preview),
    })
}

pub(super) fn bind(
    sources: &Sources,
    state: &mut frame::State,
    content: &mut frame::Content,
) -> Result<(), Outcome> {
    let fault = || data_failure(None, "qualified recurring opening work and neighbours");
    let native = &sources.native;
    bind_room(native, state, content)?;
    content.player_entry.recurring_aim_down = Some(
        quiet_profile::decode(
            &member(
                native,
                "opening-clock-reaim-input-v1",
                "recurring-aim-down.toml",
                "2755161e107cd817e55569b66a809ea5e095c6745ca64f805ef4976cd6ccc471",
            )?,
            "RecurringAimDown",
            [615, 3984],
        )
        .map_err(|_| fault())?,
    );
    let players = content.player_continuation.as_mut().ok_or_else(fault)?;
    players.recurring_waiting_actors = Some(players.actors.clone());
    let right = quiet_profile::recurring_ordinary_right(&member(
        native,
        "opening-clock-ordinary-right-input-v1",
        "recurring-ordinary-right.toml",
        "04b57cc511c075733f595cd7db4781f1dfa0b93ef8d5bbba8ab4d7a2acd34c60",
    )?)
    .map_err(|_| fault())?;
    content
        .ordinary_right
        .as_mut()
        .ok_or_else(fault)?
        .set_fractional_carry(right);
    content
        .inside_physical
        .as_mut()
        .ok_or_else(fault)?
        .admit_recurring_air_columns(content.ordinary_right.as_ref().ok_or_else(fault)?)
        .map_err(|_| fault())?;
    let contact = quiet_profile::decode(
        &member(
            native,
            "opening-clock-foot-contact-input-v1",
            "last-unmarked-foot-contact.toml",
            "5b3b94bde15f87e437d7401a55a31dc358e611ac215571b3c9f47d8c9f3634af",
        )?,
        "LastUnmarkedFootContact",
        [40, 254],
    )
    .map_err(|_| fault())?;
    content
        .contact
        .as_mut()
        .ok_or_else(fault)?
        .set_last_unmarked(contact);
    content
        .hud_refresh
        .as_mut()
        .and_then(|hud| hud.recurring.as_mut())
        .ok_or_else(fault)?
        .retain_first_half = true;
    content.recurring_scene_countdown = true;
    content.recurring_animation_advance = Some(
        content
            .animation
            .as_ref()
            .and_then(frame::appearance_caller::PhysicalContract::for_running_advance)
            .ok_or_else(fault)?,
    );
    adjacent_tail::bind_camera(
        &member(
            native,
            "opening-clock-camera-guard-v1",
            "cut-2-camera-current.private.toml",
            "780fadb6997b7afaad0610e2f619fcbc0b59bfc056cce2c677b22bcc0947166a",
        )?,
        content,
        true,
    )?;
    bind_drawing(native, content)?;
    bind_contact(native, content)
}

fn bind_room(
    native: &Path,
    state: &mut frame::State,
    content: &mut frame::Content,
) -> Result<(), Outcome> {
    let fault = || data_failure(None, "qualified recurring opening room");
    let room: Room = toml::from_str(&member(
        native,
        "opening-clock-cut2-room-v1",
        "cut-2-room-guard.private.toml",
        "2ec2f467c5052a3ed7d6aaabe0cf817a323076b6e1adbfb955d950ae1bca8a9c",
    )?)
    .map_err(|_| fault())?;
    if room.format != 1
        || room.owner != "Room"
        || room.initial_scene != "MainGameplay"
        || room.initial_departure != "Inactive"
        || state.scene != Some(frame::borrowed_gameplay::SceneController::MainGameplay)
    {
        return Err(fault());
    }
    state
        .room_tail_origin
        .as_mut()
        .ok_or_else(fault)?
        .departure_inactive = Some(frame::room_tail::InactiveDeparture);
    let [head, finish, _] = quiet_profile::steady(&member(
        native,
        "shot-steady-haze-v1",
        "steady-prelude-templates.toml",
        "b3bb0bd0f0d51a6fe185fe16a76360f8b527333b3c19d2853fa18fad5903d243",
    )?)
    .map_err(|_| fault())?;
    content.steady_haze = Some(frame::haze::SteadyProfiles { head, finish });
    Ok(())
}

fn bind_drawing(native: &Path, content: &mut frame::Content) -> Result<(), Outcome> {
    use frame::s6_drawing::{DrawingPortions, PieceProfiles, RecurringProfiles};
    let fault = || data_failure(None, "recurring Drawing and OAM work");
    let physical = member(
        native,
        "opening-clock-drawing-shapes-v1",
        "drawing-and-oam-shapes.toml",
        "8a336a9e1972de9ff9918ae2cc3cdf203d5b6e8826e51b96cf80d787c1474e3f",
    )?;
    let profile = |name, bill| -> Result<Template, Outcome> {
        quiet_profile::named(&physical, name, bill).map_err(|_| fault())
    };
    let forward = DrawingPortions::new(
        &profile("DrawingForwardBeforeTop", [1931, 12316])?,
        &profile("DrawingForwardBetweenTopAndBottom", [151, 990])?,
        &profile("DrawingForwardAfterBottom", [2096, 13452])?,
    )
    .map_err(|_| fault())?;
    let down_forward = DrawingPortions::new(
        &profile("DrawingDownForwardBeforeTop", [1966, 12542])?,
        &profile("DrawingDownForwardBetweenTopAndBottom", [151, 990])?,
        &profile("DrawingDownForwardAfterBottom", [2081, 13354])?,
    )
    .map_err(|_| fault())?;
    let pieces = PieceProfiles::new(
        &profile("DrawingLowXSmallContinues", [90, 568])?,
        &profile("DrawingLowXSmallLast", [89, 562])?,
        &profile("DrawingLowXLargeContinues", [117, 750])?,
        &profile("DrawingLowXLargeLast", [116, 744])?,
    )
    .map_err(|_| fault())?;
    content
        .s6_drawing
        .as_mut()
        .ok_or_else(fault)?
        .admit_recurring(RecurringProfiles {
            forward,
            down_forward,
            pieces,
        });
    let finish =
        adjacent_tail::finalization_before_history(profile("OamAfterUnused", [18, 120])?, content)?;
    let finalize = frame::frame_return::Finalization::new(
        &profile("OamBeforeUnused", [56, 364])?,
        &profile("OamHideOneUnused", [4, 26])?,
        &finish,
    )
    .map_err(|_| fault())?;
    let returns = content.return_work.as_mut().ok_or_else(fault)?;
    let adjacent = returns.adjacent_finalization.as_ref().ok_or_else(fault)?;
    let composed = finalize.from_drawn(9).map_err(|_| fault())?;
    let all = |profile: &data::timing::Profile| {
        profile
            .runs
            .iter()
            .flat_map(|run| (0..run.count).map(move |_| run.exposure.clone()))
            .collect::<Vec<_>>()
    };
    if all(&composed) != all(adjacent) {
        return Err(fault());
    }
    returns.recurring_finalization = Some(finalize);
    Ok(())
}

fn bind_contact(native: &Path, content: &mut frame::Content) -> Result<(), Outcome> {
    let fault = || {
        data_failure(
            None,
            "independent Contact response and reached matched work",
        )
    };
    let external: Contact = toml::from_str(&member(
        native,
        "opening-clock-contact-match-v1",
        "opening-contact-external-reply.toml",
        "31e7af2c2da36e2e2809c355110088f4ea46a30011f21f09bddf812fe50c4b77",
    )?)
    .map_err(|_| fault())?;
    if external.format != 1
        || external.group != "Contact"
        || external.cue != "Footstep"
        || external.trigger != "PrecedingActualFootstepRetryPublication"
        || external.lifetime != "UntilAnotherExternalReplyPublication"
    {
        return Err(fault());
    }
    let [prefix, remainder] = quiet_profile::contact_match(&member(
        native,
        "opening-clock-contact-match-v1",
        "contact-await-cue-matched.toml",
        "c0271edf98a71eec0506d681370969ed0e0e575e38d8739e114d5e0abeb456ee",
    )?)
    .map_err(|_| fault())?;
    content
        .sender_physical
        .as_mut()
        .and_then(|sender| sender.contact_retry.as_mut())
        .ok_or_else(fault)?
        .admit_match(
            &prefix,
            &remainder,
            MasterTicks(external.after_trigger_master_ticks),
        )
        .map_err(|_| fault())
}
