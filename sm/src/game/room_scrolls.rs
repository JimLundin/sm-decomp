//! Room scrolling lives in `sm_game::room_scrolls`; the owned-Session view hosts it.
pub(crate) use sm_game::room_scrolls::*;

impl sm_game::room_scrolls::Host for crate::game::session_runtime::Runtime<'_> {
    fn camera_screen_base(&self, second: bool) -> u8 {
        sm_game::runtime_access::SessionRam::byte(self, if second { 0x59 } else { 0x58 })
    }
    fn store_camera_background_size(&mut self, value: u16) {
        sm_game::runtime_access::SessionRam::write_word_le(
            self,
            value,
            sm_game::ram::words::SIZE_OF_BG2,
        );
    }
    fn unreachable(&mut self) -> bool {
        self.foreign(|game| crate::rtl::report_unreachable(game))
    }
    fn queue_scrolling_tiles(
        &mut self,
        k: u16,
        layout: sm_game::display::transfer::ScrollingLayout,
    ) {
        self.owned_call(|runtime| {
            sm_game::display_refresh::queue_scrolling_tiles(k, layout, runtime)
        });
    }
    fn flush_tilemaps(&mut self) {
        // SAFETY: room scrolling runs from the frame loop or a room-entry
        // effect with the backend installed and exclusively accessed.
        self.foreign(|game| unsafe { crate::game::display_refresh::flush_tilemaps(game) });
    }
}

impl sm_game::room_scrolls::state::Aliases for crate::game::session_runtime::Runtime<'_> {
    fn scroll_alias_byte(&self, index: usize) -> u8 {
        sm_game::runtime_access::SessionRam::byte(self, 0xcd20 + index)
    }
    fn store_scroll_alias_byte(&mut self, index: usize, value: u8) {
        sm_game::runtime_access::SessionRam::set_byte(self, 0xcd20 + index, value);
    }
    fn store_scroll_alias_pair(&mut self, index: usize, value: [u8; 2]) {
        sm_game::runtime_access::SessionRam::write_slice(
            self,
            0xcd20usize.checked_add(index).expect("scroll extent"),
            &value,
        );
    }
}
impl sm_game::room_scrolls::CameraHost for crate::game::session_runtime::Runtime<'_> {
    fn camera_unreachable(&mut self) -> bool {
        sm_game::room_scrolls::Host::unreachable(self)
    }
    fn publish_camera_position(&mut self, axis: usize, value: u16) {
        sm_game::runtime_access::SessionRam::write_word_le(self, value, 0x911 + 4 * axis);
    }
}
