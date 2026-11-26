//! Prepare the locked shaft's HUD in the same room batch as its timer.

use super::Fault;
use crate::exact::tilemap::{TileQueue, View};
use crate::intent::{OverlayRows, TileWrite};
use crate::production::hud::{HudPin, tilemap};
use crate::state::video::{NmiBatch, TilemapEntry};
use canonical::{Canonical, model::Model};
use interface_hud::{
    CancelButton, Controls, CountdownInputs, CountdownPicture, Dropped, FrameCount, FrameInputs,
    HerMap, HudPictures, MiniMapInputs, Room, SelectButton, Status, StockAtStart,
};
use types::Pair;

/// The carried HUD and its wrapping count of completed frames.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct LockedHud {
    /// The grid, mini-map and countdown.
    pub model: Model<HudPin>,
    /// The count from which the next room turn reads her blink phase.
    pub finished: u8,
}

/// HUD art and inputs held by the shaft's control and movement lock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HudContent {
    /// Status and map art.
    pub pictures: HudPictures,
    /// The map for the shaft's area.
    pub map: HerMap,
    /// The shaft's place and size on the map.
    pub room: Room,
    /// Her locked centre in the room.
    pub centre: Pair<u16>,
    /// Whether she has collected the area's map.
    pub map_collected: bool,
    /// Her unchanged item stock.
    pub stock: StockAtStart,
    /// Her retained reserve and ammunition, and the energy capacity.
    pub status: Status,
}

impl LockedHud {
    /// Carry the ordinary HUD and its held inputs into the locked room.
    #[must_use]
    pub fn from_ordinary(
        model: &Model<HudPin>,
        finished: u8,
        layout: (HudPictures, HerMap, Room),
        centre: Pair<u16>,
        energy: u16,
        frame: &crate::production::played::HudFrame,
    ) -> (Self, HudContent) {
        let (pictures, map, room) = layout;
        (
            Self {
                model: model.clone(),
                finished,
            },
            HudContent {
                pictures,
                map,
                room,
                centre,
                map_collected: frame.map_collected,
                stock: frame.stock_at_start,
                status: Status {
                    energy: interface_hud::Energy {
                        now: energy,
                        capacity: frame.capacity,
                    },
                    reserve: frame.reserve,
                    ammunition: frame.ammunition,
                },
            },
        )
    }

    pub(super) fn prepare(
        &mut self,
        content: &HudContent,
        energy: u16,
        countdown: CountdownInputs,
        batch: &mut NmiBatch,
    ) -> Result<CountdownPicture, Fault> {
        let mut status = content.status;
        status.energy.now = energy;
        let frame = FrameInputs {
            frame: FrameCount(self.finished),
            stock_at_start: content.stock,
            controls: Controls {
                select: SelectButton::NotPressed,
                cancel: CancelButton::NotPressed,
                can_switch: false,
            },
            mini_map: MiniMapInputs {
                runs: true,
                turned_on: false,
                position: content.centre,
                room: content.room,
                map: &content.map,
                map_collected: content.map_collected,
            },
            mother_brain_falls: false,
            status,
            collected: None,
            muted_by: None,
            dropped_before_status_bar: Dropped::Nothing,
            dropped_after_status_bar: Dropped::Nothing,
            countdown,
        };
        let out = self
            .model
            .0
            .frame_of_play(
                &interface_hud::Rules::reference(),
                &content.pictures,
                &frame,
            )
            .map_err(|_| Fault::Timer)?;
        if !out.events.is_empty() {
            return Err(Fault::Hud);
        }
        let bytes = tilemap(&self.model.0).map_err(|_| Fault::Hud)?;
        let cells = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&cell| TilemapEntry::from_word(u16::from_le_bytes(cell)))
            .collect();
        TileQueue::new(batch).write(
            View::default(),
            &TileWrite::Overlay(OverlayRows { row: 1, cells }),
        );
        Ok(out.countdown.picture)
    }
}
