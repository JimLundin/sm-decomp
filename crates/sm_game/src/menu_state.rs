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
#![forbid(unsafe_code)]
use crate::ram::CompatibilityRam;

macro_rules! menu_state {
    ($($field:ident),+ $(,)?) => {
        /// The menu words, in RAM order from $0727.
        #[allow(non_snake_case)]
        #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
        pub struct MenuState {
            $(pub $field: u16,)+
        }

        /// A menu word, by its RAM name.
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Word {
            $($field,)+
        }

        impl MenuState {
            /// Each word's field, in RAM order, for the encoding boundary.
            pub fn words_mut(&mut self) -> [&mut u16; WORDS] {
                [$(&mut self.$field),+]
            }
            pub fn words(&self) -> [u16; WORDS] {
                [$(self.$field),+]
            }
        }

        const _: () = {
            let names = [$(crate::ram::words::$field),+];
            let mut index = 0;
            while index < names.len() {
                assert!(names[index] == MENU + 2 * index);
                index += 1;
            }
        };
    };
}

menu_state!(
    menu_index,
    pausemenu_start_lr_pressed_highlight_timer,
    pausemenu_lr_animation_timer,
    pausemenu_item_selector_animation_timer,
    pausemenu_reserve_tank_animation_timer,
    UNUSED_word_7E0731,
    pausemenu_uparrow_animation_timer,
    pausemenu_downarrow_animation_timer,
    pausemenu_rightarrow_animation_timer,
    pausemenu_leftarrow_animation_timer,
    pausemenu_palette_animation_timer,
    UNUSED_word_7E073D,
    pausemenu_lr_animation_frame,
    pausemenu_item_selector_animation_frame,
    pausemenu_reserve_tank_animation_frame,
    UNUSED_word_7E0745,
    pausemenu_uparrow_animation_frame,
    pausemenu_downarrow_animation_frame,
    pausemenu_rightarrow_animation_frame,
    pausemenu_leftarrow_animation_frame,
    pausemenu_palette_animation_frame,
    pausemenu_shoulder_button_highlight,
    pausemenu_button_label_mode,
    pausemenu_equipment_category_item,
    pausemenu_reserve_tank_delay_ctr,
    UNUSED_word_7E0759,
    UNUSED_word_7E075B,
    UNUSED_word_7E075D,
    UNUSED_word_7E075F,
    UNUSED_word_7E0761,
    pause_screen_mode,
);

/// RAM address of the first menu word, `menu_index`.
pub const MENU: usize = 0x727;
pub const WORDS: usize = 31;

/// The menu state as the host's projection holds it.
pub fn current<H: CompatibilityRam + ?Sized>(h: &H) -> MenuState {
    crate::session_encoding::menu(&|at| h.compatibility_byte(at))
}

/// `word` as the host's projection holds it, low byte then high.
pub fn get<H: CompatibilityRam + ?Sized>(h: &H, word: Word) -> u16 {
    let at = MENU + 2 * word as usize;
    u16::from_le_bytes([h.compatibility_byte(at), h.compatibility_byte(at + 1)])
}

/// Store `value` to `word`, low byte then high, as the named word store did.
pub fn set<H: CompatibilityRam + ?Sized>(h: &mut H, word: Word, value: u16) {
    let at = MENU + 2 * word as usize;
    for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
        h.compatibility_write_byte(at + lane, byte);
    }
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
