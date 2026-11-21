//! Checked immutable Random operands and full physical owner at the import edge.

use super::profiles::{ProfileName, Templates};
use crate::lock::Sha256;
use data::rom::{Rom, RomAddress};
use game::production::frame::random::{ImportedRandomMultiplier, RandomSeed, SeedRequests};
use serde::Deserialize;

/// An incomplete current request or checked importer binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    Format,
    Contract,
    Seed,
    Content,
    Profile,
}

#[derive(Deserialize)]
struct SeedWork {
    format: String,
    source_coordinates_role: String,
    semantic_gameplay_generator: bool,
    expected_callbacks_used_as_inputs: bool,
    qualified_current_seed_encoding: u16,
    imported_random_multiplier_encoding: u8,
    #[serde(flatten)]
    seed: SeedDependencies,
    full_owner: String,
    source_first_exposure: u64,
    source_end_exposure_exclusive: u64,
    #[serde(flatten)]
    profile: ProfileRequirements,
    physical_owner_completion_before: String,
    request: Vec<ProductRead>,
}

#[derive(serde::Deserialize)]
struct ProfileRequirements {
    profile_requires_both_current_seed_requests: bool,
    full_profile_includes_request_publications_samples_and_completion_tail: bool,
}

#[derive(serde::Deserialize)]
struct SeedDependencies {
    requires_actual_current_seed_equality: bool,
    first_product_retained_for_completion: bool,
    first_product_used_as_second_request_operand: bool,
}
#[derive(Deserialize)]
struct ProductRead {
    producer: String,
    semantic_operand_left: String,
    semantic_operand_right: String,
    seed_part: String,
    runtime_seed_authority: String,
    runtime_multiplier_authority: String,
    left_device: String,
    right_device: String,
    operand_width_bits: u8,
    sample_authority: String,
    request_uses_prior_product: bool,
    expected_product_is_request_input: bool,
    source_sample_byte_count: u8,
    request_publication_timing_authority: String,
}
#[derive(Deserialize)]
struct Binding {
    format: String,
    runtime_recipe_role: String,
    source_indices_role: String,
    request_contract_sha256: String,
    #[serde(flatten)]
    operands: OperandChecks,
    game_receives_typed_multiplier_only: bool,
    source_first_exposure: u64,
    source_end_exposure_exclusive: u64,
    physical_owner_completion_before: String,
    operand: Vec<MultiplierByte>,
}

#[derive(serde::Deserialize)]
struct OperandChecks {
    checked_import_requires_both_operand_bytes: bool,
    checked_import_requires_declared_encoding_equality: bool,
    checked_import_requires_paired_operand_equality: bool,
}
#[derive(Deserialize)]
struct MultiplierByte {
    producer: String,
    semantic_quantity: String,
    rom_bank: u8,
    rom_offset: u16,
    byte_width: usize,
    declared_encoding: u8,
    importer_recipe: String,
    request_operand: String,
    game_content_type: String,
    source_opcode_address_exposed_to_game: bool,
}

/// Bind two current seed requests to checked ROM content and a byte-free whole owner.
/// Source coordinates only construct the immutable physical template at this edge.
/// # Errors
/// Rejects stale current seed, wrong operand roles/content and incomplete physical coverage.
pub fn decode(
    current: RandomSeed,
    rom: &Rom,
    request_text: &str,
    binding_text: &str,
    templates: &Templates,
) -> Result<(SeedRequests, data::timing::Template), Fault> {
    let request: SeedWork = toml::from_str(request_text).map_err(|_| Fault::Format)?;
    let binding: Binding = toml::from_str(binding_text).map_err(|_| Fault::Format)?;
    validate_request(&request, current)?;
    validate_binding(&binding, &request, request_text)?;
    let multiplier = import_multiplier(rom, &binding, request.imported_random_multiplier_encoding)?;
    let requests = SeedRequests { multiplier };
    let preparation = templates
        .portions
        .iter()
        .find(|portion| portion.receipt_contract == "Random")
        .ok_or(Fault::Profile)?;
    let following = templates
        .portions
        .iter()
        .find(|portion| portion.receipt_contract == "BeginSpriteList")
        .ok_or(Fault::Profile)?;
    if preparation.profile != ProfileName::PreWait
        || following.profile != preparation.profile
        || preparation.first_exposure != request.source_first_exposure
        || following.first_exposure != request.source_end_exposure_exclusive
    {
        return Err(Fault::Profile);
    }
    let profile = templates
        .profiles
        .iter()
        .find(|profile| profile.name == preparation.profile)
        .ok_or(Fault::Profile)?
        .slice(preparation.first_exposure, following.first_exposure)
        .map_err(|_| Fault::Profile)?;
    let template = profile.template().map_err(|_| Fault::Profile)?;
    let resolved = requests
        .bind(current, &template)
        .map_err(|_| Fault::Profile)?;
    let writes = resolved
        .runs
        .iter()
        .filter_map(|run| match &run.exposure.effect {
            data::timing::DeviceEffect::Publish(write) => Some(write.clone()),
            data::timing::DeviceEffect::None | data::timing::DeviceEffect::Sample(_) => None,
        })
        .collect::<Vec<_>>();
    let bound = profile.bind(&writes).map_err(|_| Fault::Profile)?;
    let expected = [
        (data::timing::Sample::ProductLow, "RandomLowSeed"),
        (data::timing::Sample::ProductHigh, "RandomLowSeed"),
        (data::timing::Sample::ProductLow, "RandomHighSeed"),
    ];
    if bound.name != ProfileName::PreWait
        || bound.samples.len() != expected.len()
        || !bound
            .samples
            .iter()
            .zip(expected)
            .all(|(sample, (device, name))| {
                sample.device == device
                    && sample
                        .arithmetic
                        .as_ref()
                        .is_some_and(|task| task.name == name)
            })
        || bound.profile != resolved
    {
        return Err(Fault::Profile);
    }
    Ok((requests, template))
}

fn validate_request(request: &SeedWork, current: RandomSeed) -> Result<(), Fault> {
    if request.format != "random-hardware-request-receipts-v1"
        || request.source_coordinates_role != "ProvenanceOnly"
        || request.semantic_gameplay_generator
        || request.expected_callbacks_used_as_inputs
        || !request.seed.requires_actual_current_seed_equality
        || !request.seed.first_product_retained_for_completion
        || request.seed.first_product_used_as_second_request_operand
        || request.full_owner != "RandomWithActualSeedRequestsAndLiveSamples"
        || !request.profile.profile_requires_both_current_seed_requests
        || !request
            .profile
            .full_profile_includes_request_publications_samples_and_completion_tail
        || request.physical_owner_completion_before != "BeginSpriteList"
        || request.request.len() != 2
    {
        return Err(Fault::Contract);
    }
    if request.qualified_current_seed_encoding != current.0 {
        return Err(Fault::Seed);
    }
    for (producer, part, bytes) in [
        ("RandomLowSeed", "LowByte", 2),
        ("RandomHighSeed", "HighByte", 1),
    ] {
        let mut selected = request
            .request
            .iter()
            .filter(|request| request.producer == producer);
        let request = selected
            .next()
            .filter(|_| selected.next().is_none())
            .ok_or(Fault::Contract)?;
        if request.semantic_operand_left != "SeedPart"
            || request.semantic_operand_right != "RandomMultiplier"
            || request.seed_part != part
            || request.runtime_seed_authority != "ActualCurrentOriginalRandomSeed"
            || request.runtime_multiplier_authority != "CheckedImportedRandomMultiplier"
            || request.left_device != "Multiplicand"
            || request.right_device != "Multiplier"
            || request.operand_width_bits != 8
            || request.sample_authority != "ActualLiveHardwareCompletedSample"
            || request.request_uses_prior_product
            || request.expected_product_is_request_input
            || request.source_sample_byte_count != bytes
            || request.request_publication_timing_authority
                != "ActualSelectedQualifiedHardwareTemplate"
        {
            return Err(Fault::Contract);
        }
    }
    Ok(())
}

fn validate_binding(binding: &Binding, request: &SeedWork, text: &str) -> Result<(), Fault> {
    if binding.format != "random-checked-rom-importer-binding-v1"
        || binding.runtime_recipe_role != "ImporterDataOnly"
        || binding.source_indices_role != "ProvenanceOnly"
        || Sha256::try_from(binding.request_contract_sha256.as_str())
            .map_err(|_| Fault::Contract)?
            != Sha256::of(text.as_bytes())
        || !binding.operands.checked_import_requires_both_operand_bytes
        || !binding
            .operands
            .checked_import_requires_declared_encoding_equality
        || !binding
            .operands
            .checked_import_requires_paired_operand_equality
        || !binding.game_receives_typed_multiplier_only
        || binding.source_first_exposure != request.source_first_exposure
        || binding.source_end_exposure_exclusive != request.source_end_exposure_exclusive
        || binding.physical_owner_completion_before != "BeginSpriteList"
        || binding.operand.len() != 2
    {
        return Err(Fault::Contract);
    }
    Ok(())
}

fn import_multiplier(
    rom: &Rom,
    binding: &Binding,
    declared: u8,
) -> Result<ImportedRandomMultiplier, Fault> {
    let mut paired = None;
    for producer in ["RandomLowSeed", "RandomHighSeed"] {
        let mut selected = binding
            .operand
            .iter()
            .filter(|operand| operand.producer == producer);
        let operand = selected
            .next()
            .filter(|_| selected.next().is_none())
            .ok_or(Fault::Content)?;
        if operand.semantic_quantity != "RandomMultiplier"
            || operand.byte_width != 1
            || operand.importer_recipe != "ReadUnsignedByteAtNormalizedRomAddress"
            || operand.request_operand != "Multiplier"
            || operand.game_content_type != "ImportedRandomMultiplier"
            || operand.source_opcode_address_exposed_to_game
            || operand.declared_encoding != declared
        {
            return Err(Fault::Content);
        }
        let bytes = rom
            .lorom()
            .read(
                RomAddress {
                    bank: operand.rom_bank,
                    offset: operand.rom_offset,
                },
                operand.byte_width,
            )
            .map_err(|_| Fault::Content)?;
        let value = *bytes.first().ok_or(Fault::Content)?;
        if value != declared || paired.is_some_and(|prior| prior != value) {
            return Err(Fault::Content);
        }
        paired = Some(value);
    }
    ImportedRandomMultiplier::new(paired.ok_or(Fault::Content)?).map_err(|_| Fault::Content)
}
