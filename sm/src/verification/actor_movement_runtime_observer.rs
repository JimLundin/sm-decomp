//! Included as a private session_runtime child only in the verification DLL.
#![forbid(unsafe_code)]
use super::Runtime;
use crate::ram_map::words as w;
use crate::types::EnemyData;
use core::mem::offset_of;

impl Runtime<'_> {
    pub(crate) fn observe_actor_movement(&mut self, tag: u8) {
        let Some(id) = crate::actor_movement_observer::active_case() else {
            return;
        };
        if tag == 4 {
            // Exact predecessor test injection through the existing projection,
            // never through a cloned owner or a second movement implementation.
            for (field, value) in [
                (offset_of!(EnemyData, x_pos), 200),
                (offset_of!(EnemyData, x_subpos), 7),
                (offset_of!(EnemyData, y_pos), 300),
                (offset_of!(EnemyData, y_subpos), 9),
            ] {
                self.write_slice_le(0xf78 + field, value);
            }
            self.write_slice_le(w::cur_enemy_index, 64);
            self.write_slice_le(0x10002 + 264 * 2, 0);
            self.set_byte(0x16402 + 264, 0x55);
            self.write_slice_le(w::room_width_in_blocks, 64);
        }
        // No export/import is added just to inspect state. After the actual
        // checked foreign failure, state is None and Game RAM is authoritative.
        let ram = if self.state.is_some() {
            (0..131072).map(|at| self.byte(at)).collect()
        } else {
            (0..self.game.ram().len())
                .map(|at| self.game.ram().read_byte(at))
                .collect::<Vec<_>>()
        };
        crate::actor_movement_observer::record_snapshot(tag, ram, self.game.g_fail);
        if tag == 2 && id == 28 {
            self.game.loaded_rom = Some(crate::rom::RomImage::zeroed(
                crate::actor_movement_cases::offset(0x849002) + 1,
            ));
        }
    }
}
