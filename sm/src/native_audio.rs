//! Original upload handshake through the emulated APU bus.
use crate::game_state::Game;

/// Requires the selected live APU/SPC backend for the ordered upload.
pub(crate) unsafe fn upload_bank(addr: u32, game: &mut Game) {
    unsafe {
        if !game.g_use_my_apu_code {
            upload(addr, game);
            return;
        }
        crate::rtl::RtlApuUpload(addr, game);
    }
}
// A transient bound value; it owns no live queue or competing audio state.
#[derive(Clone, Copy)]
enum OriginalEffect {
    Initialize,
    Bank(sm_data::OriginalMusicBank),
    Track(sm_data::OriginalMusicTrack),
    Sound(u8, sm_data::OriginalSoundEffect),
}
enum Meaning<'a> {
    None,
    Music(&'a crate::game_state::persistent_session::QueuedMusicMeaning),
    Sound(&'a crate::game_state::persistent_session::QueuedSoundMeaning),
}
pub(crate) struct CompatibilityCue<'a> {
    cue: Option<sm_data::Cue>,
    original: OriginalEffect,
    meaning: Meaning<'a>,
}
#[forbid(unsafe_code)]
pub(crate) fn initialize_cue() -> CompatibilityCue<'static> {
    CompatibilityCue {
        cue: Some(sm_data::Cue {
            source: None,
            action: sm_data::CueAction::InitializeAudio,
        }),
        original: OriginalEffect::Initialize,
        meaning: Meaning::None,
    }
}
#[forbid(unsafe_code)]
pub(crate) fn music_bank_cue(
    selector: sm_data::OriginalMusicBank,
    meaning: Option<&crate::game_state::persistent_session::QueuedMusicMeaning>,
) -> CompatibilityCue<'_> {
    let meaning =
        meaning.filter(|record| record.request & 0x8000 != 0 && record.request as u8 == selector.0);
    let source = meaning.map(|record| sm_data::CueSource {
        source_tick: record.source_tick,
        source_interval: record.source_interval,
    });
    CompatibilityCue {
        cue: Some(sm_data::Cue::music_bank(
            selector,
            source,
            meaning.map(|record| &record.meaning),
        )),
        original: OriginalEffect::Bank(selector),
        meaning: meaning.map_or(Meaning::None, Meaning::Music),
    }
}
#[forbid(unsafe_code)]
pub(crate) fn music_cue(
    selector: sm_data::OriginalMusicTrack,
    meaning: Option<&crate::game_state::persistent_session::QueuedMusicMeaning>,
) -> CompatibilityCue<'_> {
    let meaning = meaning
        .filter(|record| record.request & 0x8000 == 0 && record.request as u8 & 0x7f == selector.0);
    let source = meaning.map(|record| sm_data::CueSource {
        source_tick: record.source_tick,
        source_interval: record.source_interval,
    });
    CompatibilityCue {
        cue: Some(sm_data::Cue::music(
            selector,
            source,
            meaning.map(|record| &record.meaning),
        )),
        original: OriginalEffect::Track(selector),
        meaning: meaning.map_or(Meaning::None, Meaning::Music),
    }
}
#[forbid(unsafe_code)]
pub(crate) fn sound_cue(
    channel: u8,
    selector: sm_data::OriginalSoundEffect,
    meaning: Option<&crate::game_state::persistent_session::QueuedSoundMeaning>,
) -> CompatibilityCue<'_> {
    let meaning = meaning.filter(|record| record.request == selector.0);
    let source = meaning.map(|record| sm_data::CueSource {
        source_tick: record.source_tick,
        source_interval: record.source_interval,
    });
    let cue = sm_data::SoundChannel::from_index(channel).map(|channel| {
        sm_data::Cue::sound(
            channel,
            selector,
            source,
            meaning.map(|record| &record.meaning),
        )
    });
    CompatibilityCue {
        cue,
        original: OriginalEffect::Sound(channel, selector),
        meaning: meaning.map_or(Meaning::None, Meaning::Sound),
    }
}

#[forbid(unsafe_code)]
fn resolve_cue(emitted: Option<&sm_data::Cue>, cue: &CompatibilityCue<'_>) -> OriginalEffect {
    use sm_data::{CueAction, MusicBankCue, MusicCue, MusicMeaning, SoundCue};
    match (emitted.map(|cue| &cue.action), &cue.meaning) {
        (Some(CueAction::InitializeAudio), _) => OriginalEffect::Initialize,
        (Some(CueAction::MusicBank(MusicBankCue::Original(selector))), _) => {
            OriginalEffect::Bank(*selector)
        }
        (Some(CueAction::Music(MusicCue::Original(selector))), _) => {
            OriginalEffect::Track(*selector)
        }
        (
            Some(CueAction::Sound {
                channel,
                sound: SoundCue::Original(selector),
            }),
            _,
        ) => OriginalEffect::Sound(channel.index(), *selector),
        (
            Some(CueAction::MusicBank(MusicBankCue::Authored { bank, after })),
            Meaning::Music(record),
        ) if record.meaning
            == (MusicMeaning::Bank {
                bank: *bank,
                after: after.clone(),
            }) =>
        {
            OriginalEffect::Bank(sm_data::OriginalMusicBank(record.request as u8))
        }
        (Some(CueAction::Music(MusicCue::Authored(meaning))), Meaning::Music(record))
            if record.meaning == *meaning =>
        {
            OriginalEffect::Track(sm_data::OriginalMusicTrack(record.request as u8 & 0x7f))
        }
        (
            Some(CueAction::Sound {
                channel,
                sound: SoundCue::Authored(meaning),
            }),
            Meaning::Sound(record),
        ) if record.meaning == *meaning => OriginalEffect::Sound(
            channel.index(),
            sm_data::OriginalSoundEffect(record.request),
        ),
        // Foreign channels or uncorrelated supplied meanings retain the actual original effect.
        _ => cue.original,
    }
}
#[forbid(unsafe_code)]
fn bank_address(selector: sm_data::OriginalMusicBank, game: &Game) -> u32 {
    let offset = usize::from(selector.0);
    // The original Load24 also reads the adjacent fourth byte before masking.
    u32::from_ne_bytes(core::array::from_fn(|i| {
        game.read_rom_byte(0x8fe7e1, offset + i)
    })) & 0xffffff
}

/// Retains the original live/exclusive APU/SPC and native source-bus contracts.
/// Cue resolution runs at the original callback phase; no delivery is deferred.
pub(crate) unsafe fn emit(
    mut cue: CompatibilityCue<'_>,
    phase: sm_data::CuePhase,
    game: &mut Game,
) {
    unsafe {
        let request = game
            .audio_output
            .accept_cue(cue.cue.take().map(|cue| sm_data::EmittedCue { cue, phase }));
        match resolve_cue(request.emission.as_ref().map(|value| &value.cue), &cue) {
            OriginalEffect::Initialize => upload_bank(0xcf8000, game),
            OriginalEffect::Bank(selector) => upload_bank(bank_address(selector, game), game),
            OriginalEffect::Track(selector) => {
                crate::rtl::RtlApuWrite(crate::snes::registers::APUI00, selector.0, game)
            }
            OriginalEffect::Sound(channel, selector) => crate::rtl::RtlApuWrite(
                crate::snes::registers::APUI01 + u32::from(channel),
                selector.0,
                game,
            ),
        }
        game.audio_output.complete_cue(request);
    }
}

/// The selected physical backend must remain live and exclusive throughout the
/// upload. Device/native callbacks must not recover Game or retain device borrows;
/// source bus reads and completion writes keep their existing native contracts.
pub(crate) unsafe fn upload(source: u32, game: &mut Game) {
    unsafe {
        crate::audio_upload::prepare_physical(source, game.ram());
        let ram = game.ram().share_for_backend();
        let mut execution = game.execution();
        crate::audio_upload::physical(source, &ram, |operation| {
            use crate::audio_upload::PhysicalAccess::*;
            let (port, value) = match operation {
                ReadSource(at) => return execution.read(at),
                ReadPort0 => return execution.read_bbus(0x40),
                ReadPort1 => return execution.read_bbus(0x41),
                WritePort0(value) => (0, value),
                WritePort1(value) => (1, value),
                WritePort2(value) => (2, value),
                WritePort3(value) => (3, value),
                CompletePort1(source) => {
                    execution.write((source & 0xff0000) | 0x2141, 0);
                    return 0;
                }
                CompletePort2(source) => {
                    execution.write((source & 0xff0000) | 0x2142, 0);
                    return 0;
                }
                CompletePort3(source) => {
                    execution.write((source & 0xff0000) | 0x2143, 0);
                    return 0;
                }
            };
            crate::rtl::apu_write(&mut execution, 0x2140 + port, value);
            0
        });
    }
}

/// Preallocated stereo PCM with the byte cursor used by device delivery.
/// The first block must be supplied by the producer; later skipped refills retain
/// its previous PCM, as the original callback did. Storage is never media data.
pub struct SampleDelivery {
    samples: Vec<[i16; 2]>,
    cursor: usize,
    initialized: bool,
}

#[forbid(unsafe_code)]
impl SampleDelivery {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            cursor: 0,
            initialized: false,
        }
    }

    /// Set up outside the callback, before the device is unpaused.
    pub fn prepare(&mut self, frames: usize) {
        assert!(frames > 0, "audio block must contain frames");
        assert!(frames <= usize::MAX / 4, "audio block byte count overflow");
        self.samples = vec![[0; 2]; frames];
        self.cursor = frames * 4;
        self.initialized = false;
    }

    /// Release after device pause/close has joined the callback.
    pub fn release(&mut self) {
        *self = Self::new();
    }

    fn byte(&self, position: usize) -> u8 {
        self.samples[position / 4][(position / 2) & 1].to_ne_bytes()[position & 1]
    }

    /// Deliver to an initialized caller buffer. Device adapters use deliver_to
    /// so a writable device allocation need not contain initialized u8 values.
    pub fn deliver(
        &mut self,
        output: &mut [u8],
        volume: u8,
        refill: impl FnMut(&mut [[i16; 2]]) -> bool,
    ) -> bool {
        self.deliver_to(output.len(), volume, refill, |index, value| {
            output[index] = value
        })
    }

    /// Deliver without allocating or reading the destination. Every byte write
    /// receives an index below length. Refill reports whether it produced PCM.
    /// Missing first PCM preserves the caller's bytes and reports false; later
    /// skipped refills retain the previous produced block. Odd scaled tails are
    /// zeroed as in SDL's S16LSB mixer, whose word reads start at the byte cursor.
    pub fn deliver_to(
        &mut self,
        mut length: usize,
        volume: u8,
        mut refill: impl FnMut(&mut [[i16; 2]]) -> bool,
        mut write: impl FnMut(usize, u8),
    ) -> bool {
        let bytes = self.samples.len() * 4;
        let mut offset = 0;
        let mut available = true;
        while length != 0 {
            assert!(bytes != 0, "audio device must be prepared");
            if self.cursor == bytes {
                let produced = refill(&mut self.samples);
                self.initialized |= produced;
                self.cursor = 0;
            }
            let count = length.min(bytes - self.cursor);
            if !self.initialized {
                available = false;
                if volume == 0 {
                    for index in 0..count {
                        write(offset + index, 0);
                    }
                }
            } else if volume == 128 {
                for index in 0..count {
                    write(offset + index, self.byte(self.cursor + index));
                }
            } else {
                for index in 0..count {
                    write(offset + index, 0);
                }
                if volume != 0 {
                    for index in 0..count / 2 {
                        let position = self.cursor + index * 2;
                        let sample =
                            i16::from_le_bytes([self.byte(position), self.byte(position + 1)]);
                        // SDL narrows the scaled source to Sint16 before mixing
                        // into the zero destination; division truncates to zero.
                        let sample = ((i32::from(sample) * i32::from(volume)) / 128) as i16;
                        let encoded = sample.to_le_bytes();
                        write(offset + index * 2, encoded[0]);
                        write(offset + index * 2 + 1, encoded[1]);
                    }
                }
            }
            self.cursor += count;
            offset += count;
            length -= count;
        }
        available
    }
}
