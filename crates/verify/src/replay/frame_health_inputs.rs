//! Strict complete original health decode; validation words never initialize runtime State.

use crate::lock::Sha256;
use canonical::model::Model;
use game::production::frame::health::{Activity, Admission, Context, Region, Suits, Time};

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
