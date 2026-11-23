use data::assets::audio::{
    Articulation, EffectVoiceId, MusicEvent, MusicTurns, MusicalPitch, PitchOffset, TrackContent,
    VoiceId,
};
use std::num::NonZeroU8;
use std::ops::Range;

use super::ImportError;
use super::source::{Reader, Region, Resources, Upload};

const ARTICULATION: usize = 0x5800;
// The Ceres upload supplies four area tracks after the shared track entries.
const FIRST_AREA_TRACK: u8 = 5;
const AREA_TRACKS: usize = 0x5828;
const AREA_DIRECTORY_END: usize = 0x5830;
const VOICES: [VoiceId; 8] = [
    VoiceId::FirstMusic,
    VoiceId::SecondMusic,
    VoiceId::Effect(EffectVoiceId::V2),
    VoiceId::Effect(EffectVoiceId::V3),
    VoiceId::Effect(EffectVoiceId::V4),
    VoiceId::Effect(EffectVoiceId::V5),
    VoiceId::Effect(EffectVoiceId::V6),
    VoiceId::Effect(EffectVoiceId::V7),
];

pub(super) fn decode(
    area: &Upload<'_>,
    track: u8,
    resources: &mut Resources<'_, '_>,
) -> Result<(Vec<TrackContent>, u8), ImportError> {
    let song = area.block(AREA_TRACKS)?;
    let tables = area.slice(ARTICULATION..ARTICULATION.saturating_add(24))?;
    let slot = usize::from(
        track
            .checked_sub(FIRST_AREA_TRACK)
            .ok_or(ImportError::Arrangement)?,
    );
    let directory = song.slice(AREA_TRACKS..AREA_DIRECTORY_END)?;
    let arrangement = directory.word(AREA_TRACKS.saturating_add(slot.saturating_mul(2)))?;
    let mut reader = Reader {
        region: song.slice(AREA_DIRECTORY_END..song.end())?,
        at: arrangement,
    };
    let [
        table_lo,
        table_hi,
        repeat,
        high,
        target_lo,
        target_hi,
        end_lo,
        end_hi,
    ] = reader.take()?;
    let word = |lo, hi| usize::from(u16::from_le_bytes([lo, hi]));
    if [repeat, high, end_lo, end_hi] != [0xff, 0, 0, 0]
        || word(target_lo, target_hi) != arrangement
    {
        return Err(ImportError::Arrangement);
    }
    let table = word(table_lo, table_hi);
    if table < reader.at {
        return Err(ImportError::Arrangement);
    }
    let [a_lo, a_hi, b_lo, b_hi, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] = song.get(table)? else {
        return Err(ImportError::Arrangement);
    };
    let event_start = table.saturating_add(16);
    let mut decoder = Tracks {
        song: song.slice(event_start..song.end())?,
        tables,
        echo_delay: None,
        used: Vec::new(),
    };
    let tracks = [word(a_lo, a_hi), word(b_lo, b_hi)]
        .into_iter()
        .map(|root| decoder.track(root, resources))
        .collect::<Result<_, _>>()?;
    Ok((tracks, decoder.echo_delay.ok_or(ImportError::Music)?))
}

struct Tracks<'rom> {
    song: Region<'rom>,
    tables: Region<'rom>,
    echo_delay: Option<u8>,
    used: Vec<Range<usize>>,
}

impl Tracks<'_> {
    fn track(
        &mut self,
        root: usize,
        resources: &mut Resources<'_, '_>,
    ) -> Result<TrackContent, ImportError> {
        let mut targets = Vec::new();
        let first = self.phrase(root, Some(&mut targets), resources)?;
        let mut phrases = vec![first];
        for target in targets {
            phrases.push(self.phrase(target, None, resources)?);
        }
        Ok(TrackContent { phrases })
    }

    fn phrase(
        &mut self,
        start: usize,
        mut targets: Option<&mut Vec<usize>>,
        resources: &mut Resources<'_, '_>,
    ) -> Result<Vec<MusicEvent>, ImportError> {
        let mut reader = Reader {
            region: self.song,
            at: start,
        };
        let mut events = Vec::new();
        let mut timed = false;
        loop {
            let [opcode] = reader.take()?;
            let event = match opcode {
                0 => MusicEvent::Complete,
                1..=0x7f => self.timing(opcode, &mut reader)?,
                0x80..=0xc7 => {
                    MusicEvent::Tone(MusicalPitch::new(opcode & 0x7f).ok_or(ImportError::Music)?)
                }
                0xc8 => MusicEvent::Sustain,
                0xef => {
                    let [lo, hi, count] = reader.take()?;
                    let target = usize::from(u16::from_le_bytes([lo, hi]));
                    let calls = targets.as_deref_mut().ok_or(ImportError::Music)?;
                    if !timed || target == start {
                        return Err(ImportError::Music);
                    }
                    let phrase = calls
                        .iter()
                        .position(|&previous| previous == target)
                        .unwrap_or_else(|| {
                            calls.push(target);
                            calls.len().saturating_sub(1)
                        });
                    MusicEvent::Repeat {
                        phrase: phrase.saturating_add(1),
                        count: NonZeroU8::new(count).ok_or(ImportError::Music)?,
                    }
                }
                _ => self.control(opcode, &mut reader, resources)?,
            };
            let sounding = matches!(event, MusicEvent::Tone(_) | MusicEvent::Sustain);
            if matches!(events.last(), Some(MusicEvent::Timing { .. })) && !sounding {
                return Err(ImportError::Music);
            }
            timed |= sounding;
            let complete = event == MusicEvent::Complete;
            events.push(event);
            if complete {
                break;
            }
        }
        if !timed
            || self
                .used
                .iter()
                .any(|old| start < old.end && old.start < reader.at)
        {
            return Err(ImportError::Music);
        }
        self.used.push(start..reader.at);
        Ok(events)
    }

    fn timing(&self, opcode: u8, reader: &mut Reader<'_>) -> Result<MusicEvent, ImportError> {
        let [next] = reader.region.get(reader.at)?;
        let articulation = if next < 0x80 {
            reader.take::<1>()?;
            let [ratio] = self
                .tables
                .get(ARTICULATION.saturating_add(usize::from(next >> 4)))?;
            let [strength] = self.tables.get(
                ARTICULATION
                    .saturating_add(8)
                    .saturating_add(usize::from(next & 0xf)),
            )?;
            Some(Articulation { ratio, strength })
        } else {
            None
        };
        Ok(MusicEvent::Timing {
            duration: MusicTurns(NonZeroU8::new(opcode).ok_or(ImportError::Music)?),
            articulation,
        })
    }

    fn control(
        &mut self,
        opcode: u8,
        reader: &mut Reader<'_>,
        resources: &mut Resources<'_, '_>,
    ) -> Result<MusicEvent, ImportError> {
        let event = match opcode {
            0xe0 => {
                let [instrument] = reader.take()?;
                if instrument == 0 {
                    return Err(ImportError::Music);
                }
                MusicEvent::Timbre(resources.instrument(instrument)?)
            }
            0xe5 => {
                let [value] = reader.take()?;
                MusicEvent::MasterLoudness(value)
            }
            0xed => {
                let [value] = reader.take()?;
                MusicEvent::Loudness(value)
            }
            0xee => {
                let [turns, target] = reader.take()?;
                MusicEvent::Ramp {
                    turns: NonZeroU8::new(turns).ok_or(ImportError::Music)?,
                    target,
                }
            }
            0xf4 if reader.take::<1>()? == [0] => MusicEvent::Offset(PitchOffset::Zero),
            0xf5 => {
                let [members, left, right] = reader.take()?;
                if (left | right) & 0x80 != 0 {
                    return Err(ImportError::Music);
                }
                let voices = VOICES
                    .into_iter()
                    .zip(0..8)
                    .filter_map(|(voice, bit)| (members & (1 << bit) != 0).then_some(voice))
                    .collect();
                MusicEvent::EchoMix {
                    voices,
                    left,
                    right,
                }
            }
            0xf7 => {
                let [delay, feedback, filter] = reader.take()?;
                if delay > 15
                    || feedback > 127
                    || filter != 0
                    || *self.echo_delay.get_or_insert(delay) != delay
                {
                    return Err(ImportError::Music);
                }
                MusicEvent::EchoSpace {
                    delay,
                    feedback,
                    filter: u16::from(filter),
                }
            }
            0xfa => {
                let [percussion] = reader.take()?;
                MusicEvent::Percussion(u16::from(percussion))
            }
            _ => return Err(ImportError::Music),
        };
        Ok(event)
    }
}

#[cfg(test)]
mod tests;
