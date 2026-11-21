//! The resident arrangement, three CPU senders and transport at a common load boundary.

mod admission;
#[cfg(test)]
mod tests;

use super::{AdmissionFault, sender::Senders};
use crate::state::owned::AudioPin;
use audio_transport::{
    AudioCyclePhase, AudioInstant, AudioPhase, CpuMasterTicks, InstantFraction, Playback,
    ReleaseHistory, Request, SenderProgress, SenderState, SoundGroup,
};
use canonical::model::{Model, Pinned};
use canonical::{Canonical, Schema};
use data::assets::audio::ResidentAudio;
use num_bigint::BigInt;
use std::ops::{Mul, Sub};

/// Why the opening room's audio context or saved state could not be admitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The clock or latch start differs from the selected room load.
    Origin,
    /// A required content or saved-state relation is invalid.
    Invalid(&'static str),
    /// Saved state belongs to a different arrangement or resource bank.
    ContentIdentity,
}

/// The relation between CPU master ticks since the room load and local audio cycles.
///
/// CPU time zero is the CPU frontier at the load. Audio time zero is the SMP frontier
/// at that same load. The SMP can already be ahead of the CPU at this boundary.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockRelation {
    cpu_frequency: u64,
    audio_oscillator_frequency: u64,
    loaded_smp_offset: i64,
}

impl ClockRelation {
    /// Supply the measured rates and loaded SMP offset of the selected room.
    #[must_use]
    pub const fn new(
        cpu_frequency: u64,
        audio_oscillator_frequency: u64,
        loaded_smp_offset: i64,
    ) -> Self {
        Self {
            cpu_frequency,
            audio_oscillator_frequency,
            loaded_smp_offset,
        }
    }

    const OPENING_CERES: Self = Self::new(21_477_272, 24_607_104, 448_687_296);

    /// The local audio instant of a CPU opportunity.
    ///
    /// Returns absence for CPU instants before the loaded SMP frontier.
    #[must_use]
    pub fn at_cpu(self, ticks: &CpuMasterTicks) -> Option<AudioInstant> {
        let oscillator_time: BigInt = Mul::mul(&ticks.0, self.audio_oscillator_frequency).into();
        AudioInstant::try_from(InstantFraction {
            numerator: oscillator_time.sub(self.loaded_smp_offset),
            denominator: self.cpu_frequency.saturating_mul(24).into(),
        })
        .ok()
    }
}

/// The authenticated room origin and the clock relation recovered from it.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Origin {
    seed_identity: [u8; 32],
    clock: ClockRelation,
}

impl Origin {
    /// Bind the selected room-load identity and measured initial timing.
    ///
    /// The caller authenticates the supplied load identity at its source.
    ///
    /// # Errors
    /// Returns an admission fault if the physical relation or first observation differs from
    /// the supported room load.
    pub fn opening_ceres(
        seed_identity: [u8; 32],
        clock: ClockRelation,
        first_poll: &AudioInstant,
    ) -> Result<Self, AdmissionFault> {
        if clock != ClockRelation::OPENING_CERES || first_poll != &Self::first_poll() {
            return Err(AdmissionFault::ResidentAudio(Fault::Origin));
        }
        Ok(Self {
            seed_identity,
            clock,
        })
    }

    fn first_poll() -> AudioInstant {
        audio_transport::time::halves(811)
    }
}

struct BankPin;
impl Pinned for BankPin {
    type State = ResidentAudio;
    const NAME: &'static str = "resident-audio-bank";
    const SHAPE: u64 = 0x66D2_BAA6_6BF1_6B34;
    const TRACE_U8_SAMPLES: &'static [(&'static str, u8)] = &[("MusicalPitch", 19)];
}

/// Imported arrangement and complete checked resource context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    bank: Model<BankPin>,
    identity: [u8; 32],
    origin: Origin,
}

impl Content {
    /// Admit the ROM-selected content and bind its graph, aliases and resource identities.
    ///
    /// # Errors
    /// Returns an admission fault if the bank is incomplete or its schema changed.
    pub fn new(bank: ResidentAudio, origin: Origin) -> Result<Self, AdmissionFault> {
        canonical::model::pin_holds::<BankPin>().map_err(AdmissionFault::StatePin)?;
        admission::validate_bank(&bank).map_err(AdmissionFault::ResidentAudio)?;
        let bank = Model(bank);
        let mut encoded = canonical::to_bytes(&Schema::<Model<BankPin>>::new());
        bank.encode(&mut encoded);
        let identity = data::rom::sha256(&encoded);
        Ok(Self {
            bank,
            identity,
            origin,
        })
    }

    /// The immutable ROM-selected semantic bank.
    #[must_use]
    pub const fn bank(&self) -> &ResidentAudio {
        &self.bank.0
    }
}

/// Independently recovered room-load state shared by both audio owners.
pub struct Load {
    /// All three normalized CPU sender states from the same load as transport.
    pub senders: Senders,
    /// The CPU-to-audio effect latches at load, in their named groups.
    pub requests: enum_map::EnumMap<SoundGroup, Request>,
    /// Complete ordered effect release histories, including dormant entries.
    pub history: enum_map::EnumMap<SoundGroup, ReleaseHistory>,
    /// Loaded audio timer prescaler phase.
    pub timer_phase: AudioCyclePhase,
    /// Loaded unread timer count.
    pub timer_unread: u8,
}

/// One admitted resident session, including unpublished transport and sender work.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    transport: Model<AudioPin>,
    senders: Senders,
    origin: Origin,
    content_identity: [u8; 32],
}

impl State {
    /// Admit the independently recovered opening-room quantities and initialize transport once.
    ///
    /// Provide both ordered loaded tracks, all four ordered release histories, and both
    /// physical latch directions as part of complete transport and sender states. At the
    /// selected boundary all three senders start Ready, with empty queues and released replies.
    ///
    /// # Errors
    /// Returns a typed fault for an incomplete bank, invalid state or mismatched loaded phase.
    pub fn opening_ceres(
        mut transport: Box<audio_transport::State>,
        load: Load,
        content: &Content,
    ) -> Result<Self, AdmissionFault> {
        let slot = |sender: &SenderState| {
            sender.now == audio_transport::CpuMasterTicks(0)
                && sender.progress == SenderProgress::Ready
                && sender.reply == Request::Absent
                && sender.pending.is_none()
                && sender.queue.is_empty()
        };
        let Load {
            senders,
            requests,
            history,
            timer_phase,
            timer_unread,
        } = load;
        let first_poll = Origin::first_poll();
        let first_phase = AudioPhase::ObserveClock {
            at: first_poll.clone(),
        };
        if transport.now != audio_transport::time::cycles(0)
            || transport.clock.lattice != first_poll
            || transport.phase != first_phase
            || transport.clock.phase != timer_phase
            || transport.clock.unread != timer_unread
            || transport.requests != requests
            || !slot(&senders.action.0)
            || !slot(&senders.ambient.0)
            || !slot(&senders.contact.0)
            || transport.groups.iter().any(|(group, saved)| {
                !matches!(saved.playback, Playback::Empty)
                    || saved.reply != senders.get(group).reply
                    || saved.history != history[group]
                    || transport.requests[group] != Request::Absent
            })
        {
            return Err(AdmissionFault::ResidentAudio(Fault::Origin));
        }
        transport.phase = AudioPhase::ObserveClock {
            at: transport.now.clone(),
        };
        audio_transport::transport::initialize(&mut transport);
        if transport.phase != first_phase {
            return Err(AdmissionFault::ResidentAudio(Fault::Origin));
        }
        let state = Self {
            origin: content.origin,
            content_identity: content.identity,
            transport: Model(transport),
            senders,
        };
        state.validate(content)?;
        Ok(state)
    }

    /// The complete committed and pending audio transport.
    #[must_use]
    pub fn transport(&self) -> &audio_transport::State {
        &self.transport.0
    }

    /// The three committed and pending CPU senders.
    #[must_use]
    pub const fn senders(&self) -> &Senders {
        &self.senders
    }

    /// The loaded phase relation, retained through save and restore.
    #[must_use]
    pub const fn clock(&self) -> ClockRelation {
        self.origin.clock
    }

    /// The authenticated load identity. Fresh loading supplies it and a restore checks it.
    #[must_use]
    pub const fn seed_identity(&self) -> [u8; 32] {
        self.origin.seed_identity
    }

    pub(super) fn validate(&self, content: &Content) -> Result<(), AdmissionFault> {
        if self.content_identity != content.identity {
            return Err(AdmissionFault::ResidentAudio(Fault::ContentIdentity));
        }
        if self.origin != content.origin {
            return Err(AdmissionFault::ResidentAudio(Fault::Origin));
        }
        self.senders.validate()?;
        let bank = content.bank();
        for group in [SoundGroup::Action, SoundGroup::Ambient, SoundGroup::Contact] {
            let cues = bank
                .content
                .effects
                .keys()
                .copied()
                .filter(|cue| audio_transport::effects::group(*cue) == group);
            if !admission::valid_sender_phase(self.senders.get(group), cues) {
                return Err(AdmissionFault::ResidentAudio(Fault::Invalid(
                    "sender service phase",
                )));
            }
        }
        admission::validate_resident_state(&self.transport.0, bank)
            .map_err(AdmissionFault::ResidentAudio)
    }

    pub(super) fn advance_transport(
        &mut self,
        content: &Content,
        stop: &AudioInstant,
        presentations: &[audio_transport::transport::Input],
    ) -> Result<super::ProductionOutputs, super::ProductionFault> {
        super::advance_transport(
            &mut self.transport.0,
            &content.bank.0.content,
            stop,
            presentations,
        )
    }

    pub(super) fn advance_sender(
        &mut self,
        group: SoundGroup,
        input: super::SenderInput<'_>,
    ) -> Result<super::ProductionOutputs, super::ProductionFault> {
        super::sender::advance(self.senders.current(group), group, input)
            .map(super::ProductionOutputs::AudioSender)
    }
}

/// Validate semantic content and retained state for a transport-only start.
///
/// A scoped caller supplies the resolved timbre count and delay because this entry can
/// use source-admitted content without admitting the complete resident resource bank.
///
/// # Errors
/// Returns a typed fault for incomplete graphs, track positions, histories or voice ownership.
pub fn validate_scoped(
    state: &audio_transport::State,
    graph: &audio_transport::Content,
    timbres: usize,
) -> Result<(), Fault> {
    let echo_delay = state.song.mix.echo_delay;
    admission::validate_graph(graph, timbres, echo_delay)?;
    admission::validate_song(&state.song, graph, timbres, echo_delay)?;
    admission::validate_groups(state, graph, None)
}
