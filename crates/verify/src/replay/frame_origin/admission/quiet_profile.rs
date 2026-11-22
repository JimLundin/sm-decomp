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
    admit(profile, name, expected, Effects::None)
}

pub(super) struct HealthHud {
    pub(super) drawing: Template,
    pub(super) status: Template,
}

pub(super) fn health_hud(text: &str) -> Result<HealthHud, ProfileFault> {
    let file: File = toml::from_str(text).map_err(|_| ProfileFault::Format)?;
    let [_, _, _, _, _, drawing, _, status] = file.profile.as_slice() else {
        return Err(ProfileFault::Format);
    };
    if file.format != 1 {
        return Err(ProfileFault::Format);
    }
    Ok(HealthHud {
        drawing: admit(
            drawing,
            "SinglePendingLossDrawing",
            [5382, 34398],
            Effects::None,
        )?,
        status: admit(
            status,
            "SinglePendingLossLateStatus",
            [430, 2786],
            Effects::Divide,
        )?,
    })
}

pub(super) fn named(text: &str, name: &str, expected: [u64; 2]) -> Result<Template, ProfileFault> {
    let file: File = toml::from_str(text).map_err(|_| ProfileFault::Format)?;
    if file.format != 1 {
        return Err(ProfileFault::Format);
    }
    let mut profiles = file.profile.iter().filter(|profile| profile.name == name);
    let profile = profiles.next().ok_or(ProfileFault::Format)?;
    if profiles.next().is_some() {
        return Err(ProfileFault::Format);
    }
    admit(profile, name, expected, Effects::None)
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
        admit(down, "OddTwoColumnDown", [815, 5340], Effects::Multiply)?,
        admit(contact, "QuietFootContact", [29, 188], Effects::None)?,
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
    admit(profile, "Random", [97, 614], Effects::Multiply)
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
        admit(acknowledgment, "Acknowledgment", [36, 232], Effects::None)?,
        admit(music, "Music", [70, 466], Effects::None)?,
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
        admit(retain, "HazeRetain", [150, 988], Effects::None)?,
        admit(finish, "HazeFinish", [231, 1480], Effects::None)?,
        admit(layers, "LayerBlend", [140, 908], Effects::None)?,
    ])
}

pub(super) fn contact_retry(
    text: &str,
    port: data::timing::PublicationPort,
) -> Result<[Template; 2], ProfileFault> {
    let file: File = toml::from_str(text).map_err(|_| ProfileFault::Format)?;
    let mut profiles = file
        .profile
        .into_iter()
        .filter(|profile| profile.name == "AdjacentContactRetry");
    let mut profile = profiles.next().ok_or(ProfileFault::Format)?;
    if file.format != 1
        || profiles.next().is_some()
        || [profile.exposures, profile.intrinsic_ticks] != [26, 162]
    {
        return Err(ProfileFault::Profile);
    }
    let mut sample = None;
    let mut publication = None;
    for (index, run) in profile.run.iter_mut().enumerate() {
        match (run.device.as_str(), run.event_phase.as_str()) {
            ("", "") => {}
            ("LibraryThreeCue", "sample") if sample.replace(index).is_none() => {
                if run.count != 1
                    || cycle(run)? != (data::timing::Cycle::Six, data::timing::ExposureKind::Read)
                {
                    return Err(ProfileFault::Profile);
                }
            }
            ("LibraryThreeCue", "publication") if publication.replace(index).is_none() => {}
            _ => return Err(ProfileFault::Binding),
        }
        run.device.clear();
        run.event_phase.clear();
    }
    let sample = sample.ok_or(ProfileFault::Binding)?;
    let publication = publication.ok_or(ProfileFault::Binding)?;
    if sample >= publication {
        return Err(ProfileFault::Binding);
    }
    let mut whole = admit(&profile, "AdjacentContactRetry", [26, 162], Effects::None)?;
    whole
        .runs
        .get_mut(publication)
        .ok_or(ProfileFault::Binding)?
        .effect = TemplateEffect::Publish(port);
    let remainder = whole
        .runs
        .split_off(sample.checked_add(1).ok_or(ProfileFault::Profile)?);
    Ok([whole, Template { runs: remainder }])
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Effects {
    None,
    Multiply,
    Divide,
}

fn admit(
    profile: &Profile,
    name: &str,
    expected: [u64; 2],
    effects: Effects,
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
            ("Multiplicand", "publication") if effects == Effects::Multiply => {
                TemplateEffect::Publish(data::timing::MULTIPLICAND_PORT)
            }
            ("Multiplier", "publication") if effects == Effects::Multiply => {
                TemplateEffect::Publish(data::timing::MULTIPLIER_PORT)
            }
            ("DividendLow", "publication") if effects == Effects::Divide => {
                TemplateEffect::Publish(data::timing::DIVIDEND_LOW_PORT)
            }
            ("DividendHigh", "publication") if effects == Effects::Divide => {
                TemplateEffect::Publish(data::timing::DIVIDEND_HIGH_PORT)
            }
            ("Divisor", "publication") if effects == Effects::Divide => {
                TemplateEffect::Publish(data::timing::DIVISOR_PORT)
            }
            ("QuotientLow", "sample") if effects == Effects::Divide => {
                TemplateEffect::Sample(Sample::QuotientLow)
            }
            ("QuotientHigh", "sample") if effects == Effects::Divide => {
                TemplateEffect::Sample(Sample::QuotientHigh)
            }
            ("ProductLow", "sample") if effects != Effects::None => {
                TemplateEffect::Sample(Sample::ProductLow)
            }
            ("ProductHigh", "sample") if effects != Effects::None => {
                TemplateEffect::Sample(Sample::ProductHigh)
            }
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
