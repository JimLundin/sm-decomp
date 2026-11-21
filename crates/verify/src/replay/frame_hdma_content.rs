//! Bounded immutable HDMA tables read from the checked player's ROM.

use crate::lock::Sha256;
use data::dma::BusAddress;
use data::timing::{ContentRegion, TransferPattern};
use game::production::timing::State;
use serde::Deserialize;

/// Source admission fails before any transfer bytes are installed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Missing or incompatible qualified source contract.
    Source,
    /// Actual origin channel configuration differs from the admitted source.
    Origin,
    /// Checked ROM identity, normalization, span or byte hash differs.
    Rom,
    /// The exact admitted read closure has missing, extra or overlapping bytes.
    Coverage,
}

// Other source fields retain audit provenance only. Runtime consumes this bounded
// read closure and validates its current hardware configuration independently.
#[derive(Deserialize)]
struct ReadClosure {
    format: String,
    source_scope: String,
    checked_player_rom_sha256: String,
    #[serde(flatten)]
    backing: RomBacking,
    forecast_outputs_exported: bool,
    raw_rom_bytes_exported: bool,
    required_rom_read_addresses: Vec<u32>,
    channel_binding: Vec<Binding>,
    rom_region: Vec<Region>,
}

#[derive(serde::Deserialize)]
struct RomBacking {
    player_rom_import_required: bool,
    whole_rom_fallback_allowed: bool,
    runtime_backing_from_future_snapshot_allowed: bool,
}

#[derive(Deserialize)]
struct Binding {
    channel: usize,
    #[serde(flatten)]
    channel_origin: ChannelOrigin,
    initialization_resets_completed_and_table_cursor: bool,
    indirect: bool,
    mode: u8,
    indirect_pointer_bank: u8,
    source_configuration_authority: String,
    runtime_source_values_not_future_state: bool,
}

#[derive(serde::Deserialize)]
struct ChannelOrigin {
    #[serde(rename = "origin_enabled")]
    enabled: bool,
    #[serde(rename = "origin_completed")]
    completed: bool,
    #[serde(rename = "origin_source_bank")]
    source_bank: u8,
    #[serde(rename = "origin_source_offset")]
    source_offset: u16,
}

#[derive(Deserialize)]
struct Region {
    name: String,
    bank: u8,
    offset: u16,
    length: usize,
    normalized_rom_file_offset: usize,
    required_region_sha256: String,
    import_recipe: String,
    runtime_backing: String,
    reference_bytes_exported: bool,
    channels: Vec<usize>,
}

const fn mode(pattern: TransferPattern) -> u8 {
    match pattern {
        TransferPattern::Single => 0,
        TransferPattern::Pair => 1,
        TransferPattern::Double => 2,
        TransferPattern::DoublePair => 3,
        TransferPattern::Quad => 4,
        TransferPattern::PairPair => 5,
        TransferPattern::DoubleAgain => 6,
        TransferPattern::DoublePairAgain => 7,
    }
}

fn address(bank: u8, offset: u16) -> u32 {
    u32::from(bank) << 16 | u32::from(offset)
}

/// Import only the complete qualified immutable initial descriptor/pointer closure.
/// Numeric source metadata never supplies device bytes or future runtime values.
/// # Errors
/// Refuses changed initial channels, ROM identity, span normalization or coverage.
pub fn admit(image: &[u8], origin: &State, text: &str) -> Result<Vec<ContentRegion>, Fault> {
    let accepted_source =
        Sha256::try_from("93d38e2ece16a38549aa652f1b2c9741aad6445786de9a50520cfd4f81f7e9e6")
            .map_err(|_| Fault::Source)?;
    if Sha256::of(text.as_bytes()) != accepted_source {
        return Err(Fault::Source);
    }
    let source: ReadClosure = toml::from_str(text).map_err(|_| Fault::Source)?;
    import(image, origin, &source)
}

fn import(image: &[u8], origin: &State, source: &ReadClosure) -> Result<Vec<ContentRegion>, Fault> {
    if source.format != "bounded-initial-hdma-rom-backing-v1"
        || source.source_scope != "AcceptedInitialHdmaDescriptorPointerAndPayloadReadClosureOnly"
        || !source.backing.player_rom_import_required
        || source.backing.whole_rom_fallback_allowed
        || source.backing.runtime_backing_from_future_snapshot_allowed
        || source.forecast_outputs_exported
        || source.raw_rom_bytes_exported
        || source.channel_binding.len() != 2
        || source.rom_region.len() != 2
    {
        return Err(Fault::Source);
    }
    if Sha256::try_from(source.checked_player_rom_sha256.as_str()).map_err(|_| Fault::Rom)?
        != Sha256::of(image)
    {
        return Err(Fault::Rom);
    }
    let bound = initial_channels(origin, &source.channel_binding)?;
    let mut regions = Vec::new();
    let mut admitted_addresses = std::collections::BTreeSet::new();
    for (ordinal, region) in source.rom_region.iter().enumerate() {
        if region.name != format!("InitialHdmaRomRegion{ordinal}")
            || region.import_recipe != "ReadExactSpanFromCheckedPlayersRomAtNormalizedLoRomAddress"
            || region.runtime_backing != "ImmutableInitialTransferContent"
            || region.reference_bytes_exported
            || region.length == 0
            || region.offset < 0x8000
            || matches!(region.bank, 0x7e | 0x7f)
            || usize::from(region.offset)
                .checked_add(region.length)
                .is_none_or(|end| end > 0x10000)
            || region.channels.is_empty()
            || region.channels.iter().any(|slot| !bound.contains(slot))
        {
            return Err(Fault::Source);
        }
        let normalized =
            (usize::from(region.bank & 0x7f) << 15) | usize::from(region.offset & 0x7fff);
        if region.normalized_rom_file_offset != normalized {
            return Err(Fault::Rom);
        }
        let end = normalized.checked_add(region.length).ok_or(Fault::Rom)?;
        let bytes = image.get(normalized..end).ok_or(Fault::Rom)?;
        if Sha256::try_from(region.required_region_sha256.as_str()).map_err(|_| Fault::Rom)?
            != Sha256::of(bytes)
        {
            return Err(Fault::Rom);
        }
        for offset in 0..region.length {
            if !admitted_addresses.insert(
                address(region.bank, region.offset)
                    .checked_add(u32::try_from(offset).map_err(|_| Fault::Coverage)?)
                    .ok_or(Fault::Coverage)?,
            ) {
                return Err(Fault::Coverage);
            }
        }
        regions.push(ContentRegion {
            start: BusAddress {
                bank: region.bank,
                offset: region.offset,
            },
            bytes: bytes.to_vec(),
        });
    }
    let required: std::collections::BTreeSet<_> =
        source.required_rom_read_addresses.iter().copied().collect();
    if required.len() != source.required_rom_read_addresses.len()
        || required != admitted_addresses
        || source.channel_binding.iter().any(|binding| {
            !required.contains(&address(
                binding.channel_origin.source_bank,
                binding.channel_origin.source_offset,
            ))
        })
    {
        return Err(Fault::Coverage);
    }
    Ok(regions)
}

fn initial_channels(origin: &State, bindings: &[Binding]) -> Result<Vec<usize>, Fault> {
    let enabled: Vec<_> = origin
        .hardware
        .transfers
        .channels
        .iter()
        .enumerate()
        .filter_map(|(slot, channel)| channel.line.enabled.then_some(slot))
        .collect();
    let bound: Vec<_> = bindings.iter().map(|binding| binding.channel).collect();
    if bound != enabled {
        return Err(Fault::Origin);
    }
    for binding in bindings {
        let channel = origin
            .hardware
            .transfers
            .channels
            .get(binding.channel)
            .ok_or(Fault::Origin)?;
        if !binding.channel_origin.enabled
            || !binding.channel_origin.completed
            || !binding.initialization_resets_completed_and_table_cursor
            || !binding.runtime_source_values_not_future_state
            || binding.source_configuration_authority
                != "ActualImportedOriginHardwareChannelConfiguration"
            || channel.line.enabled != binding.channel_origin.enabled
            || channel.line.completed != binding.channel_origin.completed
            || channel.config.source_bank != binding.channel_origin.source_bank
            || channel.source_offset != binding.channel_origin.source_offset
            || channel.config.indirect != binding.indirect
            || channel.config.indirect_bank != binding.indirect_pointer_bank
            || mode(channel.config.pattern) != binding.mode
        {
            return Err(Fault::Origin);
        }
    }
    Ok(bound)
}

#[cfg(test)]
#[test]
#[ignore = "requires reviewed private initial HDMA contract, origin hardware and player ROM"]
fn checked_initial_rom_tables_reject_changed_owner_hash_and_missing_span() {
    let text =
        std::fs::read_to_string(std::env::var_os("SM_INITIAL_HDMA_PACKET").unwrap()).unwrap();
    let image = std::fs::read(std::env::var_os("SM_ROM").unwrap()).unwrap();
    let hardware = std::fs::read(std::env::var_os("SM_INITIAL_HDMA_HARDWARE").unwrap()).unwrap();
    let origin = super::frame_origin::hardware_seed::decode(
        &hardware,
        data::timing::IrqPolicy::Eligible,
        game::production::timing::PhysicalPad::default(),
    )
    .unwrap();
    let regions = admit(&image, &origin, &text).unwrap();
    assert_eq!(regions.len(), 2);
    let source: ReadClosure = toml::from_str(&text).unwrap();
    let mut changed = origin.clone();
    changed.hardware.transfers.channels[source.channel_binding[0].channel].source_offset ^= 1;
    assert_eq!(
        admit(&image, &changed, &text).map(|_| ()),
        Err(Fault::Origin)
    );
    let mut altered_image = image.clone();
    altered_image[source.rom_region[0].normalized_rom_file_offset] ^= 1;
    assert_eq!(
        admit(&altered_image, &origin, &text).map(|_| ()),
        Err(Fault::Rom)
    );
    let mut value: toml::Value = toml::from_str(&text).unwrap();
    value["rom_region"].as_array_mut().unwrap().pop();
    assert_eq!(
        admit(&image, &origin, &toml::to_string(&value).unwrap()).map(|_| ()),
        Err(Fault::Source)
    );
    let mut shortened: toml::Value = toml::from_str(&text).unwrap();
    let last = usize::from(source.rom_region[0].offset) + source.rom_region[0].length - 1;
    let removed = address(source.rom_region[0].bank, u16::try_from(last).unwrap());
    shortened["required_rom_read_addresses"]
        .as_array_mut()
        .unwrap()
        .retain(|value| value.as_integer() != Some(i64::from(removed)));
    let first = &mut shortened["rom_region"].as_array_mut().unwrap()[0];
    first["length"] = toml::Value::Integer(i64::try_from(source.rom_region[0].length - 1).unwrap());
    let start = source.rom_region[0].normalized_rom_file_offset;
    first["required_region_sha256"] = toml::Value::String(
        Sha256::of(&image[start..start + source.rom_region[0].length - 1]).to_string(),
    );
    let shortened = toml::to_string(&shortened).unwrap();
    // Reproduce the review's red case: internally consistent shortened coverage
    // passes byte/config checks, so source identity must gate the public importer.
    assert!(import(&image, &origin, &toml::from_str(&shortened).unwrap()).is_ok());
    assert_eq!(
        admit(&image, &origin, &shortened).map(|_| ()),
        Err(Fault::Source)
    );
}
