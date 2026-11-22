//! Authenticate retained division effects at the current physical exposure.

use super::{Charge, Cursor, Event, EventKind, Fault, Hardware, Math, Stage};
use data::timing::{DeviceEffect, Exposure, ExposureKind, Profile, Publication, Sample};

pub(in crate::production) fn division_prefix_holds(
    current: &Hardware,
    origin: Math,
    portions: &[(&Profile, Option<&Cursor>)],
    events: &[Event],
) -> bool {
    if origin.remaining != 0 {
        return false;
    }
    let mut device = current.clone();
    device.math = origin;
    let mut emitted = Vec::new();
    for &(profile, cursor) in portions {
        if follow_prefix(&mut device, profile, cursor, &mut emitted).is_err() {
            return false;
        }
    }
    device.math == current.math
        && events
            .iter()
            .map(|event| &event.kind)
            .filter(|effect| is_divider_event(effect))
            .eq(emitted.iter().map(|event| &event.kind))
}

const fn is_divider_event(event: &EventKind) -> bool {
    if let EventKind::Publication(write) = event {
        return matches!(
            write.effect,
            Publication::DividendLow(_) | Publication::DividendHigh(_) | Publication::Divisor(_)
        );
    }
    matches!(
        event,
        EventKind::Sample(
            Sample::QuotientLow | Sample::QuotientHigh | Sample::ProductLow | Sample::ProductHigh,
            _
        )
    )
}

fn follow_prefix(
    device: &mut Hardware,
    profile: &Profile,
    cursor: Option<&Cursor>,
    emitted: &mut Vec<Event>,
) -> Result<(), Fault> {
    let Some(cursor) = cursor else {
        for run in &profile.runs {
            complete(device, &run.exposure, run.count, emitted)?;
        }
        return Ok(());
    };
    let index = usize::try_from(cursor.run).map_err(|_| Fault::InvalidState)?;
    let (past, rest) = profile
        .runs
        .split_at_checked(index)
        .ok_or(Fault::InvalidState)?;
    for run in past {
        complete(device, &run.exposure, run.count, emitted)?;
    }
    let run = rest.first().ok_or(Fault::InvalidState)?;
    let exposure = &run.exposure;
    complete(device, exposure, cursor.occurrence, emitted)?;
    let writing = exposure.kind == ExposureKind::Write;
    let stage = &cursor.stage;
    if writing && !matches!(stage, Stage::Poll | Stage::Before) {
        device.math.edge();
    }
    let emitted_effect = match exposure.effect {
        DeviceEffect::None => false,
        DeviceEffect::Publish(_) => matches!(stage, Stage::After | Stage::Finish),
        DeviceEffect::Sample(_) => matches!(
            stage,
            Stage::Charge(Charge::AfterSample { .. })
                | Stage::Publish
                | Stage::After
                | Stage::Finish
        ),
    };
    if emitted_effect {
        apply(device, &exposure.effect, emitted)?;
    }
    if !writing && matches!(stage, Stage::Finish) {
        device.math.edge();
    }
    Ok(())
}

fn complete(
    device: &mut Hardware,
    exposure: &Exposure,
    count: u32,
    emitted: &mut Vec<Event>,
) -> Result<(), Fault> {
    let writing = exposure.kind == ExposureKind::Write;
    for _ in 0..count {
        if writing {
            device.math.edge();
        }
        apply(device, &exposure.effect, emitted)?;
        if !writing {
            device.math.edge();
        }
    }
    Ok(())
}

fn apply(
    device: &mut Hardware,
    effect: &DeviceEffect,
    emitted: &mut Vec<Event>,
) -> Result<(), Fault> {
    match effect {
        DeviceEffect::None => Ok(()),
        DeviceEffect::Publish(write)
            if matches!(
                write.effect,
                Publication::DividendLow(_)
                    | Publication::DividendHigh(_)
                    | Publication::Divisor(_)
            ) =>
        {
            device.publish(write, emitted)
        }
        DeviceEffect::Sample(
            sample @ (Sample::QuotientLow
            | Sample::QuotientHigh
            | Sample::ProductLow
            | Sample::ProductHigh),
        ) => device.sample_device(*sample, emitted),
        DeviceEffect::Publish(_) | DeviceEffect::Sample(_) => Err(Fault::InvalidState),
    }
}
