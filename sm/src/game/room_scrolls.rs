//! Room scrolling lives in `sm_game::room_scrolls`; the owned-Session view hosts it.
pub(crate) use sm_game::room_scrolls::*;

impl sm_game::room_scrolls::Host for crate::game::session_runtime::Runtime<'_> {
    fn unreachable(&mut self) -> bool {
        crate::rtl::unreachable()
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
        // The observations are refreshed on entry and on return, as the former
        // foreign call left them.
        self.owned_player_call(crate::game::display_refresh::flush_tilemaps_on_lease);
    }
}
