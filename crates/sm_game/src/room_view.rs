//! The room's area, geometry, effect and placement words for code on the
//! compatibility host.
//!
//! `Session::room` owns these words (`area`, `width`, `height`,
//! `scroll_width`, `scroll_height`); `session_encoding` projects them and
//! names their layout once. Code that runs outside a lease (the menus, the
//! display refresh, the intro) reads and stores them as [`RoomGeometry`] and
//! [`RoomEffects`] copies through `session_cohort`. Lease code uses
//! `session().room` directly.
use crate::ram::words as ram;
use crate::session::Session;
use crate::session_cohort::Cohort;

/// The owned room words this view covers.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct RoomGeometry {
    /// The area index (`area_index`).
    pub area: u16,
    /// The room's size in blocks and in scrolls.
    pub width: u16,
    pub height: u16,
    pub scroll_width: u16,
    pub scroll_height: u16,
}

/// The room's effect and placement words: the earthquake, the Tourian
/// entrance statue and the room's map position and elevator door.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct RoomEffects {
    pub earthquake_kind: u16,
    pub earthquake_timer: u16,
    pub statue_animation: u16,
    pub statue_finished: u16,
    pub map_x: u16,
    pub map_y: u16,
    pub elevator_properties: u16,
}

impl RoomGeometry {
    fn visit(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
        visit(ram::AREA_INDEX, &mut self.area);
        visit(ram::ROOM_WIDTH_IN_BLOCKS, &mut self.width);
        visit(ram::ROOM_HEIGHT_IN_BLOCKS, &mut self.height);
        visit(ram::ROOM_WIDTH_IN_SCROLLS, &mut self.scroll_width);
        visit(ram::ROOM_HEIGHT_IN_SCROLLS, &mut self.scroll_height);
    }
}

impl Cohort for RoomGeometry {
    fn words(&mut self, visit: impl FnMut(usize, &mut u16)) {
        self.visit(visit);
    }
    fn session_words(session: &mut Session, mut visit: impl FnMut(usize, &mut u16)) {
        let room = &mut session.room;
        visit(ram::AREA_INDEX, &mut room.area);
        visit(ram::ROOM_WIDTH_IN_BLOCKS, &mut room.width);
        visit(ram::ROOM_HEIGHT_IN_BLOCKS, &mut room.height);
        visit(ram::ROOM_WIDTH_IN_SCROLLS, &mut room.scroll_width);
        visit(ram::ROOM_HEIGHT_IN_SCROLLS, &mut room.scroll_height);
    }
}

impl Cohort for RoomEffects {
    fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
        visit(ram::EARTHQUAKE_TYPE, &mut self.earthquake_kind);
        visit(ram::EARTHQUAKE_TIMER, &mut self.earthquake_timer);
        visit(
            ram::TOURIAN_ENTRANCE_STATUE_ANIMSTATE,
            &mut self.statue_animation,
        );
        visit(
            ram::TOURIAN_ENTRANCE_STATUE_FINISHED,
            &mut self.statue_finished,
        );
        visit(ram::ROOM_X_COORDINATE_ON_MAP, &mut self.map_x);
        visit(ram::ROOM_Y_COORDINATE_ON_MAP, &mut self.map_y);
        visit(
            ram::ELEVATOR_DOOR_PROPERTIES_ORIENTATION,
            &mut self.elevator_properties,
        );
    }
    fn session_words(session: &mut Session, mut visit: impl FnMut(usize, &mut u16)) {
        visit(ram::EARTHQUAKE_TYPE, &mut session.feedback.earthquake_kind);
        visit(
            ram::EARTHQUAKE_TIMER,
            &mut session.feedback.earthquake_timer,
        );
        visit(
            ram::TOURIAN_ENTRANCE_STATUE_ANIMSTATE,
            &mut session.mechanisms.statue_animation,
        );
        visit(
            ram::TOURIAN_ENTRANCE_STATUE_FINISHED,
            &mut session.mechanisms.statue_finished,
        );
        let entry = &mut session.room.entry;
        visit(ram::ROOM_X_COORDINATE_ON_MAP, &mut entry.map_x);
        visit(ram::ROOM_Y_COORDINATE_ON_MAP, &mut entry.map_y);
        visit(
            ram::ELEVATOR_DOOR_PROPERTIES_ORIENTATION,
            &mut entry.elevator_properties,
        );
    }
}
