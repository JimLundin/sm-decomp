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
#![forbid(unsafe_code)]
use crate::ram::CompatibilityRam;

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
