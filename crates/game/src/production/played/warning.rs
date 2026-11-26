//! Connect the warning model to the room's art, actors and downstream owners.

use super::{Fault, HudFrame, frame};
use boss_ceres_ridley::{
    EscapeActivation, EscapeTrack, PositionedGlyph, WarningArtUpdate, WarningColourPair,
    WarningContent, WarningEnglishCell, WarningForeground, WarningInputs, WarningSheet,
    WarningStroke, step_warning,
};
use data::assets::art::{Cell, PaletteRow, Tile, TileRef, TileSheet};
use data::assets::ridley_warning::{Art, Letter, Line, Place, SheetPatch};
use types::ceres_progress::CeresDefeat;
use types::ceres_warning::WarningState;
use types::options::Language;
use types::{Area, BossKind, Colour, Flip, GameFrames, Look, Pair, Picture};
use world_doors::Palette;

/// Checked warning art and text for the admitted file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    art: Art,
    heading: Vec<PositionedGlyph>,
    english: Vec<WarningEnglishCell>,
    language: Language,
    glow: [[Colour; 3]; 16],
}

impl Content {
    /// Prepare the imported art for the language chosen by the file.
    ///
    /// Returns `None` for unsupported language or incompatible text art.
    #[must_use]
    pub fn new(art: Art, language: Language) -> Option<Self> {
        if language != Language::English || art.english.is_empty() {
            return None;
        }
        let heading = positioned_line(&art.heading)?;
        let english = art
            .english
            .iter()
            .map(|letter| {
                Some(WarningEnglishCell {
                    cell: position(letter.place),
                    glyph: match letter.glyph {
                        Some(cell) => Some(look(cell)?),
                        None => None,
                    },
                })
            })
            .collect::<Option<_>>()?;
        let mut glow = [[Colour::BLACK; 3]; 16];
        for (target, source) in glow.iter_mut().zip(art.glow) {
            for (slot, colour) in target.iter_mut().zip(source) {
                *slot = Colour::new(colour.0)?;
            }
        }
        Some(Self {
            art,
            heading,
            english,
            language,
            glow,
        })
    }
}

/// Warning effects ready for the room's distinct consumers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    /// The warning phase and its progress after this turn.
    pub state: WarningState,
    /// Tile replacements for the exact drawing consumer.
    pub art: Vec<SheetPatch>,
    /// Positioned high-priority cell replacements.
    pub text: Vec<Letter>,
    /// The warning's delayed track offer.
    pub music: Option<EscapeTrack>,
    /// The warning's stroke offer for the second effects channel.
    pub stroke: Option<WarningStroke>,
    /// The one-shot request to the station, progress and HUD.
    pub activation: Option<EscapeActivation>,
}

impl Output {
    /// The warning's request to run-progress.
    #[must_use]
    pub const fn progress_report(&self) -> Option<run_progress::Report> {
        match self.activation {
            Some(EscapeActivation {
                defeat: CeresDefeat::MainBoss,
                ..
            }) => Some(run_progress::Report::Defeated {
                area: Area::Ceres,
                boss: BossKind::MainBoss,
            }),
            None => None,
        }
    }

    pub(super) fn countdown_start(&self) -> Result<Option<interface_hud::StartTime>, Fault> {
        self.activation
            .map(|activation| {
                let time = activation.countdown;
                if time.hundredths != 0 {
                    return Err(Fault::RidleyWarning);
                }
                interface_hud::StartTime::new(time.minutes, time.seconds)
                    .ok_or(Fault::RidleyWarning)
            })
            .transpose()
    }
}

pub(super) fn record_progress(progress: &mut run_progress::Progress, output: Option<&Output>) {
    if let Some(report) = output.and_then(Output::progress_report) {
        progress.apply(&run_progress::Rules::default(), &[report]);
    }
}

pub(super) fn countdown_input(hud: &HudFrame, output: Option<&Output>) -> Result<HudFrame, Fault> {
    let Some(output) = output else {
        return Ok(*hud);
    };
    Ok(HudFrame {
        countdown_starts: output.countdown_start()?,
        ..*hud
    })
}

pub(super) fn run(
    mut state: WarningState,
    content: &Content,
    frame: frame::State,
    ordinary: bool,
    palette: &mut Palette,
) -> Result<Output, Fault> {
    let bad = || Fault::RidleyWarning;
    let colours = |origin: usize| {
        let [_, slot_one, _, slot_three] = *palette.screen.get(origin..)?.first_chunk::<4>()?;
        Some(WarningColourPair {
            slot_one,
            slot_three,
        })
    };
    let foreground = WarningForeground {
        palette_zero: colours(0).ok_or_else(bad)?,
        palette_one: colours(16).ok_or_else(bad)?,
    };
    let [first, second, third, fourth, fifth, sixth, last] = &content.art.sprites;
    let [overlap, rest @ ..] = &content.art.foreground;
    let first = [first, second, third, fourth, fifth, sixth]
        .map(|patch| [update(WarningSheet::Sprites, patch)]);
    let both = [
        update(WarningSheet::Sprites, last),
        update(WarningSheet::Foreground, &overlap.patch),
    ];
    let rest = rest
        .each_ref()
        .map(|patch| [update(WarningSheet::Foreground, &patch.patch)]);
    let schedule: Vec<&[_]> = first
        .iter()
        .map(<[_; 1]>::as_slice)
        .chain(std::iter::once(both.as_slice()))
        .chain(rest.iter().map(<[_; 1]>::as_slice))
        .collect();
    let model = WarningContent {
        schedule: &schedule,
        heading: &content.heading,
        subtitle: &[],
        english: &content.english,
        glow_palettes: &content.glow,
    };
    let effects = step_warning(
        &mut state,
        WarningInputs {
            ordinary,
            finished: GameFrames(frame.finished_frames),
            fade: false,
            passage: !ordinary,
            language: content.language,
            foreground,
        },
        &model,
    );
    for write in &effects.picture.colours {
        let position = usize::from(write.palette.number())
            .saturating_mul(16)
            .saturating_add(usize::from(write.slot));
        *palette.screen.get_mut(position).ok_or_else(bad)? = write.colour;
    }
    let art = effects
        .picture
        .art
        .iter()
        .map(|patch| {
            Some(SheetPatch {
                first: TileRef(u16::try_from(patch.first_tile).ok()?),
                tiles: TileSheet(patch.tiles.to_vec()),
            })
        })
        .collect::<Option<_>>()
        .ok_or_else(bad)?;
    let text = effects
        .picture
        .text()
        .flat_map(|write| {
            write.cells.iter().map(move |glyph| {
                Some(Letter {
                    place: Place {
                        column: u8::try_from(glyph.cell.x).ok()?,
                        row: u8::try_from(glyph.cell.y).ok()?,
                    },
                    glyph: Some(cell(glyph.glyph, write.palette.number())),
                    rest: 0,
                })
            })
        })
        .collect::<Option<_>>()
        .ok_or_else(bad)?;
    Ok(Output {
        state,
        art,
        text,
        music: effects.music,
        stroke: effects.stroke,
        activation: effects.activation,
    })
}

fn update(sheet: WarningSheet, patch: &SheetPatch) -> WarningArtUpdate<'_, Tile> {
    WarningArtUpdate {
        sheet,
        first_tile: usize::from(patch.first.0),
        tiles: &patch.tiles.0,
    }
}

fn positioned_line(line: &Line) -> Option<Vec<PositionedGlyph>> {
    line.cells
        .iter()
        .enumerate()
        .map(|(offset, &glyph)| {
            let mut cell = position(line.first);
            cell.x = cell.x.checked_add(u16::try_from(offset).ok()?)?;
            Some(PositionedGlyph {
                cell,
                glyph: look(glyph)?,
            })
        })
        .collect()
}

fn look(cell: Cell) -> Option<Look> {
    Some(Look {
        picture: Picture::new(cell.tile.0)?,
        flip: match (cell.flip_x, cell.flip_y) {
            (false, false) => Flip::None,
            (true, false) => Flip::Across,
            (false, true) => Flip::Down,
            (true, true) => Flip::Both,
        },
    })
}

const fn cell(look: Look, palette: u8) -> Cell {
    Cell {
        tile: TileRef(look.picture.index()),
        palette: PaletteRow(palette),
        priority: true,
        flip_x: matches!(look.flip, Flip::Across | Flip::Both),
        flip_y: matches!(look.flip, Flip::Down | Flip::Both),
    }
}

fn position(place: Place) -> Pair<u16> {
    Pair {
        x: u16::from(place.column),
        y: u16::from(place.row),
    }
}
