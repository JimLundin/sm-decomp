//! Committed menu UI authority and rollback-preserving working values.
#![forbid(unsafe_code)]
use crate::menu_state::{
    DeathSequence, MapScreenState, MenuScreens, MenuState, OptionsMenu, SaveSelection,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub menu: MenuState,
    pub screens: MenuScreens,
}

/// Adapter residency. A leased or revoked domain cannot supply an old value.
pub enum Domain {
    Unowned,
    Owned { state: State, epoch: u64 },
    InSession { epoch: u64 },
    Leased,
    Revoked,
}

/// A transaction's fields, independent of the compatibility encoding.
pub trait Value: Copy + PartialEq {
    fn get(state: &State) -> Self;
    fn replace(value: &Self, state: &mut State);
    fn merge(before: &Self, after: &Self, state: &mut State);
}

macro_rules! value {
    ($ty:ty, $($path:ident).+, [$($field:ident),+]) => {
        impl Value for $ty {
            fn get(state: &State) -> Self { state.$($path).+ }
            fn replace(value: &Self, state: &mut State) { state.$($path).+ = *value; }
            fn merge(before: &Self, after: &Self, state: &mut State) {
                let target = &mut state.$($path).+;
                $(if before.$field != after.$field { target.$field = after.$field; })+
            }
        }
    };
}
value!(
    MenuState,
    menu,
    [
        menu_index,
        pausemenu_start_lr_pressed_highlight_timer,
        pausemenu_lr_animation_timer,
        pausemenu_item_selector_animation_timer,
        pausemenu_reserve_tank_animation_timer,
        unused_word_7e0731,
        pausemenu_uparrow_animation_timer,
        pausemenu_downarrow_animation_timer,
        pausemenu_rightarrow_animation_timer,
        pausemenu_leftarrow_animation_timer,
        pausemenu_palette_animation_timer,
        unused_word_7e073d,
        pausemenu_lr_animation_frame,
        pausemenu_item_selector_animation_frame,
        pausemenu_reserve_tank_animation_frame,
        unused_word_7e0745,
        pausemenu_uparrow_animation_frame,
        pausemenu_downarrow_animation_frame,
        pausemenu_rightarrow_animation_frame,
        pausemenu_leftarrow_animation_frame,
        pausemenu_palette_animation_frame,
        pausemenu_shoulder_button_highlight,
        pausemenu_button_label_mode,
        pausemenu_equipment_category_item,
        pausemenu_reserve_tank_delay_ctr,
        unused_word_7e0759,
        unused_word_7e075b,
        unused_word_7e075d,
        unused_word_7e075f,
        unused_word_7e0761,
        pause_screen_mode
    ]
);
value!(
    MapScreenState,
    screens.map,
    [
        min_x_scroll,
        max_x_scroll,
        min_y_scroll,
        max_y_scroll,
        gear_switch_timer,
        scrolling_direction,
        scrolling_speed_index,
        indicator_frame,
        indicator_timer,
        indicator_loop_counter
    ]
);
value!(
    SaveSelection,
    screens.save_selection,
    [
        map_unread,
        has_area_map,
        load_station,
        map_area_index,
        selected_slot,
        nonempty_slots,
        loading_game_state,
        save_station,
        save_area
    ]
);
value!(
    OptionsMenu,
    screens.options_menu,
    [menu_option_index, debug_invincibility, screen_index]
);
value!(DeathSequence, screens.death, [index, counter, flash_timer]);

impl Value for MenuScreens {
    fn get(state: &State) -> Self {
        state.screens
    }
    fn replace(value: &Self, state: &mut State) {
        state.screens = *value;
    }
    fn merge(before: &Self, after: &Self, state: &mut State) {
        MapScreenState::merge(&before.map, &after.map, state);
        SaveSelection::merge(&before.save_selection, &after.save_selection, state);
        OptionsMenu::merge(&before.options_menu, &after.options_menu, state);
        DeathSequence::merge(&before.death, &after.death, state);
    }
}

impl<A: Value, B: Value> Value for (A, B) {
    fn get(state: &State) -> Self {
        (A::get(state), B::get(state))
    }
    fn replace(value: &Self, state: &mut State) {
        A::replace(&value.0, state);
        B::replace(&value.1, state);
    }
    fn merge(before: &Self, after: &Self, state: &mut State) {
        A::merge(&before.0, &after.0, state);
        B::merge(&before.1, &after.1, state);
    }
}

/// Reads cannot admit an unrelated gameplay lease. Only a changed commit may
/// use the caller's existing admission seam; effects run synchronously.
pub trait Access {
    fn menu_state(&self) -> State;
    fn commit_menu<C: Value>(&mut self, before: &C, after: C);
    /// Ordered unpause clear; the palette animation and screen mode survive.
    /// The adapter preserves unconditional physical stores and no lease admission.
    fn reset_pause_menu(&mut self);
}

impl Access for State {
    fn menu_state(&self) -> State {
        *self
    }
    fn commit_menu<C: Value>(&mut self, before: &C, after: C) {
        C::merge(before, &after, self);
    }
    fn reset_pause_menu(&mut self) {
        self.clear_pause_menu();
    }
}

pub fn load<C: Value, H: Access + ?Sized>(game: &H) -> C {
    C::get(&game.menu_state())
}

/// Working values are unpublished drafts. A callback failure discards them,
/// leaving earlier inventory, drawing, sound and child commits intact.
pub fn with<C: Value, H: Access + ?Sized, T>(
    game: &mut H,
    action: impl FnOnce(&mut H, &mut C) -> T,
) -> T {
    let before = load::<C, H>(game);
    let mut working = before;
    let result = action(game, &mut working);
    if working != before {
        game.commit_menu(&before, working);
    }
    result
}

impl State {
    pub fn clear_pause_menu(&mut self) {
        self.menu.menu_index = 0;
        self.menu.pausemenu_start_lr_pressed_highlight_timer = 0;
        self.menu.pausemenu_lr_animation_timer = 0;
        self.menu.pausemenu_item_selector_animation_timer = 0;
        self.menu.pausemenu_reserve_tank_animation_timer = 0;
        self.menu.unused_word_7e0731 = 0;
        self.menu.pausemenu_uparrow_animation_timer = 0;
        self.menu.pausemenu_downarrow_animation_timer = 0;
        self.menu.pausemenu_rightarrow_animation_timer = 0;
        self.menu.pausemenu_leftarrow_animation_timer = 0;
        self.menu.unused_word_7e073d = 0;
        self.menu.pausemenu_lr_animation_frame = 0;
        self.menu.pausemenu_item_selector_animation_frame = 0;
        self.menu.pausemenu_reserve_tank_animation_frame = 0;
        self.menu.unused_word_7e0745 = 0;
        self.menu.pausemenu_uparrow_animation_frame = 0;
        self.menu.pausemenu_downarrow_animation_frame = 0;
        self.menu.pausemenu_rightarrow_animation_frame = 0;
        self.menu.pausemenu_leftarrow_animation_frame = 0;
        self.menu.pausemenu_shoulder_button_highlight = 0;
        self.menu.pausemenu_button_label_mode = 0;
        self.menu.pausemenu_equipment_category_item = 0;
        self.menu.pausemenu_reserve_tank_delay_ctr = 0;
        self.menu.unused_word_7e0759 = 0;
        self.menu.unused_word_7e075b = 0;
        self.menu.unused_word_7e075d = 0;
        self.menu.unused_word_7e075f = 0;
        self.menu.unused_word_7e0761 = 0;
    }
}
