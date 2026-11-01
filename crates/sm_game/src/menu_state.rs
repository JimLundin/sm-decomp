//! The pause and file select menus' state, owned by `Session::menu`.
//!
//! # Design
//!
//! **Cohort.** The 31 words $0727..$0764: `menu_index`, the pause menu's
//! animation timers and frames, its shoulder button and category state, and
//! `pause_screen_mode`. No other RAM name aliases them. Each is the
//! little-endian value of its two RAM bytes.
//!
//! **Encoding boundary.** `session_encoding` projects every byte and decodes
//! or visits the cohort as one part; the auxiliary gameplay checkpoint
//! carries it.
//!
//! **Consumers.** The menus run on the compatibility host outside a lease and
//! write these words every frame. Each menu frame (the pause screen, the file
//! menu, the file select map and the game over menu) holds a [`MenuState`]
//! copy through [`crate::session_cohort::with`] and its functions read and
//! store the copy's fields; the words it changed are stored back once. A
//! consumer that moves onto a lease uses `session().menu` directly.
//!
//! **Raw writes.** A raw store outside a lease advances the RAM write epoch,
//! which suspends the projection, so later reads see RAM and the next lease
//! refuses a stale Session rather than exporting over the store. A lease host
//! has no raw RAM: its byte stores go through `write_projected_byte`, and
//! `foreign` exports before and imports after. Bulk writers that can cover
//! $0727..$0764 are the boot clears, the hard reset fill, and the snapshot
//! and seed loads. The clears and the fill are raw stores outside a lease;
//! a snapshot load is followed by a checked Session adoption, and a seed load
//! runs under a reference import. The unpause reset clears the range with
//! `word_transfers!` while no copy is held. No WRAM DMA, decompression, message
//! box clear or save file transfer reaches the range.

use crate::ram::words as ram;
use crate::session::Session;
use crate::session_cohort::Cohort;

macro_rules! menu_state {
    ($($field:ident: $variant:ident = $ram:ident),+ $(,)?) => {
        /// The menu words, in RAM order from $0727.
        #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
        pub struct MenuState {
            $(pub $field: u16,)+
        }

        /// A menu word, by its RAM name.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Word {
            $($variant,)+
        }

        impl MenuState {
            pub fn words(&self) -> [u16; WORDS] {
                [$(self.$field),+]
            }
            /// The menu word at RAM address `at`, when one starts there.
            pub fn word_at(&mut self, at: usize) -> Option<&mut u16> {
                let offset = at.checked_sub(MENU)?;
                if offset % 2 != 0 {
                    return None;
                }
                [$(&mut self.$field),+].into_iter().nth(offset / 2)
            }
        }

        impl Cohort for MenuState {
            fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
                let fields = [$(&mut self.$field),+];
                for (index, field) in fields.into_iter().enumerate() {
                    visit(MENU + 2 * index, field);
                }
            }
            fn session_words(session: &mut Session, visit: impl FnMut(usize, &mut u16)) {
                <Self as Cohort>::words(&mut session.menu, visit);
            }
        }

        const _: () = {
            let names = [$(crate::ram::words::$ram),+];
            let mut index = 0;
            while index < names.len() {
                assert!(names[index] == MENU + 2 * index);
                index += 1;
            }
        };
    };
}

menu_state!(
    menu_index: MenuIndex = MENU_INDEX,
    pausemenu_start_lr_pressed_highlight_timer: PausemenuStartLrPressedHighlightTimer = PAUSEMENU_START_LR_PRESSED_HIGHLIGHT_TIMER,
    pausemenu_lr_animation_timer: PausemenuLrAnimationTimer = PAUSEMENU_LR_ANIMATION_TIMER,
    pausemenu_item_selector_animation_timer: PausemenuItemSelectorAnimationTimer = PAUSEMENU_ITEM_SELECTOR_ANIMATION_TIMER,
    pausemenu_reserve_tank_animation_timer: PausemenuReserveTankAnimationTimer = PAUSEMENU_RESERVE_TANK_ANIMATION_TIMER,
    unused_word_7e0731: UnusedWord7e0731 = UNUSED_WORD_7E0731,
    pausemenu_uparrow_animation_timer: PausemenuUparrowAnimationTimer = PAUSEMENU_UPARROW_ANIMATION_TIMER,
    pausemenu_downarrow_animation_timer: PausemenuDownarrowAnimationTimer = PAUSEMENU_DOWNARROW_ANIMATION_TIMER,
    pausemenu_rightarrow_animation_timer: PausemenuRightarrowAnimationTimer = PAUSEMENU_RIGHTARROW_ANIMATION_TIMER,
    pausemenu_leftarrow_animation_timer: PausemenuLeftarrowAnimationTimer = PAUSEMENU_LEFTARROW_ANIMATION_TIMER,
    pausemenu_palette_animation_timer: PausemenuPaletteAnimationTimer = PAUSEMENU_PALETTE_ANIMATION_TIMER,
    unused_word_7e073d: UnusedWord7e073d = UNUSED_WORD_7E073D,
    pausemenu_lr_animation_frame: PausemenuLrAnimationFrame = PAUSEMENU_LR_ANIMATION_FRAME,
    pausemenu_item_selector_animation_frame: PausemenuItemSelectorAnimationFrame = PAUSEMENU_ITEM_SELECTOR_ANIMATION_FRAME,
    pausemenu_reserve_tank_animation_frame: PausemenuReserveTankAnimationFrame = PAUSEMENU_RESERVE_TANK_ANIMATION_FRAME,
    unused_word_7e0745: UnusedWord7e0745 = UNUSED_WORD_7E0745,
    pausemenu_uparrow_animation_frame: PausemenuUparrowAnimationFrame = PAUSEMENU_UPARROW_ANIMATION_FRAME,
    pausemenu_downarrow_animation_frame: PausemenuDownarrowAnimationFrame = PAUSEMENU_DOWNARROW_ANIMATION_FRAME,
    pausemenu_rightarrow_animation_frame: PausemenuRightarrowAnimationFrame = PAUSEMENU_RIGHTARROW_ANIMATION_FRAME,
    pausemenu_leftarrow_animation_frame: PausemenuLeftarrowAnimationFrame = PAUSEMENU_LEFTARROW_ANIMATION_FRAME,
    pausemenu_palette_animation_frame: PausemenuPaletteAnimationFrame = PAUSEMENU_PALETTE_ANIMATION_FRAME,
    pausemenu_shoulder_button_highlight: PausemenuShoulderButtonHighlight = PAUSEMENU_SHOULDER_BUTTON_HIGHLIGHT,
    pausemenu_button_label_mode: PausemenuButtonLabelMode = PAUSEMENU_BUTTON_LABEL_MODE,
    pausemenu_equipment_category_item: PausemenuEquipmentCategoryItem = PAUSEMENU_EQUIPMENT_CATEGORY_ITEM,
    pausemenu_reserve_tank_delay_ctr: PausemenuReserveTankDelayCtr = PAUSEMENU_RESERVE_TANK_DELAY_CTR,
    unused_word_7e0759: UnusedWord7e0759 = UNUSED_WORD_7E0759,
    unused_word_7e075b: UnusedWord7e075b = UNUSED_WORD_7E075B,
    unused_word_7e075d: UnusedWord7e075d = UNUSED_WORD_7E075D,
    unused_word_7e075f: UnusedWord7e075f = UNUSED_WORD_7E075F,
    unused_word_7e0761: UnusedWord7e0761 = UNUSED_WORD_7E0761,
    pause_screen_mode: PauseScreenMode = PAUSE_SCREEN_MODE,
);

/// Store `value` as byte `lane` (0 low, 1 high) of the menu word `word`.
pub fn set_menu_byte(word: &mut u16, lane: usize, value: u8) {
    let mut bytes = word.to_le_bytes();
    bytes[lane] = value;
    *word = u16::from_le_bytes(bytes);
}

/// RAM address of the first menu word, `menu_index`.
pub const MENU: usize = 0x727;
pub const WORDS: usize = 31;

/// The menu state and the menu screens a menu frame holds as one copy.
pub type MenuFrame = (MenuState, MenuScreens);

/// The menu screens' words outside the pause menu cohort, owned by
/// `Session::menu_screens`. Each is the little-endian value of its two RAM
/// bytes; `session_encoding` projects them and the auxiliary gameplay
/// checkpoint carries them.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct MenuScreens {
    pub map: MapScreenState,
    pub save_selection: SaveSelection,
    pub options_menu: OptionsMenu,
    pub death: DeathSequence,
}

impl Cohort for MenuScreens {
    fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
        self.map.words(&mut visit);
        self.save_selection.words(&mut visit);
        self.options_menu.words(&mut visit);
        self.death.words(&mut visit);
    }
    fn session_words(session: &mut Session, visit: impl FnMut(usize, &mut u16)) {
        session.menu_screens.words(visit);
    }
}

impl Cohort for MapScreenState {
    fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
        visit(ram::MAP_MIN_X_SCROLL, &mut self.min_x_scroll);
        visit(ram::MAP_MAX_X_SCROLL, &mut self.max_x_scroll);
        visit(ram::MAP_MIN_Y_SCROLL, &mut self.min_y_scroll);
        visit(ram::MAP_MAX_Y_SCROLL, &mut self.max_y_scroll);
        visit(
            ram::MAP_SCROLLING_GEAR_SWITCH_TIMER,
            &mut self.gear_switch_timer,
        );
        visit(ram::MAP_SCROLLING_DIRECTION, &mut self.scrolling_direction);
        visit(
            ram::MAP_SCROLLING_SPEED_INDEX,
            &mut self.scrolling_speed_index,
        );
        visit(
            ram::SAMUS_POSITION_INDICATOR_ANIMATION_FRAME,
            &mut self.indicator_frame,
        );
        visit(
            ram::SAMUS_POSITION_INDICATOR_ANIMATION_TIMER,
            &mut self.indicator_timer,
        );
        visit(
            ram::SAMUS_POSITION_INDICATOR_ANIMATION_LOOP_COUNTER,
            &mut self.indicator_loop_counter,
        );
    }
    fn session_words(session: &mut Session, visit: impl FnMut(usize, &mut u16)) {
        session.menu_screens.map.words(visit);
    }
}

impl Cohort for SaveSelection {
    fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
        visit(ram::FILE_SELECT_MAP_UNREAD, &mut self.map_unread);
        visit(ram::HAS_AREA_MAP, &mut self.has_area_map);
        visit(ram::LOAD_STATION_INDEX, &mut self.load_station);
        visit(ram::FILE_SELECT_MAP_AREA_INDEX, &mut self.map_area_index);
        visit(ram::SELECTED_SAVE_SLOT, &mut self.selected_slot);
        visit(ram::NONEMPTY_SAVE_SLOTS, &mut self.nonempty_slots);
        visit(ram::LOADING_GAME_STATE, &mut self.loading_game_state);
        visit(ram::SRAM_SAVE_STATION_INDEX, &mut self.save_station);
        visit(ram::SRAM_AREA_INDEX, &mut self.save_area);
    }
    fn session_words(session: &mut Session, visit: impl FnMut(usize, &mut u16)) {
        session.menu_screens.save_selection.words(visit);
    }
}

impl Cohort for OptionsMenu {
    fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
        visit(ram::MENU_OPTION_INDEX, &mut self.menu_option_index);
        visit(ram::DEBUG_INVINCIBILITY, &mut self.debug_invincibility);
        visit(ram::GAME_OPTIONS_SCREEN_INDEX, &mut self.screen_index);
    }
    fn session_words(session: &mut Session, visit: impl FnMut(usize, &mut u16)) {
        session.menu_screens.options_menu.words(visit);
    }
}

impl Cohort for DeathSequence {
    fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
        visit(ram::DEATH_INDEX, &mut self.index);
        visit(ram::DEATH_COUNTER, &mut self.counter);
        visit(ram::DEATH_FLASH_TIMER, &mut self.flash_timer);
    }
    fn session_words(session: &mut Session, visit: impl FnMut(usize, &mut u16)) {
        session.menu_screens.death.words(visit);
    }
}

/// The pause and file select map screens' scrolling and marker: the scroll
/// bounds ($05AC..$05B3), the scrolling gear, direction and speed
/// ($05FB..$0600), and Samus's position indicator animation ($0776..$077B).
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct MapScreenState {
    pub min_x_scroll: u16,
    pub max_x_scroll: u16,
    pub min_y_scroll: u16,
    pub max_y_scroll: u16,
    pub gear_switch_timer: u16,
    pub scrolling_direction: u16,
    pub scrolling_speed_index: u16,
    pub indicator_frame: u16,
    pub indicator_timer: u16,
    pub indicator_loop_counter: u16,
}

/// The save slot and map station selection: the file select map's unread
/// flag, the current area's collected map and the load station index
/// ($0787..$078C), the file select map area, the selected save slot and the
/// empty-slot bits ($0950..$0955), and the loading game state with the saved
/// station and area ($D914..$D919), which the save file carries.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct SaveSelection {
    pub map_unread: u16,
    pub has_area_map: u16,
    pub load_station: u16,
    pub map_area_index: u16,
    pub selected_slot: u16,
    pub nonempty_slots: u16,
    pub loading_game_state: u16,
    pub save_station: u16,
    pub save_area: u16,
}

/// The options menu's selection: the highlighted option ($099E), the debug
/// invincibility setting ($0DE0) and the options screen index ($0DE2), which
/// the title, intro, death and gameplay sequences also use as a delay.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct OptionsMenu {
    pub menu_option_index: u16,
    pub debug_invincibility: u16,
    pub screen_index: u16,
}

/// Samus's death sequence and the game over menu's progress: the death
/// sequence index, its counter and the flash timer ($0DE4..$0DE9).
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct DeathSequence {
    pub index: u16,
    pub counter: u16,
    pub flash_timer: u16,
}
