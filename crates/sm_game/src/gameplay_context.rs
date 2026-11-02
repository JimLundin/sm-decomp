//! Input, game-loop and navigation capability over their single current owners.
//! A compatibility domain is absent while Session or an external writer owns it.
#![forbid(unsafe_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub game_loop: crate::game_loop::GameLoop,
    pub input: crate::controller_state::ControllerState,
    pub room: crate::room_view::RoomGeometry,
    pub effects: crate::room_view::RoomEffects,
}
pub enum Domain {
    Unowned,
    Owned { state: State, epoch: u64 },
    InSession { epoch: u64 },
    Leased,
    Revoked,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    Loop(crate::game_loop::Word),
    Input(crate::controller_state::Word),
    Area,
    Width,
    Height,
    ScrollWidth,
    ScrollHeight,
    EarthquakeKind,
    EarthquakeTimer,
    MapX,
    MapY,
    ElevatorProperties,
}
impl Word {
    pub fn address(self) -> usize {
        match self {
            Self::Loop(word) => word.address(),
            Self::Input(word) => word.address(),
            Self::Area => crate::ram::words::AREA_INDEX,
            Self::Width => crate::ram::words::ROOM_WIDTH_IN_BLOCKS,
            Self::Height => crate::ram::words::ROOM_HEIGHT_IN_BLOCKS,
            Self::ScrollWidth => crate::ram::words::ROOM_WIDTH_IN_SCROLLS,
            Self::ScrollHeight => crate::ram::words::ROOM_HEIGHT_IN_SCROLLS,
            Self::EarthquakeKind => crate::ram::words::EARTHQUAKE_TYPE,
            Self::EarthquakeTimer => crate::ram::words::EARTHQUAKE_TIMER,
            Self::MapX => crate::ram::words::ROOM_X_COORDINATE_ON_MAP,
            Self::MapY => crate::ram::words::ROOM_Y_COORDINATE_ON_MAP,
            Self::ElevatorProperties => crate::ram::words::ELEVATOR_DOOR_PROPERTIES_ORIENTATION,
        }
    }
    pub fn get(self, state: &State) -> u16 {
        match self {
            Self::Loop(word) => *word.get(&state.game_loop),
            Self::Input(word) => *word.get_state(&state.input),
            Self::Area => state.room.area,
            Self::Width => state.room.width,
            Self::Height => state.room.height,
            Self::ScrollWidth => state.room.scroll_width,
            Self::ScrollHeight => state.room.scroll_height,
            Self::EarthquakeKind => state.effects.earthquake_kind,
            Self::EarthquakeTimer => state.effects.earthquake_timer,
            Self::MapX => state.effects.map_x,
            Self::MapY => state.effects.map_y,
            Self::ElevatorProperties => state.effects.elevator_properties,
        }
    }
    pub fn get_mut(self, state: &mut State) -> &mut u16 {
        match self {
            Self::Loop(word) => word.get_mut(&mut state.game_loop),
            Self::Input(word) => word.get_state_mut(&mut state.input),
            Self::Area => &mut state.room.area,
            Self::Width => &mut state.room.width,
            Self::Height => &mut state.room.height,
            Self::ScrollWidth => &mut state.room.scroll_width,
            Self::ScrollHeight => &mut state.room.scroll_height,
            Self::EarthquakeKind => &mut state.effects.earthquake_kind,
            Self::EarthquakeTimer => &mut state.effects.earthquake_timer,
            Self::MapX => &mut state.effects.map_x,
            Self::MapY => &mut state.effects.map_y,
            Self::ElevatorProperties => &mut state.effects.elevator_properties,
        }
    }
    pub fn get_session(self, state: &crate::session::Session) -> u16 {
        match self {
            Self::Loop(word) => *word.get(&state.game_loop),
            Self::Input(word) => *word.get(&state.control),
            Self::Area => state.room.area,
            Self::Width => state.room.width,
            Self::Height => state.room.height,
            Self::ScrollWidth => state.room.scroll_width,
            Self::ScrollHeight => state.room.scroll_height,
            Self::EarthquakeKind => state.feedback.earthquake_kind,
            Self::EarthquakeTimer => state.feedback.earthquake_timer,
            Self::MapX => state.room.entry.map_x,
            Self::MapY => state.room.entry.map_y,
            Self::ElevatorProperties => state.room.entry.elevator_properties,
        }
    }
    pub fn get_session_mut(self, state: &mut crate::session::Session) -> &mut u16 {
        match self {
            Self::Loop(word) => word.get_mut(&mut state.game_loop),
            Self::Input(word) => word.get_mut(&mut state.control),
            Self::Area => &mut state.room.area,
            Self::Width => &mut state.room.width,
            Self::Height => &mut state.room.height,
            Self::ScrollWidth => &mut state.room.scroll_width,
            Self::ScrollHeight => &mut state.room.scroll_height,
            Self::EarthquakeKind => &mut state.feedback.earthquake_kind,
            Self::EarthquakeTimer => &mut state.feedback.earthquake_timer,
            Self::MapX => &mut state.room.entry.map_x,
            Self::MapY => &mut state.room.entry.map_y,
            Self::ElevatorProperties => &mut state.room.entry.elevator_properties,
        }
    }
}
impl State {
    pub fn from_session(session: &crate::session::Session) -> Self {
        let mut input = crate::controller_state::ControllerState::default();
        for word in crate::controller_state::Word::ALL {
            *word.get_state_mut(&mut input) = *word.get(&session.control);
        }
        Self {
            game_loop: session.game_loop.clone(),
            input,
            room: crate::room_view::RoomGeometry {
                area: session.room.area,
                width: session.room.width,
                height: session.room.height,
                scroll_width: session.room.scroll_width,
                scroll_height: session.room.scroll_height,
            },
            effects: crate::room_view::RoomEffects {
                earthquake_kind: session.feedback.earthquake_kind,
                earthquake_timer: session.feedback.earthquake_timer,
                map_x: session.room.entry.map_x,
                map_y: session.room.entry.map_y,
                elevator_properties: session.room.entry.elevator_properties,
            },
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GrappleAnimation {
    pub timer: u16,
    pub tiles: u16,
}
pub trait Access {
    fn grapple_animation(&self) -> GrappleAnimation {
        panic!("grapple animation owner was not supplied")
    }
    fn commit_grapple_animation(&mut self, _before: GrappleAnimation, _after: GrappleAnimation) {
        panic!("grapple animation owner was not supplied")
    }

    fn inventory_state(&self) -> crate::inventory::Inventory {
        panic!("inventory owner was not supplied")
    }
    fn commit_inventory(
        &mut self,
        _before: &crate::inventory::Inventory,
        _after: crate::inventory::Inventory,
    ) {
        panic!("inventory owner was not supplied")
    }
    fn gameplay_word(&self, word: Word) -> u16;
    /// Direct stores preserve each physical Game or projected Runtime byte prefix.
    fn set_gameplay_byte(&mut self, word: Word, lane: usize, value: u8);
    fn set_gameplay_word(&mut self, word: Word, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.set_gameplay_byte(word, 0, low);
        self.set_gameplay_byte(word, 1, high);
    }
    fn navigation(&self) -> crate::room_view::RoomGeometry {
        use Word::*;
        crate::room_view::RoomGeometry {
            area: self.gameplay_word(Area),
            width: self.gameplay_word(Width),
            height: self.gameplay_word(Height),
            scroll_width: self.gameplay_word(ScrollWidth),
            scroll_height: self.gameplay_word(ScrollHeight),
        }
    }
    fn room_effects(&self) -> crate::room_view::RoomEffects {
        use Word::*;
        crate::room_view::RoomEffects {
            earthquake_kind: self.gameplay_word(EarthquakeKind),
            earthquake_timer: self.gameplay_word(EarthquakeTimer),
            map_x: self.gameplay_word(MapX),
            map_y: self.gameplay_word(MapY),
            elevator_properties: self.gameplay_word(ElevatorProperties),
        }
    }
}
pub fn navigation<H: Access + ?Sized>(host: &H) -> crate::room_view::RoomGeometry {
    host.navigation()
}
pub fn room_effects<H: Access + ?Sized>(host: &H) -> crate::room_view::RoomEffects {
    host.room_effects()
}
pub fn set_room<H: Access + ?Sized>(host: &mut H, word: crate::room_view::Word, value: u16) {
    let selected = match word {
        crate::room_view::Word::Area => Word::Area,
        crate::room_view::Word::Width => Word::Width,
        crate::room_view::Word::Height => Word::Height,
        crate::room_view::Word::ScrollWidth => Word::ScrollWidth,
        crate::room_view::Word::ScrollHeight => Word::ScrollHeight,
    };
    host.set_gameplay_word(selected, value);
}
pub fn set_effect<H: Access + ?Sized>(
    host: &mut H,
    word: crate::room_view::EffectWord,
    value: u16,
) {
    let selected = match word {
        crate::room_view::EffectWord::EarthquakeKind => Word::EarthquakeKind,
        crate::room_view::EffectWord::EarthquakeTimer => Word::EarthquakeTimer,
        crate::room_view::EffectWord::MapX => Word::MapX,
        crate::room_view::EffectWord::MapY => Word::MapY,
        crate::room_view::EffectWord::ElevatorProperties => Word::ElevatorProperties,
    };
    host.set_gameplay_word(selected, value);
}
pub fn enter_header<H: Access + ?Sized>(host: &mut H, bytes: [u8; 3]) {
    for (word, byte) in [Word::Area, Word::MapX, Word::MapY].into_iter().zip(bytes) {
        host.set_gameplay_byte(word, 0, byte);
    }
}
#[macro_export]
macro_rules! gameplay_word {
    ($host:ident.$name:ident = $value:expr $(,)?) => {{ let value: u16 = $value; $crate::gameplay_context::Access::set_gameplay_word($host, $crate::gameplay_word!(@field $name), value) }};
    ($host:ident.$name:ident += $value:expr $(, $reread:ident)?) => { $crate::gameplay_word!($host.$name, |old| old.wrapping_add($value) $(, $reread)?) };
    ($host:ident.$name:ident -= $value:expr $(, $reread:ident)?) => { $crate::gameplay_word!($host.$name, |old| old.wrapping_sub($value) $(, $reread)?) };
    ($host:ident.$name:ident &= $value:expr $(, $reread:ident)?) => { $crate::gameplay_word!($host.$name, |old| old & $value $(, $reread)?) };
    ($host:ident.$name:ident |= $value:expr $(, $reread:ident)?) => { $crate::gameplay_word!($host.$name, |old| old | $value $(, $reread)?) };
    ($host:ident.$name:ident ^= $value:expr $(, $reread:ident)?) => { $crate::gameplay_word!($host.$name, |old| old ^ $value $(, $reread)?) };
    ($host:ident.$name:ident, |$old:ident| $value:expr, reread) => {{ $crate::gameplay_word!($host.$name, |$old| $value); $crate::gameplay_word!($host.$name) }};
    ($host:ident.$name:ident, |$old:ident| $value:expr) => {{ let $old = $crate::gameplay_word!($host.$name); let value: u16 = $value; $crate::gameplay_word!($host.$name = value) }};
    ($host:ident.$name:ident) => { $crate::gameplay_context::Access::gameplay_word(&*$host, $crate::gameplay_word!(@field $name)) };
    ($host:ident; $($name:ident = $value:expr;)+) => {{ $( $crate::gameplay_word!($host.$name = $value); )+ }};
    ($host:ident, $value:expr; $($name:ident),+ $(,)?) => {{ let value: u16 = $value; $( $crate::gameplay_word!($host.$name = value); )+ }};
    (@field GAME_STATE) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::GameState) };
    (@field GAME_TIME_FRAMES) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::GameTimeFrames) };
    (@field GAME_TIME_SECONDS) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::GameTimeSeconds) };
    (@field GAME_TIME_MINUTES) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::GameTimeMinutes) };
    (@field GAME_TIME_HOURS) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::GameTimeHours) };
    (@field ENABLE_DEBUG) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::EnableDebug) };
    (@field JOYPAD_DBG_1) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::JoypadDbg1) };
    (@field JOYPAD_DBG_2) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::JoypadDbg2) };
    (@field JOYPAD_DBG_MISSILES_SWAP) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::JoypadDbgMissilesSwap) };
    (@field JOYPAD_DBG_SUPER_MISSILES_SWAP) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::JoypadDbgSuperMissilesSwap) };
    (@field JOYPAD_DBG_POWER_BOMBS_SWAP) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::JoypadDbgPowerBombsSwap) };
    (@field JOYPAD_DBG_FLAGS) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::JoypadDbgFlags) };
    (@field DEBUG_SAVELOAD_SCROLLPOS_TOGGLE) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::DebugSaveloadScrollposToggle) };
    (@field DEBUG_SAVED_XSCROLL) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::DebugSavedXscroll) };
    (@field DEBUG_SAVED_YSCROLL) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::DebugSavedYscroll) };
    (@field BUTTON_CONFIG_UP) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::ButtonConfigUp) };
    (@field BUTTON_CONFIG_DOWN) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::ButtonConfigDown) };
    (@field BUTTON_CONFIG_LEFT) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::ButtonConfigLeft) };
    (@field BUTTON_CONFIG_RIGHT) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::ButtonConfigRight) };
    (@field BUTTON_CONFIG_SHOOT_X_SAVED) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::ButtonConfigShootXSaved) };
    (@field BUTTON_CONFIG_JUMP_A_SAVED) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::ButtonConfigJumpASaved) };
    (@field BUTTON_CONFIG_RUN_B_SAVED) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::ButtonConfigRunBSaved) };
    (@field BUTTON_CONFIG_ITEMCANCEL_Y_SAVED) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::ButtonConfigItemcancelYSaved) };
    (@field BUTTON_CONFIG_ITEMSWITCH_SAVED) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::ButtonConfigItemswitchSaved) };
    (@field BUTTON_CONFIG_AIM_DOWN_L_SAVED) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::ButtonConfigAimDownLSaved) };
    (@field BUTTON_CONFIG_AIM_UP_R_SAVED) => { $crate::gameplay_context::Word::Loop($crate::game_loop::Word::ButtonConfigAimUpRSaved) };
    (@field JOYPAD1_LASTKEYS) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::Joypad1Lastkeys) };
    (@field JOYPAD1_NEWKEYS) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::Joypad1Newkeys) };
    (@field JOYPAD1_NEWKEYS2_UNUSED) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::Joypad1Newkeys2Unused) };
    (@field JOYPAD1_PREV) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::Joypad1Prev) };
    (@field JOYPAD1_KEYREPEAT_CTR) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::Joypad1KeyrepeatCtr) };
    (@field JOYPAD2_LAST) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::Joypad2Last) };
    (@field JOYPAD2_NEW_KEYS) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::Joypad2NewKeys) };
    (@field JOYPAD2_NEWKEYS2) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::Joypad2Newkeys2) };
    (@field JOYPAD2_PREV) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::Joypad2Prev) };
    (@field JOYPAD2_KEYREPEAT_CTR) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::Joypad2KeyrepeatCtr) };
    (@field JOYPAD_CTR_REPEAT_FIRST) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::JoypadCtrRepeatFirst) };
    (@field JOYPAD_CTR_REPEAT_NEXT) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::JoypadCtrRepeatNext) };
    (@field JOYPAD_RELEASED_KEYS) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::JoypadReleasedKeys) };
    (@field TIMED_HELD_INPUT_TIMER) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::TimedHeldInputTimer) };
    (@field TIMED_HELD_INPUT_TIMER_RESET) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::TimedHeldInputTimerReset) };
    (@field TIMED_HELD_INPUT) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::TimedHeldInput) };
    (@field PREVIOUS_TIMED_HELD_INPUT) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::PreviousTimedHeldInput) };
    (@field NEWLY_HELD_DOWN_TIMED_HELD_INPUT) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::NewlyHeldDownTimedHeldInput) };
    (@field JOYPAD1_INPUT_SAMUSFILTER) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::Joypad1InputSamusfilter) };
    (@field JOYPAD1_NEWINPUT_SAMUSFILTER) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::Joypad1NewinputSamusfilter) };
    (@field BUG_FIX_COUNTER) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::BugFixCounter) };
    (@field BUTTON_CONFIG_SHOOT_X) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::ButtonConfigShootX) };
    (@field BUTTON_CONFIG_JUMP_A) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::ButtonConfigJumpA) };
    (@field BUTTON_CONFIG_RUN_B) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::ButtonConfigRunB) };
    (@field BUTTON_CONFIG_ITEMCANCEL_Y) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::ButtonConfigItemcancelY) };
    (@field BUTTON_CONFIG_ITEMSWITCH) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::ButtonConfigItemswitch) };
    (@field BUTTON_CONFIG_AIM_DOWN_L) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::ButtonConfigAimDownL) };
    (@field BUTTON_CONFIG_AIM_UP_R) => { $crate::gameplay_context::Word::Input($crate::controller_state::Word::ButtonConfigAimUpR) };
 }

/// Copy rollback and changed-only publication match the former inventory leaf.
pub fn inventory<H: Access + ?Sized>(host: &H) -> crate::inventory::Inventory {
    host.inventory_state()
}
pub fn with_inventory<H: Access + ?Sized, T>(
    host: &mut H,
    action: impl FnOnce(&mut H, &mut crate::inventory::Inventory) -> T,
) -> T {
    let before = host.inventory_state();
    let mut after = before.clone();
    let result = action(host, &mut after);
    if before != after {
        host.commit_inventory(&before, after);
    }
    result
}

pub fn with_grapple_animation<H: Access + ?Sized, T>(
    host: &mut H,
    action: impl FnOnce(&mut H, &mut GrappleAnimation) -> T,
) -> T {
    let before = host.grapple_animation();
    let mut after = before;
    let result = action(host, &mut after);
    if before != after {
        host.commit_grapple_animation(before, after);
    }
    result
}
