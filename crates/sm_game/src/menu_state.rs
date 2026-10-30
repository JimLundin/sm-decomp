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
    pause_screen_mode,
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
