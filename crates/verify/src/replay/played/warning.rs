//! Compare an already admitted Ceres warning with the independent room records.

use super::{Frame, played};
use crate::replay::compare::Observation;
use crate::replay::ridley_warning_native as native;
use crate::replay::{data_failure, result::Outcome};
use data::assets::ridley_warning::{Art, Place};
use game::production::played::warning_drawing::TextColours;
use std::ops::Range;
use types::ceres_warning::{StrokeBeat, TypingWait, WarningPhase};

pub(super) fn selected(station: &played::station::State) -> bool {
    station.warning.is_some()
        || station.ridley.is_some_and(|ridley| {
            matches!(
                ridley.phase,
                boss_ceres_ridley::RidleyPhase::Getaway(
                    types::ceres_getaway::GetawayState::WarningReady
                )
            )
        })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Picture {
    colours: TextColours,
    footprint: Option<Footprint>,
    admitted_warning_plane: bool,
    timer: data::assets::countdown::Figures,
}

impl Picture {
    pub(super) fn new(
        timer: data::assets::countdown::Figures,
        art: Option<&Art>,
        admitted_warning_plane: bool,
    ) -> Self {
        let footprint = art.and_then(Footprint::new);
        Self {
            colours: TextColours::default(),
            footprint,
            admitted_warning_plane,
            timer,
        }
    }

    pub(super) fn checked(
        &mut self,
        frame: &Frame<'_>,
        room: &played::State,
        outputs: &played::Outputs,
    ) -> Result<(Vec<Observation>, Vec<Observation>), Outcome> {
        let Frame {
            previous,
            current,
            cut,
            ..
        } = *frame;
        let bad = || data_failure(Some(cut), "Ceres warning picture");
        let state = room.station.warning.ok_or_else(bad)?;
        let expected = native::state(current)?;
        let mut fields = vec![
            phase(expected.phase),
            named("glow", i64::from(expected.glow)),
            named("stage", i64::from(expected.station)),
            named("main-boss", i64::from(expected.main_boss)),
            named(
                "defeat-offer",
                i64::from(!native::state(previous)?.main_boss && expected.main_boss),
            ),
            named(
                "music",
                native::music(previous, current)?.map_or(-1, i64::from),
            ),
            named("stroke", i64::from(native::strokes(previous, current)?)),
        ];
        let warning = outputs.warning.as_ref();
        let mut actual = vec![
            phase(model_phase(state.phase)?),
            named("glow", i64::from(state.glow.get())),
            named(
                "stage",
                match room.station.stage {
                    boss_ceres_ridley::Stage::Calm => 0,
                    boss_ceres_ridley::Stage::Getaway => 1,
                    boss_ceres_ridley::Stage::Escape => 2,
                },
            ),
            named(
                "main-boss",
                i64::from(room.progress.defeated[types::Area::Ceres][types::BossKind::MainBoss]),
            ),
            named(
                "defeat-offer",
                i64::from(
                    warning
                        .and_then(played::warning::Output::progress_report)
                        .is_some(),
                ),
            ),
            named(
                "music",
                if warning.is_some_and(|warning| warning.music.is_some()) {
                    i64::from(boss_ceres_ridley::EscapeTrack::DELAY)
                } else {
                    -1
                },
            ),
            named(
                "stroke",
                i64::from(warning.is_some_and(|warning| warning.stroke.is_some())),
            ),
        ];
        self.colours
            .advance(&room.palette, frame.publishes)
            .map_err(|_| bad())?;
        let (live, published) = self.colours.values();
        for ((field, slots), (live, published)) in [("english", 81..84), ("heading", 97..100)]
            .into_iter()
            .zip(live.into_iter().zip(published))
        {
            let native_live = read_live_colours(current, slots.clone()).ok_or_else(bad)?;
            fields.push(words(format!("ridley.warning.{field}.live"), native_live));
            actual.push(words(
                format!("ridley.warning.{field}.live"),
                live.map(types::Colour::bits),
            ));
            let native_published = current.cgram.get(slots).ok_or_else(bad)?;
            fields.push(words(
                format!("ridley.warning.{field}.published"),
                native_published.iter().copied(),
            ));
            actual.push(words(
                format!("ridley.warning.{field}.published"),
                published.map(types::Colour::bits),
            ));
        }
        let footprint = self.footprint.as_ref().ok_or_else(bad)?;
        let (sheets, text) = footprint.ranges(expected.phase, self.admitted_warning_plane);
        for (name, spans) in [
            ("ridley.warning.art", sheets),
            ("ridley.warning.text", text),
        ] {
            fields.push(art_words(name, &current.vram, &spans).ok_or_else(bad)?);
            actual
                .push(art_words(name, &room.room_video.video.memory.vram, &spans).ok_or_else(bad)?);
        }
        Ok((fields, actual))
    }
    pub(super) fn clock(
        &self,
        frame: &Frame<'_>,
        room: &played::State,
    ) -> Result<(Vec<Observation>, Vec<Observation>), Outcome> {
        let Frame { current, cut, .. } = *frame;
        let bad = || data_failure(Some(cut), "Ceres countdown drawing");
        let (mut fields, mut actual) = (Vec::new(), Vec::new());
        if let interface_hud::Countdown::Begun(clock) =
            crate::replay::hud::countdown_origin(&current.wram)?
        {
            let sprites = game::exact::countdown::sprites(
                &self.timer,
                clock.place.whole_pixels(),
                clock.time_left,
            )
            .ok_or_else(bad)?;
            let mut batch = game::state::video::NmiBatch::default();
            batch.draw_sprites(&sprites);
            let shadow = native::contains_sprite_draw(
                current.wram.get(native::SHADOW_OAM).ok_or_else(bad)?,
                &batch,
                sprites.pieces.len(),
            );
            let published =
                native::contains_sprite_draw(&current.oam, &batch, sprites.pieces.len());
            for (field, present, image, actual_pieces) in [
                (
                    "clock.shadow",
                    shadow,
                    current.wram.get(native::SHADOW_OAM).ok_or_else(bad)?,
                    &room.clock_picture.prepared,
                ),
                (
                    "clock.published",
                    published,
                    &current.oam,
                    &room.clock_picture.published,
                ),
            ] {
                fields.push(named(field, i64::from(present)));
                let value = if actual_pieces.is_empty() {
                    0
                } else if actual_pieces.len() == sprites.pieces.len()
                    && contains_clock(image, actual_pieces)
                {
                    1
                } else {
                    -1
                };
                actual.push(named(field, value));
            }
        }
        Ok((fields, actual))
    }
}

fn contains_clock(image: &[u8], pieces: &[[u8; 5]]) -> bool {
    let Some((low, high)) = image.split_at_checked(game::state::video::OAM_SPRITES * 4) else {
        return false;
    };
    low.as_chunks::<4>()
        .0
        .windows(pieces.len())
        .enumerate()
        .any(|(start, window)| {
            window
                .iter()
                .zip(pieces)
                .enumerate()
                .all(|(offset, (raw, piece))| {
                    let at = start.saturating_add(offset);
                    raw == &piece[..4]
                        && high.get(at / 4).is_some_and(|byte| {
                            (byte >> ((at % 4).wrapping_mul(2))) & 3 == piece[4]
                        })
                })
        })
}

fn phase(phase: native::Phase) -> Observation {
    let parts = match phase {
        native::Phase::Ready => [0_u16, 0, 0, 0],
        native::Phase::Art(progress) => [1, progress, 0, 0],
        native::Phase::Heading(left) => [2, left, 0, 0],
        native::Phase::Typing {
            consumed,
            wait,
            audible,
        } => [3, consumed, wait, u16::from(audible)],
        native::Phase::EscapeReady => [4, 0, 0, 0],
        native::Phase::Escape => [5, 0, 0, 0],
    };
    words("ridley.warning.phase", parts)
}

fn model_phase(phase: WarningPhase) -> Result<native::Phase, Outcome> {
    let bad = || data_failure(None, "Ceres warning progress");
    Ok(match phase {
        WarningPhase::WarningReady => native::Phase::Ready,
        WarningPhase::SettingArt(progress) => {
            native::Phase::Art(u16::try_from(progress.get()).map_err(|_| bad())?)
        }
        WarningPhase::Heading(left) => native::Phase::Heading(u16::from(left.get())),
        WarningPhase::Typing(typing) => native::Phase::Typing {
            consumed: u16::try_from(typing.consumed).map_err(|_| bad())?,
            wait: match typing.wait {
                TypingWait::Ready => 0,
                TypingWait::OneRun => 1,
                TypingWait::TwoRuns => 2,
            },
            audible: typing.beat == StrokeBeat::Audible,
        },
        WarningPhase::EscapeReady => native::Phase::EscapeReady,
        WarningPhase::Escape => native::Phase::Escape,
        WarningPhase::SubtitleWait { .. } => return Err(bad()),
    })
}

fn named(field: &str, value: i64) -> Observation {
    Observation::quantity(format!("ridley.warning.{field}"), value)
}

fn words(field: impl Into<String>, values: impl IntoIterator<Item = u16>) -> Observation {
    Observation {
        field: field.into(),
        bytes: values.into_iter().flat_map(u16::to_le_bytes).collect(),
    }
}

fn read_live_colours(
    record: &oracle_record::FrameRecord,
    slots: std::ops::Range<usize>,
) -> Option<impl Iterator<Item = u16>> {
    let start = 0xc000_usize.checked_add(slots.start.checked_mul(2)?)?;
    let end = 0xc000_usize.checked_add(slots.end.checked_mul(2)?)?;
    Some(
        record
            .wram
            .get(start..end)?
            .as_chunks::<2>()
            .0
            .iter()
            .copied()
            .map(u16::from_le_bytes),
    )
}

/// Expected physical addresses are decoded independently of produced writes.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Footprint {
    sprites: Vec<Range<usize>>,
    foreground: Vec<Range<usize>>,
    heading: Range<usize>,
    english: Vec<(Range<usize>, bool)>,
}

impl Footprint {
    fn new(art: &Art) -> Option<Self> {
        let patch = |patch: &data::assets::ridley_warning::SheetPatch| {
            let start =
                0x6000_usize.checked_add(usize::from(patch.first.0).checked_mul(16)?)? % 32768;
            Some(start..start.checked_add(patch.tiles.0.len().checked_mul(16)?)?)
        };
        let english = art
            .english
            .iter()
            .map(|letter| Some((map(letter.place, 1)?, letter.glyph.is_some())))
            .collect::<Option<_>>()?;
        Some(Self {
            sprites: art.sprites.iter().map(patch).collect::<Option<_>>()?,
            foreground: art
                .foreground
                .iter()
                .map(|value| patch(&value.patch))
                .collect::<Option<_>>()?,
            heading: map(art.heading.first, art.heading.cells.len())?,
            english,
        })
    }

    fn ranges(
        &self,
        phase: native::Phase,
        admitted: bool,
    ) -> (Vec<Range<usize>>, Vec<Range<usize>>) {
        let (loaded, heading, consumed) = match phase {
            native::Phase::Ready => (0, false, 0),
            native::Phase::Art(loaded) => (usize::from(loaded), false, 0),
            native::Phase::Heading(_) => (14, true, 0),
            native::Phase::Typing { consumed, .. } => (14, true, usize::from(consumed)),
            native::Phase::EscapeReady | native::Phase::Escape => (14, true, self.english.len()),
        };
        let sheets = self
            .sprites
            .iter()
            .take(loaded)
            .chain(self.foreground.iter().take(loaded.saturating_sub(6)))
            .cloned()
            .collect();
        let text = heading
            .then_some(self.heading.clone())
            .into_iter()
            .chain(
                self.english
                    .iter()
                    .take(consumed)
                    .filter(|(_, glyph)| admitted || *glyph)
                    .map(|(range, _)| range.clone()),
            )
            .collect();
        (sheets, text)
    }
}

fn map(place: Place, columns: usize) -> Option<Range<usize>> {
    let start = 0x5000_usize
        .checked_add(usize::from(place.row).checked_mul(32)?)?
        .checked_add(usize::from(place.column))?;
    Some(start..start.checked_add(columns)?)
}

fn art_words(field: &str, vram: &[u16], spans: &[Range<usize>]) -> Option<Observation> {
    let mut bytes = Vec::new();
    for span in spans {
        bytes.extend(
            vram.get(span.clone())?
                .iter()
                .flat_map(|word| word.to_le_bytes()),
        );
    }
    Some(Observation {
        field: field.into(),
        bytes,
    })
}
