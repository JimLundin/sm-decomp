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
//! write these words every frame. They use [`menu!`](crate::menu), which reads
//! and stores through the host's projected bytes: the owned Session's
//! projection while one is installed, the byte image otherwise, with the same
//! bytes and order as the named word accesses it replaces. A consumer that
//! moves onto a lease uses `session().menu` directly.
//!
//! **Raw writes.** A raw store outside a lease advances the RAM write epoch,
//! which suspends the projection, so later reads see RAM and the next lease
//! refuses a stale Session rather than exporting over the store. A lease host
//! has no raw RAM: its byte stores go through `write_projected_byte`, and
//! `foreign` exports before and imports after. Bulk writers that can cover
//! $0727..$0764 are the boot clears, the hard reset fill, and the snapshot
//! and seed loads. The clears and the fill are raw stores outside a lease;
//! a snapshot load is followed by a checked Session adoption, and a seed load
//! runs under a reference import. No `word_transfers!`, WRAM DMA, decompression,
//! message box clear or save file transfer reaches the range.
use crate::ram::CompatibilityRam;
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

/// RAM address of the first menu word, `menu_index`.
pub const MENU: usize = 0x727;
pub const WORDS: usize = 31;

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
    fn of(session: &mut Session) -> &mut Self {
        &mut session.menu_screens
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
    fn of(session: &mut Session) -> &mut Self {
        &mut session.menu_screens.map
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
    fn of(session: &mut Session) -> &mut Self {
        &mut session.menu_screens.save_selection
    }
}

impl Cohort for OptionsMenu {
    fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
        visit(ram::MENU_OPTION_INDEX, &mut self.menu_option_index);
        visit(ram::DEBUG_INVINCIBILITY, &mut self.debug_invincibility);
        visit(ram::GAME_OPTIONS_SCREEN_INDEX, &mut self.screen_index);
    }
    fn of(session: &mut Session) -> &mut Self {
        &mut session.menu_screens.options_menu
    }
}

impl Cohort for DeathSequence {
    fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
        visit(ram::DEATH_INDEX, &mut self.index);
        visit(ram::DEATH_COUNTER, &mut self.counter);
        visit(ram::DEATH_FLASH_TIMER, &mut self.flash_timer);
    }
    fn of(session: &mut Session) -> &mut Self {
        &mut session.menu_screens.death
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

/// The map screen words through the host's projected bytes, by RAM address.
pub mod map_screen {
    use crate::ram::CompatibilityRam;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Word {
        MinXScroll = 0x5ac,
        MaxXScroll = 0x5ae,
        MinYScroll = 0x5b0,
        MaxYScroll = 0x5b2,
        GearSwitchTimer = 0x5fb,
        ScrollingDirection = 0x5fd,
        ScrollingSpeedIndex = 0x5ff,
        IndicatorFrame = 0x776,
        IndicatorTimer = 0x778,
        IndicatorLoopCounter = 0x77a,
    }

    const _: () = {
        use crate::ram::words as ram;
        assert!(ram::MAP_MIN_X_SCROLL == Word::MinXScroll as usize);
        assert!(ram::MAP_MAX_X_SCROLL == Word::MaxXScroll as usize);
        assert!(ram::MAP_MIN_Y_SCROLL == Word::MinYScroll as usize);
        assert!(ram::MAP_MAX_Y_SCROLL == Word::MaxYScroll as usize);
        assert!(ram::MAP_SCROLLING_GEAR_SWITCH_TIMER == Word::GearSwitchTimer as usize);
        assert!(ram::MAP_SCROLLING_DIRECTION == Word::ScrollingDirection as usize);
        assert!(ram::MAP_SCROLLING_SPEED_INDEX == Word::ScrollingSpeedIndex as usize);
        assert!(ram::SAMUS_POSITION_INDICATOR_ANIMATION_FRAME == Word::IndicatorFrame as usize);
        assert!(ram::SAMUS_POSITION_INDICATOR_ANIMATION_TIMER == Word::IndicatorTimer as usize);
        assert!(
            ram::SAMUS_POSITION_INDICATOR_ANIMATION_LOOP_COUNTER
                == Word::IndicatorLoopCounter as usize
        );
    };

    /// `word` as the host's projection holds it, low byte then high.
    pub fn get<H: CompatibilityRam + ?Sized>(h: &H, word: Word) -> u16 {
        crate::projection::word(h, word as usize)
    }
    /// Store `value` to `word`, low byte then high, as the named word store did.
    pub fn set<H: CompatibilityRam + ?Sized>(h: &mut H, word: Word, value: u16) {
        crate::projection::set_word(h, word as usize, value);
    }
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

/// The save selection words through the host's projected bytes, by RAM address.
pub mod save_selection {
    use crate::ram::CompatibilityRam;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Word {
        MapUnread = 0x787,
        HasAreaMap = 0x789,
        LoadStation = 0x78b,
        MapAreaIndex = 0x950,
        SelectedSlot = 0x952,
        NonemptySlots = 0x954,
        LoadingGameState = 0xd914,
        SaveStation = 0xd916,
        SaveArea = 0xd918,
    }

    const _: () = {
        use crate::ram::words as ram;
        assert!(ram::FILE_SELECT_MAP_UNREAD == Word::MapUnread as usize);
        assert!(ram::HAS_AREA_MAP == Word::HasAreaMap as usize);
        assert!(ram::LOAD_STATION_INDEX == Word::LoadStation as usize);
        assert!(ram::FILE_SELECT_MAP_AREA_INDEX == Word::MapAreaIndex as usize);
        assert!(ram::SELECTED_SAVE_SLOT == Word::SelectedSlot as usize);
        assert!(ram::NONEMPTY_SAVE_SLOTS == Word::NonemptySlots as usize);
        assert!(ram::LOADING_GAME_STATE == Word::LoadingGameState as usize);
        assert!(ram::SRAM_SAVE_STATION_INDEX == Word::SaveStation as usize);
        assert!(ram::SRAM_AREA_INDEX == Word::SaveArea as usize);
    };

    /// `word` as the host's projection holds it, low byte then high.
    pub fn get<H: CompatibilityRam + ?Sized>(h: &H, word: Word) -> u16 {
        crate::projection::word(h, word as usize)
    }
    /// Store `value` to `word`, low byte then high, as the named word store did.
    pub fn set<H: CompatibilityRam + ?Sized>(h: &mut H, word: Word, value: u16) {
        crate::projection::set_word(h, word as usize, value);
    }
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

/// The options menu words through the host's projected bytes, by RAM address.
pub mod options_menu {
    use crate::ram::CompatibilityRam;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Word {
        MenuOptionIndex = 0x99e,
        DebugInvincibility = 0xde0,
        ScreenIndex = 0xde2,
    }

    const _: () = {
        use crate::ram::words as ram;
        assert!(ram::MENU_OPTION_INDEX == Word::MenuOptionIndex as usize);
        assert!(ram::DEBUG_INVINCIBILITY == Word::DebugInvincibility as usize);
        assert!(ram::GAME_OPTIONS_SCREEN_INDEX == Word::ScreenIndex as usize);
    };

    /// `word` as the host's projection holds it, low byte then high.
    pub fn get<H: CompatibilityRam + ?Sized>(h: &H, word: Word) -> u16 {
        crate::projection::word(h, word as usize)
    }
    /// Store `value` to `word`, low byte then high, as the named word store did.
    pub fn set<H: CompatibilityRam + ?Sized>(h: &mut H, word: Word, value: u16) {
        crate::projection::set_word(h, word as usize, value);
    }
}

/// Samus's death sequence and the game over menu's progress: the death
/// sequence index, its counter and the flash timer ($0DE4..$0DE9).
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct DeathSequence {
    pub index: u16,
    pub counter: u16,
    pub flash_timer: u16,
}

/// The death sequence words through the host's projected bytes, by RAM address.
pub mod death_sequence {
    use crate::ram::CompatibilityRam;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Word {
        Index = 0xde4,
        Counter = 0xde6,
        FlashTimer = 0xde8,
    }

    const _: () = {
        use crate::ram::words as ram;
        assert!(ram::DEATH_INDEX == Word::Index as usize);
        assert!(ram::DEATH_COUNTER == Word::Counter as usize);
        assert!(ram::DEATH_FLASH_TIMER == Word::FlashTimer as usize);
    };

    /// `word` as the host's projection holds it, low byte then high.
    pub fn get<H: CompatibilityRam + ?Sized>(h: &H, word: Word) -> u16 {
        crate::projection::word(h, word as usize)
    }
    /// Store `value` to `word`, low byte then high, as the named word store did.
    pub fn set<H: CompatibilityRam + ?Sized>(h: &mut H, word: Word, value: u16) {
        crate::projection::set_word(h, word as usize, value);
    }
}

/// The menu state as the host's projection holds it.
pub fn current<H: CompatibilityRam + ?Sized>(h: &H) -> MenuState {
    crate::session_encoding::menu(&crate::projection::reader(h))
}

/// `word` as the host's projection holds it, low byte then high.
pub fn get<H: CompatibilityRam + ?Sized>(h: &H, word: Word) -> u16 {
    crate::projection::word(h, MENU + 2 * word as usize)
}

/// Store `value` to `word`, low byte then high, as the named word store did.
pub fn set<H: CompatibilityRam + ?Sized>(h: &mut H, word: Word, value: u16) {
    crate::projection::set_word(h, MENU + 2 * word as usize, value);
}

/// Byte `lane` (0 low, 1 high) of `word` as the host's projection holds it.
pub fn get_byte<H: CompatibilityRam + ?Sized>(h: &H, word: Word, lane: usize) -> u8 {
    crate::projection::byte(h, MENU + 2 * word as usize + lane)
}

/// Store byte `lane` (0 low, 1 high) of `word` through the host's projection.
pub fn set_byte<H: CompatibilityRam + ?Sized>(h: &mut H, word: Word, lane: usize, value: u8) {
    crate::projection::set_byte(h, MENU + 2 * word as usize + lane, value);
}

/// Typed access to the menu words through the host's projected bytes, in the
/// shapes the named word accesses used: read, store, `+=`/`-=` (reading the
/// old value first), and `-=` with a reread of the stored value.
#[macro_export]
macro_rules! menu {
    ($game:ident.$field:ident) => {
        $crate::menu_state::get(&*$game, $crate::menu_state::Word::$field)
    };
    ($game:ident.$field:ident = $value:expr $(,)?) => {{
        let value: u16 = $value;
        $crate::menu_state::set($game, $crate::menu_state::Word::$field, value)
    }};
    ($game:ident.$field:ident += $value:expr $(,)?) => {{
        let old = $crate::menu_state::get(&*$game, $crate::menu_state::Word::$field);
        let value = old.wrapping_add($value);
        $crate::menu_state::set($game, $crate::menu_state::Word::$field, value)
    }};
    ($game:ident.$field:ident -= $value:expr $(,)?) => {{
        let old = $crate::menu_state::get(&*$game, $crate::menu_state::Word::$field);
        let value = old.wrapping_sub($value);
        $crate::menu_state::set($game, $crate::menu_state::Word::$field, value)
    }};
    ($game:ident.$field:ident -= $value:expr, reread $(,)?) => {{
        $crate::menu!($game.$field -= $value);
        $crate::menu_state::get(&*$game, $crate::menu_state::Word::$field)
    }};
    ($game:ident.$field:ident += $value:expr, reread $(,)?) => {{
        $crate::menu!($game.$field += $value);
        $crate::menu_state::get(&*$game, $crate::menu_state::Word::$field)
    }};
}
