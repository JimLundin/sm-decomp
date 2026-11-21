//! One independently admitted completed-return origin for every selected frame owner.

#[cfg(test)]
use super::Sources;
use super::index::Case;
#[cfg(test)]
use super::readers::{AdmittedCase, InputReader};
use super::{data_failure, movement, result::Outcome};
use crate::lock::Sha256;
use canonical::model::Model;
#[cfg(test)]
use game::production::{Production, ProductionContent, ProductionRules, ProductionSeed};
use game::production::{appearance, frame};
use serde::Deserialize;
use std::path::PathBuf;

mod admission;
// Compare decoded observations without formatting captured values on failure.
#[cfg(test)]
macro_rules! assert_observation_eq {
    ($actual:expr, $expected:expr $(,)?) => {
        assert_observation_eq!($actual, $expected, "observed values differ")
    };
    ($actual:expr, $expected:expr, $($message:tt)+) => {
        assert!(::core::cmp::PartialEq::eq(&$actual, &$expected), $($message)+)
    };
}

#[cfg(test)]
#[path = "frame_origin/adjacent_tests.rs"]
mod adjacent_tests;
#[path = "frame_appearance_seed.rs"]
mod appearance_seed;
#[path = "frame_borrowed_receipts.rs"]
mod borrowed_receipts;
#[cfg(test)]
#[path = "frame_origin/cannon_tests.rs"]
mod cannon_tests;
#[cfg(test)]
#[path = "frame_origin/countdown_scene_tests.rs"]
mod countdown_scene_tests;
#[cfg(test)]
#[path = "frame_origin/countdown_tests.rs"]
mod countdown_tests;
#[cfg(test)]
#[path = "frame_origin/countdown_timing_tests.rs"]
mod countdown_timing_tests;
#[path = "frame_gameplay_receipt.rs"]
mod gameplay_receipt;
#[path = "frame_hardware_seed.rs"]
pub(super) mod hardware_seed;
#[cfg(test)]
#[path = "frame_origin/health_owner_tests.rs"]
mod health_owner_tests;
#[path = "frame_origin/hud_display.rs"]
mod hud_display;
#[cfg(test)]
#[path = "frame_origin/hud_map_tests.rs"]
mod hud_map_tests;
#[path = "frame_interrupt_inputs.rs"]
mod interrupt_inputs;
#[cfg(test)]
#[path = "frame_origin/late_hud_tests.rs"]
mod late_hud_tests;
#[path = "frame_layer_blend_receipt.rs"]
mod layer_blend_receipt;
#[cfg(test)]
#[path = "frame_origin/pad_ingress_tests.rs"]
mod pad_ingress_tests;
#[path = "frame_prelude_seed.rs"]
mod prelude_seed;
#[path = "frame_preparation_receipts.rs"]
mod preparation_receipts;
#[path = "frame_profiles.rs"]
pub(super) mod profiles;
#[path = "frame_random_inputs.rs"]
mod random_inputs;
#[cfg(test)]
#[path = "frame_origin/runtime_helpers.rs"]
mod runtime_helpers;
#[path = "frame_sender_seed.rs"]
mod sender_seed;
pub(super) use admission::admit;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Member {
    path: PathBuf,
    sha256: String,
}

impl Member {
    fn resolve(&mut self, root: &std::path::Path) {
        if self.path.is_relative() {
            self.path = root.join(&self.path);
        }
    }
    fn pin(&self) -> Result<Sha256, Outcome> {
        Sha256::try_from(self.sha256.as_str())
            .map_err(|_| data_failure(None, "frame source member pin"))
    }

    fn read(&self) -> Result<Vec<u8>, Outcome> {
        let bytes =
            std::fs::read(&self.path).map_err(|_| data_failure(None, "frame source member"))?;
        if Sha256::of(&bytes) != self.pin()? {
            return Err(data_failure(None, "frame source member pin"));
        }
        Ok(bytes)
    }

    fn text(&self) -> Result<String, Outcome> {
        String::from_utf8(self.read()?)
            .map_err(|_| data_failure(None, "frame source text encoding"))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InitialContent {
    member: Member,
    bank: u8,
    offset: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: u16,
    host_budget: u64,
    wram: Member,
    hardware: Member,
    pads: Member,
    current_pad_row: usize,
    preview_pad_row: usize,
    apuram: Member,
    profiles: [Member; 6],
    music_completion: Member,
    #[serde(rename = "haze_bands")]
    _haze_bands: Option<Member>,
    #[serde(rename = "haze_ready")]
    _haze_ready: Option<Member>,
    #[serde(rename = "haze_terminal")]
    _haze_terminal: Option<Member>,
    buffer_regions: Member,
    irq_current: Member,
    irq_recipe: Member,
    irq_terminal: Member,
    layer_blend: Member,
    random_requests: Member,
    random_import: Member,
    // Older authenticated manifests also name a terminal witness.
    #[serde(rename = "random_completion")]
    _random_completion: Option<Member>,
    #[serde(rename = "builder_completion")]
    _builder_completion: Option<Member>,
    #[serde(rename = "builder_terminal")]
    _builder_terminal: Option<Member>,
    gameplay_completion: Member,
    actor_completion: Member,
    debug_completion: Member,
    palette_completion: Member,
    player_entry_current: Member,
    player_entry_labels: Member,
    player_entry_links: Member,
    #[serde(default)]
    sender_current_profile: Option<Member>,
    #[serde(default)]
    sender_bus_origin: Option<Member>,
    #[serde(default)]
    resume_sender_origin: Option<Member>,
    #[serde(default)]
    hud_current_controls: Option<Member>,
    #[serde(default)]
    countdown_scene_current: Option<Member>,
    #[serde(default)]
    countdown_origin: Option<Member>,
    #[serde(default)]
    health_origin: Option<Member>,
    #[serde(default)]
    health_scope: Option<Member>,
    #[serde(default)]
    animation_liquid_origin: Option<Member>,
    #[serde(default)]
    animation_liquid_scope: Option<Member>,
    #[serde(default)]
    interruption_origin: Option<Member>,
    #[serde(default)]
    late_movement: bool,
    #[serde(default)]
    post_palette: Option<PostPalette>,
    #[serde(default)]
    palette_edge: Option<Member>,
    #[serde(default)]
    palette_locations: Option<Member>,
    #[serde(default)]
    native_return: Option<NativeReturnSource>,

    #[serde(default)]
    inside_current_physical: Option<Member>,
    #[serde(default)]
    player_current: Option<Member>,
    #[serde(default)]
    initial_hdma_rom_backing: Option<Member>,
    #[serde(default)]
    player_continuation_current: Option<Member>,
    #[serde(default)]
    motion_reset_current: Option<Member>,
    #[serde(default)]
    selected_stage_predicates: Option<Member>,
    #[serde(default)]
    selected_stage_grouping: Option<Member>,
    #[serde(default)]
    drawing_current: Option<Member>,
    #[serde(default)]
    drawing_owner_requirements: Option<Member>,
    #[serde(default)]
    movement_down_current: Option<Member>,
    #[serde(default)]
    contact_current: Option<Member>,
    #[serde(default)]
    appearance_neighbors: Option<Member>,
    #[serde(default)]
    movement_source_admission: Option<Member>,
    initial: Vec<InitialContent>,
    callbacks: Member,
    endpoint: Member,
    #[serde(default)]
    final_owners: Option<Member>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PostPalette {
    #[serde(rename = "camera_current")]
    current: Member,
    #[serde(rename = "camera_room")]
    room: Member,
    #[serde(rename = "camera_envelope")]
    envelope: Member,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeReturnSource {
    wire: Member,
    s6: Member,
    idle_poll: Member,
    footstep: Member,
    hud_queue_scope: Member,
    drawing_hook: Option<Member>,
    s6_history: Option<Member>,
    final_debug_operand: Option<Member>,
}

impl Manifest {
    fn resolve(&mut self, root: &std::path::Path) {
        if let Some(native) = &mut self.native_return {
            native.wire.resolve(root);
            native.s6.resolve(root);
            native.idle_poll.resolve(root);
            native.footstep.resolve(root);
            native.hud_queue_scope.resolve(root);
            if let Some(hook) = &mut native.drawing_hook {
                hook.resolve(root);
            }
            if let Some(history) = &mut native.s6_history {
                history.resolve(root);
            }
            if let Some(debug) = &mut native.final_debug_operand {
                debug.resolve(root);
            }
        }
        if let Some(post_palette) = &mut self.post_palette {
            post_palette.current.resolve(root);
            post_palette.room.resolve(root);
            post_palette.envelope.resolve(root);
        }
        for member in [
            &mut self.sender_current_profile,
            &mut self.sender_bus_origin,
            &mut self.resume_sender_origin,
            &mut self.hud_current_controls,
            &mut self.countdown_scene_current,
            &mut self.countdown_origin,
            &mut self.health_origin,
            &mut self.health_scope,
            &mut self.animation_liquid_origin,
            &mut self.animation_liquid_scope,
            &mut self.interruption_origin,
            &mut self.palette_edge,
            &mut self.palette_locations,
            &mut self.inside_current_physical,
            &mut self.player_current,
            &mut self.initial_hdma_rom_backing,
            &mut self.player_continuation_current,
            &mut self.motion_reset_current,
            &mut self.selected_stage_predicates,
            &mut self.selected_stage_grouping,
            &mut self.drawing_current,
            &mut self.drawing_owner_requirements,
            &mut self.movement_down_current,
            &mut self.contact_current,
            &mut self.appearance_neighbors,
            &mut self.movement_source_admission,
        ]
        .into_iter()
        .flatten()
        {
            member.resolve(root);
        }
        for member in [
            &mut self.wram,
            &mut self.hardware,
            &mut self.pads,
            &mut self.apuram,
            &mut self.music_completion,
            &mut self.buffer_regions,
            &mut self.irq_current,
            &mut self.irq_recipe,
            &mut self.irq_terminal,
            &mut self.layer_blend,
            &mut self.random_requests,
            &mut self.random_import,
            &mut self.gameplay_completion,
            &mut self.actor_completion,
            &mut self.debug_completion,
            &mut self.palette_completion,
            &mut self.player_entry_current,
            &mut self.player_entry_labels,
            &mut self.player_entry_links,
            &mut self.callbacks,
            &mut self.endpoint,
        ] {
            member.resolve(root);
        }
        if let Some(member) = &mut self.final_owners {
            member.resolve(root);
        }
        for member in &mut self.profiles {
            member.resolve(root);
        }
        for initial in &mut self.initial {
            initial.member.resolve(root);
        }
    }
}

/// Admit one reviewed origin packet; expected members are only passed to the expected reader.
fn validate_countdown_scene_scope(case: &Case) -> Result<(), Outcome> {
    let mut declarations = case
        .borrowed
        .iter()
        .filter(|value| value.dependency == "frame.scene-countdown-scheduling");
    let value = declarations
        .next()
        .ok_or_else(|| data_failure(None, "explicit countdown scene borrowed scope"))?;
    if declarations.next().is_some()
        || value.provider != "independently-qualified-current-scene-scheduling-opportunity"
        || value.fields
            != [
                "initial-current-scene-choice",
                "actual-current-scene-choice",
                "current-opportunity-identity",
                "writer-and-freshness-join",
            ]
        || value.phase != "actual-timer-selection-and-later-ceres-revalidation"
        || value.claim
            != "unselected scene scheduling borrowed; full scene gameplay and M0 unproved"
    {
        return Err(data_failure(
            None,
            "explicit countdown scene borrowed scope",
        ));
    }
    Ok(())
}

fn validate_health_scope(case: &Case, text: &str) -> Result<(), Outcome> {
    let failure = || {
        data_failure(
            None,
            "explicit complete health origin and finite context borrowing",
        )
    };
    if Sha256::of(text.as_bytes())
        != Sha256::try_from("2a268517bb13aff2ae10520d2f29861dbdb3a69431bd638e9e425ed64f5e7895")
            .map_err(|_| failure())?
    {
        return Err(failure());
    }
    validate_countdown_scene_scope(case)?;
    let proposal: toml::Value = toml::from_str(text).map_err(|_| failure())?;
    let additions = proposal
        .get("addition")
        .and_then(toml::Value::as_array)
        .ok_or_else(failure)?;
    for addition in additions {
        let string = |key| {
            addition
                .get(key)
                .and_then(toml::Value::as_str)
                .ok_or_else(failure)
        };
        let dependency = string("dependency")?;
        let mut declarations = case
            .borrowed
            .iter()
            .filter(|value| value.dependency == dependency);
        let declared = declarations.next().ok_or_else(failure)?;
        let fields = addition
            .get("fields")
            .and_then(toml::Value::as_array)
            .ok_or_else(failure)?;
        if declarations.next().is_some()
            || declared.provider != string("provider")?
            || declared.phase != string("phase")?
            || declared.claim != string("claim")?
            || !declared
                .fields
                .iter()
                .map(String::as_str)
                .map(Some)
                .eq(fields.iter().map(toml::Value::as_str))
        {
            return Err(failure());
        }
    }
    for clarification in proposal
        .get("existing_declaration_clarification")
        .and_then(toml::Value::as_array)
        .ok_or_else(failure)?
    {
        let dependency = clarification
            .get("dependency")
            .and_then(toml::Value::as_str)
            .ok_or_else(failure)?;
        let declared = case
            .borrowed
            .iter()
            .find(|value| value.dependency == dependency)
            .ok_or_else(failure)?;
        let required = clarification
            .get("proposed_fields")
            .and_then(toml::Value::as_array)
            .ok_or_else(failure)?;
        if required
            .iter()
            .filter_map(toml::Value::as_str)
            .any(|field| !declared.fields.iter().any(|value| value == field))
        {
            return Err(failure());
        }
    }
    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct InterruptionOrigin {
    format: String,
    origin_sha256: String,
    timer_offset: usize,
    knockback_direction_offset: usize,
}

fn interruption_origin(wram: &[u8], source: &str) -> Option<frame::interruption_caller::Admission> {
    if Sha256::of(source.as_bytes())
        != Sha256::try_from("e3bc30449798bfc8f51e7da0c3de622168829bee5e1a8e820cf71bbdff2240c4")
            .ok()?
    {
        return None;
    }
    let source: InterruptionOrigin = toml::from_str(source).ok()?;
    let origin = Sha256::try_from(source.origin_sha256.as_str()).ok()?;
    let inactive = |offset: usize| wram.get(offset..offset.saturating_add(2)) == Some(&[0, 0]);
    if source.format != "original-inactive-interruption-v1"
        || wram.len() != 131_072
        || Sha256::of(wram) != origin
        || !inactive(source.timer_offset)
        || !inactive(source.knockback_direction_offset)
    {
        return None;
    }
    Some(frame::interruption_caller::Admission::inactive(origin.0))
}

fn consecutive_pre_wait(
    templates: &profiles::Templates,
    owners: &[&str],
) -> Result<data::timing::Template, Outcome> {
    let fault = || data_failure(None, "adjacent PreWait semantic operations");
    let profile = templates
        .profiles
        .iter()
        .find(|profile| profile.name == profiles::ProfileName::PreWait)
        .ok_or_else(fault)?;
    let mut span = None;
    for owner in owners {
        let mut matches = templates
            .portions
            .iter()
            .filter(|part| part.profile == profile.name && part.receipt_contract == *owner);
        let part = matches
            .next()
            .filter(|_| matches.next().is_none())
            .ok_or_else(fault)?;
        span = match span {
            None => Some((part.first_exposure, part.end_exposure)),
            Some((first, end)) if end == part.first_exposure => Some((first, part.end_exposure)),
            Some(_) => return Err(fault()),
        };
    }
    let (first, end) = span.ok_or_else(fault)?;
    profile
        .slice(first, end)
        .and_then(|slice| slice.template())
        .map_err(|_| fault())
}

fn selected_portion<'a>(
    templates: &'a profiles::Templates,
    contract: &str,
) -> Result<&'a profiles::Portion, Outcome> {
    let mut portions = templates
        .portions
        .iter()
        .filter(|portion| portion.receipt_contract == contract);
    portions
        .next()
        .filter(|_| portions.next().is_none())
        .ok_or_else(|| data_failure(None, "frame unique semantic portion"))
}

fn portion(
    templates: &profiles::Templates,
    contract: &str,
) -> Result<data::timing::Template, Outcome> {
    let portion = selected_portion(templates, contract)?;
    let profile = templates
        .profiles
        .iter()
        .find(|profile| profile.name == portion.profile)
        .ok_or_else(|| data_failure(None, "frame semantic physical profile"))?;
    profile
        .slice(portion.first_exposure, portion.end_exposure)
        .and_then(|profile| profile.template())
        .map_err(|_| data_failure(None, "frame whole semantic physical template"))
}

fn sprite_history_profile(
    templates: &profiles::Templates,
) -> Result<data::timing::Profile, Outcome> {
    use data::timing::{Cycle, ExposureKind, TemplateEffect};

    let fault = || data_failure(None, "sprite readiness and final history adjacency");
    let finalization = selected_portion(templates, "Sprite-list finalization")?;
    let ready = selected_portion(templates, "SpriteDrawListReady")?;
    let history = selected_portion(templates, "Debug/history completion")?;
    let mut gates = templates
        .materializations
        .iter()
        .filter(|gate| gate.name == "SpriteDrawList");
    let gate = gates
        .next()
        .filter(|_| gates.next().is_none())
        .ok_or_else(fault)?;
    if gate.opportunity != ready.receipt_contract
        || gate.profile != Some(ready.profile)
        || gate.after_exposure != Some(ready.first_exposure)
        || finalization.profile != ready.profile
        || finalization.end_exposure != ready.first_exposure
        || ready.profile != history.profile
        || ready.end_exposure != history.first_exposure
    {
        return Err(fault());
    }

    // The builder is already published at the readiness gate. The caller must
    // still charge the actual following read before executing final history.
    let mut quiet = portion(templates, &ready.receipt_contract)?;
    if !matches!(quiet.runs.as_slice(), [read]
        if read.count == 1 && read.cycle == Cycle::Eight && read.kind == ExposureKind::Read
            && read.poll_before && read.effect == TemplateEffect::None)
    {
        return Err(fault());
    }
    quiet
        .runs
        .extend(portion(templates, &history.receipt_contract)?.runs);
    quiet.bind(&[]).map_err(|_| fault())
}

fn validate_haze_ready(templates: &profiles::Templates) -> Result<(), Outcome> {
    let preparation = templates
        .portions
        .iter()
        .find(|portion| portion.receipt_contract == "Haze")
        .ok_or_else(|| data_failure(None, "frame Haze preparation contract"))?;
    let mut gates = templates
        .materializations
        .iter()
        .filter(|gate| gate.name == "HazeBackdropBands");
    let gate = gates
        .next()
        .filter(|_| gates.next().is_none())
        .ok_or_else(|| data_failure(None, "frame unique Haze Ready gate"))?;
    if gate.initial_seed
        || gate.length != 16
        || gate.opportunity != "HazeBackdropBandsReady"
        || gate.profile != Some(preparation.profile)
        || gate.after_exposure != Some(preparation.end_exposure)
    {
        return Err(data_failure(None, "frame Haze Ready physical boundary"));
    }
    Ok(())
}

fn buffer_address(text: &str, name: &str, length: usize) -> Result<data::dma::BusAddress, Outcome> {
    #[derive(Deserialize)]
    struct Regions {
        format: u16,
        region: Vec<Region>,
    }
    #[derive(Deserialize)]
    struct Region {
        name: String,
        source_wram_offset: u16,
        length: usize,
    }
    let regions: Regions =
        toml::from_str(text).map_err(|_| data_failure(None, "frame buffer region edge"))?;
    let mut selected = regions.region.iter().filter(|region| region.name == name);
    let region = selected
        .next()
        .filter(|region| region.length == length && selected.next().is_none())
        .ok_or_else(|| data_failure(None, "frame unique buffer region"))?;
    if regions.format != 1 {
        return Err(data_failure(None, "frame buffer region format"));
    }
    Ok(data::dma::BusAddress {
        bank: 0x7e,
        offset: region.source_wram_offset,
    })
}

pub(super) struct PreparationInputs {
    scene: frame::borrowed_gameplay::SceneController,
    gameplay: frame::borrowed_gameplay::GameplayDispatchOffer,
    actors: frame::borrowed_preparation::ActorEligibilityReceipt,
    debug_current: frame::borrowed_preparation::DebugOrigin,
    debug: frame::borrowed_preparation::DebugCompletionReceipt,
    palette_current: frame::borrowed_preparation::PaletteOrigin,
    palette: frame::borrowed_preparation::PaletteCompletionReceipt,
}

fn preparation_inputs(wram: &[u8], manifest: &Manifest) -> Result<PreparationInputs, Outcome> {
    let origin = prelude_seed::decode(wram)
        .map_err(|_| data_failure(None, "frame preparation current origins"))?;

    let gameplay_text = manifest.gameplay_completion.text()?;
    let scene = gameplay_receipt::current(wram, &gameplay_text)
        .map_err(|_| data_failure(None, "frame independent current scene controller"))?;
    let gameplay = gameplay_receipt::offer(wram, &gameplay_text)
        .map_err(|_| data_failure(None, "frame current gameplay route offer"))?;
    let actors =
        preparation_receipts::actors(&origin.actors, &manifest.actor_completion.text()?)
            .map_err(|_| data_failure(None, "frame complete ordered actor membership offer"))?;
    let debug_current = preparation_receipts::debug_origin(wram)
        .map_err(|_| data_failure(None, "frame current physical debug origin"))?;
    let debug = preparation_receipts::debug(debug_current, &manifest.debug_completion.text()?)
        .map_err(|_| data_failure(None, "frame current debug completion offer"))?;
    let palette_current = preparation_receipts::palette_origin(wram)
        .map_err(|_| data_failure(None, "frame current palette origin"))?;
    let palette =
        preparation_receipts::palette(palette_current, &manifest.palette_completion.text()?)
            .map_err(|_| data_failure(None, "frame current palette completion offer"))?;
    Ok(PreparationInputs {
        scene,
        gameplay,
        actors,
        debug_current,
        debug,
        palette_current,
        palette,
    })
}

#[cfg(test)]
mod tests;

fn live_haze(
    wram: &[u8],
    origin: &frame::borrowed::HazeOrigin,
) -> Result<frame::haze::Haze, Outcome> {
    use frame::borrowed::{ColourComponent, HazeSlot};
    use types::haze::{
        HAZE_BAND_COUNT, HazeBands, HazeColour, HazeFade, HazeLevel, HazePeak, HazeState,
    };

    const BANDS_START: usize = 0x9d00;
    let fault = || data_failure(None, "frame current Haze state and transport");
    let Some(HazeSlot::BackdropRamp {
        component, maximum, ..
    }) = origin.slots.first()
    else {
        return Err(fault());
    };
    let (colour, encoding) = match component {
        ColourComponent::Red => (HazeColour::Red, 0x20),
        ColourComponent::Green => (HazeColour::Green, 0x40),
        ColourComponent::Blue => (HazeColour::Blue, 0x80),
    };
    let bands = wram
        .get(BANDS_START..BANDS_START.wrapping_add(HAZE_BAND_COUNT))
        .ok_or_else(fault)?
        .iter()
        .map(|byte| {
            if byte & 0xe0 != encoding {
                return Err(fault());
            }
            HazeLevel::new(byte & 0x1f).ok_or_else(fault)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let peak = u8::try_from(*maximum)
        .ok()
        .and_then(HazePeak::new)
        .ok_or_else(fault)?;
    let state = HazeState {
        colour,
        fade: HazeFade::Brightening(peak),
        bands: HazeBands(bands.try_into().map_err(|_| fault())?),
    };
    frame::haze::Haze::from_origin(state, origin).map_err(|_| fault())
}

/// Independently decoded initial physical owners; unavailable owners are not defaulted.
pub(super) struct Origin<'a> {
    pub wram: &'a [u8],
    pub timing: game::production::timing::State,
    pub senders: frame::Senders,
    pub content: frame::TransferContent,
    pub music: Option<frame::borrowed_receipts::MusicCompletionReceipt>,
    pub drawing: Option<frame::interrupts::DrawingState>,
    pub random: frame::random::RandomSeed,
    pub preparation: PreparationInputs,
}

/// Decode selected current quantities once; later observations supply no replacement seeds.
pub(super) fn decode(
    origin: Origin<'_>,
    content: &movement::Content,
) -> Result<frame::State, Outcome> {
    let movement = movement::seed(origin.wram)?;
    let appearance = appearance_seed::decode(origin.wram, &content.clips)
        .map_err(|_| data_failure(None, "frame current appearance origin"))?;
    let prelude = prelude_seed::decode(origin.wram)
        .map_err(|_| data_failure(None, "frame current prelude origin"))?;
    let receipt = borrowed_receipts::publication(origin.wram)
        .map_err(|_| data_failure(None, "frame current publication owner"))?;
    let state = frame::State {
        music: Some(
            borrowed_receipts::music(origin.wram)
                .map_err(|_| data_failure(None, "frame current music owner"))?,
        ),
        haze: Some(live_haze(origin.wram, &prelude.haze)?),
        random: Some(frame::random::Sequence::new(origin.random)),
        scene: Some(origin.preparation.scene),
        actor_eligibility: Some(prelude.actors),
        debug: Some(origin.preparation.debug_current),
        palette_objects: Some(origin.preparation.palette_current),
        entry_environment: None,
        sprite_builder: Some(prelude.builder),
        sender_current: None,
        sender_queues: None,
        drawing: origin.drawing,
        caller: frame::Caller::AwaitingAcknowledgment {
            prelude: frame::PreludeSource::Origin(Box::new(frame::Prelude {
                player_entry_current: None,
                player_entry_offers: None,
                player_continuation_current: None,
                player_continuation_offers: None,

                music_completion: origin.music,
                gameplay_completion: Some(origin.preparation.gameplay),
                actor_completion: Some(origin.preparation.actors),
                debug_completion: Some(origin.preparation.debug),
                palette_completion: Some(origin.preparation.palette),
            })),
            receipt: Some(receipt),
        },
        timing: origin.timing,
        movement: frame::movement::Movement::new(movement),
        movement_stage: frame::movement_stage::Stage::default(),
        inside_world: None,
        inside_inputs: None,
        inside_premises: None,
        inside_pending: None,
        inside_completed: None,
        camera: Model(super::camera::seed(origin.wram, &super::camera::Content)?),
        graphical_contacts: frame::rendering_stage::Contacts::default(),
        rendering: frame::rendering_stage::Stage::default(),
        drawing_origins: None,
        contact_current: None,
        sound_offer_current: None,
        movement_inputs: None,
        world: None,
        prepared_action: None,
        appearance: appearance::Work::with_cannon(
            samus_appearance::SuspendedAppearance::new(appearance.state, None, Vec::new()),
            appearance.cannon,
        ),
        playing_figure: Model(appearance.playing),
        hud: frame::hud::Hud::new(super::hud::decode_state(origin.wram)?),
        hud_display_current: None,
        hud_inventory: None,
        hud_frame_count: None,
        hud_map: frame::hud_map_caller::Stage::Pending,
        health: None,
        liquid_history: None,
        animation: frame::animation_caller::Stage::Pending,
        maintenance: frame::maintenance::Stage::default(),
        samus_palette: None,
        s6_camera: frame::s6_camera::Stage::default(),
        s6_drawing: frame::s6_drawing::Stage::default(),
        background: frame::suffix_caller::BackgroundScroll::Pending,
        return_progress: frame::frame_return::Stage::Pending,
        sender_progress: frame::sender_caller::Cursor::New,
        sender_history: None,
        late_hud: frame::late_hud::Stage::Pending,
        frame_history: None,
        recurring_player: None,
        pose_input_configuration: None,
        room_tail_origin: None,
        room_tail: frame::room_tail::Stage::Pending,
        play_time: frame::play_time::Stage::default(),
        room_quake: frame::room_quake::Stage::default(),
        frame_tail_current: None,
        frame_tail: frame::frame_tail::Stage::Pending,
        interruption: frame::interruption_caller::Stage::Pending,
        late_movement: frame::late_movement::Stage::Pending,
        countdown: frame::countdown_caller::Stage::Pending,
        countdown_delivered: Vec::new(),
        scene_countdown: None,
        senders: origin.senders,
        sender_stage: frame::sender_stage::Stage::default(),
        sound_offers: frame::sender_stage::Offers::default(),
        content: origin.content,
        publications: frame::Publications::default(),
    };
    state
        .validate()
        .map_err(|_| data_failure(None, "frame complete origin owners"))?;
    Ok(state)
}
