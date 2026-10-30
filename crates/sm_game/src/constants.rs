//! Shared gameplay values and bank-local addresses.
//! Bank modules re-export their original definition sets for existing callers.
#![allow(non_upper_case_globals, non_camel_case_types)]
/// Value types for the constant groups below; each is `u32`.
pub type Buttons = u32;
pub type EnemyProps = u32;
pub type GameState = u32;
pub type LoadingGameState = u32;
pub type ProjectileType = u32;
pub type SamusMovementType = u32;
pub type SamusPose = u32;
pub type Consts_60 = u32;
pub type Consts_81 = u32;
pub type Consts_82 = u32;
pub type Consts_83 = u32;
pub type Consts_84 = u32;
pub type Consts_86 = u32;
pub type Consts_87 = u32;
pub type Consts_88 = u32;
pub type Consts_89 = u32;
pub type Consts_8B = u32;
pub type Consts_8C = u32;
pub type Consts_8D = u32;
pub type Consts_8F = u32;
pub type Consts_90 = u32;
pub type Consts_91 = u32;
pub type Consts_94 = u32;
pub type Consts_9B = u32;
pub type Consts_A0 = u32;
pub type Consts_A2 = u32;
pub type Consts_A3 = u32;
pub type Consts_A4 = u32;
pub type Consts_A5 = u32;
pub type Consts_A6 = u32;
pub type Consts_A7 = u32;
pub type Consts_A8 = u32;
pub type Consts_A9 = u32;
pub type Consts_AA = u32;
pub type Consts_AD = u32;
pub type Consts_B2 = u32;
pub type Consts_B3 = u32;
pub type Consts_B4 = u32;
pub type Consts_B7 = u32;

// Declare bank-local numeric names without changing their public type or value.
#[macro_export]
macro_rules! typed_constants {
    ($type:ty; $($name:ident = $value:literal),+ $(,)?) => {
        $(pub const $name: $type = $value;)+
    };
}
pub use crate::typed_constants;

pub mod bank_60 {
    use super::Consts_60;

    crate::constants::typed_constants! { Consts_60;
        addr_unk_604800 = 0x4800, addr_unk_604C00 = 0x4c00,
        addr_unk_605000 = 0x5000, addr_unk_6053E0 = 0x53e0,
        addr_unk_605400 = 0x5400, addr_unk_605800 = 0x5800,
        addr_unk_605820 = 0x5820, addr_unk_605880 = 0x5880,
        addr_unk_6059A0 = 0x59a0, addr_unk_605BE0 = 0x5be0,
        addr_unk_6061F0 = 0x61f0, addr_unk_606300 = 0x6300,
    }
}

pub mod bank_81 {
    use super::Consts_81;
    crate::constants::typed_constants! { Consts_81;
        addr_kAreaSelectBackgroundTilemaps = 48922, addr_kAreaSelectForegroundTilemap = 46874,
        addr_kMenuTilemap_DataCleared = 46810, addr_kMenuTilemap_ClearSamusA = 46746,
        addr_kMenuTilemap_ClearWhichData = 46682, addr_kMenuTilemap_CopyCompleted = 46650,
        addr_kMenuTilemap_No = 46634, addr_kMenuTilemap_Yes = 46618,
        addr_kMenuTilemap_IsThisOk = 46594, addr_kMenuTilemap_CopySamusToSamus = 46536,
        addr_kMenuTilemap_CopySamusToWhere = 46486, addr_kMenuTilemap_CopyWhichData = 46452,
        addr_kMenuTilemap_DataClearMode = 46388, addr_kMenuTilemap_DataCopyMode = 46328,
        addr_kMenuTilemap_Exit = 46318, addr_kMenuTilemap_DataClear = 46296,
        addr_kMenuTilemap_DataCopy = 46276,
        addr_kMenuTilemap_TIME = 46240,
        addr_kMenuTilemap_SamusC = 46198,
        addr_kMenuTilemap_SamusB = 46166, addr_kMenuTilemap_SamusA = 46134,
        addr_kMenuTilemap_SamusData = 46090, addr_kDownMapScrollArrowData = 44880,
        addr_kUpMapScrollArrowData = 44870, addr_kRightMapScrollArrowData = 44860,
        addr_kLeftMapScrollArrowData = 44850, addr_kMenuTilemap_NoGoToTitle = 37792,
        addr_kMenuTilemap_YesReturnToGame = 37708, addr_kMenuTilemap_TryAgain = 37684,
        addr_kMenuTilemap_FindMetroidLarva = 37636, addr_kMenuTilemap_GameOver = 37596,
    }
}

pub mod bank_82 {
    use super::Consts_82;

    crate::constants::typed_constants! { Consts_82;
        addr_kPauseMenuMapData = 0x9717, addr_stru_82B9A0 = 0xb9a0,
        addr_stru_82B9AA = 0xb9aa, addr_stru_82B9B4 = 0xb9b4,
        addr_stru_82B9BE = 0xb9be, addr_kgameOverBabyMetridInstructionList = 0xbc27,
        addr_kEquipmentScreenTilemap_MANUAL = 0xbf22, addr_kEquipmentScreenTilemap_AUTO = 0xbf2a,
        addr_kEquipmentScreenTilemap_Blank = 0xc01a, addr_kEquipmentTilemapOffs_Tanks = 0xc068,
        addr_kEquipmentTilemapOffs_Weapons = 0xc06c, addr_kEquipmentTilemapOffs_Suits = 0xc076,
        addr_kEquipmentTilemapOffs_Boots = 0xc082, addr_kDummySamusWireframeTilemap = 0xc639,
        addr_kMapIconDataPointers = 0xc7cb, addr_off_82F4B6 = 0xf4b6,
        addr_stru_82F4B8 = 0xf4b8, addr_stru_82F4C4 = 0xf4c4,
        addr_stru_82F4CA = 0xf4ca, addr_stru_82F4D0 = 0xf4d0,
        addr_stru_82F4D6 = 0xf4d6,
    }
}

pub mod bank_84 {
    use super::Consts_84;

    crate::constants::typed_constants! { Consts_84;
        addr_PlmPreInstr_Empty4 = 0x8aa6, addr_locret_848AE0 = 0x8ae0,
        addr_kDefaultPlmDrawInstruction = 0x8da0, addr_word_84AD76 = 0xad76,
        addr_kPlmHeader_B6FF = 0xb6ff, addr_locret_84BB6A = 0xbb6a,
        addr_locret_84BBDC = 0xbbdc, addr_kPlmInstrList_C91C = 0xc91c,
        addr_kPlmInstrList_C922 = 0xc922, addr_kPlmHeader_D113 = 0xd113,
        addr_kPlmHeader_D6EA = 0xd6ea, addr_locret_84D779 = 0xd779,
    }
}

pub mod bank_86 {
    use super::Consts_86;

    crate::constants::typed_constants! { Consts_86;
        addr_stru_868BC2 = 0x8bc2, addr_stru_868BD0 = 0x8bd0,
        addr_stru_868BDE = 0x8bde, addr_stru_868BEC = 0x8bec,
        addr_stru_868BFA = 0x8bfa, addr_stru_868C08 = 0x8c08,
        addr_stru_868C16 = 0x8c16, addr_stru_868C24 = 0x8c24,
        addr_stru_868E50 = 0x8e50, addr_stru_868E5E = 0x8e5e,
        addr_word_868EDF = 0x8edf, addr_stru_868F8F = 0x8f8f,
        addr_stru_868F9D = 0x8f9d, addr_kEproj_CrocomireSpikeWallPieces = 0x90c1,
        addr_kRidleysFireball_Tab0 = 0x9408, addr_off_869574 = 0x9574,
        addr_stru_869634 = 0x9634, addr_stru_869650 = 0x9650,
        addr_stru_86966C = 0x966c, addr_stru_86967A = 0x967a,
        addr_stru_869688 = 0x9688, addr_stru_869696 = 0x9696,
        addr_stru_869734 = 0x9734, addr_stru_869742 = 0x9742,
        addr_word_86976C = 0x976c, addr_word_869772 = 0x9772,
        addr_word_869782 = 0x9782, addr_word_8697AC = 0x97ac,
        addr_off_8697B4 = 0x97b4, addr_off_8697F8 = 0x97f8,
        addr_kEproj_DestroyableFireballs = 0x9c29, addr_kEproj_StartingFireballs = 0x9c37,
        addr_kEproj_RocksKraidSpits = 0x9c45, addr_kEproj_RocksFallingKraidCeiling = 0x9c53,
        addr_kEproj_RocksWhenKraidRisesLeft = 0x9c61, addr_kEproj_RocksWhenKraidRisesRight = 0x9c6f,
        addr_kEproj_MiniKraidSpit = 0x9db0, addr_kEproj_MiniKraidSpikesLeft = 0x9dbe,
        addr_kEproj_MiniKraidSpikesRight = 0x9dcc, addr_word_869F41 = 0x9f41,
        addr_word_869F7D = 0x9f7d, addr_off_869FB9 = 0x9fb9,
        addr_off_869FE1 = 0x9fe1, addr_stru_86A17B = 0xa17b,
        addr_kEproj_PirateClaw = 0xa189, addr_off_86A28B = 0xa28b,
        addr_stru_86A379 = 0xa379, addr_kEproj_CeresElevatorPad = 0xa387,
        addr_kEproj_CeresElevatorPlatform = 0xa395, addr_kEproj_PrePhantomRoom = 0xa3b0,
        addr_stru_86A95B = 0xa95b, addr_kEproj_BombTorizoLowHealthInitialDrool = 0xa969,
        addr_kEproj_BombTorizoExplosiveSwipe = 0xa985, addr_kEproj_BombTorizoLowHealthExplode = 0xa9a1,
        addr_kEproj_BombTorizoDeathExplosion = 0xa9af, addr_kEprojInit_AB07_Tile0 = 0xa9bd,
        addr_kEprojInit_AB07_Tile1 = 0xa9fd, addr_off_86AB25 = 0xab25,
        addr_off_86AB41 = 0xab41, addr_kEprojInit_BombTorizosChozoOrbs_init0 = 0xac08,
        addr_kEprojInit_BombTorizosChozoOrbs_init1 = 0xac12, addr_kEprojInit_GoldenTorizosChozoOrbs_init0 = 0xac99,
        addr_kEprojInit_GoldenTorizosChozoOrbs_init1 = 0xaca3, addr_kEproj_BombTorizosChozoOrbs = 0xad5e,
        addr_kEproj_GoldenTorizosChozoOrbs = 0xad7a, addr_off_86ADBF = 0xadbf,
        addr_off_86ADD2 = 0xadd2, addr_off_86ADE5 = 0xade5,
        addr_kEproj_BombTorizoSonicBoom = 0xaea8, addr_kEproj_GoldenTorizoSonicBoom = 0xaeb6,
        addr_kEproj_WreckedShipChozoSpikeFootsteps_1 = 0xaf68, addr_kEproj_TourianStatueDustClouds = 0xaf84,
        addr_kEproj_TourianLandingDustCloudsRightFoot = 0xafe5, addr_stru_86AFF3 = 0xaff3,
        addr_kEprojInit_GoldenTorizoEgg0 = 0xb02f, addr_kEprojInit_GoldenTorizoEgg1 = 0xb039,
        addr_off_86B14B = 0xb14b, addr_off_86B166 = 0xb166,
        addr_off_86B190 = 0xb190, addr_off_86B1A8 = 0xb1a8,
        addr_stru_86B1C0 = 0xb1c0, addr_off_86B2EF = 0xb2ef,
        addr_kEproj_GoldenTorizoSuperMissile = 0xb31a, addr_stru_86B376 = 0xb376,
        addr_stru_86B380 = 0xb380, addr_off_86B3CD = 0xb3cd,
        addr_off_86B3E5 = 0xb3e5, addr_kEproj_GoldenTorizoEyeBeam = 0xb428,
        addr_word_86B4BF = 0xb4bf, addr_word_86B4CB = 0xb4cb,
        addr_word_86B4D7 = 0xb4d7, addr_word_86B4E3 = 0xb4e3,
        addr_kEproj_LavaSeahorseFireball = 0xb5cb, addr_off_86B5F3 = 0xb5f3,
        addr_off_86B61D = 0xb61d, addr_kEproj_EyeDoorProjectile = 0xb743,
        addr_kEproj_EyeDoorSweat = 0xb751, addr_off_86B79F = 0xb79f,
        addr_stru_86BA5C = 0xba5c, addr_kEproj_TourianStatueEyeGlow = 0xba6a,
        addr_kEproj_TourianStatueUnlockingParticle = 0xba78, addr_kEproj_TourianStatueUnlockingParticleTail = 0xba86,
        addr_kEproj_TourianStatueSoul = 0xba94, addr_kEproj_TourianStatueRidley = 0xbaa2,
        addr_kEproj_TourianStatuePhantoon = 0xbab0, addr_kEproj_TourianStatueBaseDecoration = 0xbabe,
        addr_kEproj_NuclearWaffleBody = 0xbbc7, addr_word_86BBD5 = 0xbbd5,
        addr_stru_86BD5A = 0xbd5a, addr_kEproj_ShaktoolAttackFrontCircle = 0xbe25,
        addr_kEproj_ShaktoolAttackMiddleCircle = 0xbe33, addr_kEproj_ShaktoolAttackBackCircle = 0xbe41,
        addr_kEproj_MotherBrainRoomTurrets = 0xc17e, addr_kEproj_MotherBrainRoomTurretBullets = 0xc18c,
        addr_off_86C8E1 = 0xc8e1, addr_kEproj_MotherBrainDeathExplosion = 0xcb13,
        addr_kEproj_MotherBrainExplodedEscapeDoor = 0xcb21, addr_kEproj_MotherBrainPurpleBreathBig = 0xcb2f,
        addr_kEproj_MotherBrainPurpleBreathSmall = 0xcb3d, addr_kEproj_MotherBrainBlueRingLasers = 0xcb4b,
        addr_kEproj_MotherBrainBomb = 0xcb59, addr_kEproj_MotherBrainDeathBeamCharging = 0xcb67,
        addr_kEproj_MotherBrainDeathBeamFired = 0xcb75, addr_kEproj_MotherBrainRainbowBeamCharging = 0xcb83,
        addr_kEproj_MotherBrainDrool = 0xcb91, addr_kEproj_MotherBrainDyingDrool = 0xcb9f,
        addr_kEproj_MotherBrainRainbowBeamExplosion = 0xcbad, addr_kEproj_TimeBombSetJapaneseText = 0xcbbb,
        addr_kEproj_MotherBrainTubeFalling_TopLeft = 0xcc69, addr_kEproj_MotherBrainTubeFalling_TopMiddleLeft = 0xcc77,
        addr_kEproj_MotherBrainTubeFalling_TopMiddleRight = 0xcc85, addr_kEproj_MotherBrainGlassShatteringSparkle = 0xcf0a,
        addr_off_86CF56 = 0xcf56, addr_word_86D03C = 0xd03c,
        addr_word_86D04A = 0xd04a, addr_word_86D052 = 0xd052,
        addr_off_86D218 = 0xd218, addr_kEproj_MaridiaFloatersSpikes = 0xd298,
        addr_kEproj_WreckedShipRobotLaserUpLeft = 0xd2a6, addr_kEproj_WreckedShipRobotLaserHorizontal = 0xd2b4,
        addr_kEproj_WreckedShipRobotLaserDownLeft = 0xd2c2, addr_kEproj_WreckedShipRobotLaserUpRight = 0xd2d0,
        addr_kEproj_WreckedShipRobotLaserDownRight = 0xd2de, addr_kEproj_N00bTubeShards = 0xd912,
        addr_kEproj_N00bTubeReleasedAirBubbles = 0xd920, addr_kEproj_SpikeShootingPlantSpikes = 0xdafe,
        addr_word_86DB0C = 0xdb0c, addr_Eproj_DBF2 = 0xdbf2,
        addr_word_86DC06 = 0xdc06, addr_kEproj_SporeSpawnsStalk = 0xde6c,
        addr_kEproj_Spores = 0xde7a, addr_kEproj_SporeSpawners = 0xde88,
        addr_word_86DE96 = 0xde96, addr_word_86DEA6 = 0xdea6,
        addr_stru_86DFBC = 0xdfbc, addr_stru_86DFCA = 0xdfca,
        addr_word_86DFD8 = 0xdfd8, addr_word_86DFDE = 0xdfde,
        addr_kEproj_LavaThrownByLavaman = 0xe0e0, addr_word_86E208 = 0xe208,
        addr_kEproj_DustCloudExplosion = 0xe509, addr_kEproj_EyeDoorSmoke = 0xe517,
        addr_kEproj_SaveStationElectricity = 0xe6d2, addr_kEproj_BotwoonsBody = 0xeba0,
        addr_word_86EBAE = 0xebae, addr_kEproj_BotwoonsSpit = 0xec48,
        addr_word_86EC56 = 0xec56, addr_word_86EC5C = 0xec5c,
        addr_kEproj_YappingMawsBody = 0xec95, addr_word_86ECA3 = 0xeca3,
        addr_kEproj_Pickup = 0xf337, addr_kEproj_EnemyDeathExplosion = 0xf345,
        addr_kEproj_Sparks = 0xf498,
    }
}

pub mod bank_87 {
    use super::Consts_87;

    crate::constants::typed_constants! { Consts_87;
        addr_kAnimtiles_WreckedShipTradmillRight = 0x8275, addr_kAnimtiles_WreckedShipTradmillLeft = 0x827b,
        addr_kAnimtiles_Lava = 0x82ab, addr_kAnimtiles_Acid = 0x82c9,
        addr_kAnimtiles_Rain = 0x82e7, addr_kAnimtiles_Spores = 0x82fd,
        addr_kAnimtiles_TourianStatue_Phantoon = 0x854c, addr_kAnimtiles_TourianStatue_Ridley = 0x8552,
        addr_kAnimtiles_TourianStatue_Kraid = 0x8558, addr_kAnimtiles_TourianStatue_Draygon = 0x855e,
    }
}

pub mod bank_8d {
    use super::Consts_8D;

    crate::constants::typed_constants! { Consts_8D;
        addr_locret_8DC5E3 = 0xc5e3, addr_off_8DE192 = 0xe192,
        addr_kPalfx_E194 = 0xe194, addr_kPalfx_E198 = 0xe198,
        addr_kPalfx_E19C = 0xe19c, addr_kPalfx_E1A0 = 0xe1a0,
        addr_kPalfx_E1A4 = 0xe1a4, addr_kPalfx_E1A8 = 0xe1a8,
        addr_kPalfx_E1AC = 0xe1ac, addr_kPalfx_E1B0 = 0xe1b0,
        addr_kPalfx_E1B8 = 0xe1b8, addr_kPalfx_E1BC = 0xe1bc,
        addr_stru_8DE1C0 = 0xe1c0, addr_kPalfx_E1C4 = 0xe1c4,
        addr_kPalfx_E1C8 = 0xe1c8, addr_kPalfx_E1CC = 0xe1cc,
        addr_kPalfx_E1D0 = 0xe1d0, addr_kPalfx_E1D4 = 0xe1d4,
        addr_kPalfx_E1D8 = 0xe1d8, addr_kPalfx_E1DC = 0xe1dc,
        addr_kPalfx_E1E0 = 0xe1e0, addr_kPalfx_E1E4 = 0xe1e4,
        addr_kPalfx_E1E8 = 0xe1e8, addr_stru_8DE1F0 = 0xe1f0,
        addr_stru_8DE1F4 = 0xe1f4, addr_stru_8DE1F8 = 0xe1f8,
        addr_stru_8DE1FC = 0xe1fc, addr_kPalfx_E200 = 0xe200,
        addr_off_8DE3E0 = 0xe3e0, addr_off_8DE400 = 0xe400,
        addr_off_8DE420 = 0xe420, addr_kPalfxInstrList_E45E = 0xe45e,
        addr_off_8DE68A = 0xe68a, addr_off_8DE8B6 = 0xe8b6,
        addr_word_8DEB43 = 0xeb43, addr_word_8DEC76 = 0xec76,
        addr_stru_8DF759 = 0xf759, addr_stru_8DF75D = 0xf75d,
        addr_kPalfx_FFC9 = 0xffc9, addr_kPalfx_FFCD = 0xffcd,
        addr_kPalfx_FFD1 = 0xffd1, addr_kPalfx_FFD5 = 0xffd5,
    }
}

pub mod bank_8f {
    use super::Consts_8F;

    crate::constants::typed_constants! { Consts_8F;
        addr_kRoom_96ba = 0x96ba, addr_kRoom_9804 = 0x9804,
        addr_kRoom_a66a = 0xa66a, addr_kRoom_b32e = 0xb32e,
        addr_kRoom_b457 = 0xb457, addr_kRoom_cefb = 0xcefb,
        addr_kRoom_dd58 = 0xdd58, addr_kRoom_dede = 0xdede,
        addr_kDoorClosingPlmIds = 0xe68a,
    }
}

pub mod bank_90 {
    use super::Consts_90;

    crate::constants::typed_constants! { Consts_90;
        addr_loc_908029 = 0x8029, addr_stru_909F25 = 0x9f25,
        addr_stru_909F31 = 0x9f31, addr_stru_909F3D = 0x9f3d,
        addr_kSamusSpeedTable_Normal_X = 0x9f49, addr_kSamusSpeedTable_Water_X = 0xa08d,
        addr_kSamusSpeedTable_LavaAcid_X = 0xa1dd, addr_kSuperMissileAccelerations = 0xc303,
        addr_kSuperMissileAccelerations2 = 0xc32b, addr_loc_90C4F0 = 0xc4f0,
    }
}

pub mod bank_91 {
    use super::Consts_91;

    crate::constants::typed_constants! { Consts_91;
        addr_off_91864B = 0x864b, addr_kDemoInstrs_LeaveDemo = 0x8776,
        addr_stru_91877E = 0x877e, addr_stru_918784 = 0x8784,
        addr_stru_919346 = 0x9346,
    }
}

pub mod bank_a0 {
    use super::Consts_A0;

    crate::constants::typed_constants! { Consts_A0;
        addr_kSpritemap_Nothing_A0 = 0x804d, addr_kExtendedSpritemap_Nothing_A0 = 0x804f,
        addr_kEnemyDef_DAFF = 0xdaff, addr_kEnemyDef_DD7F = 0xdd7f,
        addr_kEnemyDef_DDBF = 0xddbf, addr_kEnemyDef_DE3F = 0xde3f,
        addr_kEnemyDef_DE7F = 0xde7f, addr_kEnemyDef_DF3F = 0xdf3f,
        addr_kEnemyDef_DF7F = 0xdf7f, addr_kEnemyDef_E0FF = 0xe0ff,
        addr_kEnemyDef_E17F = 0xe17f, addr_kEnemyDef_E2BF = 0xe2bf,
        addr_kEnemyDef_E4BF = 0xe4bf, addr_kEnemyDef_E4FF = 0xe4ff,
        addr_kEnemyDef_E7FF = 0xe7ff, addr_kEnemyDef_E83F = 0xe83f,
        addr_kEnemyDef_EC3F = 0xec3f, addr_kEnemyDef_EEFF = 0xeeff,
        addr_kEnemyDef_F293 = 0xf293, addr_kEnemyDef_F353 = 0xf353,
        addr_kEnemyDef_F593 = 0xf593,
    }
}

pub mod bank_a4 {
    use super::Consts_A4;

    crate::constants::typed_constants! { Consts_A4;
        addr_kSpritemap_Nothing_A4 = 0x804d, addr_kExtendedSpritemap_Nothing_A4 = 0x804f,
        addr_kCrocomire_Ilist_BADE = 0xbade, addr_kCrocomire_Ilist_BAEA = 0xbaea,
        addr_kCrocomire_Ilist_BB36 = 0xbb36, addr_kCrocomire_Ilist_BBCA = 0xbbca,
        addr_kCrocomire_Ilist_BBCE = 0xbbce, addr_kCrocomire_Ilist_BC30 = 0xbc30,
        addr_kCrocomire_Ilist_BC34 = 0xbc34, addr_kCrocomire_Ilist_BC56 = 0xbc56,
        addr_kCrocomire_Ilist_BCD8 = 0xbcd8, addr_kCrocomire_Ilist_BD2A = 0xbd2a,
        addr_kCrocomire_Ilist_BD8E = 0xbd8e, addr_kCrocomire_Ilist_BDA2 = 0xbda2,
        addr_kCrocomire_Ilist_BDAE = 0xbdae, addr_kCrocomire_Ilist_BDB2 = 0xbdb2,
        addr_kCrocomire_Ilist_BDB6 = 0xbdb6, addr_kCrocomire_Ilist_BE56 = 0xbe56,
        addr_kCrocomire_Ilist_BE7E = 0xbe7e, addr_kCrocomire_Ilist_BF3C = 0xbf3c,
        addr_kCrocomire_Ilist_BF62 = 0xbf62, addr_kCrocomire_Ilist_BF64 = 0xbf64,
        addr_kCrocomire_Ilist_BF6C = 0xbf6c, addr_kCrocomire_Ilist_BF72 = 0xbf72,
        addr_kCrocomire_Ilist_BF78 = 0xbf78, addr_kCrocomire_Ilist_BF7E = 0xbf7e,
        addr_kCrocomire_Ilist_BF86 = 0xbf86, addr_kCrocomire_Ilist_BF8C = 0xbf8c,
        addr_kCrocomire_Ilist_BF92 = 0xbf92, addr_kCrocomire_Ilist_BF98 = 0xbf98,
        addr_kCrocomire_Ilist_BFB0 = 0xbfb0, addr_kCrocomire_BigSprmap_D51C = 0xd51c,
        addr_kCrocomire_BigSprmap_D600 = 0xd600, addr_kCrocomire_Ilist_E14A = 0xe14a,
        addr_kCrocomire_Ilist_E158 = 0xe158, addr_kCrocomire_Ilist_E1CC = 0xe1cc,
        addr_kCrocomire_Ilist_E1D2 = 0xe1d2,
    }
}

pub mod bank_a6 {
    use super::Consts_A6;

    crate::constants::typed_constants! { Consts_A6;
        addr_kBoulder_Ilist_86A7 = 0x86a7, addr_kBoulder_Ilist_86CB = 0x86cb,
        addr_kSpikeyPlatform_Ilist_8B29 = 0x8b29, addr_kFireGeyser_Ilist_8D1B = 0x8d1b,
        addr_kFireGeyser_Ilist_8DA9 = 0x8da9, addr_kNuclearWaffle_Ilist_9490 = 0x9490,
        addr_stru_A699AE = 0x99ae, addr_stru_A699C6 = 0x99c6,
        addr_kFakeKraid_Ilist_99DC = 0x99dc, addr_stru_A699FC = 0x99fc,
        addr_stru_A69A14 = 0x9a14, addr_kFakeKraid_Ilist_9A2A = 0x9a2a,
        addr_kCeresRidley_Sprmap_A329 = 0xa329, addr_loc_A6A7F9 = 0xa7f9,
        addr_word_A6A9E3 = 0xa9e3, addr_word_A6AA01 = 0xaa01,
        addr_stru_A6AA2F = 0xaa2f, addr_stru_A6AA3F = 0xaa3f,
        addr_off_A6B38C = 0xb38c, addr_off_A6B39C = 0xb39c,
        addr_off_A6B3AC = 0xb3ac, addr_off_A6B3BC = 0xb3bc,
        addr_off_A6B3CC = 0xb3cc, addr_off_A6B3DC = 0xb3dc,
        addr_loc_A6B455 = 0xb455, addr_locret_A6BF19 = 0xbf19,
        addr_kBabyMetroid_Ilist_BF31 = 0xbf31, addr_byte_A6C15D = 0xc15d,
        addr_stru_A6C3B8 = 0xc3b8, addr_stru_A6C4CB = 0xc4cb,
        addr_locret_A6C600 = 0xc600, addr_stru_A6C987 = 0xc987,
        addr_stru_A6C997 = 0xc997, addr_stru_A6C9A7 = 0xc9a7,
        addr_stru_A6C9B7 = 0xc9b7, addr_stru_A6C9C7 = 0xc9c7,
        addr_stru_A6C9D7 = 0xc9d7, addr_stru_A6C9E7 = 0xc9e7,
        addr_stru_A6C9F7 = 0xc9f7, addr_stru_A6CA07 = 0xca07,
        addr_stru_A6CA17 = 0xca17, addr_stru_A6CA27 = 0xca27,
        addr_stru_A6CA37 = 0xca37, addr_kRidleysExplosion_Ilist_CA47 = 0xca47,
        addr_kRidleysExplosion_Ilist_CA4D = 0xca4d, addr_kRidleysExplosion_Ilist_CA53 = 0xca53,
        addr_word_A6D36E = 0xd36e, addr_word_A6D37C = 0xd37c,
        addr_word_A6D38A = 0xd38a, addr_word_A6D398 = 0xd398,
        addr_word_A6D3A6 = 0xd3a6, addr_word_A6DA71 = 0xda71,
        addr_off_A6DAD0 = 0xdad0, addr_off_A6DAD4 = 0xdad4,
        addr_word_A6E16F = 0xe16f, addr_word_A6E1CF = 0xe1cf,
        addr_kRidley_Ilist_E538 = 0xe538, addr_kRidley_Ilist_E548 = 0xe548,
        addr_kRidley_Ilist_E658 = 0xe658, addr_kRidley_Ilist_E690 = 0xe690,
        addr_kRidley_Ilist_E6C8 = 0xe6c8, addr_kRidley_Ilist_E6F0 = 0xe6f0,
        addr_kRidley_Ilist_E706 = 0xe706, addr_kRidley_Ilist_E73A = 0xe73a,
        addr_kRidley_Ilist_E91D = 0xe91d, addr_kCeresDoor_Palette = 0xf4ec,
        addr_word_A6F50C = 0xf50c, addr_kCeresDoor_Sprmap_FAC7 = 0xfac7,
        addr_stru_A6FCE1 = 0xfce1, addr_stru_A6FCF9 = 0xfcf9,
    }
}

pub mod bank_a7 {
    use super::Consts_A7;

    crate::constants::typed_constants! { Consts_A7;
        addr_kDachora_Ilist_F4C1 = 62657, addr_kDachora_Ilist_F4B9 = 62649,
        addr_kDachora_Ilist_F4B3 = 62643, addr_kDachora_Ilist_F48B = 62603,
        addr_kDachora_Ilist_F45B = 62555, addr_kDachora_Ilist_F407 = 62471,
        addr_kDachora_Ilist_F3FF = 62463, addr_kDachora_Ilist_F3F7 = 62455,
        addr_kDachora_Ilist_F3C9 = 62409, addr_kDachora_Ilist_F399 = 62361,
        addr_kDachora_Ilist_F345 = 62277, addr_kDachora_Palette = 61989,
        addr_kEtecoon_Ilist_E8D6 = 59606, addr_kEtecoon_Ilist_E8CE = 59598,
        addr_kEtecoon_Ilist_E8C8 = 59592, addr_kEtecoon_Ilist_E8AC = 59564,
        addr_stru_A7E898 = 59544, addr_kEtecoon_Ilist_E894 = 59540,
        addr_stru_A7E880 = 59520, addr_kEtecoon_Ilist_E876 = 59510,
        addr_kEtecoon_Ilist_E870 = 59504, addr_stru_A7E862 = 59490,
        addr_kEtecoon_Ilist_E854 = 59476, addr_kEtecoon_Ilist_E83C = 59452,
        addr_stru_A7E828 = 59432, addr_kEtecoon_Ilist_E81E = 59422,
        addr_kKraid_Ilist_CC9D = 52381, addr_kKraid_Ilist_CC91 = 52369,
        addr_kKraid_Ilist_CC81 = 52353, addr_kKraid_Ilist_CC53 = 52307,
        addr_kKraid_Ilist_CC4D = 52301, addr_kKraid_Ilist_CC47 = 52295,
        addr_kKraid_Ilist_CC41 = 52289, addr_locret_A7B831 = 47153,
        addr_kKraidsRoomBg = 42774, addr_kKraid_Sprmap_A5DF = 42463,
        addr_kKraidTilemaps_3 = 41160, addr_kKraidTilemaps_2 = 40392,
        addr_kKraidTilemaps_1 = 39624, addr_kKraidTilemaps_0 = 38856,
        addr_stru_A79764 = 38756, addr_stru_A7974A = 38730,
        addr_stru_A796DA = 38618, addr_kKraid_Sprmap_8C6C = 35948,
        addr_kKraid_Ilist_8B0A = 35594, addr_kKraid_Ilist_8B04 = 35588,
        addr_kKraid_Ilist_8AFE = 35582, addr_kKraid_Ilist_8AF0 = 35568,
        addr_kKraid_Ilist_8AA4 = 35492, addr_kKraid_Ilist_8A41 = 35393,
        addr_kKraid_Ilist_89F3 = 35315, addr_kKraid_Ilist_8887 = 34951,
        addr_off_A78885 = 34949, addr_kKraid_Ilist_87BD = 34749,
        addr_off_A787BB = 34747, addr_kKraid_Ilist_86F3 = 34547,
        addr_kKraid_Ilist_86ED = 34541, addr_kKraid_Ilist_86E7 = 34535,
        addr_kSpritemap_Nothing_A7 = 32845,
    }

    crate::constants::typed_constants! { i32;
        fnnullsub_170_A7 = 0xa7804c,
        fnUnpauseHook_Kraid_IsDead = 0xa7c1fb, fnUnpauseHook_Kraid_IsAlive = 0xa7c24e,
        fnKraid_UnpauseHook_IsSinking = 0xa7c2a0, fnPauseHook_Kraid = 0xa7c325,
        fnnullsub_237 = 0xa7d4a8, fnPhantoon_Spawn8FireballsInCircleAtStart = 0xa7d4a9,
        fnPhantoon_WaitBetweenSpawningAndSpinningFireballs = 0xa7d4ee, fnPhantoon_SpawnFireballsBeforeFight = 0xa7d508,
        fnPhantoon_WavyFadeIn = 0xa7d54a, fnPhantoon_PickPatternForRound1 = 0xa7d596,
        fnPhantoon_MovePhantoonInFigure8ThenOpenEye = 0xa7d5e7, fnPhantoon_EyeFollowsSamusUntilTimerRunsOut = 0xa7d60d,
        fnPhantoon_BecomesSolidAndBodyVuln = 0xa7d65c, fnPhantoon_IsSwooping = 0xa7d678,
        fnPhantoon_FadeoutWithSwoop = 0xa7d6b9, fnPhantoon_WaitAfterFadeOut = 0xa7d6d4,
        fnPhantoon_MoveLeftOrRightAndPickEyeOpenPatt = 0xa7d6e2, fnPhantoon_FadeInBeforeFigure8 = 0xa7d72d,
        fnPhantoon_BecomeSolidAfterRainingFireballs = 0xa7d73f, fnPhantoon_FadeInDuringFireballRain = 0xa7d767,
        fnPhantoon_FollowSamusWithEyeDuringFireballRain = 0xa7d788, fnPhantoon_FadeOutDuringFireballRain = 0xa7d7d5,
        fnPhantoon_SpawnRainingFireballs = 0xa7d7f7, fnPhantoon_FadeOutBeforeFirstFireballRain = 0xa7d82a,
        fnPhantoon_FadeOutBeforeEnrage = 0xa7d85c, fnPhantoon_MoveEnragedPhantoonToTopCenter = 0xa7d874,
        fnPhantoon_FadeInEnragedPhantoon = 0xa7d891, fnPhantoon_Enraged = 0xa7d8ac,
        fnPhantoon_FadeoutAfterEnrage = 0xa7d916, fnPhantoon_CompleteSwoopAfterFatalShot = 0xa7d92e,
        fnPhantoon_DyingPhantoonFadeInOut = 0xa7d948, fnPhantoon_DyingPhantoonExplosions = 0xa7d98b,
        fnPhantoon_WavyDyingPhantoonAndCry = 0xa7da51, fnPhantoon_DyingFadeOut = 0xa7da86,
        fnPhantoon_AlmostDead = 0xa7dad7, fnPhantoon_Dead = 0xa7db3d,
        fnEtecoon_Func_4 = 0xa7e9af, fnEtecoon_Func_5 = 0xa7ea00,
        fnEtecoon_Func_6 = 0xa7ea37, fnEtecoon_Func_7 = 0xa7eab5,
        fnEtecoon_Func_8 = 0xa7eb02, fnEtecoon_Func_9 = 0xa7eb2c,
        fnEtecoon_Func_10 = 0xa7eb50, fnEtecoon_Func_11 = 0xa7ebcd,
        fnEtecoon_Func_12 = 0xa7ec1b, fnEtecoon_Func_16 = 0xa7ec97,
        fnEtecoon_Func_17 = 0xa7ecbb, fnEtecoon_Func_18 = 0xa7ecdf,
        fnEtecoon_Func_19 = 0xa7ed09, fnEtecoon_Func_20 = 0xa7ed2a,
        fnEtecoon_Func_21 = 0xa7ed54, fnEtecoon_Func_22 = 0xa7ed75,
        fnEtecoon_Func_23 = 0xa7edc7, fnEtecoon_Func_24 = 0xa7ee3e,
        fnEtecoon_Func_25 = 0xa7ee9a, fnEtecoon_Func_26 = 0xa7eeb8,
        fnDachora_Func_2 = 0xa7f570, fnDachora_Func_3 = 0xa7f5bc,
        fnDachora_Func_4 = 0xa7f5ed, fnDachora_Func_5 = 0xa7f65e,
        fnDachora_Func_7 = 0xa7f78f, fnDachora_Func_8 = 0xa7f806,
        fnDachora_Func_11 = 0xa7f935, fnDachora_Func_12 = 0xa7f98c,
    }
}

pub mod bank_a9 {
    use super::Consts_A9;

    crate::constants::typed_constants! { Consts_A9;
        addr_locret_A98AE4 = 0x8ae4, addr_stru_A98AE5 = 0x8ae5,
        addr_stru_A98AF5 = 0x8af5, addr_stru_A98B05 = 0x8b05,
        addr_stru_A98B25 = 0x8b25, addr_stru_A98F8F = 0x8f8f,
        addr_stru_A98FC7 = 0x8fc7, addr_stru_A98FE5 = 0x8fe5,
        addr_stru_A99003 = 0x9003, addr_stru_A9902F = 0x902f,
        addr_kMotherBrainsBrain_Palette = 0x9472, addr_kMotherBrainPalette_0 = 0x9492,
        addr_kMotherBrainPalette_1 = 0x94b2, addr_kMotherBrainPalette_2 = 0x94d2,
        addr_kMotherBrainPalette_3 = 0x94f2, addr_kMotherBrainPalette_4 = 0x9512,
        addr_kMotherBrainPalette_5 = 0x9532, addr_kMotherBrain_Ilist_97A4 = 0x97a4,
        addr_kMotherBrain_Ilist_98C6 = 0x98c6, addr_kMotherBrain_Ilist_99AA = 0x99aa,
        addr_kMotherBrain_Ilist_99C6 = 0x99c6, addr_kMotherBrain_Ilist_99E2 = 0x99e2,
        addr_kMotherBrain_Ilist_99F2 = 0x99f2, addr_kMotherBrain_Ilist_9A02 = 0x9a02,
        addr_kMotherBrain_Ilist_9A0A = 0x9a0a, addr_kMotherBrain_Ilist_9A26 = 0x9a26,
        addr_kMotherBrain_Ilist_9A42 = 0x9a42, addr_kMotherBrain_Ilist_9B7F = 0x9b7f,
        addr_kMotherBrain_Ilist_9BB3 = 0x9bb3, addr_kMotherBrain_Ilist_9BE7 = 0x9be7,
        addr_kMotherBrain_Ilist_9C13 = 0x9c13, addr_stru_A99C21 = 0x9c21,
        addr_kMotherBrain_Ilist_9C29 = 0x9c29, addr_kMotherBrain_Ilist_9C39 = 0x9c39,
        addr_stru_A99C47 = 0x9c47, addr_stru_A99C5F = 0x9c5f,
        addr_kMotherBrain_Ilist_9C77 = 0x9c77, addr_kMotherBrain_Ilist_9C87 = 0x9c87,
        addr_stru_A99C9F = 0x9c9f, addr_stru_A99CD1 = 0x9cd1,
        addr_kMotherBrain_Ilist_9D25 = 0x9d25, addr_kMotherBrain_Ilist_9D3D = 0x9d3d,
        addr_kMotherBrain_Ilist_9D7F = 0x9d7f, addr_kMotherBrain_Ilist_9DB1 = 0x9db1,
        addr_kMotherBrain_Ilist_9DBB = 0x9dbb, addr_kMotherBrain_Ilist_9ECC = 0x9ecc,
        addr_kMotherBrain_Ilist_9F00 = 0x9f00, addr_kMotherBrain_Ilist_9F34 = 0x9f34,
        addr_kMotherBrain_Ilist_9F6C = 0x9f6c, addr_kMotherBrain_Sprmap_A694 = 0xa694,
        addr_word_A9B109 = 0xb109, addr_word_A9B10F = 0xb10f,
        addr_word_A9B427 = 0xb427, addr_byte_A9B6DC = 0xb6dc,
        addr_byte_A9B6DF = 0xb6df, addr_stru_A9BE28 = 0xbe28,
        addr_locret_A9C18D = 0xc18d, addr_locret_A9C353 = 0xc353,
        addr_loc_A9C49C = 0xc49c, addr_loc_A9C959 = 0xc959,
        addr_stru_A9CA24 = 0xca24, addr_kBabyMetroid_Ilist_CFA2 = 0xcfa2,
        addr_kBabyMetroid_Ilist_CFB8 = 0xcfb8, addr_kBabyMetroid_Ilist_CFCE = 0xcfce,
        addr_stru_A9D046 = 0xd046, addr_word_A9D082 = 0xd082,
        addr_kDeadTorizo_Ilist_D6DC = 0xd6dc, addr_kDeadTorizo_Sprmap_D761 = 0xd761,
        addr_kDeadTorizo_Hitbox_D77C = 0xd77c, addr_stru_A9DD58 = 0xdd58,
        addr_stru_A9DD68 = 0xdd68, addr_stru_A9DD78 = 0xdd78,
        addr_stru_A9DE08 = 0xde08, addr_kDeadMonsters_Ilist_ECAC = 0xecac,
        addr_kDeadMonsters_Ilist_ECE3 = 0xece3, addr_kDeadMonsters_Ilist_ECE9 = 0xece9,
        addr_kDeadMonsters_Ilist_ECEF = 0xecef, addr_word_A9F6D1 = 0xf6d1,
        addr_word_A9F711 = 0xf711, addr_kDeadSidehopper_Palette_0 = 0xf8a6,
        addr_word_A9F8C6 = 0xf8c6, addr_kBabyMetroidInCutscene_Palette = 0xf8e6,
        addr_kBabyMetroid_Ilist_F906 = 0xf906, addr_kBabyMetroid_Ilist_F90E = 0xf90e,
        addr_kBabyMetroid_Ilist_F924 = 0xf924, addr_kBabyMetroid_Ilist_F93A = 0xf93a,
    }
}

pub mod bank_aa {
    use super::Consts_AA;

    crate::constants::typed_constants! { Consts_AA;
        addr_kShaktool_Ilist_E461 = 58465, addr_kShaktool_Ilist_E3A7 = 58279,
        addr_kTorizo_Ilist_D2BF = 53951, addr_kTorizo_Ilist_D2AD = 53933,
        addr_kTorizo_Ilist_D203 = 53763, addr_kTorizo_Ilist_D1F1 = 53745,
        addr_kTorizo_Ilist_CEFF = 52991, addr_kTorizo_Ilist_CEA5 = 52901,
        addr_kTorizo_Ilist_CE43 = 52803, addr_kTorizo_Ilist_CDE1 = 52705,
        addr_locret_AAC95E = 51550, addr_kTorizo_Ilist_C188 = 49544,
        addr_off_AAC0F2 = 49394, addr_kTorizo_Ilist_BDD8 = 48600,
        addr_kTorizo_Ilist_BD0E = 48398, addr_off_AABC78 = 48248,
        addr_kTorizo_Ilist_B962 = 47458, addr_kTorizo_Ilist_B1C8 = 45512,
        addr_kTorizo_Ilist_B155 = 45397, addr_kTorizo_Ilist_B0E5 = 45285,
        addr_kTorizo_ExtSprmap_87D0 = 34768, addr_kSpritemap_Nothing_AA = 32845,
    }

    crate::constants::typed_constants! { i32;
        fnnullsub_171_AA = 0xaa807b,
        fnnullsub_274 = 0xaadcaa, fnnullsub_276 = 0xaae7a6,
    }
}

pub mod bank_b4 {
    use super::Consts_B4;

    crate::constants::typed_constants! { Consts_B4;
        addr_kSpriteObject_Ilist_C3BA = 0xc3ba, addr_kSpriteObject_Ilist_C4B6 = 0xc4b6,
        addr_asc_B4DD89 = 0xdd89, addr_stru_B4EC1C = 0xec1c,
    }
}

pub mod buttons {
    use super::Buttons;

    crate::constants::typed_constants! { Buttons;
        kButton_R = 0x0010, kButton_L = 0x0020,
        kButton_X = 0x0040, kButton_A = 0x0080,
        kButton_Right = 0x0100, kButton_Left = 0x0200,
        kButton_Down = 0x0400, kButton_Up = 0x0800,
        kButton_Start = 0x1000, kButton_Select = 0x2000,
        kButton_Y = 0x4000, kButton_B = 0x8000,
    }
}

pub mod enemy_properties {
    use super::EnemyProps;

    crate::constants::typed_constants! { EnemyProps;
        kEnemyProps_Invisible = 0x0100, kEnemyProps_Deleted = 0x0200,
        kEnemyProps_Tangible = 0x0400, kEnemyProps_ProcessedOffscreen = 0x0800,
        kEnemyProps_BlockPlasmaBeam = 0x1000, kEnemyProps_DisableSamusColl = 0x2000,
        kEnemyProps_RespawnIfKilled = 0x4000,
    }
}

pub mod game_states {
    use super::GameState;

    crate::constants::typed_constants! { GameState;
        kGameState_0_Reset = 0, kGameState_1_OpeningCinematic = 1,
        kGameState_2_GameOptionsMenu = 2, kGameState_3_Unused = 3,
        kGameState_4_FileSelectMenus = 4, kGameState_5_FileSelectMap = 5,
        kGameState_6_LoadingGameData = 6, kGameState_7_MainGameplayFadeIn = 7,
        kGameState_8_MainGameplay = 8, kGameState_9_HitDoorBlock = 9,
        kGameState_10_LoadingNextRoom = 10, kGameState_11_LoadingNextRoom = 11,
        kGameState_12_Pausing = 12, kGameState_13_Pausing = 13,
        kGameState_14_Paused = 14, kGameState_15_Paused = 15,
        kGameState_16_Unpausing = 16, kGameState_17_Unpausing = 17,
        kGameState_18_Unpausing = 18, kGameState_19_SamusNoHealth = 19,
        kGameState_20_SamusNoHealth = 20, kGameState_21_SamusNoHealth = 21,
        kGameState_22_SamusNoHealth = 22, kGameState_23_SamusNoHealth = 23,
        kGameState_24_SamusNoHealth = 24, kGameState_25_SamusNoHealth = 25,
        kGameState_26_GameOverMenu = 26, kGameState_27_ReserveTanksAuto = 27,
        kGameState_28_Unused = 28, kGameState_29_DebugGameOverMenu = 29,
        kGameState_30_IntroCinematic = 30, kGameState_31_SetUpNewGame = 31,
        kGameState_32_MadeItToCeresElevator = 32, kGameState_33_BlackoutFromCeres = 33,
        kGameState_34_CeresGoesBoom = 34, kGameState_35_TimeUp = 35,
        kGameState_36_WhitingOutFromTimeUp = 36, kGameState_37_CeresGoesBoomWithSamus = 37,
        kGameState_38_SamusEscapesFromZebes = 38, kGameState_39_EndingAndCredits = 39,
        kGameState_40_TransitionToDemo = 40, kGameState_41_TransitionToDemo = 41,
        kGameState_42_PlayingDemo = 42, kGameState_43_TransitionFromDemo = 43,
        kGameState_44_TransitionFromDemo = 44,
    }
}

pub mod loading_game_states {
    use super::LoadingGameState;

    crate::constants::typed_constants! { LoadingGameState;
        kLoadingGameState_0_Intro = 0, kLoadingGameState_5_Main = 5,
        kLoadingGameState_1F_StartingAtCeres = 31, kLoadingGameState_22_EscapingCeres = 34,
    }
}

pub mod projectile_types {
    use super::ProjectileType;

    crate::constants::typed_constants! { ProjectileType;
        kProjectileType_Missile = 0x0100, kProjectileType_SuperMissile = 0x0200,
        kProjectileType_PowerBomb = 0x0300, kProjectileType_Bomb = 0x0500,
        kProjectileType_BeamExplosion = 0x0700, kProjectileType_MissileExplosion = 0x0800,
        kProjectileType_TypeMask = 0x0f00, kProjectileType_DontInteractWithSamus = 0x8000,
    }
}

pub mod samus_movement {
    use super::SamusMovementType;

    crate::constants::typed_constants! { SamusMovementType;
        kMovementType_00_Standing = 0x0000, kMovementType_01_Running = 0x0001,
        kMovementType_02_NormalJumping = 0x0002, kMovementType_03_SpinJumping = 0x0003,
        kMovementType_04_MorphBallOnGround = 0x0004, kMovementType_05_Crouching = 0x0005,
        kMovementType_06_Falling = 0x0006, kMovementType_07_Unused = 0x0007,
        kMovementType_08_MorphBallFalling = 0x0008, kMovementType_09_Unused = 0x0009,
        kMovementType_0A_KnockbackOrCrystalFlashEnding = 0x000a, kMovementType_0B_Unused = 0x000b,
        kMovementType_0C_Unused = 0x000c, kMovementType_0D_Unused = 0x000d,
        kMovementType_0E_TurningAroundOnGround = 0x000e, kMovementType_0F_CrouchingEtcTransition = 0x000f,
        kMovementType_10_Moonwalking = 0x0010, kMovementType_11_SpringBallOnGround = 0x0011,
        kMovementType_12_SpringBallInAir = 0x0012, kMovementType_13_SpringBallFalling = 0x0013,
        kMovementType_14_WallJumping = 0x0014, kMovementType_15_RanIntoWall = 0x0015,
        kMovementType_16_Grappling = 0x0016, kMovementType_17_TurningAroundJumping = 0x0017,
        kMovementType_18_TurningAroundFalling = 0x0018, kMovementType_19_DamageBoost = 0x0019,
        kMovementType_1A_GrabbedByDraygon = 0x001a, kMovementType_1B_ShinesparkEtc = 0x001b,
    }
}

pub mod samus_poses {
    use super::SamusPose;

    crate::constants::typed_constants! { SamusPose;
        kPose_00_FaceF_Powersuit = 0x0000, kPose_01_FaceR_Normal = 0x0001,
        kPose_02_FaceL_Normal = 0x0002, kPose_03_FaceR_AimU = 0x0003,
        kPose_04_FaceL_AimU = 0x0004, kPose_05_FaceR_AimUR = 0x0005,
        kPose_06_FaceL_AimUL = 0x0006, kPose_07_FaceR_AimDR = 0x0007,
        kPose_08_FaceL_AimDL = 0x0008, kPose_09_MoveR_NoAim = 0x0009,
        kPose_0A_MoveL_NoAim = 0x000a, kPose_0B_MoveR_Gun = 0x000b,
        kPose_0C_MoveL_Gun = 0x000c, kPose_0F_MoveR_AimUR = 0x000f,
        kPose_10_MoveL_AimUL = 0x0010, kPose_11_MoveR_AimDR = 0x0011,
        kPose_12_MoveL_AimDL = 0x0012, kPose_13_FaceR_Jump_NoAim_NoMove_Gun = 0x0013,
        kPose_14_FaceL_Jump_NoAim_NoMove_Gun = 0x0014, kPose_15_FaceR_Jump_AimU = 0x0015,
        kPose_16_FaceL_Jump_AimU = 0x0016, kPose_17_FaceR_Jump_AimD = 0x0017,
        kPose_18_FaceL_Jump_AimD = 0x0018, kPose_19_FaceR_SpinJump = 0x0019,
        kPose_1A_FaceL_SpinJump = 0x001a, kPose_1B_FaceR_SpaceJump = 0x001b,
        kPose_1C_FaceL_SpaceJump = 0x001c, kPose_1D_FaceR_Morphball_Ground = 0x001d,
        kPose_1E_MoveR_Morphball_Ground = 0x001e, kPose_1F_MoveL_Morphball_Ground = 0x001f,
        kPose_25_FaceR_Turn_Stand = 0x0025, kPose_26_FaceL_Turn_Stand = 0x0026,
        kPose_27_FaceR_Crouch = 0x0027, kPose_28_FaceL_Crouch = 0x0028,
        kPose_29_FaceR_Fall = 0x0029, kPose_2A_FaceL_Fall = 0x002a,
        kPose_2B_FaceR_Fall_AimU = 0x002b, kPose_2C_FaceL_Fall_AimU = 0x002c,
        kPose_2D_FaceR_Fall_AimD = 0x002d, kPose_2E_FaceL_Fall_AimD = 0x002e,
        kPose_2F_FaceR_Turn_Jump = 0x002f, kPose_30_FaceL_Turn_Jump = 0x0030,
        kPose_31_FaceR_Morphball_Air = 0x0031, kPose_32_FaceL_Morphball_Air = 0x0032,
        kPose_35_FaceR_CrouchTrans = 0x0035, kPose_36_FaceL_CrouchTrans = 0x0036,
        kPose_37_FaceR_MorphTrans = 0x0037, kPose_38_FaceL_MorphTrans = 0x0038,
        kPose_3B_FaceR_StandTrans = 0x003b, kPose_3C_FaceL_StandTrans = 0x003c,
        kPose_3D_FaceR_UnmorphTrans = 0x003d, kPose_3E_FaceL_UnmorphTrans = 0x003e,
        kPose_41_FaceL_Morphball_Ground = 0x0041, kPose_43_FaceR_Turn_Crouch = 0x0043,
        kPose_44_FaceL_Turn_Crouch = 0x0044, kPose_49_FaceL_Moonwalk = 0x0049,
        kPose_4A_FaceR_Moonwalk = 0x004a, kPose_4B_FaceR_Jumptrans = 0x004b,
        kPose_4C_FaceL_Jumptrans = 0x004c, kPose_4D_FaceR_Jump_NoAim_NoMove_NoGun = 0x004d,
        kPose_4E_FaceL_Jump_NoAim_NoMove_NoGun = 0x004e, kPose_4F_FaceL_Dmgboost = 0x004f,
        kPose_50_FaceR_Dmgboost = 0x0050, kPose_51_FaceR_Jump_NoAim_MoveF = 0x0051,
        kPose_52_FaceL_Jump_NoAim_MoveF = 0x0052, kPose_53_FaceR_Knockback = 0x0053,
        kPose_54_FaceL_Knockback = 0x0054, kPose_55_FaceR_Jumptrans_AimU = 0x0055,
        kPose_56_FaceL_Jumptrans_AimU = 0x0056, kPose_57_FaceR_Jumptrans_AimUR = 0x0057,
        kPose_58_FaceL_Jumptrans_AimUL = 0x0058, kPose_59_FaceR_Jumptrans_AimDR = 0x0059,
        kPose_5A_FaceL_Jumptrans_AimDL = 0x005a, kPose_5B = 0x005b,
        kPose_67_FaceR_Fall_Gun = 0x0067, kPose_68_FaceL_Fall_Gun = 0x0068,
        kPose_69_FaceR_Jump_AimUR = 0x0069, kPose_6A_FaceL_Jump_AimUL = 0x006a,
        kPose_6B_FaceR_Jump_AimDR = 0x006b, kPose_6C_FaceL_Jump_AimDL = 0x006c,
        kPose_6D_FaceR_Fall_AimUR = 0x006d, kPose_6E_FaceL_Fall_AimUL = 0x006e,
        kPose_6F_FaceR_Fall_AimDR = 0x006f, kPose_70_FaceL_Fall_AimDL = 0x0070,
        kPose_71_FaceR_Crouch_AimUR = 0x0071, kPose_72_FaceL_Crouch_AimUL = 0x0072,
        kPose_73_FaceR_Crouch_AimDR = 0x0073, kPose_74_FaceL_Crouch_AimDL = 0x0074,
        kPose_75_FaceL_Moonwalk_AimUL = 0x0075, kPose_76_FaceR_Moonwalk_AimUR = 0x0076,
        kPose_77_FaceL_Moonwalk_AimDL = 0x0077, kPose_78_FaceR_Moonwalk_AimDR = 0x0078,
        kPose_79_FaceR_Springball_Ground = 0x0079, kPose_7A_FaceL_Springball_Ground = 0x007a,
        kPose_7B_MoveR_Springball_Ground = 0x007b, kPose_7C_MoveL_Springball_Ground = 0x007c,
        kPose_7D_FaceR_Springball_Fall = 0x007d, kPose_7E_FaceL_Springball_Fall = 0x007e,
        kPose_7F_FaceR_Springball_Air = 0x007f, kPose_80_FaceL_Springball_Air = 0x0080,
        kPose_81_FaceR_Screwattack = 0x0081, kPose_82_FaceL_Screwattack = 0x0082,
        kPose_83_FaceR_Walljump = 0x0083, kPose_84_FaceL_Walljump = 0x0084,
        kPose_85_FaceR_Crouch_AimU = 0x0085, kPose_86_FaceL_Crouch_AimU = 0x0086,
        kPose_87_FaceR_Turn_Fall = 0x0087, kPose_88_FaceL_Turn_Fall = 0x0088,
        kPose_89_FaceR_Ranintowall = 0x0089, kPose_8A_FaceL_Ranintowall = 0x008a,
        kPose_8B_FaceR_Turn_Stand_AimU = 0x008b, kPose_8C_FaceL_Turn_Stand_AimU = 0x008c,
        kPose_8D_FaceR_Turn_Stand_AimDR = 0x008d, kPose_8E_FaceL_Turn_Stand_AimDL = 0x008e,
        kPose_8F_FaceR_Turn_Air_AimU = 0x008f, kPose_90_FaceL_Turn_Air_AimU = 0x0090,
        kPose_91_FaceR_Turn_Air_AimDDR = 0x0091, kPose_92_FaceL_Turn_Air_AimDDL = 0x0092,
        kPose_93_FaceR_Turn_Fall_AimU = 0x0093, kPose_94_FaceL_Turn_Fall_AimU = 0x0094,
        kPose_95_FaceR_Turn_Fall_AimDDR = 0x0095, kPose_96_FaceL_Turn_Fall_AimDDL = 0x0096,
        kPose_97_FaceR_Turn_Crouch_AimU = 0x0097, kPose_98_FaceL_Turn_Crouch_AimU = 0x0098,
        kPose_99_FaceR_Turn_Crouch_AimDDR = 0x0099, kPose_9A_FaceL_Turn_Crouch_AimDDL = 0x009a,
        kPose_9B_FaceF_VariaGravitySuit = 0x009b, kPose_9C_FaceR_Turn_Stand_AimUR = 0x009c,
        kPose_9D_FaceL_Turn_Stand_AimUL = 0x009d, kPose_9E_FaceR_Turn_Air_AimUR = 0x009e,
        kPose_9F_FaceL_Turn_Air_AimUL = 0x009f, kPose_A0_FaceR_Turn_Fall_AimUR = 0x00a0,
        kPose_A1_FaceL_Turn_Fall_AimUL = 0x00a1, kPose_A2_FaceR_Turn_Crouch_AimUR = 0x00a2,
        kPose_A3_FaceL_Turn_Crouch_AimUL = 0x00a3, kPose_A4_FaceR_LandJump = 0x00a4,
        kPose_A5_FaceL_LandJump = 0x00a5, kPose_A6_FaceR_LandSpinJump = 0x00a6,
        kPose_A7_FaceL_LandSpinJump = 0x00a7, kPose_A8_FaceR_Grappling = 0x00a8,
        kPose_A9_FaceL_Grappling = 0x00a9, kPose_AA_FaceR_Grappling_AimDR = 0x00aa,
        kPose_AB_FaceL_Grappling_AimDL = 0x00ab, kPose_B2_FaceR_Grapple_Air = 0x00b2,
        kPose_B3_FaceL_Grapple_Air = 0x00b3, kPose_B4_FaceR_Grappling_Crouch = 0x00b4,
        kPose_B5_FaceL_Grappling_Crouch = 0x00b5, kPose_B6_FaceR_Grappling_Crouch_AimDR = 0x00b6,
        kPose_B7_FaceL_Grappling_Crouch_AimDL = 0x00b7, kPose_B8_FaceL_GrappleWalljumpPose = 0x00b8,
        kPose_B9_FaceR_GrappleWalljumpPose = 0x00b9, kPose_BA_FaceL_Draygon_NoMove_NoAim = 0x00ba,
        kPose_BB_FaceL_Draygon_NoMove_AimUL = 0x00bb, kPose_BC_FaceL_Draygon_Fire = 0x00bc,
        kPose_BD_FaceL_Draygon_NoMove_AimDL = 0x00bd, kPose_BE_FaceL_Draygon_Move = 0x00be,
        kPose_BF_FaceR_Moonwalk_TurnjumpL = 0x00bf, kPose_C0_FaceL_Moonwalk_TurnjumpR = 0x00c0,
        kPose_C1_FaceR_Moonwalk_TurnjumpL_AimUR = 0x00c1, kPose_C2_FaceL_Moonwalk_TurnjumpR_AimUL = 0x00c2,
        kPose_C3_FaceR_Moonwalk_TurnjumpL_AimDR = 0x00c3, kPose_C4_FaceL_Moonwalk_TurnjumpR_AimDL = 0x00c4,
        kPose_C7_FaceR_ShinesparkWindup_Vert = 0x00c7, kPose_C8_FaceL_ShinesparkWindup_Vert = 0x00c8,
        kPose_C9_FaceR_Shinespark_Horiz = 0x00c9, kPose_CA_FaceL_Shinespark_Horiz = 0x00ca,
        kPose_CB_FaceR_Shinespark_Vert = 0x00cb, kPose_CC_FaceL_Shinespark_Vert = 0x00cc,
        kPose_CD_FaceR_Shinespark_Diag = 0x00cd, kPose_CE_FaceL_Shinespark_Diag = 0x00ce,
        kPose_CF_FaceR_Ranintowall_AimUR = 0x00cf, kPose_D0_FaceL_Ranintowall_AimUL = 0x00d0,
        kPose_D1_FaceR_Ranintowall_AimDR = 0x00d1, kPose_D2_FaceL_Ranintowall_AimDL = 0x00d2,
        kPose_D3_FaceR_CrystalFlash = 0x00d3, kPose_D4_FaceL_CrystalFlash = 0x00d4,
        kPose_D5_FaceR_Xray_Stand = 0x00d5, kPose_D6_FaceL_Xray_Stand = 0x00d6,
        kPose_D7_FaceR_CrystalFlashEnd = 0x00d7, kPose_D8_FaceL_CrystalFlashEnd = 0x00d8,
        kPose_D9_FaceR_Xray_Crouch = 0x00d9, kPose_DA_FaceL_Xray_Crouch = 0x00da,
        kPose_DB = 0x00db, kPose_E0_FaceR_LandJump_AimU = 0x00e0,
        kPose_E1_FaceL_LandJump_AimU = 0x00e1, kPose_E2_FaceR_LandJump_AimUR = 0x00e2,
        kPose_E3_FaceL_LandJump_AimUL = 0x00e3, kPose_E4_FaceR_LandJump_AimDR = 0x00e4,
        kPose_E5_FaceL_LandJump_AimDL = 0x00e5, kPose_E6_FaceR_LandJump_Fire = 0x00e6,
        kPose_E7_FaceL_LandJump_Fire = 0x00e7, kPose_E8_FaceR_Drained_CrouchFalling = 0x00e8,
        kPose_E9_FaceL_Drained_CrouchFalling = 0x00e9, kPose_EA_FaceR_Drained_Stand = 0x00ea,
        kPose_EB_FaceL_Drained_Stand = 0x00eb, kPose_EC_FaceR_Draygon_NoMove_NoAim = 0x00ec,
        kPose_ED_FaceR_Draygon_NoMove_AimUR = 0x00ed, kPose_EE_FaceR_Draygon_Fire = 0x00ee,
        kPose_EF_FaceR_Draygon_NoMove_AimDR = 0x00ef, kPose_F0_FaceR_Draygon_Move = 0x00f0,
        kPose_F1_FaceR_CrouchTrans_AimU = 0x00f1, kPose_F2_FaceL_CrouchTrans_AimU = 0x00f2,
        kPose_F3_FaceR_CrouchTrans_AimUR = 0x00f3, kPose_F4_FaceL_CrouchTrans_AimUL = 0x00f4,
        kPose_F5_FaceR_CrouchTrans_AimDR = 0x00f5, kPose_F6_FaceL_CrouchTrans_AimDL = 0x00f6,
        kPose_F7_FaceR_StandTrans_AimU = 0x00f7, kPose_F8_FaceL_StandTrans_AimU = 0x00f8,
        kPose_F9_FaceR_StandTrans_AimUR = 0x00f9, kPose_FA_FaceL_StandTrans_AimUL = 0x00fa,
        kPose_FB_FaceR_StandTrans_AimDR = 0x00fb, kPose_FC_FaceL_StandTrans_AimDL = 0x00fc,
    }
}
