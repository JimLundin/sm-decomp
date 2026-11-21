use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU8;
use std::ops::Range;

use data::assets::audio::{
    CueAlias, CueId, EffectClip, EffectPartContent, EffectPitch, EffectSpan, EffectTurns,
    PitchMotion, SoundGroup,
};

use super::ImportError;
use super::source::{Reader, Region, Resources, Upload};

#[derive(Clone, Copy)]
struct CueSource {
    alias: CueAlias,
    parts: usize,
}

// Physical voice allocations come from each cue's definition. The pointer set's
// extent independently delimits the admitted parts for these contiguous lists.
// Order defines dense timbre identity after the song's instruments.
const CUES: [CueSource; 8] = [
    cue(SoundGroup::Contact, 6, CueId::Footstep, 1),
    cue(SoundGroup::Action, 49, CueId::Spin, 1),
    cue(SoundGroup::Action, 50, CueId::EndSpin, 1),
    cue(SoundGroup::Contact, 5, CueId::Land, 2),
    cue(SoundGroup::Action, 11, CueId::Beam, 1),
    cue(SoundGroup::Action, 2, CueId::SamusSilence, 1),
    cue(SoundGroup::Ambient, 113, CueId::BackgroundSilence, 1),
    cue(SoundGroup::Contact, 44, CueId::Door, 2),
];

const fn cue(group: SoundGroup, byte: u8, cue: CueId, parts: usize) -> CueSource {
    CueSource {
        alias: CueAlias { group, byte, cue },
        parts,
    }
}

struct Library {
    range: Range<usize>,
    directory_end: usize,
}

const fn library(group: SoundGroup) -> Library {
    match group {
        SoundGroup::Action => Library {
            range: 0x2aed..0x3154,
            directory_end: 0x2b71,
        },
        SoundGroup::Ambient => Library {
            range: 0x39b3..0x4703,
            directory_end: 0x3ab1,
        },
        SoundGroup::Contact => Library {
            range: 0x4e8f..0x530e,
            directory_end: 0x4eed,
        },
    }
}

pub(super) fn decode(
    common: &Upload<'_>,
    area: &Upload<'_>,
    resources: &mut Resources<'_, '_>,
) -> Result<(BTreeMap<CueId, EffectClip>, Vec<CueAlias>), ImportError> {
    let mut clips = BTreeMap::new();
    let mut aliases = Vec::new();
    let mut used = Vec::new();
    let mut ports = BTreeSet::new();
    for definition in CUES {
        let alias = definition.alias;
        if !ports.insert((alias.group, alias.byte)) || clips.contains_key(&alias.cue) {
            return Err(ImportError::Cue);
        }
        let library = library(alias.group);
        if area.overlaps(&library.range) {
            return Err(ImportError::Cue);
        }
        let region = common.slice(library.range)?;
        let clip = clip(
            region,
            library.directory_end,
            definition,
            resources,
            &mut used,
        )?;
        clips.insert(alias.cue, clip);
        aliases.push(alias);
    }
    Ok((clips, aliases))
}

fn clip(
    region: Region<'_>,
    directory_end: usize,
    definition: CueSource,
    resources: &mut Resources<'_, '_>,
    used: &mut Vec<Range<usize>>,
) -> Result<EffectClip, ImportError> {
    let index = usize::from(
        definition
            .alias
            .byte
            .checked_sub(1)
            .ok_or(ImportError::Cue)?,
    );
    let entry = region.start.saturating_add(index.saturating_mul(2));
    let directory = region.slice(region.start..directory_end)?;
    let list = directory.word(entry)?;
    let region = region.slice(directory_end..region.end())?;
    let head = list.saturating_add(definition.parts.saturating_mul(2));
    let roots = (0..definition.parts)
        .map(|part| region.word(list.saturating_add(part.saturating_mul(2))))
        .collect::<Result<Vec<_>, _>>()?;
    if roots.first() != Some(&head)
        || roots
            .windows(2)
            .any(|pair| matches!(pair, [first, second] if first >= second))
    {
        return Err(ImportError::Cue);
    }

    let mut end = head;
    let parts = roots
        .iter()
        .copied()
        .zip(roots.iter().copied().skip(1).chain([region.end()]))
        .map(|(root, limit)| {
            if root != end {
                return Err(ImportError::Cue);
            }
            let reader = Reader {
                region: region.slice(root..limit)?,
                at: root,
            };
            let (result, next) = part(reader, resources)?;
            end = next;
            Ok(result)
        })
        .collect::<Result<_, _>>()?;
    if used.iter().any(|old| list < old.end && old.start < end) {
        return Err(ImportError::Cue);
    }
    used.push(list..end);
    Ok(EffectClip { parts })
}

fn part(
    mut reader: Reader<'_>,
    resources: &mut Resources<'_, '_>,
) -> Result<(EffectPartContent, usize), ImportError> {
    let mut spans = Vec::new();
    let mut loop_point = None;
    let mut motion: Option<PitchMotion> = None;
    let loop_start = loop {
        let [opcode] = reader.take()?;
        match opcode {
            0xff if loop_point.is_none() => break None,
            0xfb => break Some(loop_point.unwrap_or(0)),
            0xfe if loop_point.is_none() && reader.take::<1>()? == [0] => {
                loop_point = Some(spans.len());
            }
            0xf5 if motion.is_none() && spans.is_empty() => {
                let [speed, target] = reader.take()?;
                motion = Some(PitchMotion {
                    speed: NonZeroU8::new(speed).ok_or(ImportError::Effect)?,
                    target: pitch(target)?,
                });
            }
            0..=0xf4 => {
                let [_, _, note, duration] = reader.take()?;
                let pitch = pitch(note)?;
                if motion
                    .as_ref()
                    .is_some_and(|slide| slide.target.get() <= pitch.get())
                {
                    return Err(ImportError::Effect);
                }
                spans.push(EffectSpan {
                    duration: EffectTurns(NonZeroU8::new(duration).ok_or(ImportError::Effect)?),
                    pitch,
                    timbre: resources.instrument(opcode)?,
                    motion: motion.clone(),
                });
            }
            _ => return Err(ImportError::Effect),
        }
    };
    if spans.is_empty() || loop_start.is_some_and(|index| index >= spans.len()) {
        return Err(ImportError::Effect);
    }
    Ok((EffectPartContent { spans, loop_start }, reader.at))
}

fn pitch(value: u8) -> Result<EffectPitch, ImportError> {
    EffectPitch::new(value & 0x7f).ok_or(ImportError::Effect)
}

#[cfg(test)]
mod tests;
