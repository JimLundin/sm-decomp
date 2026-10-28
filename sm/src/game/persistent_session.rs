//! Supplied Session/display preparation and the exclusive presentation owner.
#![forbid(unsafe_code)]

use std::{collections::BTreeMap, sync::Arc};

use crate::game::external_inputs::ExternalInputs;
use crate::game::scene::{
    component::{DisplayComponent, DisplayStartMetadata},
    records::SceneMedia,
    samus_body::MarkerPublication,
    samus_content::ContentDescription,
    start::{DisplayStartDescription, DisplayStartRequirements, StartError, ValidatedDisplayStart},
    state::DisplayError,
};
use crate::game::session::Session;
use crate::game_state::{Game, PreparedSamusBinding, SamusBindingDescription, SamusExecution};

/// Independently supplied compatibility layout for the eight current actor queues.
/// Offsets are word indices in the original 114-word queue workspace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActorQueueProjection {
    pub first: usize,
    pub words: usize,
}

/// Authored bindings and live producer lifetimes supplied independently of OAM.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SamusPublications {
    pub body: crate::game::scene::records::ProducerObjectId,
    pub echoes: [Option<crate::game::scene::records::ProducerObjectId>; 4],
    pub object_atlases: BTreeMap<u8, [crate::game::scene::records::AssetRef; 2]>,
    pub object_first_order: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum MusicMeaning {
    Silence,
    Bank {
        bank: crate::game::scene::records::AssetRef,
        after: MusicContinuation,
    },
    Program {
        program: crate::game::scene::records::AssetRef,
        phase: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum MusicContinuation {
    Retain,
    Silence,
    Program {
        program: crate::game::scene::records::AssetRef,
        phase: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SoundMeaning {
    Stop,
    Program {
        program: crate::game::scene::records::AssetRef,
        phase: u64,
    },
}
/// Declared command interpretation, independent of the live request bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AudioRequestCatalog {
    pub music: BTreeMap<u16, MusicMeaning>,
    pub sounds: BTreeMap<(usize, u8), SoundMeaning>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct QueuedMusicMeaning {
    pub request: u16,
    pub source_tick: u64,
    pub source_interval: u64,
    pub meaning: MusicMeaning,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct QueuedSoundMeaning {
    pub request: u8,
    pub source_tick: u64,
    pub source_interval: u64,
    pub meaning: SoundMeaning,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MusicRequests {
    pub tracks: [u16; 8],
    pub delays: [u16; 8],
    pub write: u16,
    pub read: u16,
    pub entry: u16,
    pub timer: u16,
    pub current_track: u8,
    pub data_index: u16,
    pub track_index: u16,
    pub meanings: [Option<QueuedMusicMeaning>; 8],
    pub entry_meaning: Option<QueuedMusicMeaning>,
    pub current_meaning: Option<QueuedMusicMeaning>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SoundRequests {
    pub read: u8,
    pub write: u8,
    pub state: u8,
    pub current: u8,
    pub clear_delay: u8,
    pub maximum_queued: u8,
    pub queue: [u8; 16],
    pub meanings: [Option<QueuedSoundMeaning>; 16],
    pub current_meaning: Option<QueuedSoundMeaning>,
}
/// The supplied current request/delay owner. Device voices and accepted batches
/// belong to A/C and are not captured here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AudioRequests {
    pub music: MusicRequests,
    pub sounds: [SoundRequests; 3],
    pub uploading: u16,
    pub downtime: u16,
    pub catalog: Arc<AudioRequestCatalog>,
    pub pending_mutation: bool,
}
macro_rules! audio_words {
    ($s:ident,$f:ident,($($borrow:tt)*)) => {
        $f(0x617,$($borrow)* $s.uploading);
        for index in 0..8 {
            $f(0x619+2*index,$($borrow)* $s.music.tracks[index]);
            $f(0x629+2*index,$($borrow)* $s.music.delays[index]);
        }
        $f(0x639,$($borrow)* $s.music.write);
        $f(0x63b,$($borrow)* $s.music.read);
        $f(0x63d,$($borrow)* $s.music.entry);
        $f(0x63f,$($borrow)* $s.music.timer);
        $f(0x686,$($borrow)* $s.downtime);
        $f(0x7f3,$($borrow)* $s.music.data_index);
        $f(0x7f5,$($borrow)* $s.music.track_index);
    };
}
macro_rules! audio_bytes {
    ($s:ident,$f:ident,($($borrow:tt)*)) => {
        $f(0x64c,$($borrow)* $s.music.current_track);
        for index in 0..3 {
            $f(0x643+index,$($borrow)* $s.sounds[index].read);
            $f(0x646+index,$($borrow)* $s.sounds[index].write);
            $f(0x649+index,$($borrow)* $s.sounds[index].state);
            $f(0x64d+index,$($borrow)* $s.sounds[index].current);
            $f(0x650+index,$($borrow)* $s.sounds[index].clear_delay);
            $f(0x653+index,$($borrow)* $s.sounds[index].maximum_queued);
            for slot in 0..16 {
                $f(0x656+16*index+slot,$($borrow)* $s.sounds[index].queue[slot]);
            }
        }
    };
}
impl AudioRequests {
    pub(crate) fn projected_byte(&self, at: usize) -> Option<u8> {
        let mut result = None;
        let mut select_byte = |base: usize, byte: &u8| {
            if base == at {
                result = Some(*byte);
            }
        };
        audio_bytes!(self,select_byte,(&));
        let mut select_word = |base: usize, word: &u16| {
            if (base..base + 2).contains(&at) {
                result = Some(word.to_le_bytes()[at - base]);
            }
        };
        audio_words!(self,select_word,(&));
        result
    }
    pub(crate) fn write_projected_byte(&mut self, at: usize, byte: u8) -> bool {
        let mut found = false;
        let mut select_byte = |base: usize, value: &mut u8| {
            if base == at {
                *value = byte;
                found = true;
            }
        };
        audio_bytes!(self,select_byte,(&mut));
        let mut select_word = |base: usize, word: &mut u16| {
            if (base..base + 2).contains(&at) {
                let mut bytes = word.to_le_bytes();
                bytes[at - base] = byte;
                *word = u16::from_le_bytes(bytes);
                found = true;
            }
        };
        audio_words!(self,select_word,(&mut));
        self.pending_mutation |= found;
        found
    }
    pub(crate) fn encode_projection(&self, mut span: impl FnMut(usize, &[u8])) {
        let mut emit_byte = |at: usize, byte: &u8| span(at, &[*byte]);
        audio_bytes!(self,emit_byte,(&));
        let mut emit_word = |at: usize, word: &u16| span(at, &word.to_le_bytes());
        audio_words!(self,emit_word,(&));
    }
    pub(crate) fn validate(
        &self,
        metadata: &DisplayStartMetadata,
        required: &AudioRequestCatalog,
    ) -> Result<(), DisplayError> {
        if self.pending_mutation {
            return Err(DisplayError::Incomplete);
        }
        self.validate_at_tick(metadata, required, metadata.identity.committed_tick)
    }
    fn validate_at_tick(
        &self,
        metadata: &DisplayStartMetadata,
        required: &AudioRequestCatalog,
        tick: u64,
    ) -> Result<(), DisplayError> {
        if *self.catalog != *required {
            return Err(DisplayError::Identity);
        }
        let next_input_record = metadata
            .identity
            .next_input_record
            .checked_add(u64::from(tick > metadata.identity.committed_tick))
            .ok_or(DisplayError::Identity)?;
        metadata
            .audio
            .validate(
                &metadata.reference,
                metadata.identity.capture,
                next_input_record,
            )
            .map_err(|_| DisplayError::Identity)?;
        let declared = |asset: crate::game::scene::records::AssetRef, bank: bool| {
            metadata.audio.declarations.iter().any(|declaration| {
                declaration.revision == asset.revision
                    && if bank {
                        declaration.bank
                    } else {
                        declaration.program
                    } == asset.id
            })
        };
        for meaning in self.catalog.music.values() {
            let valid = match meaning {
                MusicMeaning::Silence => true,
                MusicMeaning::Bank { bank, after } => {
                    declared(*bank, true)
                        && match after {
                            MusicContinuation::Retain | MusicContinuation::Silence => true,
                            MusicContinuation::Program { program, .. } => declared(*program, false),
                        }
                }
                MusicMeaning::Program { program, .. } => declared(*program, false),
            };
            if !valid {
                return Err(DisplayError::Identity);
            }
        }
        for ((channel, _), meaning) in &self.catalog.sounds {
            let valid = match meaning {
                SoundMeaning::Stop => true,
                SoundMeaning::Program { program, .. } => declared(*program, false),
            };
            if *channel >= 3 || !valid {
                return Err(DisplayError::Identity);
            }
        }
        let check_music = |number: u16, record: &QueuedMusicMeaning| {
            record.request == number
                && record.source_tick <= tick
                && record.source_interval <= next_input_record
                && self.catalog.music.get(&record.request) == Some(&record.meaning)
        };
        for (index, record) in self.music.meanings.iter().enumerate() {
            if record
                .as_ref()
                .is_some_and(|record| !check_music(self.music.tracks[index], record))
            {
                return Err(DisplayError::Identity);
            }
            if self.music.delays[index] != 0 && record.is_none() {
                return Err(DisplayError::Incomplete);
            }
        }
        if self
            .music
            .entry_meaning
            .as_ref()
            .is_some_and(|record| !check_music(self.music.entry, record))
            || ((self.music.timer != 0 || self.music.entry != 0)
                && self.music.entry_meaning.is_none())
        {
            return Err(DisplayError::Incomplete);
        }
        if let Some(record) = &self.music.current_meaning {
            if record.source_tick > tick
                || record.source_interval > next_input_record
                || self.catalog.music.get(&record.request) != Some(&record.meaning)
            {
                return Err(DisplayError::Identity);
            }
            let matches_music = |program: crate::game::scene::records::AssetRef| {
                matches!(metadata.audio.music, crate::game::scene::start::OngoingMusic::Playing(anchor)
                    if anchor.program == program.id && anchor.revision == program.revision)
            };
            let valid = match &record.meaning {
                MusicMeaning::Silence => {
                    metadata.audio.music == crate::game::scene::start::OngoingMusic::Silent
                }
                MusicMeaning::Program { program, .. } => matches_music(*program),
                MusicMeaning::Bank { after, .. } => match after {
                    MusicContinuation::Retain => true,
                    MusicContinuation::Silence => {
                        metadata.audio.music == crate::game::scene::start::OngoingMusic::Silent
                    }
                    MusicContinuation::Program { program, .. } => matches_music(*program),
                },
            };
            if !valid {
                return Err(DisplayError::Identity);
            }
        } else if self.music.current_track != 0 {
            return Err(DisplayError::Incomplete);
        }
        for (channel, sound) in self.sounds.iter().enumerate() {
            if sound.state > 4 {
                return Err(DisplayError::Source);
            }
            let check = |number: u8, record: &QueuedSoundMeaning| {
                record.request == number
                    && record.source_tick <= tick
                    && record.source_interval <= next_input_record
                    && self.catalog.sounds.get(&(channel, record.request)) == Some(&record.meaning)
            };
            for (index, record) in sound.meanings.iter().enumerate() {
                if record
                    .as_ref()
                    .is_some_and(|record| !check(sound.queue[index], record))
                {
                    return Err(DisplayError::Identity);
                }
            }
            if sound
                .current_meaning
                .as_ref()
                .is_some_and(|record| !check(sound.current, record))
            {
                return Err(DisplayError::Identity);
            }
            if sound.current != 0 && sound.current_meaning.is_none() {
                return Err(DisplayError::Incomplete);
            }
            if sound.read < 16 && sound.write < 16 {
                let mut slot = sound.read;
                while slot != sound.write {
                    if sound.meanings[usize::from(slot)].is_none() {
                        return Err(DisplayError::Incomplete);
                    }
                    slot = slot.wrapping_add(1) & 15;
                }
            } else {
                return Err(DisplayError::Source);
            }
        }
        if self.music.read >= 16
            || self.music.write >= 16
            || self.music.read & 1 != 0
            || self.music.write & 1 != 0
        {
            return Err(DisplayError::Source);
        }
        let mut slot = self.music.read;
        while slot != self.music.write {
            if self.music.meanings[usize::from(slot >> 1)].is_none() {
                return Err(DisplayError::Incomplete);
            }
            slot = slot.wrapping_add(2) & 14;
        }
        Ok(())
    }
    pub(crate) fn publish_music(
        &mut self,
        slot: usize,
        request: u16,
        source_tick: u64,
        source_interval: u64,
    ) -> Result<(), DisplayError> {
        let number = self.music.tracks.get(slot).ok_or(DisplayError::Source)?;
        if *number != request {
            return Err(DisplayError::Identity);
        }
        let meaning = self
            .catalog
            .music
            .get(&request)
            .ok_or(DisplayError::Incomplete)?
            .clone();
        self.music.meanings[slot] = Some(QueuedMusicMeaning {
            request,
            source_tick,
            source_interval,
            meaning,
        });
        self.pending_mutation = true;
        Ok(())
    }
    pub(crate) fn publish_sound(
        &mut self,
        channel: usize,
        slot: usize,
        request: u8,
        source_tick: u64,
        source_interval: u64,
    ) -> Result<(), DisplayError> {
        let sound = self.sounds.get_mut(channel).ok_or(DisplayError::Source)?;
        if *sound.queue.get(slot).ok_or(DisplayError::Source)? != request {
            return Err(DisplayError::Identity);
        }
        let meaning = self
            .catalog
            .sounds
            .get(&(channel, request))
            .ok_or(DisplayError::Incomplete)?
            .clone();
        sound.meanings[slot] = Some(QueuedSoundMeaning {
            request,
            source_tick,
            source_interval,
            meaning,
        });
        self.pending_mutation = true;
        Ok(())
    }
    pub(crate) fn clear_music(&mut self, slot: usize) -> Result<(), DisplayError> {
        let number = self.music.tracks.get(slot).ok_or(DisplayError::Source)?;
        if *number != 0 || self.music.delays[slot] != 0 {
            return Err(DisplayError::Identity);
        }
        self.music.meanings[slot] = None;
        self.pending_mutation = true;
        Ok(())
    }
    pub(crate) fn clear_sound(&mut self, channel: usize, slot: usize) -> Result<(), DisplayError> {
        let sound = self.sounds.get_mut(channel).ok_or(DisplayError::Source)?;
        if *sound.queue.get(slot).ok_or(DisplayError::Source)? != 0 {
            return Err(DisplayError::Identity);
        }
        sound.meanings[slot] = None;
        self.pending_mutation = true;
        Ok(())
    }
    pub(crate) fn capture_music(
        &self,
        slot: usize,
        request: u16,
    ) -> Result<QueuedMusicMeaning, DisplayError> {
        let record = self
            .music
            .meanings
            .get(slot)
            .ok_or(DisplayError::Source)?
            .as_ref()
            .ok_or(DisplayError::Incomplete)?;
        if record.request != request {
            return Err(DisplayError::Identity);
        }
        Ok(record.clone())
    }
    pub(crate) fn capture_sound(
        &self,
        channel: usize,
        slot: usize,
        request: u8,
    ) -> Result<QueuedSoundMeaning, DisplayError> {
        let sound = self.sounds.get(channel).ok_or(DisplayError::Source)?;
        let record = sound
            .meanings
            .get(slot)
            .ok_or(DisplayError::Source)?
            .as_ref()
            .ok_or(DisplayError::Incomplete)?;
        if record.request != request {
            return Err(DisplayError::Identity);
        }
        Ok(record.clone())
    }
    pub(crate) fn select_music_entry(
        &mut self,
        record: QueuedMusicMeaning,
    ) -> Result<(), DisplayError> {
        if self.music.entry != record.request {
            return Err(DisplayError::Identity);
        }
        self.music.entry_meaning = Some(record);
        self.pending_mutation = true;
        Ok(())
    }
    pub(crate) fn select_current_sound(
        &mut self,
        channel: usize,
        mut record: QueuedSoundMeaning,
        stamp: AudioStamp,
    ) -> Result<(), DisplayError> {
        let sound = self.sounds.get_mut(channel).ok_or(DisplayError::Source)?;
        if sound.current != record.request
            || record.source_tick > stamp.tick
            || record.source_interval > stamp.source_interval
        {
            return Err(DisplayError::Identity);
        }
        record.source_tick = stamp.tick;
        record.source_interval = stamp.source_interval;
        sound.current_meaning = Some(record);
        self.pending_mutation = true;
        Ok(())
    }
    pub(crate) fn clear_music_entry(&mut self) -> Result<(), DisplayError> {
        if self.music.entry != 0 {
            return Err(DisplayError::Identity);
        }
        self.music.entry_meaning = None;
        self.pending_mutation = true;
        Ok(())
    }
    pub(crate) fn clear_current_sound(&mut self, channel: usize) -> Result<(), DisplayError> {
        let sound = self.sounds.get_mut(channel).ok_or(DisplayError::Source)?;
        if sound.current != 0 {
            return Err(DisplayError::Identity);
        }
        sound.current_meaning = None;
        self.pending_mutation = true;
        Ok(())
    }
    pub(crate) fn complete_mutation(
        &mut self,
        metadata: &DisplayStartMetadata,
        tick: u64,
    ) -> Result<(), DisplayError> {
        if tick < metadata.identity.committed_tick
            || tick
                > metadata
                    .identity
                    .committed_tick
                    .checked_add(1)
                    .ok_or(DisplayError::Identity)?
        {
            return Err(DisplayError::Identity);
        }
        self.validate_at_tick(metadata, &self.catalog, tick)?;
        self.pending_mutation = false;
        Ok(())
    }
    pub(crate) fn accept_music(
        &mut self,
        audio: &mut crate::game::scene::start::InitialAudioIntent,
        mut record: QueuedMusicMeaning,
        stamp: AudioStamp,
    ) -> Result<(), DisplayError> {
        if self.catalog.music.get(&record.request) != Some(&record.meaning)
            || record.source_tick > stamp.tick
            || record.source_interval > stamp.source_interval
        {
            return Err(DisplayError::Identity);
        }
        let program = |asset: crate::game::scene::records::AssetRef, phase| {
            crate::game::scene::start::OngoingMusic::Playing(
                crate::game::scene::start::ProgramAnchor {
                    program: asset.id,
                    revision: asset.revision,
                    source_interval: stamp.source_interval,
                    phase,
                    timing_policy: audio.timing_policy,
                },
            )
        };
        let next = match &record.meaning {
            MusicMeaning::Silence => crate::game::scene::start::OngoingMusic::Silent,
            MusicMeaning::Program {
                program: asset,
                phase,
            } => program(*asset, *phase),
            MusicMeaning::Bank { after, .. } => match after {
                MusicContinuation::Retain => audio.music,
                MusicContinuation::Silence => crate::game::scene::start::OngoingMusic::Silent,
                MusicContinuation::Program {
                    program: asset,
                    phase,
                } => program(*asset, *phase),
            },
        };
        audio.music = next;
        record.source_tick = stamp.tick;
        record.source_interval = stamp.source_interval;
        self.music.current_meaning = Some(record);
        self.pending_mutation = true;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CurrentControl {
    pub metadata: DisplayStartMetadata,
    pub requests: AudioRequests,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AudioStamp {
    pub tick: u64,
    pub source_interval: u64,
}
impl CurrentControl {
    /// Transfer semantic counterparts only from the actual declared donor.
    /// Numeric writes already completed; any rejection keeps capture suspended.
    fn normalize_audio_from(
        &mut self,
        donor: &Self,
        range: &std::ops::Range<usize>,
    ) -> Result<(), DisplayError> {
        let touches = |base: usize, bytes: usize| range.start < base + bytes && base < range.end;
        if !range
            .clone()
            .any(|at| donor.requests.projected_byte(at).is_some())
        {
            return Ok(());
        }
        self.requests.pending_mutation = true;
        donor
            .requests
            .validate(&donor.metadata, &donor.requests.catalog)?;
        if self.metadata.reference != donor.metadata.reference
            || self.metadata.audio.capture != donor.metadata.audio.capture
            || self.metadata.audio.revision != donor.metadata.audio.revision
            || self.metadata.audio.timing_policy != donor.metadata.audio.timing_policy
            || self.metadata.audio.declarations != donor.metadata.audio.declarations
            || *self.requests.catalog != *donor.requests.catalog
        {
            return Err(DisplayError::Identity);
        }
        for slot in 0..8 {
            if touches(0x619 + 2 * slot, 2) || touches(0x629 + 2 * slot, 2) {
                if self.requests.music.tracks[slot] != donor.requests.music.tracks[slot] {
                    return Err(DisplayError::Identity);
                }
                self.requests.music.meanings[slot] = donor.requests.music.meanings[slot].clone();
            }
        }
        if touches(0x639, 4) {
            if self.requests.music.read != donor.requests.music.read
                || self.requests.music.write != donor.requests.music.write
            {
                return Err(DisplayError::Identity);
            }
            let mut cursor = self.requests.music.read;
            while cursor != self.requests.music.write {
                let slot = usize::from(cursor >> 1);
                if self.requests.music.tracks[slot] != donor.requests.music.tracks[slot] {
                    return Err(DisplayError::Identity);
                }
                self.requests.music.meanings[slot] = donor.requests.music.meanings[slot].clone();
                cursor = cursor.wrapping_add(2) & 14;
            }
        }
        if touches(0x63d, 4) {
            if self.requests.music.entry != donor.requests.music.entry {
                return Err(DisplayError::Identity);
            }
            self.requests.music.entry_meaning = donor.requests.music.entry_meaning.clone();
        }
        if touches(0x64c, 1) || touches(0x7f3, 4) {
            if self.requests.music.current_track != donor.requests.music.current_track
                || self.requests.music.data_index != donor.requests.music.data_index
                || self.requests.music.track_index != donor.requests.music.track_index
            {
                return Err(DisplayError::Identity);
            }
            self.requests.music.current_meaning = donor.requests.music.current_meaning.clone();
            self.metadata.audio.music = donor.metadata.audio.music;
        }
        for channel in 0..3 {
            for slot in 0..16 {
                if touches(0x656 + 16 * channel + slot, 1) {
                    if self.requests.sounds[channel].queue[slot]
                        != donor.requests.sounds[channel].queue[slot]
                    {
                        return Err(DisplayError::Identity);
                    }
                    self.requests.sounds[channel].meanings[slot] =
                        donor.requests.sounds[channel].meanings[slot].clone();
                }
            }
            if touches(0x643 + channel, 1) || touches(0x646 + channel, 1) {
                if self.requests.sounds[channel].read != donor.requests.sounds[channel].read
                    || self.requests.sounds[channel].write != donor.requests.sounds[channel].write
                {
                    return Err(DisplayError::Identity);
                }
                let mut cursor = self.requests.sounds[channel].read;
                while cursor != self.requests.sounds[channel].write {
                    let slot = usize::from(cursor);
                    if self.requests.sounds[channel].queue[slot]
                        != donor.requests.sounds[channel].queue[slot]
                    {
                        return Err(DisplayError::Identity);
                    }
                    self.requests.sounds[channel].meanings[slot] =
                        donor.requests.sounds[channel].meanings[slot].clone();
                    cursor = cursor.wrapping_add(1) & 15;
                }
            }
            if touches(0x649 + channel, 1)
                || touches(0x64d + channel, 1)
                || touches(0x650 + channel, 1)
            {
                if self.requests.sounds[channel].current != donor.requests.sounds[channel].current {
                    return Err(DisplayError::Identity);
                }
                self.requests.sounds[channel].current_meaning =
                    donor.requests.sounds[channel].current_meaning.clone();
            }
        }
        self.requests
            .complete_mutation(&self.metadata, self.metadata.identity.committed_tick)
    }
    pub(crate) fn audio_mutation<R>(
        &mut self,
        producing: bool,
        operation: impl FnOnce(
            &mut AudioRequests,
            &mut crate::game::scene::start::InitialAudioIntent,
            AudioStamp,
        ) -> Result<R, DisplayError>,
    ) -> Result<R, DisplayError> {
        if !producing {
            return Err(DisplayError::Phase);
        }
        let stamp = AudioStamp {
            tick: self
                .metadata
                .identity
                .committed_tick
                .checked_add(1)
                .ok_or(DisplayError::Identity)?,
            source_interval: self.metadata.identity.next_input_record,
        };
        self.requests.pending_mutation = true;
        let result = operation(&mut self.requests, &mut self.metadata.audio, stamp)?;
        self.requests
            .complete_mutation(&self.metadata, stamp.tick)?;
        Ok(result)
    }
}
/// One ordered numeric/request contract shared by actual Game and Runtime callers.
pub(crate) trait AudioHost {
    fn audio_byte(&self, at: usize) -> u8;
    fn audio_store_byte(&mut self, at: usize, byte: u8);
    fn audio_mutation<R>(
        &mut self,
        operation: impl FnOnce(
            &mut AudioRequests,
            &mut crate::game::scene::start::InitialAudioIntent,
            AudioStamp,
        ) -> Result<R, DisplayError>,
    ) -> Result<Option<R>, DisplayError>;
    fn audio_word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.audio_byte(at), self.audio_byte(at + 1)])
    }
    fn audio_store_word(&mut self, at: usize, word: u16) {
        let bytes = word.to_le_bytes();
        self.audio_store_byte(at, bytes[0]);
        self.audio_store_byte(at + 1, bytes[1]);
    }
}

pub(crate) struct SessionStartDescription {
    pub gameplay: Option<Session>,
    pub observations: Option<ExternalInputs>,
    pub display: DisplayStartDescription,
    pub roles: Option<ContentDescription>,
    pub marker: Option<MarkerPublication>,
    pub actor_queue_projection: Option<[ActorQueueProjection; 8]>,
    pub samus_publications: Option<Arc<SamusPublications>>,
    pub audio_requests: Option<AudioRequests>,
}

#[derive(Clone)]
pub(crate) struct SessionStartRequirements {
    pub display: DisplayStartRequirements,
    pub roles: ContentDescription,
    pub marker: MarkerPublication,
    pub actor_queue_projection: [ActorQueueProjection; 8],
    pub samus_publications: Arc<SamusPublications>,
    pub audio_catalog: Arc<AudioRequestCatalog>,
}

#[derive(Debug)]
pub enum SessionStartError {
    MissingGameplay,
    MissingObservations,
    MissingRoles,
    MissingBodyMarker,
    MissingActorQueueProjection,
    MissingSamusPublications,
    MissingAudioRequests,
    World,
    GameplayDisplay,
    Observations,
    ActorQueues,
    Display(StartError),
    Binding(DisplayError),
    Phase,
}

/// This is an unpublished replacement. Installation must move these owners,
/// close native suspension/adoption, and never retain the supplied duplicate
/// numeric presentation fields as independently writable live state.
pub struct PreparedSessionStart {
    pub(crate) gameplay: Session,
    pub(crate) observations: ExternalInputs,
    pub(crate) display: DisplayComponent,
    pub(crate) metadata: DisplayStartMetadata,
    pub(crate) audio_requests: AudioRequests,
    pub(crate) binding: PreparedSamusBinding,
    pub(crate) marker: MarkerPublication,
    pub(crate) actor_queue_projection: [ActorQueueProjection; 8],
    pub(crate) roles: ContentDescription,
    pub(crate) samus_publications: Arc<SamusPublications>,
    pub(crate) required: Arc<SessionStartRequirements>,
}

/// Tools supplies the complete semantic construction and independent contract.
/// The app transports this value without access to mutable gameplay internals.
pub struct SuppliedSessionStart {
    description: SessionStartDescription,
    media: Arc<SceneMedia>,
    required: SessionStartRequirements,
}
pub struct SessionStartContract(SessionStartRequirements);

/// Current input, runner and semantic audio owners, exclusive with legacy bytes.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Control {
    Legacy(crate::game::room_entry::state::SharedContinuations),
    Owned(Box<CurrentControl>),
}
macro_rules! control_words {
    ($m:ident,$f:ident,($($borrow:tt)*)) => {
        $f(0x87,$($borrow)* $m.input_history.repeat_first);
        $f(0x89,$($borrow)* $m.input_history.repeat_next);
        for (index, base) in [0x8b,0x8d].into_iter().enumerate() {
            $f(base,$($borrow)* $m.input_history.ports[index].sampled);
            $f(base+4,$($borrow)* $m.input_history.ports[index].pressed);
            $f(base+8,$($borrow)* $m.input_history.ports[index].repeated);
            $f(base+12,$($borrow)* $m.input_history.ports[index].previous);
            $f(0xa3+2*index,$($borrow)* $m.input_history.ports[index].repeat_counter);
        }
        $f(0xc2,$($borrow)* $m.input_history.bug_fix_counter);
        $f(0x5d9,$($borrow)* $m.input_history.released);
        $f(0x5db,$($borrow)* $m.input_history.held_timer);
        $f(0x5dd,$($borrow)* $m.input_history.held_timer_reset);
        $f(0x5df,$($borrow)* $m.input_history.timed_held);
        $f(0x5e1,$($borrow)* $m.input_history.newly_held);
        $f(0x5e3,$($borrow)* $m.input_history.previous_timed_held);
        $f(0x9b2,$($borrow)* $m.input_history.bindings.shoot);
        $f(0x9b4,$($borrow)* $m.input_history.bindings.jump);
        $f(0x9b6,$($borrow)* $m.input_history.bindings.run);
        $f(0x9b8,$($borrow)* $m.input_history.bindings.cancel);
        $f(0x9ba,$($borrow)* $m.input_history.bindings.switch);
        $f(0x9bc,$($borrow)* $m.input_history.bindings.aim_down);
        $f(0x9be,$($borrow)* $m.input_history.bindings.aim_up);
        $f(0xdfe,$($borrow)* $m.input_history.samus_held);
        $f(0xe00,$($borrow)* $m.input_history.samus_pressed);
    };
}
impl Control {
    pub(crate) fn requests(&self) -> Result<&AudioRequests, DisplayError> {
        match self {
            Self::Owned(value) => Ok(&value.requests),
            Self::Legacy(_) => Err(DisplayError::Phase),
        }
    }
    pub(crate) fn requests_mut(&mut self) -> Result<&mut AudioRequests, DisplayError> {
        match self {
            Self::Owned(value) => Ok(&mut value.requests),
            Self::Legacy(_) => Err(DisplayError::Phase),
        }
    }
    pub(crate) fn legacy() -> Self {
        Self::Legacy(crate::game::room_entry::state::SharedContinuations {
            primary: 0,
            secondary: 0,
        })
    }
    pub(crate) fn primary(&self) -> &u8 {
        match self {
            Self::Legacy(value) => &value.primary,
            Self::Owned(value) => &value.metadata.continuation.primary,
        }
    }
    pub(crate) fn secondary(&self) -> &u8 {
        match self {
            Self::Legacy(value) => &value.secondary,
            Self::Owned(value) => &value.metadata.continuation.secondary,
        }
    }
    pub(crate) fn primary_mut(&mut self) -> &mut u8 {
        match self {
            Self::Legacy(value) => &mut value.primary,
            Self::Owned(value) => &mut value.metadata.continuation.primary,
        }
    }
    pub(crate) fn secondary_mut(&mut self) -> &mut u8 {
        match self {
            Self::Legacy(value) => &mut value.secondary,
            Self::Owned(value) => &mut value.metadata.continuation.secondary,
        }
    }
    pub(crate) fn metadata(&self) -> Result<&DisplayStartMetadata, DisplayError> {
        match self {
            Self::Owned(value) => Ok(&value.metadata),
            Self::Legacy(_) => Err(DisplayError::Phase),
        }
    }
    pub(crate) fn metadata_mut(&mut self) -> Result<&mut DisplayStartMetadata, DisplayError> {
        match self {
            Self::Owned(value) => Ok(&mut value.metadata),
            Self::Legacy(_) => Err(DisplayError::Phase),
        }
    }
    pub(crate) fn projected_byte(&self, at: usize) -> Option<u8> {
        if at == 0x612 {
            return Some(*self.primary());
        }
        if at == 0x613 {
            return Some(*self.secondary());
        }
        let Self::Owned(control) = self else {
            return None;
        };
        if let Some(byte) = control.requests.projected_byte(at) {
            return Some(byte);
        }
        let m = &control.metadata;
        match at {
            0x611 => return Some(m.continuation.outer),
            0x614 => return Some(m.continuation.music),
            0x615 => return Some(m.continuation.message),
            0x616 => return Some(m.continuation.auxiliary),
            _ => {}
        }
        let mut value = None;
        let mut select = |base: usize, word: &u16| {
            if (base..base + 2).contains(&at) {
                value = Some(word.to_le_bytes()[at - base]);
            }
        };
        control_words!(m,select,(&));
        value
    }
    pub(crate) fn write_projected_byte(&mut self, at: usize, byte: u8) -> bool {
        if at == 0x612 {
            *self.primary_mut() = byte;
            return true;
        }
        if at == 0x613 {
            *self.secondary_mut() = byte;
            return true;
        }
        let Self::Owned(control) = self else {
            return false;
        };
        if control.requests.write_projected_byte(at, byte) {
            return true;
        }
        let m = &mut control.metadata;
        match at {
            0x611 => {
                m.continuation.outer = byte;
                return true;
            }
            0x614 => {
                m.continuation.music = byte;
                return true;
            }
            0x615 => {
                m.continuation.message = byte;
                return true;
            }
            0x616 => {
                m.continuation.auxiliary = byte;
                return true;
            }
            _ => {}
        }
        let mut found = false;
        let mut select = |base: usize, word: &mut u16| {
            if (base..base + 2).contains(&at) {
                let mut bytes = word.to_le_bytes();
                bytes[at - base] = byte;
                *word = u16::from_le_bytes(bytes);
                found = true;
            }
        };
        control_words!(m,select,(&mut));
        found
    }
    pub(crate) fn encode_projection(&self, mut span: impl FnMut(usize, &[u8])) {
        if let Self::Owned(control) = self {
            control.requests.encode_projection(&mut span);
            let m = &control.metadata;
            for at in [0x611, 0x614, 0x615, 0x616] {
                span(at, &[self.projected_byte(at).expect("runner projection")]);
            }
            let mut emit = |at: usize, word: &u16| span(at, &word.to_le_bytes());
            control_words!(m,emit,(&));
        }
    }
}
impl SuppliedSessionStart {
    pub(crate) fn new(
        description: SessionStartDescription,
        media: Arc<SceneMedia>,
        required: SessionStartRequirements,
    ) -> Self {
        Self {
            description,
            media,
            required,
        }
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        SessionStartDescription,
        Arc<SceneMedia>,
        SessionStartRequirements,
    ) {
        (self.description, self.media, self.required)
    }
}
impl SessionStartContract {
    pub(crate) fn new(required: SessionStartRequirements) -> Self {
        Self(required)
    }
    pub(crate) fn requirements(&self) -> &SessionStartRequirements {
        &self.0
    }
}

pub(crate) fn prepare(
    game: &Game,
    supplied: SessionStartDescription,
    media: Arc<SceneMedia>,
    required: &SessionStartRequirements,
) -> Result<PreparedSessionStart, SessionStartError> {
    let gameplay = supplied
        .gameplay
        .ok_or(SessionStartError::MissingGameplay)?;
    if !Arc::ptr_eq(&gameplay.world, game.world())
        || gameplay.world.revision() != game.world().revision()
    {
        return Err(SessionStartError::World);
    }
    let observations = supplied
        .observations
        .ok_or(SessionStartError::MissingObservations)?;
    let roles = supplied.roles.ok_or(SessionStartError::MissingRoles)?;
    let marker = supplied
        .marker
        .ok_or(SessionStartError::MissingBodyMarker)?;
    let projection = supplied
        .actor_queue_projection
        .ok_or(SessionStartError::MissingActorQueueProjection)?;
    let publications = supplied
        .samus_publications
        .ok_or(SessionStartError::MissingSamusPublications)?;
    let display = supplied
        .display
        .display
        .as_ref()
        .ok_or(SessionStartError::Display(StartError::MissingDisplay))?;
    if *gameplay.presentation.visual() != display.visual
        || *gameplay.presentation.palette() != display.player_palette
        || *gameplay.presentation.atmospheric() != display.atmospheric
    {
        return Err(SessionStartError::GameplayDisplay);
    }
    if let Presentation::Owned {
        display: existing, ..
    } = &gameplay.presentation
    {
        if existing.owner().state != *display {
            return Err(SessionStartError::GameplayDisplay);
        }
    }
    if observations.transitions.frame_gamma != gameplay.player.transitions.frame_gamma
        || observations.frame != display.nmi.frame_word
    {
        return Err(SessionStartError::Observations);
    }
    if projection != required.actor_queue_projection {
        return Err(SessionStartError::ActorQueues);
    }
    if publications != required.samus_publications
        || !publications
            .object_atlases
            .contains_key(&display.controls.object_layout)
    {
        return Err(SessionStartError::MissingSamusPublications);
    }
    let mut producers = vec![marker.producer, publications.body];
    producers.extend(publications.echoes.iter().flatten().copied());
    for (index, producer) in producers.iter().enumerate() {
        if producers[..index].contains(producer)
            || display.producer_ids.validate_id(*producer).is_err()
        {
            return Err(SessionStartError::MissingSamusPublications);
        }
    }
    for atlases in publications.object_atlases.values() {
        for atlas in atlases {
            media
                .atlas(*atlas)
                .map_err(|_| SessionStartError::MissingSamusPublications)?;
        }
    }
    let queue_word = |at| {
        let low = crate::game::session_encoding::projected_byte(&gameplay, at)
            .ok_or(SessionStartError::ActorQueues)?;
        let high = crate::game::session_encoding::projected_byte(&gameplay, at + 1)
            .ok_or(SessionStartError::ActorQueues)?;
        Ok::<_, SessionStartError>(u16::from_le_bytes([low, high]))
    };
    let mut covered = [false; 114];
    for (layer, layout) in projection.iter().enumerate() {
        let end = layout
            .first
            .checked_add(layout.words)
            .ok_or(SessionStartError::ActorQueues)?;
        if end > covered.len()
            || display.actor_queues[layer].actors.len() != layout.words
            || display.actor_queues[layer].size != queue_word(0xf68 + layer * 2)?
        {
            return Err(SessionStartError::ActorQueues);
        }
        for index in layout.first..end {
            if covered[index]
                || queue_word(0xe84 + index * 2)?
                    != display.actor_queues[layer].actors[index - layout.first]
            {
                return Err(SessionStartError::ActorQueues);
            }
            covered[index] = true;
        }
    }
    if covered.iter().any(|value| !value) {
        return Err(SessionStartError::ActorQueues);
    }
    marker
        .validate(&media)
        .map_err(SessionStartError::Binding)?;
    if marker != required.marker || display.producer_ids.validate_id(marker.producer).is_err() {
        return Err(SessionStartError::MissingBodyMarker);
    }
    let catalog = supplied
        .display
        .samus_content
        .as_ref()
        .ok_or(SessionStartError::Display(StartError::MissingSamusContent))?
        .clone();
    let binding = game
        .prepare_samus_binding(
            SamusBindingDescription {
                world: gameplay.world.clone(),
                roles: roles.clone(),
                spritemaps: catalog.clone(),
                request_media: media.clone(),
                execution: SamusExecution::Prepared,
            },
            &SamusBindingDescription {
                world: game.world().clone(),
                roles: required.roles.clone(),
                spritemaps: required.display.samus_content.clone(),
                request_media: media.clone(),
                execution: SamusExecution::Prepared,
            },
        )
        .map_err(SessionStartError::Binding)?;
    let opened = DisplayComponent::open(
        ValidatedDisplayStart::construct(supplied.display, media, &required.display)
            .map_err(SessionStartError::Display)?,
    )
    .map_err(SessionStartError::Display)?;
    if *gameplay.control.primary() != opened.metadata.continuation.primary
        || *gameplay.control.secondary() != opened.metadata.continuation.secondary
    {
        return Err(SessionStartError::GameplayDisplay);
    }
    if gameplay
        .control
        .metadata()
        .is_ok_and(|existing| *existing != opened.metadata)
    {
        return Err(SessionStartError::GameplayDisplay);
    }
    let audio_requests = supplied
        .audio_requests
        .ok_or(SessionStartError::MissingAudioRequests)?;
    audio_requests
        .validate(&opened.metadata, &required.audio_catalog)
        .map_err(SessionStartError::Binding)?;
    if gameplay
        .control
        .requests()
        .is_ok_and(|existing| *existing != audio_requests)
    {
        return Err(SessionStartError::GameplayDisplay);
    }
    Ok(PreparedSessionStart {
        gameplay,
        observations,
        display: opened.display,
        metadata: opened.metadata,
        audio_requests,
        binding,
        marker,
        actor_queue_projection: projection,
        roles,
        samus_publications: publications,
        required: Arc::new(required.clone()),
    })
}

/// The one live gameplay owner. Runtime moves an exclusive lease of this value;
/// it never decodes a second Session while this owner exists.
#[derive(Clone)]
pub(crate) struct PersistentSession {
    pub(crate) gameplay: Session,
    pub(crate) observations: ExternalInputs,
    pub(crate) roles: ContentDescription,
    pub(crate) required: Arc<SessionStartRequirements>,
}
pub(crate) struct IntervalMetadata {
    pub input: crate::game::scene::start::InputHistory,
    pub continuation: crate::game::scene::start::ContinuationHistory,
    pub audio: crate::game::scene::start::InitialAudioIntent,
}

pub(crate) enum SessionSlot {
    Legacy,
    Ready(Box<PersistentSession>),
    Leased,
    Failed(Option<Box<PersistentSession>>),
}

impl PreparedSessionStart {
    pub(crate) fn into_session(self) -> (PersistentSession, PreparedSamusBinding) {
        let Self {
            mut gameplay,
            observations,
            display,
            metadata,
            audio_requests,
            binding,
            marker,
            actor_queue_projection,
            roles,
            samus_publications,
            required,
        } = self;
        // Replace, rather than retain, the supplied numeric construction copy.
        gameplay.presentation = Presentation::Owned {
            display: Box::new(display),
            marker,
            actor_queue_projection,
            samus_publications,
        };
        gameplay.control = Control::Owned(Box::new(CurrentControl {
            metadata,
            requests: audio_requests,
        }));
        (
            PersistentSession {
                gameplay,
                observations,
                roles,
                required,
            },
            binding,
        )
    }
}

/// Captured current owners, including final resident and separate visible history.
/// This semantic checkpoint does not run an interval or invoke the native loader.
#[derive(Clone)]
pub struct SessionCheckpoint(PersistentSession);

impl PersistentSession {
    pub(crate) fn metadata(&self) -> &DisplayStartMetadata {
        self.gameplay
            .control
            .metadata()
            .expect("persistent control owner")
    }
    pub(crate) fn metadata_mut(&mut self) -> &mut DisplayStartMetadata {
        self.gameplay
            .control
            .metadata_mut()
            .expect("persistent control owner")
    }
    pub(crate) fn begin_interval(&mut self, fresh_entry: bool) -> Result<(), DisplayError> {
        self.gameplay
            .presentation
            .owner_mut()?
            .begin_interval(fresh_entry)
    }
    pub(crate) fn finish_interval(
        &mut self,
        metadata: IntervalMetadata,
        description: crate::game::scene::records::SceneDescription,
        media: Arc<SceneMedia>,
        effects: &mut impl crate::game::scene::commit::SettlementEffects,
    ) -> Result<(), DisplayError> {
        let current = self.metadata().clone();
        let next_tick = current
            .identity
            .committed_tick
            .checked_add(1)
            .ok_or(DisplayError::Identity)?;
        if description.tick != next_tick
            || metadata.continuation.next_input_record
                != current
                    .identity
                    .next_input_record
                    .checked_add(1)
                    .ok_or(DisplayError::Identity)?
            || metadata.continuation.primary != *self.gameplay.control.primary()
            || metadata.continuation.secondary != *self.gameplay.control.secondary()
            || metadata.continuation.outer != current.continuation.outer
            || metadata.continuation.music != current.continuation.music
            || metadata.continuation.message != current.continuation.message
            || metadata.continuation.auxiliary != current.continuation.auxiliary
            || metadata.continuation.capture != current.identity.capture
            || metadata.continuation.observation_policy != current.reference.observation_policy
            || metadata.continuation.phase
                != crate::game::scene::start::CommittedPhase::SettledInterval
            || metadata.input.capture != current.identity.capture
            || metadata.input.policy != current.reference.input_policy
            || metadata.input != current.input_history
            || metadata.audio.capture != current.identity.capture
            || metadata.audio.revision != self.required.display.audio_revision
            || metadata.audio.declarations != self.required.display.audio_programs
            || metadata.audio.timing_policy != current.reference.audio_timing_policy
            || metadata.audio != current.audio
        {
            return Err(DisplayError::Identity);
        }
        let requests = self.gameplay.control.requests()?;
        if requests.pending_mutation {
            return Err(DisplayError::Incomplete);
        }
        requests.validate_at_tick(&current, &self.required.audio_catalog, next_tick)?;
        let owner = self.gameplay.presentation.owner_mut()?;
        owner.refresh()?;
        owner.begin_settlement()?;
        owner.settle(effects)?;
        owner.commit_scene(description, media)?;
        self.observations.frame = owner.state.nmi.frame_word;
        let current = self.metadata_mut();
        current.identity.committed_tick = next_tick;
        current.identity.next_input_record = metadata.continuation.next_input_record;
        current.input_history = metadata.input;
        current.continuation = metadata.continuation;
        current.audio = metadata.audio;
        Ok(())
    }
    pub(crate) fn checkpoint(&self) -> Result<SessionCheckpoint, DisplayError> {
        self.gameplay.presentation.owner()?.checkpoint_parts()?;
        self.gameplay
            .control
            .requests()?
            .validate(self.metadata(), &self.required.audio_catalog)?;
        if self.metadata().identity.committed_tick
            != self.gameplay.presentation.owner()?.committed().tick()
            || self.metadata().identity.next_input_record
                != self.metadata().continuation.next_input_record
        {
            return Err(DisplayError::Identity);
        }
        Ok(SessionCheckpoint(self.clone()))
    }
    pub(crate) fn poison(&mut self) {
        if let Ok(owner) = self.gameplay.presentation.owner_mut() {
            owner.state.phase = crate::game::scene::state::DisplayPhase::Failed;
        }
        self.gameplay.actor_identity.retain_poison();
    }
}

impl SessionCheckpoint {
    pub(crate) fn into_start(
        self,
    ) -> Result<(SessionStartDescription, Arc<SceneMedia>), DisplayError> {
        let PersistentSession {
            gameplay,
            observations,
            roles,
            required: _,
        } = self.0;
        let metadata = gameplay.control.metadata()?.clone();
        let audio_requests = gameplay.control.requests()?.clone();
        let (display, marker, projection, publications) = match &gameplay.presentation {
            Presentation::Owned {
                display,
                marker,
                actor_queue_projection,
                samus_publications,
            } => (
                display.checkpoint()?,
                marker.clone(),
                *actor_queue_projection,
                samus_publications.clone(),
            ),
            Presentation::Legacy(_) => return Err(DisplayError::Phase),
        };
        let (display, media) = display.into_start_description(metadata);
        Ok((
            SessionStartDescription {
                gameplay: Some(gameplay),
                observations: Some(observations),
                display,
                roles: Some(roles),
                marker: Some(marker),
                actor_queue_projection: Some(projection),
                samus_publications: Some(publications),
                audio_requests: Some(audio_requests),
            },
            media,
        ))
    }
}

/// One complete semantic branch observation, never an alternative live owner.
#[derive(Clone)]
pub(crate) struct SessionSnapshotPayload(pub(crate) Option<SessionCheckpoint>);
pub(crate) struct PreparedSessionAdoption(pub(crate) Option<PreparedSessionStart>);
pub(crate) fn prepare_snapshot(
    game: &Game,
    payload: &SessionSnapshotPayload,
) -> Result<PreparedSessionAdoption, SessionStartError> {
    let Some(checkpoint) = &payload.0 else {
        return Ok(PreparedSessionAdoption(None));
    };
    let required = checkpoint.0.required.clone();
    let (start, media) = checkpoint
        .clone()
        .into_start()
        .map_err(SessionStartError::Binding)?;
    prepare(game, start, media, &required).map(|start| PreparedSessionAdoption(Some(start)))
}
pub(crate) fn validate_normalized_projection(
    game: &Game,
    prepared: &PreparedSessionStart,
) -> Result<(), DisplayError> {
    let mut matches = true;
    crate::game::session_encoding::encode(&prepared.gameplay, |at, bytes| {
        for (part, byte) in bytes.iter().enumerate() {
            matches &= game.ram().read_byte(at + part) == *byte;
        }
    });
    let continuation = &prepared.metadata.continuation;
    for (at, byte) in [
        (0x611, continuation.outer),
        (0x612, continuation.primary),
        (0x613, continuation.secondary),
        (0x614, continuation.music),
        (0x615, continuation.message),
        (0x616, continuation.auxiliary),
    ] {
        matches &= game.ram().read_byte(at) == byte;
    }
    if matches {
        Ok(())
    } else {
        Err(DisplayError::Identity)
    }
}
impl SessionSnapshotPayload {
    pub(crate) fn is_owned(&self) -> bool {
        self.0.is_some()
    }
    /// Apply only a caller-declared normalization transfer from its actual branch.
    /// Numeric projections carry no permission to manufacture content or history.
    pub(crate) fn normalize_from(
        &mut self,
        source: &Self,
        range: std::ops::Range<usize>,
    ) -> Result<(), DisplayError> {
        match (&mut self.0, &source.0) {
            (None, None) => Ok(()),
            (Some(target), Some(source)) => {
                if !Arc::ptr_eq(&target.0.gameplay.world, &source.0.gameplay.world) {
                    return Err(DisplayError::Identity);
                }
                for at in range.clone() {
                    if let Some(byte) =
                        crate::game::session_encoding::projected_byte(&source.0.gameplay, at)
                    {
                        if !crate::game::session_encoding::write_projected_byte(
                            &mut target.0.gameplay,
                            at,
                            byte,
                        ) {
                            return Err(DisplayError::Incomplete);
                        }
                    }
                    match at {
                        0x611 => {
                            target.0.metadata_mut().continuation.outer =
                                source.0.metadata().continuation.outer
                        }
                        0x612 => {
                            target.0.metadata_mut().continuation.primary =
                                source.0.metadata().continuation.primary
                        }
                        0x613 => {
                            target.0.metadata_mut().continuation.secondary =
                                source.0.metadata().continuation.secondary
                        }
                        0x614 => {
                            target.0.metadata_mut().continuation.music =
                                source.0.metadata().continuation.music
                        }
                        0x615 => {
                            target.0.metadata_mut().continuation.message =
                                source.0.metadata().continuation.message
                        }
                        0x616 => {
                            target.0.metadata_mut().continuation.auxiliary =
                                source.0.metadata().continuation.auxiliary
                        }
                        _ => {}
                    }
                }
                match (&mut target.0.gameplay.control, &source.0.gameplay.control) {
                    (Control::Owned(target), Control::Owned(source)) => {
                        target.normalize_audio_from(source, &range)
                    }
                    _ => Err(DisplayError::Incomplete),
                }
            }
            _ => Err(DisplayError::Incomplete),
        }
    }
    pub(crate) fn final_continuation(&mut self, at: usize, value: u8) -> Result<(), DisplayError> {
        let Some(state) = &mut self.0 else {
            return Ok(());
        };
        match at {
            0x611 => state.0.metadata_mut().continuation.outer = value,
            0x612 => {
                state.0.metadata_mut().continuation.primary = value;
            }
            0x613 => {
                state.0.metadata_mut().continuation.secondary = value;
            }
            0x614 => state.0.metadata_mut().continuation.music = value,
            0x615 => state.0.metadata_mut().continuation.message = value,
            0x616 => state.0.metadata_mut().continuation.auxiliary = value,
            _ => return Err(DisplayError::Incomplete),
        }
        Ok(())
    }
    pub(crate) fn required(&self) -> Option<&Arc<SessionStartRequirements>> {
        self.0.as_ref().map(|state| &state.0.required)
    }
}

/// Exactly one presentation owner is selected. Legacy numeric construction does
/// not manufacture media, producer identities, Reference or a Display start.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Presentation {
    Legacy(Box<LegacyPresentation>),
    Owned {
        display: Box<DisplayComponent>,
        marker: MarkerPublication,
        actor_queue_projection: [ActorQueueProjection; 8],
        samus_publications: Arc<SamusPublications>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LegacyPresentation {
    visual: crate::game::visual_state::Visuals,
    palette: crate::game::visual_state::PlayerPalette,
    atmospheric: crate::game::player_transitions::state::AtmosphericEffects,
    actor_entries: [u16; 114],
    actor_sizes: [u16; 8],
}
// These are byte/word views of the current Display owner, including odd aliases.
// Semantic source bindings and request identities are never decoded from them.
macro_rules! display_bytes {
    ($s:ident,$f:ident,($($borrow:tt)*)) => {
        $f(0x51,$($borrow)* $s.controls.brightness);
        $f(0x52,$($borrow)* $s.controls.object_layout);
        $f(0x55,$($borrow)* $s.controls.mode);
        $f(0x56,$($borrow)* $s.controls.fake_mode);
        $f(0x57,$($borrow)* $s.controls.mosaic);
        for (index, at) in [0x58,0x59,0x5a,0x5c].into_iter().enumerate() {
            $f(at,$($borrow)* $s.controls.screens[index]);
        }
        $f(0x5b,$($borrow)* $s.backgrounds[2].screen_layout);
        for index in 0..2 {
            $f(0x5d+index,$($borrow)* $s.controls.characters[index]);
            $f(0x67+index,$($borrow)* $s.controls.window_logic[index]);
        }
        $f(0x5f,$($borrow)* $s.controls.affine_selection);
        for index in 0..3 {
            $f(0x60+index,$($borrow)* $s.controls.window_selection[index]);
            $f(0x74+index,$($borrow)* $s.controls.fixed_color[index]);
        }
        for index in 0..4 {
            $f(0x63+index,$($borrow)* $s.controls.window_positions[index]);
        }
        $f(0x69,$($borrow)* $s.controls.main_layers);
        $f(0x6a,$($borrow)* $s.controls.previous_main_layers);
        $f(0x6b,$($borrow)* $s.controls.sub_layers);
        $f(0x6c,$($borrow)* $s.controls.main_window);
        $f(0x6d,$($borrow)* $s.controls.sub_window);
        $f(0x6e,$($borrow)* $s.controls.next_color_selection);
        $f(0x6f,$($borrow)* $s.controls.color_selection);
        $f(0x70,$($borrow)* $s.controls.previous_color_selection);
        $f(0x71,$($borrow)* $s.controls.next_color_math);
        $f(0x72,$($borrow)* $s.controls.color_math);
        $f(0x73,$($borrow)* $s.controls.previous_color_math);
        $f(0x77,$($borrow)* $s.controls.geometry);
        $f(0x84,$($borrow)* $s.controls.nmi_enable);
        $f(0x85,$($borrow)* $s.controls.hdma_enable);
        $f(0x7ec,$($borrow)* $s.controls.committed_fake_mode);
    };
}
macro_rules! display_words {
    ($s:ident,$f:ident,($($borrow:tt)*)) => {
        for index in 0..6 {
            $f(0x78+2*index,$($borrow)* $s.controls.affine_words[index]);
        }
        for index in 0..8 {
            $f(0xb1+2*index,$($borrow)* $s.controls.scroll_words[index]);
        }
        $f(0xa7,$($borrow)* $s.interrupt.pending);
        $f(0xab,$($borrow)* $s.interrupt.current);
        $f(0x330,$($borrow)* $s.upload_tail);
        $f(0x334,$($borrow)* $s.mode7_tail);
        $f(0x360,$($borrow)* $s.readback_tail);
        $f(0x77c,$($borrow)* $s.hud.palette);
        $f(0x9ec,$($borrow)* $s.hud.auto_cancel);
        $f(0xa06,$($borrow)* $s.hud.previous_health);
        $f(0xa0e,$($borrow)* $s.hud.previous_selection);
        for index in 0..3 {
            $f(0xa08+2*index,$($borrow)* $s.hud.previous_ammunition[index]);
        }
        for index in 0..96 {
            $f(0xc608+2*index,$($borrow)* $s.hud.tiles[index]);
        }
        $f(0x18b0,$($borrow)* $s.hdma.flags);
        $f(0x18b2,$($borrow)* $s.hdma.selected);
        for index in 0..6 {
            $f(0x18b4+2*index,$($borrow)* $s.hdma.bindings[index].enable);
            $f(0x18c0+2*index,$($borrow)* $s.hdma.bindings[index].bank_slot);
            $f(0x18cc+2*index,$($borrow)* $s.hdma.objects[index].instruction_cursor);
            $f(0x18d8+2*index,$($borrow)* $s.hdma.bindings[index].table);
            $f(0x18e4+2*index,$($borrow)* $s.hdma.objects[index].instruction_timer);
            $f(0x18f0+2*index,$($borrow)* $s.hdma.objects[index].pre_instruction);
            $f(0x18fc+2*index,$($borrow)* $s.hdma.objects[index].pre_instruction_bank);
            $f(0x1908+2*index,$($borrow)* $s.hdma.objects[index].timer);
            for variable in 0..6 {
                $f(0x1914+12*variable+2*index,$($borrow)* $s.hdma.objects[index].variables[variable]);
            }
        }
    };
}
impl Presentation {
    pub(crate) fn requires_semantic_counterpart(at: usize) -> bool {
        // Publishing these numeric requests also needs supplied source/audio
        // identities. Their actual semantic publication seams are not closed.
        (0xd0..0x330).contains(&at) || (0x340..0x360).contains(&at) || (0x956..0x98e).contains(&at)
    }
    pub(crate) fn projected_byte(&self, at: usize) -> Option<u8> {
        let Self::Owned { display, .. } = self else {
            return None;
        };
        let state = &display.owner().state;
        if (0x370..0x592).contains(&at) {
            return state.objects.byte(at - 0x370).ok();
        }
        let mut projected = None;
        let mut select_byte = |base: usize, byte: &u8| {
            if base == at {
                projected = Some(*byte);
            }
        };
        display_bytes!(state,select_byte,(&));
        let mut select_word = |base: usize, word: &u16| {
            if (base..base + 2).contains(&at) {
                projected = Some(word.to_le_bytes()[at - base]);
            }
        };
        display_words!(state,select_word,(&));
        if projected.is_some() {
            return projected;
        }
        match at {
            0x5b4 => Some(state.nmi.pending),
            0x5b5 => Some(state.nmi.frame_byte),
            0x5b6..=0x5b7 => Some(state.nmi.frame_word.to_le_bytes()[at - 0x5b6]),
            0x5b8..=0x5b9 => Some(state.nmi.including_lag.to_le_bytes()[at - 0x5b8]),
            0x5ba => Some(state.nmi.missed),
            0x5bb => Some(state.nmi.max_missed),
            0x71d..=0x71e => Some(state.samus[at - 0x71d].enabled),
            0x71f..=0x722 => {
                let offset = at - 0x71f;
                Some(state.samus[offset / 2].pointer.to_le_bytes()[offset % 2])
            }
            0x782 => Some(state.nmi.acknowledgment),
            _ => None,
        }
    }
    pub(crate) fn write_projected_byte(&mut self, at: usize, value: u8) -> bool {
        let Self::Owned { display, .. } = self else {
            return false;
        };
        let state = &mut display.owner_mut().state;
        if (0x370..0x592).contains(&at) {
            state
                .objects
                .write_byte(at - 0x370, value)
                .expect("owned object projection");
            return true;
        }
        let write_word = |word: &mut u16, part: usize| {
            let mut bytes = word.to_le_bytes();
            bytes[part] = value;
            *word = u16::from_le_bytes(bytes);
        };
        let mut found = false;
        let mut select_byte = |base: usize, byte: &mut u8| {
            if base == at {
                *byte = value;
                found = true;
            }
        };
        display_bytes!(state,select_byte,(&mut));
        let mut select_word = |base: usize, word: &mut u16| {
            if (base..base + 2).contains(&at) {
                write_word(word, at - base);
                found = true;
            }
        };
        display_words!(state,select_word,(&mut));
        if found {
            return true;
        }
        match at {
            0x5b4 => state.nmi.pending = value,
            0x5b5 => state.nmi.frame_byte = value,
            0x5b6..=0x5b7 => write_word(&mut state.nmi.frame_word, at - 0x5b6),
            0x5b8..=0x5b9 => write_word(&mut state.nmi.including_lag, at - 0x5b8),
            0x5ba => state.nmi.missed = value,
            0x5bb => state.nmi.max_missed = value,
            0x71d..=0x71e => state.samus[at - 0x71d].enabled = value,
            0x71f..=0x722 => {
                let offset = at - 0x71f;
                write_word(&mut state.samus[offset / 2].pointer, offset % 2);
            }
            0x782 => state.nmi.acknowledgment = value,
            _ => return false,
        }
        true
    }
    pub(crate) fn encode_projection(&self, mut span: impl FnMut(usize, &[u8])) {
        if let Self::Owned { display, .. } = self {
            let state = &display.owner().state;
            let mut emit_byte = |at: usize, byte: &u8| span(at, &[*byte]);
            display_bytes!(state,emit_byte,(&));
            let mut emit_word = |at: usize, word: &u16| span(at, &word.to_le_bytes());
            display_words!(state,emit_word,(&));
            for range in [0x370..0x592, 0x5b4..0x5bc, 0x71d..0x723, 0x782..0x783] {
                for at in range {
                    span(
                        at,
                        &[self.projected_byte(at).expect("owned display projection")],
                    );
                }
            }
        }
    }
    pub(crate) fn legacy() -> Self {
        Self::Legacy(Box::new(LegacyPresentation {
            visual: Default::default(),
            palette: Default::default(),
            atmospheric: Default::default(),
            actor_entries: [0; 114],
            actor_sizes: [0; 8],
        }))
    }
    pub(crate) fn visual(&self) -> &crate::game::visual_state::Visuals {
        match self {
            Self::Legacy(state) => &state.visual,
            Self::Owned { display, .. } => &display.owner().state.visual,
        }
    }
    pub(crate) fn visual_mut(&mut self) -> &mut crate::game::visual_state::Visuals {
        match self {
            Self::Legacy(state) => &mut state.visual,
            Self::Owned { display, .. } => &mut display.owner_mut().state.visual,
        }
    }
    pub(crate) fn palette(&self) -> &crate::game::visual_state::PlayerPalette {
        match self {
            Self::Legacy(state) => &state.palette,
            Self::Owned { display, .. } => &display.owner().state.player_palette,
        }
    }
    pub(crate) fn palette_mut(&mut self) -> &mut crate::game::visual_state::PlayerPalette {
        match self {
            Self::Legacy(state) => &mut state.palette,
            Self::Owned { display, .. } => &mut display.owner_mut().state.player_palette,
        }
    }
    pub(crate) fn atmospheric(
        &self,
    ) -> &crate::game::player_transitions::state::AtmosphericEffects {
        match self {
            Self::Legacy(state) => &state.atmospheric,
            Self::Owned { display, .. } => &display.owner().state.atmospheric,
        }
    }
    pub(crate) fn atmospheric_mut(
        &mut self,
    ) -> &mut crate::game::player_transitions::state::AtmosphericEffects {
        match self {
            Self::Legacy(state) => &mut state.atmospheric,
            Self::Owned { display, .. } => &mut display.owner_mut().state.atmospheric,
        }
    }
    pub(crate) fn actor_entry(&self, index: usize) -> &u16 {
        match self {
            Self::Legacy(state) => &state.actor_entries[index],
            Self::Owned {
                display,
                actor_queue_projection,
                ..
            } => {
                let (layer, offset) = queue_lane(actor_queue_projection, index);
                &display.owner().state.actor_queues[layer].actors[offset]
            }
        }
    }
    pub(crate) fn actor_entry_mut(&mut self, index: usize) -> &mut u16 {
        match self {
            Self::Legacy(state) => &mut state.actor_entries[index],
            Self::Owned {
                display,
                actor_queue_projection,
                ..
            } => {
                let (layer, offset) = queue_lane(actor_queue_projection, index);
                &mut display.owner_mut().state.actor_queues[layer].actors[offset]
            }
        }
    }
    pub(crate) fn actor_size(&self, index: usize) -> &u16 {
        match self {
            Self::Legacy(state) => &state.actor_sizes[index],
            Self::Owned { display, .. } => &display.owner().state.actor_queues[index].size,
        }
    }
    pub(crate) fn actor_size_mut(&mut self, index: usize) -> &mut u16 {
        match self {
            Self::Legacy(state) => &mut state.actor_sizes[index],
            Self::Owned { display, .. } => &mut display.owner_mut().state.actor_queues[index].size,
        }
    }
    pub(crate) fn owner(&self) -> Result<&crate::game::scene::commit::OwnedDisplay, DisplayError> {
        match self {
            Self::Legacy(_) => Err(DisplayError::Phase),
            Self::Owned { display, .. } => Ok(display.owner()),
        }
    }
    pub(crate) fn owner_mut(
        &mut self,
    ) -> Result<&mut crate::game::scene::commit::OwnedDisplay, DisplayError> {
        match self {
            Self::Legacy(_) => Err(DisplayError::Phase),
            Self::Owned { display, .. } => Ok(display.owner_mut()),
        }
    }
    pub(crate) fn is_owned(&self) -> bool {
        matches!(self, Self::Owned { .. })
    }
    pub(crate) fn publications(&self) -> Result<&Arc<SamusPublications>, DisplayError> {
        match self {
            Self::Owned {
                samus_publications, ..
            } => Ok(samus_publications),
            Self::Legacy(_) => Err(DisplayError::Phase),
        }
    }
}
fn queue_lane(layout: &[ActorQueueProjection; 8], index: usize) -> (usize, usize) {
    layout
        .iter()
        .enumerate()
        .find_map(|(layer, range)| {
            index
                .checked_sub(range.first)
                .filter(|offset| *offset < range.words)
                .map(|offset| (layer, offset))
        })
        .expect("supplied actor queue projection")
}

use crate::game::scene::samus_content::{PreparedBytes, Registration, Role, RoleLease};
use crate::game::scene::state::{DisplayState, WordOrder};
use crate::game::scene::{samus, samus_body, samus_position_tiles as positions};

struct PreparedTable(RoleLease<PreparedBytes>);
impl positions::Table for PreparedTable {
    type Error = DisplayError;
    fn native_word(&self, offset: usize) -> Result<u16, DisplayError> {
        self.0.native_word(offset)
    }
    fn byte(&self, offset: usize) -> Result<u8, DisplayError> {
        self.0.byte(offset)
    }
}
impl samus_body::Table for PreparedTable {
    type Error = DisplayError;
    fn little_word(&self, offset: usize) -> Result<u16, DisplayError> {
        self.0.little_word(offset)
    }
    fn native_word(&self, offset: usize) -> Result<u16, DisplayError> {
        self.0.native_word(offset)
    }
}

struct BodyInputs<'a> {
    game: &'a mut Game,
    player: &'a mut crate::game::session::Player,
    observations: &'a ExternalInputs,
    publications: Arc<SamusPublications>,
    palette: crate::game::scene::records::MediaRef,
    raster: crate::game::scene::state::RasterState,
}
impl BodyInputs<'_> {
    fn source(
        &self,
        role: Role,
        registration: Registration,
        displacement: usize,
    ) -> Result<PreparedTable, DisplayError> {
        self.game
            .prepared_samus_role(role, registration, displacement)?
            .map(PreparedTable)
            .ok_or(DisplayError::Incomplete)
    }
}
impl samus::Inputs for BodyInputs<'_> {
    fn content(&mut self, _: &DisplayState) -> Result<Arc<samus::SamusCatalog>, DisplayError> {
        self.game
            .samus_spritemaps()?
            .ok_or(DisplayError::Incomplete)
    }
    fn invalid_map(&mut self, _: &mut DisplayState, cursor: u16) -> Result<(), DisplayError> {
        crate::game::session_runtime::native_rom_diagnostic(
            0x920000 | u32::from(cursor),
            self.game,
        );
        Ok(())
    }
    fn object_context(
        &mut self,
        controls: &crate::game::scene::state::DisplayControls,
        _: &crate::game::scene::state::EffectiveDisplayDescription,
    ) -> Result<crate::game::scene::records::ObjectContext, DisplayError> {
        use crate::game::scene::records::{ObjectContext, ObjectSelection};
        let atlases = *self
            .publications
            .object_atlases
            .get(&controls.object_layout)
            .ok_or(DisplayError::Incomplete)?;
        Ok(ObjectContext {
            selection: ObjectSelection {
                atlases,
                size_set: controls.object_layout >> 5,
                first_order: self.publications.object_first_order,
                interlaced: self.raster.interlaced,
                even_frame: self.raster.even_frame,
            },
            palette: self.palette,
        })
    }
}
impl positions::Inputs for BodyInputs<'_> {
    type Table = PreparedTable;
    fn ceres(&mut self) -> Result<u16, DisplayError> {
        Ok(self.observations.ceres)
    }
    fn player_x(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.x)
    }
    fn player_y(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.y)
    }
    fn set_player_x(&mut self, _: &mut DisplayState, value: u16) -> Result<(), DisplayError> {
        self.player.x = value;
        Ok(())
    }
    fn set_player_y(&mut self, _: &mut DisplayState, value: u16) -> Result<(), DisplayError> {
        self.player.y = value;
        Ok(())
    }
    fn project(
        &mut self,
        state: &mut DisplayState,
        x: u16,
        y: u16,
    ) -> Result<(u16, u16), DisplayError> {
        let point = crate::game::visual_runtime::project_cinematic_point_reads(
            crate::types::Point16U { x, y },
            |at| state.controls.affine_words[(at - 0x78) / 2],
        );
        Ok((point.x, point.y))
    }
    fn animation_frame(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.animation_frame())
    }
    fn movement(&mut self) -> Result<u8, DisplayError> {
        Ok(self.player.movement_and_facing.to_le_bytes()[1])
    }
    fn landing_order(&self) -> WordOrder {
        self.source(Role::Landing, Registration::Native, 0)
            .expect("Landing content")
            .0
            .native_order()
    }
    fn landing_offset(&mut self, offset: usize) -> Result<u16, DisplayError> {
        self.source(Role::Landing, Registration::Native, 0)?
            .0
            .native_word(offset)
    }
    fn crouching_offset(&mut self, offset: usize) -> Result<i8, DisplayError> {
        Ok(self
            .source(Role::Crouching, Registration::Native, 0)?
            .0
            .byte(offset)? as i8)
    }
    fn drained_offset(&mut self, offset: usize) -> Result<i8, DisplayError> {
        Ok(self
            .source(Role::Drained, Registration::Native, 0)?
            .0
            .byte(offset)? as i8)
    }
    fn position_source(&mut self) -> Result<PreparedTable, DisplayError> {
        self.source(Role::PositionFallback, Registration::Native, 0)
    }
    fn camera_x(&mut self) -> Result<u16, DisplayError> {
        Ok(self.observations.camera_x)
    }
    fn camera_y(&mut self) -> Result<u16, DisplayError> {
        Ok(self.observations.camera_y)
    }
    fn set_body_x(&mut self, _: &mut DisplayState, value: u16) -> Result<(), DisplayError> {
        self.player.body_position.x = value;
        Ok(())
    }
    fn set_body_y(&mut self, _: &mut DisplayState, value: u16) -> Result<(), DisplayError> {
        self.player.body_position.y = value;
        Ok(())
    }
    fn body_x(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.body_position.x)
    }
    fn body_y(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.body_position.y)
    }
    fn pose(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.pose)
    }
    fn tile_table(&mut self, which: positions::TileTable) -> Result<PreparedTable, DisplayError> {
        let role = match which {
            positions::TileTable::Pose => Role::TilePose,
            positions::TileTable::Top => Role::TileTop,
            positions::TileTable::Bottom => Role::TileBottom,
        };
        self.source(role, Registration::Loaded, 0)
    }
    fn tile_definition(
        &mut self,
        _: &mut DisplayState,
        cursor: u16,
    ) -> Result<PreparedTable, DisplayError> {
        if cursor & 0x8000 == 0 {
            crate::game::session_runtime::native_rom_diagnostic(
                0x920000 | u32::from(cursor),
                self.game,
            );
        }
        let mapped = crate::rtl::lorom_offset(0x920000 | u32::from(cursor)) as usize;
        let base = self
            .game
            .samus_role_base(Role::TileDefinition, Registration::Native)?
            .ok_or(DisplayError::Incomplete)?;
        self.source(
            Role::TileDefinition,
            Registration::Native,
            mapped.checked_sub(base).ok_or(DisplayError::Source)?,
        )
    }
    fn request_word(
        &mut self,
        state: &mut DisplayState,
        half: samus_body::Half,
        pointer: u16,
    ) -> Result<(), DisplayError> {
        let slot = match half {
            samus_body::Half::Top => 0,
            samus_body::Half::Bottom => 1,
        };
        let [low, high] = pointer.to_le_bytes();
        let mut bytes = state.samus[slot].pointer.to_le_bytes();
        bytes[0] = low;
        state.samus[slot].pointer = u16::from_le_bytes(bytes);
        bytes[1] = high;
        state.samus[slot].pointer = u16::from_le_bytes(bytes);
        Ok(())
    }
    fn request_flag(
        &mut self,
        state: &mut DisplayState,
        half: samus_body::Half,
    ) -> Result<(), DisplayError> {
        state.samus[match half {
            samus_body::Half::Top => 0,
            samus_body::Half::Bottom => 1,
        }]
        .enabled = 1;
        Ok(())
    }
    fn tile_request(
        &mut self,
        _: &DisplayState,
        half: samus_body::Half,
        pointer: u16,
    ) -> Result<positions::TileRequest, DisplayError> {
        self.game
            .samus_request(half, pointer)?
            .ok_or(DisplayError::Incomplete)
    }
}
impl samus_body::Inputs for BodyInputs<'_> {
    type BodyTable = PreparedTable;
    fn knockback(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.knockback_timer)
    }
    fn invincibility(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.invincibility)
    }
    fn shine(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.shine_timer)
    }
    fn frame_counter(&mut self) -> Result<u16, DisplayError> {
        Ok(self.observations.frame)
    }
    fn table(&mut self, half: samus_body::Half) -> Result<PreparedTable, DisplayError> {
        self.source(
            match half {
                samus_body::Half::Top => Role::BodyTop,
                samus_body::Half::Bottom => Role::BodyBottom,
            },
            Registration::Loaded,
            0,
        )
    }
    fn set_sprite(&mut self, half: samus_body::Half, index: u16) -> Result<(), DisplayError> {
        match half {
            samus_body::Half::Top => self.player.top_sprite = index,
            samus_body::Half::Bottom => self.player.bottom_sprite = index,
        };
        Ok(())
    }
}

pub(crate) fn draw_body(
    owner: &mut crate::game::scene::commit::OwnedDisplay,
    game: &mut Game,
    player: &mut crate::game::session::Player,
    observations: &ExternalInputs,
    publications: Arc<SamusPublications>,
    marker: &MarkerPublication,
    mode: samus_body::Body,
) -> Result<(), DisplayError> {
    let media = owner.media().clone();
    let producer = publications.body;
    let mut inputs = BodyInputs {
        game,
        player,
        observations,
        publications,
        palette: owner.image().palette,
        raster: owner.state.raster,
    };
    samus_body::draw_owned(
        &mut owner.state,
        &media,
        producer,
        marker,
        mode,
        &mut inputs,
    )
}
pub(crate) fn bottom_visible(
    owner: &mut crate::game::scene::commit::OwnedDisplay,
    game: &mut Game,
    player: &mut crate::game::session::Player,
    observations: &ExternalInputs,
    publications: Arc<SamusPublications>,
    marker: &MarkerPublication,
) -> Result<bool, DisplayError> {
    let media = owner.media().clone();
    let producer = publications.body;
    let mut inputs = BodyInputs {
        game,
        player,
        observations,
        publications,
        palette: owner.image().palette,
        raster: owner.state.raster,
    };
    samus_body::bottom_owned(&mut owner.state, &media, producer, marker, &mut inputs)
}
pub(crate) fn draw_sprite(
    owner: &mut crate::game::scene::commit::OwnedDisplay,
    game: &mut Game,
    player: &mut crate::game::session::Player,
    observations: &ExternalInputs,
    publications: Arc<SamusPublications>,
    producer: crate::game::scene::records::ProducerObjectId,
    index: u16,
    position: (u16, u16),
) -> Result<(), DisplayError> {
    let media = owner.media().clone();
    let mut inputs = BodyInputs {
        game,
        player,
        observations,
        publications,
        palette: owner.image().palette,
        raster: owner.state.raster,
    };
    samus::draw_owned(
        &mut owner.state,
        &media,
        producer,
        index,
        position,
        &mut inputs,
    )
}
