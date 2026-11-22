//! Independently sourced physical alternatives for carried opening owners.

use super::{Outcome, Path, Sources, data_failure, frame, member, quiet_profile};
use frame::post_down_contract::{Demo, OfferCurrent, PowerBomb, Sounds};

pub(super) fn bind(
    sources: &Sources,
    wram: &[u8],
    content: &mut frame::Content,
) -> Result<(), Outcome> {
    let native = &sources.native;
    let fault = || data_failure(None, "qualified saturated Run and second-half HUD work");
    let directory = "opening-clock-cut8-10-work-v1";
    let run = quiet_profile::named(
        &member(
            native,
            directory,
            "saturated-running-work.toml",
            "34f2b9b15a86975c980944833f59bfb002c57e4be89e5cd01433afe3b6c1f6f9",
        )?,
        "SaturatedRunning",
        [819, 5318],
    )
    .map_err(|_| fault())?;
    let map = quiet_profile::named(
        &member(
            native,
            directory,
            "second-half-hud-map.toml",
            "e6f8e159737757357d1fd3c85222fadaaa2f1039af72df1ffeb2ebbcb6a0da51",
        )?,
        "SecondHalfHudMap",
        [1699, 11176],
    )
    .map_err(|_| fault())?;
    content.run.as_mut().ok_or_else(fault)?.set_saturated(run);
    content
        .hud_refresh
        .as_mut()
        .and_then(|hud| hud.recurring.as_mut())
        .ok_or_else(fault)?
        .second_half = Some(map);
    bind_doors(native, content)?;
    bind_camera(native, content)?;
    bind_movement(native, content)?;
    bind_mixed_drawing(sources, wram, content)?;
    // The independent sound-offer capture qualifies these at the reached Footstep,
    // with scene and present power-bomb activity rejoined by production.
    content
        .post_down
        .as_mut()
        .ok_or_else(fault)?
        .recurring_footstep = Some(OfferCurrent {
        sounds: Sounds::Enabled,
        demo: Demo::Inactive,
        power_bomb: PowerBomb::Inactive,
    });
    let loop_work = quiet_profile::named(
        &member(
            native,
            "opening-clock-cut11-15-work-v1",
            "running-clip-loop-animation.toml",
            "4a374b3474009e5926c75f45e5cd88da871408891bfb4005209e80a02f8b7dd3",
        )?,
        "RunningClipLoopAnimation",
        [413, 2686],
    )
    .map_err(|_| fault())?;
    content.recurring_animation_loop = Some(frame::appearance_caller::PhysicalContract::new(
        frame::appearance_caller::Requirement::RunningLoop,
        loop_work,
    ));
    Ok(())
}

fn bind_doors(native: &Path, content: &mut frame::Content) -> Result<(), Outcome> {
    let fault = || data_failure(None, "qualified current door update and eligibility work");
    let load = |directory, file, pin, name, counts| {
        quiet_profile::named(&member(native, directory, file, pin)?, name, counts)
            .map_err(|_| fault())
    };
    let update = content.player_continuation.as_mut().ok_or_else(fault)?;
    update.closing_starts = Some(load(
        "opening-clock-cut8-10-work-v1",
        "closing-first-door-work.toml",
        "b05efe0b7a232325169201351ab4433fc6426973fb9af56d24e1ddca56a5686c",
        "ClosingFirstDoorOtherPoll",
        [1958, 12812],
    )?);
    let branches = "opening-clock-cut9-10-branches-v1";
    update.closing_wait = Some(load(
        branches,
        "closing-first-door-wait-work.toml",
        "31dcbfb08e955d6228cb3bd5128c091a707b600064f9f00a34331b33a720b169",
        "ClosingFirstDoorWaitWork",
        [1978, 12968],
    )?);
    let later = "opening-clock-cut11-15-work-v1";
    update.closing_other_poll = Some(load(
        later,
        "closing-first-door-second-poll.toml",
        "eaad8fbe39ca5690260fce96878a1c645819e1a2fdd02535f733852b7451c26d",
        "ClosingDoorSecondPoll",
        [2108, 13806],
    )?);
    update.closing_advance = Some(load(
        later,
        "closing-door-timer-advance.toml",
        "e369534a83454db27fdb392d700909d9e87a31d77abaa99a19bf6fb0aad13795",
        "ClosingDoorAdvance",
        [2041, 13374],
    )?);
    content.preparation.two_interactive_actors = Some(load(
        branches,
        "two-actor-eligibility.toml",
        "f3ff7d44ade1c512a50ffefbc1b99d328d84b22ac7e6bb23a2834d3f5a454941",
        "TwoActorEligibility",
        [1435, 9250],
    )?);
    Ok(())
}

fn bind_camera(native: &Path, content: &mut frame::Content) -> Result<(), Outcome> {
    let fault = || data_failure(None, "qualified live Camera edge work");
    let load = |directory, file, pin, name, counts| {
        quiet_profile::multiplying(&member(native, directory, file, pin)?, name, counts)
            .map_err(|_| fault())
    };
    let backward = load(
        "opening-clock-cut9-10-branches-v1",
        "camera-crosses-backward-aim.toml",
        "b23a0e3f17282ed52690b780f3c398d7f77d00112fa5a33ce388a832c3321c15",
        "CameraCrossesBackwardAim",
        [784, 5108],
    )?;
    let rightward_past = load(
        "opening-clock-cut10-aim-camera-v1",
        "camera-rightward-room-edge.toml",
        "249e4637e236b587019e162c7a0df38ed91fa1fe53efefb4000032cc22ecd2ba",
        "CameraRightwardRoomEdge",
        [801, 5218],
    )?;
    let rightward_before = load(
        "opening-clock-cut11-15-work-v1",
        "camera-rightward-no-aim-crossing.toml",
        "8bab5ba8db2e6b4aad077eb1f42eaeb0697338d21e220142d75db7367cad6085",
        "CameraRightwardNoAimCrossing",
        [787, 5122],
    )?;
    content
        .s6_camera
        .as_mut()
        .ok_or_else(fault)?
        .admit_edge_profiles([backward, rightward_past, rightward_before])
        .ok_or_else(fault)
}

fn bind_movement(native: &Path, content: &mut frame::Content) -> Result<(), Outcome> {
    let fault = || data_failure(None, "qualified movement over the current two-slot roster");
    let load = |directory, file, pin, name, counts| {
        quiet_profile::multiplying(&member(native, directory, file, pin)?, name, counts)
            .map_err(|_| fault())
    };
    let opening = "opening-clock-cut8-10-work-v1";
    let later = "opening-clock-cut11-15-work-v1";
    let branches = "opening-clock-cut9-10-branches-v1";
    let carry = load(
        opening,
        "two-door-miss-movement.toml",
        "9c7a775033a2d41e02346ef7b9c63dea6898cdae8c6ca76f45e70b55b03f6c05",
        "TwoDoorMissMovement",
        [1058, 6922],
    )?;
    let no_carry = load(
        later,
        "two-door-no-carry-movement.toml",
        "896d32128a55cec13be64a09b17b3d8fdb83f54a8254cbbe098a6b23f3518cdb",
        "TwoDoorNoCarryMovement",
        [1051, 6872],
    )?;
    let aim_right = load(
        "opening-clock-cut10-aim-camera-v1",
        "two-candidate-aim-right.toml",
        "d2aa9d8f64c133abd29dbe00205c4e78693cf9e6ec5927352f6288f0f4c40ef0",
        "TwoCandidateAimRight",
        [1003, 6550],
    )?;
    let even = load(
        branches,
        "two-actor-ground-following-down.toml",
        "3d016985887b8ee415b4160d8b03d67992497fa98ba475969d479f34fbe3d9ad",
        "TwoActorGroundFollowingDown",
        [927, 6060],
    )?;
    let odd = load(
        later,
        "two-actor-odd-ground-following-down.toml",
        "17ab8a22fb9babf6e19885c82ff0dd3d851cb371ea16a04db58d77c104b73cf4",
        "TwoActorOddGroundFollowingDown",
        [931, 6088],
    )?;
    content
        .ordinary_right
        .as_mut()
        .ok_or_else(fault)?
        .set_two_candidate_profiles(no_carry, Some(carry));
    content
        .aim_right
        .as_mut()
        .ok_or_else(fault)?
        .set_two_candidate_profiles(aim_right, None);
    content
        .compound_support
        .as_mut()
        .ok_or_else(fault)?
        .admit_two_interactive();
    content
        .ground_following_down
        .as_mut()
        .ok_or_else(fault)?
        .set_two_candidate_profiles([even, odd]);
    Ok(())
}

fn bind_mixed_drawing(
    sources: &Sources,
    wram: &[u8],
    content: &mut frame::Content,
) -> Result<(), Outcome> {
    use data::enemy::program::{CERES_DOOR_SPRITEMAPS, EnemyPrograms};
    let fault = || {
        data_failure(
            None,
            "imported current actor graphics and selected layer work",
        )
    };
    let directory = "opening-clock-mixed-drawing-v1";
    member(
        &sources.native,
        directory,
        "mixed-actor-samus-drawing-input.toml",
        "764b3cab1d68b8e36fa5002c379c28ed5317243e84d6a9d805e277ad2445fed4",
    )?;
    let splice = quiet_profile::named(
        &member(
            &sources.native,
            directory,
            "single-visible-door-drawing.toml",
            "972ba0f6a00931d65c6dc8c34b11f1c638a6a7c015e9c2d55e614ca00e237f7d",
        )?,
        "SingleVisibleDoorDrawing",
        [947, 6090],
    )
    .map_err(|_| fault())?;
    let rom = data::rom::Rom::new(std::fs::read(&sources.rom).map_err(|_| fault())?)
        .map_err(|_| fault())?;
    let sprites = EnemyPrograms::slice_zero(rom.lorom()).map_err(|_| fault())?;
    let figure = |index: usize| {
        CERES_DOOR_SPRITEMAPS
            .get(index)
            .copied()
            .and_then(|map| sprites.spritemap_id(map))
            .and_then(|id| sprites.sprites(id))
            .cloned()
            .ok_or_else(fault)
    };
    let word = |address: usize| {
        let [low, high] = wram
            .get(address..address.saturating_add(2))
            .ok_or_else(fault)?
        else {
            return Err(fault());
        };
        Ok(u16::from_le_bytes([*low, *high]))
    };
    content
        .s6_drawing
        .as_mut()
        .ok_or_else(fault)?
        .admit_mixed_actor(
            &splice,
            frame::rendering_stage::DoorArt {
                figures: [figure(4)?, figure(3)?],
                tile_base: word(0x0f98)?,
                attributes: word(0x0f96)?,
            },
        )
        .map_err(|_| fault())
}

pub(super) fn bind_contact_reply(
    native: &Path,
    content: &mut frame::Content,
) -> Result<(), Outcome> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ExternalReply {
        format: u16,
        scope: String,
        group: String,
        reply: String,
        trigger: String,
        lifetime: String,
        after_trigger_master_ticks: u64,
    }
    let fault = || {
        data_failure(
            None,
            "independently timed Footstep ingress after the actual later retry",
        )
    };
    let external: ExternalReply = toml::from_str(&member(
        native,
        "opening-clock-cut11-15-work-v1",
        "opening-contact-later-footstep-reply.toml",
        "08540b234adff38cbb4ba7da5eeb744276a0c47ddb2719772dd2ca1225bc545a",
    )?)
    .map_err(|_| fault())?;
    if external.format != 1
        || external.scope != "OpeningGroundCorridorThroughCut15"
        || external.group != "Contact"
        || external.reply != "Footstep"
        || external.trigger != "NextActualFootstepRetryAfterFirstHandshake"
        || external.lifetime != "UntilAnotherExternalReplyPublication"
    {
        return Err(fault());
    }
    content
        .sender_physical
        .as_mut()
        .and_then(|physical| physical.contact_continuation.as_mut())
        .ok_or_else(fault)?
        .admit_later_reply(data::timing::MasterTicks(
            external.after_trigger_master_ticks,
        ))
        .map_err(|_| fault())
}
