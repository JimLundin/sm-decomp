//! The HUD's independently observed neighbours in each frame: released controls, the
//! powered-off map's premises and the status values as the frame ends.

use std::path::Path;

use enum_map::EnumMap;
use game::production::health_tail::{HudFrame, MapInput};
use interface_hud::{
    Ammo, Ammunition, CancelButton, Controls, CountdownInputs, Equipped, FrameCount, Reserve,
    ReserveMode, Room, SelectButton, Stock, StockAtStart,
};
use serde::Deserialize;
use types::Pair;

use super::{CUTS, ORIGIN, ROW, Tape, read};
use crate::replay::{data_failure, hud, result::Outcome};

const PIN: &str = "a081de5b1d80cf60a7f0a1a475dbcf54efb2d8372fc15ae3ee6db0dfc51f2fec";

#[derive(Deserialize)]
struct Source {
    initial_cut: u64,
    initial_wram_sha256: String,
    #[serde(flatten)]
    tape: Tape<Opportunity>,
}

#[derive(Deserialize)]
struct Opportunity {
    end_cut: u64,
    frame_count: u8,
    controls: TapeControls,
    map: MapPremises,
    status: TapeStatus,
}

#[derive(Deserialize)]
struct TapeControls {
    select: Select,
    cancel: Cancel,
    can_switch: bool,
    grapple_beam: bool,
    x_ray_scope: bool,
}

#[derive(Clone, Copy, Deserialize)]
enum Select {
    NotPressed,
    Pressed,
}

#[derive(Clone, Copy, Deserialize)]
enum Cancel {
    NotPressed,
    Pressed,
    Held,
}

#[derive(Deserialize)]
struct MapPremises {
    runs: bool,
    #[serde(flatten)]
    events: MapEvents,
    room_cell: [i32; 2],
    room_blocks: [u16; 2],
    area: u8,
    position: [u16; 2],
    map_collected: bool,
}

#[derive(Deserialize)]
struct MapEvents {
    turned_on: bool,
    mother_brain_falls: bool,
}

#[derive(Deserialize)]
struct TapeStatus {
    energy_capacity: u16,
    reserve_mode: Refill,
    reserve_energy: u16,
    missiles: [u16; 2],
    super_missiles: [u16; 2],
    power_bombs: [u16; 2],
    muted_by: Absent,
    collected: Absent,
    dropped_before: Drop,
    dropped_after: Drop,
}

#[derive(Clone, Copy, Deserialize)]
enum Refill {
    Auto,
    Manual,
}

/// The tape names every drop; this scope admits only `Nothing`.
#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
enum Drop {
    Nothing,
    Selection,
    OneShotItemAlone,
}

#[derive(Deserialize)]
enum Absent {
    None,
}

/// Each frame's HUD inputs. Her stock as the first frame begins is the origin record's;
/// as the second begins, the stock the first frame's status bar drew.
pub(super) fn load(
    native: &Path,
    origin_pin: &str,
    origin_wram: &[u8],
) -> Result<[HudFrame; 2], Outcome> {
    let source: Source = read(
        native,
        "ceres-health-hud-operands-v1/hud-operands.toml",
        PIN,
    )?;
    if source.initial_cut != ORIGIN
        || source.initial_wram_sha256 != origin_pin
        || source.tape.row != ROW
        || source
            .tape
            .opportunity
            .each_ref()
            .map(|entry| entry.end_cut)
            != CUTS
    {
        return Err(data_failure(None, "Health tail HUD origin and schedule"));
    }
    // The origin is main gameplay and the HUD frames are its next two, so the scene runs the
    // escape countdown in each, nothing starts it, and the count of finished frames goes on
    // from the origin's by one.
    let countdown = CountdownInputs {
        runs: hud::countdown_runs(origin_wram, origin_wram)?,
        start: None,
        finished: hud::finished_frames(origin_wram)?,
    };
    let [first, next] = source.tape.opportunity;
    let first = inputs(first, hud::stock_at_start(origin_wram)?.counts, countdown)?;
    let drawn = EnumMap::from_fn(|ammo| first.ammunition.of(ammo).count);
    let finished = interface_hud::FinishedFrames(countdown.finished.0.wrapping_add(1));
    let next = inputs(
        next,
        drawn,
        CountdownInputs {
            finished,
            ..countdown
        },
    )?;
    Ok([first, next])
}

fn inputs(
    observed: Opportunity,
    counts_at_start: EnumMap<Ammo, u16>,
    countdown: CountdownInputs,
) -> Result<HudFrame, Outcome> {
    let Opportunity {
        end_cut,
        frame_count,
        controls,
        map,
        status,
    } = observed;
    let TapeStatus {
        energy_capacity,
        reserve_mode,
        reserve_energy,
        missiles,
        super_missiles,
        power_bombs,
        muted_by: Absent::None,
        collected: Absent::None,
        dropped_before,
        dropped_after,
    } = status;
    if !map.runs
        || map.events.turned_on
        || map.events.mother_brain_falls
        || data::room::Area::try_from(map.area) != Ok(data::room::Area::Ceres)
        || dropped_before != Drop::Nothing
        || dropped_after != Drop::Nothing
    {
        return Err(data_failure(
            Some(end_cut),
            "Health tail HUD frame premises",
        ));
    }
    let stock = |[count, capacity]: [u16; 2]| Stock { count, capacity };
    let ammunition = Ammunition::new(enum_map::enum_map! {
        Ammo::Missiles => stock(missiles),
        Ammo::SuperMissiles => stock(super_missiles),
        Ammo::PowerBombs => stock(power_bombs),
    })
    .ok_or_else(|| data_failure(Some(end_cut), "Health tail HUD ammunition range"))?;
    let pair = |[x, y]: [u16; 2]| Pair { x, y };
    let [x, y] = map.room_cell;
    Ok(HudFrame {
        frame: FrameCount(frame_count),
        stock_at_start: StockAtStart {
            counts: counts_at_start,
            equipped: Equipped {
                grapple_beam: controls.grapple_beam,
                x_ray_scope: controls.x_ray_scope,
            },
        },
        controls: Controls {
            select: match controls.select {
                Select::NotPressed => SelectButton::NotPressed,
                Select::Pressed => SelectButton::Pressed,
            },
            cancel: match controls.cancel {
                Cancel::NotPressed => CancelButton::NotPressed,
                Cancel::Pressed => CancelButton::Pressed,
                Cancel::Held => CancelButton::Held,
            },
            can_switch: controls.can_switch,
        },
        map: MapInput {
            position: pair(map.position),
            room: Room {
                cell: Pair { x, y },
                blocks: pair(map.room_blocks),
            },
            map_collected: map.map_collected,
        },
        capacity: energy_capacity,
        reserve: Reserve {
            energy: reserve_energy,
            mode: match reserve_mode {
                Refill::Auto => ReserveMode::Auto,
                Refill::Manual => ReserveMode::Manual,
            },
        },
        ammunition,
        countdown,
    })
}
