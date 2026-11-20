//! Original retained-physics decoding, distinct from current effects and Animation scratch.

use crate::lock::Sha256;
use game::production::frame::liquid_history::{Admission, Meaning};

/// Unauthenticated or unsupported original retained physics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fault;

fn string<'a>(value: &'a toml::Value, key: &str) -> Result<&'a str, Fault> {
    value.get(key).and_then(toml::Value::as_str).ok_or(Fault)
}

fn values<'a>(value: &'a toml::Value, key: &str) -> Result<&'a [toml::Value], Fault> {
    value
        .get(key)
        .and_then(toml::Value::as_array)
        .map(Vec::as_slice)
        .ok_or(Fault)
}

fn additions(case: &super::index::Case, proposal: &toml::Value) -> Result<(), Fault> {
    for addition in values(proposal, "addition")? {
        let dependency = string(addition, "dependency")?;
        let mut matches = case
            .borrowed
            .iter()
            .filter(|value| value.dependency == dependency);
        let declared = matches.next().ok_or(Fault)?;
        if matches.next().is_some()
            || declared.provider != string(addition, "provider")?
            || declared.phase != string(addition, "phase")?
            || declared.claim != string(addition, "claim")?
            || !declared
                .fields
                .iter()
                .map(String::as_str)
                .map(Some)
                .eq(values(addition, "fields")?.iter().map(toml::Value::as_str))
            || values(addition, "excludes")?
                .iter()
                .filter_map(toml::Value::as_str)
                .any(|limit| !case.limits.iter().any(|value| value == limit))
        {
            return Err(Fault);
        }
    }
    Ok(())
}

fn relations(case: &super::index::Case, proposal: &toml::Value) -> Result<(), Fault> {
    for relation in values(proposal, "existing_relation")? {
        let dependency = match string(relation, "dependency")? {
            "frame.environment-run" => "frame.prelude",
            "frame.nmi" => "frame.nmi",
            _ => return Err(Fault),
        };
        let claim = string(relation, "claim")?;
        if !case.borrowed.iter().any(|value| {
            value.dependency == dependency && value.fields.iter().any(|field| field == claim)
        }) {
            return Err(Fault);
        }
    }
    Ok(())
}

/// Admit the original owner only with explicit reviewed timing, borrowing and exclusions.
/// # Errors
/// Missing or substituted case declarations cannot widen the finite source qualification.
pub fn admission(
    bytes: &[u8],
    text: &str,
    scope: &str,
    case: &super::index::Case,
) -> Result<Admission, Fault> {
    if Sha256::of(scope.as_bytes())
        != Sha256::try_from("0e1f0d1b90f1d37e93aaa2655e3325b4bb14bda44355c5200328e8e211e3eb99")
            .map_err(|_| Fault)?
    {
        return Err(Fault);
    }
    let proposal: toml::Value = toml::from_str(scope).map_err(|_| Fault)?;
    additions(case, &proposal)?;
    relations(case, &proposal)?;
    origin(bytes, text)
}

/// Decode original retained Air only from the complete authenticated original image.
/// # Errors
/// Changed mappings, later images and unsupported history cannot establish an owner.
pub fn origin(bytes: &[u8], text: &str) -> Result<Admission, Fault> {
    let original = "bfee6ac13da413a36ad42194a36aa3224af6a84e818092b1e7040ed4782f03db";
    if bytes.len() != 131_072
        || Sha256::of(bytes) != Sha256::try_from(original).map_err(|_| Fault)?
        || Sha256::of(text.as_bytes())
            != Sha256::try_from("909430820c4e1c028cb6a8ef54f3094227a5594b2e0e25e13549c77b6feeeef2")
                .map_err(|_| Fault)?
    {
        return Err(Fault);
    }
    let projection: toml::Value = toml::from_str(text).map_err(|_| Fault)?;
    let at = projection
        .get("private_history_offset")
        .and_then(toml::Value::as_integer)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(Fault)?;
    let raw = bytes.get(at..at.saturating_add(2)).ok_or(Fault)?;
    // The accepted producer decode establishes this original encoding's Air meaning.
    if Sha256::of(raw)
        != Sha256::try_from("96a296d224f285c67bee93c30f8a309157f0daa35dc5b87e410b78630a09cfc7")
            .map_err(|_| Fault)?
    {
        return Err(Fault);
    }
    Ok(Admission {
        origin: Sha256::try_from(original).map_err(|_| Fault)?.0,
        meaning: Meaning::Air,
    })
}
