//! Decode the two current shared colour banks and the bounded ordinary opportunity.

use super::{data_failure, frame_origin::profiles as frame_profiles, result::Outcome};
use crate::lock::Sha256;
use data::timing::ContentRegion;
use game::production::frame::samus_palette::{Contract, Palette};

#[derive(serde::Deserialize)]
struct Location {
    name: String,
    offset: usize,
    width: usize,
}
#[derive(serde::Deserialize)]
struct Locations {
    field: Vec<Location>,
}
#[derive(serde::Deserialize)]
struct Span {
    ordinal: usize,
    first_exposure: u64,
    end_exposure: u64,
}
#[derive(serde::Deserialize)]
struct Codecs {
    room_words: std::collections::BTreeMap<String, u16>,
    grapple_words: std::collections::BTreeMap<String, u16>,
}
#[derive(serde::Deserialize)]
struct Edge {
    template: Span,
    codecs: Codecs,
}

pub(super) fn origin(
    wram: &[u8],
    locations: &str,
    edge: &str,
    templates: &frame_profiles::Templates,
    initial: &mut Vec<ContentRegion>,
) -> Result<(Palette, Contract), Outcome> {
    let fault = || data_failure(None, "ordinary palette original authority");
    let authenticated =
        |bytes: &[u8], pin: &str| Sha256::try_from(pin).is_ok_and(|pin| Sha256::of(bytes) == pin);
    if !authenticated(
        wram,
        "bfee6ac13da413a36ad42194a36aa3224af6a84e818092b1e7040ed4782f03db",
    ) || !authenticated(
        locations.as_bytes(),
        "661f6c81c9473a54431150a622f669823c233be0f81a84b3e412489e8682fd82",
    ) || !authenticated(
        edge.as_bytes(),
        "e9aa2da95751b4e1153c3104523bffe9e07eabd9d63decb46f830a8508b775c2",
    ) {
        return Err(fault());
    }
    let locations: Locations = toml::from_str(locations).map_err(|_| fault())?;
    let edge: Edge = toml::from_str(edge).map_err(|_| fault())?;
    let region = |name: &str, width: usize| {
        let loc = locations
            .field
            .iter()
            .find(|loc| loc.name == name && loc.width == width)?;
        wram.get(loc.offset..loc.offset.checked_add(width)?)
    };
    let word = |name| Some(u16::from_le_bytes(region(name, 2)?.try_into().ok()?));
    if [
        "appearance override",
        "stored glow",
        "beam charge",
        "special colouring",
        "hurt flash",
    ]
    .into_iter()
    .any(|name| word(name) != Some(0))
        || word("room colour treatment") != edge.codecs.room_words.get("Standard").copied()
        || word("grapple") != edge.codecs.grapple_words.get("Dormant").copied()
    {
        return Err(fault());
    }
    let bank = |name| {
        region(name, 512)?
            .as_chunks::<2>()
            .0
            .iter()
            .map(|bytes| types::Colour::new(u16::from_le_bytes(*bytes)))
            .collect::<Option<Vec<_>>>()?
            .try_into()
            .ok()
    };
    let current = bank("current colours").ok_or_else(fault)?;
    let target = bank("target colours").ok_or_else(fault)?;
    let bytes = region("current colours", 512).ok_or_else(fault)?;
    let mut mirrored = initial
        .iter()
        .enumerate()
        .filter(|(_, item)| item.bytes == bytes);
    let (index, _) = mirrored
        .next()
        .filter(|_| mirrored.next().is_none())
        .ok_or_else(fault)?;
    let mirror = initial.remove(index);
    let [delay, step] = word("visor schedule").ok_or_else(fault)?.to_le_bytes();
    let charge = word("charge-glow position").ok_or_else(fault)?;
    if step % 2 != 0 || charge % 2 != 0 {
        return Err(fault());
    }
    let sequences = samus_appearance::ColourSequences {
        visor_delay: samus_appearance::VisorDelay(delay),
        visor_step: samus_appearance::VisorStep(usize::from(step / 2)),
        charge_glow_position: samus_appearance::ChargeGlowPosition(usize::from(charge / 2)),
    };
    let span = edge.template;
    let part = templates.portions.get(span.ordinal).ok_or_else(fault)?;
    if part.profile != frame_profiles::ProfileName::PreWait
        || part.semantic_owner != "S5.Palette"
        || part.first_exposure != span.first_exposure
        || part.end_exposure != span.end_exposure
    {
        return Err(fault());
    }
    let template = super::frame_appearance_owner::pre_wait_template(templates, "S5.Palette")
        .map_err(|_| fault())?;
    let origin = Sha256::of(wram).0;
    Ok((
        Palette::new(
            origin,
            sequences,
            types::Palettes { current, target },
            mirror.start,
        ),
        Contract::new(origin, &template).map_err(|_| fault())?,
    ))
}
