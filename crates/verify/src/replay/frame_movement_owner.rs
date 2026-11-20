//! Fixed current Run predicates and checked ROM operands at the import edge.

use super::frame_origin::profiles::{ProfileName, Templates};
use game::production::frame::movement_caller::{RunContract, RunRequired};

/// A missing gated current predicate or incomplete whole physical owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The fixed source projection or checked immutable operand is invalid.
    Source,
    /// Whole physical portions do not exactly match the shared profile decoder.
    Profile,
}

#[derive(serde::Deserialize)]
struct SourceProjection {
    format: String,
    native_terminal_values_exported: bool,
    source_callback_at_runtime: bool,
    expected_timing_or_result_selectors: bool,
    run_current_word: Vec<CurrentWord>,
    immutable_operand: Vec<Operand>,
    physical_portion: Vec<Part>,
    operation: Vec<Operation>,
}

#[derive(serde::Deserialize)]
struct CurrentWord {
    name: String,
    quantity_width: usize,
    required_current_encoding: u16,
    role: String,
    runtime_initializer: bool,
    terminal_result: bool,
}

#[derive(serde::Deserialize)]
struct Operand {
    name: String,
    rom_bank: u8,
    rom_offset: u16,
    byte_width: usize,
    required_imported_encoding: u16,
    role: String,
    runtime_initializer: bool,
    terminal_result: bool,
}

#[derive(serde::Deserialize)]
struct Part {
    profile: ProfileName,
    first_exposure: u64,
    end_exposure: u64,
    semantic_owner: String,
    receipt_contract: String,
    completed_producer_receipts: Vec<String>,
}

#[derive(serde::Deserialize)]
struct Operation {
    owner: String,
    profile: ProfileName,
    first_exposure: u64,
    end_exposure_exclusive: u64,
    whole_envelope_required: bool,
    all_portions_once: bool,
    per_local_physical_rule_invented: bool,
}

/// Decode the frozen projection without reading an initial or future movement seed.
/// The caller requires its independent source gate before admitting this contract.
/// All source quantities remain immutable predicates for actual current Run receipts.
/// # Errors
/// Different projection, missing/aliased operands, wrong ROM linkage or whole profile.
pub fn contract(
    native: &super::movement::Content,
    templates: &Templates,
    text: &str,
) -> Result<RunContract, Fault> {
    use crate::lock::Sha256;
    if Sha256::of(text.as_bytes())
        != Sha256::try_from("0b57c1ef79d6497f2a8abe2e9aed13029f59bdd6e16dd657e06e1ee43e62f251")
            .map_err(|_| Fault::Source)?
    {
        return Err(Fault::Source);
    }
    let source: SourceProjection = toml::from_str(text).map_err(|_| Fault::Source)?;
    if source.format != "portable-selected-stage-current-predicates-v1"
        || source.native_terminal_values_exported
        || source.source_callback_at_runtime
        || source.expected_timing_or_result_selectors
        || source.run_current_word.len() != 4
        || source.immutable_operand.len() != 8
    {
        return Err(Fault::Source);
    }
    for field in &source.run_current_word {
        if field.quantity_width != 2
            || field.role != "TypedCurrentEntryValidationOnly"
            || field.runtime_initializer
            || field.terminal_result
        {
            return Err(Fault::Source);
        }
    }
    for operand in &source.immutable_operand {
        let bytes = native
            .rom()
            .lorom()
            .read(
                data::rom::RomAddress::new(operand.rom_bank, operand.rom_offset),
                operand.byte_width,
            )
            .map_err(|_| Fault::Source)?;
        let encoding = u16::from_le_bytes(bytes.try_into().map_err(|_| Fault::Source)?);
        if operand.byte_width != 2
            || encoding != operand.required_imported_encoding
            || operand.role != "CheckedPlayersRomTypedRuleOperandOnly"
            || operand.runtime_initializer
            || operand.terminal_result
        {
            return Err(Fault::Source);
        }
    }
    let current = |name: &str| {
        let mut matching = source
            .run_current_word
            .iter()
            .filter(|field| field.name == name);
        let field = matching.next().ok_or(Fault::Source)?;
        if matching.next().is_some() {
            return Err(Fault::Source);
        }
        Ok(field.required_current_encoding)
    };
    let operand = |name: &str| {
        let mut matching = source
            .immutable_operand
            .iter()
            .filter(|field| field.name == name);
        let field = matching.next().ok_or(Fault::Source)?;
        if matching.next().is_some() {
            return Err(Fault::Source);
        }
        Ok(field.required_imported_encoding)
    };
    let fixed = |whole, fraction| types::fixed(core::num::Wrapping(whole), fraction);
    Ok(RunContract::new(
        RunRequired {
            dash_before: fixed(current("DashBeforeWhole")?, current("DashBeforeFraction")?),
            speed_before: fixed(
                current("SpeedBeforeWhole")?,
                current("SpeedBeforeFraction")?,
            ),
            dash_gain: fixed(operand("DashGainWhole")?, operand("DashGainFraction")?),
            dash_cap_whole: operand("DashCapWhole")?,
            speed_step: fixed(operand("SpeedStepWhole")?, operand("SpeedStepFraction")?),
            speed_top: fixed(operand("SpeedTopWhole")?, operand("SpeedTopFraction")?),
            horizontal_limit_whole: operand("HorizontalLimitWhole")?,
        },
        whole(templates, &source)?,
    ))
}

fn whole(
    templates: &Templates,
    source: &SourceProjection,
) -> Result<data::timing::Template, Fault> {
    let mut operations = source
        .operation
        .iter()
        .filter(|operation| operation.owner == "S3.Run");
    let operation = operations.next().ok_or(Fault::Profile)?;
    if operations.next().is_some()
        || operation.profile != ProfileName::PreWait
        || !operation.whole_envelope_required
        || !operation.all_portions_once
        || operation.per_local_physical_rule_invented
    {
        return Err(Fault::Profile);
    }
    let required = source
        .physical_portion
        .iter()
        .filter(|part| part.semantic_owner == "S3.Run")
        .collect::<Vec<_>>();
    let actual = templates
        .portions
        .iter()
        .filter(|part| part.semantic_owner == "S3.Run")
        .collect::<Vec<_>>();
    if required.is_empty() || actual.len() != required.len() {
        return Err(Fault::Profile);
    }
    let mut end = operation.first_exposure;
    for (actual, required) in actual.iter().zip(required) {
        if actual.profile != operation.profile
            || actual.first_exposure != end
            || actual.end_exposure <= end
            || actual.profile != required.profile
            || actual.first_exposure != required.first_exposure
            || actual.end_exposure != required.end_exposure
            || actual.receipt_contract != required.receipt_contract
            || actual.completed_producer_receipts != required.completed_producer_receipts
        {
            return Err(Fault::Profile);
        }
        end = actual.end_exposure;
    }
    if end != operation.end_exposure_exclusive {
        return Err(Fault::Profile);
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == operation.profile);
    let profile = profiles.next().ok_or(Fault::Profile)?;
    if profiles.next().is_some() {
        return Err(Fault::Profile);
    }
    profile
        .slice(operation.first_exposure, end)
        .map_err(|_| Fault::Profile)?
        .template()
        .map_err(|_| Fault::Profile)
}
