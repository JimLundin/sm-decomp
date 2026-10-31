//! Detached Session checkpoint components; registered by the production Session.
use crate::actors::snapshot::ActorsCheckpoint;
use crate::actors::snapshot::CheckpointImages;
use crate::actors::snapshot::CheckpointReader;
use crate::checkpoint_binding::{audio_reference_bytes, observation_binding};
use crate::scene::{
    component::DisplayStartMetadata,
    records::{AssetId, AssetRef, Revision},
};
use crate::session::Session;
use crate::session_owners::{
    AudioRequestCatalog, AudioRequests, MusicContinuation, MusicMeaning, MusicRequests,
    QueuedMusicMeaning, QueuedSoundMeaning, SoundMeaning, SoundRequests,
};
use std::sync::Arc;

/// Detached binary fields awaiting whole-Session validation. This value is never
/// installed as a competing gameplay owner or filled from a live Session.
#[derive(Clone, Debug, PartialEq)]
pub struct AuxiliaryGameplayCheckpoint {
    pub botwoon_death: crate::actors::botwoon_death::DeathProtocol,
    pub timing: crate::session::Timing,
    pub flow: crate::flow::Flow,
    pub menu: crate::menu_state::MenuState,
    pub gameplay_flags: crate::gameplay_flag_state::GameplayFlags,
    pub flag_compatibility: crate::gameplay_flag_state::FlagCompatibility,
    pub sprite_effects: crate::sprite_effect_state::SpriteEffects,
    pub sprite_compatibility: crate::sprite_effect_state::SpriteCompatibilityTails,
    pub minimap: crate::player_update::state::State,
    pub inventory: crate::item_pickups::Inventory,
    pub mechanisms: crate::session::Mechanisms,
    pub feedback: crate::session::Feedback,
    pub hdma: crate::hdma::HdmaObjects,
    pub messages: crate::session::MessageRequests,
    pub spawn_scratch: crate::session::SpawnScratch,
    pub compatibility_enemy_scratch: u16,
    pub shared_sequence: crate::player_collision::state::SequenceWorkspace,
    pub multiply_scratch: crate::player_collision::state::MultiplyScratch,
    pub pointer_scratch: crate::session::PointerScratch,
}
fn audio_asset(bytes: &mut Vec<u8>, asset: AssetRef) {
    bytes.extend_from_slice(&asset.id.0.to_le_bytes());
    bytes.extend_from_slice(&asset.revision.0);
}
fn audio_read_asset(read: &mut CheckpointReader<'_>) -> Result<AssetRef, String> {
    Ok(AssetRef {
        id: AssetId(read.id()?),
        revision: Revision(read.bytes(32)?.try_into().unwrap()),
    })
}
fn audio_continuation(bytes: &mut Vec<u8>, value: &MusicContinuation) {
    match value {
        MusicContinuation::Retain => bytes.push(0),
        MusicContinuation::Silence => bytes.push(1),
        MusicContinuation::Program { program, phase } => {
            bytes.push(2);
            audio_asset(bytes, *program);
            bytes.extend_from_slice(&phase.to_le_bytes());
        }
    }
}
fn audio_read_continuation(read: &mut CheckpointReader<'_>) -> Result<MusicContinuation, String> {
    match read.byte()? {
        0 => Ok(MusicContinuation::Retain),
        1 => Ok(MusicContinuation::Silence),
        2 => Ok(MusicContinuation::Program {
            program: audio_read_asset(read)?,
            phase: read.counter()?,
        }),
        _ => Err("unknown audio checkpoint continuation".into()),
    }
}
fn audio_music_meaning(bytes: &mut Vec<u8>, value: &MusicMeaning) {
    match value {
        MusicMeaning::Silence => bytes.push(0),
        MusicMeaning::Bank { bank, after } => {
            bytes.push(1);
            audio_asset(bytes, *bank);
            audio_continuation(bytes, after);
        }
        MusicMeaning::Program { program, phase } => {
            bytes.push(2);
            audio_asset(bytes, *program);
            bytes.extend_from_slice(&phase.to_le_bytes());
        }
    }
}
fn audio_music_record(bytes: &mut Vec<u8>, value: &Option<QueuedMusicMeaning>) {
    let Some(value) = value else {
        bytes.push(0);
        return;
    };
    bytes.push(1);
    bytes.extend_from_slice(&value.request.to_le_bytes());
    bytes.extend_from_slice(&value.source_tick.to_le_bytes());
    bytes.extend_from_slice(&value.source_interval.to_le_bytes());
    audio_music_meaning(bytes, &value.meaning);
}
fn audio_read_music_record(
    read: &mut CheckpointReader<'_>,
) -> Result<Option<QueuedMusicMeaning>, String> {
    match read.byte()? {
        0 => return Ok(None),
        1 => {}
        _ => return Err("unknown audio checkpoint music option".into()),
    }
    let request = read.word()?;
    let source_tick = read.counter()?;
    let source_interval = read.counter()?;
    let meaning = match read.byte()? {
        0 => MusicMeaning::Silence,
        1 => MusicMeaning::Bank {
            bank: audio_read_asset(read)?,
            after: audio_read_continuation(read)?,
        },
        2 => MusicMeaning::Program {
            program: audio_read_asset(read)?,
            phase: read.counter()?,
        },
        _ => return Err("unknown audio checkpoint music meaning".into()),
    };
    Ok(Some(QueuedMusicMeaning {
        request,
        source_tick,
        source_interval,
        meaning,
    }))
}
fn audio_sound_meaning(bytes: &mut Vec<u8>, value: &SoundMeaning) {
    match value {
        SoundMeaning::Stop => bytes.push(0),
        SoundMeaning::Program { program, phase } => {
            bytes.push(1);
            audio_asset(bytes, *program);
            bytes.extend_from_slice(&phase.to_le_bytes());
        }
    }
}
fn audio_sound_record(bytes: &mut Vec<u8>, value: &Option<QueuedSoundMeaning>) {
    let Some(value) = value else {
        bytes.push(0);
        return;
    };
    bytes.push(1);
    bytes.push(value.request);
    bytes.extend_from_slice(&value.source_tick.to_le_bytes());
    bytes.extend_from_slice(&value.source_interval.to_le_bytes());
    audio_sound_meaning(bytes, &value.meaning);
}
fn audio_read_sound_record(
    read: &mut CheckpointReader<'_>,
) -> Result<Option<QueuedSoundMeaning>, String> {
    match read.byte()? {
        0 => return Ok(None),
        1 => {}
        _ => return Err("unknown audio checkpoint sound option".into()),
    }
    let request = read.byte()?;
    let source_tick = read.counter()?;
    let source_interval = read.counter()?;
    let meaning = match read.byte()? {
        0 => SoundMeaning::Stop,
        1 => SoundMeaning::Program {
            program: audio_read_asset(read)?,
            phase: read.counter()?,
        },
        _ => return Err("unknown audio checkpoint sound meaning".into()),
    };
    Ok(Some(QueuedSoundMeaning {
        request,
        source_tick,
        source_interval,
        meaning,
    }))
}
fn audio_words<const N: usize>(read: &mut CheckpointReader<'_>) -> Result<[u16; N], String> {
    let mut values = Vec::with_capacity(N);
    for _ in 0..N {
        values.push(read.word()?);
    }
    values
        .try_into()
        .map_err(|_| "audio checkpoint word count".into())
}
fn audio_music_records<const N: usize>(
    read: &mut CheckpointReader<'_>,
) -> Result<[Option<QueuedMusicMeaning>; N], String> {
    let mut values = Vec::with_capacity(N);
    for _ in 0..N {
        values.push(audio_read_music_record(read)?);
    }
    values
        .try_into()
        .map_err(|_| "audio checkpoint music record count".into())
}
fn audio_sound_records<const N: usize>(
    read: &mut CheckpointReader<'_>,
) -> Result<[Option<QueuedSoundMeaning>; N], String> {
    let mut values = Vec::with_capacity(N);
    for _ in 0..N {
        values.push(audio_read_sound_record(read)?);
    }
    values
        .try_into()
        .map_err(|_| "audio checkpoint sound record count".into())
}
fn audio_catalog_bytes(catalog: &AudioRequestCatalog) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(catalog.music.len() as u64).to_le_bytes());
    for (selector, meaning) in &catalog.music {
        bytes.extend_from_slice(&selector.to_le_bytes());
        audio_music_meaning(&mut bytes, meaning);
    }
    bytes.extend_from_slice(&(catalog.sounds.len() as u64).to_le_bytes());
    for ((channel, selector), meaning) in &catalog.sounds {
        bytes.extend_from_slice(&(*channel as u64).to_le_bytes());
        bytes.push(*selector);
        audio_sound_meaning(&mut bytes, meaning);
    }
    bytes
}
impl AudioRequests {
    /// Complete detached request component. Capture rejects any pending publication.
    pub fn encode_checkpoint(
        &self,
        metadata: &DisplayStartMetadata,
        required: &AudioRequestCatalog,
    ) -> Result<Vec<u8>, String> {
        self.validate(metadata, required)
            .map_err(|error| format!("audio checkpoint {error:?}"))?;
        if metadata.identity.reference != metadata.reference.fingerprint {
            return Err("audio checkpoint Reference identity mismatch".into());
        }
        let mut bytes = Vec::from(b"SMAUDIO\x02".as_slice());
        let reference = audio_reference_bytes(&metadata.reference);
        bytes.extend_from_slice(&(reference.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&reference);
        bytes.push(u8::from(self.pending_mutation));
        let catalog_bytes = audio_catalog_bytes(&self.catalog);
        bytes.extend_from_slice(&(catalog_bytes.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&catalog_bytes);
        bytes.extend_from_slice(&self.uploading.to_le_bytes());
        bytes.extend_from_slice(&self.downtime.to_le_bytes());
        for word in self.music.tracks.iter().chain(&self.music.delays) {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        for word in [
            self.music.write,
            self.music.read,
            self.music.entry,
            self.music.timer,
            self.music.data_index,
            self.music.track_index,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.push(self.music.current_track);
        for record in &self.music.meanings {
            audio_music_record(&mut bytes, record);
        }
        audio_music_record(&mut bytes, &self.music.entry_meaning);
        audio_music_record(&mut bytes, &self.music.current_meaning);
        for sound in &self.sounds {
            bytes.extend_from_slice(&[
                sound.read,
                sound.write,
                sound.state,
                sound.current,
                sound.clear_delay,
                sound.maximum_queued,
            ]);
            bytes.extend_from_slice(&sound.queue);
            for record in &sound.meanings {
                audio_sound_record(&mut bytes, record);
            }
            audio_sound_record(&mut bytes, &sound.current_meaning);
        }
        Ok(bytes)
    }
    /// Decode into a new value against independent declarations, before adoption.
    /// Catalog meanings are compared to explicit encoded records, never inferred
    /// from queue numbers. Unknown tags, trailing bytes and missing records reject.
    pub fn decode_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
        catalog: Arc<AudioRequestCatalog>,
    ) -> Result<Self, String> {
        let mut read = CheckpointReader(bytes);
        if read.bytes(8)? != b"SMAUDIO\x02" {
            return Err("unsupported audio checkpoint schema".into());
        }
        let reference_size = usize::try_from(read.counter()?)
            .map_err(|_| "audio checkpoint Reference length overflow")?;
        if metadata.identity.reference != metadata.reference.fingerprint
            || read.bytes(reference_size)? != audio_reference_bytes(&metadata.reference)
        {
            return Err("audio checkpoint Reference declaration changed".into());
        }
        let pending_mutation = match read.byte()? {
            0 => false,
            _ => return Err("incomplete audio checkpoint publication".into()),
        };
        let catalog_size = usize::try_from(read.counter()?)
            .map_err(|_| "audio checkpoint catalog length overflow")?;
        if read.bytes(catalog_size)? != audio_catalog_bytes(&catalog) {
            return Err("audio checkpoint catalog declarations changed".into());
        }
        let uploading = read.word()?;
        let downtime = read.word()?;
        let tracks = audio_words(&mut read)?;
        let delays = audio_words(&mut read)?;
        let write = read.word()?;
        let music_read = read.word()?;
        let entry = read.word()?;
        let timer = read.word()?;
        let data_index = read.word()?;
        let track_index = read.word()?;
        let current_track = read.byte()?;
        let meanings = audio_music_records(&mut read)?;
        let entry_meaning = audio_read_music_record(&mut read)?;
        let current_meaning = audio_read_music_record(&mut read)?;
        let music = MusicRequests {
            tracks,
            delays,
            write,
            read: music_read,
            entry,
            timer,
            data_index,
            track_index,
            current_track,
            meanings,
            entry_meaning,
            current_meaning,
        };
        let mut sounds = Vec::with_capacity(3);
        for _ in 0..3 {
            sounds.push(SoundRequests {
                read: read.byte()?,
                write: read.byte()?,
                state: read.byte()?,
                current: read.byte()?,
                clear_delay: read.byte()?,
                maximum_queued: read.byte()?,
                queue: read.bytes(16)?.try_into().unwrap(),
                meanings: audio_sound_records(&mut read)?,
                current_meaning: audio_read_sound_record(&mut read)?,
            });
        }
        if !read.0.is_empty() {
            return Err("trailing audio checkpoint fields".into());
        }
        let result = Self {
            music,
            sounds: sounds
                .try_into()
                .map_err(|_| "audio checkpoint channel count")?,
            uploading,
            downtime,
            catalog,
            pending_mutation,
        };
        result
            .validate(metadata, &result.catalog)
            .map_err(|error| format!("audio checkpoint {error:?}"))?;
        Ok(result)
    }
}

use crate::scene::start::{
    ActionBindings, AudioProgramDeclaration, CommittedPhase, ContinuationHistory,
    InitialAudioIntent, InputHistory, OngoingMusic, PortHistory, ProgramAnchor, StartIdentity,
};
use crate::session_owners::{Control, CurrentControl, SessionStartRequirements};

fn control_identity(read: &mut CheckpointReader<'_>) -> Result<[u8; 32], String> {
    Ok(read.bytes(32)?.try_into().unwrap())
}
fn control_bool(read: &mut CheckpointReader<'_>) -> Result<bool, String> {
    match read.byte()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err("unknown Control checkpoint boolean".into()),
    }
}
fn control_anchor(bytes: &mut Vec<u8>, value: &ProgramAnchor) {
    bytes.extend_from_slice(&value.program.0.to_le_bytes());
    bytes.extend_from_slice(&value.revision.0);
    bytes.extend_from_slice(&value.source_interval.to_le_bytes());
    bytes.extend_from_slice(&value.phase.to_le_bytes());
    bytes.extend_from_slice(&value.timing_policy);
}
fn control_read_anchor(read: &mut CheckpointReader<'_>) -> Result<ProgramAnchor, String> {
    Ok(ProgramAnchor {
        program: AssetId(read.id()?),
        revision: Revision(control_identity(read)?),
        source_interval: read.counter()?,
        phase: read.counter()?,
        timing_policy: control_identity(read)?,
    })
}
impl Control {
    /// Lossless current Control component; immutable inputs are explicitly supplied.
    pub fn encode_checkpoint(
        &self,
        required: &SessionStartRequirements,
    ) -> Result<Vec<u8>, String> {
        let Self::Owned(current) = self else {
            return Err("no supplied Control checkpoint owner".into());
        };
        let m = &current.metadata;
        crate::scene::start::validate_control_metadata(m, &required.display)
            .map_err(|error| format!("Control checkpoint {error:?}"))?;
        let audio_requests = current
            .requests
            .encode_checkpoint(m, &required.audio_catalog)?;
        let mut bytes = Vec::from(b"SMCONTROL\x01".as_slice());
        bytes.extend_from_slice(&m.identity.schema.to_le_bytes());
        for identity in [
            m.identity.gameplay,
            m.identity.reference,
            m.identity.capture,
            m.identity.media.0,
        ] {
            bytes.extend_from_slice(&identity);
        }
        bytes.extend_from_slice(&m.identity.committed_tick.to_le_bytes());
        bytes.extend_from_slice(&m.identity.next_input_record.to_le_bytes());
        let reference = audio_reference_bytes(&m.reference);
        bytes.extend_from_slice(&(reference.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&reference);
        bytes.extend_from_slice(&m.samus_content.revision.0);
        let input = m.input_history;
        bytes.extend_from_slice(&input.capture);
        bytes.extend_from_slice(&input.policy);
        for port in input.ports {
            for word in [
                port.sampled,
                port.previous,
                port.pressed,
                port.repeated,
                port.repeat_counter,
            ] {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        bytes.push(u8::from(input.debug_second_port));
        for word in [
            input.repeat_first,
            input.repeat_next,
            input.released,
            input.timed_held,
            input.previous_timed_held,
            input.newly_held,
            input.held_timer,
            input.held_timer_reset,
            input.samus_held,
            input.samus_pressed,
            input.bindings.shoot,
            input.bindings.jump,
            input.bindings.run,
            input.bindings.cancel,
            input.bindings.switch,
            input.bindings.aim_down,
            input.bindings.aim_up,
            input.bug_fix_counter,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        let continuation = m.continuation;
        bytes.extend_from_slice(&continuation.capture);
        bytes.extend_from_slice(&continuation.observation_policy);
        bytes.push(match continuation.phase {
            CommittedPhase::InitialObservation => 0,
            CommittedPhase::SettledInterval => 1,
        });
        bytes.extend_from_slice(&continuation.next_input_record.to_le_bytes());
        bytes.extend_from_slice(&[
            continuation.outer,
            continuation.primary,
            continuation.secondary,
            continuation.music,
            continuation.message,
            continuation.auxiliary,
        ]);
        match continuation.captured_mode {
            None => bytes.push(0),
            Some(mode) => {
                bytes.push(1);
                bytes.extend_from_slice(&mode.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&m.audio.capture);
        bytes.extend_from_slice(&m.audio.revision.0);
        bytes.extend_from_slice(&m.audio.timing_policy);
        bytes.extend_from_slice(&(m.audio.declarations.len() as u64).to_le_bytes());
        for value in m.audio.declarations.iter() {
            bytes.extend_from_slice(&value.program.0.to_le_bytes());
            bytes.extend_from_slice(&value.bank.0.to_le_bytes());
            bytes.extend_from_slice(&value.revision.0);
            bytes.extend_from_slice(&value.definition);
        }
        match m.audio.music {
            OngoingMusic::Silent => bytes.push(0),
            OngoingMusic::Playing(anchor) => {
                bytes.push(1);
                control_anchor(&mut bytes, &anchor);
            }
        }
        bytes.extend_from_slice(&(m.audio.ambience.len() as u64).to_le_bytes());
        for anchor in m.audio.ambience.iter() {
            control_anchor(&mut bytes, anchor);
        }
        bytes.extend_from_slice(&(audio_requests.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&audio_requests);
        Ok(bytes)
    }
    /// Reconstruct only serialized mutable state. Resolution uses the declared
    /// Reference/catalog; no before/mine/default/live-RAM values are imported.
    pub fn decode_checkpoint(
        bytes: &[u8],
        required: &SessionStartRequirements,
    ) -> Result<Self, String> {
        let mut read = CheckpointReader(bytes);
        if read.bytes(10)? != b"SMCONTROL\x01" {
            return Err("unsupported Control checkpoint schema".into());
        }
        let identity = StartIdentity {
            schema: read.word()?,
            gameplay: control_identity(&mut read)?,
            reference: control_identity(&mut read)?,
            capture: control_identity(&mut read)?,
            media: Revision(control_identity(&mut read)?),
            committed_tick: read.counter()?,
            next_input_record: read.counter()?,
        };
        let reference_size =
            usize::try_from(read.counter()?).map_err(|_| "Control reference length overflow")?;
        if read.bytes(reference_size)? != audio_reference_bytes(&required.display.reference) {
            return Err("Control checkpoint Reference declaration changed".into());
        }
        if control_identity(&mut read)? != required.display.samus_content.revision.0 {
            return Err("Control checkpoint Samus catalog revision changed".into());
        }
        let capture = control_identity(&mut read)?;
        let policy = control_identity(&mut read)?;
        let mut ports = Vec::with_capacity(2);
        for _ in 0..2 {
            ports.push(PortHistory {
                sampled: read.word()?,
                previous: read.word()?,
                pressed: read.word()?,
                repeated: read.word()?,
                repeat_counter: read.word()?,
            });
        }
        let input_history = InputHistory {
            capture,
            policy,
            ports: ports.try_into().map_err(|_| "Control port count")?,
            debug_second_port: control_bool(&mut read)?,
            repeat_first: read.word()?,
            repeat_next: read.word()?,
            released: read.word()?,
            timed_held: read.word()?,
            previous_timed_held: read.word()?,
            newly_held: read.word()?,
            held_timer: read.word()?,
            held_timer_reset: read.word()?,
            samus_held: read.word()?,
            samus_pressed: read.word()?,
            bindings: ActionBindings {
                shoot: read.word()?,
                jump: read.word()?,
                run: read.word()?,
                cancel: read.word()?,
                switch: read.word()?,
                aim_down: read.word()?,
                aim_up: read.word()?,
            },
            bug_fix_counter: read.word()?,
        };
        let capture = control_identity(&mut read)?;
        let observation_policy = control_identity(&mut read)?;
        let phase = match read.byte()? {
            0 => CommittedPhase::InitialObservation,
            1 => CommittedPhase::SettledInterval,
            _ => return Err("unknown Control continuation phase".into()),
        };
        let next_input_record = read.counter()?;
        let outer = read.byte()?;
        let primary = read.byte()?;
        let secondary = read.byte()?;
        let music = read.byte()?;
        let message = read.byte()?;
        let auxiliary = read.byte()?;
        let captured_mode = match read.byte()? {
            0 => None,
            1 => Some(read.word()?),
            _ => return Err("unknown Control captured mode option".into()),
        };
        let continuation = ContinuationHistory {
            capture,
            observation_policy,
            phase,
            next_input_record,
            outer,
            primary,
            secondary,
            music,
            message,
            auxiliary,
            captured_mode,
        };
        let capture = control_identity(&mut read)?;
        let revision = Revision(control_identity(&mut read)?);
        let timing_policy = control_identity(&mut read)?;
        let count = usize::try_from(read.counter()?)
            .map_err(|_| "Control audio declaration length overflow")?;
        if count != required.display.audio_programs.len() {
            return Err("Control audio declaration count changed".into());
        }
        let mut declarations = Vec::with_capacity(count);
        for _ in 0..count {
            declarations.push(AudioProgramDeclaration {
                program: AssetId(read.id()?),
                bank: AssetId(read.id()?),
                revision: Revision(control_identity(&mut read)?),
                definition: control_identity(&mut read)?,
            });
        }
        let music = match read.byte()? {
            0 => OngoingMusic::Silent,
            1 => OngoingMusic::Playing(control_read_anchor(&mut read)?),
            _ => return Err("unknown Control music anchor".into()),
        };
        let count =
            usize::try_from(read.counter()?).map_err(|_| "Control ambience length overflow")?;
        if count > read.0.len() / 84 {
            return Err("truncated Control ambience".into());
        }
        let mut ambience = Vec::with_capacity(count);
        for _ in 0..count {
            ambience.push(control_read_anchor(&mut read)?);
        }
        let audio = InitialAudioIntent {
            capture,
            revision,
            timing_policy,
            declarations: declarations.into(),
            music,
            ambience: ambience.into(),
        };
        let metadata = DisplayStartMetadata {
            identity,
            reference: required.display.reference,
            input_history,
            continuation,
            audio,
            samus_content: required.display.samus_content.clone(),
        };
        crate::scene::start::validate_control_metadata(&metadata, &required.display)
            .map_err(|error| format!("Control checkpoint {error:?}"))?;
        let requests_size =
            usize::try_from(read.counter()?).map_err(|_| "Control requests length overflow")?;
        let requests = AudioRequests::decode_checkpoint(
            read.bytes(requests_size)?,
            &metadata,
            required.audio_catalog.clone(),
        )?;
        if !read.0.is_empty() {
            return Err("trailing Control checkpoint fields".into());
        }
        Ok(Self::Owned(Box::new(CurrentControl { metadata, requests })))
    }
}

use crate::external_inputs::ExternalInputs;
use crate::player_transitions::state::{FrameGammaMode, TransitionObservations};

// One explicit field list serves both borrow directions; no Rust layout or RAM
// addresses enter this detached component. Full-u16 observations stay lossless.
macro_rules! checkpoint_observations {
    ($($field:ident),+ $(,)?) => {
            pub fn encode_observations_checkpoint(inputs: &ExternalInputs, metadata: &DisplayStartMetadata) -> Result<Vec<u8>, String> {
                if metadata.identity.reference != metadata.reference.fingerprint {
                    return Err("input checkpoint Reference identity mismatch".into());
                }
                let mut bytes = Vec::from(b"SMINPUT\x01".as_slice());
                let binding = observation_binding(metadata);
                bytes.extend_from_slice(&(binding.len() as u64).to_le_bytes());
                bytes.extend_from_slice(&binding);
                match inputs.transitions.frame_gamma {
                    FrameGammaMode::Unknown(word) => { bytes.push(1); bytes.extend_from_slice(&word.to_le_bytes()); }
                    mode => { bytes.push(0); bytes.extend_from_slice(&mode.to_raw().to_le_bytes()); }
                }
                for word in [inputs.transitions.debug_invincibility, inputs.transitions.moonwalk_enabled,
                    inputs.transitions.turn_request, inputs.transitions.game_state, inputs.transitions.game_time_frames,
                    inputs.transitions.fx_kind] { bytes.extend_from_slice(&word.to_le_bytes()); }
                bytes.push(inputs.nmi_counter_low);
                $(bytes.extend_from_slice(&inputs.$field.to_le_bytes());)+
                Ok(bytes)
            }
            pub fn decode_observations_checkpoint(bytes: &[u8], metadata: &DisplayStartMetadata) -> Result<ExternalInputs, String> {
                let mut read = CheckpointReader(bytes);
                if read.bytes(8)? != b"SMINPUT\x01" { return Err("unsupported input checkpoint schema".into()); }
                let binding_size = usize::try_from(read.counter()?).map_err(|_| "input checkpoint binding length overflow")?;
                if metadata.identity.reference != metadata.reference.fingerprint
                    || read.bytes(binding_size)? != observation_binding(metadata) {
                    return Err("input checkpoint source identity or Reference changed".into());
                }
                let tag = read.byte()?;
                let word = read.word()?;
                let frame_gamma = match tag {
                    0 => match FrameGammaMode::from_raw(word) {
                        FrameGammaMode::Unknown(_) => return Err("unknown tagged input continuation".into()),
                        mode => mode,
                    },
                    1 => FrameGammaMode::Unknown(word),
                    _ => return Err("unknown input continuation tag".into()),
                };
                let transitions = TransitionObservations { frame_gamma, debug_invincibility: read.word()?,
                    moonwalk_enabled: read.word()?, turn_request: read.word()?, game_state: read.word()?,
                    game_time_frames: read.word()?, fx_kind: read.word()? };
                let result = ExternalInputs { transitions, nmi_counter_low: read.byte()?, $($field: read.word()?,)+ };
                if !read.0.is_empty() { return Err("trailing input checkpoint fields".into()); }
                Ok(result)
            }
    };
}
checkpoint_observations!(
    debug_enemy_freeze,
    enemy_zero_health,
    blending_config,
    lava_y,
    current_enemy,
    random,
    random_enemy_counter,
    fx_y,
    drop_origin_x,
    drop_origin_y,
    shoot_button,
    jump_button,
    cancel_button,
    switch_button,
    run_button,
    aim_up_button,
    aim_down_button,
    held,
    pressed,
    filtered_held,
    filtered_pressed,
    frozen,
    cinematic,
    hud_auto_cancel,
    camera_x,
    background_page_x,
    power_bomb_active,
    camera_y,
    frame,
    ceres,
    m7_x,
    m7_y,
    m7_a,
    m7_b,
    m7_c,
    grapple_x,
    grapple_y,
);

use crate::session::{
    Controls, Echoes, PowerBomb, Preparation, Projectiles, Shot, Trail, TrailPair,
};

fn projectile_shot(bytes: &mut Vec<u8>, value: &Shot) {
    for word in [
        value.x,
        value.y,
        value.x_fraction,
        value.y_fraction,
        value.x_radius,
        value.y_radius,
        value.x_speed,
        value.y_speed,
        value.direction,
        value.kind,
        value.damage,
        value.cursor,
        value.remaining,
        value.motion,
        value.variable,
        value.trail_time,
        value.auxiliary,
        value.sprite,
    ] {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
}
fn projectile_read_shot(read: &mut CheckpointReader<'_>) -> Result<Shot, String> {
    Ok(Shot {
        x: read.word()?,
        y: read.word()?,
        x_fraction: read.word()?,
        y_fraction: read.word()?,
        x_radius: read.word()?,
        y_radius: read.word()?,
        x_speed: read.word()?,
        y_speed: read.word()?,
        direction: read.word()?,
        kind: read.word()?,
        damage: read.word()?,
        cursor: read.word()?,
        remaining: read.word()?,
        motion: read.word()?,
        variable: read.word()?,
        trail_time: read.word()?,
        auxiliary: read.word()?,
        sprite: read.word()?,
    })
}
fn projectile_trail(bytes: &mut Vec<u8>, value: &Trail) {
    for word in [value.remaining, value.cursor, value.tile, value.x, value.y] {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
}
fn projectile_read_trail(read: &mut CheckpointReader<'_>) -> Result<Trail, String> {
    Ok(Trail {
        remaining: read.word()?,
        cursor: read.word()?,
        tile: read.word()?,
        x: read.word()?,
        y: read.word()?,
    })
}
impl Projectiles {
    /// Complete current typed projectile fields. Interpretation of compatibility
    /// selectors stays with the supplied content owners; no media ID is inferred.
    pub fn encode_checkpoint(&self, metadata: &DisplayStartMetadata) -> Result<Vec<u8>, String> {
        if metadata.identity.reference != metadata.reference.fingerprint {
            return Err("projectile checkpoint Reference identity mismatch".into());
        }
        let mut bytes = Vec::from(b"SMSHOTS\x01".as_slice());
        let binding = observation_binding(metadata);
        bytes.extend_from_slice(&(binding.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&binding);
        for shot in &self.shots {
            projectile_shot(&mut bytes, shot);
        }
        for pair in &self.trails {
            projectile_trail(&mut bytes, &pair.left);
            projectile_trail(&mut bytes, &pair.right);
        }
        let value = &self.controls;
        for word in [
            value.cooldown,
            value.spread_charge,
            value.orbit_step,
            value.charge,
            value.previous_charge,
            value.glow,
            value.charge_palette,
            value.resume_sound,
            value.echo_sound,
            value.pending_aim,
            value.protection,
            value.selected_weapon,
            value.automatic_weapon,
            value.weapon_changed,
            value.cannon_flags,
            value.cannon_frame,
            value.cannon_drawing_mode,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        for word in value.flare_frames.iter().chain(&value.flare_timers) {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        for word in [
            self.selected,
            self.count,
            self.bomb_count,
            self.preparation.demo_held,
            self.preparation.demo_pressed,
            self.preparation.demo_filtered_held,
            self.preparation.demo_filtered_pressed,
            self.preparation.aim_history,
            self.preparation.previous_movement_and_facing,
            self.echoes.index,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        for word in self
            .echoes
            .x
            .iter()
            .chain(&self.echoes.y)
            .chain(&self.echoes.x_speed)
        {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        for word in [self.power_bomb.flag, self.power_bomb.x, self.power_bomb.y] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        for value in self.launch.displacements {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        Ok(bytes)
    }
    pub fn decode_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
    ) -> Result<Self, String> {
        let mut read = CheckpointReader(bytes);
        if read.bytes(8)? != b"SMSHOTS\x01" {
            return Err("unsupported projectile checkpoint schema".into());
        }
        let binding_size = usize::try_from(read.counter()?)
            .map_err(|_| "projectile checkpoint binding length overflow")?;
        if metadata.identity.reference != metadata.reference.fingerprint
            || read.bytes(binding_size)? != observation_binding(metadata)
        {
            return Err("projectile checkpoint source identity or Reference changed".into());
        }
        let mut shots = Vec::with_capacity(10);
        for _ in 0..10 {
            shots.push(projectile_read_shot(&mut read)?);
        }
        let mut trails = Vec::with_capacity(18);
        for _ in 0..18 {
            trails.push(TrailPair {
                left: projectile_read_trail(&mut read)?,
                right: projectile_read_trail(&mut read)?,
            });
        }
        let controls = Controls {
            cooldown: read.word()?,
            spread_charge: read.word()?,
            orbit_step: read.word()?,
            charge: read.word()?,
            previous_charge: read.word()?,
            glow: read.word()?,
            charge_palette: read.word()?,
            resume_sound: read.word()?,
            echo_sound: read.word()?,
            pending_aim: read.word()?,
            protection: read.word()?,
            selected_weapon: read.word()?,
            automatic_weapon: read.word()?,
            weapon_changed: read.word()?,
            cannon_flags: read.word()?,
            cannon_frame: read.word()?,
            cannon_drawing_mode: read.word()?,
            flare_frames: audio_words(&mut read)?,
            flare_timers: audio_words(&mut read)?,
        };
        let selected = read.word()?;
        let count = read.word()?;
        let bomb_count = read.word()?;
        let preparation = Preparation {
            demo_held: read.word()?,
            demo_pressed: read.word()?,
            demo_filtered_held: read.word()?,
            demo_filtered_pressed: read.word()?,
            aim_history: read.word()?,
            previous_movement_and_facing: read.word()?,
        };
        let echoes = Echoes {
            index: read.word()?,
            x: audio_words(&mut read)?,
            y: audio_words(&mut read)?,
            x_speed: audio_words(&mut read)?,
        };
        let power_bomb = PowerBomb {
            flag: read.word()?,
            x: read.word()?,
            y: read.word()?,
        };
        let mut displacements = Vec::with_capacity(4);
        for _ in 0..4 {
            displacements.push(i32::from_le_bytes(read.bytes(4)?.try_into().unwrap()));
        }
        if !read.0.is_empty() {
            return Err("trailing projectile checkpoint fields".into());
        }
        Ok(Self {
            shots: shots
                .try_into()
                .map_err(|_| "projectile checkpoint shot count")?,
            trails: trails
                .try_into()
                .map_err(|_| "projectile checkpoint trail count")?,
            controls,
            selected,
            count,
            bomb_count,
            preparation,
            echoes,
            power_bomb,
            launch: crate::launch_motion::LaunchMotion {
                displacements: displacements
                    .try_into()
                    .map_err(|_| "projectile checkpoint launch count")?,
            },
        })
    }
}

// Explicit typed field tables share the two directions of this existing codec.
// They do not use native layout, addresses, Default, or an earlier live owner.
macro_rules! checkpoint_fields {
    ($write:ident, $read:ident, $($kind:ident)::+, $( $field:ident: $encode:ident / $decode:ident ),+ $(,)?) => {
        fn $write(bytes: &mut Vec<u8>, value: &$($kind)::+) {
            $($encode(bytes, &value.$field);)+
        }
        fn $read(read: &mut CheckpointReader<'_>) -> Result<$($kind)::+, String> {
            Ok($($kind)::+ { $($field: $decode(read)?,)+ })
        }
    };
}
fn checkpoint_word(bytes: &mut Vec<u8>, value: &u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
fn checkpoint_read_word(read: &mut CheckpointReader<'_>) -> Result<u16, String> {
    read.word()
}
fn checkpoint_byte(bytes: &mut Vec<u8>, value: &u8) {
    bytes.push(*value);
}
fn checkpoint_read_byte(read: &mut CheckpointReader<'_>) -> Result<u8, String> {
    read.byte()
}
fn checkpoint_words<const N: usize>(bytes: &mut Vec<u8>, values: &[u16; N]) {
    for value in values {
        checkpoint_word(bytes, value);
    }
}
fn checkpoint_pair_motion(bytes: &mut Vec<u8>, values: &[crate::session::FixedMotion; 2]) {
    for value in values {
        checkpoint_motion(bytes, value);
    }
}
fn checkpoint_read_pair_motion(
    read: &mut CheckpointReader<'_>,
) -> Result<[crate::session::FixedMotion; 2], String> {
    Ok([checkpoint_read_motion(read)?, checkpoint_read_motion(read)?])
}
use crate::player_collision::state as collision;
use crate::player_transitions::state as transitions;
use crate::session::{ContactPhysics, FixedMotion, Player, Point, Position};
checkpoint_fields!(checkpoint_point, checkpoint_read_point, Point,
    x: checkpoint_word/checkpoint_read_word, y: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_position, checkpoint_read_position, Position,
    whole: checkpoint_word/checkpoint_read_word, fraction: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_motion, checkpoint_read_motion, FixedMotion,
    fraction: checkpoint_word/checkpoint_read_word, whole: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_demo, checkpoint_read_demo, crate::player_transitions::demo::DemoInput,
    pre_action: checkpoint_word/checkpoint_read_word, delay: checkpoint_word/checkpoint_read_word,
    program: checkpoint_word/checkpoint_read_word, held: checkpoint_word/checkpoint_read_word,
    pressed: checkpoint_word/checkpoint_read_word, enabled: checkpoint_word/checkpoint_read_word,
    previous_held: checkpoint_word/checkpoint_read_word, previous_pressed: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_pose_record, checkpoint_read_pose_record, collision::PoseRecord,
    pose: checkpoint_word/checkpoint_read_word, movement_and_facing: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_history, checkpoint_read_history, collision::PoseHistory,
    previous: checkpoint_pose_record/checkpoint_read_pose_record,
    last_different: checkpoint_pose_record/checkpoint_read_pose_record);
checkpoint_fields!(checkpoint_previous_position, checkpoint_read_previous_position, collision::PreviousPosition,
    x: checkpoint_position/checkpoint_read_position, y: checkpoint_position/checkpoint_read_position);
checkpoint_fields!(checkpoint_animation_clock, checkpoint_read_animation_clock, collision::AnimationClock,
    remaining: checkpoint_word/checkpoint_read_word, frame: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_collision, checkpoint_read_collision, collision::CollisionState,
    grapple_ejection: checkpoint_words/audio_words,
    flags: checkpoint_word/checkpoint_read_word, slope_adjusted: checkpoint_word/checkpoint_read_word,
    solid_actor_flags: checkpoint_word/checkpoint_read_word, pose_input: checkpoint_word/checkpoint_read_word,
    horizontal_slope_flags: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_clearance, checkpoint_read_clearance, collision::PoseClearance,
    actor_flags: checkpoint_word/checkpoint_read_word, terrain_flags: checkpoint_word/checkpoint_read_word,
    terrain_up: checkpoint_word/checkpoint_read_word, radius_difference: checkpoint_word/checkpoint_read_word,
    terrain_down: checkpoint_word/checkpoint_read_word, actor_up: checkpoint_word/checkpoint_read_word,
    actor_down: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_contact, checkpoint_read_contact, ContactPhysics,
    radius: checkpoint_word/checkpoint_read_word, x_radius: checkpoint_word/checkpoint_read_word,
    acceleration_mode: checkpoint_word/checkpoint_read_word, total_x: checkpoint_motion/checkpoint_read_motion,
    deceleration: checkpoint_word/checkpoint_read_word, momentum: checkpoint_word/checkpoint_read_word,
    speed_boost: checkpoint_word/checkpoint_read_word, extra_run: checkpoint_motion/checkpoint_read_motion,
    base_run: checkpoint_motion/checkpoint_read_motion, y_direction: checkpoint_word/checkpoint_read_word,
    y_speed: checkpoint_motion/checkpoint_read_motion, external_x: checkpoint_motion/checkpoint_read_motion,
    external_y: checkpoint_motion/checkpoint_read_motion, y_acceleration: checkpoint_motion/checkpoint_read_motion,
    auto_jump: checkpoint_word/checkpoint_read_word, collision_direction: checkpoint_word/checkpoint_read_word,
    contact_attack: checkpoint_word/checkpoint_read_word, reaction_point: checkpoint_word/checkpoint_read_word,
    sand_visits: checkpoint_word/checkpoint_read_word, movement_speed_table: checkpoint_word/checkpoint_read_word);

// The enum tag keeps Unknown(known_raw) distinct from the named variant.
macro_rules! checkpoint_mode {
    ($write:ident, $read:ident, $($kind:ident)::+) => {
        fn $write(bytes: &mut Vec<u8>, value: &$($kind)::+) {
            bytes.push(if matches!(value, $($kind)::+::Unknown(_)) { 1 } else { 0 });
            bytes.extend_from_slice(&value.to_raw().to_le_bytes());
        }
        fn $read(read: &mut CheckpointReader<'_>) -> Result<$($kind)::+, String> {
            let tag = read.byte()?;
            let raw = read.word()?;
            match tag {
                0 => match $($kind)::+::from_raw(raw) {
                    $($kind)::+::Unknown(_) => Err("unknown named player checkpoint mode".into()),
                    value => Ok(value),
                },
                1 => Ok($($kind)::+::Unknown(raw)),
                _ => Err("unknown player checkpoint mode tag".into()),
            }
        }
    };
}
checkpoint_mode!(
    checkpoint_movement_mode,
    checkpoint_read_movement_mode,
    transitions::MovementMode
);
checkpoint_mode!(
    checkpoint_draw_mode,
    checkpoint_read_draw_mode,
    transitions::DrawMode
);
checkpoint_mode!(
    checkpoint_input_mode,
    checkpoint_read_input_mode,
    transitions::InputMode
);
checkpoint_mode!(
    checkpoint_alpha_mode,
    checkpoint_read_alpha_mode,
    transitions::FrameAlphaMode
);
checkpoint_mode!(
    checkpoint_beta_mode,
    checkpoint_read_beta_mode,
    transitions::FrameBetaMode
);
checkpoint_mode!(
    checkpoint_gamma_mode,
    checkpoint_read_gamma_mode,
    transitions::FrameGammaMode
);
macro_rules! checkpoint_selector {
    ($write:ident, $read:ident, $($kind:ident)::+) => {
        fn $write(bytes: &mut Vec<u8>, value: &$($kind)::+) {
            bytes.extend_from_slice(&value.to_raw().to_le_bytes());
        }
        fn $read(read: &mut CheckpointReader<'_>) -> Result<$($kind)::+, String> {
            Ok($($kind)::+::from_raw(read.word()?))
        }
    };
}
checkpoint_selector!(
    checkpoint_pose_request,
    checkpoint_read_pose_request,
    transitions::PoseRequestWord
);
checkpoint_selector!(
    checkpoint_momentum,
    checkpoint_read_momentum,
    transitions::MomentumAction
);
checkpoint_selector!(
    checkpoint_interruption,
    checkpoint_read_interruption,
    transitions::InterruptionAction
);
checkpoint_selector!(
    checkpoint_hurt,
    checkpoint_read_hurt,
    transitions::HurtAction
);
checkpoint_selector!(
    checkpoint_animation_skip,
    checkpoint_read_animation_skip,
    transitions::AnimationSkip
);
checkpoint_selector!(
    checkpoint_bomb_jump,
    checkpoint_read_bomb_jump,
    transitions::BombJump
);
checkpoint_selector!(
    checkpoint_bounce,
    checkpoint_read_bounce,
    transitions::LandingBounce
);
checkpoint_fields!(checkpoint_transitions, checkpoint_read_transitions, transitions::PlayerTransitions,
    ordinary: checkpoint_pose_request/checkpoint_read_pose_request,
    interrupted: checkpoint_pose_request/checkpoint_read_pose_request,
    transitional: checkpoint_pose_request/checkpoint_read_pose_request,
    momentum: checkpoint_momentum/checkpoint_read_momentum,
    interruption: checkpoint_interruption/checkpoint_read_interruption,
    hurt: checkpoint_hurt/checkpoint_read_hurt,
    animation_skip: checkpoint_animation_skip/checkpoint_read_animation_skip,
    landing_compatibility: checkpoint_word/checkpoint_read_word,
    frame_alpha: checkpoint_alpha_mode/checkpoint_read_alpha_mode,
    frame_beta: checkpoint_beta_mode/checkpoint_read_beta_mode,
    frame_gamma: checkpoint_gamma_mode/checkpoint_read_gamma_mode,
    draw_handler: checkpoint_draw_mode/checkpoint_read_draw_mode,
    knockback: checkpoint_word/checkpoint_read_word,
    bomb_jump: checkpoint_bomb_jump/checkpoint_read_bomb_jump,
    input_mode: checkpoint_input_mode/checkpoint_read_input_mode,
    xray_angle: checkpoint_word/checkpoint_read_word,
    animation_buffer: checkpoint_word/checkpoint_read_word,
    grapple_walljump_timer: checkpoint_word/checkpoint_read_word,
    elevator_fade_timer: checkpoint_word/checkpoint_read_word,
    shinespark_start_stop_timer: checkpoint_word/checkpoint_read_word,
    crystal_flash_ammo: checkpoint_word/checkpoint_read_word,
    liquid_physics: checkpoint_word/checkpoint_read_word,
    landing_motion_compatibility: checkpoint_word/checkpoint_read_word,
    bounce: checkpoint_bounce/checkpoint_read_bounce,
    falling: checkpoint_word/checkpoint_read_word,
    vertical_motion_compatibility: checkpoint_word/checkpoint_read_word,
    vertical_auxiliary_compatibility: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_player, checkpoint_read_player, Player,
    demo: checkpoint_demo/checkpoint_read_demo,
    transitions: checkpoint_transitions/checkpoint_read_transitions,
    x_fraction: checkpoint_word/checkpoint_read_word, y_fraction: checkpoint_word/checkpoint_read_word,
    grapple_endpoint: checkpoint_point/checkpoint_read_point,
    grapple_flags: checkpoint_word/checkpoint_read_word, grapple_interaction_flags: checkpoint_word/checkpoint_read_word,
    grapple_extension_compatibility: checkpoint_point/checkpoint_read_point,
    grapple_direction: checkpoint_word/checkpoint_read_word, grapple_direction_compatibility: checkpoint_word/checkpoint_read_word,
    grapple_delay: checkpoint_word/checkpoint_read_word, grapple_slow_scrolling: checkpoint_word/checkpoint_read_word,
    grapple_angle: checkpoint_word/checkpoint_read_word, grapple_length: checkpoint_word/checkpoint_read_word,
    grapple_origin_offset: checkpoint_point/checkpoint_read_point,
    grapple_endpoint_fraction: checkpoint_point/checkpoint_read_point,
    grapple_offset: checkpoint_pair_motion/checkpoint_read_pair_motion,
    grapple_velocity: checkpoint_point/checkpoint_read_point,
    grapple_animation_delay: checkpoint_words/audio_words, grapple_animation_program: checkpoint_words/audio_words,
    periodic_damage_fraction: checkpoint_word/checkpoint_read_word,
    x: checkpoint_word/checkpoint_read_word, y: checkpoint_word/checkpoint_read_word,
    body_position: checkpoint_point/checkpoint_read_point,
    pose: checkpoint_word/checkpoint_read_word, previous_health_for_flash: checkpoint_word/checkpoint_read_word,
    top_sprite: checkpoint_word/checkpoint_read_word, bottom_sprite: checkpoint_word/checkpoint_read_word,
    scripted_movement_direction: checkpoint_word/checkpoint_read_word,
    crouch_transition_work: checkpoint_word/checkpoint_read_word,
    history: checkpoint_history/checkpoint_read_history,
    previous_position: checkpoint_previous_position/checkpoint_read_previous_position,
    camera_distance: checkpoint_pair_motion/checkpoint_read_pair_motion,
    animation_clock: checkpoint_animation_clock/checkpoint_read_animation_clock,
    collision: checkpoint_collision/checkpoint_read_collision,
    clearance: checkpoint_clearance/checkpoint_read_clearance,
    movement_and_facing: checkpoint_word/checkpoint_read_word, grapple: checkpoint_word/checkpoint_read_word,
    movement_handler: checkpoint_movement_mode/checkpoint_read_movement_mode,
    shine_timer: checkpoint_word/checkpoint_read_word, shine_tick: checkpoint_word/checkpoint_read_word,
    palette_frame: checkpoint_word/checkpoint_read_word, special_palette_flags: checkpoint_word/checkpoint_read_word,
    contact: checkpoint_contact/checkpoint_read_contact,
    invincibility: checkpoint_word/checkpoint_read_word, periodic_damage: checkpoint_word/checkpoint_read_word,
    speed_divisor: checkpoint_word/checkpoint_read_word, knockback_timer: checkpoint_word/checkpoint_read_word,
    knockback_x_direction: checkpoint_word/checkpoint_read_word,
    swing: checkpoint_swing/checkpoint_read_swing);
checkpoint_fields!(checkpoint_swing, checkpoint_read_swing, crate::grapple_swing::SwingState,
    mirrored_angle: checkpoint_word/checkpoint_read_word,
    length_delta: checkpoint_word/checkpoint_read_word,
    start: checkpoint_point/checkpoint_read_point,
    angular_velocity: checkpoint_word/checkpoint_read_word,
    angle_acceleration: checkpoint_word/checkpoint_read_word,
    input_acceleration: checkpoint_word/checkpoint_read_word,
    rising_deceleration: checkpoint_word/checkpoint_read_word,
    extra_angular_velocity: checkpoint_word/checkpoint_read_word,
    kick_cooldown: checkpoint_word/checkpoint_read_word,
    failed_movement_counter: checkpoint_word/checkpoint_read_word,
    start_marks: checkpoint_words/audio_words,
    probe: checkpoint_point/checkpoint_read_point,
    probe_block: checkpoint_point/checkpoint_read_point,
    start_animation_timer: checkpoint_word/checkpoint_read_word,
    start_animation_tiles: checkpoint_word/checkpoint_read_word);
impl Player {
    pub fn encode_checkpoint(&self, metadata: &DisplayStartMetadata) -> Result<Vec<u8>, String> {
        if metadata.identity.reference != metadata.reference.fingerprint {
            return Err("player checkpoint Reference identity mismatch".into());
        }
        let mut bytes = Vec::from(b"SMPLAYER\x03".as_slice());
        let binding = observation_binding(metadata);
        bytes.extend_from_slice(&(binding.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&binding);
        checkpoint_player(&mut bytes, self);
        Ok(bytes)
    }
    pub fn decode_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
    ) -> Result<Self, String> {
        let mut read = CheckpointReader(bytes);
        if read.bytes(9)? != b"SMPLAYER\x03" {
            return Err("unsupported player checkpoint schema".into());
        }
        let size = usize::try_from(read.counter()?)
            .map_err(|_| "player checkpoint binding length overflow")?;
        if metadata.identity.reference != metadata.reference.fingerprint
            || read.bytes(size)? != observation_binding(metadata)
        {
            return Err("player checkpoint source identity or Reference changed".into());
        }
        let player = checkpoint_read_player(&mut read)?;
        if !read.0.is_empty() {
            return Err("trailing player checkpoint fields".into());
        }
        Ok(player)
    }
}

use crate::session::{DropMetadata, EnemyProjectile, EnemyProjectiles, Rectangle, SpawnContext};
checkpoint_fields!(checkpoint_rectangle, checkpoint_read_rectangle, Rectangle,
    origin: checkpoint_point/checkpoint_read_point, extent: checkpoint_point/checkpoint_read_point);
checkpoint_fields!(checkpoint_drop, checkpoint_read_drop, DropMetadata,
    flags: checkpoint_word/checkpoint_read_word, enemy_definition: checkpoint_word/checkpoint_read_word,
    killed_enemy: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_enemy_projectile, checkpoint_read_enemy_projectile, EnemyProjectile,
    definition: checkpoint_word/checkpoint_read_word, graphics: checkpoint_word/checkpoint_read_word,
    counter: checkpoint_word/checkpoint_read_word, pre_action: checkpoint_word/checkpoint_read_word,
    x: checkpoint_position/checkpoint_read_position, y: checkpoint_position/checkpoint_read_position,
    velocity: checkpoint_point/checkpoint_read_point,
    behavior_a: checkpoint_word/checkpoint_read_word, behavior_b: checkpoint_word/checkpoint_read_word,
    program: checkpoint_word/checkpoint_read_word, spritemap: checkpoint_word/checkpoint_read_word,
    delay: checkpoint_word/checkpoint_read_word, radius: checkpoint_word/checkpoint_read_word,
    properties: checkpoint_word/checkpoint_read_word, behavior_c: checkpoint_word/checkpoint_read_word,
    angle: checkpoint_word/checkpoint_read_word, drop: checkpoint_drop/checkpoint_read_drop);
checkpoint_fields!(checkpoint_spawn_context, checkpoint_read_spawn_context, SpawnContext,
    primary: checkpoint_word/checkpoint_read_word, secondary: checkpoint_word/checkpoint_read_word,
    angle: checkpoint_byte/checkpoint_read_byte,
    rectangle: checkpoint_rectangle/checkpoint_read_rectangle,
    variant: checkpoint_word/checkpoint_read_word, point: checkpoint_point/checkpoint_read_point,
    drop_definition: checkpoint_word/checkpoint_read_word);
impl EnemyProjectiles {
    pub fn encode_checkpoint(&self, metadata: &DisplayStartMetadata) -> Result<Vec<u8>, String> {
        if metadata.identity.reference != metadata.reference.fingerprint {
            return Err("enemy projectile checkpoint Reference identity mismatch".into());
        }
        let mut bytes = Vec::from(b"SMESHOT\x01".as_slice());
        let binding = observation_binding(metadata);
        bytes.extend_from_slice(&(binding.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&binding);
        for slot in &self.slots {
            checkpoint_enemy_projectile(&mut bytes, slot);
        }
        checkpoint_word(&mut bytes, &self.flags);
        checkpoint_word(&mut bytes, &self.selected);
        checkpoint_spawn_context(&mut bytes, &self.spawn);
        checkpoint_word(&mut bytes, &self.drop_bias);
        Ok(bytes)
    }
    pub fn decode_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
    ) -> Result<Self, String> {
        let mut read = CheckpointReader(bytes);
        if read.bytes(8)? != b"SMESHOT\x01" {
            return Err("unsupported enemy projectile checkpoint schema".into());
        }
        let size = usize::try_from(read.counter()?)
            .map_err(|_| "enemy projectile checkpoint binding length overflow")?;
        if metadata.identity.reference != metadata.reference.fingerprint
            || read.bytes(size)? != observation_binding(metadata)
        {
            return Err("enemy projectile checkpoint source identity or Reference changed".into());
        }
        let mut slots = Vec::with_capacity(18);
        for _ in 0..18 {
            slots.push(checkpoint_read_enemy_projectile(&mut read)?);
        }
        let result = Self {
            slots: slots
                .try_into()
                .map_err(|_| "enemy projectile checkpoint slot count")?,
            flags: read.word()?,
            selected: read.word()?,
            spawn: checkpoint_read_spawn_context(&mut read)?,
            drop_bias: read.word()?,
        };
        if !read.0.is_empty() {
            return Err("trailing enemy projectile checkpoint fields".into());
        }
        Ok(result)
    }
}

use crate::gameplay_flag_state::{FlagCompatibility, GameplayFlags};
use crate::item_pickups::{Inventory, Loadout, Supply};
use crate::session::{Feedback, Mechanism, Mechanisms, MessageRequests, SpawnScratch, Timing};
use crate::sprite_effect_state::{SpriteCompatibilityTails, SpriteEffect, SpriteEffects};
fn checkpoint_bytes<const N: usize>(bytes: &mut Vec<u8>, values: &[u8; N]) {
    bytes.extend_from_slice(values);
}
fn checkpoint_read_bytes<const N: usize>(
    read: &mut CheckpointReader<'_>,
) -> Result<[u8; N], String> {
    Ok(read.bytes(N)?.try_into().unwrap())
}
checkpoint_fields!(checkpoint_loadout, checkpoint_read_loadout, Loadout,
    owned: checkpoint_word/checkpoint_read_word, active: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_supply, checkpoint_read_supply, Supply,
    current: checkpoint_word/checkpoint_read_word, maximum: checkpoint_word/checkpoint_read_word);
fn checkpoint_ammunition(bytes: &mut Vec<u8>, values: &[Supply; 3]) {
    for value in values {
        checkpoint_supply(bytes, value);
    }
}
fn checkpoint_read_ammunition(read: &mut CheckpointReader<'_>) -> Result<[Supply; 3], String> {
    Ok([
        checkpoint_read_supply(read)?,
        checkpoint_read_supply(read)?,
        checkpoint_read_supply(read)?,
    ])
}
checkpoint_fields!(checkpoint_inventory, checkpoint_read_inventory, Inventory,
    equipment: checkpoint_loadout/checkpoint_read_loadout, beams: checkpoint_loadout/checkpoint_read_loadout,
    hyper_beam: checkpoint_word/checkpoint_read_word, health: checkpoint_supply/checkpoint_read_supply,
    health_fraction: checkpoint_word/checkpoint_read_word, reserve_capacity: checkpoint_word/checkpoint_read_word,
    reserve_mode: checkpoint_word/checkpoint_read_word, reserve_health: checkpoint_word/checkpoint_read_word,
    reserve_missiles: checkpoint_word/checkpoint_read_word, ammunition: checkpoint_ammunition/checkpoint_read_ammunition);
fn checkpoint_minimap_tiles(bytes: &mut Vec<u8>, values: &[[u16; 5]; 3]) {
    for value in values {
        checkpoint_words(bytes, value);
    }
}
fn checkpoint_read_minimap_tiles(read: &mut CheckpointReader<'_>) -> Result<[[u16; 5]; 3], String> {
    Ok([audio_words(read)?, audio_words(read)?, audio_words(read)?])
}
checkpoint_fields!(checkpoint_minimap, checkpoint_read_minimap, crate::player_update::state::State,
    explored: checkpoint_bytes/checkpoint_read_bytes, tiles: checkpoint_minimap_tiles/checkpoint_read_minimap_tiles,
    disabled: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_mechanism, checkpoint_read_mechanism, Mechanism,
    header: checkpoint_word/checkpoint_read_word, block: checkpoint_word/checkpoint_read_word,
    pre_action: checkpoint_word/checkpoint_read_word, program: checkpoint_word/checkpoint_read_word,
    counter: checkpoint_word/checkpoint_read_word, room_argument: checkpoint_word/checkpoint_read_word,
    work0: checkpoint_word/checkpoint_read_word, instruction_delay: checkpoint_word/checkpoint_read_word,
    draw_program: checkpoint_word/checkpoint_read_word, return_program: checkpoint_word/checkpoint_read_word,
    work1: checkpoint_word/checkpoint_read_word);
fn checkpoint_mechanism_slots(bytes: &mut Vec<u8>, values: &[Mechanism; 40]) {
    for value in values {
        checkpoint_mechanism(bytes, value);
    }
}
fn checkpoint_read_mechanism_slots(
    read: &mut CheckpointReader<'_>,
) -> Result<[Mechanism; 40], String> {
    let mut values = Vec::with_capacity(40);
    for _ in 0..40 {
        values.push(checkpoint_read_mechanism(read)?);
    }
    values
        .try_into()
        .map_err(|_| "mechanism checkpoint slot count".into())
}
checkpoint_fields!(checkpoint_mechanisms, checkpoint_read_mechanisms, Mechanisms,
    slots: checkpoint_mechanism_slots/checkpoint_read_mechanism_slots,
    flags: checkpoint_word/checkpoint_read_word, draw_cursor: checkpoint_word/checkpoint_read_word,
    selected: checkpoint_word/checkpoint_read_word, block_x: checkpoint_word/checkpoint_read_word,
    block_y: checkpoint_word/checkpoint_read_word, next_graphics: checkpoint_word/checkpoint_read_word,
    saved_graphics: checkpoint_words/audio_words, tilemap: checkpoint_words/audio_words,
    custom_draw: checkpoint_words/audio_words, save_locked: checkpoint_word/checkpoint_read_word,
    statue_finished: checkpoint_word/checkpoint_read_word, statue_animation: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_flags, checkpoint_read_flags, GameplayFlags,
    events: checkpoint_bytes/checkpoint_read_bytes, bosses_by_area: checkpoint_bytes/checkpoint_read_bytes,
    used_room_objects: checkpoint_bytes/checkpoint_read_bytes, collected_locations: checkpoint_bytes/checkpoint_read_bytes,
    opened_doors: checkpoint_bytes/checkpoint_read_bytes);
checkpoint_fields!(checkpoint_flag_compatibility, checkpoint_read_flag_compatibility, FlagCompatibility,
    bitmask: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_sprite, checkpoint_read_sprite, SpriteEffect,
    instruction: checkpoint_word/checkpoint_read_word, timer: checkpoint_word/checkpoint_read_word,
    appearance: checkpoint_word/checkpoint_read_word, x: checkpoint_position/checkpoint_read_position,
    y: checkpoint_position/checkpoint_read_position, disable_flags: checkpoint_word/checkpoint_read_word);
fn checkpoint_sprites(bytes: &mut Vec<u8>, values: &[SpriteEffect; 32]) {
    for value in values {
        checkpoint_sprite(bytes, value);
    }
}
fn checkpoint_read_sprites(read: &mut CheckpointReader<'_>) -> Result<[SpriteEffect; 32], String> {
    let mut values = Vec::with_capacity(32);
    for _ in 0..32 {
        values.push(checkpoint_read_sprite(read)?);
    }
    values
        .try_into()
        .map_err(|_| "sprite checkpoint slot count".into())
}
checkpoint_fields!(checkpoint_sprite_effects, checkpoint_read_sprite_effects, SpriteEffects,
    active: checkpoint_sprites/checkpoint_read_sprites, selected: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_sprite_tails, checkpoint_read_sprite_tails, SpriteCompatibilityTails,
    instruction: checkpoint_words/audio_words, timer: checkpoint_words/audio_words,
    appearance: checkpoint_words/audio_words, x: checkpoint_words/audio_words,
    x_fraction: checkpoint_words/audio_words, y: checkpoint_words/audio_words,
    y_fraction: checkpoint_words/audio_words, disable_flags: checkpoint_words/audio_words);
checkpoint_fields!(checkpoint_feedback, checkpoint_read_feedback, Feedback,
    earthquake_kind: checkpoint_word/checkpoint_read_word, earthquake_timer: checkpoint_word/checkpoint_read_word,
    sound_disabled: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_messages, checkpoint_read_messages, MessageRequests,
    queued: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_spawn_scratch, checkpoint_read_spawn_scratch, SpawnScratch,
    header: checkpoint_word/checkpoint_read_word, coordinates: checkpoint_word/checkpoint_read_word,
    argument: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_timing, checkpoint_read_timing, Timing,
    frame_counter: checkpoint_word/checkpoint_read_word, random_number: checkpoint_word/checkpoint_read_word,
    time_frozen: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_menu, checkpoint_read_menu, crate::menu_state::MenuState,
    menu_index: checkpoint_word/checkpoint_read_word, pausemenu_start_lr_pressed_highlight_timer: checkpoint_word/checkpoint_read_word, pausemenu_lr_animation_timer: checkpoint_word/checkpoint_read_word, pausemenu_item_selector_animation_timer: checkpoint_word/checkpoint_read_word, pausemenu_reserve_tank_animation_timer: checkpoint_word/checkpoint_read_word, unused_word_7e0731: checkpoint_word/checkpoint_read_word, pausemenu_uparrow_animation_timer: checkpoint_word/checkpoint_read_word, pausemenu_downarrow_animation_timer: checkpoint_word/checkpoint_read_word, pausemenu_rightarrow_animation_timer: checkpoint_word/checkpoint_read_word, pausemenu_leftarrow_animation_timer: checkpoint_word/checkpoint_read_word, pausemenu_palette_animation_timer: checkpoint_word/checkpoint_read_word, unused_word_7e073d: checkpoint_word/checkpoint_read_word, pausemenu_lr_animation_frame: checkpoint_word/checkpoint_read_word, pausemenu_item_selector_animation_frame: checkpoint_word/checkpoint_read_word, pausemenu_reserve_tank_animation_frame: checkpoint_word/checkpoint_read_word, unused_word_7e0745: checkpoint_word/checkpoint_read_word, pausemenu_uparrow_animation_frame: checkpoint_word/checkpoint_read_word, pausemenu_downarrow_animation_frame: checkpoint_word/checkpoint_read_word, pausemenu_rightarrow_animation_frame: checkpoint_word/checkpoint_read_word, pausemenu_leftarrow_animation_frame: checkpoint_word/checkpoint_read_word, pausemenu_palette_animation_frame: checkpoint_word/checkpoint_read_word, pausemenu_shoulder_button_highlight: checkpoint_word/checkpoint_read_word, pausemenu_button_label_mode: checkpoint_word/checkpoint_read_word, pausemenu_equipment_category_item: checkpoint_word/checkpoint_read_word, pausemenu_reserve_tank_delay_ctr: checkpoint_word/checkpoint_read_word, unused_word_7e0759: checkpoint_word/checkpoint_read_word, unused_word_7e075b: checkpoint_word/checkpoint_read_word, unused_word_7e075d: checkpoint_word/checkpoint_read_word, unused_word_7e075f: checkpoint_word/checkpoint_read_word, unused_word_7e0761: checkpoint_word/checkpoint_read_word, pause_screen_mode: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_flow, checkpoint_read_flow, crate::flow::Flow,
    cinematic_function: checkpoint_word/checkpoint_read_word, demo_timer: checkpoint_word/checkpoint_read_word,
    demo_set: checkpoint_word/checkpoint_read_word, demo_scene: checkpoint_word/checkpoint_read_word,
    num_demo_sets: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_sequence, checkpoint_read_sequence, collision::SequenceWorkspace,
    stage: checkpoint_word/checkpoint_read_word, value: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_multiply, checkpoint_read_multiply, collision::MultiplyScratch,
    left: checkpoint_word/checkpoint_read_word, right: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_pointer, checkpoint_read_pointer, crate::session::PointerScratch,
    address: checkpoint_word/checkpoint_read_word, bank: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_botwoon_death, checkpoint_read_botwoon_death, crate::actors::botwoon_death::DeathProtocol,
    pending_hit: checkpoint_word/checkpoint_read_word, active: checkpoint_word/checkpoint_read_word,
    tail_ready: checkpoint_word/checkpoint_read_word, previous_health: checkpoint_word/checkpoint_read_word,
    body_landed: checkpoint_word/checkpoint_read_word);

// One complete subset table encodes from the actual Session's borrowed fields,
// then constructs detached fields solely from the bounded binary reader.
macro_rules! checkpoint_auxiliary {
    ($( $field:ident: $encode:ident / $decode:ident ),+ $(,)?) => {
        impl Session {
            pub fn encode_auxiliary_checkpoint(&self, metadata: &DisplayStartMetadata) -> Result<Vec<u8>, String> {
                if metadata.identity.reference != metadata.reference.fingerprint {
                    return Err("auxiliary gameplay checkpoint Reference identity mismatch".into());
                }
                let mut bytes = Vec::from(b"SMAUXGS\x06".as_slice());
                let binding = observation_binding(metadata);
                bytes.extend_from_slice(&(binding.len() as u64).to_le_bytes());
                bytes.extend_from_slice(&binding);
                $($encode(&mut bytes, &self.$field);)+
                Ok(bytes)
            }
            pub fn decode_auxiliary_checkpoint(bytes: &[u8], metadata: &DisplayStartMetadata) -> Result<AuxiliaryGameplayCheckpoint, String> {
                let mut read = CheckpointReader(bytes);
                if read.bytes(8)? != b"SMAUXGS\x06" { return Err("unsupported auxiliary gameplay checkpoint schema".into()); }
                let size = usize::try_from(read.counter()?).map_err(|_| "auxiliary gameplay checkpoint binding length overflow")?;
                if metadata.identity.reference != metadata.reference.fingerprint || read.bytes(size)? != observation_binding(metadata) {
                    return Err("auxiliary gameplay checkpoint source identity or Reference changed".into());
                }
                let result = AuxiliaryGameplayCheckpoint { $($field: $decode(&mut read)?,)+ };
                if !read.0.is_empty() { return Err("trailing auxiliary gameplay checkpoint fields".into()); }
                Ok(result)
            }
        }
    };
}
checkpoint_fields!(checkpoint_hdma_object, checkpoint_read_hdma_object, crate::hdma::HdmaObject,
    channels: checkpoint_word/checkpoint_read_word,
    bank_slot: checkpoint_word/checkpoint_read_word,
    instruction_list: checkpoint_word/checkpoint_read_word,
    table: checkpoint_word/checkpoint_read_word,
    instruction_timer: checkpoint_word/checkpoint_read_word,
    pre_instruction: checkpoint_word/checkpoint_read_word,
    pre_instruction_bank: checkpoint_word/checkpoint_read_word,
    timer: checkpoint_word/checkpoint_read_word,
    a: checkpoint_word/checkpoint_read_word,
    b: checkpoint_word/checkpoint_read_word,
    c: checkpoint_word/checkpoint_read_word,
    d: checkpoint_word/checkpoint_read_word,
    e: checkpoint_word/checkpoint_read_word,
    f: checkpoint_word/checkpoint_read_word);
fn checkpoint_hdma(bytes: &mut Vec<u8>, value: &crate::hdma::HdmaObjects) {
    checkpoint_word(bytes, &value.enabled);
    checkpoint_word(bytes, &value.selected);
    for object in &value.objects {
        checkpoint_hdma_object(bytes, object);
    }
}
fn checkpoint_read_hdma(
    read: &mut CheckpointReader<'_>,
) -> Result<crate::hdma::HdmaObjects, String> {
    let enabled = read.word()?;
    let selected = read.word()?;
    let mut objects = [crate::hdma::HdmaObject::default(); crate::hdma::OBJECTS];
    for object in &mut objects {
        *object = checkpoint_read_hdma_object(read)?;
    }
    Ok(crate::hdma::HdmaObjects {
        enabled,
        selected,
        objects,
    })
}
checkpoint_auxiliary!(
    botwoon_death: checkpoint_botwoon_death/checkpoint_read_botwoon_death,
    timing: checkpoint_timing/checkpoint_read_timing,
    flow: checkpoint_flow/checkpoint_read_flow,
    menu: checkpoint_menu/checkpoint_read_menu,
    gameplay_flags: checkpoint_flags/checkpoint_read_flags,
    flag_compatibility: checkpoint_flag_compatibility/checkpoint_read_flag_compatibility,
    sprite_effects: checkpoint_sprite_effects/checkpoint_read_sprite_effects,
    sprite_compatibility: checkpoint_sprite_tails/checkpoint_read_sprite_tails,
    minimap: checkpoint_minimap/checkpoint_read_minimap,
    inventory: checkpoint_inventory/checkpoint_read_inventory,
    mechanisms: checkpoint_mechanisms/checkpoint_read_mechanisms,
    feedback: checkpoint_feedback/checkpoint_read_feedback,
    hdma: checkpoint_hdma/checkpoint_read_hdma,
    messages: checkpoint_messages/checkpoint_read_messages,
    spawn_scratch: checkpoint_spawn_scratch/checkpoint_read_spawn_scratch,
    compatibility_enemy_scratch: checkpoint_word/checkpoint_read_word,
    shared_sequence: checkpoint_sequence/checkpoint_read_sequence,
    multiply_scratch: checkpoint_multiply/checkpoint_read_multiply,
    pointer_scratch: checkpoint_pointer/checkpoint_read_pointer);

use crate::room_entry::{content as room_content, state::RoomEntry, world};
use crate::session::{Cell, Room, TERRAIN_CELLS, TILE_DEFINITIONS, Terrain};
fn checkpoint_id32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
fn checkpoint_read_id32(read: &mut CheckpointReader<'_>) -> Result<u32, String> {
    Ok(u32::from_le_bytes(read.bytes(4)?.try_into().unwrap()))
}
fn checkpoint_count(bytes: &mut Vec<u8>, value: usize) {
    bytes.extend_from_slice(&(value as u64).to_le_bytes());
}
fn checkpoint_resource_option(bytes: &mut Vec<u8>, value: Option<world::ResourceId>) {
    match value {
        None => bytes.push(0),
        Some(id) => {
            bytes.push(1);
            checkpoint_id32(bytes, id.0);
        }
    }
}
/// Canonical authored topology and compatibility declarations, compared against
/// the independently supplied World. Attached content catalogs bind separately;
/// the process-local WorldRevision is not a portable file identity.
fn checkpoint_world_binding(binding: &world::WorldBinding) -> Result<Vec<u8>, String> {
    let value = binding
        .definition()
        .ok_or("room checkpoint requires authored World topology")?;
    let mut bytes = Vec::from(b"SMWORLD\x01".as_slice());
    checkpoint_count(&mut bytes, value.rooms.len());
    for (id, room) in &value.rooms {
        checkpoint_id32(&mut bytes, id.0);
        bytes.extend_from_slice(&[
            room.index,
            room.area,
            room.map_x,
            room.map_y,
            room.width,
            room.height,
            room.up_scroll,
            room.down_scroll,
            room.cre,
        ]);
        checkpoint_id32(&mut bytes, room.doors.0);
        checkpoint_count(&mut bytes, room.selectors.len());
        for selector in &room.selectors {
            match selector.condition {
                world::StateCondition::Default => bytes.push(0),
                world::StateCondition::Event(key) => bytes.extend_from_slice(&[1, key]),
                world::StateCondition::Boss(key) => bytes.extend_from_slice(&[2, key]),
                world::StateCondition::BossOne => bytes.push(3),
                world::StateCondition::MorphAndMissiles => bytes.push(4),
                world::StateCondition::PowerBombs => bytes.push(5),
            }
            checkpoint_id32(&mut bytes, selector.state.0);
        }
    }
    checkpoint_count(&mut bytes, value.states.len());
    for (id, state) in &value.states {
        checkpoint_id32(&mut bytes, id.0);
        checkpoint_id32(&mut bytes, state.tileset.0);
        checkpoint_id32(&mut bytes, state.level.0);
        bytes.extend_from_slice(&[state.music_data, state.music_track]);
        checkpoint_resource_option(&mut bytes, state.fx);
        checkpoint_id32(&mut bytes, state.enemy_population.0);
        checkpoint_id32(&mut bytes, state.enemy_tilesets.0);
        bytes.extend_from_slice(&state.layer2_policy);
        checkpoint_resource_option(&mut bytes, state.main_code);
        match &state.scrolls {
            world::ScrollDefinition::Generated(policy) => bytes.extend_from_slice(&[0, *policy]),
            world::ScrollDefinition::Cells(cells) => {
                bytes.push(1);
                checkpoint_count(&mut bytes, cells.len());
                bytes.extend_from_slice(cells);
            }
        }
        checkpoint_count(&mut bytes, state.objects.len());
        for object in &state.objects {
            bytes.extend_from_slice(&[object.x, object.y]);
            checkpoint_id32(&mut bytes, object.definition.0);
            checkpoint_word(&mut bytes, &object.argument);
        }
        checkpoint_count(&mut bytes, state.background.len());
        for command in &state.background {
            use world::BackgroundCommand as Command;
            let target = |target: &world::BackgroundTarget| match target {
                world::BackgroundTarget::Main => 0,
                world::BackgroundTarget::Secondary => 1,
                world::BackgroundTarget::Fx => 2,
            };
            match command {
                Command::Transfer {
                    source,
                    destination,
                    bytes: count,
                } => {
                    bytes.push(0);
                    checkpoint_id32(&mut bytes, source.0);
                    bytes.push(target(destination));
                    checkpoint_word(&mut bytes, count);
                }
                Command::Decompress {
                    source,
                    destination,
                } => {
                    bytes.push(1);
                    checkpoint_id32(&mut bytes, source.0);
                    bytes.push(target(destination));
                }
                Command::ClearFx => bytes.push(2),
                Command::ClearBackground => bytes.push(3),
                Command::ClearDualBackground => bytes.push(4),
                Command::TransferAndSetCopyPolicy {
                    source,
                    destination,
                    bytes: count,
                } => {
                    bytes.push(5);
                    checkpoint_id32(&mut bytes, source.0);
                    bytes.push(target(destination));
                    checkpoint_word(&mut bytes, count);
                }
                Command::MatchExitTransfer {
                    exit,
                    source,
                    destination,
                    bytes: count,
                } => {
                    bytes.push(6);
                    checkpoint_id32(&mut bytes, exit.0);
                    checkpoint_id32(&mut bytes, source.0);
                    bytes.push(target(destination));
                    checkpoint_word(&mut bytes, count);
                }
            }
        }
        checkpoint_resource_option(&mut bytes, state.setup);
    }
    checkpoint_count(&mut bytes, value.exits.len());
    for (id, exit) in &value.exits {
        checkpoint_id32(&mut bytes, id.0);
        checkpoint_id32(&mut bytes, exit.destination.0);
        let arrival = exit.arrival;
        bytes.extend_from_slice(&[
            arrival.flags,
            arrival.direction,
            arrival.x,
            arrival.y,
            arrival.close_x,
            arrival.close_y,
        ]);
        bytes.extend_from_slice(&arrival.distance.to_le_bytes());
        checkpoint_resource_option(&mut bytes, exit.setup);
        bytes.push(u8::from(exit.skip_cre_upload));
    }
    checkpoint_count(&mut bytes, value.starts.len());
    for (id, start) in &value.starts {
        checkpoint_id32(&mut bytes, id.0);
        checkpoint_id32(&mut bytes, start.room.0);
        checkpoint_id32(&mut bytes, start.exit.0);
        for word in [
            start.camera_x,
            start.camera_y,
            start.samus_x_offset,
            start.samus_y_offset,
        ] {
            checkpoint_word(&mut bytes, &word);
        }
    }
    checkpoint_count(&mut bytes, value.door_lists.len());
    for (id, doors) in &value.door_lists {
        checkpoint_id32(&mut bytes, id.0);
        checkpoint_count(&mut bytes, doors.len());
        for door in doors {
            checkpoint_id32(&mut bytes, door.0);
        }
    }
    checkpoint_count(&mut bytes, value.tilesets.len());
    for (id, tileset) in &value.tilesets {
        checkpoint_id32(&mut bytes, id.0);
        for resource in [tileset.tile_table, tileset.tiles, tileset.palette] {
            checkpoint_id32(&mut bytes, resource.0);
        }
    }
    checkpoint_count(&mut bytes, value.resources.len());
    for (id, kind) in &value.resources {
        checkpoint_id32(&mut bytes, id.0);
        bytes.push(match kind {
            world::ResourceKind::Level => 0,
            world::ResourceKind::TileTable => 1,
            world::ResourceKind::Tiles => 2,
            world::ResourceKind::Palette => 3,
            world::ResourceKind::Fx => 4,
            world::ResourceKind::EnemyPopulation => 5,
            world::ResourceKind::EnemyTilesets => 6,
            world::ResourceKind::ActorSprites => 7,
            world::ResourceKind::ActorTilemap => 8,
            world::ResourceKind::RoomObject => 9,
            world::ResourceKind::DoorSetup => 10,
            world::ResourceKind::RoomSetup => 11,
            world::ResourceKind::RoomMain => 12,
            world::ResourceKind::Background => 13,
        });
    }
    checkpoint_count(&mut bytes, value.closing_doors.len());
    for (key, resource) in &value.closing_doors {
        checkpoint_word(&mut bytes, key);
        checkpoint_resource_option(&mut bytes, *resource);
    }
    let keys = binding.compatibility_keys();
    checkpoint_count(&mut bytes, keys.starts.len());
    for (id, site) in &keys.starts {
        checkpoint_id32(&mut bytes, id.0);
        checkpoint_word(&mut bytes, &site.area);
        checkpoint_word(&mut bytes, &site.index);
    }
    // Distinct map order is part of the schema, even if numeric IDs overlap.
    macro_rules! identity_keys {
        ($map:expr) => {
            checkpoint_count(&mut bytes, $map.len());
            for (id, key) in $map {
                checkpoint_id32(&mut bytes, id.0);
                checkpoint_word(&mut bytes, key);
            }
        };
    }
    identity_keys!(&keys.rooms);
    identity_keys!(&keys.states);
    identity_keys!(&keys.exits);
    identity_keys!(&keys.door_lists);
    checkpoint_count(&mut bytes, keys.resources.len());
    for (id, key) in &keys.resources {
        checkpoint_id32(&mut bytes, id.0);
        match key {
            world::ResourceKey::Word { bank, offset } => {
                bytes.extend_from_slice(&[0, *bank]);
                checkpoint_word(&mut bytes, offset);
            }
            world::ResourceKey::Long(world::LegacyAddress24(address)) => {
                bytes.push(1);
                bytes.extend_from_slice(address);
            }
        }
    }
    Ok(bytes)
}
macro_rules! checkpoint_room_identity {
    ($write:ident, $read:ident, $id:ident) => {
        fn $write(bytes: &mut Vec<u8>, value: &world::Identity<world::$id>) {
            match value {
                world::Identity::Known(id) => {
                    bytes.push(0);
                    checkpoint_id32(bytes, id.0);
                }
                world::Identity::LegacyUnresolved(raw) => {
                    bytes.push(1);
                    checkpoint_word(bytes, raw);
                }
            }
        }
        fn $read(read: &mut CheckpointReader<'_>) -> Result<world::Identity<world::$id>, String> {
            match read.byte()? {
                0 => Ok(world::Identity::Known(world::$id(checkpoint_read_id32(
                    read,
                )?))),
                1 => Ok(world::Identity::LegacyUnresolved(read.word()?)),
                _ => Err("unknown room checkpoint identity tag".into()),
            }
        }
    };
}
checkpoint_room_identity!(checkpoint_room_id, checkpoint_read_room_id, RoomId);
checkpoint_room_identity!(checkpoint_exit_id, checkpoint_read_exit_id, ExitId);
checkpoint_room_identity!(checkpoint_state_id, checkpoint_read_state_id, StateId);
fn checkpoint_door_list(bytes: &mut Vec<u8>, value: &room_content::DoorList) {
    match value {
        room_content::DoorList::Legacy(raw) => {
            bytes.push(0);
            checkpoint_word(bytes, raw);
        }
        room_content::DoorList::Prepared(id) => {
            bytes.push(1);
            checkpoint_id32(bytes, id.0);
        }
    }
}
fn checkpoint_read_door_list(
    read: &mut CheckpointReader<'_>,
) -> Result<room_content::DoorList, String> {
    match read.byte()? {
        0 => Ok(room_content::DoorList::Legacy(read.word()?)),
        1 => Ok(room_content::DoorList::Prepared(world::DoorListId(
            checkpoint_read_id32(read)?,
        ))),
        _ => Err("unknown room checkpoint door list tag".into()),
    }
}
fn checkpoint_room_asset(bytes: &mut Vec<u8>, value: &room_content::AssetRef) {
    match value {
        room_content::AssetRef::Legacy(raw) => {
            bytes.push(0);
            bytes.extend_from_slice(raw);
        }
        room_content::AssetRef::Prepared(id) => {
            bytes.push(1);
            checkpoint_id32(bytes, id.0);
        }
    }
}
fn checkpoint_read_room_asset(
    read: &mut CheckpointReader<'_>,
) -> Result<room_content::AssetRef, String> {
    match read.byte()? {
        0 => Ok(room_content::AssetRef::Legacy(
            read.bytes(3)?.try_into().unwrap(),
        )),
        1 => Ok(room_content::AssetRef::Prepared(world::ResourceId(
            checkpoint_read_id32(read)?,
        ))),
        _ => Err("unknown room checkpoint asset tag".into()),
    }
}
checkpoint_fields!(checkpoint_room_entry, checkpoint_read_room_entry, RoomEntry,
    exit: checkpoint_exit_id/checkpoint_read_exit_id,
    direction: checkpoint_word/checkpoint_read_word, elevator_properties: checkpoint_word/checkpoint_read_word,
    elevator_interlock: checkpoint_word/checkpoint_read_word, enemy_interlock: checkpoint_word/checkpoint_read_word,
    elevator_direction: checkpoint_word/checkpoint_read_word,
    room: checkpoint_room_id/checkpoint_read_room_id,
    room_index: checkpoint_word/checkpoint_read_word, map_x: checkpoint_word/checkpoint_read_word,
    map_y: checkpoint_word/checkpoint_read_word, scroll_up: checkpoint_word/checkpoint_read_word,
    scroll_down: checkpoint_word/checkpoint_read_word, previous_cre: checkpoint_word/checkpoint_read_word,
    cre: checkpoint_word/checkpoint_read_word, doors: checkpoint_door_list/checkpoint_read_door_list,
    state: checkpoint_state_id/checkpoint_read_state_id,
    level: checkpoint_room_asset/checkpoint_read_room_asset, tile_table: checkpoint_room_asset/checkpoint_read_room_asset,
    tiles: checkpoint_room_asset/checkpoint_read_room_asset, palette: checkpoint_room_asset/checkpoint_read_room_asset,
    music_track: checkpoint_word/checkpoint_read_word, music_data: checkpoint_word/checkpoint_read_word,
    fx_key: checkpoint_word/checkpoint_read_word, enemy_population_key: checkpoint_word/checkpoint_read_word,
    enemy_tilesets_key: checkpoint_word/checkpoint_read_word, main_code_key: checkpoint_word/checkpoint_read_word,
    scrolling_hook: checkpoint_word/checkpoint_read_word,
    layer2_x: checkpoint_byte/checkpoint_read_byte, layer2_y: checkpoint_byte/checkpoint_read_byte,
    frame: checkpoint_word/checkpoint_read_word, arrival_x: checkpoint_word/checkpoint_read_word,
    arrival_y: checkpoint_word/checkpoint_read_word, speed_fraction: checkpoint_word/checkpoint_read_word,
    speed_whole: checkpoint_word/checkpoint_read_word, elevator_delay: checkpoint_word/checkpoint_read_word,
    completion: checkpoint_word/checkpoint_read_word, transition_selector: checkpoint_word/checkpoint_read_word,
    elevator_flags: checkpoint_word/checkpoint_read_word, elevator_status: checkpoint_word/checkpoint_read_word,
    irq_policy: checkpoint_word/checkpoint_read_word);
fn checkpoint_terrain(bytes: &mut Vec<u8>, value: &Terrain) {
    for cell in value.cells() {
        checkpoint_word(bytes, &cell.level);
        bytes.push(cell.behavior);
    }
    for tile in value.tiles() {
        checkpoint_words(bytes, tile);
    }
}
fn checkpoint_read_terrain(read: &mut CheckpointReader<'_>) -> Result<Terrain, String> {
    // Bound the complete allocation before allocating; capacities are physical,
    // independent of geometry and compatibility accesses beyond the room edge.
    let size = TERRAIN_CELLS * 3 + TILE_DEFINITIONS * 8;
    let mut terrain = CheckpointReader(read.bytes(size)?);
    let mut cells = Vec::with_capacity(TERRAIN_CELLS);
    for _ in 0..TERRAIN_CELLS {
        cells.push(Cell {
            level: terrain.word()?,
            behavior: terrain.byte()?,
        });
    }
    let mut tiles = Vec::with_capacity(TILE_DEFINITIONS);
    for _ in 0..TILE_DEFINITIONS {
        tiles.push(audio_words(&mut terrain)?);
    }
    Terrain::from_owned(cells.into_boxed_slice(), tiles.into_boxed_slice())
        .map_err(|error| format!("room checkpoint {error:?}"))
}
checkpoint_fields!(checkpoint_liquid, checkpoint_read_liquid, crate::liquid::Liquid,
    surface: checkpoint_position/checkpoint_read_position, lava_acid: checkpoint_position/checkpoint_read_position,
    tilemap: checkpoint_word/checkpoint_read_word, rising: checkpoint_word/checkpoint_read_word,
    kind: checkpoint_word/checkpoint_read_word, offset: checkpoint_position/checkpoint_read_position,
    tide_phase: checkpoint_word/checkpoint_read_word, base_fraction: checkpoint_word/checkpoint_read_word);
checkpoint_fields!(checkpoint_room, checkpoint_read_room, Room,
    entry: checkpoint_room_entry/checkpoint_read_room_entry, terrain: checkpoint_terrain/checkpoint_read_terrain,
    area: checkpoint_word/checkpoint_read_word, width: checkpoint_word/checkpoint_read_word,
    height: checkpoint_word/checkpoint_read_word, scroll_width: checkpoint_word/checkpoint_read_word,
    scroll_height: checkpoint_word/checkpoint_read_word, size_bytes: checkpoint_word/checkpoint_read_word,
    collision_cursor: checkpoint_word/checkpoint_read_word, liquid_options: checkpoint_word/checkpoint_read_word,
    liquid_height: checkpoint_word/checkpoint_read_word, liquid_target: checkpoint_word/checkpoint_read_word,
    liquid_velocity: checkpoint_word/checkpoint_read_word, liquid_delay: checkpoint_word/checkpoint_read_word,
    liquid: checkpoint_liquid/checkpoint_read_liquid,
    enemy_deaths: checkpoint_word/checkpoint_read_word, enemy_quota: checkpoint_word/checkpoint_read_word,
    loaded_items: checkpoint_word/checkpoint_read_word, power_bomb_radius: checkpoint_word/checkpoint_read_word,
    enemy_processing_stage: checkpoint_word/checkpoint_read_word, collision_detection_index: checkpoint_word/checkpoint_read_word,
    slope_behavior: checkpoint_word/checkpoint_read_word, main_variables: checkpoint_words/audio_words,
    camera: checkpoint_camera/checkpoint_read_camera);
checkpoint_fields!(checkpoint_camera, checkpoint_read_camera, crate::camera::Camera,
    layer1_block: checkpoint_words/audio_words, layer2_block: checkpoint_words/audio_words,
    previous_layer1_block: checkpoint_words/audio_words, previous_layer2_block: checkpoint_words/audio_words,
    bg1_block: checkpoint_words/audio_words, bg2_block: checkpoint_words/audio_words,
    layer1_fraction: checkpoint_words/audio_words, layer1: checkpoint_words/audio_words,
    layer2: checkpoint_words/audio_words, bg1_offset: checkpoint_words/audio_words,
    bg2_scroll: checkpoint_words/audio_words, ideal_layer1: checkpoint_words/audio_words);
fn checkpoint_validate_room(value: &Room, binding: &world::WorldBinding) -> Result<(), String> {
    let expected = binding
        .definition()
        .ok_or("room checkpoint requires authored World topology")?;
    if matches!(value.entry.room, world::Identity::Known(id) if !expected.rooms.contains_key(&id))
        || matches!(value.entry.state, world::Identity::Known(id) if !expected.states.contains_key(&id))
        || matches!(value.entry.exit, world::Identity::Known(id) if !expected.exits.contains_key(&id))
        || matches!(&value.entry.doors, room_content::DoorList::Prepared(id) if !expected.door_lists.contains_key(id))
    {
        return Err("room checkpoint missing authored World identity".into());
    }
    for (asset, kind) in [
        (&value.entry.level, world::ResourceKind::Level),
        (&value.entry.tile_table, world::ResourceKind::TileTable),
        (&value.entry.tiles, world::ResourceKind::Tiles),
        (&value.entry.palette, world::ResourceKind::Palette),
    ] {
        if let room_content::AssetRef::Prepared(id) = asset
            && (expected.resources.get(id) != Some(&kind)
                || !matches!(
                    binding.compatibility_keys().resources.get(id),
                    Some(world::ResourceKey::Long(_))
                ))
        {
            return Err("room checkpoint missing or mismatched authored asset".into());
        }
    }
    Ok(())
}
impl Room {
    pub fn encode_checkpoint(
        &self,
        metadata: &DisplayStartMetadata,
        binding: &world::WorldBinding,
    ) -> Result<Vec<u8>, String> {
        if metadata.identity.reference != metadata.reference.fingerprint {
            return Err("room checkpoint Reference identity mismatch".into());
        }
        let topology = checkpoint_world_binding(binding)?;
        checkpoint_validate_room(self, binding)?;
        let mut bytes = Vec::from(b"SMROOMS\x03".as_slice());
        let source = observation_binding(metadata);
        checkpoint_count(&mut bytes, source.len());
        bytes.extend_from_slice(&source);
        checkpoint_count(&mut bytes, topology.len());
        bytes.extend_from_slice(&topology);
        checkpoint_room(&mut bytes, self);
        Ok(bytes)
    }
    pub fn decode_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
        expected: &world::WorldBinding,
    ) -> Result<Self, String> {
        let mut read = CheckpointReader(bytes);
        if read.bytes(8)? != b"SMROOMS\x03" {
            return Err("unsupported room checkpoint schema".into());
        }
        let size = usize::try_from(read.counter()?)
            .map_err(|_| "room checkpoint source length overflow")?;
        if metadata.identity.reference != metadata.reference.fingerprint
            || read.bytes(size)? != observation_binding(metadata)
        {
            return Err("room checkpoint source identity or Reference changed".into());
        }
        let topology = checkpoint_world_binding(expected)?;
        let size = usize::try_from(read.counter()?)
            .map_err(|_| "room checkpoint World length overflow")?;
        if read.bytes(size)? != topology {
            return Err("room checkpoint authored World definition or keys changed".into());
        }
        let value = checkpoint_read_room(&mut read)?;
        if !read.0.is_empty() {
            return Err("trailing room checkpoint fields".into());
        }
        checkpoint_validate_room(&value, expected)?;
        Ok(value)
    }
}

/// Complete numeric gameplay, decoded without live RAM or a default Session.
/// Actor identity/media, Control, observations and Display remain separate owners
/// that the complete Session codec must reconstruct before validated installation.
pub struct NumericGameplayCheckpoint {
    actors: crate::actors::state::Actors,
    player: Player,
    projectiles: Projectiles,
    enemy_projectiles: crate::session::EnemyProjectiles,
    auxiliary: AuxiliaryGameplayCheckpoint,
    room: Room,
    world: Arc<world::WorldBinding>,
}
fn gameplay_component<'a>(read: &mut CheckpointReader<'a>) -> Result<&'a [u8], String> {
    let size = usize::try_from(read.counter()?)
        .map_err(|_| "gameplay checkpoint component length overflow")?;
    read.bytes(size)
}
impl Session {
    pub fn encode_gameplay_checkpoint(
        &self,
        metadata: &DisplayStartMetadata,
    ) -> Result<Vec<u8>, String> {
        // Fixed schema order; each reached codec binds the same independently
        // supplied source/Reference/Tick, and Room additionally seals World.
        let components = [
            self.actors.encode_numeric_checkpoint(metadata)?,
            self.player.encode_checkpoint(metadata)?,
            self.projectiles.encode_checkpoint(metadata)?,
            self.enemy_projectiles.encode_checkpoint(metadata)?,
            self.encode_auxiliary_checkpoint(metadata)?,
            self.room.encode_checkpoint(metadata, &self.world)?,
        ];
        let mut bytes = Vec::from(b"SMGAMEPLAY\x01".as_slice());
        for component in components {
            checkpoint_count(&mut bytes, component.len());
            bytes.extend_from_slice(&component);
        }
        Ok(bytes)
    }
    pub fn decode_gameplay_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
        world: Arc<world::WorldBinding>,
    ) -> Result<NumericGameplayCheckpoint, String> {
        let mut read = CheckpointReader(bytes);
        if read.bytes(11)? != b"SMGAMEPLAY\x01" {
            return Err("unsupported gameplay checkpoint schema".into());
        }
        let actors = crate::actors::state::Actors::decode_numeric_checkpoint(
            gameplay_component(&mut read)?,
            metadata,
        )?;
        let player = Player::decode_checkpoint(gameplay_component(&mut read)?, metadata)?;
        let projectiles = Projectiles::decode_checkpoint(gameplay_component(&mut read)?, metadata)?;
        let enemy_projectiles = crate::session::EnemyProjectiles::decode_checkpoint(
            gameplay_component(&mut read)?,
            metadata,
        )?;
        let auxiliary =
            Self::decode_auxiliary_checkpoint(gameplay_component(&mut read)?, metadata)?;
        let room = Room::decode_checkpoint(gameplay_component(&mut read)?, metadata, &world)?;
        if !read.0.is_empty() {
            return Err("trailing gameplay checkpoint components".into());
        }
        Ok(NumericGameplayCheckpoint {
            actors,
            player,
            projectiles,
            enemy_projectiles,
            auxiliary,
            room,
            world,
        })
    }
}
impl Session {
    pub fn encode_actor_identity_checkpoint(
        &self,
        metadata: &DisplayStartMetadata,
        images: CheckpointImages<'_>,
    ) -> Result<Vec<u8>, String> {
        self.actor_identity
            .encode_owned_checkpoint(&self.actors, &self.world, metadata, images)
    }
}
impl NumericGameplayCheckpoint {
    pub fn decode_actor_identity(
        &self,
        bytes: &[u8],
        current: &crate::actors::identity::ActorIdentityState,
        metadata: &DisplayStartMetadata,
        images: CheckpointImages<'_>,
    ) -> Result<crate::actors::identity::ActorIdentityState, String> {
        crate::actors::identity::ActorIdentityState::decode_owned_checkpoint(
            bytes,
            current,
            &self.actors,
            &self.world,
            metadata,
            images,
        )
    }
    /// Constructs a detached replacement from decoded numeric fields and the
    /// other explicitly reconstructed owners. This does not install or adopt it.
    pub fn into_session(
        self,
        actor_identity: crate::actors::identity::ActorIdentityState,
        control: crate::session_owners::Control,
        presentation: crate::session_owners::Presentation,
    ) -> Session {
        let AuxiliaryGameplayCheckpoint {
            botwoon_death,
            timing,
            flow,
            menu,
            gameplay_flags,
            flag_compatibility,
            sprite_effects,
            sprite_compatibility,
            minimap,
            inventory,
            mechanisms,
            feedback,
            hdma,
            messages,
            spawn_scratch,
            compatibility_enemy_scratch,
            shared_sequence,
            multiply_scratch,
            pointer_scratch,
        } = self.auxiliary;
        // Exhaustive construction also makes future Session/auxiliary fields a
        // compiler error until their checkpoint participation is explicit.
        Session {
            actor_identity,
            actors: self.actors,
            botwoon_death,
            timing,
            flow,
            menu,
            world: self.world,
            control,
            gameplay_flags,
            flag_compatibility,
            sprite_effects,
            sprite_compatibility,
            presentation,
            projectiles: self.projectiles,
            enemy_projectiles: self.enemy_projectiles,
            player: self.player,
            minimap,
            inventory,
            mechanisms,
            room: self.room,
            feedback,
            hdma,
            messages,
            spawn_scratch,
            compatibility_enemy_scratch,
            shared_sequence,
            multiply_scratch,
            pointer_scratch,
        }
    }
}
