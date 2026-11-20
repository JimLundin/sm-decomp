//! Independent current HUD drawing inputs and checked physical handler templates.

use super::profiles::{Effect, ProfileName, Templates};
use data::timing::Sample;
use game::production::frame::interrupts::{
    DrawingContent, DrawingInputs, DrawingProfiles, DrawingReceipt, DrawingRequest, DrawingTarget,
    NextRequest, VideoSettings,
};
use serde::Deserialize;

/// Invalid current handler inputs or incomplete physical producer bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterruptInputFault {
    /// Current physical WRAM is incomplete.
    Length,
    /// The current software drawing request is outside the qualified arms.
    Request,
    /// A separately requested interrupt requires an unqualified arm.
    NextRequest,
    /// A physical profile or complete current producer binding is missing or incompatible.
    Profile,
    /// Complete preceding-writer and schedule-owner provenance is absent or incompatible.
    Provenance,
    /// Immutable operand recipe or imported ROM content is incomplete or incompatible.
    Content,
}

fn byte(bytes: &[u8], at: usize) -> Result<u8, InterruptInputFault> {
    bytes.get(at).copied().ok_or(InterruptInputFault::Length)
}

fn word(bytes: &[u8], at: usize) -> Result<u16, InterruptInputFault> {
    Ok(u16::from_le_bytes([
        byte(bytes, at)?,
        byte(bytes, at.wrapping_add(1))?,
    ]))
}

/// Decode complete current software request and gameplay video mirror fields.
///
/// Source admission separately proves any borrowed field invariant up to each
/// reached reader; entry stability alone does not admit a first-cut replacement.
/// No handler result or publication byte is read by this decoder.
/// # Errors
/// Rejects incomplete memory and unsupported request or next-request arms.
pub fn current(bytes: &[u8]) -> Result<DrawingInputs, InterruptInputFault> {
    if bytes.len() != 131_072 {
        return Err(InterruptInputFault::Length);
    }
    let request = match word(bytes, 171)? {
        4 => DrawingRequest::Begin,
        6 => DrawingRequest::End,
        _ => return Err(InterruptInputFault::Request),
    };
    if byte(bytes, 167)? != 0 {
        return Err(InterruptInputFault::NextRequest);
    }
    Ok(DrawingInputs {
        request,
        next_request: NextRequest::None,
        video: VideoSettings {
            background_three_map: byte(bytes, 91)?,
            colour_window: byte(bytes, 112)?,
            colour_math: byte(bytes, 115)?,
            main_layers: byte(bytes, 106)?,
        },
    })
}

/// Import a separately pinned terminal schedule fact, without using source coordinates.
/// # Errors
/// Rejects early publication, missing native return and incompatible owner contracts.
pub fn terminal_schedule(text: &str) -> Result<[DrawingRequest; 2], InterruptInputFault> {
    #[derive(Deserialize)]
    struct Projection {
        format: String,
        schedule_owner: String,
        next_request: String,
        next_request_is_terminal_schedule: bool,
        #[serde(flatten)]
        completion: HandlerReturnPolicy,
        terminal_identity_binding: String,
        source_coordinates_role: String,
        expected_callback_authority: bool,
        terminal: Vec<HandlerReturn>,
    }

    #[derive(serde::Deserialize)]
    struct HandlerReturnPolicy {
        completion_authority: String,
        no_additional_zero_cost_schedule_stage: bool,
        consume_once_at_actual_handler_completion: bool,
    }
    #[derive(Deserialize)]
    struct HandlerReturn {
        handler: ProfileName,
        terminal_schedule: ProfileName,
        source_last_return_function: String,
        native_entry_separate_hardware_work: bool,
        native_return_already_in_full_profile_bill: bool,
        publish_terminal_only_when_native_return_complete: bool,
    }
    let projection: Projection =
        toml::from_str(text).map_err(|_| InterruptInputFault::Provenance)?;
    if projection.format != "irq-terminal-hud-schedule-v1"
        || projection.schedule_owner != "HardwareHudOverlaySchedule"
        || projection.next_request != "None"
        || projection.next_request_is_terminal_schedule
        || projection.completion.completion_authority
            != "ActualFullQualifiedHandlerProfileIncludingNativeReturn"
        || !projection.completion.no_additional_zero_cost_schedule_stage
        || projection.terminal_identity_binding != "ActualAdmittedInterruptCompletionIdentity"
        || !projection
            .completion
            .consume_once_at_actual_handler_completion
        || projection.source_coordinates_role != "ProvenanceOnly"
        || projection.expected_callback_authority
        || projection.terminal.len() != 2
    {
        return Err(InterruptInputFault::Provenance);
    }
    [ProfileName::HudDrawingBegin, ProfileName::HudDrawingEnd]
        .map(|handler| {
            let mut selected = projection
                .terminal
                .iter()
                .filter(|terminal| terminal.handler == handler);
            let terminal = selected
                .next()
                .filter(|_| selected.next().is_none())
                .ok_or(InterruptInputFault::Provenance)?;
            if terminal.source_last_return_function != "rti"
                || !terminal.native_entry_separate_hardware_work
                || !terminal.native_return_already_in_full_profile_bill
                || !terminal.publish_terminal_only_when_native_return_complete
            {
                return Err(InterruptInputFault::Provenance);
            }
            match (handler, terminal.terminal_schedule) {
                (ProfileName::HudDrawingBegin, ProfileName::HudDrawingEnd) => {
                    Ok(DrawingRequest::End)
                }
                (ProfileName::HudDrawingEnd, ProfileName::HudDrawingBegin) => {
                    Ok(DrawingRequest::Begin)
                }
                _ => Err(InterruptInputFault::Provenance),
            }
        })
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| InterruptInputFault::Provenance)
}

#[derive(Deserialize)]
struct Provenance {
    format: String,
    prehandler_inputs_are_actual_current_video: bool,
    expected_publication_bytes_used_as_inputs: bool,
    schedule_owner: String,
    end_next_request: String,
    field: Vec<Field>,
}

#[derive(Deserialize)]
struct Field {
    name: String,
    handler: ProfileName,
    private_wram_offset: usize,
    quantity_width: usize,
    origin_equal_at_handler_entry: bool,
    preceding_source_writer_indices: Vec<u64>,
    preceding_owner_names: Vec<String>,
}

#[derive(Deserialize)]
struct ContentRecipe {
    format: String,
    immutable_content_authority: String,
    mirrors_authority: String,
    expected_callback_bytes_used_as_inputs: bool,
    immutable_content_identity: String,
    publication_recipe: Vec<Operand>,
    constant: Vec<Constant>,
}

#[derive(Deserialize, PartialEq, Eq)]
enum Origin {
    ContentConstant,
    PreHandlerMirror,
    Zero,
}

#[derive(Deserialize)]
struct Operand {
    handler: ProfileName,
    device: String,
    origin_kind: Origin,
    source_byte_ordinal: usize,
    source_constant_index: Option<u64>,
    private_current_mirror_offset: Option<usize>,
}

#[derive(Deserialize)]
struct Constant {
    source_index: u64,
    private_rom_operand_file_offset: usize,
    quantity_width: usize,
}

impl ContentRecipe {
    fn operand(&self, handler: ProfileName, device: &str) -> Result<&Operand, InterruptInputFault> {
        let mut operands = self
            .publication_recipe
            .iter()
            .filter(|operand| operand.handler == handler && operand.device == device);
        let operand = operands.next().ok_or(InterruptInputFault::Content)?;
        if operands.next().is_some() {
            return Err(InterruptInputFault::Content);
        }
        Ok(operand)
    }

    fn constant(
        &self,
        rom: &[u8],
        handler: ProfileName,
        device: &str,
        ordinal: usize,
        width: usize,
    ) -> Result<u8, InterruptInputFault> {
        let operand = self.operand(handler, device)?;
        if operand.origin_kind != Origin::ContentConstant
            || operand.source_byte_ordinal != ordinal
            || operand.private_current_mirror_offset.is_some()
        {
            return Err(InterruptInputFault::Content);
        }
        let key = operand
            .source_constant_index
            .ok_or(InterruptInputFault::Content)?;
        let mut constants = self
            .constant
            .iter()
            .filter(|constant| constant.source_index == key);
        let constant = constants.next().ok_or(InterruptInputFault::Content)?;
        if constants.next().is_some() || constant.quantity_width != width || ordinal >= width {
            return Err(InterruptInputFault::Content);
        }
        let at = constant
            .private_rom_operand_file_offset
            .checked_add(ordinal)
            .ok_or(InterruptInputFault::Content)?;
        rom.get(at).copied().ok_or(InterruptInputFault::Content)
    }

    fn target(
        &self,
        rom: &[u8],
        handler: ProfileName,
    ) -> Result<DrawingTarget, InterruptInputFault> {
        for (low, high) in [
            ("IrqVerticalLow", "IrqVerticalHigh"),
            ("IrqHorizontalLow", "IrqHorizontalHigh"),
        ] {
            if self.operand(handler, low)?.source_constant_index
                != self.operand(handler, high)?.source_constant_index
            {
                return Err(InterruptInputFault::Content);
            }
        }
        let vertical = u16::from_le_bytes([
            self.constant(rom, handler, "IrqVerticalLow", 0, 2)?,
            self.constant(rom, handler, "IrqVerticalHigh", 1, 2)?,
        ]);
        let horizontal = u16::from_le_bytes([
            self.constant(rom, handler, "IrqHorizontalLow", 0, 2)?,
            self.constant(rom, handler, "IrqHorizontalHigh", 1, 2)?,
        ]);
        if horizontal > 511 || vertical > 511 {
            return Err(InterruptInputFault::Content);
        }
        Ok(DrawingTarget {
            horizontal,
            vertical,
        })
    }
}

/// Import HUD constants and beam coordinates from checked ROM producer operands.
///
/// The source loader must separately admit the recipe hash and checked ROM identity.
/// Current gameplay mirror bytes are validated as sources, never imported from a
/// future handler snapshot or device publication.
/// # Errors
/// Rejects incomplete recipes, incompatible operand identities and invalid targets.
pub fn content(rom: &[u8], text: &str) -> Result<DrawingContent, InterruptInputFault> {
    let recipe: ContentRecipe = toml::from_str(text).map_err(|_| InterruptInputFault::Content)?;
    if recipe.format != "irq-static-content-and-current-recipe-v1"
        || recipe.immutable_content_authority != "CheckedRomProducerOperand"
        || recipe.mirrors_authority != "ActualCurrentPreHandlerVideoOwner"
        || recipe.expected_callback_bytes_used_as_inputs
        || recipe.immutable_content_identity != "CheckedRomIdentity"
        || recipe.publication_recipe.len() != 16
        || recipe.constant.len() != 6
    {
        return Err(InterruptInputFault::Content);
    }
    let constants = recipe
        .constant
        .iter()
        .map(|constant| constant.source_index)
        .collect::<std::collections::BTreeSet<_>>();
    let referenced = recipe
        .publication_recipe
        .iter()
        .filter_map(|operand| operand.source_constant_index)
        .collect::<std::collections::BTreeSet<_>>();
    if constants.len() != 6 || constants != referenced {
        return Err(InterruptInputFault::Content);
    }
    for device in ["ColorWindow", "ColorMath"] {
        let operand = recipe.operand(ProfileName::HudDrawingBegin, device)?;
        if operand.origin_kind != Origin::Zero
            || operand.source_byte_ordinal != 0
            || operand.source_constant_index.is_some()
            || operand.private_current_mirror_offset.is_some()
        {
            return Err(InterruptInputFault::Content);
        }
    }
    for (device, offset) in [
        ("ColorWindow", 112),
        ("ColorMath", 115),
        ("BackgroundThreeMap", 91),
        ("MainLayers", 106),
    ] {
        let operand = recipe.operand(ProfileName::HudDrawingEnd, device)?;
        if operand.origin_kind != Origin::PreHandlerMirror
            || operand.source_byte_ordinal != 0
            || operand.source_constant_index.is_some()
            || operand.private_current_mirror_offset != Some(offset)
        {
            return Err(InterruptInputFault::Content);
        }
    }
    Ok(DrawingContent {
        hud: VideoSettings {
            background_three_map: recipe.constant(
                rom,
                ProfileName::HudDrawingBegin,
                "BackgroundThreeMap",
                0,
                1,
            )?,
            colour_window: 0,
            colour_math: 0,
            main_layers: recipe.constant(rom, ProfileName::HudDrawingBegin, "MainLayers", 0, 1)?,
        },
        after_begin: recipe.target(rom, ProfileName::HudDrawingBegin)?,
        after_end: recipe.target(rom, ProfileName::HudDrawingEnd)?,
    })
}

/// Admit both handler receipts from complete unchanged-video source provenance.
///
/// The source loader must separately check this document's approved hash. The
/// actual request selects a receipt at use; the End request comes from completed
/// Begin work, independently of the origin request and expected interrupt counts.
/// # Errors
/// Rejects missing, duplicate, incompatible or stale full provenance predicates.
pub fn receipts(
    current: DrawingInputs,
    text: &str,
) -> Result<[DrawingReceipt; 2], InterruptInputFault> {
    let proof: Provenance = toml::from_str(text).map_err(|_| InterruptInputFault::Provenance)?;
    if proof.format != "current-irq-video-owner-provenance-v1"
        || !proof.prehandler_inputs_are_actual_current_video
        || proof.expected_publication_bytes_used_as_inputs
        || proof.schedule_owner != "HardwareHudOverlaySchedule"
        || proof.end_next_request != "None"
        || proof.field.len() != 12
        || current.next_request != NextRequest::None
    {
        return Err(InterruptInputFault::Provenance);
    }
    for handler in [ProfileName::HudDrawingBegin, ProfileName::HudDrawingEnd] {
        for (name, offset, width) in [
            ("ColourWindow", 112, 1),
            ("ColourMath", 115, 1),
            ("BackgroundMap", 91, 1),
            ("MainLayers", 106, 1),
            ("NextRequest", 167, 1),
            ("HudSchedule", 171, 2),
        ] {
            let mut fields = proof
                .field
                .iter()
                .filter(|field| field.name == name && field.handler == handler);
            let field = fields.next().ok_or(InterruptInputFault::Provenance)?;
            if fields.next().is_some()
                || field.private_wram_offset != offset
                || field.quantity_width != width
            {
                return Err(InterruptInputFault::Provenance);
            }
            if name == "HudSchedule" && handler == ProfileName::HudDrawingEnd {
                if field.origin_equal_at_handler_entry
                    || field.preceding_source_writer_indices.len() != 1
                    || field.preceding_owner_names != ["QualifiedSourceWindow"]
                {
                    return Err(InterruptInputFault::Provenance);
                }
            } else if !field.origin_equal_at_handler_entry
                || !field.preceding_source_writer_indices.is_empty()
                || !field.preceding_owner_names.is_empty()
            {
                return Err(InterruptInputFault::Provenance);
            }
        }
    }
    Ok(
        [DrawingRequest::Begin, DrawingRequest::End].map(|request| DrawingReceipt {
            dependency: DrawingInputs { request, ..current },
        }),
    )
}

/// Check the complete handler publication namespaces and live hardware samples.
///
/// Imported HUD settings and targets come from independently qualified content;
/// gameplay video settings remain actual current owner fields.
/// # Errors
/// Rejects missing profiles, extra/missing effects or incompatible producer bindings.
pub fn profiles(
    templates: &Templates,
    content: DrawingContent,
) -> Result<DrawingProfiles, InterruptInputFault> {
    let begin = checked(
        templates,
        ProfileName::HudDrawingBegin,
        [0x2109, 0x2130, 0x2131, 0x212c],
    )?;
    let end = checked(
        templates,
        ProfileName::HudDrawingEnd,
        [0x2130, 0x2131, 0x2109, 0x212c],
    )?;
    if content.after_begin.horizontal > 511
        || content.after_begin.vertical > 511
        || content.after_end.horizontal > 511
        || content.after_end.vertical > 511
    {
        return Err(InterruptInputFault::Profile);
    }
    Ok(DrawingProfiles {
        begin,
        end,
        content,
    })
}

fn checked(
    templates: &Templates,
    name: ProfileName,
    settings_ports: [u16; 4],
) -> Result<data::timing::Template, InterruptInputFault> {
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == name);
    let profile = profiles.next().ok_or(InterruptInputFault::Profile)?;
    if profiles.next().is_some() {
        return Err(InterruptInputFault::Profile);
    }
    let expected_ports = settings_ports
        .into_iter()
        .chain([0x4209, 0x420a, 0x4207, 0x4208]);
    let mut expected_ports = expected_ports;
    let mut samples = [Sample::AcknowledgeIrq, Sample::Blanking].into_iter();
    for run in &profile.runs {
        for _ in 0..run.count {
            match &run.effect {
                Effect::None => {}
                Effect::Sample(sample, arithmetic) => {
                    if arithmetic.is_some() || samples.next() != Some(*sample) {
                        return Err(InterruptInputFault::Profile);
                    }
                }
                Effect::Publication(publication) => {
                    if publication.producer != "ActualHudDrawingOwner"
                        || publication.arithmetic.is_some()
                        || expected_ports.next() != Some(publication.port.0)
                    {
                        return Err(InterruptInputFault::Profile);
                    }
                }
            }
        }
    }
    if expected_ports.next().is_some() || samples.next().is_some() {
        return Err(InterruptInputFault::Profile);
    }
    profile.template().map_err(|_| InterruptInputFault::Profile)
}
