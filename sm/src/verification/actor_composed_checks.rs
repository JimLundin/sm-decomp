//! Passive staging/birth/scene observations and an explicit rejected-spawn check.
//! No receipt, slot or identity seeding; only owned observations are retained.
#![forbid(unsafe_code)]
use super::{
    composed_fixture as fixture,
    content::ActorContentError,
    identity::InstanceHandle,
    media::AppearanceId,
    scene::{ActorDrawRequest, ActorVisualSelection},
};
use std::cell::RefCell;
use {crate::game::session_runtime::Runtime, sm_game::room_entry::world::Identity};
// Keep checkpoint operations beside the private actor lifecycle they exercise.
pub(crate) fn checkpoint_stage_tilesets(h: &mut Runtime<'_>, tilesets: u16) {
    h.session_mut().room.entry.enemy_tilesets_key = tilesets;
    super::staging::stage_tilesets(h);
}
pub(crate) fn checkpoint_spawn(
    h: &mut Runtime<'_>,
    request: super::api::ActorSpawnRequest,
    remove: bool,
) {
    let handle = super::population::spawn(h, request).unwrap().unwrap();
    let key = h
        .session()
        .actor_identity
        .resolve(handle, &h.session().actors)
        .unwrap();
    if remove {
        h.session_mut().actors.controls.selected = key.byte_index();
        super::population::delete_connected(h);
    }
}

#[derive(Default)]
struct Observations {
    active: bool,
    source_births: usize,
    destination_births: usize,
    source_palette_phases: usize,
    destination_palette_phases: usize,
    rejected_spawns: usize,
    source_handles: Vec<InstanceHandle>,
    source_draws: Vec<AppearanceId>,
    destination_draws: Vec<AppearanceId>,
}
thread_local! { static OBS: RefCell<Observations> = RefCell::new(Observations::default()); }
pub(crate) fn begin() {
    OBS.with(|o| {
        *o.borrow_mut() = Observations {
            active: true,
            ..Default::default()
        }
    });
}
pub(crate) fn stop() {
    OBS.with(|o| o.borrow_mut().active = false);
}
fn resident(h: &Runtime<'_>, appearance: AppearanceId) -> Result<(), ActorContentError> {
    h.session()
        .world
        .actor_media()
        .validate_appearance_residency(
            appearance,
            h.session().actor_identity.palette_residency(),
            &h.loaded_image().expect("loaded image"),
        )
}
pub(crate) fn palette_phase(h: &Runtime<'_>) {
    OBS.with(|o| {
        let mut o = o.borrow_mut();
        if !o.active {
            return;
        }
        assert!(
            matches!(
                resident(h, fixture::APPEARANCES[0]),
                Err(ActorContentError::PaletteNotResident(_))
            ),
            "composed.palette.old-row-rejected-after-real-decode"
        );
        assert!(
            matches!(
                resident(h, fixture::APPEARANCES[2]),
                Err(ActorContentError::PaletteNotResident(_))
            ),
            "composed.palette.destination-not-yet-staged"
        );
        match h.session().room.entry.room {
            Identity::Known(crate::room_world_fixture::ROOM_A) => o.source_palette_phases += 1,
            Identity::Known(crate::room_world_fixture::ROOM_B) => {
                assert!(
                    o.source_births >= 2,
                    "composed.palette.source-was-installed"
                );
                o.destination_palette_phases += 1;
            }
            _ => panic!("composed.palette.unexpected-room"),
        }
    });
}
/// Deliberate verification operation at the real transition boundary. This is
/// separate from passive observation; vanilla does not request this spawn here.
pub(crate) fn reject_unstaged_spawn(h: &mut Runtime<'_>) {
    let active = OBS.with(|o| o.borrow().active);
    if !active || h.session().room.entry.room != Identity::Known(crate::room_world_fixture::ROOM_B)
    {
        return;
    }
    let before: Vec<u8> = (0..131072).map(|at| h.byte(at)).collect();
    let identity = h.session().actor_identity.clone();
    let result = super::population::spawn(
        h,
        super::api::ActorSpawnRequest {
            template: fixture::TEMPLATE,
            appearance: Some(fixture::APPEARANCES[0]),
        },
    );
    assert!(
        matches!(
            result,
            Err(super::identity::ActorIdentityError::Content(
                ActorContentError::PaletteNotResident(_)
            ))
        ),
        "composed.transition.actual-spawn-rejected-before-staging"
    );
    assert_eq!(
        h.session().actor_identity,
        identity,
        "composed.transition.rejection-preserves-identity-and-receipts"
    );
    for (at, value) in before.into_iter().enumerate() {
        assert_eq!(
            h.byte(at),
            value,
            "composed.transition.rejection-preserves-projected-ram-{at:x}"
        );
    }
    OBS.with(|o| o.borrow_mut().rejected_spawns += 1);
}

pub(crate) fn stage(h: &Runtime<'_>, event: u16, key: u16) {
    if event != 5 {
        return;
    }
    OBS.with(|o| {
        let mut o = o.borrow_mut();
        if !o.active {
            return;
        }
        let actor = &h.session().actors.slots[usize::from(key / 64)];
        let observation = h.session().actor_identity.observe(
            key / 64,
            actor.definition != 0,
            h.session().world.actors().species_for_key(actor.definition),
        );
        assert_eq!(
            observation.authored_species,
            Some(fixture::SPECIES),
            "composed.birth.species"
        );
        let appearance = observation.appearance.expect("composed.birth.appearance");
        resident(h, appearance).expect("composed.birth.actual-staging-receipt");
        let palette_index = fixture::APPEARANCES
            .iter()
            .position(|id| *id == appearance)
            .unwrap();
        let row = if palette_index == 1 { 10 } else { 9 };
        for lane in 0..32 {
            assert_eq!(
                h.byte(0xc200 + row * 32 + lane),
                0x11 + palette_index as u8,
                "composed.birth.actual-palette-copy-bytes"
            );
        }
        let handle = observation.handle.expect("composed.birth.live-handle");
        if h.session().room.entry.room == Identity::Known(crate::room_world_fixture::ROOM_A) {
            assert!(
                o.source_palette_phases > 0,
                "composed.birth.source-after-palette"
            );
            let expected = match actor.parameters[0] {
                0xe1 => fixture::APPEARANCES[0],
                0x17 | 0xd5 => fixture::APPEARANCES[1],
                _ => panic!("composed.birth.source-entry"),
            };
            assert_eq!(
                appearance, expected,
                "composed.birth.appearance-follows-entry-not-slot"
            );
            assert!(
                !o.source_handles.contains(&handle),
                "composed.birth.unique-handle"
            );
            o.source_handles.push(handle);
            o.source_births += 1;
        } else {
            assert_eq!(
                h.session().room.entry.room,
                Identity::Known(crate::room_world_fixture::ROOM_B)
            );
            assert!(
                o.destination_palette_phases > 0,
                "composed.birth.destination-after-palette-invalidation"
            );
            assert_eq!(appearance, fixture::APPEARANCES[2]);
            assert!(
                matches!(
                    resident(h, fixture::APPEARANCES[0]),
                    Err(ActorContentError::PaletteNotResident(_))
                ),
                "composed.birth.old-resource-rejected-after-row-reuse"
            );
            for old in &o.source_handles {
                assert!(
                    h.session()
                        .actor_identity
                        .resolve(*old, &h.session().actors)
                        .is_err(),
                    "composed.birth.source-handle-stale"
                );
            }
            o.destination_births += 1;
        }
    });
}
pub(super) fn draw(h: &Runtime<'_>, request: &ActorDrawRequest) {
    OBS.with(|o| {
        let mut o = o.borrow_mut();
        if !o.active {
            return;
        }
        // During initial setup the compatible empty map may precede animation.
        let ActorVisualSelection::Prepared(visual) = &request.visual else {
            return;
        };
        assert_eq!(visual.pose, fixture::POSE, "composed.scene.animated-pose");
        resident(h, visual.appearance).expect("composed.scene.resident-material");
        if h.session().room.entry.room == Identity::Known(crate::room_world_fixture::ROOM_A) {
            o.source_draws.push(visual.appearance);
        } else {
            o.destination_draws.push(visual.appearance);
        }
    });
}
pub(crate) fn source_transfer(h: &mut Runtime<'_>) -> bool {
    super::staging::transfer_actor_tiles_and_initialize(h);
    h.session().actors.staging.transfer_source == 0xffff
}
pub(crate) fn source(h: &mut Runtime<'_>, reversed: bool) {
    assert_eq!(
        h.session().actors.controls.population_count,
        2,
        "composed.source.nonempty-install"
    );
    assert_eq!(h.session().actors.controls.first_free, 128);
    for (slot, tag) in (if reversed { [0x17, 0xe1] } else { [0xe1, 0x17] })
        .into_iter()
        .enumerate()
    {
        assert_eq!(
            h.session().actors.slots[slot].parameters[0],
            tag,
            "composed.source.reordered-entry"
        );
    }
    let handle = super::population::spawn(
        h,
        super::api::ActorSpawnRequest {
            template: fixture::TEMPLATE,
            appearance: Some(fixture::APPEARANCES[1]),
        },
    )
    .expect("composed.dynamic.accepted")
    .expect("composed.dynamic.created");
    let key = h
        .session()
        .actor_identity
        .resolve(handle, &h.session().actors)
        .unwrap();
    assert_eq!(
        key.byte_index(),
        128,
        "composed.dynamic.actual-first-free-slot"
    );
    assert_eq!(
        h.session().actors.slots[2].parameters[0],
        0xd5,
        "composed.dynamic.template-entry"
    );
    run_actor_frame(h, 3);
    OBS.with(|o| {
        let o = o.borrow();
        assert_eq!(
            o.source_births, 3,
            "composed.source.population-plus-typed-spawn"
        );
        for appearance in &fixture::APPEARANCES[..2] {
            assert!(
                o.source_draws.contains(appearance),
                "composed.source.two-distinct-appearances-emitted"
            );
        }
    });
}
fn run_actor_frame(h: &mut Runtime<'_>, count: usize) {
    let visits = h.session().actors.controls.selection_visits;
    let frames: Vec<_> = h.session().actors.slots[..count]
        .iter()
        .map(|a| a.frame)
        .collect();
    super::scheduling::select(h);
    assert_eq!(
        h.session().actors.controls.selection_visits,
        visits.wrapping_add(1)
    );
    assert_eq!(
        h.session().actors.schedule.active_write_position,
        count as u16 * 2,
        "composed.select.all-actual-births"
    );
    super::scheduling::advance_selected(h);
    for (i, frame) in frames.into_iter().enumerate() {
        let actor = &h.session().actors.slots[i];
        assert_eq!(
            actor.frame,
            frame.wrapping_add(1),
            "composed.behavior.actual-update"
        );
        assert_eq!(
            actor.spritemap,
            fixture::MAP,
            "composed.animation.actual-script"
        );
        assert_eq!(actor.instruction, 0xe004, "composed.animation.cursor");
    }
    super::drawing::draw_scene(h);
}
pub(crate) fn destination(h: &mut Runtime<'_>) {
    assert_eq!(
        h.session().actors.controls.population_count,
        1,
        "composed.destination.nonempty-install"
    );
    assert_eq!(h.session().actors.slots[0].parameters[0], 0xb3);
    run_actor_frame(h, 1);
    OBS.with(|o| {
        let o = o.borrow();
        assert_eq!(o.destination_births, 1, "composed.destination.actual-birth");
        assert!(
            o.rejected_spawns > 0,
            "composed.destination.actual-admission-rejection-covered"
        );
        assert!(
            o.destination_draws.contains(&fixture::APPEARANCES[2]),
            "composed.destination.appearance-emitted"
        );
    });
}
