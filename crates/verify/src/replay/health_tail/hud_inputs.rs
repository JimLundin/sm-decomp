//! Independent early controls, powered-off map premises and late status neighbours.

use std::path::Path;

use game::production::health_tail::{Input, MapInput};
use interface_hud::{
    Ammo, Ammunition, CancelButton, Dropped, Equipped, FrameCount, ItemControlInputs, Reserve,
    ReserveMode, Room, SelectButton, Stock,
};
use serde::Deserialize;
use types::Pair;

use super::{CUTS, ORIGIN, ROW, Tape, read};
use crate::replay::{data_failure, result::Outcome};

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
    controls: Controls,
    map: MapPremises,
    status: Status,
}

#[derive(Deserialize)]
struct Controls {
    select: SelectButton,
    cancel: CancelButton,
    can_switch: bool,
    grapple_beam: bool,
    x_ray_scope: bool,
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
struct Status {
    energy_capacity: u16,
    reserve_mode: ReserveMode,
    reserve_energy: u16,
    missiles: [u16; 2],
    super_missiles: [u16; 2],
    power_bombs: [u16; 2],
    muted_by: Absent,
    collected: Absent,
    dropped_before: Dropped,
    dropped_after: Dropped,
}

#[derive(Deserialize)]
enum Absent {
    None,
}

#[derive(Clone, Copy)]
pub(super) struct Inputs {
    pub controls: Input,
    pub map: Input,
    pub status: Input,
}

pub(super) fn load(native: &Path, origin_pin: &str) -> Result<[Inputs; 2], Outcome> {
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
    let [first, next] = source.tape.opportunity;
    Ok([inputs(first, 0)?, inputs(next, 1)?])
}

fn inputs(observed: Opportunity, opportunity: u64) -> Result<Inputs, Outcome> {
    let Opportunity {
        end_cut,
        frame_count,
        controls,
        map,
        status,
    } = observed;
    let Status {
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
        || dropped_before != Dropped::Nothing
        || dropped_after != Dropped::Nothing
    {
        return Err(data_failure(
            Some(end_cut),
            "Health tail HUD opportunity premises",
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
    Ok(Inputs {
        controls: Input::HudControls {
            opportunity,
            frame: FrameCount(frame_count),
            controls: ItemControlInputs {
                select: controls.select,
                cancel: controls.cancel,
                can_switch: controls.can_switch,
                ammunition,
                equipped: Equipped {
                    grapple_beam: controls.grapple_beam,
                    x_ray_scope: controls.x_ray_scope,
                },
            },
        },
        map: Input::HudMap(MapInput {
            position: pair(map.position),
            room: Room {
                cell: Pair { x, y },
                blocks: pair(map.room_blocks),
            },
            map_collected: map.map_collected,
        }),
        status: Input::Status {
            capacity: energy_capacity,
            reserve: Reserve {
                energy: reserve_energy,
                mode: reserve_mode,
            },
            ammunition,
        },
    })
}
