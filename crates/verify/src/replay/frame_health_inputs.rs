//! Strict complete original health decode; validation words never initialize runtime State.

use crate::lock::Sha256;
use canonical::model::Model;
use game::production::frame::health::{Activity, Admission, Context, Region, Suits, Time};

/// Bind the no-writer interval to the checked native origin and causal controller schedules.
pub(super) fn adjacent_context(
    case: &super::index::Case,
    pad_source: Option<&Sha256>,
    origin: Option<&Admission>,
) -> Result<Option<Context>, super::result::Outcome> {
    use super::index::Start;
    let mut declarations = case
        .borrowed
        .iter()
        .filter(|borrow| borrow.dependency == "frame.adjacent-health-context");
    let Some(borrow) = declarations.next() else {
        return Ok(None);
    };
    let qualified_origin = matches!(&case.start, Start::NativeSeed { sha256, .. }
        if sha256 == "1e040c393ed88ec8a0b46184879e0681bc110355e928b8160c50170c533abc15");
    let qualified_pads = [
        "0842ac84e469ea39847459083110cff9c36c7de3993c333a216f602f1ae918d0",
        "4c7ad2f8ed191a89273bd15d5c301843238cca3da4cf46da39c7b389d61ffe66",
    ]
    .into_iter()
    .any(|pin| Sha256::try_from(pin).is_ok_and(|expected| Some(&expected) == pad_source));
    if !qualified_origin
        || !qualified_pads
        || declarations.next().is_some()
        || case.first_cut() != Some(1)
        || case.end.cut_inclusive != 2
        || borrow.provider != "qualified-adjacent-health-writer-history"
        || borrow.phase != "first-maintenance-return-through-adjacent-animation"
        || borrow.fields
            != [
                "health-and-context-writer-history",
                "current-actor-protection",
                "drawing-interrupt-preservation",
            ]
        || borrow.claim
            != "external writers and context only; Animation reads carried production Health"
    {
        return Err(super::data_failure(
            None,
            "adjacent Health preservation scope",
        ));
    }
    origin
        .map(|source| Some(source.context))
        .ok_or_else(|| super::data_failure(None, "carried Health origin"))
}

/// Independent origin, mapping or finite context was not qualified.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Wrong projection/origin, missing scalar or unsupported source envelope.
    Source,
}

fn word(bytes: &[u8], projection: &toml::Value, name: &str) -> Result<u16, Fault> {
    let fields = projection
        .get("field")
        .and_then(toml::Value::as_array)
        .ok_or(Fault::Source)?;
    let mut matching = fields
        .iter()
        .filter(|field| field.get("name").and_then(toml::Value::as_str) == Some(name));
    let field = matching.next().ok_or(Fault::Source)?;
    if matching.next().is_some()
        || field
            .get("quantity_width")
            .and_then(toml::Value::as_integer)
            != Some(2)
    {
        return Err(Fault::Source);
    }
    let at = field
        .get("private_wram_offset")
        .and_then(toml::Value::as_integer)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(Fault::Source)?;
    let raw = bytes.get(at..at.saturating_add(2)).ok_or(Fault::Source)?;
    Ok(u16::from_le_bytes(
        raw.try_into().map_err(|_| Fault::Source)?,
    ))
}

/// Decode all five health quantities and every separately borrowed neighbour from original data.
/// The pinned descriptor authenticates mappings; its validation encodings are never read here.
/// # Errors
/// Missing, substituted or unsupported original inputs cannot admit selected health.
pub fn origin(bytes: &[u8], text: &str) -> Result<Admission, Fault> {
    if bytes.len() != 131_072
        || Sha256::of(bytes)
            != Sha256::try_from("bfee6ac13da413a36ad42194a36aa3224af6a84e818092b1e7040ed4782f03db")
                .map_err(|_| Fault::Source)?
        || Sha256::of(text.as_bytes())
            != Sha256::try_from("1da57970f43e717213aab7168fdd0934c7d149f15441bcc6eb911210a4f571fe")
                .map_err(|_| Fault::Source)?
    {
        return Err(Fault::Source);
    }
    let projection: toml::Value = toml::from_str(text).map_err(|_| Fault::Source)?;
    let whole = samus_health::WholeEnergy::new(word(bytes, &projection, "energy_whole")?)
        .ok_or(Fault::Source)?;
    let energy = samus_health::EnergyAmount {
        whole,
        fraction: word(bytes, &projection, "energy_fraction")?,
    };
    let pending_loss = samus_health::EnergyLoss {
        whole: samus_health::WholeEnergy::new(word(bytes, &projection, "pending_whole")?)
            .ok_or(Fault::Source)?,
        fraction: word(bytes, &projection, "pending_fraction")?,
    };
    let warning = if word(bytes, &projection, "warning")? == 0 {
        samus_health::Warning::Off
    } else {
        return Err(Fault::Source);
    };
    if energy.whole.get() < 31
        || energy.fraction != 0
        || pending_loss != samus_health::EnergyLoss::ZERO
        || word(bytes, &projection, "time")? != 0
        || word(bytes, &projection, "suits")? != 0
        || word(bytes, &projection, "power_bomb")? != 0
        || word(bytes, &projection, "elevator")? != 0
        || word(bytes, &projection, "samus_invincibility")? != 0
    {
        return Err(Fault::Source);
    }
    let region = data::room::Area::try_from(
        u8::try_from(word(bytes, &projection, "region")?).map_err(|_| Fault::Source)?,
    )
    .map_err(|_| Fault::Source)?;
    if region != data::room::Area::Ceres {
        return Err(Fault::Source);
    }
    Ok(Admission {
        initial: Model(samus_health::State {
            energy,
            pending_loss,
            warning,
            protection: samus_health::ProtectionFrames::ZERO,
        }),
        context: Context {
            time: Time::Running,
            suits: Suits {
                varia: false,
                gravity: false,
            },
            power_bomb: Activity::Inactive,
            elevator: Activity::Inactive,
            region: Region::Ceres,
            protection: Activity::Inactive,
        },
    })
}
