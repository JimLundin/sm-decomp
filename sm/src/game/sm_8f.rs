use crate::c_library::strcpy;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_87::*;
pub use crate::game::prelude::*;
use crate::game::progress_flags::SetEventHappened;
use crate::game::scrolling_sky::{
    initialize_land, initialize_ocean, queue_land_rows, queue_ocean_rows,
};
use crate::game::sm_80::{NextRandom, QueueSfx2_Max6};
use crate::game::plm_spawning::SpawnHardcodedPlm;
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_87::SpawnAnimtiles;
use crate::game::sm_88::FxTypeFunc_2C_Haze;
use crate::game::sm_89::RoomCode_CeresElevatorShaft;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_94::Samus_MoveDown_NoSolidColl;
use crate::game::sm_a6::HandleCeresRidleyGetawayCutscene;
use crate::game::sprite_objects::create_sprite;

crate::game::constants::typed_constants! { i32;
    kCoroutineNone = 0,
    fnPauseHook_DraygonRoom = 0x8fc8f6, fnUnpauseHook_DraygonRoom = 0x8fc8fc,
}
const RomPtr_8F: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8f>;
pub unsafe fn RoomCode_GenRandomExplodes_Nonblank(game: &mut Game) {
    if word!(game.time_is_frozen_flag) == 0 && (word!(game.nmi_frame_counter_word) & 1) == 0 {
        let mut Random: u16 = NextRandom(game);
        let x: u16 = (word!(game.layer1_x_pos)).wrapping_add((Random as u8) as u16);
        let y: u16 = (word!(game.layer1_y_pos)).wrapping_add((byte!(Random; 1)) as u16);
        let prod: u16 = Mult8x8((y >> 4) as u8, word!(game.room_width_in_blocks) as u8);
        if (*(ram_ptr!(game, u16; 0x10002)).offset((prod as i32 + (x as i32 >> 4)) as isize)
            & 0x3ff)
            != 255
        {
            RoomCode_GenRandomExplodes(x, y, game);
        }
    }
}
pub unsafe fn RoomCode_GenRandomExplodes(x_r18: u16, y_r20: u16, game: &mut Game) {
    static kRoomCode_GenRandomExplodes_Sprite: [u8; 8] = [3, 3, 9, 12, 12, 18, 18, 21];
    static kRoomCode_GenRandomExplodes_Sfx: [u8; 8] = [0x24, 0, 0, 0x25, 0, 0, 0, 0];
    let v1: u16 = (NextRandom(game)) & (0xfu16);
    if v1 < 8 {
        let v2: u16 = kRoomCode_GenRandomExplodes_Sfx[v1 as usize] as u16;
        if v2 != 0 {
            QueueSfx2_Max6(v2, game);
        }
    }
    create_sprite(
        x_r18,
        y_r20,
        kRoomCode_GenRandomExplodes_Sprite[(v1 as i32 & 7) as usize] as u16,
        0,
        game,
    );
}
pub unsafe fn PauseHook_DraygonRoom(game: &mut Game) {
    word!(game.irqhandler_next_handler = 4);
}
pub unsafe fn UnpauseHook_DraygonRoom(game: &mut Game) -> CoroutineRet {
    if word!(game.hdma_object_channels_bitmask[1]) == 8 {
        word!(game.irqhandler_next_handler = 12);
    }
    kCoroutineNone as CoroutineRet
}

static kRoomCode_SpawnCeresFallingDebris_Tab: [u16; 16] = [
    0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x110, 0x130, 0x150, 0x170,
    0x190,
];
macro_rules! select_room_earthquake_update {
    (
        $update_earthquake:ident, $game:ident, $timer_reference:ident, $earthquake_kind:ident,
        $ending_kind:literal, $random_threshold:literal, $starting_kind:literal $(,)?
    ) => {
        let $update_earthquake: bool = if *(ram_ptr!($game, u16; 0x7e1)) != 0 {
            crate::update_place! {
                reference $timer_reference = *(ram_ptr!($game, u16; 0x7e1))
                    => (*$timer_reference).wrapping_sub(1);
            }
            (*$timer_reference == 0)
                && ({
                    $earthquake_kind = $ending_kind;
                    true
                })
        } else {
            (NextRandom($game) < $random_threshold)
                && ({
                    strcpy(
                        ram_ptr!($game, c_char; 0x7e1),
                        b"*\0" as *const u8 as *const c_char,
                    );
                    $earthquake_kind = $starting_kind;
                    true
                })
        };
    };
}

crate::game::callback_dispatch! {
    pub unsafe fn CallRoomSetupCode(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [9408916] RoomSetup_AfterSavingAnimals [] {
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0x3d, 0xb, 0xbb30 as VoidP), game);
        word! {game; earthquake_type = 24; earthquake_timer = -1i32 as u16; }
    };
    [9408957] RoomSetup_ShakeDuringEscape [] {
        word! {game; earthquake_type = 6; earthquake_timer = -1i32 as u16; }
        initialize_land(game);
    };
    [9408983] RoomSetup_RunStatueUnlockAnims [] {
        SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Kraid as u16, game);
        SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Phantoon as u16, game);
        SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Draygon as u16, game);
        SpawnAnimtiles(addr_kAnimtiles_TourianStatue_Ridley as u16, game);
    };
    [9423069] RoomCode_SetPauseCodeForDraygon [] {
        mov24(
            ram_ptr!(game, LongPtr; 0x601),
            fnPauseHook_DraygonRoom as u32,
        );
        mov24(
            ram_ptr!(game, LongPtr; 0x604),
            fnUnpauseHook_DraygonRoom as u32,
        );
    };
    [9423114] RoomCode_SetCollectedMap [] {
        let v0: u16 = (*(ram_ptr!(game, u16; 0xd908, word!(game.area_index) as isize))) | (1u16);
        *(ram_ptr!(game, u16; 0xd908, word!(game.area_index) as isize)) = v0;
        word!(game.has_area_map = v0);
    };
    [9423135] RoomCode_SetZebesTimebombEvent [] {
        SetEventHappened(0xe, game);
        word! {game; earthquake_type = 18; earthquake_timer = -1i32 as u16; }
    };
    [9423155] RoomCode_SetLightHorizRoomShake [] {
        word!(game.earthquake_type = 18);
        *(ram_ptr!(game, u16; 0x7e1, 2)) = 18;
        *(ram_ptr!(game, u16; 0x7e1)) = 0;
        word!(game.earthquake_timer = -1i32 as u16);
    };
    [9423174] RoomCode_SetMediumHorizRoomShake [] {
        word! {game; earthquake_type = 21; earthquake_timer = -1i32 as u16; }
    };
    [9423187] RoomCode_Escape4_SetMediumHorizRoomShake [] {
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x10, 0x10, 0xb968 as VoidP),
            game,
        );
        word!(game.earthquake_type = 21);
        *(ram_ptr!(game, u16; 0x7e1, 2)) = 21;
        *(ram_ptr!(game, u16; 0x7e1)) = 0;
        word!(game.earthquake_timer = -1i32 as u16);
    };
    [9423214] RoomCode_SetCeresDoorSolid [] {
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0xf, 0x26, 0xba48 as VoidP), game);
        FxTypeFunc_2C_Haze(game);
    };
    [9423222] RoomCode_CeresColorMathHdma [] {
        FxTypeFunc_2C_Haze(game);
    };
    [9423227] RoomCode_CeresColorMathHdma_BgBase [] {
        *ram_ptr!(game; 0x5d) = 102;
        FxTypeFunc_2C_Haze(game);
        word!(game.hdma_data_table_in_ceres = 9);
    };
    @forward [pub unsafe fn (game: &mut Game)];
    [9408937] RoomSetup_AutoDestroyWallAfterEscape => SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x10, 0x87, 0xb964 as VoidP),
        game,
    );
    [9408946] RoomSetup_TurnWallIntoShotblocks => SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0xf, 0xa, 0xb9ed as VoidP),
        game,
    );
    @forward [pub unsafe fn (game: &mut Game)];
    [9408969] RoomSetup_ScrollingSkyLand => initialize_land(game);
    [9408974] RoomSetup_ScrollingSkyOcean => initialize_ocean(game);
    @forward [pub unsafe fn (game: &mut Game)];
    [9423048] RoomCode_8FC8C8 => SpawnEprojWithRoomGfx(addr_kEproj_PrePhantomRoom as u16, 0, game);
    [9423059]
    RoomCode_SetupShaktoolRoomPlm => SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0, 0, 0xb8eb as VoidP), game);
    @external {
        9408955 | 9408956 | 9408979 | 9408980 | 9408981 | 9408982 | 9409012 | 9409013 | 9409014
                | 9409015 | 9423047 | 9423056 | 9423057 | 9423058 | 9423068 | 9423134 => [()];
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallDoorDefSetupCode(ea: u32, game: &mut Game) => ea;
    call (game); default {
        if !crate::game::room_scrolls::apply(ea, game) { Unreachable(game); }
    };
    @functions [pub unsafe fn (game: &mut Game)];
    [9419121] DoorCode_StartWreckedShipTreadmillWest [] {
        SpawnAnimtiles(addr_kAnimtiles_WreckedShipTradmillRight as u16, game);
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0x4, 0x9, 0xb64b as VoidP), game);
    };
    [9429464] DoorCode_StartWreckedSkipTreadmill_East [] {
        SpawnAnimtiles(addr_kAnimtiles_WreckedShipTradmillLeft as u16, game);
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0x4, 0x9, 0xb64f as VoidP), game);
    };
    [9429612] DoorCode_SetupElevatubeFromSouth [] {
        *(ram_ptr!(game, u16; 0x7e1, 4)) = -256i32 as u16;
        *(ram_ptr!(game, u16; 0x7e1, 2)) = 2496;
        *(ram_ptr!(game, u16; 0x7e1, 6)) = -32i32 as u16;
        CallSomeSamusCode(0, game);
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0x1, 0, 0xb8f9 as VoidP), game);
    };
    [9429649] DoorCode_SetupElevatubeFromNorth [] {
        *(ram_ptr!(game, u16; 0x7e1, 4)) = 256;
        strcpy(
            ram_ptr!(game, c_char; 0x7e1, 2),
            b"@\0" as *const u8 as *const c_char,
        );
        strcpy(
            ram_ptr!(game, c_char; 0x7e1, 6),
            b" \0" as *const u8 as *const c_char,
        );
        CallSomeSamusCode(0, game);
        SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0x1, 0, 0xb8f9 as VoidP), game);
    };
    [9429761] DoorCode_ResetElevatubeNorthExit [] {
        CallSomeSamusCode(1, game);
    };
    [9429769] DoorCode_ResetElevatubeSouthExit [] {
        *(ram_ptr!(game, u16; 0xcd20)) = 514;
        CallSomeSamusCode(1, game);
    };
    [9430240] DoorCode_CeresElevatorShaft [] {
        crate::rtl::write_register_and_mirror(BGMODE as u16, 7, 0x56, true, game);
        word! { game, 256; reg_M7A, reg_M7D }
        word! { game, 0; reg_M7B, reg_M7C }
        word! {game; reg_M7X = 128; reg_M7Y = 1008; irq_enable_mode7 = 1; }
        strcpy(
            ram_ptr!(game, c_char; 0x7e1),
            b"\"\0" as *const u8 as *const c_char,
        );
        strcpy(
            ram_ptr!(game, c_char; 0x7e1, 2),
            b"<\0" as *const u8 as *const c_char,
        );
    };
    [9430291] DoorCode_CeresElevatorShaft_2 [] {
        *ram_ptr!(game; 0x56) = 9;
        word!(game.irq_enable_mode7 = 0);
    };

}
static kRoomCode_RidleyRoomShaking_X: [i16; 8] = [0, 2, 2, 2, 0, -2, -2, -2];
static kRoomCode_RidleyRoomShaking_Y: [i16; 8] = [-2, -2, 0, 2, 2, 2, 0, -2];
crate::game::callback_dispatch! {
    pub unsafe fn CallRoomCode(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [9421088] RoomCode_ScrollingSkyLand_Shakes [] {
        queue_land_rows(game);
        RoomCode_GenRandomExplodes_Nonblank(game);
        word!(game.earthquake_timer |= (0x8000u16));
    };
    [9421092] RoomCode_ExplodeShakes [] {
        RoomCode_GenRandomExplodes_Nonblank(game);
        word!(game.earthquake_timer |= (0x8000u16));
    };
    [9430432] RoomCode_GenRandomExplodes_4th [] {
        if word!(game.time_is_frozen_flag) == 0 && (word!(game.nmi_frame_counter_word) & 3) == 0 {
            let mut Random: u16 = NextRandom(game);
            RoomCode_GenRandomExplodes(
                (word!(game.layer1_x_pos)).wrapping_add((Random as u8) as u16),
                (word!(game.layer1_y_pos)).wrapping_add((byte!(Random; 1)) as u16),
                game,
            );
        }
    };
    [9421286] RoomCode_ScrollRightDachora [] {
        if *ram_ptr!(game; 0xcd20, 11) == 2
            && word!(game.layer1_y_pos) < 0x500
            && word!(game.layer1_x_pos) < 0x380
        {
            word!(checked_i32, game.layer1_x_pos += ((word!(game.layer1_x_pos) >= 0x380) as i32 + 3));
        }
    };
    [9429686] RoomCode_Elevatube [] {
        word! {game; samus_x_pos = 128; samus_x_subpos = 0; }
        let amt: i32 = (*(ram_ptr!(game, u16; 0x7e1, 4)) as i16 as i32) << 8;
        let mut v0: u16 = ((amt as u32).wrapping_add(*(ram_ptr!(game, u32; 0x7e1))) >> 16) as u16;
        crate::update_place! { unaligned_word fresh13 = &raw mut (*(ram_ptr!(game, u16; 0x7e1))) => (u16::from_le(fresh13.read_unaligned()) as i32 + amt) as u16; }
        *(ram_ptr!(game, u16; 0x7e1, 2)) = v0;
        Samus_MoveDown_NoSolidColl(amt, game);
        if ((*(ram_ptr!(game, u16; 0x7e1, 6)) as i32
            + *(ram_ptr!(game, u16; 0x7e1, 4)) as i32
            + 3616) as u16)
            < 0x1c41
        {
            crate::update_place! { unaligned_word fresh14 = &raw mut (*(ram_ptr!(game, u16; 0x7e1, 4))) => (u16::from_le(fresh14.read_unaligned())).wrapping_add(*(ram_ptr!(game, u16; 0x7e1, 6))); }
        }
    };
    [9430303] RoomCode_CeresElevatorShaft_ [] {
        RoomCode_CeresElevatorShaft(game);
    };
    [9430309] RoomCode_SpawnCeresFallingDebris [] {
        if word!(game.ceres_status) != 0
            && {
                crate::update_place! { unaligned_word fresh12 = &raw mut (*(ram_ptr!(game, u16; 0x7e1))) => (u16::from_le(fresh12.read_unaligned())).wrapping_sub(1); }
                *(ram_ptr!(game, i16; 0x7e1)) < 0
            }
        {
            *ram_ptr!(game; 0x7e1, 0) = 8;
            *ram_ptr!(game; 0x7e1, 1) = 0;
            let v0: u16 = (if word!(game.random_number) & 0x8000 != 0 {
                addr_stru_869742 as i32
            } else {
                addr_stru_869734 as i32
            }) as u16;
            SpawnEprojWithRoomGfx(
                v0,
                kRoomCode_SpawnCeresFallingDebris_Tab
                    [(word!(game.random_number) as i32 & 0xf) as usize],
                game,
            );
        }
    };
    [9430385] RoomCode_HandleCeresRidleyGetaway [] {
        if word!(game.ceres_status) & 1 != 0 {
            HandleCeresRidleyGetawayCutscene(game);
        }
    };
    [9430396] RoomCode_ShakeScreenHorizDiag [] {
        let mut v0: u16 = 0;

        select_room_earthquake_update!(update_earthquake, game, fresh11, v0, 18, 0x200, 23);
        if update_earthquake {
            word!(game.earthquake_type = v0);
        }
        RoomCode_GenRandomExplodes_4th(game);
    };
    [9430436] RoomCode_ShakeScreenHorizDiagStrong [] {
        let mut v0: i16 = 0;

        select_room_earthquake_update!(update_earthquake, game, fresh10, v0, 21, 0x180, 26);
        if update_earthquake {
            *(ram_ptr!(game, u16; 0x7e1, 2)) = v0 as u16;
        }
        word!(game.earthquake_type = *(ram_ptr!(game, u16; 0x7e1, 2)));
        RoomCode_GenRandomExplodes_4th(game);
    };
    [9431245] RoomCode_CrocomireShaking [] {
        let mut ai_var_D: u16 = 0;
        if ((*(ram_ptr!(game, EnemyData; 0xf78))).properties & 0x200) == 0 {
            if (*(ram_ptr!(game, EnemyData; 0xf78))).ai_var_A == 64 {
                word!(
                    game.reg_BG1VOFS =
                        ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).ai_var_D as i32
                            + word!(game.bg1_y_offset) as i32
                            + word!(game.layer1_y_pos) as i32) as u16
                );
            } else if ((*(ram_ptr!(game, EnemyData; 0xf78))).ai_var_B & 0x400) != 0 {
                let fresh5 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).ai_var_D;
                fresh5.write_unaligned(fresh5.read_unaligned().wrapping_sub(1));
                if fresh5.read_unaligned() as i32 + 7 & 0x8000 != 0 {
                    ai_var_D = ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).ai_var_D as i32
                        - 2i32
                            * ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).ai_var_D as i32 + 7))
                        as u16;
                } else {
                    ai_var_D = (*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).ai_var_D;
                }
                word!(game.reg_BG1VOFS += ai_var_D);
                word!(game.reg_BG2VOFS = (ai_var_D).wrapping_sub(48));
            } else if (*(ram_ptr!(game, EnemyData; 0xf78))).ai_var_C == 34
                && (*(ram_ptr!(game, EnemyData; 0xf78))).ai_var_D != 0
            {
                let fresh7 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78))).ai_var_D;
                fresh7.write_unaligned(fresh7.read_unaligned().wrapping_sub(1));
                if ((*(ram_ptr!(game, EnemyData; 0xf78))).ai_var_D & 1) != 0 {
                    word!(game.layer1_x_pos -= 4);
                } else {
                    word!(game.layer1_x_pos += 4);
                }
            }
        }
    };
    [9431376] RoomCode_RidleyRoomShaking [] {
        let ai_var_A: u16 = (*(ram_ptr!(game, EnemyData; 0xf78)).offset(4)).ai_var_A;
        if (*(ram_ptr!(game, EnemyData; 0xf78)).offset(4)).ai_var_A != 0 {
            let fresh0 = &raw mut (*(ram_ptr!(game, EnemyData; 0xf78)).offset(4)).ai_var_A;
            fresh0.write_unaligned(fresh0.read_unaligned().wrapping_sub(1));
            let v1: i32 = ai_var_A as i32 - 1i32;
            word!(game.reg_BG1HOFS += (kRoomCode_RidleyRoomShaking_X[v1 as usize]) as u16);
            word!(game.reg_BG2HOFS += (kRoomCode_RidleyRoomShaking_X[v1 as usize]) as u16);
            word!(game.reg_BG1VOFS += (kRoomCode_RidleyRoomShaking_Y[v1 as usize]) as u16);
            word!(game.reg_BG2VOFS += (kRoomCode_RidleyRoomShaking_Y[v1 as usize]) as u16);
        }
    };
    @forward [pub unsafe fn (game: &mut Game)];
    [9421078] RoomCode_ScrollingSkyLand_ => queue_land_rows(game);
    [9421083] RoomCode_ScrollingSkyOcean_ => queue_ocean_rows(game);
    @external {
        9430308 => [()];
    }
}
pub unsafe fn RunRoomMainCode(game: &mut Game) {
    if word!(game.room_main_code_ptr) != 0 {
        CallRoomCode(
            ((word!(game.room_main_code_ptr)) as u32) | (0x8f0000u32),
            game,
        );
    }
}
