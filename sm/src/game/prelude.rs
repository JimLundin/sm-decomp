//! Common gameplay names from shared owners; bank-specific names stay local.

pub(crate) use crate::c_library::{__assert_fail, memcpy, memset, printf};
pub(crate) use crate::game::callback_tables::callback_table;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::enemy_properties::*;
pub use crate::game::constants::game_states::*;
pub use crate::game::constants::loading_game_states::*;
pub use crate::game::constants::projectile_types::*;
pub use crate::game::constants::samus_movement::*;
pub use crate::game::constants::samus_poses::*;
pub(crate) use crate::game_state::Game;
pub(crate) use crate::ram_map::{ram_ptr, word};
pub(crate) use crate::rtl::{
    abs16, mov24, rom_ptr, words, AddToHiLo, Load24, MakePairU16, Mult8x8, ReadRegWord, RomPtr,
    RtlApuWrite, RtlWriteSram, SnesDivide, SnesModulus, Unreachable, WriteReg, WriteRegWord,
};
pub use crate::snes::registers::*;
pub(crate) use crate::types::TileDrawOrder;
pub use crate::types::{
    size_t, Buttons, CheckEnemyColl_Result, CheckSolidEnemyColl_Result, CoroutineRet, DoorDef,
    EnemyBase, EnemyData, EnemyDef, EnemySpawnData, EnemyTileset, Enemy_MotherBrain, EprojDef,
    ExpandingSquareTransitionHdma, ExtendedSpriteMap, ExtraEnemyRam7800, ExtraEnemyRam8000,
    Func_U8, Func_V, Func_Y_V, Func_Y_Y, Hitbox, LongPtr, MapIconDataPointers, MapScrollArrowData,
    Mode7VramWriteQueue, OamEnt, PairU16, Pair_Bool_Amt, Point16U, Point32, Ram3000, Ram3800,
    Ram4000, Ram8000_Default, Rect16U, RoomDefHeader, RoomDefRoomstate, RoomPlmEntry, SamusPose,
    SamusPoseParams, SnesRegs, SpawnHardcodedPlmArgs, SpawnHdmaObject_Args, StartDmaCopy,
    TileTable, TileTables, VoidP, VramReadQueueEnt, VramWriteEntry, Vulnerability,
};
pub(crate) use ::core::ffi::{c_char, c_long, c_void};
pub(crate) use ::core::mem::transmute;
pub(crate) use ::core::ptr::{null, null_mut};
