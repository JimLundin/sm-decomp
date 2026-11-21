//! Authenticate each following return's causal pad schedule and expected members.

use super::super::{Manifest, Member, hardware_seed};
use super::quiet_profile;
use crate::replay::{
    Sources, data_failure, frame,
    frame_countdown_scene::AdjacentProjection,
    index::{Case, Start},
    result::Outcome,
};
use data::timing::MasterTicks;
use game::production::timing::{PadChange, PhysicalPad};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Supplement {
    format: u16,
    pad_schedule: Member,
    settling_haze: Option<Member>,
    adjacent_pose_input: Option<Member>,
    adjacent_actor_update: Option<Member>,
    adjacent_down_contact: Option<Member>,
    adjacent_hud_map: Option<Member>,
    adjacent_countdown_scene: Option<Member>,
    #[serde(rename = "adjacent_hud_observations")]
    _adjacent_hud_observations: Option<Member>,
    returns: Vec<Return>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Return {
    cut: u64,
    callbacks: Member,
    endpoint: Member,
    final_owners: Option<Member>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PadSchedule {
    initial_row: usize,
    previous_sample_row: usize,
    presentation: Vec<Presentation>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Presentation {
    tick: u64,
    row: usize,
}

impl Return {
    fn first(manifest: &Manifest, cut: u64) -> Self {
        Self {
            cut,
            callbacks: manifest.callbacks.clone(),
            endpoint: manifest.endpoint.clone(),
            final_owners: manifest.final_owners.clone(),
        }
    }

    fn resolve(&mut self, root: &Path) {
        self.callbacks.resolve(root);
        self.endpoint.resolve(root);
        if let Some(member) = &mut self.final_owners {
            member.resolve(root);
        }
    }

    fn expected(self) -> Result<frame::Return, Outcome> {
        Ok(frame::Return {
            callbacks_sha256: self.callbacks.pin()?,
            callbacks: self.callbacks.path,
            endpoint_sha256: self.endpoint.pin()?,
            endpoint: self.endpoint.path,
            final_owners: self
                .final_owners
                .map(|member| {
                    let pin = member.pin()?;
                    Ok((member.path, pin))
                })
                .transpose()?,
        })
    }
}

pub(super) struct PadOrigin {
    pub(super) presented: PhysicalPad,
    pub(super) previous_sample: PhysicalPad,
}

pub(super) struct Prepared {
    pub(super) expected: frame::Expected,
    pub(super) schedule: Vec<u64>,
    pub(super) pads: Vec<PadChange>,
    pub(super) pad_origin: Option<PadOrigin>,
    pub(super) settling_haze: Option<data::timing::Template>,
    pub(super) adjacent_pose_input: Option<data::timing::Template>,
    pub(super) adjacent_actor_update: Option<data::timing::Template>,
    pub(super) adjacent_down_contact: Option<[data::timing::Template; 2]>,
    pub(super) adjacent_hud_map: Option<super::super::hud_display::AdjacentShape>,
    pub(super) adjacent_countdown_scene: Option<AdjacentProjection>,
}

pub(super) fn prepare(
    case: &Case,
    sources: &Sources,
    manifest: &Manifest,
    first_cut: u64,
    next_pad: PadChange,
) -> Result<Prepared, Outcome> {
    let first_return = Return::first(manifest, first_cut);
    let mut expected = BTreeMap::from([(first_cut, first_return.expected()?)]);
    let (
        pad_origin,
        pads,
        settling_haze,
        adjacent_pose_input,
        adjacent_actor_update,
        adjacent_down_contact,
        adjacent_hud_map,
        adjacent_countdown_scene,
    ) = if let Some(reference) = &case.end.native_returns {
        let path = sources.native.join(&reference.path);
        let root = path
            .parent()
            .ok_or_else(|| data_failure(None, "native return supplement directory"))?
            .to_path_buf();
        let mut supplement: Supplement = toml::from_str(
            &Member {
                path,
                sha256: reference.sha256.clone(),
            }
            .text()?,
        )
        .map_err(|_| data_failure(None, "native return supplement"))?;
        if supplement.format != 1 {
            return Err(data_failure(None, "consecutive native returns"));
        }
        extend_expected_returns(&mut expected, supplement.returns, case, first_cut, &root)?;
        supplement.pad_schedule.resolve(&root);
        let (pad_origin, pads) = read_pads(&supplement.pad_schedule, &manifest.pads)?;
        if pads.first().is_none_or(|pad| pad.at != next_pad.at) {
            return Err(data_failure(
                None,
                "native physical pad schedule starts at admission",
            ));
        }
        let settling =
            read_quiet_profile(supplement.settling_haze, &root, "SettlingHaze", [159, 1044])?;
        let pose = read_quiet_profile(
            supplement.adjacent_pose_input,
            &root,
            "AdjacentPoseInput",
            [731, 4672],
        )?;
        let actors = read_quiet_profile(
            supplement.adjacent_actor_update,
            &root,
            "AdjacentActorUpdate",
            [1810, 11822],
        )?;
        let down_contact = read_down_contact(supplement.adjacent_down_contact, &root)?;
        let hud_map = read_hud_map(supplement.adjacent_hud_map, &root)?;
        let countdown_scene =
            read_adjacent_countdown_scene(supplement.adjacent_countdown_scene, &root)?;
        (
            Some(pad_origin),
            pads,
            settling,
            pose,
            actors,
            down_contact,
            hud_map,
            countdown_scene,
        )
    } else {
        (None, vec![next_pad], None, None, None, None, None, None)
    };
    validate_adjacent_countdown_scope(case, adjacent_countdown_scene.is_some())?;
    let schedule = expected.keys().copied().collect();
    let random = case
        .expectations
        .native_random
        .as_ref()
        .map(|source| {
            Ok(frame::random::Source {
                path: sources.native.join(&source.path),
                sha256: crate::lock::Sha256::try_from(source.sha256.as_str())
                    .map_err(|_| data_failure(None, "Random observation pin"))?,
            })
        })
        .transpose()?;
    Ok(Prepared {
        expected: frame::Expected {
            returns: expected,
            random,
        },
        schedule,
        pads,
        pad_origin,
        settling_haze,
        adjacent_pose_input,
        adjacent_actor_update,
        adjacent_down_contact,
        adjacent_hud_map,
        adjacent_countdown_scene,
    })
}

fn validate_adjacent_countdown_scope(case: &Case, present: bool) -> Result<(), Outcome> {
    let mut declarations = case
        .borrowed
        .iter()
        .filter(|value| value.dependency == "frame.adjacent-countdown-scene");
    let Some(value) = declarations.next() else {
        return if present {
            Err(data_failure(
                None,
                "explicit adjacent countdown scene borrowed scope",
            ))
        } else {
            Ok(())
        };
    };
    // The reviewed manifest authenticates the exact ordered first Timer profile
    // reused by the adjacent producer.
    let Start::NativeSeed { sha256, .. } = &case.start else {
        return Err(data_failure(None, "adjacent countdown source origin"));
    };
    if sha256 != "1e040c393ed88ec8a0b46184879e0681bc110355e928b8160c50170c533abc15"
        || !present
        || declarations.next().is_some()
        || case.first_cut().and_then(|first| first.checked_add(1)) != Some(case.end.cut_inclusive)
        || value.provider != "independently-qualified-adjacent-scene-scheduling"
        || value.fields
            != [
                "independent-origin-scene-choice",
                "first-native-return-and-current-hud-interrupt-preservation",
                "current-timer-and-ceres-dispatch",
            ]
        || value.phase != "first-native-return-through-next-ceres-revalidation"
        || value.claim != "adjacent scene continuity and selection only"
    {
        return Err(data_failure(
            None,
            "explicit adjacent countdown scene borrowed scope",
        ));
    }
    Ok(())
}

fn read_adjacent_countdown_scene(
    member: Option<Member>,
    root: &Path,
) -> Result<Option<AdjacentProjection>, Outcome> {
    member
        .map(|mut member| {
            member.resolve(root);
            AdjacentProjection::read(&member.text()?)
                .map_err(|_| data_failure(None, "adjacent countdown scene projection"))
        })
        .transpose()
}

fn extend_expected_returns(
    expected: &mut BTreeMap<u64, frame::Return>,
    returns: Vec<Return>,
    case: &Case,
    first_cut: u64,
    root: &Path,
) -> Result<(), Outcome> {
    let schedule_fault = || data_failure(None, "consecutive native returns");
    let mut prior_cut = first_cut;
    for mut next in returns {
        if prior_cut.checked_add(1) != Some(next.cut)
            || next.final_owners.is_some() != case.compares_final_frame_owners()
        {
            return Err(schedule_fault());
        }
        prior_cut = next.cut;
        next.resolve(root);
        expected.insert(next.cut, next.expected()?);
    }
    if prior_cut != case.end.cut_inclusive {
        return Err(schedule_fault());
    }
    Ok(())
}

fn read_hud_map(
    member: Option<Member>,
    root: &Path,
) -> Result<Option<super::super::hud_display::AdjacentShape>, Outcome> {
    member
        .map(|mut member| {
            member.resolve(root);
            toml::from_str(&member.text()?)
                .map_err(|_| data_failure(None, "adjacent HUD input shape"))
        })
        .transpose()
}

fn read_down_contact(
    member: Option<Member>,
    root: &Path,
) -> Result<Option<[data::timing::Template; 2]>, Outcome> {
    member
        .map(|mut member| {
            member.resolve(root);
            quiet_profile::down_contact(&member.text()?)
                .map_err(|_| data_failure(None, "native Down and FootContact profiles"))
        })
        .transpose()
}

fn read_quiet_profile(
    member: Option<Member>,
    root: &Path,
    name: &str,
    expected: [u64; 2],
) -> Result<Option<data::timing::Template>, Outcome> {
    member
        .map(|mut member| {
            member.resolve(root);
            quiet_profile::decode(&member.text()?, name, expected)
                .map_err(|_| data_failure(None, &format!("native {name} physical profile")))
        })
        .transpose()
}

fn read_pads(schedule: &Member, pads: &Member) -> Result<(PadOrigin, Vec<PadChange>), Outcome> {
    let fault = || data_failure(None, "native return physical pad schedule");
    let schedule: PadSchedule = toml::from_str(&schedule.text()?).map_err(|_| fault())?;
    let pads = pads.text()?;
    let rows: Vec<_> = pads.lines().collect();
    let decode = |row| {
        let raw = rows.get(row).copied().ok_or_else(fault)?;
        hardware_seed::pad(raw).map_err(|_| fault())
    };
    let origin = PadOrigin {
        presented: decode(schedule.initial_row)?,
        previous_sample: decode(schedule.previous_sample_row)?,
    };
    let presentations = schedule
        .presentation
        .into_iter()
        .map(|presentation| {
            Ok(PadChange {
                at: MasterTicks(presentation.tick),
                pad: decode(presentation.row)?,
            })
        })
        .collect::<Result<Vec<_>, Outcome>>()?;
    Ok((origin, presentations))
}
