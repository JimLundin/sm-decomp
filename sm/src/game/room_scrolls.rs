//! Room scrolling lives in `sm_game::room_scrolls`; the owned-Session view hosts it.
pub(crate) use sm_game::room_scrolls::*;

impl sm_game::room_scrolls::Host for crate::game::session_runtime::Runtime<'_> {
    fn unreachable(&mut self) -> bool {
        self.foreign(|game| crate::rtl::report_unreachable(game))
    }
    fn queue_scrolling_tiles(
        &mut self,
        k: u16,
        layout: sm_game::display::transfer::ScrollingLayout,
    ) {
        self.foreign(|game| crate::game::display_refresh::queue_scrolling_tiles(k, layout, game));
    }
    fn flush_tilemaps(&mut self) {
        // SAFETY: room scrolling runs from the frame loop or a room-entry
        // effect with the backend installed and exclusively accessed.
        self.foreign(|game| unsafe { crate::game::display_refresh::flush_tilemaps(game) });
    }
}
