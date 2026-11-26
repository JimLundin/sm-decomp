//! Adapter for the WX rows of the shared vector file.

use boss_ceres_ridley::{
    CountdownStart, EscapeTrack, HeadingHold, PositionedGlyph, Stage, StrokeBeat, SubtitleHold,
    TypingWait, WARNING_GLOW_LOOKS, WarningArtProgress, WarningArtUpdate, WarningColourPair,
    WarningContent, WarningEffects, WarningEnglishCell, WarningForeground, WarningGlow,
    WarningInputs, WarningPalette, WarningPhase, WarningSheet, WarningState, WarningStroke,
    WarningTypingProgress,
};
use serde::Deserialize;
use types::ceres_progress::CeresDefeat;
use types::options::Language;
use types::{Colour, Look, Pair, Picture};

#[derive(Clone, Copy, Deserialize)]
pub enum WarningLanguage {
    English,
    JapaneseSubtitle,
}

impl From<WarningLanguage> for Language {
    fn from(language: WarningLanguage) -> Self {
        match language {
            WarningLanguage::English => Self::English,
            WarningLanguage::JapaneseSubtitle => Self::Japanese,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
enum PhaseName {
    WarningReady,
    SettingArt,
    Heading,
    SubtitleWait,
    Typing,
    EscapeReady,
    Escape,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WarningRow {
    phase: PhaseName,
    glow: u8,
    loaded: Option<usize>,
    left: Option<u8>,
    consumed: Option<usize>,
    wait: Option<u8>,
    silent: Option<bool>,
}

impl WarningRow {
    fn build(&self, content: &WarningContent<'_, &str>) -> WarningState {
        let phase = match self.phase {
            PhaseName::WarningReady => WarningPhase::WarningReady,
            PhaseName::SettingArt => WarningPhase::SettingArt(
                WarningArtProgress::new(self.loaded.unwrap(), content.schedule.len()).unwrap(),
            ),
            PhaseName::Heading => {
                WarningPhase::Heading(HeadingHold::new(self.left.unwrap()).unwrap())
            }
            PhaseName::SubtitleWait => WarningPhase::SubtitleWait {
                left: SubtitleHold::new(self.left.unwrap()).unwrap(),
                typing: self.typing(content.english.len()),
            },
            PhaseName::Typing => WarningPhase::Typing(self.typing(content.english.len())),
            PhaseName::EscapeReady => WarningPhase::EscapeReady,
            PhaseName::Escape => WarningPhase::Escape,
        };
        let state = WarningState {
            phase,
            glow: WarningGlow::new(self.glow).unwrap(),
        };
        assert_eq!(
            *self,
            Self::from(state),
            "only fields of the restored phase belong in the row"
        );
        state
    }

    fn typing(&self, text_len: usize) -> WarningTypingProgress {
        let consumed = self.consumed.unwrap();
        assert!(consumed <= text_len);
        WarningTypingProgress {
            consumed,
            wait: match self.wait.unwrap() {
                0 => TypingWait::Ready,
                1 => TypingWait::OneRun,
                2 => TypingWait::TwoRuns,
                value => panic!("invalid typing wait {value}"),
            },
            beat: if self.silent.unwrap() {
                StrokeBeat::Silent
            } else {
                StrokeBeat::Audible
            },
        }
    }

    fn record_typing(&mut self, typing: WarningTypingProgress) {
        self.consumed = Some(typing.consumed);
        self.wait = Some(match typing.wait {
            TypingWait::Ready => 0,
            TypingWait::OneRun => 1,
            TypingWait::TwoRuns => 2,
        });
        self.silent = Some(typing.beat == StrokeBeat::Silent);
    }
}

impl From<WarningState> for WarningRow {
    fn from(state: WarningState) -> Self {
        let mut row = Self {
            phase: PhaseName::WarningReady,
            glow: state.glow.get(),
            loaded: None,
            left: None,
            consumed: None,
            wait: None,
            silent: None,
        };
        row.phase = match state.phase {
            WarningPhase::WarningReady => PhaseName::WarningReady,
            WarningPhase::SettingArt(progress) => {
                row.loaded = Some(progress.get());
                PhaseName::SettingArt
            }
            WarningPhase::Heading(left) => {
                row.left = Some(left.get());
                PhaseName::Heading
            }
            WarningPhase::SubtitleWait { left, typing } => {
                row.left = Some(left.get());
                row.record_typing(typing);
                PhaseName::SubtitleWait
            }
            WarningPhase::Typing(typing) => {
                row.record_typing(typing);
                PhaseName::Typing
            }
            WarningPhase::EscapeReady => PhaseName::EscapeReady,
            WarningPhase::Escape => PhaseName::Escape,
        };
        row
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct GlyphRow(u16, u16, String);

#[derive(Deserialize)]
#[serde(untagged)]
pub enum EnglishCellRow {
    Letter(u16, u16, String),
    Blank(u16, u16),
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
enum EscapeStageRow {
    Escape,
}

#[derive(Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WarningEffectsRow {
    colour_copy: Option<Vec<[u8; 4]>>,
    art: Option<Vec<String>>,
    heading: Option<bool>,
    subtitle: Option<bool>,
    glyph: Option<GlyphRow>,
    glow_palette: Option<u8>,
    music_delay: Option<u8>,
    stroke: Option<bool>,
    stage: Option<EscapeStageRow>,
    defeat: Option<CeresDefeat>,
    countdown: Option<[u8; 2]>,
}

/// Each slot has a different colour, so the comparison can tell which slot was copied.
pub const fn foreground() -> WarningForeground {
    WarningForeground {
        palette_zero: WarningColourPair {
            slot_one: Colour::new(1).unwrap(),
            slot_three: Colour::new(3).unwrap(),
        },
        palette_one: WarningColourPair {
            slot_one: Colour::new(11).unwrap(),
            slot_three: Colour::new(13).unwrap(),
        },
    }
}

fn look(index: usize) -> Look {
    Look::unflipped(Picture::new(u16::try_from(index).unwrap()).unwrap())
}

fn english_cells(rows: &[EnglishCellRow]) -> (Vec<WarningEnglishCell>, Vec<&str>) {
    // When the vector has no text fixture, it names none of the glyphs. Use 58 blank cells
    // so the recorded end-of-text progress remains valid.
    if rows.is_empty() {
        return (
            (0..58)
                .map(|x| WarningEnglishCell {
                    cell: Pair { x, y: 0 },
                    glyph: None,
                })
                .collect(),
            Vec::new(),
        );
    }
    let mut names = Vec::new();
    let cells = rows
        .iter()
        .map(|row| match row {
            EnglishCellRow::Blank(x, y) => WarningEnglishCell {
                cell: Pair { x: *x, y: *y },
                glyph: None,
            },
            EnglishCellRow::Letter(x, y, name) => {
                let glyph = look(names.len());
                names.push(name.as_str());
                WarningEnglishCell {
                    cell: Pair { x: *x, y: *y },
                    glyph: Some(glyph),
                }
            }
        })
        .collect();
    (cells, names)
}

fn art_run(names: &[String]) -> Vec<WarningArtUpdate<'_, &str>> {
    // Each marker stands for one whole test tile. The schedule can mix sheets in a run.
    names
        .iter()
        .enumerate()
        .map(|(first_tile, name)| WarningArtUpdate {
            sheet: if name.starts_with("Background") {
                WarningSheet::Foreground
            } else {
                WarningSheet::Sprites
            },
            first_tile,
            tiles: match name.as_str() {
                "SpriteA" => &["SpriteA"],
                "SpriteB" => &["SpriteB"],
                "BackgroundA" => &["BackgroundA"],
                "BackgroundB" => &["BackgroundB"],
                "DoorA" => &["DoorA"],
                name => panic!("unknown fixture art {name}"),
            },
        })
        .collect()
}

fn glow_palettes() -> [[Colour; 3]; WARNING_GLOW_LOOKS] {
    core::array::from_fn(|look| {
        core::array::from_fn(|slot| {
            let index = look.saturating_mul(10).saturating_add(slot);
            Colour::new(100u16.saturating_add(u16::try_from(index).unwrap())).unwrap()
        })
    })
}

fn record_colours(
    effects: &WarningEffects<'_, &str>,
    content: &WarningContent<'_, &str>,
    row: &mut WarningEffectsRow,
) {
    let colours = &effects.picture.colours;
    match colours.len() {
        0 => {}
        3 => {
            let slots: Vec<_> = colours
                .iter()
                .map(|write| {
                    assert_eq!(write.palette, WarningPalette::HeadingText);
                    (write.slot, write.colour)
                })
                .collect();
            let look = content
                .glow_palettes
                .iter()
                .position(|colours| slots == [(1, colours[0]), (2, colours[1]), (3, colours[2])])
                .unwrap();
            row.glow_palette = Some(u8::try_from(look).unwrap());
        }
        4 => {
            row.colour_copy = Some(
                colours
                    .iter()
                    .map(|write| {
                        let colour = u8::try_from(write.colour.bits()).unwrap();
                        [colour / 10, colour % 10, write.palette.number(), write.slot]
                    })
                    .collect(),
            );
        }
        count => panic!("unexpected {count} colour writes"),
    }
}

fn record_words(
    effects: &WarningEffects<'_, &str>,
    content: &WarningContent<'_, &str>,
    glyph_names: &[&str],
    row: &mut WarningEffectsRow,
) {
    let mut letter_written = false;
    for write in effects.picture.text() {
        assert!(!letter_written, "nothing follows the letter write");
        match write.palette {
            WarningPalette::HeadingText => {
                assert_eq!(write.palette.number(), 6);
                assert_eq!(write.cells, content.heading);
                assert!(row.heading.replace(true).is_none());
            }
            WarningPalette::Japanese => {
                assert_eq!(write.palette.number(), 7);
                assert_eq!(write.cells, content.subtitle);
                assert!(row.subtitle.replace(true).is_none());
            }
            WarningPalette::English => {
                assert_eq!(write.palette.number(), 5);
                let [letter] = write.cells else {
                    panic!("expected one letter")
                };
                let name = glyph_names[usize::from(letter.glyph.picture.index())];
                assert_eq!(letter.glyph, Look::unflipped(letter.glyph.picture));
                row.glyph = Some(GlyphRow(letter.cell.x, letter.cell.y, name.to_owned()));
                letter_written = true;
            }
        }
    }
}

fn effects_row(
    effects: &WarningEffects<'_, &str>,
    content: &WarningContent<'_, &str>,
    glyph_names: &[&str],
) -> WarningEffectsRow {
    let mut row = WarningEffectsRow::default();
    record_colours(effects, content, &mut row);
    record_words(effects, content, glyph_names, &mut row);
    if !effects.picture.art.is_empty() {
        row.art = Some(
            effects
                .picture
                .art
                .iter()
                .flat_map(|update| update.tiles.iter().map(|name| (*name).to_owned()))
                .collect(),
        );
    }
    row.music_delay = effects.music.map(|EscapeTrack| EscapeTrack::DELAY);
    row.stroke = effects.stroke.map(|WarningStroke| {
        assert_eq!(WarningStroke::EFFECTS_CHANNEL, 2);
        assert_eq!(WarningStroke::QUEUE_LIMIT, 3);
        true
    });
    if let Some(activation) = effects.activation {
        assert_eq!(activation.stage, Stage::Escape);
        row.stage = Some(EscapeStageRow::Escape);
        row.defeat = Some(activation.defeat);
        let CountdownStart {
            minutes,
            seconds,
            hundredths,
        } = activation.countdown;
        assert_eq!(hundredths, 0);
        row.countdown = Some([minutes, seconds]);
    }
    row
}

pub fn run(
    before: &WarningRow,
    inputs: WarningInputs,
    art: Option<&[Vec<String>]>,
    text: Option<&[EnglishCellRow]>,
) -> (WarningRow, WarningEffectsRow) {
    let fallback = [vec!["SpriteA".to_owned()]];
    let art: Vec<_> = art
        .unwrap_or(&fallback)
        .iter()
        .map(|batch| art_run(batch))
        .collect();
    let schedule: Vec<_> = art.iter().map(Vec::as_slice).collect();
    let (english, names) = english_cells(text.unwrap_or(&[]));
    let heading = [PositionedGlyph {
        cell: Pair { x: 11, y: 6 },
        glyph: look(0),
    }];
    let subtitle: Vec<_> = [12, 12, 11, 11]
        .into_iter()
        .zip(20..24)
        .flat_map(|(width, y)| {
            (10..10u16.saturating_add(width)).map(move |x| PositionedGlyph {
                cell: Pair { x, y },
                glyph: look(1),
            })
        })
        .collect();
    let content = WarningContent {
        schedule: &schedule,
        heading: &heading,
        subtitle: &subtitle,
        english: &english,
        glow_palettes: &glow_palettes(),
    };
    let state = before.build(&content);
    let saved = toml::to_string(&state).unwrap();
    let mut restored = toml::from_str::<WarningState>(&saved).unwrap();
    assert_eq!(restored, state);
    let effects = boss_ceres_ridley::step_warning(&mut restored, inputs, &content);
    (restored.into(), effects_row(&effects, &content, &names))
}

pub fn escape_activation_survives_restore() {
    let art = [WarningArtUpdate {
        sheet: WarningSheet::Sprites,
        first_tile: 0,
        tiles: &["SpriteA"],
    }];
    let heading = [PositionedGlyph {
        cell: Pair { x: 11, y: 6 },
        glyph: look(0),
    }];
    let english = [WarningEnglishCell {
        cell: Pair { x: 5, y: 8 },
        glyph: Some(look(1)),
    }];
    let content = WarningContent {
        schedule: &[&art],
        heading: &heading,
        subtitle: &[],
        english: &english,
        glow_palettes: &glow_palettes(),
    };
    let mut inputs = WarningInputs {
        ordinary: true,
        finished: types::GameFrames(0),
        fade: false,
        passage: false,
        language: Language::English,
        foreground: foreground(),
    };
    let mut warning = WarningState::ready();
    let mut activations = Vec::new();
    // A one-batch, one-letter warning finishes well within 150 ordinary runs.
    for frame in 0..150 {
        inputs.finished = types::GameFrames(frame);
        let effects = boss_ceres_ridley::step_warning(&mut warning, inputs, &content);
        if let Some(activation) = effects.activation {
            activations.push(activation);
            let saved = toml::to_string(&warning).unwrap();
            warning = toml::from_str(&saved).unwrap();
        }
    }
    assert_eq!(warning.phase, WarningPhase::Escape);
    assert_eq!(activations.len(), 1);
    assert_eq!(activations.first().unwrap().defeat, CeresDefeat::MainBoss);
}
