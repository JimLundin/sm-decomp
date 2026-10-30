//! Common gameplay names from shared owners; bank-specific names stay local.

pub(crate) use crate::c_library::printf;
pub(crate) use crate::game::callback_tables::callback_table;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::enemy_properties::*;
pub use crate::game::constants::game_states::*;
pub use crate::game::constants::loading_game_states::*;
pub use crate::game::constants::projectile_types::*;
pub use crate::game::constants::samus_movement::*;
pub use crate::game::constants::samus_poses::*;
pub(crate) use crate::game_state::Game;
pub(crate) use crate::ram_map::word;
pub(crate) use crate::rtl::{
    MakePairU16, Mult8x8, ReadRegWord, SnesDivide, Unreachable, WriteReg, abs16, words,
};
pub use crate::snes::registers::*;
pub use crate::types::{
    Buttons, CheckEnemyColl_Result, CheckSolidEnemyColl_Result, CoroutineRet, DoorDef,
    Enemy_MotherBrain, EnemyBase, EnemyData, EnemyDef, EnemySpawnData, EnemyTileset, EprojDef,
    ExpandingSquareTransitionHdma, ExtendedSpriteMap, ExtraEnemyRam7800, ExtraEnemyRam8000,
    Func_U8, Func_V, Func_Y_V, Func_Y_Y, Hitbox, LongPtr, MapIconDataPointers, MapScrollArrowData,
    OamEnt, Pair_Bool_Amt, PairU16, Point16U, Ram3000, Ram3800, Ram4000, Ram8000_Default, Rect16U,
    RoomDefHeader, RoomDefRoomstate, SamusPose, SamusPoseParams, SnesRegs, SpawnHardcodedPlmArgs,
    SpawnHdmaObject_Args, TileTable, TileTables, VoidP, VramWriteEntry, Vulnerability, size_t,
};
pub(crate) use ::core::ffi::{c_char, c_long};
