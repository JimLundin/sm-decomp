//! Independently admitted energy and complete physical frame-start input ownership.

use game::production::frame::{
    borrowed_player_preparation as preparation,
    drawing_scalar_owners::Physical,
};
use serde::Deserialize;

/// Source admission or the actual live producer is unavailable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// An independently reviewed source or initial current-owner image differs.
    Source,
}

#[derive(Deserialize)]
struct Projection {
    format: String,
    source_projection_only: bool,
    runtime_ports_qualified: bool,
    generic_scalar_callbacks_allowed: bool,
    binding: Vec<Binding>,
}

#[derive(Deserialize)]
struct Binding {
    name: String,
    private_wram_offset: usize,
    quantity_width: usize,
    alias_group: String,
    actual_owner_revalidation_required: bool,
}

/// Decode initial current energy from the independently admitted original owner image.
/// This is an independent source comparison quantity, never another writable energy owner.
/// The actual complete health authority supplies Drawing after its Maintenance completion.
/// Private future validation encodings are neither parsed nor used as initializers.
/// # Errors
/// Different pinned source, incomplete aliases or unavailable actual original quantity.
pub fn energy_origin(bytes: &[u8], text: &str) -> Result<u16, Fault> {
    use crate::lock::Sha256;
    let pin = |bytes: &[u8], expected: &str| {
        Sha256::try_from(expected).is_ok_and(|expected| Sha256::of(bytes) == expected)
    };
    if !pin(
        bytes,
        "bfee6ac13da413a36ad42194a36aa3224af6a84e818092b1e7040ed4782f03db",
    ) || !pin(
        text.as_bytes(),
        "0c81d8e253980a49b9973e8fc83caa9e883aeb989ba19a7c6ef7fe4fdcd911c3",
    ) {
        return Err(Fault::Source);
    }
    let source: Projection = toml::from_str(text).map_err(|_| Fault::Source)?;
    if source.format != "drawing-scalar-bindings-v1"
        || !source.source_projection_only
        || source.runtime_ports_qualified
        || source.generic_scalar_callbacks_allowed
    {
        return Err(Fault::Source);
    }
    let aliases = source
        .binding
        .iter()
        .filter(|binding| binding.name == "CurrentHealth")
        .collect::<Vec<_>>();
    let [first, second] = aliases.as_slice() else {
        return Err(Fault::Source);
    };
    if first.private_wram_offset != second.private_wram_offset
        || first.quantity_width != 2
        || second.quantity_width != 2
        || first.alias_group != second.alias_group
        || !first.actual_owner_revalidation_required
        || !second.actual_owner_revalidation_required
    {
        return Err(Fault::Source);
    }
    let end = first
        .private_wram_offset
        .checked_add(2)
        .ok_or(Fault::Source)?;
    let value = bytes
        .get(first.private_wram_offset..end)
        .ok_or(Fault::Source)?;
    let value = u16::from_le_bytes(value.try_into().map_err(|_| Fault::Source)?);
    Ok(value)
}

/// Join full physical held/pressed source aliases to the existing admitted PoseInput origin.
/// The source image is the frame-start owner left by its prior poll; no current-frame poll
/// completion is required. Bindings remain the actual independently admitted production owner.
/// # Errors
/// A source pin, complete alias or actual current dependency differs.
pub fn physical_origin(
    bytes: &[u8],
    text: &str,
    actual: &preparation::PoseInputOrigin,
) -> Result<Physical, Fault> {
    // Energy admission checks the same complete source pins and projection identity.
    let _ = energy_origin(bytes, text)?;
    let source: Projection = toml::from_str(text).map_err(|_| Fault::Source)?;
    for (name, expected, count) in [
        ("CurrentFramePad", actual.held.0, 2),
        ("CurrentFramePressed", actual.pressed.0, 1),
    ] {
        let aliases = source
            .binding
            .iter()
            .filter(|binding| binding.name == name)
            .collect::<Vec<_>>();
        if aliases.len() != count {
            return Err(Fault::Source);
        }
        let first = aliases.first().ok_or(Fault::Source)?;
        for alias in &aliases {
            if alias.private_wram_offset != first.private_wram_offset
                || alias.quantity_width != 2
                || alias.alias_group != first.alias_group
                || !alias.actual_owner_revalidation_required
            {
                return Err(Fault::Source);
            }
        }
        let end = first
            .private_wram_offset
            .checked_add(2)
            .ok_or(Fault::Source)?;
        let raw = bytes
            .get(first.private_wram_offset..end)
            .ok_or(Fault::Source)?;
        let value = u16::from_le_bytes(raw.try_into().map_err(|_| Fault::Source)?);
        if value != expected {
            return Err(Fault::Source);
        }
    }
    Ok(Physical::from_origin(actual))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_live_origin_cannot_become_current_energy() {
        assert!(matches!(energy_origin(&[], ""), Err(Fault::Source)));
    }

    #[test]
    #[ignore = "requires independently admitted private scalar source and current origin"]
    fn actual_energy_source_aliases_match_complete_health_origin() {
        let bytes = std::fs::read(std::env::var("SM_FRAME_CURRENT_ORIGIN").unwrap()).unwrap();
        let text = std::fs::read_to_string(std::env::var("SM_FRAME_DRAWING_SCALAR_PROOF").unwrap())
            .unwrap();
        let health_projection = std::fs::read_to_string(std::env::var("SM_HEALTH_PROJECTION").unwrap()).unwrap();
        let health = super::super::frame_health_inputs::origin(&bytes, &health_projection).unwrap();
        assert!(energy_origin(&bytes, &text).unwrap() == health.initial.0.energy.whole.get());
    }
}
