//! Immutable physical templates and semantic navigation at the verification edge.
//! Publication values come from current owners; this module executes no work.
use data::timing::{Cycle, ExposureKind, PublicationPort, Sample};
use serde::Deserialize;
use std::collections::BTreeMap;

/// Failure to decode a qualified physical template or its complete bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileFault {
    /// Malformed input text.
    Format,
    /// Missing or unsupported profile, device or binding namespace.
    Unsupported,
    /// Invalid profile shape or physical cycle.
    Profile,
    /// Incomplete, overlapping or inconsistent exposure coverage.
    Coverage,
    /// Missing, duplicate or incompatible producer binding.
    Binding,
}
/// Physical profile identity; semantic selection belongs to actual owners.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProfileName {
    /// Work before the hardware wait.
    PreWait,
    /// Current HUD drawing begin handler.
    HudDrawingBegin,
    /// Current HUD drawing end handler.
    HudDrawingEnd,
    /// Prepared video work.
    NmiVideo,
    /// Hardware-owned wait cadence.
    WaitTurn,
    /// Hardware-owned native entry.
    NativeEntry,
}
/// Semantic arithmetic dependency, never an expected product.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArithmeticTask {
    /// Actual semantic task name.
    pub name: String,
    /// Named current quantities required before its publications.
    pub quantities: Vec<String>,
}
/// Current owner that supplies a byte at the publication opportunity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Publication {
    /// Qualified CPU-space device port.
    pub port: PublicationPort,
    /// Physical device name.
    pub device: String,
    /// Actual current producer identity.
    pub producer: String,
    /// Semantic operand contract, where applicable.
    pub arithmetic: Option<ArithmeticTask>,
}
/// Physical effect template with no unresolved byte replaced by a default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// No named physical effect.
    None,
    /// Actual hardware result with optional semantic arithmetic consumer.
    Sample(Sample, Option<ArithmeticTask>),
    /// Current producer must supply the publication byte at use.
    Publication(Publication),
}
/// Repeated equivalent physical opportunities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    /// Number of equivalent opportunities.
    pub count: u32,
    /// Qualified cycle class.
    pub cycle: Cycle,
    /// Physical charging and arithmetic ordering.
    pub kind: ExposureKind,
    /// Interrupt sample before this opportunity.
    pub poll_before: bool,
    /// Unresolved physical effect.
    pub effect: Effect,
}
/// Complete immutable physical template.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    /// Profile identity retained through slicing.
    pub name: ProfileName,
    /// Equivalent physical opportunities in order.
    pub runs: Vec<Run>,
    /// Total completed-exposure coordinate extent.
    pub exposures: u64,
}
/// Semantic consumer of one actual physical sample, in physical sample order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SampleBinding {
    /// Physical result device.
    pub device: Sample,
    /// Semantic arithmetic contract that consumes this result.
    pub arithmetic: Option<ArithmeticTask>,
}
/// Resolved immutable physical work and its retained semantic sample consumers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundProfile {
    /// Physical identity retained through stable slicing and binding.
    pub name: ProfileName,
    /// Physical projection containing actual current producer publications.
    pub profile: data::timing::Profile,
    /// Semantic consumers in actual physical sample order, without dispatch coordinates.
    pub samples: Vec<SampleBinding>,
}
/// Semantic navigation and dependency metadata, never a source dispatcher.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Portion {
    /// Physical template identity.
    pub profile: ProfileName,
    /// Inclusive starting completed-exposure coordinate.
    pub first_exposure: u64,
    /// Exclusive ending completed-exposure coordinate.
    pub end_exposure: u64,
    /// Actual semantic owner name.
    pub semantic_owner: String,
    /// Required full predicate contract, not a family-only admission.
    pub receipt_contract: String,
    /// Producer readiness names required at their causal opportunities.
    pub completed_producer_receipts: Vec<String>,
    intrinsic_ticks: u64,
    first_run: usize,
    first_repetition: u32,
    end_run: usize,
    end_repetition: u32,
    end_is_exclusive: bool,
    publication_inputs: Vec<u32>,
    hardware_result_exposures: Vec<u64>,
}
/// Independently qualified content readiness boundary.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Materialization {
    /// Named content owner.
    pub name: String,
    /// Required producer Ready receipt.
    pub opportunity: String,
    /// Initial content is admitted once, independently of future producers.
    pub initial_seed: bool,
    /// Physical template for a completed producer trigger.
    pub profile: Option<ProfileName>,
    /// Frozen completed-count trigger, not the later snapshot cutoff ordinal.
    pub after_exposure: Option<u64>,
    /// Imported content length; no bytes are present in the template.
    pub length: usize,
}
/// Complete decoded templates with exposure coverage and bindings validated.
#[derive(Clone, Debug)]
pub struct Templates {
    /// All six physical profiles; hardware owns native entry and wait.
    pub profiles: Vec<Profile>,
    /// Complete half-open semantic portions for the four semantic profiles.
    pub portions: Vec<Portion>,
    /// Initial or actual producer Ready content contracts.
    pub materializations: Vec<Materialization>,
}
#[derive(Deserialize)]
struct ProfileFile {
    format: u32,
    profile: Vec<RawProfile>,
}
#[derive(Deserialize)]
struct RawProfile {
    name: ProfileName,
    intrinsic_ticks: u64,
    exposures: u64,
    run: Vec<RawRun>,
}
/// One physical source run, shared by named frame profiles and supplemental apertures.
#[derive(Deserialize)]
pub struct RawRun {
    cycle_ticks: u8,
    chunks: Vec<u8>,
    alu_side: String,
    /// Sample eligible interrupt transitions before this run.
    pub poll_before: bool,
    pending_sensitive: bool,
    /// Optional device effect name.
    pub device: String,
    /// Optional device effect placement.
    pub event_phase: String,
    /// Repeated equivalent cycles.
    pub count: u32,
}
#[derive(Deserialize)]
struct PortionFile {
    format: u32,
    portion: Vec<Portion>,
}
#[derive(Deserialize)]
struct PublicationFile {
    format: u32,
    publication: Vec<RawPublication>,
}
#[derive(Deserialize, Clone)]
struct RawPublication {
    profile: ProfileName,
    exposure: u64,
    device: String,
    producer: String,
    ordinal: u32,
    task_ordinal: Option<u32>,
    intrinsic_milestone: u64,
    role: String,
    cursor_stage: String,
}
#[derive(Deserialize)]
struct SampleFile {
    format: u32,
    sample: Vec<RawSample>,
}
#[derive(Deserialize)]
struct RawSample {
    profile: ProfileName,
    exposure: u64,
    device: String,
    task_ordinal: Option<u32>,
    intrinsic_sample_boundary: u64,
    role: String,
    cursor_stage: String,
}
#[derive(Deserialize)]
struct ArithmeticFile {
    format: u32,
    task: Vec<RawTask>,
}
#[derive(Deserialize)]
struct RawTask {
    ordinal: u32,
    producer: String,
    quantity_bindings: Vec<String>,
    actual_owner_receipt_required: bool,
    fixture_quantity_values_exported: bool,
}
#[derive(Deserialize)]
struct MaterializationFile {
    format: u32,
    buffer: Vec<Materialization>,
}

fn parse<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, ProfileFault> {
    toml::from_str(text).map_err(|_| ProfileFault::Format)
}
fn arithmetic(
    tasks: &BTreeMap<u32, ArithmeticTask>,
    ordinal: Option<u32>,
) -> Result<Option<ArithmeticTask>, ProfileFault> {
    ordinal
        .map(|key| tasks.get(&key).cloned().ok_or(ProfileFault::Binding))
        .transpose()
}
fn sample(device: &str) -> Result<Sample, ProfileFault> {
    match device {
        "ProductLow" => Ok(Sample::ProductLow),
        "ProductHigh" => Ok(Sample::ProductHigh),
        "Joypad0Low" => Ok(Sample::ControllerLow),
        "Joypad0High" => Ok(Sample::ControllerHigh),
        "AcknowledgeIrq" => Ok(Sample::AcknowledgeIrq),
        "AcknowledgeNmi" => Ok(Sample::AcknowledgeNmi),
        "BeamBlankingStatus" => Ok(Sample::Blanking),
        _ => Err(ProfileFault::Unsupported),
    }
}
/// Resolve a source run's physical cycle and access form.
/// # Errors
/// Rejects unsupported phase splits and zero-duration runs.
pub fn cycle(raw: &RawRun) -> Result<(Cycle, ExposureKind), ProfileFault> {
    let cycle = match raw.cycle_ticks {
        6 => Cycle::Six,
        8 => Cycle::Eight,
        12 => Cycle::Twelve,
        _ => return Err(ProfileFault::Profile),
    };
    let kind = match (
        raw.alu_side.as_str(),
        raw.pending_sensitive,
        raw.chunks.as_slice(),
    ) {
        ("After", false, [first, 4]) if *first == raw.cycle_ticks.wrapping_sub(4) => {
            ExposureKind::Read
        }
        ("Before", false, [whole]) if *whole == raw.cycle_ticks => ExposureKind::Write,
        ("After", false, [6]) if cycle == Cycle::Six => ExposureKind::Internal,
        ("After", true, [6]) if cycle == Cycle::Six => ExposureKind::PendingInternal,
        _ => return Err(ProfileFault::Profile),
    };
    if raw.count == 0 {
        return Err(ProfileFault::Profile);
    }
    Ok((cycle, kind))
}

const fn template_fault(fault: data::timing::TemplateFault) -> ProfileFault {
    match fault {
        data::timing::TemplateFault::InvalidTemplate => ProfileFault::Profile,
        data::timing::TemplateFault::Publication => ProfileFault::Binding,
    }
}
impl Run {
    const fn physical(&self) -> data::timing::TemplateRun {
        let effect = match &self.effect {
            Effect::None => data::timing::TemplateEffect::None,
            Effect::Sample(device, _) => data::timing::TemplateEffect::Sample(*device),
            Effect::Publication(template) => data::timing::TemplateEffect::Publish(template.port),
        };
        data::timing::TemplateRun {
            count: self.count,
            cycle: self.cycle,
            kind: self.kind,
            poll_before: self.poll_before,
            effect,
        }
    }
}
impl Profile {
    /// Export the byte-free canonical physical template, retaining metadata at this edge.
    /// # Errors
    /// Rejects invalid physical shape or inconsistent complete exposure extent.
    pub fn template(&self) -> Result<data::timing::Template, ProfileFault> {
        let exposures = self.runs.iter().try_fold(0_u64, |count, run| {
            count
                .checked_add(u64::from(run.count))
                .ok_or(ProfileFault::Profile)
        })?;
        if exposures != self.exposures {
            return Err(ProfileFault::Profile);
        }
        let template = data::timing::Template {
            runs: self.runs.iter().map(Run::physical).collect(),
        };
        template.validate().map_err(template_fault)?;
        Ok(template)
    }
    /// Resolve a stable slice using already produced and admitted current owner writes.
    ///
    /// The data template owns the single physical binding implementation. Metadata
    /// here retains semantic sample consumers in physical order without dispatch coordinates.
    /// No owner transition, callback, clock or scheduling operation runs here.
    /// # Errors
    /// Rejects missing/extra writes, mismatched ports/configurations or invalid templates.
    pub fn bind(
        &self,
        publications: &[data::timing::DeviceWrite],
    ) -> Result<BoundProfile, ProfileFault> {
        let profile = self
            .template()?
            .bind(publications)
            .map_err(template_fault)?;
        let mut samples = Vec::new();
        for run in &self.runs {
            if let Effect::Sample(device, arithmetic) = &run.effect {
                for _ in 0..run.count {
                    samples.push(SampleBinding {
                        device: *device,
                        arithmetic: arithmetic.clone(),
                    });
                }
            }
        }
        Ok(BoundProfile {
            name: self.name,
            profile,
            samples,
        })
    }
    /// Slice exact half-open completed-exposure positions, preserving repetitions.
    /// # Errors
    /// Returns Coverage for an empty, inverted or out-of-bounds interval.
    pub fn slice(&self, first: u64, end: u64) -> Result<Self, ProfileFault> {
        if first >= end || end > self.exposures {
            return Err(ProfileFault::Coverage);
        }
        let mut at = 0_u64;
        let mut runs = Vec::new();
        for run in &self.runs {
            let next = at
                .checked_add(u64::from(run.count))
                .ok_or(ProfileFault::Coverage)?;
            let start = at.max(first);
            let stop = next.min(end);
            if start < stop {
                let mut sliced = run.clone();
                sliced.count =
                    u32::try_from(stop.wrapping_sub(start)).map_err(|_| ProfileFault::Coverage)?;
                runs.push(sliced);
            }
            at = next;
        }
        if at != self.exposures {
            return Err(ProfileFault::Coverage);
        }
        Ok(Self {
            name: self.name,
            runs,
            exposures: end.wrapping_sub(first),
        })
    }
}
const VIDEO: [&str; 52] = [
    "Display",
    "ObjectLayout",
    "ObjectAddressLow",
    "ObjectAddressHigh",
    "ObjectData",
    "BackgroundMode",
    "Mosaic",
    "BackgroundOneMap",
    "BackgroundTwoMap",
    "BackgroundThreeMap",
    "BackgroundFourMap",
    "BackgroundCharacterBasesOneTwo",
    "BackgroundCharacterBasesThreeFour",
    "BackgroundOneHorizontal",
    "BackgroundOneVertical",
    "BackgroundTwoHorizontal",
    "BackgroundTwoVertical",
    "BackgroundThreeHorizontal",
    "BackgroundThreeVertical",
    "BackgroundFourHorizontal",
    "BackgroundFourVertical",
    "VramLayout",
    "VramAddressLow",
    "VramAddressHigh",
    "VramDataLow",
    "VramDataHigh",
    "ModeSevenSettings",
    "ModeSevenA",
    "ModeSevenB",
    "ModeSevenC",
    "ModeSevenD",
    "ModeSevenX",
    "ModeSevenY",
    "PaletteAddress",
    "PaletteData",
    "BackgroundWindowOneTwo",
    "BackgroundWindowThreeFour",
    "ObjectColorWindow",
    "WindowOneLeft",
    "WindowOneRight",
    "WindowTwoLeft",
    "WindowTwoRight",
    "BackgroundWindowCombination",
    "ObjectColorWindowCombination",
    "MainLayers",
    "SubLayers",
    "MainWindowLayers",
    "SubWindowLayers",
    "ColorWindow",
    "ColorMath",
    "BackdropComponent",
    "ScreenSettings",
];
fn publication_port(device: &str) -> Result<PublicationPort, ProfileFault> {
    if let Some(index) = VIDEO.iter().position(|name| *name == device) {
        return Ok(PublicationPort(0x2100_u16.wrapping_add(
            u16::try_from(index).map_err(|_| ProfileFault::Unsupported)?,
        )));
    }
    if let Some(suffix) = device.strip_prefix("DmaChannel") {
        let slot = suffix
            .bytes()
            .next()
            .filter(|slot| (b'0'..=b'7').contains(slot))
            .ok_or(ProfileFault::Unsupported)?;
        let field = suffix.get(1..).ok_or(ProfileFault::Unsupported)?;
        let offset = match field {
            "Control" => 0,
            "Target" => 1,
            "SourceLow" => 2,
            "SourceHigh" => 3,
            "SourceBank" => 4,
            "SizeLow" => 5,
            "SizeHigh" => 6,
            _ => return Err(ProfileFault::Unsupported),
        };
        return Ok(PublicationPort(
            0x4300_u16
                .wrapping_add(u16::from(slot.wrapping_sub(b'0')).wrapping_mul(16))
                .wrapping_add(offset),
        ));
    }
    let port = match device {
        "LibraryThreeCue" => 0x2143,
        "InterruptEnable" => 0x4200,
        "Multiplicand" => 0x4202,
        "Multiplier" => 0x4203,
        "DividendLow" => 0x4204,
        "IrqHorizontalLow" => 0x4207,
        "IrqHorizontalHigh" => 0x4208,
        "IrqVerticalLow" => 0x4209,
        "IrqVerticalHigh" => 0x420a,
        "DmaEnable" => 0x420b,
        "HdmaEnable" => 0x420c,
        _ => return Err(ProfileFault::Unsupported),
    };
    Ok(PublicationPort(port))
}

struct Bindings {
    tasks: BTreeMap<u32, ArithmeticTask>,
    publications: BTreeMap<(ProfileName, u64), RawPublication>,
    samples: BTreeMap<(ProfileName, u64), RawSample>,
}
impl Bindings {
    fn new(
        arithmetic: ArithmeticFile,
        publications: PublicationFile,
        samples: SampleFile,
    ) -> Result<Self, ProfileFault> {
        let mut bindings = Self {
            tasks: BTreeMap::new(),
            publications: BTreeMap::new(),
            samples: BTreeMap::new(),
        };
        for task in arithmetic.task {
            if !task.actual_owner_receipt_required
                || task.fixture_quantity_values_exported
                || task.producer.is_empty()
                || task.quantity_bindings.is_empty()
                || bindings
                    .tasks
                    .insert(
                        task.ordinal,
                        ArithmeticTask {
                            name: task.producer,
                            quantities: task.quantity_bindings,
                        },
                    )
                    .is_some()
            {
                return Err(ProfileFault::Binding);
            }
        }
        for publication in publications.publication {
            if bindings
                .publications
                .insert((publication.profile, publication.exposure), publication)
                .is_some()
            {
                return Err(ProfileFault::Binding);
            }
        }
        for sample in samples.sample {
            if bindings
                .samples
                .insert((sample.profile, sample.exposure), sample)
                .is_some()
            {
                return Err(ProfileFault::Binding);
            }
        }
        Ok(bindings)
    }
    fn effect(
        &mut self,
        profile: ProfileName,
        at: u64,
        intrinsic: u64,
        raw: &RawRun,
        kind: ExposureKind,
    ) -> Result<Effect, ProfileFault> {
        let key = (profile, at);
        match (raw.event_phase.as_str(), raw.device.as_str()) {
            ("", "") => Ok(Effect::None),
            ("publication", device) if kind == ExposureKind::Write => {
                let binding = self
                    .publications
                    .remove(&key)
                    .ok_or(ProfileFault::Binding)?;
                if binding.device != device
                    || binding.intrinsic_milestone != intrinsic
                    || binding.role
                        != "actual_current_producer_input; never expected callback bytes"
                    || binding.cursor_stage
                        != "publication after all exposure chunks; before completion"
                {
                    return Err(ProfileFault::Binding);
                }
                let valid = match binding.producer.as_str() {
                    "FrameMathTasks" => binding.task_ordinal.is_some(),
                    "ActualHudDrawingOwner"
                    | "FootstepsQueue"
                    | "PreparedVideoSettings"
                    | "VideoControl"
                    | "VideoUploadRequests" => binding.task_ordinal.is_none(),
                    _ => false,
                };
                if !valid {
                    return Err(ProfileFault::Unsupported);
                }
                Ok(Effect::Publication(Publication {
                    port: publication_port(device)?,
                    device: device.to_owned(),
                    producer: binding.producer,
                    arithmetic: arithmetic(&self.tasks, binding.task_ordinal)?,
                }))
            }
            ("sample", device) if kind == ExposureKind::Read => {
                let binding = self.samples.remove(&key).ok_or(ProfileFault::Binding)?;
                if binding.device != device
                    || binding.intrinsic_sample_boundary != intrinsic.wrapping_sub(4)
                    || binding.role != "actual_hardware_result_not_expected_input"
                    || binding.cursor_stage != "sample after first exposure chunk; result only"
                {
                    return Err(ProfileFault::Binding);
                }
                Ok(Effect::Sample(
                    sample(device)?,
                    arithmetic(&self.tasks, binding.task_ordinal)?,
                ))
            }
            _ => Err(ProfileFault::Unsupported),
        }
    }
    fn profile(&mut self, raw: &RawProfile) -> Result<Profile, ProfileFault> {
        let mut runs = Vec::new();
        let mut at = 0_u64;
        let mut intrinsic = 0_u64;
        for run in &raw.run {
            let (cycle, kind) = cycle(run)?;
            if run.event_phase.is_empty() && run.device.is_empty() {
                intrinsic = intrinsic
                    .checked_add(
                        u64::from(run.count)
                            .checked_mul(u64::from(cycle.ticks()))
                            .ok_or(ProfileFault::Profile)?,
                    )
                    .ok_or(ProfileFault::Profile)?;
                runs.push(Run {
                    count: run.count,
                    cycle,
                    kind,
                    poll_before: run.poll_before,
                    effect: Effect::None,
                });
                at = at
                    .checked_add(u64::from(run.count))
                    .ok_or(ProfileFault::Profile)?;
            } else {
                for _ in 0..run.count {
                    intrinsic = intrinsic
                        .checked_add(u64::from(cycle.ticks()))
                        .ok_or(ProfileFault::Profile)?;
                    let effect = self.effect(raw.name, at, intrinsic, run, kind)?;
                    runs.push(Run {
                        count: 1,
                        cycle,
                        kind,
                        poll_before: run.poll_before,
                        effect,
                    });
                    at = at.checked_add(1).ok_or(ProfileFault::Profile)?;
                }
            }
        }
        if at != raw.exposures || intrinsic != raw.intrinsic_ticks {
            return Err(ProfileFault::Profile);
        }
        Ok(Profile {
            name: raw.name,
            runs,
            exposures: at,
        })
    }
}
fn coordinate(raw: &RawProfile, run: usize, repetition: u32) -> Result<u64, ProfileFault> {
    if run > raw.run.len() || (run == raw.run.len() && repetition != 0) {
        return Err(ProfileFault::Coverage);
    }
    if let Some(current) = raw.run.get(run)
        && repetition >= current.count
    {
        return Err(ProfileFault::Coverage);
    }
    raw.run
        .iter()
        .take(run)
        .try_fold(u64::from(repetition), |at, row| {
            at.checked_add(u64::from(row.count))
                .ok_or(ProfileFault::Coverage)
        })
}
fn coverage(
    raw: &RawProfile,
    profile: &Profile,
    portions: &[Portion],
    publications: &BTreeMap<(ProfileName, u64), RawPublication>,
) -> Result<(), ProfileFault> {
    if matches!(raw.name, ProfileName::NativeEntry | ProfileName::WaitTurn) {
        return if portions.iter().any(|part| part.profile == raw.name) {
            Err(ProfileFault::Coverage)
        } else {
            Ok(())
        };
    }
    let mut at = 0_u64;
    for part in portions.iter().filter(|part| part.profile == raw.name) {
        if part.first_exposure != at
            || !part.end_is_exclusive
            || part.semantic_owner.is_empty()
            || part.receipt_contract.is_empty()
            || coordinate(raw, part.first_run, part.first_repetition)? != part.first_exposure
            || coordinate(raw, part.end_run, part.end_repetition)? != part.end_exposure
        {
            return Err(ProfileFault::Coverage);
        }
        let sliced = profile.slice(part.first_exposure, part.end_exposure)?;
        let ticks = sliced.runs.iter().try_fold(0_u64, |ticks, run| {
            ticks
                .checked_add(
                    u64::from(run.count)
                        .checked_mul(u64::from(run.cycle.ticks()))
                        .ok_or(ProfileFault::Coverage)?,
                )
                .ok_or(ProfileFault::Coverage)
        })?;
        if ticks != part.intrinsic_ticks {
            return Err(ProfileFault::Coverage);
        }
        let mut cursor = part.first_exposure;
        let mut actual_publications = Vec::new();
        let mut samples = Vec::new();
        for run in &sliced.runs {
            match &run.effect {
                Effect::Publication(_) => actual_publications.push(
                    publications
                        .get(&(raw.name, cursor))
                        .ok_or(ProfileFault::Binding)?
                        .ordinal,
                ),
                Effect::Sample(_, _) => samples.push(cursor),
                Effect::None => {}
            }
            cursor = cursor
                .checked_add(u64::from(run.count))
                .ok_or(ProfileFault::Coverage)?;
        }
        if actual_publications != part.publication_inputs
            || samples != part.hardware_result_exposures
        {
            return Err(ProfileFault::Binding);
        }
        at = part.end_exposure;
    }
    if at != profile.exposures {
        return Err(ProfileFault::Coverage);
    }
    Ok(())
}

fn materialization_bindings(
    buffers: &[Materialization],
    profiles: &[Profile],
) -> Result<(), ProfileFault> {
    if buffers.len() != 5 {
        return Err(ProfileFault::Binding);
    }
    for (index, buffer) in buffers.iter().enumerate() {
        let opportunity = match buffer.name.as_str() {
            "HazeLineTable" | "PaletteState" if buffer.initial_seed => "InitialProducerState",
            "HazeBackdropBands" if !buffer.initial_seed => "HazeBackdropBandsReady",
            "HudTilemap" if !buffer.initial_seed => "HudTilemapReady",
            "SpriteDrawList" if !buffer.initial_seed => "SpriteDrawListReady",
            _ => return Err(ProfileFault::Unsupported),
        };
        if buffer.opportunity != opportunity
            || buffer.length == 0
            || buffers
                .iter()
                .take(index)
                .any(|previous| previous.name == buffer.name)
        {
            return Err(ProfileFault::Binding);
        }
        match (buffer.initial_seed, buffer.profile, buffer.after_exposure) {
            (true, None, None) => {}
            (false, Some(name), Some(at))
                if profiles
                    .iter()
                    .any(|profile| profile.name == name && at <= profile.exposures) => {}
            _ => return Err(ProfileFault::Binding),
        }
    }
    Ok(())
}
/// Decode the six accepted template/binding TOML texts without loading reference bytes.
///
/// Text order is profiles, portions, publications, samples, materializations, arithmetic.
/// Source provenance and exact artifact integrity remain separate admission obligations.
/// # Errors
/// Rejects missing profiles, malformed effects, duplicate bindings and coverage gaps.
pub fn decode(texts: [&str; 6]) -> Result<Templates, ProfileFault> {
    let [
        profile_text,
        portion_text,
        publication_text,
        sample_text,
        materialization_text,
        arithmetic_text,
    ] = texts;
    let raw: ProfileFile = parse(profile_text)?;
    let portions: PortionFile = parse(portion_text)?;
    let publications: PublicationFile = parse(publication_text)?;
    let samples: SampleFile = parse(sample_text)?;
    let materializations: MaterializationFile = parse(materialization_text)?;
    let arithmetic: ArithmeticFile = parse(arithmetic_text)?;
    if [
        raw.format,
        portions.format,
        publications.format,
        samples.format,
        materializations.format,
        arithmetic.format,
    ]
    .iter()
    .any(|format| *format != 1)
    {
        return Err(ProfileFault::Format);
    }
    let mut bindings = Bindings::new(arithmetic, publications, samples)?;
    let mut profiles = Vec::new();
    let publication_navigation = bindings.publications.clone();
    for profile in &raw.profile {
        if profiles
            .iter()
            .any(|existing: &Profile| existing.name == profile.name)
        {
            return Err(ProfileFault::Profile);
        }
        let decoded = bindings.profile(profile)?;
        coverage(
            profile,
            &decoded,
            &portions.portion,
            &publication_navigation,
        )?;
        profiles.push(decoded);
    }
    if profiles.len() != 6 || !bindings.publications.is_empty() || !bindings.samples.is_empty() {
        return Err(ProfileFault::Binding);
    }
    materialization_bindings(&materializations.buffer, &profiles)?;
    Ok(Templates {
        profiles,
        portions: portions.portion,
        materializations: materializations.buffer,
    })
}
