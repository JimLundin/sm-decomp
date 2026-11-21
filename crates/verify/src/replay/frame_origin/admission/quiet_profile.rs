//! Admit finite byte-free physical profiles independently of expected observations.

use super::super::profiles::{ProfileFault, RawRun, cycle};
use serde::Deserialize;

#[derive(Deserialize)]
struct File {
    format: u32,
    profile: Vec<Profile>,
}

#[derive(Deserialize)]
struct Profile {
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
    use data::timing::{Template, TemplateEffect, TemplateRun};

    let file: File = toml::from_str(text).map_err(|_| ProfileFault::Format)?;
    let [profile] = file.profile.as_slice() else {
        return Err(ProfileFault::Format);
    };
    if file.format != 1
        || profile.name != name
        || [profile.exposures, profile.intrinsic_ticks] != expected
    {
        return Err(ProfileFault::Profile);
    }
    let mut exposures = 0_u64;
    let mut ticks = 0_u64;
    let mut runs = Vec::new();
    for run in &profile.run {
        if !run.device.is_empty() || !run.event_phase.is_empty() {
            return Err(ProfileFault::Binding);
        }
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
            effect: TemplateEffect::None,
        });
    }
    if exposures != profile.exposures || ticks != profile.intrinsic_ticks {
        return Err(ProfileFault::Profile);
    }
    let profile = Template { runs };
    profile.validate().map_err(|_| ProfileFault::Profile)?;
    Ok(profile)
}
