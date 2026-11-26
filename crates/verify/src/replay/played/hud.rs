//! The HUD's inputs held from the seed, the frame it is brought, and its compared fields.

use super::platform::Controller;
use crate::replay::compare::Observation;
use crate::replay::hud;
use crate::replay::result::Outcome;
use crate::source;
use game::production::played;
use interface_hud::{Countdown, CountdownExpiry, StartTime};

/// The HUD's inputs that no owner in the loop produces.
#[derive(Clone)]
pub(super) struct Held {
    bindings: hud::Bindings,
    stock_at_start: interface_hud::StockAtStart,
    status: interface_hud::Status,
    map_collected: bool,
}

/// The HUD's inputs as the seed holds them.
pub(super) fn held(seed: &[u8]) -> Result<Held, Outcome> {
    Ok(Held {
        bindings: hud::bindings(seed)?,
        stock_at_start: hud::stock_at_start(seed)?,
        status: hud::status(seed)?,
        map_collected: hud::map_collected(seed)?,
    })
}

/// A new file inherits its bindings and starts with no reserve, ammunition or map.
pub(super) fn fresh(
    bindings: hud::Bindings,
    progress: &run_progress::Progress,
) -> Result<Held, Outcome> {
    let inventory = &progress.inventory;
    let energy = run_progress::Supply::Energy;
    Ok(Held {
        bindings,
        stock_at_start: interface_hud::StockAtStart {
            counts: enum_map::EnumMap::default(),
            equipped: interface_hud::Equipped::default(),
        },
        status: interface_hud::Status {
            energy: interface_hud::Energy {
                now: inventory.held.0[energy].0,
                capacity: inventory.capacity.0[energy].0,
            },
            reserve: interface_hud::Reserve {
                energy: 0,
                mode: interface_hud::ReserveMode::Manual,
            },
            ammunition: interface_hud::Ammunition::new(enum_map::EnumMap::default())
                .ok_or_else(|| crate::replay::data_failure(None, "fresh ammunition"))?,
        },
        map_collected: false,
    })
}

/// Checked adaptation of imported cell art into the opening HUD.
pub(super) fn opening(
    asset: &data::assets::start::Hud,
) -> Result<(interface_hud::OpeningFrame, interface_hud::OpeningPreview), Outcome> {
    use interface_hud::{ColourSet, HudGrid, OpeningFrame, Place, PreviewCell, Tile};
    let bad = || crate::replay::data_failure(None, "played opening HUD art");
    let tile = |cell: data::assets::start::HudCell| -> Result<Tile, Outcome> {
        Ok(Tile::new(
            cell.look,
            ColourSet::new(cell.colours).ok_or_else(bad)?,
        ))
    };
    let [upper, lower @ ..] = &asset.frame;
    let upper_row = upper
        .iter()
        .copied()
        .map(tile)
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| bad())?;
    let [[first, ..], ..] = lower;
    let mut lower_rows = HudGrid::filled(tile(*first)?);
    for (y, row) in (0..).zip(lower) {
        for (x, &cell) in (0..).zip(row) {
            lower_rows.set(Place { x, y }, tile(cell)?);
        }
    }
    Ok((
        OpeningFrame {
            upper_row,
            lower_rows,
        },
        asset
            .preview
            .map(|row| row.map(|look| look.map_or(PreviewCell::Empty, PreviewCell::Shows))),
    ))
}

/// The full video publication in hardware order, one observation per owner.
pub(super) fn video_expected(
    record: &oracle_record::FrameRecord,
) -> Result<[Observation; 2], Outcome> {
    const TOP: usize = 0x5800;
    const LOWER: usize = TOP + 32;
    const END: usize = LOWER + 96;
    let words = |range: core::ops::Range<usize>| {
        record
            .vram
            .get(range)
            .map(|words| words.iter().flat_map(|word| word.to_le_bytes()).collect())
            .ok_or_else(|| crate::replay::data_failure(None, "played HUD VRAM"))
    };
    Ok([
        Observation {
            field: "hud.upper-row".into(),
            bytes: words(TOP..LOWER)?,
        },
        Observation {
            field: "hud.video-grid".into(),
            bytes: words(LOWER..END)?,
        },
    ])
}

pub(super) fn video_actual(
    upper_row: Option<&interface_hud::UpperRow>,
    state: &interface_hud::HudState,
) -> Result<[Observation; 2], Outcome> {
    use game::production::hud::{cell, tilemap};
    Ok([
        Observation {
            field: "hud.upper-row".into(),
            bytes: upper_row
                .into_iter()
                .flatten()
                .flat_map(|tile| cell(*tile))
                .collect(),
        },
        Observation {
            field: "hud.video-grid".into(),
            bytes: tilemap(state)
                .map_err(|_| crate::replay::data_failure(None, "played HUD grid"))?,
        },
    ])
}

/// What a frame of play brings the HUD: the held facts, with the item buttons `controller`
/// reads and the countdown's start, borrowed from the records.
pub(super) const fn frame(
    held: &Held,
    controller: Controller,
    countdown_starts: Option<StartTime>,
) -> played::HudFrame {
    let status = held.status;
    played::HudFrame {
        buttons: held.bindings.buttons(controller.pressed, controller.held),
        stock_at_start: held.stock_at_start,
        capacity: status.energy.capacity,
        reserve: status.reserve,
        ammunition: status.ammunition,
        map_collected: held.map_collected,
        // The HUD's frame of play is a frame of main gameplay.
        countdown: hud::MAIN_GAMEPLAY_COUNTDOWN,
        countdown_starts,
    }
}

/// The game state in which the escape's time has run out (`$90:E0EC` sets it that frame).
const TIME_UP: u16 = 0x23;

/// The escape countdown's compared fields: whether it has begun, its time left in hundredths,
/// its age, and its place in 1/256ths of a pixel; a dormant countdown's are 0.
pub(super) fn countdown(countdown: Countdown) -> Vec<Observation> {
    let (begun, time, age, place) = match countdown {
        Countdown::Dormant => (false, 0, 0, interface_hud::Pair { x: 0, y: 0 }),
        Countdown::Begun(begun) => (
            true,
            begun.time_left.hundredths(),
            begun.age.frames(),
            begun.place.subpixels(),
        ),
    };
    vec![
        Observation::quantity("hud.countdown.begun", i64::from(begun)),
        Observation::quantity("hud.countdown.time", i64::from(time)),
        Observation::quantity("hud.countdown.age", i64::from(age)),
        Observation::quantity("hud.countdown.x", i64::from(place.x)),
        Observation::quantity("hud.countdown.y", i64::from(place.y)),
    ]
}

/// The expiry answer the record shows at the end of a frame of play: the game state turns to
/// the time-up's that frame.
pub(super) fn expiry_expected(wram: &[u8]) -> Result<Observation, Outcome> {
    let state = super::word(wram, source::GAME_STATE)?;
    Ok(expiry(state == TIME_UP))
}

/// The expiry answer the HUD's frame of play gave.
pub(super) fn expiry_actual(answer: CountdownExpiry) -> Observation {
    expiry(answer == CountdownExpiry::Expired)
}

fn expiry(expired: bool) -> Observation {
    Observation::quantity("hud.countdown.expiry", i64::from(expired))
}

/// The mini-map's window: rows 0 to 2, columns 26 to 30 of the HUD's 32 columns.
const MAP_WINDOW: core::ops::RangeInclusive<usize> = 26..=30;
const HUD_COLUMNS: usize = 32;

/// The HUD's observations with its cells split into the mini-map's window and the status bar,
/// so a difference names the one that differs.
pub(super) fn parts(fields: Vec<Observation>) -> Vec<Observation> {
    let mut parts = Vec::with_capacity(fields.len().saturating_add(1));
    for field in fields {
        if field.field != "hud.grid" {
            parts.push(field);
            continue;
        }
        let (window, bar): (Vec<_>, Vec<_>) = field
            .bytes
            .as_chunks::<2>()
            .0
            .iter()
            .enumerate()
            .partition(|(cell, _)| MAP_WINDOW.contains(&(cell % HUD_COLUMNS)));
        let bytes =
            |cells: Vec<(usize, &[u8; 2])>| cells.into_iter().flat_map(|(_, cell)| *cell).collect();
        parts.push(Observation {
            field: "hud.map-window".into(),
            bytes: bytes(window),
        });
        parts.push(Observation {
            field: "hud.status-bar".into(),
            bytes: bytes(bar),
        });
    }
    parts
}
