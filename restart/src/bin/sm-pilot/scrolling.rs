//! Material postconditions for Ceres descent, first corridor, and stairwell.
use sm_native_host::{Game, ScrollCounters, State};

pub(super) fn author_inputs(mut recorded_route: Vec<u16>) -> Result<Vec<u16>, String> {
    if recorded_route.len() < 9380 || recorded_route[8379] != 0x081 {
        return Err("scrolling reference requires the retained Ceres route through step 9380".into());
    }
    recorded_route.truncate(9380);
    // One new firing edge while grounded and running right, before the ledge.
    // The runtime checks its reached context before consuming this input.
    recorded_route[8379] |= 0x200;
    Ok(recorded_route)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loading_and_mode7_transfers_do_not_count_as_ordinary_gameplay() {
        let mut material = Material::default();
        let mut ram = vec![0; 0x20000];
        let mut state = State { game_state: 11, area: 6, room: 0xdf8d, ..State::default() };
        let mut counts =
            ScrollCounters { consumed_columns: [17, 0], vram_dma_bytes: 2176, ..ScrollCounters::default() };
        material.observe(&state, &ram, counts);
        state.game_state = 8;
        counts.consumed_columns[0] += 1;
        material.observe(&state, &ram, counts);
        assert_eq!(material.corridor_columns, [0, 0]);
        ram[0x783] = 1;
        counts.consumed_columns[0] += 1;
        material.observe(&state, &ram, counts);
        assert_eq!(material.corridor_columns, [0, 0]);
        ram[0x783] = 0;
        counts.consumed_columns[0] += 1;
        counts.vram_dma_bytes += 128;
        material.observe(&state, &ram, counts);
        assert_eq!(material.corridor_columns, [1, 0]);
        assert_eq!(material.corridor_dma_bytes, 128);
    }
}

#[derive(Default)]
pub(super) struct Material {
    pub camera_x: u16,
    pub camera_y: u16,
    shaft_y_min: Option<u16>,
    shaft_y_max: u16,
    corridor_x_min: Option<u16>,
    corridor_x_max: u16,
    corridor_entered: bool,
    corridor_resumed: bool,
    previous_columns: [u64; 2],
    previous_rows: [u64; 2],
    corridor_columns: [u64; 2],
    stairs_rows: [u64; 2],
    stairs_y_min: Option<u16>,
    stairs_y_max: u16,
    previous_context: (u16, u16, u16),
    previous_dma_bytes: u64,
    corridor_dma_bytes: u64,
    stairs_dma_bytes: u64,
}

impl Material {
    pub fn observe(&mut self, state: &State, ram: &[u8], counts: ScrollCounters) {
        self.camera_x = u16::from_le_bytes([ram[0x0911], ram[0x0912]]);
        self.camera_y = u16::from_le_bytes([ram[0x0915], ram[0x0916]]);
        let columns = std::array::from_fn::<_, 2, _>(|i| counts.consumed_columns[i] - self.previous_columns[i]);
        let rows = std::array::from_fn::<_, 2, _>(|i| counts.consumed_rows[i] - self.previous_rows[i]);
        let dma_bytes = counts.vram_dma_bytes - self.previous_dma_bytes;
        let context = (state.game_state, state.area, state.room);
        let ordinary_gameplay =
            state.game_state == 8 && context == self.previous_context && ram[0x783..0x785] == [0, 0];
        self.previous_context = context;
        self.previous_columns = counts.consumed_columns;
        self.previous_rows = counts.consumed_rows;
        self.previous_dma_bytes = counts.vram_dma_bytes;
        if state.area != 6 {
            return;
        }
        if state.room == 0xdf45 && state.game_state == 8 {
            self.shaft_y_min = Some(self.shaft_y_min.map_or(self.camera_y, |n| n.min(self.camera_y)));
            self.shaft_y_max = self.shaft_y_max.max(self.camera_y);
        }
        if state.room == 0xdf8d {
            if ordinary_gameplay {
                for (total, count) in self.corridor_columns.iter_mut().zip(columns) {
                    *total += count;
                }
                self.corridor_dma_bytes += dma_bytes;
            }
            self.corridor_entered = true;
            if state.game_state == 8 {
                self.corridor_resumed = true;
                self.corridor_x_min = Some(self.corridor_x_min.map_or(self.camera_x, |n| n.min(self.camera_x)));
                self.corridor_x_max = self.corridor_x_max.max(self.camera_x);
            }
        }
        if state.room == 0xdfd7 {
            if ordinary_gameplay {
                for (total, count) in self.stairs_rows.iter_mut().zip(rows) {
                    *total += count;
                }
                self.stairs_dma_bytes += dma_bytes;
            }
            if state.game_state == 8 {
                self.stairs_y_min = Some(self.stairs_y_min.map_or(self.camera_y, |n| n.min(self.camera_y)));
                self.stairs_y_max = self.stairs_y_max.max(self.camera_y);
            }
        }
    }

    fn camera_complete(&self) -> bool {
        self.corridor_entered
            && self.corridor_resumed
            && self.shaft_y_min.is_some_and(|min| self.shaft_y_max.saturating_sub(min) >= 256)
            && self.corridor_x_min.is_some_and(|min| self.corridor_x_max.saturating_sub(min) >= 64)
            && self.stairs_y_min.is_some_and(|min| self.stairs_y_max.saturating_sub(min) >= 64)
    }

    pub fn complete(&self, game: &Game) -> bool {
        let counts = game.scroll_counters();
        self.camera_complete()
            && game.state().game_state == 8
            && game.state().area == 6
            && game.state().room == 0xdfd7
            && (0..3).all(|i| counts.c_entries[i] + counts.rust_completed[i] > 0)
            && counts.flush_requests[0] == 17 * (counts.c_entries[0] + counts.rust_completed[0])
            && counts.flush_requests == counts.flush_completions
            && counts.foreign_requests == counts.foreign_completions
            && counts.total_flush_requests == counts.total_flush_completions
            && self.corridor_columns[0] > 0
            && self.stairs_rows[0] > 0
            && self.corridor_dma_bytes > 0
            && self.stairs_dma_bytes > 0
            && counts.vram_dma_bytes > 0
    }

    pub fn report(&self, game: &Game) -> sm_binary_evidence::Value {
        let counts = game.scroll_counters();
        let mut report = sm_binary_evidence::Map::new();
        for (name, values) in [
            ("c_entries", counts.c_entries),
            ("rust_entries", counts.rust_entries),
            ("rust_completed", counts.rust_completed),
            ("rust_refused", counts.rust_refused),
            ("flush_requests", counts.flush_requests),
            ("flush_completions", counts.flush_completions),
            ("entry_vram_dma_bytes", counts.entry_vram_dma_bytes),
        ] {
            report.insert(
                name.into(),
                sm_binary_evidence::Value::Array(values.into_iter().map(|n| (n as i64).into()).collect()),
            );
        }
        for (name, values) in [
            ("consumed_columns", counts.consumed_columns),
            ("consumed_rows", counts.consumed_rows),
            ("corridor_columns", self.corridor_columns),
            ("stairs_rows", self.stairs_rows),
        ] {
            report.insert(
                name.into(),
                sm_binary_evidence::Value::Array(values.into_iter().map(|n| (n as i64).into()).collect()),
            );
        }
        for (name, value) in [
            ("foreign_requests", counts.foreign_requests),
            ("foreign_completions", counts.foreign_completions),
            ("total_flush_requests", counts.total_flush_requests),
            ("total_flush_completions", counts.total_flush_completions),
            ("vram_dma_bytes", counts.vram_dma_bytes),
            ("corridor_gameplay_dma_bytes", self.corridor_dma_bytes),
            ("stairs_gameplay_dma_bytes", self.stairs_dma_bytes),
        ] {
            report.insert(name.into(), (value as i64).into());
        }
        report.insert("shaft_camera_y_min".into(), self.shaft_y_min.map_or(-1, i64::from).into());
        report.insert("shaft_camera_y_max".into(), i64::from(self.shaft_y_max).into());
        report.insert("corridor_camera_x_min".into(), self.corridor_x_min.map_or(-1, i64::from).into());
        report.insert("corridor_camera_x_max".into(), i64::from(self.corridor_x_max).into());
        report.insert("stairs_camera_y_min".into(), self.stairs_y_min.map_or(-1, i64::from).into());
        report.insert("stairs_camera_y_max".into(), i64::from(self.stairs_y_max).into());
        report.insert("corridor_resumed".into(), self.corridor_resumed.into());
        report.insert("material_complete".into(), self.complete(game).into());
        sm_binary_evidence::Value::Table(report)
    }
}
