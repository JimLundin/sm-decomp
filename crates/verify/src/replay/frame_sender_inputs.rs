//! Checked immutable encoding for actual frame sender requests.

use super::frame_origin::profiles::{Effect, ProfileName, Templates};
use data::rom::{Rom, RomAddress};
use game::production::frame::sender_stage::CueBinding;
use serde::Deserialize;

/// Missing or inconsistent independently admitted encoding provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Malformed or different source contract.
    Contract,
    /// ROM operand does not match its checked address/file relation.
    Operand,
    /// Missing or ambiguous actual outgoing publication namespace.
    Publication,
}

#[derive(Deserialize)]
struct Proof {
    format: String,
    cue: String,
    private_native_operand_address: u32,
    private_rom_operand_offset: usize,
    quantity_width: usize,
    consumer_device: String,
    consumer_owner: String,
    consumer_profile: String,
    other_sender_services: String,
}

/// Read the checked immutable Footstep operand and its qualified outgoing device port.
/// The caller pins this independently admitted source proof before decoding it;
/// source publication ordinals never dispatch runtime sender services.
///
/// # Errors
/// Unsupported proof, inconsistent native relation or ambiguous publication binding.
pub fn footstep(rom: &Rom, proof: &str, templates: &Templates) -> Result<CueBinding, Fault> {
    let proof: Proof = toml::from_str(proof).map_err(|_| Fault::Contract)?;
    if proof.format != "immutable-footstep-encoding-provenance-v1"
        || proof.cue != "CueId::Footstep"
        || proof.quantity_width != 2
        || proof.consumer_device != "LibraryThreeCue"
        || proof.consumer_owner != "FootstepsQueue"
        || proof.consumer_profile != "PreWait"
        || proof.other_sender_services != "ReadyEmptySoundLibrary"
    {
        return Err(Fault::Contract);
    }
    let [low, high, bank, upper] = proof.private_native_operand_address.to_le_bytes();
    if upper != 0 {
        return Err(Fault::Operand);
    }
    let address = RomAddress::from_long([low, high, bank]);
    let view = rom.lorom();
    if view.file_offset(address).map_err(|_| Fault::Operand)? != proof.private_rom_operand_offset {
        return Err(Fault::Operand);
    }
    let operand: [u8; 2] = view
        .read(address, 2)
        .map_err(|_| Fault::Operand)?
        .try_into()
        .map_err(|_| Fault::Operand)?;
    let value = u8::try_from(u16::from_le_bytes(operand)).map_err(|_| Fault::Operand)?;
    let mut ports = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == ProfileName::PreWait)
        .flat_map(|profile| &profile.runs)
        .filter_map(|run| match &run.effect {
            Effect::Publication(publication)
                if publication.device == proof.consumer_device
                    && publication.producer == proof.consumer_owner =>
            {
                Some(publication.port)
            }
            Effect::None | Effect::Sample(_, _) | Effect::Publication(_) => None,
        });
    let port = ports.next().ok_or(Fault::Publication)?;
    if ports.next().is_some() || port.output_class() != Ok(2) {
        return Err(Fault::Publication);
    }
    Ok(CueBinding {
        cue: audio_transport::CueId::Footstep,
        port,
        value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::Sha256;

    #[derive(Deserialize)]
    struct Member {
        path: std::path::PathBuf,
        sha256: String,
    }

    #[derive(Deserialize)]
    struct Manifest {
        profiles: [Member; 6],
    }

    fn pinned(path: &std::path::Path, hash: &str) -> Vec<u8> {
        let bytes = std::fs::read(path).unwrap();
        assert!(
            Sha256::of(&bytes) == Sha256::try_from(hash).unwrap(),
            "source member pin"
        );
        bytes
    }

    #[test]
    #[ignore = "requires reviewed frame source manifest, immutable encoding proof and checked player ROM"]
    fn actual_checked_operand_and_publication_binding_reject_broken_relations() {
        let path: std::path::PathBuf = std::env::var_os("SM_FRAME_SENDER_SOURCE").unwrap().into();
        let bytes = pinned(&path, &std::env::var("SM_FRAME_SENDER_SOURCE_SHA").unwrap());
        let manifest: Manifest = toml::from_str(std::str::from_utf8(&bytes).unwrap()).unwrap();
        let root = path.parent().unwrap();
        let texts: Vec<String> = manifest
            .profiles
            .iter()
            .map(|member| {
                let path = if member.path.is_absolute() {
                    member.path.clone()
                } else {
                    root.join(&member.path)
                };
                String::from_utf8(pinned(&path, &member.sha256)).unwrap()
            })
            .collect();
        let texts: [&str; 6] = texts
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let templates = super::super::frame_origin::profiles::decode(texts).unwrap();
        let proof_path: std::path::PathBuf =
            std::env::var_os("SM_FRAME_SENDER_ENCODING").unwrap().into();
        let proof = String::from_utf8(pinned(
            &proof_path,
            &std::env::var("SM_FRAME_SENDER_ENCODING_SHA").unwrap(),
        ))
        .unwrap();
        let rom = Rom::new(std::fs::read(std::env::var_os("SM_ROM").unwrap()).unwrap()).unwrap();
        let binding = footstep(&rom, &proof, &templates).unwrap();
        assert!(
            binding.cue == audio_transport::CueId::Footstep,
            "actual semantic cue binding"
        );
        assert!(
            binding.port.output_class() == Ok(2),
            "actual outgoing publication namespace"
        );
        let mut altered: toml::Value = toml::from_str(&proof).unwrap();
        let offset = altered.get_mut("private_rom_operand_offset").unwrap();
        *offset = toml::Value::Integer(offset.as_integer().unwrap().wrapping_add(1));
        assert!(
            footstep(&rom, &toml::to_string(&altered).unwrap(), &templates) == Err(Fault::Operand),
            "address/file relation fault"
        );
        let mut absent = templates.clone();
        for profile in &mut absent.profiles {
            profile.runs.retain(|run| !matches!(&run.effect, Effect::Publication(publication) if publication.device == "LibraryThreeCue"));
        }
        assert!(
            footstep(&rom, &proof, &absent) == Err(Fault::Publication),
            "missing output identity fault"
        );
        assert!(
            footstep(&rom, &proof, &templates) == Ok(binding),
            "restored complete binding"
        );
    }
}
