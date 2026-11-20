//! Static sender physical contracts imported from independently pinned source evidence.

use super::frame_origin::profiles::{ProfileName, Templates};
use game::production::frame::{
    Senders,
    sender_contract::{
        Admission, BusRegime, Contract, Current, PhysicalContent, QueueWindow, Queues,
    },
};
use serde::Deserialize;

/// Source evidence does not describe the admitted current sender origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Incomplete independently pinned origin memory.
    Origin,
    /// Invalid current physical queue representation.
    Queue,
    /// Incomplete or unsupported static execution and current-source contract.
    Contract,
    /// Missing or inconsistent complete physical source envelope.
    Profile,
}

/// Actual corrected-origin physical quantities retained with the production frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Origin {
    /// Physical positions alongside the sole clean queues.
    pub queues: Queues,
    /// Actual current service downtime and fetch regime.
    pub current: Current,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(from = "bool")]
enum Fact {
    True,
    False,
}

impl From<bool> for Fact {
    fn from(value: bool) -> Self {
        if value { Self::True } else { Self::False }
    }
}

impl Fact {
    const fn holds(self) -> bool {
        matches!(self, Self::True)
    }
}

#[derive(Deserialize)]
struct BusProof {
    format: String,
    bus: String,
    authority: String,
    current_bus_is_native_cpu_mode_state: Fact,
    later_hardware_snapshot_reseeding: Fact,
    bus_revalidate_with_actual_retained_clock_owner: Fact,
    broader_bus: String,
    scope: String,
    rom_speed_origin: SpeedOrigin,
    downtime_origin: DowntimeOrigin,
    static_execution: StaticExecution,
}

#[derive(Deserialize)]
struct StaticExecution {
    native_modes_are_importer_only_proof: Fact,
    actual_game_queues_and_progress_required: Fact,
    native_ring_metadata_seed_once_at_origin: Fact,
    actual_offer_and_dequeue_receipts_update_metadata: Fact,
}

#[derive(Deserialize)]
struct SpeedOrigin {
    source_field: String,
    initial_context_status_base: usize,
    raw_status_offset: usize,
    initial_context_field_offset: usize,
    quantity_width: usize,
    private_current_encoding: u64,
    typed_projection: String,
    source_field_is_cpu_speed_for_fast_rom_addresses: Fact,
    serialized_in_retained_cpu_context: Fact,
    current_cpu_memsel_writes_through_service_completion: usize,
    hardware_reset_is_in_bounded_frame_scope: Fact,
}

#[derive(Deserialize)]
struct DowntimeOrigin {
    source_guard: String,
    private_wram_offset: usize,
    quantity_width: usize,
    private_origin_encoding: u64,
    origin_equal_at_actual_use: Fact,
    preceding_writer_indices: Vec<usize>,
    typed_projection: String,
}

fn quantity(bytes: &[u8], offset: usize, width: usize) -> Result<u64, Fault> {
    if !matches!(width, 1 | 2 | 4) {
        return Err(Fault::Origin);
    }
    let end = offset.checked_add(width).ok_or(Fault::Origin)?;
    let bytes = bytes.get(offset..end).ok_or(Fault::Origin)?;
    let mut encoded = [0_u8; 8];
    encoded
        .get_mut(..width)
        .ok_or(Fault::Origin)?
        .copy_from_slice(bytes);
    Ok(u64::from_le_bytes(encoded))
}

/// Import actual corrected-origin quantities using the independently pinned backing proof.
/// `initial_context` is the same retained hardware origin member used by frame admission.
/// No later hardware or sender snapshot may be supplied here.
/// # Errors
/// Different backing relation, unsupported regime, changed downtime or queue mismatch.
pub fn origin(
    bus_proof: &str,
    wram: &[u8],
    initial_context: &[u8],
    senders: &Senders,
) -> Result<Origin, Fault> {
    let proof: BusProof = toml::from_str(bus_proof).map_err(|_| Fault::Contract)?;
    let speed = &proof.rom_speed_origin;
    let downtime = &proof.downtime_origin;
    if proof.format != "retained-origin-sender-bus-and-downtime-v1"
        || proof.bus != "Fast"
        || proof.authority != "ActualRetainedCorrectedOriginHardwareClockOwner"
        || proof.current_bus_is_native_cpu_mode_state.holds()
        || proof.later_hardware_snapshot_reseeding.holds()
        || !proof
            .bus_revalidate_with_actual_retained_clock_owner
            .holds()
        || proof.broader_bus != "Unsupported"
        || proof.scope != "CompletedReturnThroughActualFirstThreeServicesCompletion"
        || !proof
            .static_execution
            .native_modes_are_importer_only_proof
            .holds()
        || !proof
            .static_execution
            .actual_game_queues_and_progress_required
            .holds()
        || !proof
            .static_execution
            .native_ring_metadata_seed_once_at_origin
            .holds()
        || !proof
            .static_execution
            .actual_offer_and_dequeue_receipts_update_metadata
            .holds()
        || speed.source_field != "rom_speed"
        || speed.typed_projection != "BusRegimeFast"
        || !speed
            .source_field_is_cpu_speed_for_fast_rom_addresses
            .holds()
        || !speed.serialized_in_retained_cpu_context.holds()
        || speed.current_cpu_memsel_writes_through_service_completion != 0
        || speed.hardware_reset_is_in_bounded_frame_scope.holds()
        || speed
            .initial_context_status_base
            .checked_add(speed.raw_status_offset)
            != Some(speed.initial_context_field_offset)
        || downtime.source_guard != "GlobalSoundDowntime"
        || downtime.typed_projection != "CurrentSoundDowntime"
        || !downtime.origin_equal_at_actual_use.holds()
        || !downtime.preceding_writer_indices.is_empty()
    {
        return Err(Fault::Contract);
    }
    let fetch = quantity(
        initial_context,
        speed.initial_context_field_offset,
        speed.quantity_width,
    )?;
    let delay = quantity(wram, downtime.private_wram_offset, downtime.quantity_width)?;
    if fetch != speed.private_current_encoding || delay != downtime.private_origin_encoding {
        return Err(Fault::Origin);
    }
    let bus = match fetch {
        6 => BusRegime::Fast,
        8 => BusRegime::Slow,
        _ => return Err(Fault::Origin),
    };
    let current = Current {
        downtime: u8::try_from(delay).map_err(|_| Fault::Origin)?,
        bus,
    };
    current.validate().map_err(|_| Fault::Origin)?;
    Ok(Origin {
        queues: queues(wram, senders)?,
        current,
    })
}

#[derive(Deserialize)]
struct Proof {
    format: String,
    native_ring_seed_once_only: Fact,
    later_native_ring_reseeding: Fact,
    actual_clean_queue_and_progress_are_authority: Fact,
    source_ordinals_are_runtime_selectors: Fact,
    sender_span: Vec<Span>,
    execution_layout: Execution,
    current_sender_guard: Vec<Guard>,
}

#[derive(Deserialize)]
struct Guard {
    name: String,
    #[serde(default)]
    private_wram_offset: Option<usize>,
    #[serde(default)]
    quantity_width: Option<usize>,
    #[serde(default)]
    private_origin_encoding: Option<u64>,
    #[serde(default)]
    source_read_index: Option<usize>,
    origin_equal_at_use: Fact,
    origin_owner_unchanged: Fact,
    actual_completed_offer_or_sender_receipt_join_required: Fact,
    current_source_values_used_as_reseed: Fact,
    preceding_writer_indices: Vec<usize>,
}

/// Forward actual sound-offer origins from the independently accepted current guards.
/// The legacy cinematic guard names the power-bomb owner, not Contact suppression.
/// # Errors
/// Changed source, missing actual origin or a scene outside the checked demo boundary.
pub fn offer_current(
    proof: &str,
    actual_origin_wram: &[u8],
    checked_rom: &[u8],
) -> Result<game::production::frame::post_down_contract::OfferCurrent, Fault> {
    use crate::lock::Sha256;
    use game::production::frame::post_down_contract::{Demo, OfferCurrent, PowerBomb, Sounds};
    if Sha256::of(proof.as_bytes())
        != Sha256::try_from("ea6e75a9bcdf313e051d6c9de9fc40746f82d2f6fcd6919414c8a51323050792")
            .map_err(|_| Fault::Contract)?
        || actual_origin_wram.len() != 131_072
    {
        return Err(Fault::Contract);
    }
    data::rom::Rom::new(checked_rom.to_vec()).map_err(|_| Fault::Contract)?;
    let proof: Proof = toml::from_str(proof).map_err(|_| Fault::Contract)?;
    proof.validate()?;
    let mut values = Vec::new();
    let mut scene_offset = None;
    for (name, source) in [
        ("OfferDisableGuard", 3090),
        ("OfferSceneGuard", 3092),
        ("OfferCinematicGuard", 3095),
    ] {
        let mut guards = proof
            .current_sender_guard
            .iter()
            .filter(|guard| guard.name == name);
        let guard = guards.next().ok_or(Fault::Contract)?;
        let offset = guard.private_wram_offset.ok_or(Fault::Contract)?;
        if guards.next().is_some()
            || guard.quantity_width != Some(2)
            || guard.source_read_index != Some(source)
        {
            return Err(Fault::Contract);
        }
        let value = quantity(actual_origin_wram, offset, 2)?;
        if Some(value) != guard.private_origin_encoding {
            return Err(Fault::Origin);
        }
        if name == "OfferSceneGuard" {
            scene_offset = Some(offset);
        }
        values.push(value);
    }
    if !below_demo_range(
        actual_origin_wram,
        checked_rom,
        scene_offset.ok_or(Fault::Contract)?,
    )? {
        return Err(Fault::Contract);
    }
    let [sounds, _, power_bomb] = values.as_slice() else {
        return Err(Fault::Contract);
    };
    Ok(OfferCurrent {
        sounds: if *sounds == 0 {
            Sounds::Enabled
        } else {
            Sounds::Disabled
        },
        demo: Demo::Inactive,
        power_bomb: if *power_bomb == 0 {
            PowerBomb::Inactive
        } else {
            PowerBomb::Active
        },
    })
}

#[derive(Deserialize)]
struct Span {
    name: String,
    first_exposure: u64,
    end_exposure: u64,
    intrinsic_ticks: u64,
    full_completion_required: Fact,
}

#[derive(Deserialize)]
struct Execution {
    native_emulation_mode: Fact,
    direct_page_low_byte_zero: Fact,
    decimal_arithmetic: Fact,
    body_accumulator_width_bytes: u8,
    body_index_register_width_bytes: u8,
    instruction_fetch_cycle_ticks: u8,
    wram_data_cycle_ticks: u8,
    internal_cycle_ticks: u8,
    outgoing_port_cycle_ticks: u8,
    native_execution_fields_are_runtime_frame_state: Fact,
    hardware_bus_regime_matches_qualified_origin_required: Fact,
    indexed_native_ring_reads_have_no_page_cross: Fact,
    cpu_rom_speed_writes_before_service_completion: usize,
}

impl Proof {
    fn validate(&self) -> Result<(), Fault> {
        let execution = &self.execution_layout;
        if self.format != "bounded-origin-current-sender-profile-v1"
            || !self.native_ring_seed_once_only.holds()
            || self.later_native_ring_reseeding.holds()
            || !self.actual_clean_queue_and_progress_are_authority.holds()
            || self.source_ordinals_are_runtime_selectors.holds()
            || execution.native_emulation_mode.holds()
            || !execution.direct_page_low_byte_zero.holds()
            || execution.decimal_arithmetic.holds()
            || execution.body_accumulator_width_bytes != 1
            || execution.body_index_register_width_bytes != 1
            || execution.instruction_fetch_cycle_ticks != 6
            || execution.wram_data_cycle_ticks != 8
            || execution.internal_cycle_ticks != 6
            || execution.outgoing_port_cycle_ticks != 6
            || execution
                .native_execution_fields_are_runtime_frame_state
                .holds()
            || !execution
                .hardware_bus_regime_matches_qualified_origin_required
                .holds()
            || !execution
                .indexed_native_ring_reads_have_no_page_cross
                .holds()
            || execution.cpu_rom_speed_writes_before_service_completion != 0
        {
            return Err(Fault::Contract);
        }
        let names = [
            "ContactOfferTail",
            "ContactOfferHead",
            "OfferDisableGuard",
            "OfferSceneGuard",
            "OfferCinematicGuard",
            "GlobalSoundDowntime",
            "ActionProgress",
            "AmbientProgress",
            "ContactProgress",
            "ActionServiceHead",
            "ActionServiceTail",
            "AmbientServiceHead",
            "AmbientServiceTail",
            "ContactServiceHead",
            "ContactServiceTail",
            "ContactServiceCueSlotGuard",
            "ContactServiceCueBytes",
        ];
        if self.current_sender_guard.len() != names.len() {
            return Err(Fault::Contract);
        }
        for name in names {
            let mut guards = self
                .current_sender_guard
                .iter()
                .filter(|guard| guard.name == name);
            let guard = guards.next().ok_or(Fault::Contract)?;
            let joined = matches!(name, "ContactServiceTail" | "ContactServiceCueBytes");
            if guards.next().is_some()
                || guard.current_source_values_used_as_reseed.holds()
                || guard
                    .actual_completed_offer_or_sender_receipt_join_required
                    .holds()
                    != joined
                || guard.origin_owner_unchanged.holds() == joined
                || guard.origin_equal_at_use.holds() == joined
                || (!joined && !guard.preceding_writer_indices.is_empty())
            {
                return Err(Fault::Contract);
            }
        }
        Ok(())
    }

    fn template(
        &self,
        templates: &Templates,
        name: &str,
        ticks: u64,
    ) -> Result<data::timing::Template, Fault> {
        let mut spans = self.sender_span.iter().filter(|span| span.name == name);
        let span = spans.next().ok_or(Fault::Profile)?;
        if spans.next().is_some()
            || span.intrinsic_ticks != ticks
            || !span.full_completion_required.holds()
        {
            return Err(Fault::Profile);
        }
        let mut profiles = templates
            .profiles
            .iter()
            .filter(|profile| profile.name == ProfileName::PreWait);
        let profile = profiles.next().ok_or(Fault::Profile)?;
        if profiles.next().is_some() {
            return Err(Fault::Profile);
        }
        profile
            .slice(span.first_exposure, span.end_exposure)
            .map_err(|_| Fault::Profile)?
            .template()
            .map_err(|_| Fault::Profile)
    }
}

/// Import source-qualified complete envelopes; all source coordinates end at this edge.
/// The caller pins the independently accepted proof and all complete templates first.
/// # Errors
/// Unsupported execution/current contract or inconsistent complete physical envelopes.
pub fn content(proof: &str, templates: &Templates) -> Result<PhysicalContent, Fault> {
    use audio_transport::SoundGroup;
    let proof: Proof = toml::from_str(proof).map_err(|_| Fault::Contract)?;
    proof.validate()?;
    if proof.sender_span.len() != 7 {
        return Err(Fault::Profile);
    }
    Ok(PhysicalContent {
        admission: Admission::new(proof.template(templates, "Admission", 276)?)
            .map_err(|_| Fault::Profile)?,
        action: Contract::new(
            SoundGroup::Action,
            proof.template(templates, "ActionBody", 122)?,
            proof.template(templates, "ActionTail", 190)?,
        )
        .map_err(|_| Fault::Profile)?,
        ambient: Contract::new(
            SoundGroup::Ambient,
            proof.template(templates, "AmbientBody", 122)?,
            proof.template(templates, "AmbientTail", 190)?,
        )
        .map_err(|_| Fault::Profile)?,
        contact: Contract::new(
            SoundGroup::Contact,
            proof.template(templates, "ContactBody", 398)?,
            proof.template(templates, "ContactTail", 130)?,
        )
        .map_err(|_| Fault::Profile)?,
    })
}

/// Import complete immutable content and actual origin quantities together.
/// All inputs must be the independently pinned accepted members of one frame source.
/// # Errors
/// Either backing proof, physical envelope or actual origin fails admission.
pub fn admit(
    profile_proof: &str,
    bus_proof: &str,
    templates: &Templates,
    wram: &[u8],
    initial_context: &[u8],
    senders: &Senders,
) -> Result<(Origin, PhysicalContent), Fault> {
    let physical = content(profile_proof, templates)?;
    let origin = origin(bus_proof, wram, initial_context, senders)?;
    physical
        .admission
        .qualify(senders, &origin.queues, origin.current)
        .map_err(|_| Fault::Contract)?;
    Ok((origin, physical))
}

/// Import physical positions from the same corrected origin as the sole clean queues.
/// The caller independently pins the complete origin member before decoding it.
/// These positions never come from a later service snapshot.
/// # Errors
/// Incomplete origin memory, malformed positions or disagreement with clean queues.
pub fn queues(wram: &[u8], senders: &Senders) -> Result<Queues, Fault> {
    if wram.len() != 131_072 {
        return Err(Fault::Origin);
    }
    let window = |group: usize| {
        let head = wram
            .get(0x0643_usize.checked_add(group).ok_or(Fault::Origin)?)
            .copied()
            .ok_or(Fault::Origin)?;
        let tail = wram
            .get(0x0646_usize.checked_add(group).ok_or(Fault::Origin)?)
            .copied()
            .ok_or(Fault::Origin)?;
        QueueWindow::new(head, tail).map_err(|_| Fault::Queue)
    };
    Queues::new(window(0)?, window(1)?, window(2)?, senders).map_err(|_| Fault::Queue)
}

#[cfg(test)]
mod tests;

// Immutable boundary forwarded from accepted DEMO-IMPORTER.md (8b517ae37922628d).
fn below_demo_range(
    actual_origin_wram: &[u8],
    checked_rom: &[u8],
    scene_offset: usize,
) -> Result<bool, Fault> {
    // `guard` is the exact accepted OfferSceneGuard descriptor validated above.
    // `checked_rom` already has the accepted player-ROM identity; unheadered bytes.
    let scene_bytes: [u8; 2] = actual_origin_wram
        .get(scene_offset..scene_offset.checked_add(2).ok_or(Fault::Contract)?)
        .ok_or(Fault::Contract)?
        .try_into()
        .map_err(|_| Fault::Contract)?;
    let boundary_bytes: [u8; 2] = checked_rom
        .get(4466..4468)
        .ok_or(Fault::Contract)?
        .try_into()
        .map_err(|_| Fault::Contract)?;
    let admitted = u16::from_le_bytes(scene_bytes) < u16::from_le_bytes(boundary_bytes);
    if !admitted {
        return Err(Fault::Contract);
    }

    Ok(admitted)
}
