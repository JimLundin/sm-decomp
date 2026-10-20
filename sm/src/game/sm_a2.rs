use crate::game::callback_tables::callback_table;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::enemy_properties::*;
pub use crate::game::constants::game_states::*;
pub use crate::game::constants::loading_game_states::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::enemy_instruction;
use crate::game::sm_80::{CheckEventHappened, NextRandom, QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game::sm_81::SaveToSram;
use crate::game::sm_85::DisplayMessageBox_Poll;
use crate::game::sm_86::{
    Math_MultByCos, Math_MultBySin, SpawnEprojWithGfx, SpawnEprojWithRoomGfx,
};
use crate::game::sm_8d::SpawnPalfxObject;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_91::{
    MakeSamusFaceForward, Samus_RestoreHealth, Samus_RestoreMissiles, Samus_RestorePowerBombs,
    Samus_RestoreSuperMissiles,
};
use crate::game::sm_94::kSinCosTable8bit_Sext;
use crate::game::sm_a0::{
    CalculateAngleFromXY, CalculateAngleOfSamusFromEnemy, CallEnemyPreInstr,
    CheckIfEnemyIsOnScreen, CheckIfEnemyTouchesSamus, CompareDistToSamus_X, ConvertAngleToXy,
    DeleteEnemyAndConnectedEnemies, EnemyFunc_AC67, EnemyFunc_BC76, EnemyGrappleDeath,
    Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes, Enemy_MoveRight_SlopesAsWalls,
    Enemy_SwitchToFrozenAi, GetSamusEnemyDelta_X, IsSamusWithinEnemy_X, IsSamusWithinEnemy_Y,
    NormalEnemyFrozenAI, NormalEnemyPowerBombAi, NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy,
    NormalEnemyShotAi, NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi,
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy, RinkasDeathAnimation, SamusLatchesOnWithGrapple,
};
use crate::game::sm_a9::MoveEnemyWithVelocity;
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::word;
use crate::rtl::abs16;
use crate::rtl::rom_ptr;
use crate::rtl::words;
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{Mult8x8, RomPtr, Unreachable};
use crate::set_fields;
use ::core::ptr::null_mut;

pub use crate::types::Buttons;
pub use crate::types::Consts_86;
pub use crate::types::Consts_8D;
pub use crate::types::Consts_A2;
pub use crate::types::GameState;
pub use crate::types::LoadingGameState;
pub use crate::types::{
    EnemyBase, EnemyData, Func_V, LongPtr, Point16U, SamusPose, VoidP, VramWriteEntry,
};
pub use crate::types::{EnemyProps, EnemySpawnData};
crate::game::constants::typed_constants! { Consts_A2;
    addr_kLavaSeahorse_Ilist_E5A1 = 58785, addr_kLavaSeahorse_Ilist_E59B = 58779,
    addr_kRipper_Ilist_E48B = 58507, addr_kRipper_Ilist_E477 = 58487,
    addr_kGripper_Sprmap_E44B = 58443, addr_kGripper_Sprmap_E43F = 58431,
    addr_kRipper_Ilist_E2F4 = 58100, addr_kRipper_Ilist_E2E0 = 58080,
    addr_kGripper_Ilist_E1AF = 57775, addr_kGripper_Ilist_E19B = 57755,
    addr_kHirisingSlowfalling_Ilist_D840 = 55360, addr_kHirisingSlowfalling_Ilist_D834 = 55348,
    addr_kHirisingSlowfalling_Ilist_D82C = 55340, addr_kMaridiaLargeSnail_Ilist_CA4B = 51787,
    addr_kLowerNorfairRio_Ilist_C6B0 = 50864, addr_kLowerNorfairRio_Ilist_C686 = 50822,
    addr_kLowerNorfairRio_Ilist_C674 = 50804, addr_kLowerNorfairRio_Ilist_C662 = 50786,
    addr_kLowerNorfairRio_Ilist_C65A = 50778, addr_kLowerNorfairRio_Ilist_C630 = 50736,
    addr_kLowerNorfairRio_Ilist_C61A = 50714, addr_kNorfairRio_Ilist_C1A3 = 49571,
    addr_kNorfairRio_Ilist_C18F = 49551, addr_kNorfairRio_Ilist_C179 = 49529,
    addr_kNorfairRio_Ilist_C145 = 49477, addr_kNorfairRio_Ilist_C12F = 49455,
    addr_kNorfairRio_Ilist_C107 = 49415, addr_kNorfairRio_Ilist_C0F1 = 49393,
    addr_kNorfairLavajumpingEnemy_Ilist_BE62 = 48738, addr_kNorfairLavajumpingEnemy_Ilist_BE42 = 48706,
    addr_kNorfairLavajumpingEnemy_Ilist_BE3C = 48700, addr_kRio_Ilist_BBA3 = 48035,
    addr_kRio_Ilist_BB97 = 48023, addr_kRio_Ilist_BB7F = 47999,
    addr_stru_A2BB53 = 47955, addr_kRio_Ilist_BB4B = 47947,
    addr_kRinka_Ilist_BA0C = 47628, addr_kRinka_Ilist_B9E0 = 47584,
    addr_kNorfairErraticFireball_Ilist_B51A = 46362, addr_kNorfairErraticFireball_Ilist_B2DC = 45788,
    addr_kFlies_Ilist_B013 = 45075, addr_kGunshipTop_Ilist_A61C = 42524,
    addr_kGunshipTop_Ilist_A616 = 42518, addr_kGunshipTop_Ilist_A60E = 42510,
    addr_kGunshipTop_Ilist_A5EE = 42478, addr_kGunshipTop_Ilist_A5BE = 42430,
    addr_kMaridiaSpikeyShell_Ilist_A3BD = 41917, addr_kMaridiaSpikeyShell_Ilist_A3AB = 41899,
    addr_kSpikeShootingPlant_Ilist_9F00 = 40704, addr_kSpikeShootingPlant_Ilist_9EDA = 40666,
    addr_kSpikeShootingPlant_Ilist_9EB0 = 40624, addr_kSpikeShootingPlant_Ilist_9E8A = 40586,
    addr_kThinHoppingBlobs_Ilist_9A01 = 39425, addr_kThinHoppingBlobs_Ilist_99FB = 39419,
    addr_kThinHoppingBlobs_Ilist_99F5 = 39413, addr_kThinHoppingBlobs_Ilist_99EF = 39407,
    addr_kThinHoppingBlobs_Ilist_99E9 = 39401, addr_kThinHoppingBlobs_Ilist_99D5 = 39381,
    addr_kThinHoppingBlobs_Ilist_99C1 = 39361, addr_kThinHoppingBlobs_Ilist_99AD = 39341,
    addr_locret_A28E09 = 36361, addr_kMaridiaBeybladeTurtle_Ilist_8D40 = 36160,
    addr_kMaridiaBeybladeTurtle_Ilist_8D28 = 36136, addr_kMaridiaBeybladeTurtle_Ilist_8D14 = 36116,
    addr_kMaridiaBeybladeTurtle_Ilist_8D00 = 36096, addr_kMaridiaBeybladeTurtle_Ilist_8C72 = 35954,
    addr_kMaridiaBeybladeTurtle_Ilist_8C62 = 35938, addr_kMaridiaBeybladeTurtle_Ilist_8C4A = 35914,
    addr_kMaridiaBeybladeTurtle_Ilist_8C44 = 35908, addr_kMaridiaBeybladeTurtle_Ilist_8C30 = 35888,
    addr_kMaridiaBeybladeTurtle_Ilist_8C1C = 35868, addr_kMaridiaBeybladeTurtle_Ilist_8C02 = 35842,
    addr_kMaridiaBeybladeTurtle_Ilist_8BD2 = 35794, addr_kMaridiaBeybladeTurtle_Ilist_8B80 = 35712,
    addr_kMiniCrocomire_Ilist_896E = 35182, addr_kMiniCrocomire_Ilist_8958 = 35160,
    addr_kMiniCrocomire_Ilist_8948 = 35144, addr_kMiniCrocomire_Ilist_8932 = 35122,
    addr_kBouncingGoofball_Ilist_86BF = 34495, addr_kBouncingGoofball_Ilist_86A7 = 34471,
    addr_kSpritemap_Nothing_A2 = 32845,
}
pub use crate::types::Point32;
pub type Enemy_BouncingGoofball = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_Rio = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_Flies = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_LavaSeahorse = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_NorfairRio = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_MaridiaBeybladeTurtle = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_MaridiaTurtle = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_MaridiaLargeSnail = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_MaridiaSpikeyShell = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_MiniCrocomire = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_NorfairLavajumpingEnemy = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_Rinka = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_SpikeShootingPlant = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_Gripper = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_GunshipBottom = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_GunshipTop = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_HirisingSlowfalling = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_JetPowerRipper = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_LavaquakeRocks = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_LowerNorfairRio = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_MiniMaridiaTurtle = crate::game::enemy_layouts::EnemyLayout40;

pub type Enemy_MiniMaridiaBeybladeTurtle = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_NorfairErraticFireball = crate::game::enemy_layouts::EnemyLayout8;

pub use crate::types::Ram8000_Default;
pub type Enemy_Ripper = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_ThinHoppingBlobs = crate::game::enemy_layouts::EnemyLayout40;

const RomPtr_A2: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa2>;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
crate::game::constants::typed_constants! { i32;
    fnSamus_FrameHandlerAlfa_Func11 = 0x90e695, fnSamus_FrameHandlerBeta_Func17 = 0x90e725,
    fnMiniCrocomire_PreInstr5 = 0xa28a43, fnMiniCrocomire_PreInstr6 = 0xa28a5c,
    fnnullsub_175 = 0xa28a75,
}
crate::game::constants::typed_constants! { i32;
    fnSpikeShootingPlant_2 = 0xa29fba, fnSpikeShootingPlant_3 = 0xa29fec,
    fnnullsub_182 = 0xa2a01b, fnnullsub_187 = 0xa2a7d7,
    fnGunshipTop_3 = 0xa2a80c, fnGunshipTop_4 = 0xa2a8d0,
    fnGunshipTop_5 = 0xa2a942, fnGunshipTop_6 = 0xa2a950,
    fnGunshipTop_7 = 0xa2a987, fnGunshipTop_8 = 0xa2a9bd,
    fnGunshipTop_9 = 0xa2aa4f, fnGunshipTop_10 = 0xa2aa5d,
    fnGunshipTop_11 = 0xa2aa94, fnGunshipTop_12 = 0xa2aaa2,
    fnGunshipTop_13 = 0xa2ab1f, fnGunshipTop_14 = 0xa2ab60,
    fnGunshipTop_15 = 0xa2ab6e, fnGunshipTop_16 = 0xa2aba5,
    fnGunshipTop_17 = 0xa2abc7, fnGunshipTop_18 = 0xa2ac1b,
    fnGunshipTop_19 = 0xa2acd7, fnGunshipTop_20 = 0xa2ad0e,
    fnGunshipTop_21 = 0xa2ad2d, fnFlies_4 = 0xa2b14e,
    fnFlies_5 = 0xa2b17c, fnFlies_6 = 0xa2b1aa,
    fnFlies_7 = 0xa2b1d2,
    fnNorfairLavajumpingEnemy_Func_1 = 0xa2bedc,
    fnNorfairLavajumpingEnemy_Func_2 = 0xa2bf1a, fnNorfairLavajumpingEnemy_Func_3 = 0xa2bf3e,
    fnNorfairLavajumpingEnemy_Func_4 = 0xa2bf7c, fnNorfairLavajumpingEnemy_Func_5 = 0xa2bfbc,
    fnNorfairRio_Func_1 = 0xa2c281, fnNorfairRio_Func_2 = 0xa2c2e7,
    fnNorfairRio_Func_3 = 0xa2c33f, fnNorfairRio_Func_4 = 0xa2c361,
    fnNorfairRio_Func_5 = 0xa2c3b1, fnNorfairRio_Func_6 = 0xa2c406,
    fnLowerNorfairRio_Func_1 = 0xa2c72e, fnLowerNorfairRio_Func_2 = 0xa2c771,
    fnLowerNorfairRio_Func_3 = 0xa2c7bb, fnLowerNorfairRio_Func_4 = 0xa2c7d6,
    fnLowerNorfairRio_Func_5 = 0xa2c82d, fnLowerNorfairRio_Func_6 = 0xa2c888,
    fnMaridiaLargeSnail_Func_7 = 0xa2cf66,
    fnMaridiaLargeSnail_Func_8 = 0xa2cfa9,
    fnLavaSeahorse_Func_1 = 0xa2e654,
    fnLavaSeahorse_Func_2 = 0xa2e6ad, fnLavaSeahorse_Func_3 = 0xa2e6f1,
    fnLavaSeahorse_Func_4 = 0xa2e734, fnLavaSeahorse_Func_5 = 0xa2e749,
    fnnullsub_196 = 0xa2e781,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    gEnemySpawnData(o: i32) -> EnemySpawnData = [0x7000, o as isize];
    gRam8000_Default(o: i32) -> Ram8000_Default = [0x8000, o as isize];
    Get_Flies(o: i32) -> Enemy_Flies = [0xf78, o as isize];
    Get_Gripper(o: i32) -> Enemy_Gripper = [0xf78, o as isize];
    Get_GunshipBottom(o: i32) -> Enemy_GunshipBottom = [0xf78, o as isize];
    Get_GunshipTop(o: i32) -> Enemy_GunshipTop = [0xf78, o as isize];
    Get_HirisingSlowfalling(o: i32) -> Enemy_HirisingSlowfalling = [0xf78, o as isize];
    Get_JetPowerRipper(o: i32) -> Enemy_JetPowerRipper = [0xf78, o as isize];
    Get_LavaSeahorse(o: i32) -> Enemy_LavaSeahorse = [0xf78, o as isize];
    Get_LavaquakeRocks(o: i32) -> Enemy_LavaquakeRocks = [0xf78, o as isize];
    Get_LowerNorfairRio(o: i32) -> Enemy_LowerNorfairRio = [0xf78, o as isize];
    Get_MaridiaBeybladeTurtle(o: i32) -> Enemy_MaridiaBeybladeTurtle = [0xf78, o as isize];
    Get_MaridiaLargeSnail(o: i32) -> Enemy_MaridiaLargeSnail = [0xf78, o as isize];
    Get_MaridiaSpikeyShell(o: i32) -> Enemy_MaridiaSpikeyShell = [0xf78, o as isize];
    Get_MaridiaTurtle(o: i32) -> Enemy_MaridiaTurtle = [0xf78, o as isize];
    Get_MiniMaridiaBeybladeTurtle(o: i32) -> Enemy_MiniMaridiaBeybladeTurtle = [0xf78, o as isize];
    Get_MiniMaridiaTurtle(o: i32) -> Enemy_MiniMaridiaTurtle = [0xf78, o as isize];
    Get_NorfairErraticFireball(o: i32) -> Enemy_NorfairErraticFireball = [0xf78, o as isize];
    Get_NorfairLavajumpingEnemy(o: i32) -> Enemy_NorfairLavajumpingEnemy = [0xf78, o as isize];
    Get_NorfairRio(o: i32) -> Enemy_NorfairRio = [0xf78, o as isize];
    Get_Rinka(o: i32) -> Enemy_Rinka = [0xf78, o as isize];
    Get_Rio(o: i32) -> Enemy_Rio = [0xf78, o as isize];
    Get_Ripper(o: i32) -> Enemy_Ripper = [0xf78, o as isize];
    Get_SpikeShootingPlant(o: i32) -> Enemy_SpikeShootingPlant = [0xf78, o as isize];
    Get_ThinHoppingBlobs(o: i32) -> Enemy_ThinHoppingBlobs = [0xf78, o as isize];
}
static g_word_A28D50: u16 = 0x30;
static g_word_A28D52: u16 = 1;
static g_word_A28D5E: u16 = 3;
static g_word_A28D60: u16 = 0x1e8;
static g_word_A28D62: u16 = 7;
static g_word_A28D64: u16 = 0x1e;
static g_word_A28D66: u16 = 4;
static g_word_A28D68: u16 = 0xfffd;
static g_word_A28D6A: u16 = 3;
static g_word_A28E80: [i16; 48] = [
    -16, -16, -16, -16, -15, -15, -15, -15, -15, -14, -13, -13, -12, -11, -10, -9, -8, -7, -6, -5,
    -4, -4, 0, 0, -16, -16, -16, -15, -15, -15, -14, -13, -12, -11, -10, -9, -8, -7, -6, -5, -4,
    -3, -3, -2, 0, 0, 0, 0,
];
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Enemy_GrappleReact_CancelBeam_A2 => Enemy_SwitchToFrozenAi(game);
    Enemy_GrappleReact_KillEnemy_A2 => EnemyGrappleDeath(game);
    Enemy_GrappleReact_SamusLatchesOn_A2 => SamusLatchesOnWithGrapple(game);
    Enemy_NormalFrozenAI_A2 => NormalEnemyFrozenAI(game);
    Enemy_NormalPowerBombAI_A2 => NormalEnemyPowerBombAi(game);
    Enemy_NormalTouchAI_A2 => NormalEnemyTouchAi(game);
    Enemy_NormalShotAI_A2 => NormalEnemyShotAi(game);
}
pub fn BouncingGoofball_Func1(k: u16, game: &mut Game) {
    loop {
        let phase = GoofballState::new(game, k).phase();
        let product = goofball_movement_product(k, phase, game);
        GoofballState::new(game, k).set_product(product);
        let accumulated = GoofballState::new(game, k)
            .accumulated()
            .wrapping_add(product);
        GoofballState::new(game, k).set_accumulated(accumulated);
        let phase = GoofballState::new(game, k).phase().wrapping_add(1);
        GoofballState::new(game, k).set_phase(phase);
        if (accumulated.wrapping_sub(GoofballState::new(game, k).distance()) as i16) >= 0 {
            break;
        }
    }
    let phase = GoofballState::new(game, k).phase();
    GoofballState::new(game, k).set_limit(phase);
    GoofballState::new(game, k).set_active(1);
    GoofballState::new(game, k).set_animation_ready(1);
}
struct GoofballState<'a> {
    bytes: &'a mut [u8],
}
impl<'a> GoofballState<'a> {
    fn accumulated(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, var_D))
    }
    fn distance(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, var_C))
    }
    fn set_limit(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_BouncingGoofball, var_00), value);
    }
    fn initialized(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, var_05))
    }
    fn set_initialized(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_BouncingGoofball, var_05), value);
    }
    fn saved_scale(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, var_03))
    }
    fn set_saved_scale(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_BouncingGoofball, var_03), value);
    }
    fn animation_ready(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, var_04))
    }
    fn set_animation_ready(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_BouncingGoofball, var_04), value);
    }
    fn active(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, var_02))
    }
    fn animation_trigger(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, var_F))
    }
    fn state(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, var_01))
    }
    fn parameter_low(&self) -> u8 {
        self.bytes[core::mem::offset_of!(Enemy_BouncingGoofball, parameter_1)]
    }
    fn parameter_high(&self) -> u8 {
        self.bytes[core::mem::offset_of!(Enemy_BouncingGoofball, parameter_1) + 1]
    }
    fn set_scale(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_BouncingGoofball, var_A), value);
    }
    fn set_distance(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_BouncingGoofball, var_C), value);
    }
    fn set_accumulated(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_BouncingGoofball, var_D), value);
    }
    fn new(game: &'a mut Game, index: u16) -> Self {
        Self {
            bytes: &mut game.g_ram[0xf78 + usize::from(index)..],
        }
    }
    fn read(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.bytes[offset], self.bytes[offset + 1]])
    }
    fn write(&mut self, offset: usize, value: u16) {
        self.bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn phase(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, var_E))
    }
    fn set_phase(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_BouncingGoofball, var_E), value);
    }
    fn scale(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, var_A))
    }
    fn set_product(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_BouncingGoofball, var_B), value);
    }
    fn limit(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, var_00))
    }
    fn set_state(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_BouncingGoofball, var_01), value);
    }
    fn set_active(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_BouncingGoofball, var_02), value);
    }
    fn set_animation_trigger(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_BouncingGoofball, var_F), value);
    }
    fn y_pos(&self) -> u16 {
        self.read(
            core::mem::offset_of!(Enemy_BouncingGoofball, base)
                + core::mem::offset_of!(EnemyBase, y_pos),
        )
    }
    fn set_y_pos(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, base)
                + core::mem::offset_of!(EnemyBase, y_pos),
            value,
        );
    }
}
fn goofball_movement_product(index: u16, phase: u16, game: &mut Game) -> u16 {
    // The ROM byte is read even when the signed gate replaces it with FF.
    let sample = game.read_rom_byte(0xa28701, usize::from(phase));
    let sample = if (phase.wrapping_sub(23) as i16) >= 0 {
        255
    } else {
        sample
    };
    u16::from(sample) * u16::from(GoofballState::new(game, index).scale() as u8)
}
pub fn BouncingGoofball_SamusCloseX(game: &mut Game) -> u16 {
    let slot = game.cur_enemy_index();
    let index = game.cur_enemy_index();
    let offset =
        0xf78 + usize::from(slot) + core::mem::offset_of!(Enemy_BouncingGoofball, parameter_2);
    let radius = u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
    IsSamusWithinEnemy_X(index, radius, game)
}
fn set_goofball_animation(index: u16, instruction: u16, game: &mut Game) {
    game.enemy_instructions(index)
        .set_current_instruction(instruction);
    game.enemy_instructions(index).set_instruction_timer(1);
    game.enemy_instructions(index).set_timer(0);
}
pub fn BouncingGoofball_Instr_88C5(_k: u16, jp: *const u16) -> *const u16 {
    jp
}
crate::instruction_callbacks! { pub unsafe fn (_k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    BouncingGoofball_Instr_88C6 [jp] {
        let index = game.cur_enemy_index();
        let offset = 0xf78 + usize::from(index) + core::mem::offset_of!(Enemy_BouncingGoofball, var_02);
        game.g_ram[offset..offset + 2].copy_from_slice(&0u16.to_le_bytes());
        QueueSfx2_Max6(0xe, game);
    };
    MiniCrocomire_Instr_897E [jp.add(1)] {
        let parameter = *jp;
        let index = game.cur_enemy_index();
        SpawnEprojWithGfx(parameter, index, addr_Eproj_DBF2 as u16, game);
    };
}
crate::instruction_callbacks! { pub fn (_k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    @prefix { let index = game.cur_enemy_index(); };
    MiniCrocomire_Instr_8990 [jp] {  set_mini_crocomire_preinstruction(index, 0x8a43, 0, game); };
    MiniCrocomire_Instr_899D [jp] {  set_mini_crocomire_preinstruction(index, 0x8a5c, 1, game); };
}
fn set_mini_crocomire_preinstruction(index: u16, preinstruction: u16, state: u16, game: &mut Game) {
    let base = 0xf78 + usize::from(index);
    let field = base + core::mem::offset_of!(Enemy_MiniCrocomire, var_F);
    game.g_ram[field..field + 2].copy_from_slice(&preinstruction.to_le_bytes());
    let field = base + core::mem::offset_of!(Enemy_MiniCrocomire, var_E);
    game.g_ram[field..field + 2].copy_from_slice(&state.to_le_bytes());
}

struct MiniCrocomireState<'a> {
    bytes: &'a mut [u8],
}
impl<'a> MiniCrocomireState<'a> {
    fn new(game: &'a mut Game, index: u16) -> Self {
        Self {
            bytes: &mut game.g_ram[0xf78 + usize::from(index)..],
        }
    }
    fn read(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.bytes[offset], self.bytes[offset + 1]])
    }
    fn write(&mut self, offset: usize, value: u16) {
        self.bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn parameter_1(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, parameter_1))
    }
    fn parameter_2(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, parameter_2))
    }
    fn var_a(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, var_A))
    }
    fn var_b(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, var_B))
    }
    fn var_c(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, var_C))
    }
    fn var_d(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, var_D))
    }
    fn state(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, var_E))
    }
    fn frame_counter_low(&self) -> u8 {
        self.bytes[core::mem::offset_of!(Enemy_MiniCrocomire, base)
            + core::mem::offset_of!(EnemyBase, frame_counter)]
    }
    fn set_var_b(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, var_B), value);
    }
    fn set_var_a(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, var_A), value);
    }
    fn set_var_d(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, var_D), value);
    }
    fn set_var_c(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, var_C), value);
    }
    fn preinstruction(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, var_F))
    }
    fn set_preinstruction(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, var_F), value);
    }
    fn set_state(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, var_E), value);
    }
}
pub unsafe fn MiniCrocomire_Main(game: &mut Game) {
    let index = game.cur_enemy_index();
    let preinstruction = MiniCrocomireState::new(game, index).preinstruction();
    CallEnemyPreInstr(u32::from(preinstruction) | 0xa20000, game);
}
fn set_mini_crocomire_animation(index: u16, instruction: u16, game: &mut Game) {
    game.enemy_instructions(index).set_instruction_timer(1);
    game.enemy_instructions(index).set_timer(0);
    game.enemy_instructions(index)
        .set_current_instruction(instruction);
}
pub unsafe fn MiniCrocomire_PreInstr5(_k: u16, game: &mut Game) {
    let index = game.cur_enemy_index();
    let high = MiniCrocomireState::new(game, index).var_d();
    let low = MiniCrocomireState::new(game, index).var_c();
    MiniCrocomire_Func7((u32::from(high) << 16 | u32::from(low)) as i32, game);
    if MiniCrocomire_Func9(game) != 0 {
        MiniCrocomire_Func2(game);
    }
}
pub unsafe fn MiniCrocomire_PreInstr6(_k: u16, game: &mut Game) {
    let index = game.cur_enemy_index();
    let high = MiniCrocomireState::new(game, index).var_b();
    let low = MiniCrocomireState::new(game, index).var_a();
    MiniCrocomire_Func7((u32::from(high) << 16 | u32::from(low)) as i32, game);
    if MiniCrocomire_Func9(game) != 0 {
        MiniCrocomire_Func4(game);
    }
}
pub unsafe fn MiniCrocomire_Func7(amt: i32, game: &mut Game) {
    let index = game.cur_enemy_index();
    if Enemy_MoveRight_IgnoreSlopes(index, amt, game) == 0 {
        let index = game.cur_enemy_index();
        if EnemyFunc_BC76(index, 2 << 16, game) != 0 {
            return;
        }
    }
    MiniCrocomire_Func8(game);
}
pub fn MiniCrocomire_Func8(game: &mut Game) {
    MiniCrocomire_Func1(game);
    let index = game.cur_enemy_index();
    if MiniCrocomireState::new(game, index).state() != 1 {
        MiniCrocomire_Func3(game);
    }
}
pub unsafe fn MiniCrocomire_Func9(game: &mut Game) -> u8 {
    NextRandom(game);
    let index = game.cur_enemy_index();
    let frame = MiniCrocomireState::new(game, index).frame_counter_low();
    if frame.wrapping_add(game.random_number() as u8) < 2 {
        MiniCrocomireState::new(game, index).set_preinstruction(0x8a75);
        return 1;
    }
    0
}
crate::prefixed_callbacks! {
    pub fn (game: &mut Game);
    @prefix { let index = game.cur_enemy_index(); };
    BouncingGoofball_Init [] {
        game.enemy_instructions(index)
            .set_spritemap_pointer(addr_kSpritemap_Nothing_A2 as u16);
        BouncingGoofball_Func2(game);
        GoofballState::new(game, index).set_state(1);
        let parameter = GoofballState::new(game, index).parameter_low();
        let scale = game.read_rom_word(0xa286df, usize::from(parameter) * 2);
        GoofballState::new(game, index).set_scale(scale);
        let parameter = GoofballState::new(game, index).parameter_high();
        let distance = game.read_rom_word(0xa286ef, usize::from(parameter) * 2);
        GoofballState::new(game, index).set_distance(distance);
        GoofballState::new(game, index).set_accumulated(0);
        GoofballState::new(game, index).set_phase(0);
        GoofballState::new(game, index).set_animation_trigger(0);
    };
    BouncingGoofball_Main [] {
        if GoofballState::new(game, index).initialized() == 0 {
            BouncingGoofball_Func1(game.cur_enemy_index(), game);
            GoofballState::new(game, index).set_initialized(1);
            return;
        }
        let scale = GoofballState::new(game, index).scale();
        GoofballState::new(game, index).set_saved_scale(scale);
        if BouncingGoofball_SamusCloseX(game) != 0 {
            GoofballState::new(game, index).set_active(0);
            GoofballState::new(game, index).set_animation_ready(0);
            if GoofballState::new(game, index).animation_trigger() == 0 {
                GoofballState::new(game, index).set_animation_trigger(1);
                BouncingGoofball_Func3(game);
            }
        } else if GoofballState::new(game, index).active() != 0 {
            if GoofballState::new(game, index).animation_ready() == 0 {
                GoofballState::new(game, index).set_animation_ready(1);
                BouncingGoofball_Func2(game);
            }
            return;
        }
        let scale = GoofballState::new(game, index).saved_scale();
        GoofballState::new(game, index).set_scale(scale);
        match GoofballState::new(game, index).state() {
            0 => BouncingGoofball_State0(game),
            1 => BouncingGoofball_State1(game),
            _ => panic!("invalid Goofball state"),
        }
    };
    BouncingGoofball_State0 [] {
        let phase = GoofballState::new(game, index).phase().wrapping_add(1);
        GoofballState::new(game, index).set_phase(phase);
        let phase = GoofballState::new(game, index).phase();
        let product = goofball_movement_product(index, phase, game);
        GoofballState::new(game, index).set_product(product);
        let y = GoofballState::new(game, index)
            .y_pos()
            .wrapping_add(product >> 8);
        GoofballState::new(game, index).set_y_pos(y);
        if (GoofballState::new(game, index)
            .phase()
            .wrapping_sub(GoofballState::new(game, index).limit()) as i16)
            >= 0
        {
            GoofballState::new(game, index).set_state(1);
            GoofballState::new(game, index).set_active(1);
            GoofballState::new(game, index).set_animation_trigger(0);
        }
    };
    BouncingGoofball_State1 [] {
        let phase = GoofballState::new(game, index).phase();
        let product = goofball_movement_product(index, phase, game);
        GoofballState::new(game, index).set_product(product);
        let y = GoofballState::new(game, index)
            .y_pos()
            .wrapping_sub(product >> 8);
        GoofballState::new(game, index).set_y_pos(y);
        let phase = GoofballState::new(game, index).phase().wrapping_sub(1);
        GoofballState::new(game, index).set_phase(phase);
        if GoofballState::new(game, index).phase() & 0x8000 != 0 {
            GoofballState::new(game, index).set_state(0);
        }
    };
    BouncingGoofball_Func2 [] {
        set_goofball_animation(index, addr_kBouncingGoofball_Ilist_86A7 as u16, game);
    };
    BouncingGoofball_Func3 [] {
        set_goofball_animation(index, addr_kBouncingGoofball_Ilist_86BF as u16, game);
    };
    MiniCrocomire_Init [] {
        game.enemy_instructions(index)
            .set_spritemap_pointer(addr_kSpritemap_Nothing_A2 as u16);
        // Parameter indexing is wider than a SNES word; do not wrap this offset.
        let table_offset = usize::from(MiniCrocomireState::new(game, index).parameter_2()) * 8;
        let value = game.read_rom_word(0xa28187, table_offset);
        MiniCrocomireState::new(game, index).set_var_b(value);
        let value = game.read_rom_word(0xa28187, table_offset + 2);
        MiniCrocomireState::new(game, index).set_var_a(value);
        let value = game.read_rom_word(0xa28187, table_offset + 4);
        MiniCrocomireState::new(game, index).set_var_d(value);
        let value = game.read_rom_word(0xa28187, table_offset + 6);
        MiniCrocomireState::new(game, index).set_var_c(value);
        MiniCrocomire_Func1(game);
        MiniCrocomireState::new(game, index).set_preinstruction(0x8a43);
        let parameter = MiniCrocomireState::new(game, index).parameter_1();
        MiniCrocomireState::new(game, index).set_state(parameter);
        if parameter != 0 {
            MiniCrocomire_Func3(game);
            MiniCrocomireState::new(game, index).set_preinstruction(0x8a5c);
        }
    };
    MiniCrocomire_Func1 [] {
        set_mini_crocomire_animation(index, addr_kMiniCrocomire_Ilist_8932 as u16, game);
    };
    MiniCrocomire_Func2 [] {
        set_mini_crocomire_animation(index, addr_kMiniCrocomire_Ilist_8948 as u16, game);
    };
    MiniCrocomire_Func3 [] {
        set_mini_crocomire_animation(index, addr_kMiniCrocomire_Ilist_8958 as u16, game);
    };
    MiniCrocomire_Func4 [] {
        set_mini_crocomire_animation(index, addr_kMiniCrocomire_Ilist_896E as u16, game);
    };
    MaridiaBeybladeTurtle_Init [] {
        let properties = game.enemy_instructions(index).properties();
        game.enemy_instructions(index)
            .set_properties(properties | kEnemyProps_DisableSamusColl as u16);
        game.enemy_instructions(index)
            .set_spritemap_pointer(addr_kSpritemap_Nothing_A2 as u16);
        game.enemy_instructions(index).set_instruction_timer(1);
        game.enemy_instructions(index).set_timer(0);
        game.enemy_coordinates(index).set_y_height(0);
        game.enemy_instructions(index)
            .set_current_instruction(addr_kMaridiaBeybladeTurtle_Ilist_8C44 as u16);
        let base = 0xf78 + usize::from(index);
        let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, var_A);
        game.g_ram[field..field + 2].copy_from_slice(&0x8dd8u16.to_le_bytes());
        let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, var_F);
        game.g_ram[field..field + 2].copy_from_slice(&g_word_A28D52.to_le_bytes());
    };
    MiniMaridiaBeybladeTurtle_Init [] {
        let base = 0xf78 + usize::from(index);
        let x = game.enemy_coordinates(index).x_pos();
        let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, var_C);
        game.g_ram[field..field + 2].copy_from_slice(&x.to_le_bytes());
        let y = game.enemy_coordinates(index).y_pos();
        let radius = game.enemy_coordinates(index).y_height();
        let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, var_D);
        game.g_ram[field..field + 2].copy_from_slice(&y.wrapping_sub(radius).to_le_bytes());
        let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, var_A);
        game.g_ram[field..field + 2].copy_from_slice(&0x9142u16.to_le_bytes());
        game.enemy_instructions(index).set_instruction_timer(1);
        game.enemy_instructions(index).set_timer(0);
        let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, parameter_1);
        let parameter = u16::from_le_bytes([game.g_ram[field], game.g_ram[field + 1]]);
        let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, var_E);
        game.g_ram[field..field + 2].copy_from_slice(&parameter.to_le_bytes());
        let instruction = if (parameter as i16) >= 0 {
            addr_kMaridiaBeybladeTurtle_Ilist_8C72 as u16
        } else {
            addr_kMaridiaBeybladeTurtle_Ilist_8B80 as u16
        };
        game.enemy_instructions(index)
            .set_current_instruction(instruction);
    };
}
crate::game::address_dispatch! {
    pub unsafe fn CallMaridiaBeybladeTurtleFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        0xa28dd8 => MaridiaBeybladeTurtle_Func1; 0xa28e0a => MaridiaBeybladeTurtle_Func2;
        0xa28ee0 => MaridiaBeybladeTurtle_Func3; 0xa28f3f => MaridiaBeybladeTurtle_Func5;
        0xa28f8d => MaridiaBeybladeTurtle_Func7; 0xa28feb => MaridiaBeybladeTurtle_Func8;
        0xa29083 => MaridiaBeybladeTurtle_Func9; 0xa290cc => MaridiaBeybladeTurtle_Func10;
        0xa290e1 => MaridiaBeybladeTurtle_Func11; 0xa28e09 => [{}];
        0xa29142 => MiniMaridiaBeybladeTurtle_Func1; 0xa2916e => MiniMaridiaBeybladeTurtle_Func2;
        0xa29198 => MiniMaridiaBeybladeTurtle_Func3; 0xa291f8 => MiniMaridiaBeybladeTurtle_Func4;
        0xa29239 => MiniMaridiaBeybladeTurtle_Func6;
        0xa2925e => [MiniMaridiaBeybladeTurtle_Func7(game)];
    }
}

pub unsafe fn MaridiaBeybladeTurtle_Main(game: &mut Game) {
    let index = game.cur_enemy_index();
    let preinstruction_offset =
        0xf78 + usize::from(index) + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, var_A);
    let preinstruction = u16::from_le_bytes([
        game.g_ram[preinstruction_offset],
        game.g_ram[preinstruction_offset + 1],
    ]);
    let live_index = game.cur_enemy_index();
    CallMaridiaBeybladeTurtleFunc(0xa20000 | u32::from(preinstruction), live_index, game);
}
pub fn MaridiaBeybladeTurtle_Func1(k: u16, game: &mut Game) {
    let base = 0xf78 + usize::from(k);
    let palette_field = core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, base)
        + core::mem::offset_of!(EnemyBase, palette_index);
    let field = base + palette_field;
    let palette = [game.g_ram[field], game.g_ram[field + 1]];
    // Child offsets remain wide even when k is near the end of a word.
    for child in [64, 128, 192, 256] {
        let field = base + child + palette_field;
        game.g_ram[field..field + 2].copy_from_slice(&palette);
    }
    let tiles_field = core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, base)
        + core::mem::offset_of!(EnemyBase, vram_tiles_index);
    let field = base + tiles_field;
    let tiles = [game.g_ram[field], game.g_ram[field + 1]];
    for child in [64, 128, 192, 256] {
        let field = base + child + tiles_field;
        game.g_ram[field..field + 2].copy_from_slice(&tiles);
    }
    for child in [64, 128, 192, 256] {
        let field = base + child + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, var_B);
        game.g_ram[field..field + 2].copy_from_slice(&k.to_le_bytes());
    }
    let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, var_A);
    game.g_ram[field..field + 2].copy_from_slice(&0x8e0au16.to_le_bytes());
}
pub fn MaridiaBeybladeTurtle_Func2(k: u16, game: &mut Game) {
    let base = 0xf78 + usize::from(k);
    let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, var_F);
    let active = u16::from_le_bytes([game.g_ram[field], game.g_ram[field + 1]]);
    if active != 0 {
        game.enemy_coordinates(k).set_y_height(0);
        let delta = game
            .enemy_coordinates(k)
            .x_pos()
            .wrapping_sub(game.samus_x_pos());
        let magnitude = if delta & 0x8000 != 0 {
            delta.wrapping_neg()
        } else {
            delta
        };
        if magnitude < 24 {
            let table_index = if delta & 0x8000 != 0 {
                magnitude + 24
            } else {
                magnitude
            };
            let index = game.cur_enemy_index();
            let radius = g_word_A28E80[usize::from(table_index)].wrapping_neg() as u16;
            game.enemy_coordinates(k).set_y_height(radius);
            let properties = game.enemy_instructions(k).properties();
            game.enemy_instructions(k)
                .set_properties(properties | 0x8000);
            if CheckIfEnemyTouchesSamus(index, game) != 0 {
                let top = game
                    .enemy_coordinates(k)
                    .y_pos()
                    .wrapping_sub(game.enemy_coordinates(k).y_height());
                let samus_radius = game.samus_y_radius();
                let samus_y = game.samus_y_pos();
                if (samus_radius.wrapping_add(samus_y).wrapping_sub(top) as i16) >= 0 {
                    let displacement = game.extra_samus_y_displacement();
                    let samus_radius = game.samus_y_radius();
                    let samus_y = game.samus_y_pos();
                    game.set_extra_samus_y_displacement(
                        displacement
                            .wrapping_add(top.wrapping_sub(samus_radius.wrapping_add(samus_y))),
                    );
                }
            }
        }
    } else {
        let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, var_A);
        game.g_ram[field..field + 2].copy_from_slice(&0x8ee0u16.to_le_bytes());
        let properties = game.enemy_instructions(k).properties();
        game.enemy_instructions(k)
            .set_properties(properties & !(kEnemyProps_Tangible as u16));
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func3(k: u16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    if (*ram_ptr!(game; 0x5b5) & 1) == 0 {
        if CheckIfEnemyTouchesSamus(k, game) != 0 {
            word!(game.extra_samus_x_displacement, wrapping_sub(1));
        }
        let E = Get_MaridiaBeybladeTurtle(k as i32, game);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        (*E).base.y_height = 16;
        if ((*E).base.y_pos & 1) != 0 {
            (*E).base.x_pos = (*E).base.x_pos.wrapping_sub(1);
        } else {
            (*E).base.x_pos = (*E).base.x_pos.wrapping_add(1);
        }
        if Enemy_MoveRight_IgnoreSlopes(k, (1i16 as i32) << 16, game) == 0 {
            enemy_instruction! {[(*E).base]; instruction = addr_kMaridiaBeybladeTurtle_Ilist_8C4A as u16, timer = 1}
            (*E).var_A = addr_locret_A28E09 as u16;
        }
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func5(k: u16, game: &mut Game) {
    let mut v1: u16 = addr_kMaridiaBeybladeTurtle_Ilist_8C1C as u16;
    let E = Get_MaridiaBeybladeTurtle(k as i32, game);
    if (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0 {
        v1 = addr_kMaridiaBeybladeTurtle_Ilist_8D00 as u16;
    }
    enemy_instruction! {[(*E).base]; instruction = v1, timer = 1}
    (*E).var_A = addr_locret_A28E09 as u16;
}
pub unsafe fn MaridiaBeybladeTurtle_Func6(k: u16, game: &mut Game) {
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        let E = Get_MaridiaTurtle(k as i32, game);
        (*E).var_A = ((0xa29083i32) as u16);
        words!(
            game.extra_samus_x_displacement,
            extra_samus_x_subdisplacement += -(words!(i32, (*E), var_E, var_03)) as u32
        );
        if word!(game.extra_samus_x_displacement) as i32 + 16 & 0x8000 != 0 {
            word!(game.extra_samus_x_displacement = -16i32 as u16);
        }
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func7(k: u16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    if Enemy_MoveDown(k, (-1i16 as i32) << 16, game) == 0 {
        if CheckIfEnemyTouchesSamus(k, game) != 0 {
            word!(game.extra_samus_y_displacement, wrapping_sub(1));
        }
        let E = Get_MaridiaTurtle(k as i32, game);
        let v2: u16 = ((*E).var_00).wrapping_sub(1);
        (*E).var_00 = v2;
        if v2 == 0 {
            let mut v3: u16 = 0;
            if (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0 {
                v3 = 4;
            }
            (*E).var_01 = *((rom_ptr!(game, u16; 0xa28d56) as *mut u8).offset(v3 as i32 as isize)
                as *mut u16);
            (*E).var_02 = *((rom_ptr!(game, u16; 0xa28d58) as *mut u8).offset(v3 as i32 as isize)
                as *mut u16);
            set_fields! {[(*E)], 0; var_E, var_03}
            (*E).var_A = ((0xa28febi32) as u16);
        }
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func8(k: u16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    let E = Get_MaridiaTurtle(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(k, (words!((*E), var_E, var_03)) as i32, game) != 0 {
        words!(
            (*E),
            var_E,
            var_03 = -(words!(i32, (*E), var_E, var_03)) as u32
        );
        words!(
            (*E),
            var_02,
            var_01 = -(words!(i32, (*E), var_02, var_01)) as u32
        );
        word!(game.earthquake_type = 0);
        word!(game.earthquake_timer = 16);
        QueueSfx2_Max6(0x1b, game);
    } else {
        MaridiaBeybladeTurtle_Func6(k, game);
        words!((*E), var_E, var_03 += words!((*E), var_02, var_01));
        if ((abs16((*E).var_E)) as i16).wrapping_sub((g_word_A28D5E) as i16) >= 0 {
            (*E).var_E = (if ((*E).var_E & 0x8000) != 0 {
                -(g_word_A28D5E as i32)
            } else {
                g_word_A28D5E as i32
            }) as u16;
        }
    };
}
pub unsafe fn MaridiaBeybladeTurtle_Func9(k: u16, game: &mut Game) {
    let E = Get_MaridiaTurtle(k as i32, game);
    MaridiaBeybladeTurtle_Func4(game);
    let mut v2: u16 = if (((*E).base.y_pos) as i16).wrapping_sub((g_word_A28D60) as i16) < 0 {
        (*E).var_00 = g_word_A28D64;
        ((0xa290cci32) as u16)
    } else {
        if CheckIfEnemyTouchesSamus(k, game) != 0 {
            (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(g_word_A28D62);
            word!(game.extra_samus_y_displacement, wrapping_sub(g_word_A28D62));
            return;
        }
        ((0xa290e1i32) as u16)
    };
    (*E).var_A = v2;
    set_fields! {[(*E)], 0; var_07, var_04}
}
pub unsafe fn MaridiaBeybladeTurtle_Func10(k: u16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    let E = Get_MaridiaTurtle(k as i32, game);
    let v2: u16 = ((*E).var_00).wrapping_sub(1);
    (*E).var_00 = v2;
    if v2 == 0 {
        (*E).var_A = ((0xa290e1i32) as u16);
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func11(k: u16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    let E = Get_MaridiaTurtle(k as i32, game);
    if (((*Get_MaridiaTurtle(0i32, game)).var_04) as i16).wrapping_sub((g_word_A28D66) as i16) < 0 {
        words!((*E), var_04, var_07 += 0x2000);
    }
    if Enemy_MoveDown(k, ((*E).var_04 as i16 as i32) << 16, game) != 0 {
        enemy_instruction! {[(*E).base]; instruction = (if ((*E).var_E & 0x8000) == 0 {
            addr_kMaridiaBeybladeTurtle_Ilist_8D28 as i32
        } else {
            addr_kMaridiaBeybladeTurtle_Ilist_8C4A as i32
        }) as u16, timer = 1}
        (*E).var_A = addr_locret_A28E09 as u16;
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Main(game: &mut Game) {
    let index = game.cur_enemy_index();
    let base = 0xf78 + usize::from(index);
    let parent_offset = base + core::mem::offset_of!(Enemy_MiniMaridiaTurtle, var_B);
    let parent = u16::from_le_bytes([game.g_ram[parent_offset], game.g_ram[parent_offset + 1]]);
    let parent_trigger_offset =
        0xf78 + usize::from(parent) + core::mem::offset_of!(Enemy_MiniMaridiaTurtle, var_06);
    game.g_ram[parent_trigger_offset..parent_trigger_offset + 2]
        .copy_from_slice(&0u16.to_le_bytes());
    let preinstruction_offset = base + core::mem::offset_of!(Enemy_MiniMaridiaTurtle, var_A);
    let preinstruction = u16::from_le_bytes([
        game.g_ram[preinstruction_offset],
        game.g_ram[preinstruction_offset + 1],
    ]);
    let live_index = game.cur_enemy_index();
    CallMaridiaBeybladeTurtleFunc(0xa20000 | u32::from(preinstruction), live_index, game);
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func1(k: u16, game: &mut Game) {
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        let E = Get_MiniMaridiaTurtle(k as i32, game);
        (*E).var_A = ((0xa2916ei32) as u16);
        (*E).var_05 = 4;
        let mut v2: u16 = addr_kMaridiaBeybladeTurtle_Ilist_8C30 as u16;
        if ((*E).var_E & 0x8000) == 0 {
            v2 = addr_kMaridiaBeybladeTurtle_Ilist_8D14 as u16;
        }
        enemy_instruction! {[(*E).base]; instruction = v2, timer = 1}
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func2(k: u16, game: &mut Game) {
    let E = Get_MiniMaridiaTurtle(k as i32, game);
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        (*E).var_05 = 4;
    } else {
        let v2: u16 = ((*E).var_05).wrapping_sub(1);
        (*E).var_05 = v2;
        if v2 == 0 {
            (*E).var_A = ((0xa29198i32) as u16);
            (*E).var_00 = 60;
        }
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func3(k: u16, game: &mut Game) {
    let E = Get_MiniMaridiaTurtle(k as i32, game);
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        (*E).var_A = ((0xa291f8i32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kMaridiaBeybladeTurtle_Ilist_8BD2 as u16, timer = 1}
        (*E).var_04 = 1;
        let mut v2: u16 = g_word_A28D68;
        if (*ram_ptr!(game; 0xa1e) & 0xf) == 8 {
            v2 = g_word_A28D6A;
        }
        (*E).var_E = v2;
    } else {
        let v4: u16 = ((*E).var_00).wrapping_sub(1);
        (*E).var_00 = v4;
        if v4 == 0 {
            let mut v5: u16 = addr_kMaridiaBeybladeTurtle_Ilist_8B80 as u16;
            if ((*E).var_E & 0x8000) == 0 {
                v5 = addr_kMaridiaBeybladeTurtle_Ilist_8C72 as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v5, timer = 1}
            (*E).var_A = ((0xa29142i32) as u16);
        }
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func4(k: u16, game: &mut Game) {
    let E = Get_MiniMaridiaTurtle(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(k, (words!(((*E).var_E, 0u16))) as i32, game) != 0 {
        (*E).var_E = -((*E).var_E as i32) as u16;
    } else {
        Enemy_MoveDown(k, ((*E).var_04 as i16 as i32) << 16, game);
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func6(k: u16, game: &mut Game) {
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        let mut v1: u16 = addr_kMaridiaBeybladeTurtle_Ilist_8B80 as u16;
        let E = Get_MiniMaridiaBeybladeTurtle(k as i32, game);
        if ((*E).var_E & 0x8000) == 0 {
            v1 = addr_kMaridiaBeybladeTurtle_Ilist_8C72 as u16;
        }
        enemy_instruction! {[(*E).base]; instruction = v1, timer = 1}
        (*E).var_A = ((0xa29142i32) as u16);
    } else {
        MiniMaridiaBeybladeTurtle_Func4(k, game);
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func7(game: &mut Game) {
    let E = Get_MiniMaridiaTurtle(word!(game.cur_enemy_index) as i32, game);
    (*Get_MiniMaridiaTurtle((*E).var_B as i32, game)).var_06 = (*E).base.y_height;
    if CheckIfEnemyTouchesSamus(word!(game.cur_enemy_index), game) == 0 {
        (*Get_MiniMaridiaTurtle(word!(game.cur_enemy_index) as i32, game)).var_A =
            ((0xa29142i32) as u16);
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Touch(game: &mut Game) {
    let E = Get_MiniMaridiaTurtle(word!(game.cur_enemy_index) as i32, game);
    if ((*E).base.properties & 0x8000) == 0 {
        Enemy_NormalTouchAI_A2(game);
        (*E).var_A = ((0xa290e1i32) as u16);
        (*E).var_04 = 2;
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Touch(game: &mut Game) {
    let mut v2: i16 = 0;
    let E = Get_MiniMaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_A != 0x925e {
        if ((*E).var_E & 0x8000) != 0 {
            (*E).base.current_instruction = addr_kMaridiaBeybladeTurtle_Ilist_8C72 as u16;
            v2 = 1;
        } else {
            (*E).base.current_instruction = addr_kMaridiaBeybladeTurtle_Ilist_8B80 as u16;
            v2 = -1i16;
        }
        (*E).var_E = v2 as u16;
        (*E).base.instruction_timer = 1;
        let mut v3: u16 =
            if (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) >= 0 {
                ((*E).base.x_width as i32
                    + word!(game.samus_x_radius) as i32
                    + word!(game.samus_x_pos) as i32) as u16
            } else {
                (((((word!(game.samus_x_pos)) as u32)
                    .wrapping_sub((word!(game.samus_x_radius)) as u32))
                    << 16i32
                    | word!(game.samus_x_pos) as u32)
                    .wrapping_sub(words!(((*E).base.x_width, word!(game.samus_x_radius))))
                    >> 16) as u16
            };
        (*E).base.x_pos = v3;
        (*E).var_A = ((0xa29142i32) as u16);
        Enemy_MoveRight_IgnoreSlopes(
            word!(game.cur_enemy_index),
            (words!(((*E).var_E, 0u16))) as i32,
            game,
        );
        MaridiaBeybladeTurtle_Func21(game);
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func21(game: &mut Game) {
    let mbte_var_B: u16 =
        (*Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game)).var_B;
    let E = Get_MaridiaBeybladeTurtle(mbte_var_B as i32, game);
    if (*E).var_F != 0 {
        (*E).var_F = (*E).var_F.wrapping_sub(1);
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    MaridiaBeybladeTurtle_Func21(game);
}
pub unsafe fn MaridiaBeybladeTurtle_Func4(game: &mut Game) {
    let E = Get_MaridiaBeybladeTurtle(0, game);
    let r18: u16 = ((*E).base.x_pos as i32 - (*E).base.x_width as i32 - 8i32) as u16;
    let r20: u16 = ((*E).base.x_width as i32 + (*E).base.x_pos as i32 + 8) as u16;
    let r22: u16 = ((*E).base.y_pos as i32 - (*E).base.y_height as i32 + 4) as u16;
    let r24: u16 = ((*E).base.y_height as i32 + (*E).base.y_pos as i32 - 4i32) as u16;
    if word!(game.samus_x_pos) as i32 - word!(game.samus_x_radius) as i32 - 1i32 - r20 as i32
        & 0x8000
        != 0
        && word!(game.samus_x_radius) as i32 + word!(game.samus_x_pos) as i32 - r18 as i32 & 0x8000
            == 0
        && word!(game.samus_y_pos) as i32 - word!(game.samus_y_radius) as i32 + 1 - r24 as i32
            & 0x8000
            != 0
        && word!(game.samus_y_radius) as i32 + word!(game.samus_y_pos) as i32 - r22 as i32 & 0x8000
            == 0
        && word!(game.samus_invincibility_timer) == 0
    {
        NormalEnemyTouchAi(game);
    }
}
crate::instruction_callbacks! { pub unsafe fn (
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16;
    MaridiaBeybladeTurtle_Instr_9381 [jp] {
        let mut R48: u16 = 0;
        if CheckIfEnemyTouchesSamus(word!(game.cur_enemy_index), game) != 0 {
            word!(
                game.extra_samus_x_displacement,
                wrapping_add(
                    (*Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game)).var_E
                )
            );
            R48 = 1;
        }
        let E = Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game);
        crate::update_place! { previous R50: u16 = (*E).base.y_pos => (*E).var_D; }
        Enemy_MoveRight_IgnoreSlopes(
            word!(game.cur_enemy_index),
            ((*E).var_E as i16 as i32) << 16,
            game,
        );
        if (*Get_MaridiaBeybladeTurtle((*E).var_B as i32, game)).var_A == 0x8e0a {
            let x_pos: u16 = (*Get_MaridiaBeybladeTurtle((*E).var_B as i32, game))
                .base
                .x_pos;
            let xd: u16 = (x_pos).wrapping_sub((*E).base.x_pos);
            let mut v7: u16 = abs16(xd);
            let mut v8: u16 = if v7 as i32 - 24i32 & 0x8000 != 0 {
                if xd & 0x8000 != 0 {
                    v7 = (v7).wrapping_add(24);
                }
                g_word_A28E80[v7 as usize] as u16
            } else {
                1
            };
            Enemy_MoveDown(
                word!(game.cur_enemy_index),
                (v8 as i16 as i32) << 16,
                game,
            );
            if R48 != 0 {
                word!(game.extra_samus_y_displacement, |fresh1| (fresh1 as i32
                    + ((*Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game))
                        .base
                        .y_pos as i32
                        - R50 as i32))
                    as u16);
            }
        }
    };
    MaridiaBeybladeTurtle_Instr_9412 [addr_kMaridiaBeybladeTurtle_Ilist_8C72 as i32 as usize as *const u16] {
        let mut v4: i16 = 0;
        let E = Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game);
        let xd: u16 = ((*E).var_C).wrapping_sub((*E).base.x_pos);
        if ((abs16(xd)) as i16).wrapping_sub((g_word_A28D50) as i16) >= 0 {
            if xd & 0x8000 == 0 {
                v4 = 1;
            } else {
                v4 = -1i16;
            }
            (*E).var_E = v4 as u16;
        }
        if ((*Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game)).var_E & 0x8000)
            != 0
        {
            return addr_kMaridiaBeybladeTurtle_Ilist_8B80 as i32 as usize as *const u16;
        }
    };
    MaridiaBeybladeTurtle_Instr_9447 [jp] {
        (*Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game)).var_A =
            ((0xa28f3fi32) as u16);
    };
    MaridiaBeybladeTurtle_Instr_9451 [addr_kMaridiaBeybladeTurtle_Ilist_8C02 as i32 as usize as *const u16] {
        let E = Get_MaridiaTurtle(word!(game.cur_enemy_index) as i32, game);
        (*E).var_A = ((0xa28f8di32) as u16);
        (*E).var_E = -1i32 as u16;
        (*E).var_00 = 16;
    };
    MaridiaBeybladeTurtle_Instr_946B [addr_kMaridiaBeybladeTurtle_Ilist_8C02 as i32 as usize as *const u16] {
        let E = Get_MaridiaTurtle(word!(game.cur_enemy_index) as i32, game);
        (*E).var_A = ((0xa28f8di32) as u16);
        (*E).var_E = 1;
        (*E).var_00 = 16;
    };
    MaridiaBeybladeTurtle_Instr_9485 [jp] {
        if CheckIfEnemyTouchesSamus(word!(game.cur_enemy_index), game) != 0 {
            if ((*Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game)).var_E
                & 0x8000)
                == 0
            {
                return addr_kMaridiaBeybladeTurtle_Ilist_8D40 as i32 as usize as *const u16;
            } else {
                return addr_kMaridiaBeybladeTurtle_Ilist_8C62 as i32 as usize as *const u16;
            }
        }
    };
    MaridiaBeybladeTurtle_Instr_94A1 [addr_kMaridiaBeybladeTurtle_Ilist_8B80 as i32 as usize as *const u16] {
        let E = Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game);
        if CheckIfEnemyTouchesSamus(word!(game.cur_enemy_index), game) != 0 {
            (*E).var_A = ((0xa2925ei32) as u16);
        } else {
            (*E).var_A = ((0xa29142i32) as u16);
        }
        if ((*E).var_E & 0x8000) == 0 {
            return addr_kMaridiaBeybladeTurtle_Ilist_8C72 as i32 as usize as *const u16;
        }
    };
    MaridiaBeybladeTurtle_Instr_94C7 [jp] {
        let E = Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game);
        (*E).var_A = ((0xa29239i32) as u16);
    };
    MaridiaBeybladeTurtle_Instr_94D1 [jp] { QueueSfx2_Max6(0x3a, game); };
}
pub unsafe fn sub_A29A6C(a: u16, game: &mut Game) {
    let v1 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    enemy_instruction! {[(*v1)]; instruction = a, timer = 1}
    (*v1).timer = 0;
}
crate::game::address_dispatch! {
    pub unsafe fn CallThinHoppingBlobs(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10656613 => [{
                    ThinHoppingBlobs_Func6(word!(game.cur_enemy_index), game);
                }];
        10656641 => ThinHoppingBlobs_Func7;
    }
}
pub unsafe fn ThinHoppingBlobs_Func1(k: u16, game: &mut Game) {
    let E = Get_ThinHoppingBlobs(k as i32, game);
    if (*E).var_00 as i32 - 3i32 & 0x8000 != 0 {
        TwinHoppingBlobs_Func2(game);
    }
    ThinHoppingBlobs_Func3(game);
    ThinHoppingBlobs_Func5(game);
}
pub unsafe fn ThinHoppingBlobs_Func4(game: &mut Game) -> u16 {
    NextRandom(game);
    ((*Get_ThinHoppingBlobs(word!(game.cur_enemy_index) as i32, game))
        .base
        .frame_counter as i32
        + word!(game.random_number) as i32
        & 7) as u16
}
pub unsafe fn ThinHoppingBlobs_Func6(k: u16, game: &mut Game) {
    let E = Get_ThinHoppingBlobs(k as i32, game);
    (*E).var_C = (*E).var_C.wrapping_sub(1);
    if ((*E).var_C & 0x8000) != 0 {
        (*E).var_D = ((0xa29b81i32) as u16);
        (*E).var_C = (*E).parameter_1;
        (*E).var_01 = 1;
        ThinHoppingBlobs_Func1(k, game);
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallThinHoppingBlobsVarE(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10657035 => ThinHoppingBlobs_Func11; 10657067 => ThinHoppingBlobs_Func12;
        10657099 => ThinHoppingBlobs_Func13; 10657131 => ThinHoppingBlobs_Func14;
        10657176 => ThinHoppingBlobs_Func15; 10657229 => ThinHoppingBlobs_Func16;
    }
}
pub unsafe fn ThinHoppingBlobs_Func11(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let E = Get_ThinHoppingBlobs(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_04 != 0 || (*E).var_01 == 0 {
        (*E).var_01 = 0;
        sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99D5 as u16, game);
    }
}
pub unsafe fn ThinHoppingBlobs_Func12(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let E = Get_ThinHoppingBlobs(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_04 != 0 || (*E).var_01 == 0 {
        (*E).var_01 = 0;
        sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99C1 as u16, game);
    }
}
pub unsafe fn ThinHoppingBlobs_Func13(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let E = Get_ThinHoppingBlobs(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_04 != 0 || (*E).var_01 == 0 {
        (*E).var_01 = 0;
        sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99AD as u16, game);
    }
}
pub unsafe fn ThinHoppingBlobs_Func14(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let E = Get_ThinHoppingBlobs(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_04 == 0 {
        if (*E).var_01 != 0 {
            return;
        }
        (*E).var_00 = 0;
        (*E).var_D = ((0xa29b65i32) as u16);
    }
    (*E).var_01 = 0;
    sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99D5 as u16, game);
}
crate::prefixed_callbacks! {
    pub unsafe fn (game: &mut Game);
    @prefix { let E = Get_ThinHoppingBlobs(word!(game.cur_enemy_index) as i32, game); };
    ThinHoppingBlobs_Init [] {
        (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as u16;
        (*E).var_A = 0;
        sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99AD as u16, game);
        (*E).var_00 = 0;
        (*E).var_D = ((0xa29b65i32) as u16);
        (*E).var_C = (*E).parameter_1;
        (*E).var_04 = 0;
    };
    ThinHoppingBlobs_Main [] {
        CallThinHoppingBlobs((((*E).var_D) as u32) | (0xa20000u32), game);
    };
    TwinHoppingBlobs_Func2 [] {
        (*E).var_00 = IsSamusWithinEnemy_X(word!(game.cur_enemy_index), (*E).parameter_2, game);
    };
    ThinHoppingBlobs_Func3 [] {
        (*E).var_02 = 1;
        if (GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) == 0 {
            (*E).var_02 = 0;
        }
        if (*E).var_04 != 0 {
            (*E).var_02 = (*E).var_05;
        }
        (*E).var_04 = 0;
        let mut R28: u16 = ThinHoppingBlobs_Func4(game);
        let mut thbs_var_00: u16 = (*E).var_00;
        if thbs_var_00 as i32 - 3i32 & 0x8000 != 0 {
            if thbs_var_00 == 0 {
                R28 = (R28) & (1u16);
            }
            thbs_var_00 = R28;
            if R28 as i32 - 2i32 & 0x8000 == 0 {
                thbs_var_00 = 2;
            }
        }
        let v3: u16 = (8u16).wrapping_mul(thbs_var_00);
        (*E).var_F = v3;
        (*E).var_E =
            *((rom_ptr!(game, u16; 0xa29a0d) as *mut u8).offset(v3 as i32 as isize) as *mut u16);
    };
    ThinHoppingBlobs_Func5 [] {
        let mut thbs_var_F: u16 = (*E).var_F;
        let mut r22: u16 = 0;
        let mut r24: u16 = 0;
        loop {
            r22 = (r22).wrapping_add(
                *((rom_ptr!(game, u16; 0xa29a09) as *mut u8).offset(thbs_var_F as i32 as isize)
                    as *mut u16),
            );
            r24 = (r24).wrapping_add(
                *((rom_ptr!(game, u16; 0xa2838f) as *mut u8)
                    .offset((8i32 * ((r22 as i32 & 0xff00i32) >> 8i32)) as isize)
                    .offset(1) as *mut u16),
            );
            thbs_var_F = (*E).var_F;
            let mut v2: u16 = u16::swap_bytes(
                *((rom_ptr!(game, u16; 0xa29a07) as *mut u8).offset(thbs_var_F as i32 as isize)
                    as *mut u16),
            );
            if (v2 as i32 - r24 as i32 & 0x8000 != 0) {
                break;
            }
        }
        (*E).var_B = r22;
        (*E).var_03 = 0;
        let v3: u16 = (r22 >> 1);
        (*E).var_07 = (r22 >> 1);
        (*E).var_06 = ((*E).var_07 as i32 + (v3 as i32 >> 1)) as u16;
    };
    ThinHoppingBlobs_Func7 [] {
        CallThinHoppingBlobsVarE((((*E).var_E) as u32) | (0xa20000u32), game);
    };
    ThinHoppingBlobs_Func8 [] {
        let mut thbs_var_B: u16 = (*E).var_B;
        if thbs_var_B as i32 - 0x4000i32 & 0x8000 == 0 {
            thbs_var_B = 0x4000;
        }
        let mut v6: u16 = (8 * (thbs_var_B as i32 >> 8)) as u16;
        if (*E).var_03 == 0 {
            v6 = (v6).wrapping_add(4);
        }
        if Enemy_MoveDown(
            word!(game.cur_enemy_index),
            *(rom_ptr!(game, u32; 0xa0cbc7)).offset((v6 >> 2) as isize) as i32,
            game,
        ) != 0
        {
            if (*E).var_03 != 0 {
                (*E).var_D = ((0xa29b65i32) as u16);
            } else {
                (*E).var_04 = 1;
                (*E).var_05 = ((*E).var_02) ^ (1u16);
                (*E).var_00 = 4;
                (*E).var_E = ((0xa29d98i32) as u16);
            }
            (*E).var_01 = 0;
        } else {
            let thbs_var_F: u16 = (*E).var_F;
            if (*E).var_03 != 0 {
                ThinHoppingBlobs_Func10(game);
                (*E).var_B = ((*E).var_B).wrapping_add(
                    *((rom_ptr!(game, u16; 0xa29a0b) as *mut u8).offset(thbs_var_F as i32 as isize)
                        as *mut u16),
                );
            } else {
                ThinHoppingBlobs_Func9(game);
                (*E).var_B = ((*E).var_B).wrapping_sub(
                    *((rom_ptr!(game, u16; 0xa29a0b) as *mut u8).offset(thbs_var_F as i32 as isize)
                        as *mut u16),
                );
            }
            if (*E).var_B & 0x8000 != 0 {
                (*E).var_03 = 1;
                (*E).var_B = 0;
            }
            let mut R20: u16 = ((*((rom_ptr!(game, u16; 0xa29a09) as *mut u8)
                .offset((*E).var_F as i32 as isize) as *mut u16) as i32
                & 0xff00)
                >> 8) as u16;
            if (*E).var_02 != 0 {
                R20 = -(R20 as i32) as u16;
            }
            if Enemy_MoveRight_IgnoreSlopes(
                word!(game.cur_enemy_index),
                (R20 as i16 as i32) << 16,
                game,
            ) != 0
            {
                (*E).var_04 = 1;
                (*E).var_05 = ((*E).var_02) ^ (1u16);
                (*E).var_01 = 0;
                (*E).var_00 = 4;
                (*E).var_E = ((0xa29d98i32) as u16);
            }
        };
    };
    ThinHoppingBlobs_Func9 [] {
        let mut v2: u16 = if (*E).var_02 != 0 {
            let thbs_var_B: u16 = (*E).var_B;
            if ((thbs_var_B) as i16).wrapping_sub(((*E).var_06) as i16) >= 0 {
                addr_kThinHoppingBlobs_Ilist_9A01 as u16
            } else if ((thbs_var_B) as i16).wrapping_sub(((*E).var_07) as i16) >= 0 {
                addr_kThinHoppingBlobs_Ilist_99FB as u16
            } else {
                addr_kThinHoppingBlobs_Ilist_99F5 as u16
            }
        } else {
            let v1: u16 = (*E).var_B;
            if ((v1) as i16).wrapping_sub(((*E).var_06) as i16) >= 0 {
                addr_kThinHoppingBlobs_Ilist_99E9 as u16
            } else if ((v1) as i16).wrapping_sub(((*E).var_07) as i16) >= 0 {
                addr_kThinHoppingBlobs_Ilist_99EF as u16
            } else {
                addr_kThinHoppingBlobs_Ilist_99F5 as u16
            }
        };
        sub_A29A6C(v2, game);
    };
    ThinHoppingBlobs_Func10 [] {
        let mut v2: u16 = if (*E).var_02 != 0 {
            let thbs_var_B: u16 = (*E).var_B;
            if ((thbs_var_B) as i16).wrapping_sub(((*E).var_07) as i16) < 0 {
                addr_kThinHoppingBlobs_Ilist_99F5 as u16
            } else if ((thbs_var_B) as i16).wrapping_sub(((*E).var_06) as i16) < 0 {
                addr_kThinHoppingBlobs_Ilist_99EF as u16
            } else {
                addr_kThinHoppingBlobs_Ilist_99E9 as u16
            }
        } else {
            let v1: u16 = (*E).var_B;
            if ((v1) as i16).wrapping_sub(((*E).var_07) as i16) < 0 {
                addr_kThinHoppingBlobs_Ilist_99F5 as u16
            } else if ((v1) as i16).wrapping_sub(((*E).var_06) as i16) < 0 {
                addr_kThinHoppingBlobs_Ilist_99FB as u16
            } else {
                addr_kThinHoppingBlobs_Ilist_9A01 as u16
            }
        };
        sub_A29A6C(v2, game);
    };
    ThinHoppingBlobs_Func15 [] {
        let thbs_var_F: u16 = (*E).var_F;
        let R20: u16 = ((*((rom_ptr!(game, u16; 0xa29a0b) as *mut u8).offset(thbs_var_F as i32 as isize)
            as *mut u16) as i32
            & 0xff00)
            >> 8) as u16;
        let R18: u16 = ((byte!(@pointer rom_ptr!(game, u16; 0xa29a0b); thbs_var_F as i32 as isize)
            as i32)
            << 8) as u16;
        if Enemy_MoveDown(
            word!(game.cur_enemy_index),
            (words!((R20, R18))) as i32,
            game,
        ) != 0
        {
            let R28: u16 = ThinHoppingBlobs_Func4(game);
            (*E).var_00 = ((R28 as i32 & 1) + 5) as u16;
            (*E).var_D = ((0xa29b65i32) as u16);
        }
    };
}
pub unsafe fn ThinHoppingBlobs_Func16(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let E = Get_ThinHoppingBlobs(word!(game.cur_enemy_index) as i32, game);
    if (*E).var_01 == 0 {
        (*E).var_01 = 0;
        (*E).var_00 = 3;
        (*E).var_D = ((0xa29b65i32) as u16);
        sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99D5 as u16, game);
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    SpikeShootingPlant_Instr_9F2A [jp] { QueueSfx2_Max6(0x34, game); };
}
pub unsafe fn SpikeShootingPlant_2(k: u16, game: &mut Game) {
    let E = Get_SpikeShootingPlant(word!(game.cur_enemy_index) as i32, game);
    words!((*E).base, x_pos, x_subpos += words!((*E), var_D, var_C));
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_00) as i16) < 0 {
        (*E).var_F = ((0xa29feci32) as u16);
        (*E).var_E = 1;
    }
    SpikeShootingPlant_4(game);
}
pub unsafe fn SpikeShootingPlant_3(k: u16, game: &mut Game) {
    let E = Get_SpikeShootingPlant(word!(game.cur_enemy_index) as i32, game);
    words!((*E).base, x_pos, x_subpos += words!((*E), var_B, var_A));
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_01) as i16) >= 0 {
        (*E).var_F = ((0xa29fbai32) as u16);
        (*E).var_E = 0;
    }
    SpikeShootingPlant_4(game);
}
pub unsafe fn SpikeShootingPlant_4(game: &mut Game) {
    NextRandom(game);
    let E = Get_SpikeShootingPlant(word!(game.cur_enemy_index) as i32, game);
    if ((byte!((*E).base.frame_counter; 0)).wrapping_add((word!(game.random_number)) as u8)) as i32
        - 3i32
        & 0x8000
        != 0
    {
        (*E).var_F = ((0xa2a01bi32) as u16);
        if byte!((*E).parameter_1; 1) != 0 {
            SpikeShootingPlant_6(game);
        } else {
            SpikeShootingPlant_8(game);
        }
    }
}
crate::prefixed_callbacks! {
    pub unsafe fn (game: &mut Game);
    @prefix { let E = Get_SpikeShootingPlant(word!(game.cur_enemy_index) as i32, game); };
    SpikeShootingPlant_Init [] {
        (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as u16;
        if byte!((*E).parameter_1; 1) != 0 {
            SpikeShootingPlant_5(game);
        } else {
            SpikeShootingPlant_7(game);
        }
        let sspt_parameter_1_low: u16 = byte!((*E).parameter_1; 0) as u16;
        (*E).var_E = sspt_parameter_1_low;
        (*E).var_F = crate::rtl::rom_table_word!(game, 0xa29f42, sspt_parameter_1_low as isize);
        let v2: i32 = byte!((*E).parameter_2; 0) as i32;
        (*E).var_01 =
            (crate::rtl::rom_table_word!(game, 0xa29f36, v2 as isize)).wrapping_add((*E).base.x_pos);
        (*E).var_00 =
            ((*E).base.x_pos).wrapping_sub(crate::rtl::rom_table_word!(game, 0xa29f36, v2 as isize));
        let v3: i32 = 8 * byte!((*E).parameter_2; 1) as i32 >> 1;
        (*E).var_B = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
        (*E).var_A = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1) as isize);
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2) as isize);
        (*E).var_C = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3) as isize);
    };
    SpikeShootingPlant_Main [] {
        CallEnemyPreInstr((((*E).var_F) as u32) | (0xa20000u32), game);
    };
    SpikeShootingPlant_5 [] {
        enemy_instruction! {[(*E).base]; instruction = addr_kSpikeShootingPlant_Ilist_9E8A as u16, timer = 1}
        (*E).base.timer = 0;
    };
    SpikeShootingPlant_6 [] {
        enemy_instruction! {[(*E).base]; instruction = addr_kSpikeShootingPlant_Ilist_9EB0 as u16, timer = 1}
        (*E).base.timer = 0;
    };
    SpikeShootingPlant_7 [] {
        enemy_instruction! {[(*E).base]; instruction = addr_kSpikeShootingPlant_Ilist_9EDA as u16, timer = 1}
        (*E).base.timer = 0;
    };
    SpikeShootingPlant_8 [] {
        enemy_instruction! {[(*E).base]; instruction = addr_kSpikeShootingPlant_Ilist_9F00 as u16, timer = 1}
        (*E).base.timer = 0;
    };
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    SpikeShootingPlant_Instr_A095 [jp] {
        let E = Get_SpikeShootingPlant(k as i32, game);
        (*E).var_F = ((0xa29fbai32) as u16);
        if (*E).var_E != 0 {
            (*E).var_F = ((0xa29feci32) as u16);
        }
    };
    SpikeShootingPlant_Instr_A0A7 [jp.offset(1)] {
        SpawnEprojWithGfx(
            *jp,
            word!(game.cur_enemy_index),
            addr_kEproj_SpikeShootingPlantSpikes as u16,
            game,
        );
    };
}
static off_A2A3D3: [Option<Func_V>; 7] = callback_table! { unsafe Func_V;
    MaridiaSpikeyShell_1, MaridiaSpikeyShell_2, MaridiaSpikeyShell_3, MaridiaSpikeyShell_4,
    MaridiaSpikeyShell_5, MaridiaSpikeyShell_6, MaridiaSpikeyShell_7,
};
pub unsafe fn MaridiaSpikeyShell_Main(game: &mut Game) {
    let e = Get_MaridiaSpikeyShell(word!(game.cur_enemy_index) as i32, game);
    off_A2A3D3[((*e).var_E as i16 as i32 + 2i32) as usize].expect("non-null function pointer")(
        game,
    );
}
pub unsafe fn MaridiaSpikeyShell_8(k: u16, game: &mut Game) {
    NextRandom(game);
    let E = Get_MaridiaSpikeyShell(k as i32, game);
    if ((byte!((*E).base.frame_counter; 0)).wrapping_add((word!(game.random_number)) as u8)) as i32
        - 6i32
        & 0x8000
        != 0
    {
        (*E).var_E = 3;
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    MaridiaSpikeyShell_Instr_A56D [jp] { (*Get_MaridiaSpikeyShell(k as i32, game)).var_E = 0; };
    MaridiaSpikeyShell_Instr_A571 [jp] { (*Get_MaridiaSpikeyShell(k as i32, game)).var_E = 1; };
}
crate::prefixed_callbacks! {
    pub unsafe fn (game: &mut Game);
    @prefix { let E = Get_MaridiaSpikeyShell(word!(game.cur_enemy_index) as i32, game); };
    MaridiaSpikeyShell_Init [] {
        (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as u16;
        let mssl_parameter_1_low: u16 = byte!((*E).parameter_1; 0) as u16;
        (*E).var_E = mssl_parameter_1_low;
        off_A2A3D3[(mssl_parameter_1_low as i32 & 1i32) as usize].expect("non-null function pointer")(
            game,
        );
        let v3: i32 = 4 * byte!((*E).parameter_2; 0) as i32;
        (*E).var_B = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
        (*E).var_A = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1) as isize);
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2) as isize);
        (*E).var_C = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3) as isize);
        let v4: i32 = byte!((*E).parameter_2; 1) as i32;
        (*E).var_02 =
            (crate::rtl::rom_table_word!(game, 0xa2a3dd, v4 as isize)).wrapping_add((*E).base.x_pos);
        (*E).var_01 =
            ((*E).base.x_pos).wrapping_sub(crate::rtl::rom_table_word!(game, 0xa2a3dd, v4 as isize));
        (*E).var_00 = crate::rtl::rom_table_word!(
            game,
            0xa2a3ed,
            *(&raw mut (*E).parameter_1 as *mut u8).offset(1) as isize
        );
        (*E).var_F = 0;
        if (*E).var_E == 2 {
            (*E).var_F = 16;
            (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(16);
        }
    };
    MaridiaSpikeyShell_1 [] {
        enemy_instruction! {[(*E).base]; instruction = addr_kMaridiaSpikeyShell_Ilist_A3AB as u16, timer = 1}
        (*E).base.timer = 0;
    };
    MaridiaSpikeyShell_2 [] {
        enemy_instruction! {[(*E).base]; instruction = addr_kMaridiaSpikeyShell_Ilist_A3BD as u16, timer = 1}
        (*E).base.timer = 0;
    };
    MaridiaSpikeyShell_3 [] {
        words!((*E).base, x_pos, x_subpos += words!((*E), var_D, var_C));
        if (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_01) as i16) < 0 {
            (*E).var_E = (*E).var_E.wrapping_sub(1);
        }
        MaridiaSpikeyShell_8(word!(game.cur_enemy_index), game);
    };
    MaridiaSpikeyShell_4 [] {
        words!((*E).base, x_pos, x_subpos += words!((*E), var_B, var_A));
        if (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_02) as i16) >= 0 {
            (*E).var_E = 0;
        }
        MaridiaSpikeyShell_8(word!(game.cur_enemy_index), game);
    };
    MaridiaSpikeyShell_5 [] {
        let v1: u16 = ((*E).var_00).wrapping_sub(1);
        (*E).var_00 = v1;
        if v1 == 0 {
            (*E).var_E = 4;
        }
    };
    MaridiaSpikeyShell_6 [] {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
        (*E).var_F = (*E).var_F.wrapping_add(1);
        if (*E).var_F as i32 - 16i32 & 0x8000 == 0 {
            (*E).var_E = 2;
            (*E).var_00 = crate::rtl::rom_table_word!(
                game,
                0xa2a3ed,
                *(&raw mut (*E).parameter_1 as *mut u8).offset(1) as isize
            );
        }
    };
    MaridiaSpikeyShell_7 [] {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        crate::update_place! { previous fresh29 = (*E).var_F => (*E).var_F.wrapping_sub(1); }
        if fresh29 == 1 {
            (*E).var_E = ((byte!(@pointer ram_ptr!(game, u16; 0x5e5); 1)) as u16) & (1u16);
        }
    };
    MaridiaSpikeyShell_Shot [] {
        if (*E).var_E as i32 - 1i32 & 0x8000 != 0 {
            Enemy_NormalShotAI_A2(game);
        }
    };
}
pub unsafe fn GunshipTop_Init(game: &mut Game) {
    let E = Get_GunshipTop(word!(game.cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties as i32
        | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_Tangible as i32))
        as u16;
    (*E).base.instruction_timer = 1;
    (*E).base.timer = 0;
    (*E).base.current_instruction = addr_kGunshipTop_Ilist_A616 as u16;
    (*E).base.palette_index = 3584;
    if word!(game.game_state) != 40 || word!(game.demo_set) != 0 {
        if word!(game.loading_game_state) as i32 == kLoadingGameState_22_EscapingCeres as i32 {
            (*E).base.y_pos = (word!(game.samus_y_pos)).wrapping_sub(17);
            (*E).var_F = ((0xa2a80ci32) as u16);
        } else {
            let v3: u16 = ((*E).base.y_pos).wrapping_sub(25);
            (*E).base.y_pos = v3;
            (*E).var_E = v3;
            (*E).var_F = ((0xa2a9bdi32) as u16);
        }
    } else {
        word!(game.samus_y_pos = 1138);
        let v5: u16 = ((*E).base.y_pos).wrapping_sub(25);
        (*E).base.y_pos = v5;
        (*E).var_E = v5;
        (*E).var_F = ((0xa2ab60i32) as u16);
        (*Get_GunshipTop(0i32, game)).var_A = 144;
    }
    SpawnPalfxObject(addr_stru_8DE1C0 as u16, game);
    (*Get_GunshipTop(word!(game.cur_enemy_index) as i32, game)).var_D = 1;
    (*Get_GunshipTop(0i32, game)).var_C = 0;
}
pub unsafe fn GunshipBottom_Init(game: &mut Game) {
    let E = Get_GunshipBottom(word!(game.cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties as i32
        | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_Tangible as i32))
        as u16;
    (*E).base.instruction_timer = 1;
    (*E).base.timer = 0;
    if (*E).parameter_2 != 0 {
        (*E).base.current_instruction = addr_kGunshipTop_Ilist_A60E as u16;
    } else {
        (*E).base.current_instruction = addr_kGunshipTop_Ilist_A61C as u16;
    }
    let v2: i32 = word!(game.cur_enemy_index) as i32 >> 1;
    (*E).base.vram_tiles_index = word!(game.enemy_drawing_queue[(v2 + 106)]);
    (*E).base.palette_index = 3584;
    if (*E).parameter_2 != 0 {
        (*E).base.y_pos = (word!(game.enemy_drawing_queue[(v2 + 61)])).wrapping_sub(1);
        if word!(game.game_state) as i32 == kPose_28_FaceL_Crouch as i32
            && word!(game.demo_set) == 0
        {
            enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kGunshipTop_Ilist_A5BE as u16}
        }
    } else if word!(game.loading_game_state) as i32 == kGameState_34_CeresGoesBoom as i32 {
        (*E).base.y_pos = (word!(game.samus_y_pos)).wrapping_add(23);
    } else {
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(15);
        (*E).var_D = 71;
    }
    (*E).var_F = ((0xa2a7d7i32) as u16);
}
pub unsafe fn GunshipTop_Main(game: &mut Game) {
    let E1 = Get_GunshipTop(word!(game.cur_enemy_index) as i32 + 64, game);
    let v2: bool = (*E1).var_D == 1;
    (*E1).var_D = (*E1).var_D.wrapping_sub(1);
    let v3: bool = ((*E1).var_D & 0x8000) != 0;
    if v2 as i32 != 0 || v3 as i32 != 0 {
        QueueSfx2_Max6(0x4d, game);
        (*E1).var_D = 70;
    }
    let E0 = Get_GunshipTop(word!(game.cur_enemy_index) as i32, game);
    if (*E0).var_F as i32 + 0x56be & 0x8000 == 0 && (*E0).var_F as i32 + 0x53e5 & 0x8000 != 0 {
        GunshipTop_1(word!(game.cur_enemy_index), game);
    }
    CallEnemyPreInstr((((*E0).var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn GunshipTop_1(k: u16, game: &mut Game) {
    let E0 = Get_GunshipTop(k as i32, game);
    let v3: bool = (*E0).var_D == 1;
    (*E0).var_D = (*E0).var_D.wrapping_sub(1);
    let v4: bool = ((*E0).var_D & 0x8000) != 0;
    if v3 as i32 != 0 || v4 as i32 != 0 {
        let v5: u16 = (2u16).wrapping_mul((*E0).var_C);
        (*E0).var_D = byte!(@pointer rom_ptr!(game; 0xa2a7cf); v5 as isize) as u16;
        let v6: u16 =
            byte!(@pointer rom_ptr!(game; 0xa2a7cf); (v5 as i32 + 1) as isize) as i8 as u16;
        (*E0).base.y_pos = ((*E0).base.y_pos).wrapping_add(v6);
        let E1 = Get_GunshipTop(k as i32 + 64, game);
        (*E1).base.y_pos = ((*E1).base.y_pos).wrapping_add(v6);
        let E2 = Get_GunshipTop(k as i32 + 128, game);
        (*E2).base.y_pos = ((*E2).base.y_pos).wrapping_add(v6);
        (*E0).var_C = ((*E0).var_C as i32 + 1 & 3) as u16;
    }
}
pub unsafe fn GunshipTop_3(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(k as i32, game);
    if (*E).base.y_pos as i32 - 768i32 & 0x8000 != 0 {
        words!(game.samus_y_pos, samus_y_subpos += 0x48000);
        let v3 = Get_GunshipTop(k as i32 + 128, game);
        words!((*v3).base, y_pos, y_subpos += 0x48000);
        let v5 = Get_GunshipTop(k as i32 + 64, game);
        words!((*v5).base, y_pos, y_subpos += 0x48000);
        words!((*E).base, y_pos, y_subpos += 0x48000);
    } else {
        words!(game.samus_y_pos, samus_y_subpos += 0x28000);
        let v8 = Get_GunshipTop(k as i32 + 128, game);
        words!((*v8).base, y_pos, y_subpos += 0x28000);
        let v10 = Get_GunshipTop(k as i32 + 64, game);
        words!((*v10).base, y_pos, y_subpos += 0x28000);
        words!((*E).base, y_pos, y_subpos += 0x28000);
        if (*E).base.y_pos as i32 - 1119i32 & 0x8000 == 0 {
            (*E).base.y_pos = 1119;
            (*v10).base.y_pos = 1159;
            (*v8).base.y_pos = ((*E).base.y_pos).wrapping_sub(1);
            (*E).var_F = ((0xa2a8d0i32) as u16);
            (*E).var_E = 0;
        }
    };
}
pub unsafe fn GunshipTop_4(k: u16, game: &mut Game) {
    let GunshipTop = Get_GunshipTop(k as i32, game);
    let mut v2: i32 = (*GunshipTop).var_E as i32;
    word!(
        game.samus_y_pos,
        wrapping_add(crate::rtl::rom_table_word!(game, 0xa2a622, v2 as isize))
    );
    let v3 = Get_GunshipTop(k as i32 + 128, game);
    (*v3).base.y_pos =
        ((*v3).base.y_pos).wrapping_add(crate::rtl::rom_table_word!(game, 0xa2a622, v2 as isize));
    let v4 = Get_GunshipTop(k as i32 + 64, game);
    (*v4).base.y_pos =
        ((*v4).base.y_pos).wrapping_add(crate::rtl::rom_table_word!(game, 0xa2a622, v2 as isize));
    (*GunshipTop).base.y_pos = ((*GunshipTop).base.y_pos)
        .wrapping_add(crate::rtl::rom_table_word!(game, 0xa2a622, v2 as isize));
    let mut v5: u16 = ((*GunshipTop).var_E).wrapping_add(1);
    (*GunshipTop).var_E = v5;
    if v5 as i32 - 17i32 & 0x8000 == 0 {
        (*GunshipTop).var_F = ((0xa2a942i32) as u16);
        (*GunshipTop).var_E = (*GunshipTop).base.y_pos;
        (*GunshipTop).var_D = 1;
        let v6 = Get_GunshipTop(0, game);
        (*v6).var_C = 0;
        word!(game.samus_x_pos = ((*GunshipTop).base.x_pos).wrapping_add(1));
        word!(game.samus_prev_x_pos = word!(game.samus_x_pos));
        let v7 = Get_GunshipTop(k as i32 + 128, game);
        enemy_instruction! {[(*v7).base]; timer = 1, instruction = addr_kGunshipTop_Ilist_A5BE as u16}
        (*v6).var_A = 144;
        QueueSfx3_Max6(0x14, game);
    }
}
pub unsafe fn GunshipTop_6(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(k as i32, game);
    let r18: u16 = ((*E).var_E).wrapping_sub(30);
    if word!(game.samus_y_pos, wrapping_sub(1), reread) as i32 - r18 as i32 & 0x8000 != 0 {
        (*E).var_F = ((0xa2a987i32) as u16);
        let E2 = Get_GunshipTop(k as i32 + 128, game);
        enemy_instruction! {[(*E2).base]; timer = 1, instruction = addr_kGunshipTop_Ilist_A5EE as u16}
        (*Get_GunshipTop(0i32, game)).var_A = 144;
        QueueSfx3_Max6(0x15, game);
    }
}
pub unsafe fn GunshipTop_8(k: u16, game: &mut Game) {
    if word!(game.game_state) == 8 && word!(game.frame_handler_alfa) as i32 == 0x90e695 & 0xffff {
        let E = Get_GunshipTop(k as i32, game);
        if (((*E).base.x_pos as i32 - 8i32 - word!(game.samus_x_pos) as i32) as i16) < 0
            && (((*E).base.x_pos as i32 + 8 - word!(game.samus_x_pos) as i32) as i16) >= 0
            && (((*E).base.y_pos as i32 - 64i32 - word!(game.samus_y_pos) as i32) as i16) < 0
            && (((*E).base.y_pos) as i16).wrapping_sub((word!(game.samus_y_pos)) as i16) >= 0
            && *ram_ptr!(game; 0xa1f) == 0
            && (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) != 0
        {
            (*E).var_F = ((0xa2aa4fi32) as u16);
            if word!(game.samus_x_pos) != 1152 {
                word!(game.samus_x_pos = (*E).base.x_pos);
                word!(game.samus_prev_x_pos = word!(game.samus_x_pos));
            }
            MakeSamusFaceForward(game);
            CallSomeSamusCode(0x1a, game);
            word!(game.elevator_status = 0);
            let v3: u16 = ((*E).base.y_pos).wrapping_sub(1);
            let E2 = Get_GunshipTop(k as i32 + 128, game);
            (*E2).base.y_pos = v3;
            enemy_instruction! {[(*E2).base]; timer = 1, instruction = addr_kGunshipTop_Ilist_A5BE as u16}
            (*Get_GunshipTop(0i32, game)).var_A = 144;
            QueueSfx3_Max6(0x14, game);
        }
    }
}
pub unsafe fn GunshipTop_10(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(k as i32, game);
    let r18: u16 = ((*E).var_E).wrapping_add(18);
    word!(game.samus_y_pos, wrapping_add(2));
    if word!(game.samus_y_pos) as i32 - r18 as i32 & 0x8000 == 0 {
        (*E).var_F = ((0xa2aa94i32) as u16);
        let E2 = Get_GunshipTop(k as i32 + 128, game);
        enemy_instruction! {[(*E2).base]; timer = 1, instruction = addr_kGunshipTop_Ilist_A5EE as u16}
        (*Get_GunshipTop(0i32, game)).var_A = 144;
        QueueSfx3_Max6(0x15, game);
    }
}
pub unsafe fn GunshipTop_12(k: u16, game: &mut Game) {
    if CheckEventHappened(0xe, game) != 0 {
        (*Get_GunshipTop(k as i32, game)).var_F = ((0xa2abc7i32) as u16);
        let E = Get_GunshipTop(k as i32 + 64, game);
        set_fields! {[(*E)], 0; var_F, var_E}
        word! { game, 0; substate, suit_pickup_light_beam_pos }
        *(ram_ptr!(game, u16; 0xdf0)) = 0;
        *(ram_ptr!(game, u16; 0xdf2)) = 0;
        CallSomeSamusCode(0xa, game);
    } else {
        Samus_RestoreHealth(2, game);
        Samus_RestoreMissiles(2, game);
        Samus_RestoreSuperMissiles(2, game);
        Samus_RestorePowerBombs(2, game);
        if ((word!(game.samus_reserve_health)) as i16)
            .wrapping_sub((word!(game.samus_max_reserve_health)) as i16)
            >= 0
            && ((word!(game.samus_health)) as i16)
                .wrapping_sub((word!(game.samus_max_health)) as i16)
                >= 0
            && ((word!(game.samus_missiles)) as i16)
                .wrapping_sub((word!(game.samus_max_missiles)) as i16)
                >= 0
            && ((word!(game.samus_super_missiles)) as i16)
                .wrapping_sub((word!(game.samus_max_super_missiles)) as i16)
                >= 0
            && ((word!(game.samus_power_bombs)) as i16)
                .wrapping_sub((word!(game.samus_max_power_bombs)) as i16)
                >= 0
        {
            (*Get_GunshipTop(k as i32, game)).var_F = ((0xa2ab1fi32) as u16);
        }
    };
}
pub unsafe fn GunshipTop_13(k: u16, game: &mut Game) {
    let rv: i32 = DisplayMessageBox_Poll(0x1c, game);
    if rv < 0 {
        return;
    }
    if rv != 2 {
        crate::update_place! { reference fresh3 = *(ram_ptr!(game, u16; 0xd8f8)) => (*fresh3) | (1u16); }
        word!(game.load_station_index = 0);
        SaveToSram(word!(game.selected_save_slot), game);
    }
    (*Get_GunshipTop(k as i32, game)).var_F = ((0xa2ab60i32) as u16);
    let E = Get_GunshipTop(k as i32 + 128, game);
    enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kGunshipTop_Ilist_A5BE as u16}
    (*Get_GunshipTop(0i32, game)).var_A = 144;
    QueueSfx3_Max6(0x14, game);
}
pub unsafe fn GunshipTop_15(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(k as i32, game);
    let r18: u16 = ((*E).var_E).wrapping_sub(30);
    word!(game.samus_y_pos, wrapping_sub(2));
    if word!(game.samus_y_pos) as i32 - r18 as i32 & 0x8000 != 0 {
        (*E).var_F = ((0xa2aba5i32) as u16);
        let E2 = Get_GunshipTop(k as i32 + 128, game);
        enemy_instruction! {[(*E2).base]; timer = 1, instruction = addr_kGunshipTop_Ilist_A5EE as u16}
        (*Get_GunshipTop(0i32, game)).var_A = 144;
        QueueSfx3_Max6(0x15, game);
    }
}
crate::prefixed_callbacks! {
    pub unsafe fn (k: u16, game: &mut Game);
    @prefix { let E = Get_GunshipTop(0, game); };
    GunshipTop_5 [] {
        let v2: bool = (*E).var_A == 1;
        (*E).var_A = (*E).var_A.wrapping_sub(1);
        let v3: bool = ((*E).var_A & 0x8000) != 0;
        if v2 as i32 != 0 || v3 as i32 != 0 {
            (*Get_GunshipTop(k as i32, game)).var_F = ((0xa2a950i32) as u16);
        }
    };
    GunshipTop_7 [] {
        let v2: bool = (*E).var_A == 1;
        (*E).var_A = (*E).var_A.wrapping_sub(1);
        let v3: bool = ((*E).var_A & 0x8000) != 0;
        if v2 as i32 != 0 || v3 as i32 != 0 {
            (*Get_GunshipTop(k as i32, game)).var_F = ((0xa2a9bdi32) as u16);
            word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
            word!(game.frame_handler_beta = ((0x90e725i32) as u16));
            word!(game.loading_game_state = kLoadingGameState_5_Main as u16);
            crate::update_place! { reference fresh11 = *(ram_ptr!(game, u16; 0xd8f8)) => (*fresh11) | (1u16); }
            word!(game.load_station_index = 0);
            SaveToSram(word!(game.selected_save_slot), game);
        }
    };
    GunshipTop_9 [] {
        let v2: bool = (*E).var_A == 1;
        (*E).var_A = (*E).var_A.wrapping_sub(1);
        let v3: bool = ((*E).var_A & 0x8000) != 0;
        if v2 as i32 != 0 || v3 as i32 != 0 {
            (*Get_GunshipTop(k as i32, game)).var_F = ((0xa2aa5di32) as u16);
        }
    };
    GunshipTop_11 [] {
        let v2: bool = (*E).var_A == 1;
        (*E).var_A = (*E).var_A.wrapping_sub(1);
        let v3: bool = ((*E).var_A & 0x8000) != 0;
        if v2 as i32 != 0 || v3 as i32 != 0 {
            (*Get_GunshipTop(k as i32, game)).var_F = ((0xa2aaa2i32) as u16);
        }
    };
    GunshipTop_14 [] {
        let v2: bool = (*E).var_A == 1;
        (*E).var_A = (*E).var_A.wrapping_sub(1);
        let v3: bool = ((*E).var_A & 0x8000) != 0;
        if v2 as i32 != 0 || v3 as i32 != 0 {
            (*Get_GunshipTop(k as i32, game)).var_F = ((0xa2ab6ei32) as u16);
        }
    };
    GunshipTop_16 [] {
        let v2: bool = (*E).var_A == 1;
        (*E).var_A = (*E).var_A.wrapping_sub(1);
        let v3: bool = ((*E).var_A & 0x8000) != 0;
        if v2 as i32 != 0 || v3 as i32 != 0 {
            (*Get_GunshipTop(k as i32, game)).var_F = ((0xa2a9bdi32) as u16);
            if word!(game.game_state) as i32 - kGameState_40_TransitionToDemo as i32 & 0x8000 != 0 {
                word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
                word!(game.frame_handler_beta = ((0x90e725i32) as u16));
            }
        }
    };
}
pub unsafe fn GunshipTop_17(k: u16, game: &mut Game) {
    let v1: u16 = word!(game.substate);
    let mut v2: u16 = word!(game.vram_write_queue_tail);
    (*gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game)).size = 1024;
    v2 = (v2).wrapping_add(2);
    let v3: i32 = v1 as i32 >> 1;
    (*gVramWriteEntry(v2 as i32, game)).size =
        crate::rtl::rom_table_word!(game, 0xa2ac07, v3 as isize);
    v2 = (v2).wrapping_add(2);
    crate::update_place! { previous fresh5 = v2 => v2.wrapping_add(1); }
    *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
        fresh5 as i32,
        game,
    ))
    .size as *mut u8) = -108i32 as u8;
    (*gVramWriteEntry(v2 as i32, game)).size =
        crate::rtl::rom_table_word!(game, 0xa2ac11, v3 as isize);
    word!(game.vram_write_queue_tail = (v2).wrapping_add(2));
    word!(game.substate, wrapping_add(2));
    if word!(game.substate) as i32 - 10i32 & 0x8000 == 0 {
        (*Get_GunshipTop(k as i32, game)).var_F = ((0xa2ac1bi32) as u16);
        word!(game.substate = 0);
    }
}
pub unsafe fn GunshipTop_18(k: u16, game: &mut Game) {
    let E1 = Get_GunshipTop(k as i32 + 64, game);
    let gtp_var_E: u16 = (*E1).var_E;
    if gtp_var_E as i32 - 64i32 & 0x8000 != 0 {
        let mut v3: u16 = if (gtp_var_E & 1) != 0 {
            (word!(game.samus_y_pos)).wrapping_add(1)
        } else {
            (word!(game.samus_y_pos)).wrapping_sub(1)
        };
        word!(game.samus_y_pos = v3);
        word!(game.samus_prev_y_pos = v3);
    } else {
        let mut v4: u16 = if (gtp_var_E & 1) != 0 {
            (word!(game.samus_y_pos)).wrapping_add(2)
        } else {
            (word!(game.samus_y_pos)).wrapping_sub(2)
        };
        word!(game.samus_y_pos = v4);
        word!(game.samus_prev_y_pos = v4);
    }
    let mut v5: i16 = ((word!(game.samus_y_pos)) as i16).wrapping_sub(17);
    let E0 = Get_GunshipTop(k as i32, game);
    (*E0).base.y_pos = (word!(game.samus_y_pos)).wrapping_sub(17);
    (*Get_GunshipTop(k as i32 + 128i32, game)).base.y_pos = ((v5) as u16).wrapping_sub(1);
    (*E1).base.y_pos = (word!(game.samus_y_pos)).wrapping_add(23);
    let v7: u16 = ((*E1).var_E).wrapping_add(1);
    (*E1).var_E = v7;
    if v7 as i32 - 128i32 & 0x8000 != 0 {
        if v7 == 64 {
            SpawnEprojWithRoomGfx(addr_stru_86A379 as u16, 0, game);
            SpawnEprojWithRoomGfx(addr_stru_86A379 as u16, 2, game);
            SpawnEprojWithRoomGfx(addr_stru_86A379 as u16, 4, game);
            SpawnEprojWithRoomGfx(addr_stru_86A379 as u16, 6, game);
            SpawnEprojWithRoomGfx(addr_stru_86A379 as u16, 8, game);
            SpawnEprojWithRoomGfx(addr_stru_86A379 as u16, 0xa, game);
        }
    } else {
        (*E0).var_F = ((0xa2acd7i32) as u16);
        (*Get_GunshipTop(0i32, game)).var_A = 0;
    };
}
pub unsafe fn GunshipTop_19(k: u16, game: &mut Game) {
    word!(game.samus_y_pos, wrapping_sub(2));
    let mut v1: i16 = ((word!(game.samus_y_pos)) as i16).wrapping_sub(17);
    let E0 = Get_GunshipTop(k as i32, game);
    (*E0).base.y_pos = (word!(game.samus_y_pos)).wrapping_sub(17);
    (*Get_GunshipTop(k as i32 + 128i32, game)).base.y_pos = ((v1) as u16).wrapping_sub(1);
    let E1 = Get_GunshipTop(k as i32 + 64, game);
    (*E1).base.y_pos = (word!(game.samus_y_pos)).wrapping_add(23);
    if (*E0).base.y_pos as i32 - 896i32 & 0x8000 != 0 {
        (*E0).var_F = ((0xa2ad0ei32) as u16);
        (*E1).var_F = 512;
    }
}
pub unsafe fn GunshipTop_20(k: u16, game: &mut Game) {
    GunshipTop_21(k, game);
    let E = Get_GunshipTop(k as i32, game);
    if (*E).base.y_pos as i32 - 256i32 & 0x8000 != 0 {
        (*E).var_F = ((0xa2ad2di32) as u16);
        word!(game.game_state = 38);
        word! { game, 0; screen_fade_delay, screen_fade_counter }
    }
}
pub unsafe fn GunshipTop_21(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(k as i32 + 64, game);
    let v3: u16 = ((*E).var_F).wrapping_add(64);
    (*E).var_F = v3;
    if (v3 as i32 & 0xff00) - 2560i32 & 0x8000 == 0 {
        (*E).var_F = 2304;
    }
    let v6: u32 = (words!(game.samus_y_pos, samus_y_subpos)).wrapping_sub(words!((
        ((*E).var_F >> 8),
        (((*E).var_F as i32) << 8) as u16
    )));
    word!(game.samus_y_subpos = v6 as u16);
    word!(game.samus_y_pos = (v6 >> 16) as u16);
    let v5: u16 = (word!(game.samus_y_pos)).wrapping_sub(17);
    (*Get_GunshipTop(k as i32, game)).base.y_pos = v5;
    (*Get_GunshipTop(k as i32 + 128i32, game)).base.y_pos = (v5).wrapping_sub(1);
    (*E).base.y_pos = (word!(game.samus_y_pos)).wrapping_add(23);
}
pub unsafe fn Flies_Init(game: &mut Game) {
    let E = Get_Flies(word!(game.cur_enemy_index) as i32, game);
    (*E).var_E = 0;
    (*E).var_F = ((0xa2b14ei32) as u16);
    (*E).base.current_instruction = addr_kFlies_Ilist_B013 as u16;
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as u16;
    (*E).base.instruction_timer = 1;
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as u16;
}
pub unsafe fn CarryAdd16(rv: *mut u16, v: u16) -> bool {
    let i: i32 = *rv as i32 + v as i32;
    *rv = i as u16;
    i >> 16 != 0
}
pub unsafe fn Flies_1(k: u16, game: &mut Game) {
    let E = Get_Flies(k as i32, game);
    let v2: i32 = (*E).var_E as i32 >> 1;
    let mut carry: bool = CarryAdd16(
        &raw mut (*E).base.x_subpos,
        ((byte!(@pointer (&raw const kSinCosTable8bit_Sext as *const i16).offset((v2 + 64i32) as isize); 0)
            as i32)
            << 8) as u16,
    );
    (*E).base.x_pos = ((*E).base.x_pos as i32
        + (byte!(@pointer (&raw const kSinCosTable8bit_Sext as *const i16).offset((v2 + 64i32) as isize); 1)
            as i8 as i32
            + carry as i32)) as u16;
    carry = CarryAdd16(
        &raw mut (*E).base.y_subpos,
        ((byte!(@pointer (&raw const kSinCosTable8bit_Sext as *const i16).offset(v2 as isize); 0)
            as i32)
            << 8) as u16,
    );
    (*E).base.y_pos = ((*E).base.y_pos as i32
        + (byte!(@pointer (&raw const kSinCosTable8bit_Sext as *const i16).offset(v2 as isize); 1)
            as i8 as i32
            + carry as i32)) as u16;
}
pub unsafe fn Flies_2(k: u16, game: &mut Game) -> u16 {
    let E = Get_Flies(k as i32, game);
    let mut carry: bool = CarryAdd16(
        &raw mut (*E).base.x_subpos,
        ((byte!((*E).var_B; 0) as i32) << 8) as u16,
    );
    (*E).base.x_pos =
        ((*E).base.x_pos as i32 + (byte!((*E).var_B; 1) as i8 as i32 + carry as i32)) as u16;
    carry = CarryAdd16(
        &raw mut (*E).base.y_subpos,
        ((byte!((*E).var_C; 0) as i32) << 8) as u16,
    );
    let result: u16 =
        ((*E).base.y_pos as i32 + byte!((*E).var_C; 1) as i8 as i32 + carry as i32) as u16;
    (*E).base.y_pos = result;
    result
}
pub unsafe fn Flies_Main(game: &mut Game) {
    NextRandom(game);
    let E = Get_Flies(word!(game.cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn Flies_3(k: u16, game: &mut Game) {
    let v1: i32 = CalculateAngleOfSamusFromEnemy(k, game) as i32;
    let E = Get_Flies(k as i32, game);
    (*E).var_B = (2u16).wrapping_mul((kSinCosTable8bit_Sext[(v1 + 64) as usize]) as u16);
    (*E).var_C = (4u16).wrapping_mul((kSinCosTable8bit_Sext[v1 as usize]) as u16);
    (*E).var_D = word!(game.samus_y_pos);
    (*E).var_F = ((0xa2b1aai32) as u16);
}
pub unsafe fn Flies_4(k: u16, game: &mut Game) {
    let E = Get_Flies(k as i32, game);
    let flies_var_A: u16 = (*E).var_A;
    if flies_var_A != 0 {
        (*E).var_A = (flies_var_A).wrapping_sub(1);
    } else if CompareDistToSamus_X(k, 0x70, game) & 1 == 0 {
        Flies_3(k, game);
        return;
    }
    Flies_1(k, game);
    let v3: bool = ((*E).var_E as i32 + 32 & 0x1ff) == 0;
    (*E).var_E = ((*E).var_E as i32 + 32 & 0x1ff) as u16;
    if v3 {
        (*E).var_F = ((0xa2b17ci32) as u16);
    }
}
pub unsafe fn Flies_5(k: u16, game: &mut Game) {
    let E = Get_Flies(k as i32, game);
    let flies_var_A: u16 = (*E).var_A;
    if flies_var_A != 0 {
        (*E).var_A = (flies_var_A).wrapping_sub(1);
    } else if CompareDistToSamus_X(k, 0x70, game) & 1 == 0 {
        Flies_3(k, game);
        return;
    }
    Flies_1(k, game);
    let v3: bool = ((*E).var_E as i32 - 32i32 & 0x1ff) == 0;
    (*E).var_E = ((*E).var_E as i32 - 32i32 & 0x1ff) as u16;
    if v3 {
        (*E).var_F = ((0xa2b14ei32) as u16);
    }
}
pub unsafe fn Flies_6(k: u16, game: &mut Game) {
    let v1: u16 = Flies_2(k, game);
    let E = Get_Flies(k as i32, game);
    (*E).var_A = (*E).var_A.wrapping_add(1);
    let reverse_direction: bool = if ((*E).var_C & 0x8000) == 0 {
        if v1 < (*E).var_D {
            return;
        }
        true
    } else {
        (v1 < (*E).var_D)
    };
    if reverse_direction {
        (*E).var_C = -((*E).var_C as i32) as u16;
        (*E).var_F = ((0xa2b1d2i32) as u16);
    };
}
pub unsafe fn Flies_7(k: u16, game: &mut Game) {
    Flies_2(k, game);
    let E = Get_Flies(k as i32, game);
    (*E).var_A = (*E).var_A.wrapping_sub(1);
    if ((*E).var_A & 0x8000) != 0 {
        (*E).var_A = 24;
        (*E).var_F = ((0xa2b14ei32) as u16);
    }
}
pub unsafe fn NorfairErraticFireball_Init(game: &mut Game) {
    let E = Get_NorfairErraticFireball(word!(game.cur_enemy_index) as i32, game);
    let pt: Point32 = ConvertAngleToXy((*E).parameter_1, byte!((*E).parameter_2; 0) as u16, game);
    (*E).var_C = (pt.x >> 16) as u16;
    (*E).var_D = pt.x as u16;
    (*E).var_E = (pt.y >> 16) as u16;
    (*E).var_F = pt.y as u16;
    (*E).base.current_instruction = addr_kNorfairErraticFireball_Ilist_B2DC as u16;
}
pub unsafe fn NorfairErraticFireball_Main(game: &mut Game) {
    let E = Get_NorfairErraticFireball(word!(game.cur_enemy_index) as i32, game);
    let pt: Point32 = ConvertAngleToXy((*E).parameter_1, byte!((*E).parameter_2; 0) as u16, game);
    (*E).var_C = (pt.x >> 16) as u16;
    (*E).var_D = pt.x as u16;
    (*E).var_E = (pt.y >> 16) as u16;
    (*E).var_F = pt.y as u16;
    if ((*E).parameter_1 as i32 + 64 & 0x80) == 0 {
        (*E).var_C = !((*E).var_C as i32) as u16;
        (*E).var_D = -((*E).var_D as i32) as u16;
    }
    if Enemy_MoveRight_IgnoreSlopes(
        word!(game.cur_enemy_index),
        (words!((*E), var_C, var_D)) as i32,
        game,
    ) != 0
    {
        (*E).parameter_1 = ((*E).parameter_1) ^ (0x40u16);
    } else {
        if ((*E).parameter_1 as i32 + 128 & 0x80) == 0 {
            (*E).var_E = !((*E).var_E as i32) as u16;
            (*E).var_F = -((*E).var_F as i32) as u16;
        }
        if Enemy_MoveDown(
            word!(game.cur_enemy_index),
            (words!((*E), var_E, var_F)) as i32,
            game,
        ) != 0
        {
            (*E).parameter_1 = ((*E).parameter_1) ^ (0xc0u16);
        }
    };
}
pub unsafe fn LavaquakeRocks_Init(game: &mut Game) {
    let E = Get_LavaquakeRocks(word!(game.cur_enemy_index) as i32, game);
    enemy_instruction! {[(*E).base]; instruction = addr_kNorfairErraticFireball_Ilist_B51A as u16, timer = 1}
    (*E).base.timer = 0;
    (*E).var_A = ((0xa2b596i32) as u16);
    word!(game.random_number = 17);
}
crate::game::address_dispatch! {
    pub unsafe fn CallLavaQuakeRocksFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10663318 => LavaquakeRocks_1; 10663346 => LavaquakeRocks_2; 10663402 => LavaquakeRocks_3;
    }
}
pub unsafe fn LavaquakeRocks_Main(game: &mut Game) {
    let E = Get_LavaquakeRocks(word!(game.cur_enemy_index) as i32, game);
    CallLavaQuakeRocksFunc((((*E).var_A) as u32) | (0xa20000u32), game);
}
pub unsafe fn LavaquakeRocks_1(game: &mut Game) {
    if IsSamusWithinEnemy_X(word!(game.cur_enemy_index), 0x40, game) != 0
        && IsSamusWithinEnemy_Y(word!(game.cur_enemy_index), 0x40, game) != 0
    {
        (*Get_LavaquakeRocks(word!(game.cur_enemy_index) as i32, game)).var_A =
            ((0xa2b5b2i32) as u16);
    }
}
pub unsafe fn LavaquakeRocks_2(game: &mut Game) {
    word!(
        game.cinematic_var9 = crate::rtl::rom_table_word!(
            game,
            0xa2b550,
            ((NextRandom(game) as i32 & 0x1e) >> 1) as isize,
        )
    );
    let v1: i32 = (NextRandom(game) as i32 & 0x1e) >> 1;
    SpawnEprojWithGfx(
        crate::rtl::rom_table_word!(game, 0xa2b530, v1 as isize),
        word!(game.cur_enemy_index),
        addr_stru_86BD5A as u16,
        game,
    );
    let E = Get_LavaquakeRocks(word!(game.cur_enemy_index) as i32, game);
    (*E).var_A = ((0xa2b5eai32) as u16);
    (*E).var_B = crate::rtl::rom_table_word!(
        game,
        0xa2b520,
        (((NextRandom(game)) & (0xeu16)) >> 1) as isize
    );
}
pub unsafe fn LavaquakeRocks_3(game: &mut Game) {
    let E = Get_LavaquakeRocks(word!(game.cur_enemy_index) as i32, game);
    (*E).var_B = (*E).var_B.wrapping_sub(1);
    if ((*E).var_B & 0x8000) != 0 {
        (*E).var_A = ((0xa2b596i32) as u16);
    }
}
pub unsafe fn Rinka_Init(game: &mut Game) {
    let E = Get_Rinka(word!(game.cur_enemy_index) as i32, game);
    if (*E).parameter_1 != 0 {
        Rinka_1(word!(game.cur_enemy_index), game);
        (*E).base.properties = ((*E).base.properties as i32
            & !(kEnemyProps_RespawnIfKilled as i32
                | kEnemyProps_DisableSamusColl as i32
                | kEnemyProps_ProcessedOffscreen as i32
                | kEnemyProps_Tangible as i32)
            | 0x2c00) as u16;
    } else {
        (*E).base.properties = ((*E).base.properties as i32
            & !(kEnemyProps_RespawnIfKilled as i32
                | kEnemyProps_DisableSamusColl as i32
                | kEnemyProps_ProcessedOffscreen as i32
                | kEnemyProps_Tangible as i32)
            | 0x6400) as u16;
    }
    (*E).base.palette_index = 1024;
    Rinka_Init3(word!(game.cur_enemy_index), game);
}
pub unsafe fn Rinka_1(k: u16, game: &mut Game) {
    let mut v2: u16 = 0;
    let ES = gEnemySpawnData(k as i32, game);
    if Rinka_10((*ES).x_pos, (*ES).y_pos, game) as i32 != 0 || {
        v2 = Rinka_2(k, game);
        (*gRam8000_Default(v2 as i32, game).offset(31)).var_3F & 1 != 0
    } {
        for v3 in (0u16..66).step_by(6) {
            let v4: i32 = v3 as i32 >> 1;
            let x: u16 = crate::rtl::rom_table_word!(game, 0xa2b75b, v4 as isize);
            let y: u16 = crate::rtl::rom_table_word!(game, 0xa2b75b, (v4 + 1) as isize);
            if !Rinka_10(x, y, game)
                && (*gRam8000_Default(
                    crate::rtl::rom_table_word!(game, 0xa2b75b, (v4 + 2) as isize) as i32,
                    game,
                )
                .offset(31))
                .var_3F
                    & 1
                    == 0
            {
                let E = Get_Rinka(k as i32, game);
                (*ES).x_pos = x;
                (*E).base.x_pos = x;
                (*ES).y_pos = y;
                (*E).base.y_pos = y;
                let v9: u16 =
                    crate::rtl::rom_table_word!(game, 0xa2b75b, ((v3 as i32 >> 1) + 2) as isize);
                (*gRam8000_Default(v9 as i32, game).offset(31)).var_3F = -1i32 as u16;
                (*E).var_D = v9;
                return;
            }
        }
        let mut v10: u16 = 0;
        while (*gRam8000_Default(
            crate::rtl::rom_table_word!(game, 0xa2b75b, ((v10 as i32 >> 1) + 2) as isize) as i32,
            game,
        )
        .offset(31))
        .var_3F
            & 1
            != 0
        {
            v10 = (v10).wrapping_add(6);
            if ((v10) as i16).wrapping_sub(66) >= 0 {
                return;
            }
        }
        let v11: i32 = v10 as i32 >> 1;
        let v12: u16 = crate::rtl::rom_table_word!(game, 0xa2b75b, v11 as isize);
        (*ES).x_pos = v12;
        let E_0 = Get_Rinka(k as i32, game);
        (*E_0).base.x_pos = v12;
        let v15: u16 = crate::rtl::rom_table_word!(game, 0xa2b75b, (v11 + 1) as isize);
        (*ES).y_pos = v15;
        (*E_0).base.y_pos = v15;
        let v16: u16 = crate::rtl::rom_table_word!(game, 0xa2b75b, (v11 + 2) as isize);
        (*E_0).var_D = v16;
        (*gRam8000_Default(v16 as i32, game).offset(31)).var_3F = -1i32 as u16;
    } else {
        (*Get_Rinka(k as i32, game)).var_D = v2;
        (*gRam8000_Default(v2 as i32, game).offset(31)).var_3F = -1i32 as u16;
    };
}
pub unsafe fn Rinka_2(k: u16, game: &mut Game) -> u16 {
    let mut v1: u16 = 0;
    loop {
        let v2: i32 = v1 as i32 >> 1;
        let mut v3 = gEnemySpawnData(k as i32, game);
        if crate::rtl::rom_table_word!(game, 0xa2b75b, v2 as isize) == (*v3).x_pos
            && crate::rtl::rom_table_word!(game, 0xa2b75b, (v2 + 1) as isize) == (*v3).y_pos
        {
            break;
        }
        v1 = (v1).wrapping_add(6);
        if v1 as i32 - 66i32 & 0x8000 == 0 {
            v1 = 0;
            return crate::rtl::rom_table_word!(game, 0xa2b75b, ((v1 as i32 >> 1) + 2) as isize);
        }
    }
    crate::rtl::rom_table_word!(game, 0xa2b75b, ((v1 as i32 >> 1) + 2) as isize)
}
crate::game::address_dispatch! {
    pub unsafe fn CallRinkaFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        10663903 => Rinka_3; 10664004 => Rinka_4; 10664018 => Rinka_5; 10664027 => Rinka_B85B;
    }
}
pub unsafe fn Rinka_Main(game: &mut Game) {
    let E = Get_Rinka(word!(game.cur_enemy_index) as i32, game);
    if (*E).parameter_1 != 0 && (*Get_Rinka(0i32, game)).var_1D != 0 {
        Rinka_6(word!(game.cur_enemy_index), game);
        Rinka_8(word!(game.cur_enemy_index), game);
        let mut v2: u16 = 0;
        RinkasDeathAnimation(v2, game);
    } else {
        CallRinkaFunc(
            (((*E).var_A) as u32) | (0xa20000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
}
crate::game::countdown_callbacks! {
    pub unsafe fn (k: u16, game: &mut Game);
    before {
        let E = Get_Rinka(k as i32, game);
        (*E).var_F = (*E).var_F.wrapping_sub(1);
    } when ((*E).var_F & 0x8000) != 0;
    Rinka_3 {
        (*E).var_A = ((0xa2b85bi32) as u16);
        let mut v3: u16 = if (*E).parameter_1 != 0 {
            ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16
        } else {
            ((*E).base.properties & 0xf3ff | 0x800)
        };
        (*E).base.properties = v3;
        let r18: u16 = -(CalculateAngleFromXY(
            (word!(game.samus_x_pos)).wrapping_sub((*E).base.x_pos),
            (word!(game.samus_y_pos)).wrapping_sub((*E).base.y_pos),
            game,
        ) as i32
            + 0x80i32) as u8 as u16;
        (*E).var_B = Math_MultBySin(0x120, r18);
        (*E).var_C = Math_MultByCos(0x120, r18);
    };
    Rinka_4 {
        (*E).base.health = 10;
        Rinka_Init2(k, game);
    };
}
pub unsafe fn Rinka_5(k: u16, game: &mut Game) {
    if Rinka_9(k, game) & 1 != 0 {
        Rinka_B865(k, game);
    }
}
pub unsafe fn Rinka_B85B(k: u16, game: &mut Game) {
    MoveEnemyWithVelocity(game);
    if Rinka_9(k, game) & 1 != 0 {
        Rinka_B865(k, game);
    }
}
pub unsafe fn Rinka_B865(k: u16, game: &mut Game) {
    if (*Get_Rinka(k as i32, game)).parameter_1 != 0 && {
        Rinka_8(k, game);
        (*Get_Rinka(0i32, game)).var_1D != 0
    } {
        Rinka_6(k, game);
        DeleteEnemyAndConnectedEnemies(game);
    } else {
        Rinka_6(k, game);
        Rinka_Init2(k, game);
    };
}
pub unsafe fn Rinka_9(k: u16, game: &mut Game) -> u8 {
    let E = Get_Rinka(k as i32, game);
    let mut y_pos: i16 = (*E).base.y_pos as i16;
    let result: u8 = 1;
    if y_pos >= 0 {
        let mut v3: i16 = (y_pos as i32 + 16 - word!(game.layer1_y_pos) as i32) as i16;
        if v3 >= 0 && v3 as i32 - 256i32 & 0x8000 != 0 {
            let mut x_pos: i16 = (*E).base.x_pos as i16;
            if x_pos >= 0 {
                let mut v5: i16 = (x_pos as i32 + 16 - word!(game.layer1_x_pos) as i32) as i16;
                if v5 >= 0 && v5 as i32 - 288i32 & 0x8000 != 0 {
                    return 0;
                }
            }
        }
    }
    result
}
pub unsafe fn Rinka_10(r18: u16, r20: u16, game: &mut Game) -> bool {
    (r20 & 0x8000) != 0
        || ((r20) as i16).wrapping_sub((word!(game.layer1_y_pos)) as i16) < 0
        || r20 as i32 - word!(game.layer1_y_pos) as i32 - 224i32 & 0x8000 == 0
        || (r20 & 0x8000) != 0
        || ((r18) as i16).wrapping_sub((word!(game.layer1_x_pos)) as i16) < 0
        || r18 as i32 - word!(game.layer1_x_pos) as i32 - 256i32 & 0x8000 == 0
}
pub unsafe fn Rinka_Frozen(k: u16, game: &mut Game) {
    if Rinka_9(k, game) & 1 != 0 {
        (*Get_Rinka(k as i32, game)).base.frozen_timer = 0;
    }
    NormalEnemyFrozenAI(game);
    if (*Get_Rinka(0i32, game)).var_1D != 0 {
        Rinka_6(k, game);
        Rinka_8(k, game);
        let v1: u16 = 0;
        RinkasDeathAnimation(v1, game);
    }
}
pub unsafe fn Rinka_Touch(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
    Rinka_B960(word!(game.cur_enemy_index), game);
}
pub unsafe fn Rinka_Shot(game: &mut Game) {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    Rinka_B960(word!(game.cur_enemy_index), game);
}
pub unsafe fn Rinka_Powerbomb(k: u16, game: &mut Game) {
    if ((*Get_Rinka(k as i32, game)).base.properties as i32 & kEnemyProps_Invisible as i32) == 0 {
        NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
        Rinka_B960(k, game);
    }
}
crate::prefixed_callbacks! {
    pub unsafe fn (k: u16, game: &mut Game);
    @prefix { let E = Get_Rinka(k as i32, game); };
    Rinka_Init2 [] {
        if (*E).parameter_1 != 0 {
            Rinka_1(k, game);
        }
        let mut v2 = gEnemySpawnData(k as i32, game);
        (*E).base.x_pos = (*v2).x_pos;
        (*E).base.y_pos = (*v2).y_pos;
        Rinka_Init3(k, game);
    };
    Rinka_Init3 [] {
        (*E).var_A = ((0xa2b852i32) as u16);
        (*E).var_F = 26;
        set_fields! {[(*E)], 0; var_B, var_C}
        if (*E).parameter_1 != 0 {
            if (*Get_Rinka(0i32, game)).var_1D != 0 {
                (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
            } else {
                enemy_instruction! {[(*E).base]; instruction = addr_kRinka_Ilist_BA0C as u16, timer = 1}
                (*E).base.timer = 0;
            }
        } else {
            enemy_instruction! {[(*E).base]; instruction = addr_kRinka_Ilist_B9E0 as u16, timer = 1}
            (*E).base.timer = 0;
        };
    };
    Rinka_6 [] {
        if (*E).parameter_1 != 0 && ((*E).base.properties as i32 & kEnemyProps_Invisible as i32) == 0 {
            let E0 = Get_Rinka(0, game);
            let mut v3: i16 = (((*E0).var_1E) as i16).wrapping_sub(1);
            if v3 < 0 {
                v3 = 0;
            }
            (*E0).var_1E = v3 as u16;
        }
    };
    Rinka_8 [] {
        if (*E).parameter_1 != 0 {
            let rinka_var_D: u16 = (*E).var_D;
            if rinka_var_D != 0 {
                (*gRam8000_Default(rinka_var_D as i32, game).offset(31)).var_3F = 0;
                (*E).var_D = 0;
            }
        }
    };
    Rinka_B960 [] {
        if (*E).base.health == 0 {
            Rinka_6(k, game);
            Rinka_8(k, game);
            if (*E).parameter_1 != 0 {
                (*E).base.properties = ((*E).base.properties as i32
                    | (kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
                    as u16;
                game.eproj_spawn_pt = Point16U {
                    x: (*E).base.x_pos,
                    y: (*E).base.y_pos,
                };
                SpawnEprojWithRoomGfx(0xe509, 3, game);
                (*E).var_A = ((0xa2b844i32) as u16);
                (*E).var_F = 1;
            } else {
                RinkasDeathAnimation(0, game);
            }
        }
    };
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    Rinka_Instr_B9B3 [jp] {
        let E = Get_Rinka(k as i32, game);
        (*E).base.properties = ((*E).base.properties as i32
            | (kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
            as u16;
    };
    Rinka_Instr_B9BD [jp] {
        let E = Get_Rinka(k as i32, game);
        (*E).base.properties = ((*E).base.properties as i32
            | (kEnemyProps_ProcessedOffscreen as i32
                | kEnemyProps_Tangible as i32
                | kEnemyProps_Invisible as i32)) as u16;
    };
    Rinka_Instr_B9C7 [jp] {
        let E = Get_Rinka(k as i32, game);
        (*E).base.properties = ((*E).base.properties as i32
            & !(kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
            as u16;
        (*E).var_A = ((0xa2b7dfi32) as u16);
        let E0 = Get_Rinka(0, game);
        (*E0).var_1E = (*E0).var_1E.wrapping_add(1);
    };
    EnemyInstr_Rio_Instr_1 [jp] {
        (*Get_Rio(word!(game.cur_enemy_index) as i32, game)).var_E = 1;
    };
}
pub unsafe fn Rio_Init(game: &mut Game) {
    let E = Get_Rio(word!(game.cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0; var_E, var_F}
    (*E).base.current_instruction = addr_kRio_Ilist_BB4B as u16;
    (*E).var_B = ((0xa2bbedi32) as u16);
}
crate::game::address_dispatch! {
    pub unsafe fn CallRioFunc(ea: u32, game: &mut Game) => ea;
        setup { let k: u16 = word!(game.cur_enemy_index); }
        call (k, game); default { Unreachable(game); };
    {
        10664941 => Rio_1; 10665010 => Rio_2; 10665032 => Rio_3; 10665143 => Rio_4;
        10665215 => Rio_5;
    }
}
pub unsafe fn Rio_Main(game: &mut Game) {
    NextRandom(game);
    let E = Get_Rio(word!(game.cur_enemy_index) as i32, game);
    CallRioFunc((((*E).var_B) as u32) | (0xa20000u32), game);
}
pub unsafe fn Rio_1(k: u16, game: &mut Game) {
    if CompareDistToSamus_X(k, 0xa0, game) & 1 == 0 {
        let E = Get_Rio(k as i32, game);
        (*E).var_C = crate::rtl::rom_table_word!(game, 0xa2bbbb, 0);
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa2bbbf, 0);
        if ((word!(game.samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) < 0 {
            (*E).var_D = -((*E).var_D as i32) as u16;
        }
        Rio_6(addr_kRio_Ilist_BB7F as u16, game);
        (*E).var_B = ((0xa2bc48i32) as u16);
        if CheckIfEnemyIsOnScreen(game) == 0 {
            QueueSfx2_Max6(0x65, game);
        }
    }
}
pub unsafe fn Rio_2(k: u16, game: &mut Game) {
    let E = Get_Rio(k as i32, game);
    if (*E).var_E != 0 {
        (*E).var_E = 0;
        Rio_6(addr_stru_A2BB53 as u16, game);
        (*E).var_B = ((0xa2bbedi32) as u16);
    }
}
pub unsafe fn Rio_3(k: u16, game: &mut Game) {
    let E = Get_Rio(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(k, ((*E).var_D as i16 as i32) << 8, game) != 0 {
        (*E).var_D = -((*E).var_D as i32) as u16;
    } else if (Enemy_MoveDown(k, ((*E).var_C as i16 as i32) << 8, game) == 0) {
        let mut v4: i16 = (((*E).var_C) as i16).wrapping_sub(24);
        (*E).var_C = v4 as u16;
        if v4 < 0 {
            (*E).var_A = (*E).var_D;
            set_fields! {[(*E)], 0; var_D, var_C}
            (*E).var_B = ((0xa2bcffi32) as u16);
        } else if (*E).var_E != 0 {
            (*E).var_E = 0;
            Rio_6(addr_kRio_Ilist_BB97 as u16, game);
        }
        return;
    }
    (*E).var_C = -((*E).var_C as i32) as u16;
    (*E).var_B = ((0xa2bcb7i32) as u16);
}
pub unsafe fn Rio_4(k: u16, game: &mut Game) {
    let E = Get_Rio(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(k, ((*E).var_D as i16 as i32) << 8, game) != 0 {
        (*E).var_D = -((*E).var_D as i32) as u16;
    }
    if Enemy_MoveDown(k, ((*E).var_C as i16 as i32) << 8, game) != 0 {
        Rio_6(addr_kRio_Ilist_BBA3 as u16, game);
        (*E).var_B = ((0xa2bc32i32) as u16);
    } else {
        (*E).var_C = ((*E).var_C).wrapping_sub(24);
    };
}
pub unsafe fn Rio_5(k: u16, game: &mut Game) {
    let E = Get_Rio(k as i32, game);
    if (((*Get_Rio(k as i32, game)).base.y_pos) as i16)
        .wrapping_sub((word!(game.samus_y_pos)) as i16)
        >= 0
    {
        (*E).var_D = (*E).var_A;
        (*E).var_C = -1i32 as u16;
        (*E).var_B = ((0xa2bcb7i32) as u16);
    } else {
        let v1: i32 = CalculateAngleOfSamusFromEnemy(k, game) as i32;
        (*E).var_D = kSinCosTable8bit_Sext[(v1 + 64) as usize] as u16;
        (*E).var_C = kSinCosTable8bit_Sext[v1 as usize] as u16;
        Enemy_MoveRight_IgnoreSlopes(k, ((*E).var_D as i16 as i32) << 8, game);
        Enemy_MoveDown(k, ((*E).var_C as i16 as i32) << 8, game);
    };
}
pub unsafe fn Rio_6(a: u16, game: &mut Game) {
    let E = Get_Rio(word!(game.cur_enemy_index) as i32, game);
    if a != (*E).var_F {
        (*E).var_F = a;
        enemy_instruction! {[(*E).base]; instruction = a, timer = 1}
        (*E).base.timer = 0;
    }
}
pub unsafe fn NorfairLavajumpingEnemy_Instr_BE8E(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_NorfairLavajumpingEnemy(word!(game.cur_enemy_index) as i32, game)).var_00 = 1;
    jp
}
pub unsafe fn NorfairLavajumpingEnemy_Init(game: &mut Game) {
    let E = Get_NorfairLavajumpingEnemy(word!(game.cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0; var_00, var_01}
    if ((*E).parameter_1 & 0x8000) != 0 {
        (*E).base.current_instruction = addr_kNorfairLavajumpingEnemy_Ilist_BE62 as u16;
        (*E).var_F = ((0xa2bedci32) as u16);
    } else {
        (*E).var_D = (*E).base.x_pos;
        (*E).var_E = (*E).base.y_pos;
        (*E).base.current_instruction = addr_kNorfairLavajumpingEnemy_Ilist_BE3C as u16;
        (*E).var_F = ((0xa2bf1ai32) as u16);
    };
}
pub unsafe fn NorfairLavajumpingEnemy_Main(game: &mut Game) {
    NextRandom(game);
    let E = Get_NorfairLavajumpingEnemy(word!(game.cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).var_F) as u32) | (0xa20000u32), game);
}
crate::prefixed_callbacks! {
    pub unsafe fn (k: u16, game: &mut Game);
    @prefix { let E = Get_NorfairLavajumpingEnemy(k as i32, game); };
    NorfairLavajumpingEnemy_Func_1 [] {
        let v1: i32 = k as i32 >> 1;
        if word!(game.enemy_drawing_queue[(v1 + 100)]) != 0 {
            let v3: u16 = word!(game.enemy_drawing_queue[(v1 + 109)]);
            (*E).base.frozen_timer = v3;
            if v3 != 0 || (word!(game.enemy_drawing_queue_sizes[(v1 + 2)]) & 0x8000) == 0 {
                (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
            } else {
                (*E).base.properties =
                    ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
                (*E).base.y_pos = word!(game.enemy_drawing_queue[(v1 + 93)]);
            }
        } else {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        };
    };
    NorfairLavajumpingEnemy_Func_2 [] {
        (*E).var_C = crate::rtl::rom_table_word!(
            game,
            0xa2be86,
            ((((*(ram_ptr!(game, u16; 0x5e5) as *mut u8).offset(1)) as u16) & (6u16)) >> 1) as isize,
        );
        (*E).var_F = ((0xa2bf3ei32) as u16);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_ProcessedOffscreen) as u16);
        QueueSfx2_Max6(0xd, game);
    };
    NorfairLavajumpingEnemy_Func_3 [] {
        let t: i32 = (*E).base.y_subpos as i32 + ((byte!((*E).var_C; 0) as i32) << 8);
        (*E).base.y_subpos = t as u16;
        (*E).base.y_pos =
            ((*E).base.y_pos as i32 + (byte!((*E).var_C; 1) as i8 as i32 + (t >> 16))) as u16;
        let v3: u16 = ((*E).var_C).wrapping_add(56);
        (*E).var_C = v3;
        if v3 >= 0xfc00 {
            NorfairLavajumpingEnemy_Func_6(addr_kNorfairLavajumpingEnemy_Ilist_BE42 as u16, game);
            (*E).var_F = ((0xa2bf7ci32) as u16);
        }
    };
    NorfairLavajumpingEnemy_Func_4 [] {
        let t: i32 = (*E).base.y_subpos as i32 + ((byte!((*E).var_C; 0) as i32) << 8);
        (*E).base.y_subpos = t as u16;
        (*E).base.y_pos =
            ((*E).base.y_pos as i32 + (byte!((*E).var_C; 1) as i8 as i32 + (t >> 16))) as u16;
        (*E).var_C = ((*E).var_C).wrapping_add(56);
        if (*E).var_00 != 0 {
            (*E).var_00 = 0;
            (*E).var_F = ((0xa2bfbci32) as u16);
        }
    };
    NorfairLavajumpingEnemy_Func_5 [] {
        let t: i32 = (*E).base.y_subpos as i32 + ((byte!((*E).var_C; 0) as i32) << 8);
        (*E).base.y_subpos = t as u16;
        (*E).base.y_pos =
            ((*E).base.y_pos as i32 + byte!((*E).var_C; 1) as i8 as i32 + (t >> 16)) as u16;
        if ((*E).base.y_pos & 0xf0) == 240 {
            (*E).base.x_pos = (*E).var_D;
            (*E).base.y_pos = (*E).var_E;
            NorfairLavajumpingEnemy_Func_6(addr_kNorfairLavajumpingEnemy_Ilist_BE3C as u16, game);
            (*E).var_F = ((0xa2bf1ai32) as u16);
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_ProcessedOffscreen as i32)) as u16;
        } else {
            (*E).var_C = ((*E).var_C).wrapping_add(56);
        };
    };
}
pub unsafe fn NorfairLavajumpingEnemy_Func_6(a: u16, game: &mut Game) {
    let E = Get_NorfairLavajumpingEnemy(word!(game.cur_enemy_index) as i32, game);
    if a != (*E).var_01 {
        (*E).var_01 = a;
        enemy_instruction! {[(*E).base]; instruction = a, timer = 1}
        (*E).base.timer = 0;
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    NorfairRio_Instr_C1C9 [jp] {
        (*Get_NorfairRio(word!(game.cur_enemy_index) as i32, game)).var_01 = 1;
    };
    NorfairRio_Instr_C1D4 [jp] {
        (*Get_NorfairRio(word!(game.cur_enemy_index) as i32, game)).var_02 = 8;
    };
    NorfairRio_Instr_C1DF [jp] {
        (*Get_NorfairRio(word!(game.cur_enemy_index) as i32, game)).var_02 = 8;
    };
    NorfairRio_Instr_C1EA [jp] {
        (*Get_NorfairRio(word!(game.cur_enemy_index) as i32, game)).var_02 = 12;
    };
    NorfairRio_Instr_C1F5 [jp] {
        (*Get_NorfairRio(word!(game.cur_enemy_index) as i32, game)).var_02 = -12i32 as u16;
    };
    NorfairRio_Instr_C200 [jp] {
        (*Get_NorfairRio(word!(game.cur_enemy_index) as i32, game)).var_02 = 4;
    };
    NorfairRio_Instr_C20B [jp] {
        (*Get_NorfairRio(word!(game.cur_enemy_index) as i32, game)).var_02 = 0;
    };
    NorfairRio_Instr_C216 [jp] {
        (*Get_NorfairRio(word!(game.cur_enemy_index) as i32, game)).var_02 = -4i32 as u16;
    };
    NorfairRio_Instr_C221 [jp] {
        (*Get_NorfairRio(word!(game.cur_enemy_index) as i32, game)).var_02 = -12i32 as u16;
    };
    NorfairRio_Instr_C22C [jp] {
        (*Get_NorfairRio(word!(game.cur_enemy_index) as i32, game)).var_02 = -16i32 as u16;
    };
    NorfairRio_Instr_C237 [jp] {
        (*Get_NorfairRio(word!(game.cur_enemy_index) as i32, game)).var_02 = 12;
    };
}
pub unsafe fn NorfairRio_Init(game: &mut Game) {
    let E = Get_NorfairRio(word!(game.cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0; var_01, var_02}
    if ((*E).parameter_1 & 0x8000) != 0 {
        (*E).var_00 = addr_kNorfairRio_Ilist_C18F as u16;
        (*E).base.current_instruction = addr_kNorfairRio_Ilist_C18F as u16;
        (*E).var_F = ((0xa2c281i32) as u16);
    } else {
        (*E).var_00 = addr_kNorfairRio_Ilist_C0F1 as u16;
        (*E).base.current_instruction = addr_kNorfairRio_Ilist_C0F1 as u16;
        (*E).var_F = ((0xa2c2e7i32) as u16);
    };
}
pub unsafe fn NorfairRio_Main(game: &mut Game) {
    NextRandom(game);
    let E = Get_NorfairRio(word!(game.cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn NorfairRio_Func_6(k: u16, game: &mut Game) {
    (*Get_NorfairRio(k as i32, game)).var_F = ((0xa2c2e7i32) as u16);
}
pub unsafe fn NorfairRio_Func_7(a: u16, game: &mut Game) {
    let E = Get_NorfairRio(word!(game.cur_enemy_index) as i32, game);
    if a != (*E).var_00 {
        (*E).var_00 = a;
        enemy_instruction! {[(*E).base]; instruction = a, timer = 1}
        (*E).base.timer = 0;
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    LowerNorfairRio_Instr_C6D2 [jp] {
        (*Get_NorfairRio(word!(game.cur_enemy_index) as i32, game)).var_01 = 1;
    };
    LowerNorfairRio_Instr_C6DD [jp] {
        (*Get_NorfairRio(word!(game.cur_enemy_index) as i32, game)).var_02 = 0;
    };
    LowerNorfairRio_Instr_C6E8 [jp] {
        (*Get_NorfairRio(word!(game.cur_enemy_index) as i32, game)).var_02 = 1;
    };
}
pub unsafe fn LowerNorfairRio_Init(game: &mut Game) {
    let E = Get_NorfairRio(word!(game.cur_enemy_index) as i32, game);
    (*E).var_01 = 0;
    if ((*E).parameter_1 & 0x8000) == 0 {
        (*E).var_F = ((0xa2c771i32) as u16);
        (*E).var_00 = addr_kLowerNorfairRio_Ilist_C61A as u16;
        (*E).base.current_instruction = addr_kLowerNorfairRio_Ilist_C61A as u16;
    } else {
        (*E).var_F = ((0xa2c72ei32) as u16);
        (*E).var_00 = addr_kLowerNorfairRio_Ilist_C6B0 as u16;
        (*E).base.current_instruction = addr_kLowerNorfairRio_Ilist_C6B0 as u16;
    };
}
pub unsafe fn LowerNorfairRio_Main(game: &mut Game) {
    NextRandom(game);
    let E = Get_LowerNorfairRio(word!(game.cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn LowerNorfairRio_Func_1(k: u16, game: &mut Game) {
    let E = Get_LowerNorfairRio(k as i32, game);
    let v1: i32 = k as i32 >> 1;
    if word!(game.enemy_drawing_queue[(v1 + 100)]) != 0 {
        let v3: u16 = word!(game.enemy_drawing_queue[(v1 + 109)]);
        (*E).base.frozen_timer = v3;
        if v3 == 0 && (*gEnemySpawnData(k as i32, game).offset(31)).field_4 != 0 {
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
            (*E).base.x_pos = word!(game.enemy_drawing_queue[(v1 + 91)]);
            (*E).base.y_pos = (word!(game.enemy_drawing_queue[(v1 + 93)])).wrapping_add(12);
        } else {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        }
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    };
}
pub unsafe fn LowerNorfairRio_Func_4(k: u16, game: &mut Game) {
    let E = Get_LowerNorfairRio(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(k, ((*E).var_D as i16 as i32) << 8, game) != 0 {
        (*E).var_D = -((*E).var_D as i32) as u16;
    }
    if Enemy_MoveDown(k, ((*E).var_C as i16 as i32) << 8, game) != 0 || {
        let mut v4: i16 = (((*E).var_C) as i16).wrapping_sub(32);
        (*E).var_C = v4 as u16;
        v4 < 0
    } {
        (*E).var_C = -1i32 as u16;
        LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C662 as u16, game);
        (*E).var_F = ((0xa2c82di32) as u16);
        QueueSfx2_Max6(0x64, game);
    }
}
crate::prefixed_callbacks! {
    pub unsafe fn (k: u16, game: &mut Game);
    @prefix { let E = Get_NorfairRio(k as i32, game); };
    NorfairRio_Func_1 [] {
        let v1: i32 = k as i32 >> 1;
        if word!(game.enemy_drawing_queue[(v1 + 100)]) != 0 {
            let v3: u16 = word!(game.enemy_drawing_queue[(v1 + 109)]);
            (*E).base.frozen_timer = v3;
            if v3 != 0 || {
                (*E).base.properties =
                    ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
                (*gEnemySpawnData(k as i32, game).offset(31)).some_flag as i32
                    == addr_kNorfairRio_Ilist_C0F1 as i32
            } {
                (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
            } else {
                let mut r18: u16 = addr_kNorfairRio_Ilist_C18F as u16;
                let mut v6 = gEnemySpawnData(k as i32, game);
                if ((*v6.offset(31)).field_4 & 0x8000) != 0 {
                    r18 = addr_kNorfairRio_Ilist_C1A3 as u16;
                }
                NorfairRio_Func_7(r18, game);
                (*E).base.properties =
                    ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
                (*E).base.x_pos = word!(game.enemy_drawing_queue[(v1 + 91)]);
                (*E).base.y_pos =
                    ((*v6.offset(31)).field_4).wrapping_add(word!(game.enemy_drawing_queue[(v1 + 93)]));
            }
        } else {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        };
    };
    NorfairRio_Func_2 [] {
        if (word!(game.random_number) & 0x101) != 0 && CompareDistToSamus_X(k, 0xc0, game) & 1 == 0 {
            (*E).var_A = crate::rtl::rom_table_word!(
                game,
                0xa2c1c1,
                ((word!(game.random_number) as i32 >> 1 & 2) as u16 >> 1) as isize,
            );
            (*E).var_B = crate::rtl::rom_table_word!(game, 0xa2c1c5, 0);
            if ((word!(game.samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) < 0 {
                (*E).var_B = -((*E).var_B as i32) as u16;
            }
            NorfairRio_Func_7(addr_kNorfairRio_Ilist_C107 as u16, game);
            (*E).var_F = ((0xa2c33fi32) as u16);
        } else if (*E).var_01 != 0 {
            (*E).var_01 = 0;
            NorfairRio_Func_7(addr_kNorfairRio_Ilist_C0F1 as u16, game);
        }
    };
    NorfairRio_Func_3 [] {
        if (*E).var_01 != 0 {
            (*E).var_01 = 0;
            NorfairRio_Func_7(addr_kNorfairRio_Ilist_C12F as u16, game);
            (*E).var_F = ((0xa2c361i32) as u16);
            QueueSfx2_Max6(0x65, game);
        }
    };
    NorfairRio_Func_4 [] {
        if Enemy_MoveRight_IgnoreSlopes(k, ((*E).var_B as i16 as i32) << 8, game) != 0 {
            (*E).var_B = -((*E).var_B as i32) as u16;
        }
        if Enemy_MoveDown(k, ((*E).var_A as i16 as i32) << 8, game) != 0 || {
            let mut v4: i16 = (((*E).var_A) as i16).wrapping_sub(32);
            (*E).var_A = v4 as u16;
            v4 < 0
        } {
            (*E).var_A = -1i32 as u16;
            NorfairRio_Func_7(addr_kNorfairRio_Ilist_C145 as u16, game);
            (*E).var_F = ((0xa2c3b1i32) as u16);
        }
    };
    NorfairRio_Func_5 [] {
        if Enemy_MoveRight_IgnoreSlopes(k, ((*E).var_B as i16 as i32) << 8, game) != 0 {
            (*E).var_B = -((*E).var_B as i32) as u16;
        }
        if Enemy_MoveDown(k, ((*E).var_A as i16 as i32) << 8, game) != 0 {
            (*E).var_F = ((0xa2c406i32) as u16);
        } else {
            (*E).var_A = ((*E).var_A).wrapping_sub(32);
            if (*E).var_01 != 0 {
                (*E).var_01 = 0;
                NorfairRio_Func_7(addr_kNorfairRio_Ilist_C179 as u16, game);
            }
        };
    };
    LowerNorfairRio_Func_2 [] {
        if (word!(game.random_number) & 0x101) != 0 && CompareDistToSamus_X(k, 0x70, game) & 1 == 0 {
            (*E).var_C = crate::rtl::rom_table_word!(game, 0xa2c6ca, 0);
            (*E).var_D = crate::rtl::rom_table_word!(game, 0xa2c6ce, 0);
            if ((word!(game.samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) < 0 {
                (*E).var_D = -((*E).var_D as i32) as u16;
            }
            LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C630 as u16, game);
            (*E).var_F = ((0xa2c7bbi32) as u16);
        } else {
            (*E).var_01 = 0;
            LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C61A as u16, game);
        };
    };
    LowerNorfairRio_Func_3 [] {
        if (*E).var_01 != 0 {
            (*E).var_01 = 0;
            LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C65A as u16, game);
            (*E).var_F = ((0xa2c7d6i32) as u16);
        }
    };
    LowerNorfairRio_Func_5 [] {
        if Enemy_MoveRight_IgnoreSlopes(k, ((*E).var_D as i16 as i32) << 8, game) != 0 {
            (*E).var_D = -((*E).var_D as i32) as u16;
        }
        if Enemy_MoveDown(k, ((*E).var_C as i16 as i32) << 8, game) != 0 {
            LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C686 as u16, game);
            (*E).var_F = ((0xa2c888i32) as u16);
        } else {
            (*E).var_C = ((*E).var_C).wrapping_sub(32);
            if (*E).var_01 != 0 {
                (*E).var_01 = 0;
                LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C674 as u16, game);
            }
        };
    };
    LowerNorfairRio_Func_6 [] {
        if (*E).var_01 != 0 {
            (*E).var_01 = 0;
            LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C686 as u16, game);
            (*E).var_F = ((0xa2c771i32) as u16);
        }
    };
}
pub use self::NorfairRio_Func_7 as LowerNorfairRio_Func_7;
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    MaridiaLargeSnail_Instr_CB6B [jp] { QueueSfx2_Max6(0xe, game); };
    MaridiaLargeSnail_Instr_CCB3 [jp] {
        (*Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game)).var_02 = 1;
    };
    MaridiaLargeSnail_Instr_CCBE [jp] {
        (*Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game)).var_03 = 1;
    };
    MaridiaLargeSnail_Instr_CCC9 [jp] {
        (*Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game)).var_03 = 0;
    };
}
crate::game::address_dispatch! {
    pub unsafe fn CallMaridiaLargeSnailFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10669542 => MaridiaLargeSnail_Func_4; 10669611 => MaridiaLargeSnail_Func_5;
        10669888 => MaridiaLargeSnail_Func_6;
    }
}
pub unsafe fn MaridiaLargeSnail_Main(game: &mut Game) {
    MaridiaLargeSnail_Func_1(word!(game.cur_enemy_index), game);
    let E = Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game);
    CallMaridiaLargeSnailFunc((((*E).var_A) as u32) | (0xa20000u32), game);
    MaridiaLargeSnail_Func_2(word!(game.cur_enemy_index), game);
    MaridiaLargeSnail_Func_3(word!(game.cur_enemy_index), game);
}
pub unsafe fn MaridiaLargeSnail_Func_1(k: u16, game: &mut Game) {
    MaridiaLargeSnail_Func_11(game);
    let E = Get_MaridiaLargeSnail(k as i32, game);
    (*E).var_06 = (*E).base.x_pos;
    (*E).var_08 = (*E).base.y_pos;
}
crate::prefixed_callbacks! {
    pub unsafe fn (k: u16, game: &mut Game);
    @prefix { let E = Get_MaridiaLargeSnail(k as i32, game); };
    MaridiaLargeSnail_Func_2 [] {
        (*E).var_0B = 0;
        if CheckIfEnemyTouchesSamus(k, game) != 0 {
            word!(game.extra_samus_y_displacement = ((*E).base.y_pos).wrapping_sub((*E).var_08));
        }
        if (*E).var_04 != 0 {
            let r18: u16 = ((*E).base.x_pos).wrapping_sub((*E).var_06);
            if (r18 & 0x8000) != 0 {
                if (*E).var_05 == 0 {
                    return;
                }
            } else if (*E).var_05 != 0 {
                return;
            }
            word!(game.extra_samus_x_displacement, wrapping_add(r18));
        }
    };
    MaridiaLargeSnail_Func_3 [] {
        if (*E).var_04 != 0 {
            let mut v2: i16 = (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_06) as i16);
            if v2 == 0 {
                if (*E).var_05 != 0 {
                    if (*E).var_D == 0 || (word!(game.joypad1_lastkeys) & 0x100) == 0 {
                        return;
                    }
                } else if (*E).var_D != 0 || (word!(game.joypad1_lastkeys) & 0x200) == 0 {
                    return;
                }
                (*E).var_0B = 1;
                (*E).base.x_pos = (*E).var_06;
                return;
            }
            if v2 < 0 {
                if (word!(game.joypad1_lastkeys) & 0x100) != 0 {
                    (*E).var_0B = 1;
                    (*E).base.x_pos = (*E).var_06;
                }
            } else if (word!(game.joypad1_lastkeys) & 0x200) != 0 {
                (*E).var_0B = 1;
                (*E).base.x_pos = (*E).var_06;
            }
        }
    };
    MaridiaLargeSnail_Func_7 [] {
        let mut v2: u16 = ((*E).var_B).wrapping_add(384);
        if v2 as i32 - 0x4000i32 & 0x8000 == 0 {
            v2 = 0x4000;
        }
        (*E).var_B = v2;
        let v5: i32 = 8 * (v2 as i32 >> 8) >> 1;
        if Enemy_MoveDown(
            k,
            *(rom_ptr!(game, u32; 0xa0cbc7)).offset((v5 >> 1) as isize) as i32,
            game,
        ) != 0
        {
            (*E).var_E = (*E).var_E.wrapping_sub(1);
            let v6: i16 = (((*E).var_B) as i16).wrapping_sub(4096);
            if v6 < 0 {
                (*E).var_E = 0;
            }
            (*E).var_B = v6 as u16;
            (*E).var_F = ((0xa2cfa9i32) as u16);
        }
    };
    MaridiaLargeSnail_Func_8 [] {
        let v2: i16 = (((*E).var_B) as i16).wrapping_sub(384);
        (*E).var_B = v2 as u16;
        if v2 >= 0 {
            let v3: u16 = ((v2 as i32 & 0x7f00) >> 8) as u16;
            let v5: i32 = 8 * v3 as i32 >> 1;
            Enemy_MoveDown(
                k,
                *(rom_ptr!(game, u32; 0xa0cbc7)).offset((v5 + 2 >> 1) as isize) as i32,
                game,
            );
        } else {
            (*E).var_B = 0;
            (*E).var_F = ((0xa2cf66i32) as u16);
        };
    };
    MaridiaLargeSnail_Func_10 [] {
        let mlsl_var_00: u16 = (*E).var_00;
        if mlsl_var_00 != (*E).var_01 {
            (*E).var_01 = mlsl_var_00;
            enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa2cb77, mlsl_var_00 as isize), timer = 1}
            (*E).base.timer = 0;
        }
    };
}
crate::prefixed_callbacks! {
    pub unsafe fn (game: &mut Game);
    @prefix { let E = Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game); };
    MaridiaLargeSnail_Init [] {
        set_fields! {[(*E)], 0; var_B, var_D, var_00, var_01, var_02, var_03, var_0A}
        (*E).var_E = 3;
        (*E).var_C = 128;
        (*E).base.current_instruction = addr_kMaridiaLargeSnail_Ilist_CA4B as u16;
        (*E).var_A = ((0xa2cde6i32) as u16);
        (*E).var_F = ((0xa2cf66i32) as u16);
    };
    MaridiaLargeSnail_Func_4 [] {
        if (*E).var_E != 0 {
            CallEnemyPreInstr((((*E).var_F) as u32) | (0xa20000u32), game);
        } else {
            (*E).var_00 = 0;
            if (GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) != 0 {
                set_fields! {[(*E)], 1; var_D, var_00}
            }
            MaridiaLargeSnail_Func_10(word!(game.cur_enemy_index), game);
            if IsSamusWithinEnemy_X(word!(game.cur_enemy_index), 0x18, game) != 0 {
                (*E).var_00 = ((*E).var_00) | (2u16);
                MaridiaLargeSnail_Func_10(word!(game.cur_enemy_index), game);
                (*E).var_A = ((0xa2ce2bi32) as u16);
            }
        };
    };
    MaridiaLargeSnail_Func_5 [] {
        if (*E).var_E != 0 {
            CallEnemyPreInstr((((*E).var_F) as u32) | (0xa20000u32), game);
        } else if Enemy_MoveDown(word!(game.cur_enemy_index), (1i16 as i32) << 16, game) == 0 {
            (*E).var_B = 0;
            (*E).var_F = ((0xa2cf66i32) as u16);
            (*E).var_E = 3;
        }
        if (*E).var_0B == 0 {
            let mut R36: u16 = 0;
            (*E).var_C = (*E).var_C.wrapping_sub(1);
            let v4: bool = ((*E).var_C & 0x8000) != 0;
            if v4
                && {
                    (*E).var_C = 0;
                    IsSamusWithinEnemy_X(word!(game.cur_enemy_index), 0x20, game) != 0
                }
                && (*E).var_03 != 0
                && (*E).var_E == 0
            {
                set_fields! {[(*E)], 0; var_00, var_D}
                if (GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) != 0 {
                    set_fields! {[(*E)], 1; var_00, var_D}
                }
                (*E).var_C = 128;
                (*E).var_00 = ((*E).var_00 & 1 | 4);
                MaridiaLargeSnail_Func_10(word!(game.cur_enemy_index), game);
                (*E).var_A = ((0xa2cf40i32) as u16);
            } else {
                let mut v5: u16 = 128;
                if (*E).var_D != 0 {
                    v5 = 132;
                }
                let v6: i32 = v5 as i32 >> 1;
                let r18: u16 = crate::rtl::rom_table_word!(game, 0xa28187, (v6 + 1) as isize);
                let mut r20: u16 = crate::rtl::rom_table_word!(game, 0xa28187, v6 as isize);
                if (*E).var_04 != 0 && (*E).var_D == (*E).var_05 {
                    if (*E).var_D != 0 {
                        r20 = (r20).wrapping_sub(16);
                    } else {
                        r20 = (r20).wrapping_add(16);
                    }
                }
                if Enemy_MoveRight_SlopesAsWalls(
                    word!(game.cur_enemy_index),
                    (words!((r20, r18))) as i32,
                    game,
                ) != 0
                {
                    R36 = R36.wrapping_add(1);
                }
                if (*E).var_04 != 0 && (*E).var_D == (*E).var_05 {
                    let mut v7: u16 = if (*E).var_D != 0 {
                        ((*E).base.x_pos).wrapping_add(16)
                    } else {
                        ((*E).base.x_pos).wrapping_sub(16)
                    };
                    (*E).base.x_pos = v7;
                }
                if R36 != 0 {
                    (*E).var_D = ((*E).var_D) ^ (1u16);
                    (*E).var_00 = ((*E).var_00) ^ (4u16);
                    MaridiaLargeSnail_Func_10(word!(game.cur_enemy_index), game);
                }
            }
        }
    };
    MaridiaLargeSnail_Func_6 [] {
        if (*E).var_02 != 0 {
            (*E).var_02 = 0;
            (*E).var_00 = ((*E).var_00).wrapping_sub(2);
            MaridiaLargeSnail_Func_10(word!(game.cur_enemy_index), game);
            (*E).var_A = ((0xa2ce2bi32) as u16);
        }
    };
    MaridiaLargeSnail_Func_11 [] {
        set_fields! {[(*E)], 0; var_04, var_05}
        if IsSamusWithinEnemy_Y(word!(game.cur_enemy_index), 0x20, game) != 0
            && IsSamusWithinEnemy_X(word!(game.cur_enemy_index), 0x18, game) != 0
        {
            (*E).var_04 = 1;
            if (GetSamusEnemyDelta_X(word!(game.cur_enemy_index), game) & 0x8000) != 0 {
                (*E).var_05 = 1;
            }
        }
    };
}
pub unsafe fn MaridiaLargeSnail_Func_12(game: &mut Game) {
    Enemy_NormalTouchAI_A2(game);
    MaridiaLargeSnail_Touch(game);
}
pub unsafe fn MaridiaLargeSnail_Touch(game: &mut Game) {
    let k: u16 = word!(game.cur_enemy_index);
    if CheckIfEnemyTouchesSamus(k, game) == 0 {
        if ((word!(game.samus_x_pos)) as i16).wrapping_sub(
            ((*Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game))
                .base
                .x_pos) as i16,
        ) < 0
        {
            word!(game.extra_samus_x_displacement, wrapping_sub(4));
        } else {
            word!(game.extra_samus_x_displacement, wrapping_add(4));
        }
    }
}
pub unsafe fn MaridiaLargeSnail_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    QueueSfx2_Max6(0x57, game);
}
unsafe fn HirisingSlowfalling_Func_1(k: u16, game: &mut Game) -> u16 {
    let E = Get_HirisingSlowfalling(k as i32, game);
    Mult8x8((*E).parameter_1 as u8, (*E).var_F as u8)
}
pub unsafe fn HirisingSlowfalling_Init(game: &mut Game) {
    HirisingSlowfalling_Func_3(addr_kHirisingSlowfalling_Ilist_D82C as u16, game);
    let E = Get_HirisingSlowfalling(word!(game.cur_enemy_index) as i32, game);
    (*E).var_A = ((0xa2e035i32) as u16);
    (*E).var_00 = (*E).base.x_pos;
    (*E).var_01 = (*E).base.y_pos;
    let v2: i32 = byte!((*E).parameter_1; 1) as i32;
    (*E).var_E = crate::rtl::rom_table_word!(game, 0xa2df5e, v2 as isize);
    (*E).var_F = *(RomPtr_A2(
        crate::rtl::rom_table_word!(game, 0xa2df6a, v2 as isize),
        game,
    ) as *mut u16);
    let r18: u16 = HirisingSlowfalling_Func_1(word!(game.cur_enemy_index), game);
    HirisingSlowfalling_Func_2(word!(game.cur_enemy_index), r18, game);
    (*E).var_03 = ((*E).var_01).wrapping_sub(r18);
    (*E).var_02 = (*E).var_00;
    (*E).var_B = (*E).var_06;
}
pub unsafe fn HirisingSlowfalling_Func_2(k: u16, r18: u16, game: &mut Game) {
    let mut v: u32 = 0;
    let mut r24: u16 = 0;
    loop {
        r24 = (r24).wrapping_add(512);
        let v1: i32 = 4 * (r24 as i32 >> 8);
        v = v.wrapping_add(words!((
            crate::rtl::rom_table_word!(game, 0xa2838f, (v1 + 1) as isize),
            crate::rtl::rom_table_word!(game, 0xa2838f, v1 as isize)
        )));
        if ((v >> 16i32).wrapping_sub(r18 as u32) & 0x8000 == 0) {
            break;
        }
    }
    (*Get_HirisingSlowfalling(k as i32, game)).var_06 = r24;
}
pub unsafe fn HirisingSlowfalling_Func_3(a: u16, game: &mut Game) {
    let E = Get_HirisingSlowfalling(word!(game.cur_enemy_index) as i32, game);
    enemy_instruction! {[(*E).base]; instruction = a, timer = 1}
    (*E).base.timer = 0;
}
crate::game::address_dispatch! {
    pub unsafe fn CallHirisingSlowfalling(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10674229 => HirisingSlowfalling_Func_4; 10674255 => HirisingSlowfalling_Func_5;
        10674282 => HirisingSlowfalling_Func_6; 10674381 => HirisingSlowfalling_Func_7;
    }
}
pub unsafe fn HirisingSlowfalling_Main(game: &mut Game) {
    let E = Get_HirisingSlowfalling(word!(game.cur_enemy_index) as i32, game);
    CallHirisingSlowfalling((((*E).var_A) as u32) | (0xa20000u32), game);
}
pub unsafe fn HirisingSlowfalling_Func_4(game: &mut Game) {
    if IsSamusWithinEnemy_X(word!(game.cur_enemy_index), 0x50, game) != 0 {
        let E = Get_HirisingSlowfalling(word!(game.cur_enemy_index) as i32, game);
        (*E).var_07 = (*E).parameter_2;
        (*E).var_A = ((0xa2e04fi32) as u16);
    }
}
pub unsafe fn HirisingSlowfalling_Func_5(game: &mut Game) {
    let E = Get_HirisingSlowfalling(word!(game.cur_enemy_index) as i32, game);
    let v1: i16 = (((*E).var_07) as i16).wrapping_sub(1);
    (*E).var_07 = v1 as u16;
    if v1 < 0 {
        HirisingSlowfalling_Func_3(addr_kHirisingSlowfalling_Ilist_D834 as u16, game);
        (*E).var_A = ((0xa2e06ai32) as u16);
    }
}
pub unsafe fn HirisingSlowfalling_Func_6(game: &mut Game) {
    let E = Get_HirisingSlowfalling(word!(game.cur_enemy_index) as i32, game);
    let v2: i32 = 4 * byte!((*E).var_B; 1) as i32;
    words!(
        (*E).base,
        y_pos,
        y_subpos += words!((
            crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 3) as isize),
            crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 2) as isize)
        ))
    );
    let v5: i16 = (((*E).var_B) as i16).wrapping_sub(512);
    (*E).var_B = v5 as u16;
    if v5 < 0 {
        (*E).var_04 = (*E).var_02;
        (*E).base.x_pos = (*E).var_04;
        (*E).var_05 = (*E).var_03;
        (*E).base.y_pos = (*E).var_05;
        (*E).var_C = 0;
        (*E).var_D = ((byte!((*E).parameter_1; 0)) as u16).wrapping_sub(1);
        HirisingSlowfalling_Func_3(addr_kHirisingSlowfalling_Ilist_D840 as u16, game);
        (*E).var_A = ((0xa2e0cdi32) as u16);
    }
}
pub unsafe fn HirisingSlowfalling_Func_7(game: &mut Game) {
    let E = Get_HirisingSlowfalling(word!(game.cur_enemy_index) as i32, game);
    let v1 = RomPtr_A2(
        ((*E).var_E as i32 + 4 * byte!((*E).var_C; 1) as i32) as u16,
        game,
    );
    if *(v1 as *mut u16) == 0x8000 {
        (*E).var_05 = ((*E).var_05).wrapping_add((*E).var_F);
        (*E).var_C = 0;
        (*E).var_D = (*E).var_D.wrapping_sub(1);
        if ((*E).var_D & 0x8000) != 0 {
            (*E).var_B = (*E).var_06;
            (*E).base.x_pos = (*E).var_00;
            (*E).base.x_subpos = 0;
            (*E).base.y_pos = (*E).var_01;
            (*E).base.y_subpos = 0;
            HirisingSlowfalling_Func_3(addr_kHirisingSlowfalling_Ilist_D82C as u16, game);
            (*E).var_A = ((0xa2e035i32) as u16);
        }
    } else {
        (*E).base.x_pos = (*(v1 as *mut u16)).wrapping_add((*E).var_04);
        (*E).base.y_pos = (*(v1.offset(2) as *mut u16)).wrapping_add((*E).var_05);
        (*E).var_C = ((*E).var_C).wrapping_add(256);
    };
}
pub unsafe fn Gripper_Init(game: &mut Game) {
    let E = Get_Gripper(word!(game.cur_enemy_index) as i32, game);
    let v1: u16 = (8u16).wrapping_mul((byte!((*E).base.current_instruction; 0)) as u16);
    (*E).var_E = v1;
    let v2: i32 = v1 as i32 >> 1;
    let mut v3: u16 = if ((*E).base.current_instruction & 0xfeff) != 0 {
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa28187, v2 as isize);
        crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 1) as isize)
    } else {
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 2) as isize);
        crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 3) as isize)
    };
    (*E).var_C = v3;
    let mut v4: u16 = addr_kGripper_Ilist_E19B as u16;
    if ((*E).var_D & 0x8000) == 0 {
        v4 = addr_kGripper_Ilist_E1AF as u16;
    }
    (*E).base.current_instruction = v4;
    (*E).var_A = (*E).parameter_1;
    (*E).var_B = (*E).parameter_2;
}
pub unsafe fn Gripper_Main(game: &mut Game) {
    let E = Get_Gripper(word!(game.cur_enemy_index) as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(
        word!(game.cur_enemy_index),
        (words!((*E), var_D, var_C)) as i32,
        game,
    ) != 0
        || Gripper_Func_1(word!(game.cur_enemy_index), game) & 1 != 0
        || Gripper_Func_2(word!(game.cur_enemy_index), game) & 1 != 0
    {
        let v3: i32 = (*E).var_E as i32 >> 1;
        let mut v4: u16 = if ((*E).var_D & 0x8000) == 0 {
            (*E).var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2) as isize);
            (*E).var_C = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3) as isize);
            addr_kGripper_Ilist_E19B as u16
        } else {
            (*E).var_D = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
            (*E).var_C = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1) as isize);
            addr_kGripper_Ilist_E1AF as u16
        };
        enemy_instruction! {[(*E).base]; instruction = v4, timer = 1}
        (*E).base.timer = 0;
    }
}
pub unsafe fn Gripper_Func_1(k: u16, game: &mut Game) -> u8 {
    let E = Get_Gripper(k as i32, game);
    (((*E).var_D & 0x8000) != 0 && (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_A) as i16) < 0)
        as u8
}
pub unsafe fn Gripper_Func_2(k: u16, game: &mut Game) -> u8 {
    let E = Get_Gripper(k as i32, game);
    (((*E).var_D & 0x8000) == 0
        && (((*E).base.x_pos) as i16).wrapping_sub(((*E).var_B) as i16) >= 0) as u8
}
pub unsafe fn JetPowerRipper_Init(game: &mut Game) {
    let mut v0: u16 = addr_kRipper_Ilist_E2E0 as u16;
    let E = Get_JetPowerRipper(word!(game.cur_enemy_index) as i32, game);
    if (*E).parameter_2 != 0 {
        v0 = addr_kRipper_Ilist_E2F4 as u16;
    }
    (*E).base.current_instruction = v0;
    let v2: u16 = (8u16).wrapping_mul((*E).parameter_1);
    (*E).var_E = v2;
    let v3: i32 = v2 as i32 >> 1;
    let mut v4: u16 = if (*E).parameter_2 != 0 {
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
        crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1) as isize)
    } else {
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2) as isize);
        crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3) as isize)
    };
    (*E).var_C = v4;
}

macro_rules! move_ripper_and_reverse_on_collision {
    (
        $get_enemy:path, $game:ident, $instruction:ident, $positive_instruction:literal,
        $negative_instruction:literal $(,)?
    ) => {
        let E = $get_enemy(word!($game.cur_enemy_index) as i32, $game);
        if Enemy_MoveRight_IgnoreSlopes(
            word!($game.cur_enemy_index),
            (words!((*E), var_D, var_C)) as i32,
            $game,
        ) != 0
        {
            let v2: i32 = (*E).var_E as i32 >> 1;
            if ((*E).var_D & 0x8000) == 0 {
                (*E).var_D = crate::rtl::rom_table_word!($game, 0xa28187, (v2 + 2) as isize);
                (*E).var_C = crate::rtl::rom_table_word!($game, 0xa28187, (v2 + 3) as isize);
                $instruction = -$positive_instruction;
            } else {
                (*E).var_D = crate::rtl::rom_table_word!($game, 0xa28187, v2 as isize);
                (*E).var_C = crate::rtl::rom_table_word!($game, 0xa28187, (v2 + 1) as isize);
                $instruction = -$negative_instruction;
            }
            enemy_instruction! {[(*E).base]; instruction = $instruction as u16, timer = 1}
            (*E).base.timer = 0;
        }
    };
}

pub unsafe fn JetPowerRipper_Main(game: &mut Game) {
    let mut v3: i16 = 0;
    move_ripper_and_reverse_on_collision!(Get_JetPowerRipper, game, v3, 7456i16, 7436i16);
}
pub unsafe fn JetPowerRipper_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    let E = Get_JetPowerRipper(word!(game.cur_enemy_index) as i32, game);
    if (*E).base.frozen_timer != 0 {
        let mut v1: i16 = -7105i16;
        if ((*E).var_D & 0x8000) == 0 {
            v1 = -7093i16;
        }
        (*E).base.spritemap_pointer = v1 as u16;
    }
}
pub unsafe fn Ripper_Init(game: &mut Game) {
    let mut v0: u16 = addr_kRipper_Ilist_E477 as u16;
    let E = Get_Ripper(word!(game.cur_enemy_index) as i32, game);
    if (*E).parameter_2 == 0 {
        v0 = addr_kRipper_Ilist_E48B as u16;
    }
    (*E).base.current_instruction = v0;
    let v2: u16 = (8u16).wrapping_mul((*E).parameter_1);
    (*E).var_E = v2;
    let v3: i32 = v2 as i32 >> 1;
    let mut v4: u16 = if (*E).parameter_2 != 0 {
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
        crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1) as isize)
    } else {
        (*E).var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2) as isize);
        crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3) as isize)
    };
    (*E).var_C = v4;
}
pub unsafe fn Ripper_Main(game: &mut Game) {
    let mut v3: i16 = 0;
    move_ripper_and_reverse_on_collision!(Get_Ripper, game, v3, 7029i16, 7049i16);
}
pub unsafe fn LavaSeahorse_Instr_E5FB(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_LavaSeahorse(word!(game.cur_enemy_index) as i32, game)).var_02 = 1;
    jp
}
pub unsafe fn LavaSeahorse_Init(game: &mut Game) {
    let E = Get_LavaSeahorse(word!(game.cur_enemy_index) as i32, game);
    (*E).var_02 = 0;
    if (*E).parameter_1 != 0 {
        set_fields! {[(*E)], 2; var_00, var_01}
        (*E).base.current_instruction = addr_kLavaSeahorse_Ilist_E5A1 as u16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        (*E).var_F = ((0xa2e781i32) as u16);
    } else {
        set_fields! {[(*E)], 0; var_00, var_01}
        (*E).base.current_instruction = addr_kLavaSeahorse_Ilist_E59B as u16;
        (*E).var_F = ((0xa2e654i32) as u16);
    };
}
pub unsafe fn LavaSeahorse_Main(game: &mut Game) {
    let E = Get_LavaSeahorse(word!(game.cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).var_F) as u32) | (0xa20000u32), game);
}
crate::game::countdown_callbacks! {
    pub unsafe fn (k: u16, game: &mut Game);
    before {
        let E = Get_LavaSeahorse(k as i32, game);
        (*E).var_D = (*E).var_D.wrapping_sub(1);
    } when ((*E).var_D & 0x8000) != 0;
    LavaSeahorse_Func_1 {
        (*E).var_D = 48;
        (*E).var_F = ((0xa2e6adi32) as u16);
        let t: u16 = GetSamusEnemyDelta_X(k, game);
        (*E).var_A = ((*E).var_A & 0x7fff | t & 0x8000);
        if ((*E).var_A & 0x8000) != 0 {
            crate::update_place! { reference fresh16 = *(&raw mut (*E).var_00 as *mut u8) => (*fresh16 & !(1)); }
            let E1 = Get_LavaSeahorse(k as i32 + 64, game);
            crate::update_place! { reference fresh17 = *(&raw mut (*E1).var_00 as *mut u8) => (*fresh17 & !(1)); }
        } else {
            crate::update_place! { reference fresh18 = *(&raw mut (*E).var_00 as *mut u8) => (*fresh18) | (1u8); }
            let E1_0 = Get_LavaSeahorse(k as i32 + 64, game);
            crate::update_place! { reference fresh19 = *(&raw mut (*E1_0).var_00 as *mut u8) => (*fresh19) | (1u8); }
        }
        LavaSeahorse_Func_6(game);
        LavaSeahorse_Func_7(game);
    };
    LavaSeahorse_Func_2 {
        (*E).var_00 = ((*E).var_00).wrapping_add(4);
        (*E).var_D = 3;
        (*E).var_F = ((0xa2e6f1i32) as u16);
    } after {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        let E1 = Get_LavaSeahorse(k as i32 + 64, game);
        (*E1).base.y_pos = (*E1).base.y_pos.wrapping_sub(1);
    };
    LavaSeahorse_Func_5 {
        (*E).var_D = 128;
        (*E).var_F = ((0xa2e654i32) as u16);
    } after {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
        let E1 = Get_LavaSeahorse(k as i32 + 64, game);
        (*E1).base.y_pos = (*E1).base.y_pos.wrapping_add(1);
    };
}
pub unsafe fn LavaSeahorse_Func_3(k: u16, game: &mut Game) {
    LavaSeahorse_Func_6(game);
    let E = Get_LavaSeahorse(k as i32, game);
    if (*E).var_02 != 0 {
        (*E).var_02 = 0;
        (*E).var_01 = -1i32 as u16;
        SpawnEprojWithGfx(0xffff, k, addr_kEproj_LavaSeahorseFireball as u16, game);
        QueueSfx2_Max6(0x61, game);
        crate::update_place! { previous fresh20 = (*E).var_D => (*E).var_D.wrapping_sub(1); }
        if fresh20 == 1 {
            (*E).var_00 = ((*E).var_00).wrapping_sub(4);
            (*E).var_D = 96;
            (*E).var_F = ((0xa2e734i32) as u16);
        }
    }
}
pub unsafe fn LavaSeahorse_Func_4(k: u16, game: &mut Game) {
    let E = Get_LavaSeahorse(k as i32, game);
    crate::update_place! { previous fresh21 = (*E).var_D => (*E).var_D.wrapping_sub(1); }
    if fresh21 == 1 {
        (*E).var_D = 48;
        (*E).var_F = ((0xa2e749i32) as u16);
        LavaSeahorse_Func_6(game);
    }
}
pub unsafe fn LavaSeahorse_Func_6(game: &mut Game) {
    let E = Get_LavaSeahorse(word!(game.cur_enemy_index) as i32, game);
    let lse_var_00: u16 = (*E).var_00;
    if lse_var_00 != (*E).var_01 {
        (*E).var_01 = lse_var_00;
        enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa2e5ef, lse_var_00 as isize), timer = 1}
        (*E).base.timer = 0;
    }
}
pub unsafe fn LavaSeahorse_Func_7(game: &mut Game) {
    let E = Get_LavaSeahorse(word!(game.cur_enemy_index) as i32 + 64, game);
    let lse_var_00: u16 = (*E).var_00;
    if lse_var_00 != (*E).var_01 {
        (*E).var_01 = lse_var_00;
        enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa2e5ef, lse_var_00 as isize), timer = 1}
        (*E).base.timer = 0;
    }
}
pub unsafe fn LavaSeahorse_Touch(game: &mut Game) {
    Enemy_NormalTouchAI_A2(game);
    LavaSeahorse_E7DA(game);
}
pub unsafe fn LavaSeahorse_Shot(game: &mut Game) {
    Enemy_NormalShotAI_A2(game);
    LavaSeahorse_E7DA(game);
}
pub unsafe fn LavaSeahorse_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_A2(game);
    LavaSeahorse_E7DA(game);
}
pub unsafe fn LavaSeahorse_E7DA(game: &mut Game) {
    let E = Get_LavaSeahorse(word!(game.cur_enemy_index) as i32, game);
    if (*E).base.health != 0 {
        let shake_timer: u16 = (*E).base.shake_timer;
        let E1 = Get_LavaSeahorse(word!(game.cur_enemy_index) as i32 + 64, game);
        (*E1).base.shake_timer = shake_timer;
        (*E1).base.invincibility_timer = (*E).base.invincibility_timer;
        (*E1).base.flash_timer = (*E).base.flash_timer;
        (*E1).base.frozen_timer = (*E).base.frozen_timer;
        (*E1).base.ai_handler_bits = (*E).base.ai_handler_bits;
    } else {
        let E1_0 = Get_LavaSeahorse(word!(game.cur_enemy_index) as i32 + 64, game);
        (*E1_0).base.properties = ((*E1_0).base.properties) | (0x200u16);
    };
}
