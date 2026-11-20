//! Actual-current Camera receipt qualification and checked immutable tuning at the edge.

use super::frame_origin::profiles::{Effect, Profile, ProfileName, Templates};
use crate::lock::Sha256;
use data::{
    rom::{Rom, RomAddress},
    timing::{Sample, Template},
};
use game::production::frame::camera_stage::{
    Activity, Inputs, Lead, PhysicalContract, RequiredCentre, RequiredDependencies,
    RequiredScreens, RequiredView, Room, Screen, Tuning,
};
use serde::Deserialize;

#[cfg(test)]
mod tests;

/// Missing checked content, actual current dependency or complete physical owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The source contract or origin does not match its accepted pin.
    Source,
    /// Current origin quantities cannot form the admitted neighbour inputs.
    Inputs,
    /// Immutable tuning is missing or does not roundtrip its exact ROM bytes.
    Tuning,
    /// Full owner boundaries or arithmetic namespaces do not match the chosen operation.
    Profile,
}

#[derive(Deserialize)]
struct Source {
    format: String,
    camera_current_input: Vec<Field>,
    checked_camera_tuning: Vec<Parameter>,
    #[serde(rename = "source_pin")]
    pins: Vec<Pin>,
}
#[derive(Deserialize)]
struct Field {
    name: String,
    private_wram_offset: usize,
    quantity_width: usize,
    private_source_encoding: u16,
    actual_model_join_required: bool,
}
#[derive(Deserialize)]
struct Parameter {
    name: String,
    private_rom_address: u32,
    quantity_width: usize,
    #[serde(rename = "typed_parameter")]
    value: u16,
    content_bytes_sha256: String,
}
#[derive(Deserialize)]
struct Pin {
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
struct RoomSource {
    current_room_projection: Vec<RoomField>,
}
#[derive(Deserialize)]
struct RoomField {
    private_wram_offset: usize,
    quantity_width: usize,
    private_current_encoding: Option<u16>,
    private_current_sha256: Option<String>,
}

fn room_source(origin: &[u8], text: &str) -> Result<(), Fault> {
    let pin = Sha256::try_from("8deb6ea24a7d2871830f077eda5abbb926e61227c7ddb88e3baf9df3247562e0")
        .map_err(|_| Fault::Source)?;
    if Sha256::of(text.as_bytes()) != pin {
        return Err(Fault::Source);
    }
    let source: RoomSource = toml::from_str(text).map_err(|_| Fault::Source)?;
    for field in source.current_room_projection {
        let end = field
            .private_wram_offset
            .checked_add(field.quantity_width)
            .ok_or(Fault::Inputs)?;
        let bytes = origin
            .get(field.private_wram_offset..end)
            .ok_or(Fault::Inputs)?;
        match (field.private_current_encoding, field.private_current_sha256) {
            (Some(value), None) if field.quantity_width == 2 => {
                if bytes != value.to_le_bytes() {
                    return Err(Fault::Inputs);
                }
            }
            (None, Some(hash)) => {
                if Sha256::of(bytes)
                    != Sha256::try_from(hash.as_str()).map_err(|_| Fault::Source)?
                {
                    return Err(Fault::Inputs);
                }
            }
            (Some(_), None | Some(_)) | (None, None) => return Err(Fault::Source),
        }
    }
    Ok(())
}

/// Source-gated unchanged neighbour inputs plus validation-only current producer predicates.
/// No recorded Movement or future Camera result supplies the gameplay input projection.
pub struct Content {
    inputs: Inputs,
    fields: Vec<Field>,
}

fn word(origin: &[u8], at: usize) -> Result<u16, Fault> {
    let end = at.checked_add(2).ok_or(Fault::Inputs)?;
    let bytes = origin.get(at..end).ok_or(Fault::Inputs)?;
    Ok(u16::from_le_bytes(
        bytes.try_into().map_err(|_| Fault::Inputs)?,
    ))
}

fn read_field(origin: &[u8], field: &Field) -> Result<u16, Fault> {
    match field.quantity_width {
        1 => origin
            .get(field.private_wram_offset)
            .copied()
            .map(u16::from)
            .ok_or(Fault::Inputs),
        2 => word(origin, field.private_wram_offset),
        _ => Err(Fault::Source),
    }
}

fn parameter(rom: &Rom, source: &Source, name: &str) -> Result<u16, Fault> {
    let mut parameters = source
        .checked_camera_tuning
        .iter()
        .filter(|parameter| parameter.name == name);
    let parameter = parameters.next().ok_or(Fault::Tuning)?;
    if parameters.next().is_some() || parameter.quantity_width != 2 {
        return Err(Fault::Tuning);
    }
    let [low, high, bank, _] = parameter.private_rom_address.to_le_bytes();
    let bytes = rom
        .lorom()
        .read(RomAddress::from_long([low, high, bank]), 2)
        .map_err(|_| Fault::Tuning)?;
    let pin =
        Sha256::try_from(parameter.content_bytes_sha256.as_str()).map_err(|_| Fault::Tuning)?;
    if Sha256::of(bytes) != pin || bytes != parameter.value.to_le_bytes() {
        return Err(Fault::Tuning);
    }
    Ok(parameter.value)
}

impl Content {
    /// Admit the accepted complete source contract and its one independent current origin.
    /// Only unchanged neighbour quantities and checked ROM parameters become game inputs.
    /// # Errors
    /// Wrong source/origin pin, unavailable unchanged owner, malformed room or wrong ROM tuning.
    pub fn import(origin: &[u8], rom: &Rom, text: &str, room_text: &str) -> Result<Self, Fault> {
        room_source(origin, room_text)?;
        let pin =
            Sha256::try_from("de6df0ee7e1eb8f25257f301769c0ba8254bad570abb971085d87fb6e699211c")
                .map_err(|_| Fault::Source)?;
        if Sha256::of(text.as_bytes()) != pin {
            return Err(Fault::Source);
        }
        let source: Source = toml::from_str(text).map_err(|_| Fault::Source)?;
        if source.format != "bounded-current-camera-owner-v4" {
            return Err(Fault::Source);
        }
        let mut origins = source
            .pins
            .iter()
            .filter(|pin| pin.path.ends_with("corrected-origin-wram.bin"));
        let origin_pin = origins.next().ok_or(Fault::Source)?;
        if origins.next().is_some()
            || Sha256::of(origin)
                != Sha256::try_from(origin_pin.sha256.as_str()).map_err(|_| Fault::Source)?
        {
            return Err(Fault::Source);
        }
        for field in &source.camera_current_input {
            if !field.actual_model_join_required
                && read_field(origin, field)? != field.private_source_encoding
            {
                return Err(Fault::Inputs);
            }
        }
        let minimum = parameter(rom, &source, "HorizontalMinimumAdvance")?;
        if minimum != parameter(rom, &source, "VerticalMinimumAdvance")? {
            return Err(Fault::Tuning);
        }
        let right = parameter(rom, &source, "RightLeadDistance")?;
        let mut tuning = Tuning::capture(&world_camera::Rules::REFERENCE);
        tuning.extra = u32::from(minimum) << 16;
        let [_, left] = tuning.ahead;
        tuning.ahead = [right, left];
        let screens = origin
            .get(0xcd20..0xcd52)
            .ok_or(Fault::Inputs)?
            .iter()
            .map(|value| match value {
                0 => Screen::Closed,
                1 => Screen::Short,
                _ => Screen::Open,
            })
            .collect();
        let inputs = Inputs {
            room: Room {
                identity: import::world::id(data::room::RoomKey::from_room_pointer(word(
                    origin, 0x079b,
                )?))
                .ok_or(Fault::Inputs)?,
                screens,
                size: [word(origin, 0x07a9)?, word(origin, 0x07ab)?],
                heights: [word(origin, 0x07ad)?, word(origin, 0x07af)?],
            },
            lead: Lead::Ahead,
            time_stopped: word(origin, 0x0a78)? != 0,
            tuning,
            grapple: if word(origin, 0x0cf8)? == 0 {
                Activity::Inactive
            } else {
                Activity::Active
            },
            knockback: if word(origin, 0x0a52)? == 0 {
                Activity::Inactive
            } else {
                Activity::Active
            },
            hook: if word(origin, 0x07e9)? == 0 {
                Activity::Inactive
            } else {
                Activity::Active
            },
        };
        Ok(Self {
            inputs,
            fields: source.camera_current_input,
        })
    }

    /// Complete typed current neighbour projection, with no native selector or source address.
    #[must_use]
    pub fn inputs(&self) -> Inputs {
        self.inputs.clone()
    }

    /// Normalize accepted source evidence into immutable production-owned physical content.
    /// # Errors
    /// Inconsistent source facts, unsupported semantic requirements or incomplete physical owner.
    pub fn contract(
        &self,
        templates: &Templates,
        envelope_text: &str,
    ) -> Result<PhysicalContract, Fault> {
        Ok(PhysicalContract::new(
            self.dependencies()?,
            contract_template(templates, envelope_text)?,
        ))
    }

    fn dependencies(&self) -> Result<RequiredDependencies, Fault> {
        let value = |name: &str| {
            let mut fields = self.fields.iter().filter(|field| field.name == name);
            let first = fields.next().ok_or(Fault::Source)?.private_source_encoding;
            if fields.any(|field| field.private_source_encoding != first) {
                return Err(Fault::Source);
            }
            Ok(first)
        };
        if value("CurrentMovementKind")? & 0xff != 1
            || value("CurrentHeading")? & 0xff != 8
            || value("HorizontalAccelerationMode")? != 0
            || value("CurrentCameraYScreen")? != value("CurrentCameraY")? >> 8
            || value("RoomWidthScreens")? != self.inputs.room.size[0]
            || value("RoomHeightScreens")? != self.inputs.room.size[1]
            || value("FreezeLow")? != u16::from(self.inputs.time_stopped)
            || value("FreezeHigh")? != u16::from(self.inputs.time_stopped)
            || value("GrappleScrollMode")? != 0
            || value("KnockbackDirection")? != 0
            || value("ScrollFinishedHook")? != 0
        {
            return Err(Fault::Source);
        }
        let centre = |prefix: &str| -> Result<RequiredCentre, Fault> {
            Ok(RequiredCentre {
                pixels: [
                    value(&format!("{prefix}CentreX"))?,
                    value(&format!("{prefix}CentreY"))?,
                ],
                fractions: [
                    value(&format!("{prefix}CentreSubX"))?,
                    value(&format!("{prefix}CentreSubY"))?,
                ],
            })
        };
        let screens = self
            .fields
            .iter()
            .filter(|field| field.name == "CurrentScrollColour")
            .map(|field| {
                let decode = |byte| match byte {
                    0 => Ok(Screen::Closed),
                    1 => Ok(Screen::Short),
                    2 => Ok(Screen::Open),
                    _ => Err(Fault::Source),
                };
                let [low, high] = field.private_source_encoding.to_le_bytes();
                Ok(RequiredScreens {
                    cell: field
                        .private_wram_offset
                        .checked_sub(0xcd20)
                        .ok_or(Fault::Source)?,
                    screens: [decode(low)?, decode(high)?],
                })
            })
            .collect::<Result<Vec<_>, Fault>>()?;
        Ok(RequiredDependencies {
            current: centre("Current")?,
            previous: centre("Previous")?,
            view: RequiredView {
                x: value("PreviousCameraX")?,
                fraction: value("PreviousCameraSubX")?,
                y: value("CurrentCameraY")?,
            },
            inputs: Box::new(self.inputs.clone()),
            screens,
        })
    }
}

fn contract_template(templates: &Templates, text: &str) -> Result<Template, Fault> {
    let profile = camera_profile(templates, &envelope(text)?)?;
    arithmetic(&profile)?;
    profile.template().map_err(|_| Fault::Profile)
}

#[derive(Deserialize)]
struct Envelope {
    profile: ProfileName,
    first_exposure: u64,
    end_exposure: u64,
    constituent_portion: Vec<Part>,
}
#[derive(Deserialize)]
struct Part {
    profile: ProfileName,
    first_exposure: u64,
    end_exposure: u64,
    receipt_contract: String,
}

fn envelope(text: &str) -> Result<Envelope, Fault> {
    let pin = Sha256::try_from("0c61adb22f24e7d2a39e8d636dac6d0690d9ce39928f0adc76e168cb898e8a27")
        .map_err(|_| Fault::Source)?;
    if Sha256::of(text.as_bytes()) != pin {
        return Err(Fault::Source);
    }
    toml::from_str(text).map_err(|_| Fault::Source)
}

fn camera_profile(templates: &Templates, envelope: &Envelope) -> Result<Profile, Fault> {
    if envelope.profile != ProfileName::PreWait || envelope.constituent_portion.len() != 6 {
        return Err(Fault::Profile);
    }
    let mut end = envelope.first_exposure;
    for part in &envelope.constituent_portion {
        let mut portions = templates.portions.iter().filter(|portion| {
            portion.profile == part.profile
                && portion.first_exposure == part.first_exposure
                && portion.end_exposure == part.end_exposure
                && portion.receipt_contract == part.receipt_contract
        });
        if portions.next().is_none()
            || portions.next().is_some()
            || part.profile != envelope.profile
            || part.first_exposure != end
            || part.end_exposure <= part.first_exposure
        {
            return Err(Fault::Profile);
        }
        end = part.end_exposure;
    }
    if end != envelope.end_exposure {
        return Err(Fault::Profile);
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == envelope.profile);
    let profile = profiles.next().ok_or(Fault::Profile)?;
    if profiles.next().is_some() {
        return Err(Fault::Profile);
    }
    profile
        .slice(envelope.first_exposure, envelope.end_exposure)
        .map_err(|_| Fault::Profile)
}

fn arithmetic(profile: &super::frame_origin::profiles::Profile) -> Result<(), Fault> {
    let mut writes = Vec::new();
    let mut samples = Vec::new();
    for run in &profile.runs {
        for _ in 0..run.count {
            match &run.effect {
                Effect::None => {}
                Effect::Publication(publication) => {
                    writes.push(publication.arithmetic.as_ref().ok_or(Fault::Profile)?);
                }
                Effect::Sample(device, task) => {
                    samples.push((*device, task.as_ref().ok_or(Fault::Profile)?));
                }
            }
        }
    }
    if writes.len() != 4 || samples.len() != 4 {
        return Err(Fault::Profile);
    }
    for (index, task) in writes.iter().enumerate() {
        let name = if index < 2 {
            "VerticalScrollZonePrimary"
        } else {
            "VerticalScrollZoneSecondary"
        };
        if task.name != name || task.quantities != ["ViewportYScreen", "RoomWidthScreens"] {
            return Err(Fault::Profile);
        }
    }
    for ((device, task), (expected_device, expected_task)) in samples.iter().zip([
        (Sample::ProductLow, "VerticalScrollZonePrimary"),
        (Sample::ProductHigh, "VerticalScrollZonePrimary"),
        (Sample::ProductLow, "VerticalScrollZoneSecondary"),
        (Sample::ProductHigh, "VerticalScrollZoneSecondary"),
    ]) {
        if *device != expected_device
            || task.name != expected_task
            || task.quantities != ["ViewportYScreen", "RoomWidthScreens"]
        {
            return Err(Fault::Profile);
        }
    }
    Ok(())
}
