//! Pure exclusive Session control and presentation owners and their projections.
//! Backend adapters stay in persistent_session; no native Game is required here.
use crate::scene::{
    component::{DisplayComponent, DisplayStartMetadata},
    samus_body::MarkerPublication,
    samus_content::ContentDescription,
    start::DisplayStartRequirements,
    state::DisplayError,
};
use std::{collections::BTreeMap, sync::Arc};

/// Independently supplied compatibility layout for the eight current actor queues.
/// Offsets are word indices in the original 114-word queue workspace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActorQueueProjection {
    pub first: usize,
    pub words: usize,
}

/// Authored bindings and live producer lifetimes supplied independently of OAM.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SamusPublications {
    pub body: crate::scene::records::ProducerObjectId,
    pub echoes: [Option<crate::scene::records::ProducerObjectId>; 4],
    pub object_atlases: BTreeMap<u8, [crate::scene::records::AssetRef; 2]>,
    pub object_first_order: Option<u8>,
}

pub use sm_data::{MusicContinuation, MusicMeaning, SoundMeaning};
/// Declared command interpretation, independent of the live request bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioRequestCatalog {
    pub music: BTreeMap<u16, MusicMeaning>,
    pub sounds: BTreeMap<(usize, u8), SoundMeaning>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueuedMusicMeaning {
    pub request: u16,
    pub source_tick: u64,
    pub source_interval: u64,
    pub meaning: MusicMeaning,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueuedSoundMeaning {
    pub request: u8,
    pub source_tick: u64,
    pub source_interval: u64,
    pub meaning: SoundMeaning,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MusicRequests {
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
pub struct SoundRequests {
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
pub struct AudioRequests {
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
    pub fn projected_byte(&self, at: usize) -> Option<u8> {
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
    pub fn write_projected_byte(&mut self, at: usize, byte: u8) -> bool {
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
    pub fn encode_projection(&self, mut span: impl FnMut(usize, &[u8])) {
        let mut emit_byte = |at: usize, byte: &u8| span(at, &[*byte]);
        audio_bytes!(self,emit_byte,(&));
        let mut emit_word = |at: usize, word: &u16| span(at, &word.to_le_bytes());
        audio_words!(self,emit_word,(&));
    }
    pub fn validate(
        &self,
        metadata: &DisplayStartMetadata,
        required: &AudioRequestCatalog,
    ) -> Result<(), DisplayError> {
        if self.pending_mutation {
            return Err(DisplayError::Incomplete);
        }
        self.validate_at_tick(metadata, required, metadata.identity.committed_tick)
    }
    pub fn validate_at_tick(
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
        let declared = |asset: crate::scene::records::AssetRef, bank: bool| {
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
            let matches_music = |program: crate::scene::records::AssetRef| {
                matches!(metadata.audio.music, crate::scene::start::OngoingMusic::Playing(anchor)
                    if anchor.program == program.id && anchor.revision == program.revision)
            };
            let valid = match &record.meaning {
                MusicMeaning::Silence => {
                    metadata.audio.music == crate::scene::start::OngoingMusic::Silent
                }
                MusicMeaning::Program { program, .. } => matches_music(*program),
                MusicMeaning::Bank { after, .. } => match after {
                    MusicContinuation::Retain => true,
                    MusicContinuation::Silence => {
                        metadata.audio.music == crate::scene::start::OngoingMusic::Silent
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
    pub fn publish_music(
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
    pub fn publish_sound(
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
    pub fn clear_music(&mut self, slot: usize) -> Result<(), DisplayError> {
        let number = self.music.tracks.get(slot).ok_or(DisplayError::Source)?;
        if *number != 0 || self.music.delays[slot] != 0 {
            return Err(DisplayError::Identity);
        }
        self.music.meanings[slot] = None;
        self.pending_mutation = true;
        Ok(())
    }
    pub fn clear_sound(&mut self, channel: usize, slot: usize) -> Result<(), DisplayError> {
        let sound = self.sounds.get_mut(channel).ok_or(DisplayError::Source)?;
        if *sound.queue.get(slot).ok_or(DisplayError::Source)? != 0 {
            return Err(DisplayError::Identity);
        }
        sound.meanings[slot] = None;
        self.pending_mutation = true;
        Ok(())
    }
    pub fn capture_music(
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
    pub fn capture_sound(
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
    pub fn select_music_entry(&mut self, record: QueuedMusicMeaning) -> Result<(), DisplayError> {
        if self.music.entry != record.request {
            return Err(DisplayError::Identity);
        }
        self.music.entry_meaning = Some(record);
        self.pending_mutation = true;
        Ok(())
    }
    pub fn select_current_sound(
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
    pub fn clear_music_entry(&mut self) -> Result<(), DisplayError> {
        if self.music.entry != 0 {
            return Err(DisplayError::Identity);
        }
        self.music.entry_meaning = None;
        self.pending_mutation = true;
        Ok(())
    }
    pub fn clear_current_sound(&mut self, channel: usize) -> Result<(), DisplayError> {
        let sound = self.sounds.get_mut(channel).ok_or(DisplayError::Source)?;
        if sound.current != 0 {
            return Err(DisplayError::Identity);
        }
        sound.current_meaning = None;
        self.pending_mutation = true;
        Ok(())
    }
    pub fn complete_mutation(
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
    pub fn accept_music(
        &mut self,
        audio: &mut crate::scene::start::InitialAudioIntent,
        mut record: QueuedMusicMeaning,
        stamp: AudioStamp,
    ) -> Result<(), DisplayError> {
        if self.catalog.music.get(&record.request) != Some(&record.meaning)
            || record.source_tick > stamp.tick
            || record.source_interval > stamp.source_interval
        {
            return Err(DisplayError::Identity);
        }
        let program = |asset: crate::scene::records::AssetRef, phase| {
            crate::scene::start::OngoingMusic::Playing(crate::scene::start::ProgramAnchor {
                program: asset.id,
                revision: asset.revision,
                source_interval: stamp.source_interval,
                phase,
                timing_policy: audio.timing_policy,
            })
        };
        let next = match &record.meaning {
            MusicMeaning::Silence => crate::scene::start::OngoingMusic::Silent,
            MusicMeaning::Program {
                program: asset,
                phase,
            } => program(*asset, *phase),
            MusicMeaning::Bank { after, .. } => match after {
                MusicContinuation::Retain => audio.music,
                MusicContinuation::Silence => crate::scene::start::OngoingMusic::Silent,
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
pub struct CurrentControl {
    pub metadata: DisplayStartMetadata,
    pub requests: AudioRequests,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioStamp {
    pub tick: u64,
    pub source_interval: u64,
}
impl CurrentControl {
    /// Transfer semantic counterparts only from the actual declared donor.
    /// Numeric writes already completed; any rejection keeps capture suspended.
    pub fn normalize_audio_from(
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
    pub fn audio_mutation<R>(
        &mut self,
        producing: bool,
        operation: impl FnOnce(
            &mut AudioRequests,
            &mut crate::scene::start::InitialAudioIntent,
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
pub trait AudioHost {
    fn audio_byte(&self, at: usize) -> u8;
    fn audio_store_byte(&mut self, at: usize, byte: u8);
    fn audio_mutation<R>(
        &mut self,
        operation: impl FnOnce(
            &mut AudioRequests,
            &mut crate::scene::start::InitialAudioIntent,
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

#[derive(Clone)]
pub struct SessionStartRequirements {
    pub display: DisplayStartRequirements,
    pub roles: ContentDescription,
    pub marker: MarkerPublication,
    pub actor_queue_projection: [ActorQueueProjection; 8],
    pub samus_publications: Arc<SamusPublications>,
    pub audio_catalog: Arc<AudioRequestCatalog>,
}

/// Current input, runner and semantic audio owners, exclusive with legacy bytes.
#[derive(Clone, Debug, PartialEq)]
pub enum Control {
    Legacy(crate::room_entry::state::SharedContinuations),
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
    pub fn requests(&self) -> Result<&AudioRequests, DisplayError> {
        match self {
            Self::Owned(value) => Ok(&value.requests),
            Self::Legacy(_) => Err(DisplayError::Phase),
        }
    }
    pub fn requests_mut(&mut self) -> Result<&mut AudioRequests, DisplayError> {
        match self {
            Self::Owned(value) => Ok(&mut value.requests),
            Self::Legacy(_) => Err(DisplayError::Phase),
        }
    }
    pub fn legacy() -> Self {
        Self::Legacy(crate::room_entry::state::SharedContinuations {
            primary: 0,
            secondary: 0,
        })
    }
    pub fn primary(&self) -> &u8 {
        match self {
            Self::Legacy(value) => &value.primary,
            Self::Owned(value) => &value.metadata.continuation.primary,
        }
    }
    pub fn secondary(&self) -> &u8 {
        match self {
            Self::Legacy(value) => &value.secondary,
            Self::Owned(value) => &value.metadata.continuation.secondary,
        }
    }
    pub fn primary_mut(&mut self) -> &mut u8 {
        match self {
            Self::Legacy(value) => &mut value.primary,
            Self::Owned(value) => &mut value.metadata.continuation.primary,
        }
    }
    pub fn secondary_mut(&mut self) -> &mut u8 {
        match self {
            Self::Legacy(value) => &mut value.secondary,
            Self::Owned(value) => &mut value.metadata.continuation.secondary,
        }
    }
    pub fn metadata(&self) -> Result<&DisplayStartMetadata, DisplayError> {
        match self {
            Self::Owned(value) => Ok(&value.metadata),
            Self::Legacy(_) => Err(DisplayError::Phase),
        }
    }
    pub fn metadata_mut(&mut self) -> Result<&mut DisplayStartMetadata, DisplayError> {
        match self {
            Self::Owned(value) => Ok(&mut value.metadata),
            Self::Legacy(_) => Err(DisplayError::Phase),
        }
    }
    pub fn projected_byte(&self, at: usize) -> Option<u8> {
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
    pub fn write_projected_byte(&mut self, at: usize, byte: u8) -> bool {
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
    pub fn encode_projection(&self, mut span: impl FnMut(usize, &[u8])) {
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
/// Exactly one presentation owner is selected. Legacy numeric construction does
/// not manufacture media, producer identities, Reference or a Display start.
#[derive(Clone, Debug, PartialEq)]
pub enum Presentation {
    Legacy(Box<LegacyPresentation>),
    Owned {
        display: Box<DisplayComponent>,
        marker: MarkerPublication,
        actor_queue_projection: [ActorQueueProjection; 8],
        samus_publications: Arc<SamusPublications>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyPresentation {
    visual: crate::visual_state::Visuals,
    palette: crate::visual_state::PlayerPalette,
    atmospheric: crate::player_transitions::state::AtmosphericEffects,
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
    pub fn requires_semantic_counterpart(at: usize) -> bool {
        // Publishing these numeric requests also needs supplied source/audio
        // identities. Their actual semantic publication seams are not closed.
        (0xd0..0x330).contains(&at) || (0x340..0x360).contains(&at) || (0x956..0x98e).contains(&at)
    }
    pub fn projected_byte(&self, at: usize) -> Option<u8> {
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
    pub fn write_projected_byte(&mut self, at: usize, value: u8) -> bool {
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
    pub fn encode_projection(&self, mut span: impl FnMut(usize, &[u8])) {
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
    pub fn legacy() -> Self {
        Self::Legacy(Box::new(LegacyPresentation {
            visual: Default::default(),
            palette: Default::default(),
            atmospheric: Default::default(),
            actor_entries: [0; 114],
            actor_sizes: [0; 8],
        }))
    }
    pub fn visual(&self) -> &crate::visual_state::Visuals {
        match self {
            Self::Legacy(state) => &state.visual,
            Self::Owned { display, .. } => &display.owner().state.visual,
        }
    }
    pub fn visual_mut(&mut self) -> &mut crate::visual_state::Visuals {
        match self {
            Self::Legacy(state) => &mut state.visual,
            Self::Owned { display, .. } => &mut display.owner_mut().state.visual,
        }
    }
    pub fn palette(&self) -> &crate::visual_state::PlayerPalette {
        match self {
            Self::Legacy(state) => &state.palette,
            Self::Owned { display, .. } => &display.owner().state.player_palette,
        }
    }
    pub fn palette_mut(&mut self) -> &mut crate::visual_state::PlayerPalette {
        match self {
            Self::Legacy(state) => &mut state.palette,
            Self::Owned { display, .. } => &mut display.owner_mut().state.player_palette,
        }
    }
    pub fn atmospheric(&self) -> &crate::player_transitions::state::AtmosphericEffects {
        match self {
            Self::Legacy(state) => &state.atmospheric,
            Self::Owned { display, .. } => &display.owner().state.atmospheric,
        }
    }
    pub fn atmospheric_mut(&mut self) -> &mut crate::player_transitions::state::AtmosphericEffects {
        match self {
            Self::Legacy(state) => &mut state.atmospheric,
            Self::Owned { display, .. } => &mut display.owner_mut().state.atmospheric,
        }
    }
    pub fn actor_entry(&self, index: usize) -> &u16 {
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
    pub fn actor_entry_mut(&mut self, index: usize) -> &mut u16 {
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
    pub fn actor_size(&self, index: usize) -> &u16 {
        match self {
            Self::Legacy(state) => &state.actor_sizes[index],
            Self::Owned { display, .. } => &display.owner().state.actor_queues[index].size,
        }
    }
    pub fn actor_size_mut(&mut self, index: usize) -> &mut u16 {
        match self {
            Self::Legacy(state) => &mut state.actor_sizes[index],
            Self::Owned { display, .. } => &mut display.owner_mut().state.actor_queues[index].size,
        }
    }
    pub fn owner(&self) -> Result<&crate::scene::commit::OwnedDisplay, DisplayError> {
        match self {
            Self::Legacy(_) => Err(DisplayError::Phase),
            Self::Owned { display, .. } => Ok(display.owner()),
        }
    }
    pub fn owner_mut(&mut self) -> Result<&mut crate::scene::commit::OwnedDisplay, DisplayError> {
        match self {
            Self::Legacy(_) => Err(DisplayError::Phase),
            Self::Owned { display, .. } => Ok(display.owner_mut()),
        }
    }
    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned { .. })
    }
    pub fn publications(&self) -> Result<&Arc<SamusPublications>, DisplayError> {
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
