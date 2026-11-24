//! The HUD's inputs held from the seed, the frame it is brought, and its compared fields.

use super::platform::Controller;
use crate::replay::compare::Observation;
use crate::replay::hud;
use crate::replay::result::Outcome;
use game::production::played;

/// The HUD's inputs that no owner in the loop produces.
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

/// What a frame of play brings the HUD: the held facts, with the item buttons `controller`
/// reads.
pub(super) const fn frame(held: &Held, controller: Controller) -> played::HudFrame {
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
    }
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
