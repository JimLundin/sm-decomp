//! Spritemap drawing on the scheduler's lease. The object stores go straight
//! to the presentation's object owner, which the former `foreign` drawing
//! published through RAM and the next import read back.
#![forbid(unsafe_code)]
use super::Runtime;
use sm_game::session_owners::OBJECTS;
use sm_game::sprite_drawing::ObjectAccess;

/// The lease as a spritemap drawing host.
///
/// Object stores inside the table and cursor skip the observation refresh
/// that `Runtime::set_byte` makes after each byte: no observation reads
/// $0370..$0591. `Runtime::draw_objects` refreshes them around the drawing,
/// as `foreign` did on entry to the Game and on return.
pub(crate) struct LeaseDrawing<'r, 'g>(&'r mut Runtime<'g>);

impl LeaseDrawing<'_, '_> {
    fn native_order(&self) -> Option<sm_game::scene::state::WordOrder> {
        self.0
            .session()
            .presentation
            .owner()
            .ok()
            .map(|owner| owner.state.objects.native_order)
    }
    fn encode(&self, value: u16) -> [u8; 2] {
        self.native_order()
            .map_or_else(|| value.to_ne_bytes(), |order| order.encode(value))
    }
    /// Store `value` at `offset` from the first object entry; past the cursor
    /// word, the store reaches the RAM after it, as `Runtime::set_byte` does.
    fn store(&mut self, offset: usize, value: u8) {
        if offset < OBJECTS.len() {
            self.0
                .session_mut()
                .presentation
                .objects_mut()
                .write_byte(offset, value)
                .expect("object staging offset");
        } else {
            self.0.object_write_byte(offset, value);
        }
    }
}

impl LeaseDrawing<'_, '_> {
    /// Draw trail `pair`'s left or right object at the object cursor.
    pub(crate) fn draw_trail(&mut self, pair: usize, right: bool, frozen: bool) {
        let saved = self.0.object_little_word(544);
        if (saved as i16).wrapping_sub(512) >= 0 {
            return;
        }
        let (timer, x_field, y_field, tile_field) = if right {
            (0xd67c, 0xd754, 0xd79c, 0xd70c)
        } else {
            (0xd658, 0xd730, 0xd778, 0xd6e8)
        };
        if frozen && self.0.word(timer + 2 * pair) == 0 {
            return;
        }
        crate::verify_probe::hit(match (frozen, right) {
            (true, false) => "trails.draw.frozen_left",
            (true, true) => "trails.draw.frozen_right",
            (false, false) => "trails.draw.active_left",
            (false, true) => "trails.draw.active_right",
        });
        let x = self
            .0
            .word(x_field + 2 * pair)
            .wrapping_sub(self.0.word(0x911));
        if x & 0xff00 == 0 {
            // The cursor is below 512, so the entry's four bytes are objects.
            let entry = usize::from(saved);
            self.store_bytes(entry, &x.to_ne_bytes());
            // The original rereads after each overlapping native store.
            let y = self
                .0
                .word(y_field + 2 * pair)
                .wrapping_sub(self.0.word(0x915));
            if y & 0xff00 == 0 {
                self.store_bytes(entry + 1, &y.to_ne_bytes());
                let tile = self.0.word(tile_field + 2 * pair);
                self.store_bytes(entry + 2, &tile.to_ne_bytes());
                self.store_bytes(544, &saved.wrapping_add(4).to_le_bytes());
            }
        }
    }
    fn store_bytes(&mut self, offset: usize, bytes: &[u8]) {
        for (lane, &value) in bytes.iter().enumerate() {
            self.store(offset + lane, value);
        }
    }
}

impl sm_game::cartridge::LoadedImage for LeaseDrawing<'_, '_> {
    fn loaded_image(&self) -> &crate::rom::RomImage {
        sm_game::cartridge::LoadedImage::loaded_image(&*self.0)
    }
}

impl sm_game::records::Records for LeaseDrawing<'_, '_> {
    fn records(&self) -> &sm_game::records::RomRecords {
        sm_game::records::Records::records(&*self.0)
    }
}

impl ObjectAccess for LeaseDrawing<'_, '_> {
    fn object_native_word(&self, offset: usize) -> u16 {
        self.0.object_native_word(offset)
    }
    fn object_little_word(&self, offset: usize) -> u16 {
        self.0.object_little_word(offset)
    }
    fn object_write_byte(&mut self, offset: usize, value: u8) {
        self.store(offset, value);
    }
    fn object_write_native_slice(&mut self, offset: usize, value: u16) {
        // The slice store checks its whole extent before storing either byte.
        if offset + 2 <= OBJECTS.len() {
            let [low, high] = self.encode(value);
            self.store(offset, low);
            self.store(offset + 1, high);
        } else {
            self.0.object_write_native_slice(offset, value);
        }
    }
    fn object_write_native_bytes(&mut self, offset: usize, value: u16) {
        let [low, high] = self.encode(value);
        self.store(offset, low);
        self.store(offset + 1, high);
    }
}

/// The words Game's sprite output reads, from the lease's projection.
impl sm_game::sprite_output::SpriteWords for LeaseDrawing<'_, '_> {
    fn sprite_word(&self, word: sm_game::sprite_output::SpriteWord) -> u16 {
        self.0
            .word(crate::game::menu_host::sprite_word_address(word))
    }
    fn sprite_camera(&self) -> [u16; 2] {
        crate::game::menu_host::SPRITE_CAMERA.map(|at| self.0.word(at))
    }
    fn projectile_sprite_cursor(&self, slot: u16) -> u16 {
        self.0
            .native_word(crate::game::menu_host::projectile_sprite_address(slot))
    }
    fn echo_frame(&self) -> u16 {
        let runtime: &Runtime<'_> = self.0;
        sm_game::frame_display_word!(runtime.NmiFrameWordPair)
    }
    fn samus_pose(&self) -> u16 {
        self.0.session().player.pose
    }
}

impl sm_game::sprite_output::MapHost for LeaseDrawing<'_, '_> {
    fn warn_invalid_rom(&mut self, address: u32) {
        sm_game::sprite_output::MapHost::warn_invalid_rom(&mut *self.0.game, address);
    }
    fn set_failure(&mut self) {
        sm_game::sprite_output::MapHost::set_failure(&mut *self.0.game);
    }
}

impl Runtime<'_> {
    /// Run `draw` on the lease's object owner, refreshing the observations on
    /// entry and return as the former `foreign` drawing did.
    pub(crate) fn draw_objects<R>(
        &mut self,
        draw: impl FnOnce(&mut LeaseDrawing<'_, '_>) -> R,
    ) -> R {
        self.owned_player_call(|runtime| draw(&mut LeaseDrawing(runtime)))
    }
}
