//! Moved to `sm_game::menus::map_menu`.
pub(crate) use sm_game::menus::map_menu::*;

// The menus address these enemy record fields by offset.
const _: () = assert!(core::mem::offset_of!(crate::types::EnemyData, x_pos) == 2);
const _: () = assert!(core::mem::offset_of!(crate::types::EnemyData, palette_index) == 30);
