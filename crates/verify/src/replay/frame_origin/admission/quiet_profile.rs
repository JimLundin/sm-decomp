//! Admit finite byte-free physical profiles independently of expected observations.

use super::super::profiles::{ProfileFault, RawRun, cycle};
use data::timing::{Sample, Template, TemplateEffect, TemplateRun};
use serde::Deserialize;

#[derive(Deserialize)]
struct File {
    format: u32,
    profile: Vec<Profile>,
}

#[derive(Deserialize)]
struct Profile {
    #[serde(alias = "owner")]
    name: String,
    exposures: u64,
    intrinsic_ticks: u64,
    run: Vec<RawRun>,
}

pub(super) fn decode(
    text: &str,
    name: &str,
    expected: [u64; 2],
) -> Result<data::timing::Template, ProfileFault> {
    let file: File = toml::from_str(text).map_err(|_| ProfileFault::Format)?;
    let [profile] = file.profile.as_slice() else {
        return Err(ProfileFault::Format);
    };
    if file.format != 1 {
        return Err(ProfileFault::Format);
    }
    admit(profile, name, expected, false)
}

pub(super) fn down_contact(text: &str) -> Result<[Template; 2], ProfileFault> {
    let file: File = toml::from_str(text).map_err(|_| ProfileFault::Format)?;
    let [down, contact] = file.profile.as_slice() else {
        return Err(ProfileFault::Format);
    };
    if file.format != 1 {
        return Err(ProfileFault::Format);
    }
    Ok([
        admit(down, "OddTwoColumnDown", [815, 5340], true)?,
        admit(contact, "QuietFootContact", [29, 188], false)?,
    ])
}

pub(in crate::replay) fn current_random(text: &str) -> Result<Template, ProfileFault> {
    let file: File = toml::from_str(text).map_err(|_| ProfileFault::Format)?;
    let [profile] = file.profile.as_slice() else {
        return Err(ProfileFault::Format);
    };
    if file.format != 1 {
        return Err(ProfileFault::Format);
    }
    admit(profile, "Random", [97, 614], true)
}

/// Admit effect-free local prelude exposures at their existing owner boundaries.
pub(super) fn acknowledgment_and_music(text: &str) -> Result<[Template; 2], ProfileFault> {
    #[derive(Deserialize)]
    struct Portions {
        format: u32,
        portion: Vec<Profile>,
    }
    let source: Portions = toml::from_str(text).map_err(|_| ProfileFault::Format)?;
    let [acknowledgment, music] = source.portion.as_slice() else {
        return Err(ProfileFault::Format);
    };
    if source.format != 1 {
        return Err(ProfileFault::Format);
    }
    Ok([
        admit(acknowledgment, "Acknowledgment", [36, 232], false)?,
        admit(music, "Music", [70, 466], false)?,
    ])
}

pub(in crate::replay) fn steady(text: &str) -> Result<[Template; 3], ProfileFault> {
    #[derive(Deserialize)]
    struct Portions {
        format: u32,
        portion: Vec<Profile>,
    }
    let source: Portions = toml::from_str(text).map_err(|_| ProfileFault::Format)?;
    let [_, _, retain, finish, layers, ..] = source.portion.as_slice() else {
        return Err(ProfileFault::Format);
    };
    if source.format != 1 {
        return Err(ProfileFault::Format);
    }
    Ok([
        admit(retain, "HazeRetain", [150, 988], false)?,
        admit(finish, "HazeFinish", [231, 1480], false)?,
        admit(layers, "LayerBlend", [140, 908], false)?,
    ])
}

fn admit(
    profile: &Profile,
    name: &str,
    expected: [u64; 2],
    math: bool,
) -> Result<Template, ProfileFault> {
    if profile.name != name || [profile.exposures, profile.intrinsic_ticks] != expected {
        return Err(ProfileFault::Profile);
    }
    let mut exposures = 0_u64;
    let mut ticks = 0_u64;
    let mut runs = Vec::new();
    for run in &profile.run {
        let effect = match (run.device.as_str(), run.event_phase.as_str()) {
            ("", "") => TemplateEffect::None,
            ("Multiplicand", "publication") if math => {
                TemplateEffect::Publish(data::timing::MULTIPLICAND_PORT)
            }
            ("Multiplier", "publication") if math => {
                TemplateEffect::Publish(data::timing::MULTIPLIER_PORT)
            }
            ("ProductLow", "sample") if math => TemplateEffect::Sample(Sample::ProductLow),
            ("ProductHigh", "sample") if math => TemplateEffect::Sample(Sample::ProductHigh),
            _ => return Err(ProfileFault::Binding),
        };
        let (cycle, kind) = cycle(run)?;
        let count = u64::from(run.count);
        exposures = exposures.checked_add(count).ok_or(ProfileFault::Profile)?;
        let duration = count
            .checked_mul(u64::from(cycle.ticks()))
            .ok_or(ProfileFault::Profile)?;
        ticks = ticks.checked_add(duration).ok_or(ProfileFault::Profile)?;
        runs.push(TemplateRun {
            count: run.count,
            cycle,
            kind,
            poll_before: run.poll_before,
            effect,
        });
    }
    if exposures != profile.exposures || ticks != profile.intrinsic_ticks {
        return Err(ProfileFault::Profile);
    }
    let profile = Template { runs };
    profile.validate().map_err(|_| ProfileFault::Profile)?;
    Ok(profile)
}
