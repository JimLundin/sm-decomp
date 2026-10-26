use crate::game::actors;
use crate::game::actors::access::{AccessMode::LittleEndian as Le, ActorAccess, ActorKey as Key};
use crate::game::actors::api as actor_api;
use crate::game::enemy_script::EnemyScript;
pub use crate::game::prelude::*;
use crate::game::rippers::{self, Kind};
use crate::game::save_slots;
use crate::game::session_runtime::{Runtime, actor_species_pre_instruction};

pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_86::*;

use crate::game::enemy_projectiles::{
    Math_MultByCos, Math_MultBySin, SpawnEprojWithGfx, SpawnEprojWithRoomGfx,
};
use crate::game::gameplay_flag_runtime::CheckEventHappened;
use crate::game::message_boxes;
use crate::game::music::{QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game::palette_effects;
use crate::game::session_runtime::player_command as CallSomeSamusCode;
use crate::game::session_runtime::player_face_forward;
use crate::game::sm_80::NextRandom;
use crate::game::sm_a9::MoveEnemyWithVelocity;

pub use crate::types::Consts_A2;

unsafe fn update_randomized_pre_instruction(game: &mut Game) {
    unsafe {
        NextRandom(game);
        let phase = {
            let runtime = Runtime::new(game);
            let key = Key::from_legacy(runtime.word(0xe54));
            runtime
                .resolve_actor(key, actors::access::AccessMode::Native)
                .pre_instruction
        };
        actor_species_pre_instruction(0xa20000 | u32::from(phase), game);
    }
}

crate::game::constants::typed_constants! { Consts_A2;
    addr_kLavaSeahorse_Ilist_E5A1 = 58785, addr_kLavaSeahorse_Ilist_E59B = 58779,
    addr_kGripper_Sprmap_E44B = 58443, addr_kGripper_Sprmap_E43F = 58431,
    addr_kHirisingSlowfalling_Ilist_D840 = 55360, addr_kHirisingSlowfalling_Ilist_D834 = 55348,
    addr_kHirisingSlowfalling_Ilist_D82C = 55340, addr_kMaridiaLargeSnail_Ilist_CA4B = 51787,
    addr_kRinka_Ilist_BA0C = 47628, addr_kRinka_Ilist_B9E0 = 47584,
    addr_kNorfairErraticFireball_Ilist_B51A = 46362,
     addr_kGunshipTop_Ilist_A61C = 42524,
    addr_kGunshipTop_Ilist_A616 = 42518, addr_kGunshipTop_Ilist_A60E = 42510,
    addr_kGunshipTop_Ilist_A5EE = 42478, addr_kGunshipTop_Ilist_A5BE = 42430,
    addr_kMaridiaSpikeyShell_Ilist_A3BD = 41917, addr_kMaridiaSpikeyShell_Ilist_A3AB = 41899,
    addr_locret_A28E09 = 36361, addr_kMaridiaBeybladeTurtle_Ilist_8D40 = 36160,
    addr_kMaridiaBeybladeTurtle_Ilist_8D28 = 36136, addr_kMaridiaBeybladeTurtle_Ilist_8D14 = 36116,
    addr_kMaridiaBeybladeTurtle_Ilist_8D00 = 36096, addr_kMaridiaBeybladeTurtle_Ilist_8C72 = 35954,
    addr_kMaridiaBeybladeTurtle_Ilist_8C62 = 35938, addr_kMaridiaBeybladeTurtle_Ilist_8C4A = 35914,
    addr_kMaridiaBeybladeTurtle_Ilist_8C44 = 35908, addr_kMaridiaBeybladeTurtle_Ilist_8C30 = 35888,
    addr_kMaridiaBeybladeTurtle_Ilist_8C1C = 35868, addr_kMaridiaBeybladeTurtle_Ilist_8C02 = 35842,
    addr_kMaridiaBeybladeTurtle_Ilist_8BD2 = 35794, addr_kMaridiaBeybladeTurtle_Ilist_8B80 = 35712,
    addr_kMiniCrocomire_Ilist_896E = 35182, addr_kMiniCrocomire_Ilist_8958 = 35160,
    addr_kMiniCrocomire_Ilist_8948 = 35144, addr_kMiniCrocomire_Ilist_8932 = 35122,
    addr_kSpritemap_Nothing_A2 = 32845,
}

crate::type_aliases! {
    pub Enemy_LavaSeahorse, Enemy_MaridiaTurtle,
        Enemy_MaridiaLargeSnail, Enemy_MaridiaSpikeyShell,
        Enemy_Rinka, Enemy_GunshipTop, Enemy_HirisingSlowfalling,
        Enemy_LavaquakeRocks, Enemy_MiniMaridiaTurtle =
        crate::game::enemy_layouts::EnemyLayout40;
}

crate::type_aliases! {
    pub Enemy_MaridiaBeybladeTurtle, Enemy_MiniCrocomire,
        Enemy_GunshipBottom,
        Enemy_MiniMaridiaBeybladeTurtle =
        crate::game::enemy_layouts::EnemyLayout8;
}

const RomPtr_A2: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa2>;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
crate::game::constants::typed_constants! { i32;
    fnSamus_FrameHandlerAlfa_Func11 = 0x90e695, fnSamus_FrameHandlerBeta_Func17 = 0x90e725,
    fnMiniCrocomire_PreInstr5 = 0xa28a43, fnMiniCrocomire_PreInstr6 = 0xa28a5c,
    fnnullsub_175 = 0xa28a75,
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
    fnGunshipTop_21 = 0xa2ad2d,
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
    Get_GunshipBottom(o: i32) -> Enemy_GunshipBottom = [0xf78, o as isize];
    Get_GunshipTop(o: i32) -> Enemy_GunshipTop = [0xf78, o as isize];
    Get_HirisingSlowfalling(o: i32) -> Enemy_HirisingSlowfalling = [0xf78, o as isize];
    Get_LavaSeahorse(o: i32) -> Enemy_LavaSeahorse = [0xf78, o as isize];
    Get_LavaquakeRocks(o: i32) -> Enemy_LavaquakeRocks = [0xf78, o as isize];
    Get_MaridiaBeybladeTurtle(o: i32) -> Enemy_MaridiaBeybladeTurtle = [0xf78, o as isize];
    Get_MaridiaLargeSnail(o: i32) -> Enemy_MaridiaLargeSnail = [0xf78, o as isize];
    Get_MaridiaSpikeyShell(o: i32) -> Enemy_MaridiaSpikeyShell = [0xf78, o as isize];
    Get_MaridiaTurtle(o: i32) -> Enemy_MaridiaTurtle = [0xf78, o as isize];
    Get_MiniMaridiaBeybladeTurtle(o: i32) -> Enemy_MiniMaridiaBeybladeTurtle = [0xf78, o as isize];
    Get_MiniMaridiaTurtle(o: i32) -> Enemy_MiniMaridiaTurtle = [0xf78, o as isize];
    Get_Rinka(o: i32) -> Enemy_Rinka = [0xf78, o as isize];
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
fn restart_actor_animation(index: u16, instruction: u16, game: &mut Game) {
    Runtime::new(game).edit_actor(Key::from_legacy(index), Le, |actor| {
        actor.restart_animation(instruction)
    });
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
pub unsafe fn MiniCrocomire_Func7(amt: i32, game: &mut Game) {
    {
        let index = game.cur_enemy_index();
        if {
            let (actor, amount) = (index, amt);
            (actor_api::move_horizontal(
                &mut Runtime::new(game),
                Key::from_legacy(actor),
                amount,
                actor_api::SlopePolicy::Ignore,
            ) == actor_api::MovementResult::Blocked) as u8
        } == 0
        {
            let index = game.cur_enemy_index();
            if {
                let (actor, amount) = (index, 2 << 16);
                (actor_api::probe_vertical(
                    &mut Runtime::new(game),
                    Key::from_legacy(actor),
                    amount,
                ) == actor_api::MovementResult::Blocked) as u8
            } != 0
            {
                return;
            }
        }
        MiniCrocomire_Func8(game);
    }
}
pub fn MiniCrocomire_Func8(game: &mut Game) {
    MiniCrocomire_Func1(game);
    let index = game.cur_enemy_index();
    if MiniCrocomireState::new(game, index).state() != 1 {
        MiniCrocomire_Func3(game);
    }
}
pub unsafe fn MiniCrocomire_Func9(game: &mut Game) -> u8 {
    unsafe {
        NextRandom(game);
        let index = game.cur_enemy_index();
        let frame = MiniCrocomireState::new(game, index).frame_counter_low();
        if frame.wrapping_add(game.random_number() as u8) < 2 {
            MiniCrocomireState::new(game, index).set_preinstruction(0x8a75);
            return 1;
        }
        0
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallMaridiaBeybladeTurtleFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub fn (k: u16, game: &mut Game)];
    [0xa28dd8] MaridiaBeybladeTurtle_Func1 [] {
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
    };
    [0xa28e0a] MaridiaBeybladeTurtle_Func2 [] {
        let base = 0xf78 + usize::from(k);
        let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, var_F);
        let active = u16::from_le_bytes([game.g_ram[field], game.g_ram[field + 1]]);
        if active != 0 {
            { let actor = Key::from_legacy(k); let value = 0; Runtime::new(game).edit_actor(actor, Le, |actor| actor.radius.y = value); };
            let actor_x = Runtime::new(game).resolve_actor(Key::from_legacy(k), Le).x.whole;
            let delta = actor_x.wrapping_sub(game.samus_x_pos());
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
                { let actor = Key::from_legacy(k); let value = radius; Runtime::new(game).edit_actor(actor, Le, |actor| actor.radius.y = value); };
                let properties = Runtime::new(game).resolve_actor(Key::from_legacy(k), Le).properties;
                { let actor = Key::from_legacy(k); let value = properties | 0x8000; Runtime::new(game).edit_actor(actor, Le, |actor| actor.properties = value); };
                if ({ let actor = index; 0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(&Runtime::new(game), Key::from_legacy(actor)))) }) != 0 {
                    let actor_y = Runtime::new(game).resolve_actor(Key::from_legacy(k), Le).y.whole;
                    let actor_radius = Runtime::new(game).resolve_actor(Key::from_legacy(k), Le).radius.y;
                    let top = actor_y.wrapping_sub(actor_radius);
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
            let properties = Runtime::new(game).resolve_actor(Key::from_legacy(k), Le).properties;
            { let actor = Key::from_legacy(k); let value = properties & !(kEnemyProps_Tangible as u16); Runtime::new(game).edit_actor(actor, Le, |actor| actor.properties = value); };
        }
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [0xa28ee0] MaridiaBeybladeTurtle_Func3 [] {
        MaridiaBeybladeTurtle_Func4(game);
        if (*ram_ptr!(game; 0x5b5) & 1) == 0 {
            if ({ let actor = k; 0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(&Runtime::new(game), Key::from_legacy(actor)))) }) != 0 {
                word!(game.extra_samus_x_displacement -= 1);
            }
            let E = Get_MaridiaBeybladeTurtle(k as i32, game);
            (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
            (*E).base.y_height = 16;
            if ((*E).base.y_pos & 1) != 0 {
                (*E).base.x_pos = (*E).base.x_pos.wrapping_sub(1);
            } else {
                (*E).base.x_pos = (*E).base.x_pos.wrapping_add(1);
            }
            if ({ let (actor, amount) = (k, (1i16 as i32) << 16); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) == 0 {
                enemy_instruction! {[(*E).base]; instruction = addr_kMaridiaBeybladeTurtle_Ilist_8C4A as u16, timer = 1}
                (*E).var_A = addr_locret_A28E09 as u16;
            }
        }
    };
    [0xa28f3f] MaridiaBeybladeTurtle_Func5 [] {
        let mut v1: u16 = addr_kMaridiaBeybladeTurtle_Ilist_8C1C as u16;
        let E = Get_MaridiaBeybladeTurtle(k as i32, game);
        if (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0 {
            v1 = addr_kMaridiaBeybladeTurtle_Ilist_8D00 as u16;
        }
        enemy_instruction! {[(*E).base]; instruction = v1, timer = 1}
        (*E).var_A = addr_locret_A28E09 as u16;
    };
    [0xa28f8d] MaridiaBeybladeTurtle_Func7 [] {
        MaridiaBeybladeTurtle_Func4(game);
        if ({ let (actor, amount) = (k, (-1i16 as i32) << 16); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) == 0 {
            if ({ let actor = k; 0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(&Runtime::new(game), Key::from_legacy(actor)))) }) != 0 {
                word!(game.extra_samus_y_displacement -= 1);
            }
            let E = Get_MaridiaTurtle(k as i32, game);
            let v2: u16 = ((*E).var_00).wrapping_sub(1);
            (*E).var_00 = v2;
            if v2 == 0 {
                let mut v3: u16 = 0;
                if (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0 {
                    v3 = 4;
                }
                (*E).var_01 = *((rom_ptr!(game, u16; 0xa28d56) as *mut u8)
                    .offset(v3 as i32 as isize) as *mut u16);
                (*E).var_02 = *((rom_ptr!(game, u16; 0xa28d58) as *mut u8)
                    .offset(v3 as i32 as isize) as *mut u16);
                set_fields! {[(*E)], 0; var_E, var_03}
                (*E).var_A = (0xa28febi32) as u16;
            }
        }
    };
    [0xa28feb] MaridiaBeybladeTurtle_Func8 [] {
        MaridiaBeybladeTurtle_Func4(game);
        let E = Get_MaridiaTurtle(k as i32, game);
        if ({ let (actor, amount) = (k, (words!((*E), var_E, var_03)) as i32); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
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
            word! {game; earthquake_type = 0; earthquake_timer = 16; }
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
    };
    [0xa29083] MaridiaBeybladeTurtle_Func9 [] {
        let E = Get_MaridiaTurtle(k as i32, game);
        MaridiaBeybladeTurtle_Func4(game);
        let mut v2: u16 = if (((*E).base.y_pos) as i16).wrapping_sub((g_word_A28D60) as i16) < 0 {
            (*E).var_00 = g_word_A28D64;
            (0xa290cci32) as u16
        } else {
            if ({ let actor = k; 0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(&Runtime::new(game), Key::from_legacy(actor)))) }) != 0 {
                (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(g_word_A28D62);
                word!(game.extra_samus_y_displacement -= g_word_A28D62);
                return;
            }
            (0xa290e1i32) as u16
        };
        (*E).var_A = v2;
        set_fields! {[(*E)], 0; var_07, var_04}
    };
    [0xa290cc] MaridiaBeybladeTurtle_Func10 [] {
        MaridiaBeybladeTurtle_Func4(game);
        let E = Get_MaridiaTurtle(k as i32, game);
        let v2: u16 = ((*E).var_00).wrapping_sub(1);
        (*E).var_00 = v2;
        if v2 == 0 {
            (*E).var_A = (0xa290e1i32) as u16;
        }
    };
    [0xa290e1] MaridiaBeybladeTurtle_Func11 [] {
        MaridiaBeybladeTurtle_Func4(game);
        let E = Get_MaridiaTurtle(k as i32, game);
        if (((*Get_MaridiaTurtle(0i32, game)).var_04) as i16).wrapping_sub((g_word_A28D66) as i16)
            < 0
        {
            words!((*E), var_04, var_07 += 0x2000);
        }
        if ({ let (actor, amount) = (k, ((*E).var_04 as i16 as i32) << 16); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            enemy_instruction! {[(*E).base]; instruction = (if ((*E).var_E & 0x8000) == 0 {
                addr_kMaridiaBeybladeTurtle_Ilist_8D28 as i32
            } else {
                addr_kMaridiaBeybladeTurtle_Ilist_8C4A as i32
            }) as u16, timer = 1}
            (*E).var_A = addr_locret_A28E09 as u16;
        }
    };
    [0xa29142] MiniMaridiaBeybladeTurtle_Func1 [] {
        if ({ let actor = k; 0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(&Runtime::new(game), Key::from_legacy(actor)))) }) != 0 {
            let E = Get_MiniMaridiaTurtle(k as i32, game);
            (*E).var_A = (0xa2916ei32) as u16;
            (*E).var_05 = 4;
            let mut v2: u16 = addr_kMaridiaBeybladeTurtle_Ilist_8C30 as u16;
            if ((*E).var_E & 0x8000) == 0 {
                v2 = addr_kMaridiaBeybladeTurtle_Ilist_8D14 as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v2, timer = 1}
        }
    };
    [0xa2916e] MiniMaridiaBeybladeTurtle_Func2 [] {
        let E = Get_MiniMaridiaTurtle(k as i32, game);
        if ({ let actor = k; 0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(&Runtime::new(game), Key::from_legacy(actor)))) }) != 0 {
            (*E).var_05 = 4;
        } else {
            let v2: u16 = ((*E).var_05).wrapping_sub(1);
            (*E).var_05 = v2;
            if v2 == 0 {
                (*E).var_A = (0xa29198i32) as u16;
                (*E).var_00 = 60;
            }
        };
    };
    [0xa29198] MiniMaridiaBeybladeTurtle_Func3 [] {
        let E = Get_MiniMaridiaTurtle(k as i32, game);
        if ({ let actor = k; 0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(&Runtime::new(game), Key::from_legacy(actor)))) }) != 0 {
            (*E).var_A = (0xa291f8i32) as u16;
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
                (*E).var_A = (0xa29142i32) as u16;
            }
        };
    };
    [0xa291f8] MiniMaridiaBeybladeTurtle_Func4 [] {
        let E = Get_MiniMaridiaTurtle(k as i32, game);
        if ({ let (actor, amount) = (k, (words!(((*E).var_E, 0u16))) as i32); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            (*E).var_E = -((*E).var_E as i32) as u16;
        } else {
            ({ let (actor, amount) = (k, ((*E).var_04 as i16 as i32) << 16); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 });
        };
    };
    [0xa29239] MiniMaridiaBeybladeTurtle_Func6 [] {
        if ({ let actor = k; 0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(&Runtime::new(game), Key::from_legacy(actor)))) }) != 0 {
            let mut v1: u16 = addr_kMaridiaBeybladeTurtle_Ilist_8B80 as u16;
            let E = Get_MiniMaridiaBeybladeTurtle(k as i32, game);
            if ((*E).var_E & 0x8000) == 0 {
                v1 = addr_kMaridiaBeybladeTurtle_Ilist_8C72 as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v1, timer = 1}
            (*E).var_A = (0xa29142i32) as u16;
        } else {
            MiniMaridiaBeybladeTurtle_Func4(k, game);
        };
    };
    @external {
        0xa28e09 => [{}];
        0xa2925e => [MiniMaridiaBeybladeTurtle_Func7(game)];
    }
}

pub unsafe fn MaridiaBeybladeTurtle_Func6(k: u16, game: &mut Game) {
    unsafe {
        if {
            let actor = k;
            0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(
                &Runtime::new(game),
                Key::from_legacy(actor),
            )))
        } != 0
        {
            let E = Get_MaridiaTurtle(k as i32, game);
            (*E).var_A = (0xa29083i32) as u16;
            words!(
                game.extra_samus_x_displacement,
                extra_samus_x_subdisplacement += -(words!(i32, (*E), var_E, var_03)) as u32
            );
            if word!(game.extra_samus_x_displacement) as i32 + 16 & 0x8000 != 0 {
                word!(game.extra_samus_x_displacement = -16i32 as u16);
            }
        }
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func7(game: &mut Game) {
    unsafe {
        let E = Get_MiniMaridiaTurtle(word!(game.cur_enemy_index) as i32, game);
        (*Get_MiniMaridiaTurtle((*E).var_B as i32, game)).var_06 = (*E).base.y_height;
        if {
            let actor = word!(game.cur_enemy_index);
            0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(
                &Runtime::new(game),
                Key::from_legacy(actor),
            )))
        } == 0
        {
            (*Get_MiniMaridiaTurtle(word!(game.cur_enemy_index) as i32, game)).var_A =
                (0xa29142i32) as u16;
        }
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func21(game: &mut Game) {
    unsafe {
        let mbte_var_B: u16 =
            (*Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game)).var_B;
        let E = Get_MaridiaBeybladeTurtle(mbte_var_B as i32, game);
        if (*E).var_F != 0 {
            (*E).var_F = (*E).var_F.wrapping_sub(1);
        }
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func4(game: &mut Game) {
    unsafe {
        let E = Get_MaridiaBeybladeTurtle(0, game);
        let r18: u16 = ((*E).base.x_pos as i32 - (*E).base.x_width as i32 - 8i32) as u16;
        let r20: u16 = ((*E).base.x_width as i32 + (*E).base.x_pos as i32 + 8) as u16;
        let r22: u16 = ((*E).base.y_pos as i32 - (*E).base.y_height as i32 + 4) as u16;
        let r24: u16 = ((*E).base.y_height as i32 + (*E).base.y_pos as i32 - 4i32) as u16;
        if word!(game.samus_x_pos) as i32 - word!(game.samus_x_radius) as i32 - 1i32 - r20 as i32
            & 0x8000
            != 0
            && word!(game.samus_x_radius) as i32 + word!(game.samus_x_pos) as i32 - r18 as i32
                & 0x8000
                == 0
            && word!(game.samus_y_pos) as i32 - word!(game.samus_y_radius) as i32 + 1 - r24 as i32
                & 0x8000
                != 0
            && word!(game.samus_y_radius) as i32 + word!(game.samus_y_pos) as i32 - r22 as i32
                & 0x8000
                == 0
            && word!(game.samus_invincibility_timer) == 0
        {
            actors::combat::contact_reaction(
                &mut Runtime::new(game),
                actors::api::DeathHandling::Finalize,
            );
        }
    }
}
fn thin_blob_phase(variable: usize, game: &mut Game) -> u32 {
    let runtime = Runtime::new(game);
    let key = Key::from_legacy(runtime.word(0xe54));
    0xa20000
        | u32::from(
            runtime
                .resolve_actor(key, actors::access::AccessMode::Native)
                .variables[variable],
        )
}
unsafe fn thin_blob_random(game: &mut Game) -> u16 {
    unsafe {
        NextRandom(game);
    }
    let runtime = Runtime::new(game);
    let key = Key::from_legacy(runtime.word(0xe54));
    runtime
        .resolve_actor(key, actors::access::AccessMode::Native)
        .frame
        .wrapping_add(runtime.word(0x5e5))
        & 7
}
unsafe fn prepare_thin_blob(k: u16, game: &mut Game) {
    {
        let mut runtime = Runtime::new(game);
        if runtime.native_word(0x7800 + usize::from(k)).wrapping_sub(3) & 0x8000 != 0 {
            let raw = runtime.word(0xe54);
            let key = Key::from_legacy(raw);
            let distance = runtime
                .resolve_actor(key, actors::access::AccessMode::Native)
                .parameters[1];
            let close = actor_api::player_within(&mut runtime, key, actor_api::Axis::X, distance);
            runtime.write_slice_native(0x7800 + usize::from(raw), u16::from(close));
        }
    }
    let raw = game.cur_enemy_index();
    crate::game::hopping_blobs::choose_direction(raw, game);
    let random = unsafe { thin_blob_random(game) };
    crate::game::hopping_blobs::choose_jump(raw, random, game);
    crate::game::hopping_blobs::prepare_velocity(game);
}
pub unsafe fn ThinHoppingBlobs_Func6(k: u16, game: &mut Game) {
    let ready = {
        let mut runtime = Runtime::new(game);
        let key = Key::from_legacy(k);
        runtime.edit_actor(key, actors::access::AccessMode::Native, |actor| {
            actor.variables[2] = actor.variables[2].wrapping_sub(1);
        });
        if runtime
            .resolve_actor(key, actors::access::AccessMode::Native)
            .variables[2]
            & 0x8000
            == 0
        {
            false
        } else {
            runtime.edit_actor(key, actors::access::AccessMode::Native, |actor| {
                actor.variables[3] = 0x9b81;
                actor.variables[2] = actor.parameters[0];
            });
            runtime.write_slice_native(0x7802 + usize::from(k), 1);
            true
        }
    };
    if ready {
        unsafe {
            prepare_thin_blob(k, game);
        }
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallThinHoppingBlobs(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    ThinHoppingBlobs_Init [] { crate::game::hopping_blobs::initialize(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    ThinHoppingBlobs_Main [] { CallThinHoppingBlobs(thin_blob_phase(3, game), game); };
    [10656641] ThinHoppingBlobs_Func7 [] { CallThinHoppingBlobsVarE(thin_blob_phase(4, game), game); };
    ThinHoppingBlobs_Func15 [] {
        let raw = word!(game.cur_enemy_index);
        if crate::game::hopping_blobs::fall(game) {
            let random = thin_blob_random(game);
            let mut runtime = Runtime::new(game);
            runtime.write_slice_native(0x7800 + usize::from(raw), (random & 1) + 5);
            runtime.edit_actor(Key::from_legacy(raw), actors::access::AccessMode::Native, |actor| actor.variables[3] = 0x9b65);
        }
    };
    @external { 10656613 => [{ ThinHoppingBlobs_Func6(word!(game.cur_enemy_index), game); }]; }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallThinHoppingBlobsVarE(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [10657035] ThinHoppingBlobs_Func11 [] { crate::game::hopping_blobs::land(crate::game::hopping_blobs::Landing::Animate(0x99d5), game); };
    [10657067] ThinHoppingBlobs_Func12 [] { crate::game::hopping_blobs::land(crate::game::hopping_blobs::Landing::Animate(0x99c1), game); };
    [10657099] ThinHoppingBlobs_Func13 [] { crate::game::hopping_blobs::land(crate::game::hopping_blobs::Landing::Animate(0x99ad), game); };
    [10657131] ThinHoppingBlobs_Func14 [] { crate::game::hopping_blobs::land(crate::game::hopping_blobs::Landing::Restart, game); };
    [10657229] ThinHoppingBlobs_Func16 [] { crate::game::hopping_blobs::land(crate::game::hopping_blobs::Landing::Repeat, game); };
    @external { 10657176 => ThinHoppingBlobs_Func15; }
}
pub unsafe fn SpikeShootingPlant_4(game: &mut Game) {
    unsafe {
        NextRandom(game);
    }
    crate::game::spike_plant::try_shoot(game);
}
static off_A2A3D3: [Func_V; 7] = callback_table! { nonnull Func_V;
    MaridiaSpikeyShell_1, MaridiaSpikeyShell_2, MaridiaSpikeyShell_3, MaridiaSpikeyShell_4,
    MaridiaSpikeyShell_5, MaridiaSpikeyShell_6, MaridiaSpikeyShell_7,
};
pub unsafe fn MaridiaSpikeyShell_8(k: u16, game: &mut Game) {
    unsafe {
        NextRandom(game);
        let E = Get_MaridiaSpikeyShell(k as i32, game);
        if ((byte!((*E).base.frame_counter; 0)).wrapping_add((word!(game.random_number)) as u8))
            as i32
            - 6i32
            & 0x8000
            != 0
        {
            (*E).var_E = 3;
        }
    }
}
pub unsafe fn GunshipTop_1(k: u16, game: &mut Game) {
    unsafe {
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
}
crate::game::callback_dispatch! {
    pub unsafe fn CallLavaQuakeRocksFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10663318] LavaquakeRocks_1 [] {
        if ({ let (actor, distance) = (word!(game.cur_enemy_index), 0x40); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
            && ({ let (actor, distance) = (word!(game.cur_enemy_index), 0x40); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y, distance)) }) != 0
        {
            (*Get_LavaquakeRocks(word!(game.cur_enemy_index) as i32, game)).var_A =
                (0xa2b5b2i32) as u16;
        }
    };
    [10663346] LavaquakeRocks_2 [] {
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
        (*E).var_A = (0xa2b5eai32) as u16;
        (*E).var_B = crate::rtl::rom_table_word!(
            game,
            0xa2b520,
            (((NextRandom(game)) & (0xeu16)) >> 1) as isize
        );
    };
    [10663402] LavaquakeRocks_3 [] {
        let E = Get_LavaquakeRocks(word!(game.cur_enemy_index) as i32, game);
        (*E).var_B = (*E).var_B.wrapping_sub(1);
        if ((*E).var_B & 0x8000) != 0 {
            (*E).var_A = (0xa2b596i32) as u16;
        }
    };
}
pub unsafe fn Rinka_1(k: u16, game: &mut Game) {
    unsafe {
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
                    let v9: u16 = crate::rtl::rom_table_word!(
                        game,
                        0xa2b75b,
                        ((v3 as i32 >> 1) + 2) as isize
                    );
                    (*gRam8000_Default(v9 as i32, game).offset(31)).var_3F = -1i32 as u16;
                    (*E).var_D = v9;
                    return;
                }
            }
            let mut v10: u16 = 0;
            while (*gRam8000_Default(
                crate::rtl::rom_table_word!(game, 0xa2b75b, ((v10 as i32 >> 1) + 2) as isize)
                    as i32,
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
}
pub unsafe fn Rinka_2(k: u16, game: &mut Game) -> u16 {
    unsafe {
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
                return crate::rtl::rom_table_word!(
                    game,
                    0xa2b75b,
                    ((v1 as i32 >> 1) + 2) as isize
                );
            }
        }
        crate::rtl::rom_table_word!(game, 0xa2b75b, ((v1 as i32 >> 1) + 2) as isize)
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallRinkaFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [10664018] Rinka_5 [] {
        if Rinka_9(k, game) & 1 != 0 {
            Rinka_B865(k, game);
        }
    };
    [10664027] Rinka_B85B [] {
        MoveEnemyWithVelocity(game);
        if Rinka_9(k, game) & 1 != 0 {
            Rinka_B865(k, game);
        }
    };
    @countdown [pub unsafe fn (k: u16, game: &mut Game)];
    before {
        let E = Get_Rinka(k as i32, game);
        (*E).var_F = (*E).var_F.wrapping_sub(1);
    }
    when ((*E).var_F & 0x8000) != 0;
    [10663903] Rinka_3 {
        (*E).var_A = (0xa2b85bi32) as u16;
        let mut v3: u16 = if (*E).parameter_1 != 0 {
            ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16
        } else {
            (*E).base.properties & 0xf3ff | 0x800
        };
        (*E).base.properties = v3;
        let r18: u16 = -(actor_api::angle_from_delta((word!(game.samus_x_pos)).wrapping_sub((*E).base.x_pos), (word!(game.samus_y_pos)).wrapping_sub((*E).base.y_pos)) as i32
            + 0x80i32) as u8 as u16;
        (*E).var_B = Math_MultBySin(0x120, r18);
        (*E).var_C = Math_MultByCos(0x120, r18);
    };
    [10664004] Rinka_4 {
        (*E).base.health = 10;
        Rinka_Init2(k, game);
    };
}
pub unsafe fn Rinka_B865(k: u16, game: &mut Game) {
    unsafe {
        if (*Get_Rinka(k as i32, game)).parameter_1 != 0 && {
            Rinka_8(k, game);
            (*Get_Rinka(0i32, game)).var_1D != 0
        } {
            Rinka_6(k, game);
            actors::population::delete_connected(&mut Runtime::new(game));
        } else {
            Rinka_6(k, game);
            Rinka_Init2(k, game);
        };
    }
}
pub unsafe fn Rinka_9(k: u16, game: &mut Game) -> u8 {
    unsafe {
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
}
pub fn Rinka_10(r18: u16, r20: u16, game: &mut Game) -> bool {
    {
        (r20 & 0x8000) != 0
            || ((r20) as i16).wrapping_sub((word!(game.layer1_y_pos)) as i16) < 0
            || r20 as i32 - word!(game.layer1_y_pos) as i32 - 224i32 & 0x8000 == 0
            || (r20 & 0x8000) != 0
            || ((r18) as i16).wrapping_sub((word!(game.layer1_x_pos)) as i16) < 0
            || r18 as i32 - word!(game.layer1_x_pos) as i32 - 256i32 & 0x8000 == 0
    }
}
pub unsafe fn Rinka_Frozen(k: u16, game: &mut Game) {
    unsafe {
        if Rinka_9(k, game) & 1 != 0 {
            (*Get_Rinka(k as i32, game)).base.frozen_timer = 0;
        }
        actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
        if (*Get_Rinka(0i32, game)).var_1D != 0 {
            Rinka_6(k, game);
            Rinka_8(k, game);
            let v1: u16 = 0;
            ({
                let animation = v1;
                actors::combat::die(
                    &mut Runtime::new(game),
                    actors::api::DeathKind::Rinka { animation },
                )
            });
        }
    }
}
pub unsafe fn Rinka_Powerbomb(k: u16, game: &mut Game) {
    unsafe {
        if ((*Get_Rinka(k as i32, game)).base.properties as i32 & kEnemyProps_Invisible as i32) == 0
        {
            actors::combat::power_bomb_reaction(
                &mut Runtime::new(game),
                actors::api::DeathHandling::LeavePending,
            );
            Rinka_B960(k, game);
        }
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
        (*E).var_A = (0xa2b852i32) as u16;
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
                (*E).var_A = (0xa2b844i32) as u16;
                (*E).var_F = 1;
            } else {
                ({ let animation = 0; actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Rinka { animation }) });
            }
        }
    };
}
crate::game::callback_dispatch! {
    pub unsafe fn CallMaridiaLargeSnailFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game); };
    MaridiaLargeSnail_Init [] {
        set_fields! {[(*E)], 0; var_B, var_D, var_00, var_01, var_02, var_03, var_0A}
        (*E).var_E = 3;
        (*E).var_C = 128;
        (*E).base.current_instruction = addr_kMaridiaLargeSnail_Ilist_CA4B as u16;
        (*E).var_A = (0xa2cde6i32) as u16;
        (*E).var_F = (0xa2cf66i32) as u16;
    };
    [10669542] MaridiaLargeSnail_Func_4 [] {
        if (*E).var_E != 0 {
            actor_species_pre_instruction((((*E).var_F) as u32) | (0xa20000u32), game);
        } else {
            (*E).var_00 = 0;
            if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) != 0 {
                set_fields! {[(*E)], 1; var_D, var_00}
            }
            MaridiaLargeSnail_Func_10(word!(game.cur_enemy_index), game);
            if ({ let (actor, distance) = (word!(game.cur_enemy_index), 0x18); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0 {
                (*E).var_00 = ((*E).var_00) | (2u16);
                MaridiaLargeSnail_Func_10(word!(game.cur_enemy_index), game);
                (*E).var_A = (0xa2ce2bi32) as u16;
            }
        };
    };
    [10669611] MaridiaLargeSnail_Func_5 [] {
        if (*E).var_E != 0 {
            actor_species_pre_instruction((((*E).var_F) as u32) | (0xa20000u32), game);
        } else if ({ let (actor, amount) = (word!(game.cur_enemy_index), (1i16 as i32) << 16); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) == 0 {
            (*E).var_B = 0;
            (*E).var_F = (0xa2cf66i32) as u16;
            (*E).var_E = 3;
        }
        if (*E).var_0B == 0 {
            let mut R36: u16 = 0;
            (*E).var_C = (*E).var_C.wrapping_sub(1);
            let v4: bool = ((*E).var_C & 0x8000) != 0;
            if v4
                && {
                    (*E).var_C = 0;
                    ({ let (actor, distance) = (word!(game.cur_enemy_index), 0x20); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
                }
                && (*E).var_03 != 0
                && (*E).var_E == 0
            {
                set_fields! {[(*E)], 0; var_00, var_D}
                if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) != 0 {
                    set_fields! {[(*E)], 1; var_00, var_D}
                }
                (*E).var_C = 128;
                (*E).var_00 = (*E).var_00 & 1 | 4;
                MaridiaLargeSnail_Func_10(word!(game.cur_enemy_index), game);
                (*E).var_A = (0xa2cf40i32) as u16;
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
                if ({ let (actor, amount) = (word!(game.cur_enemy_index), (words!((r20, r18))) as i32); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Solid) == actor_api::MovementResult::Blocked) as u8 }) != 0
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
    [10669888] MaridiaLargeSnail_Func_6 [] {
        if (*E).var_02 != 0 {
            (*E).var_02 = 0;
            (*E).var_00 = ((*E).var_00).wrapping_sub(2);
            MaridiaLargeSnail_Func_10(word!(game.cur_enemy_index), game);
            (*E).var_A = (0xa2ce2bi32) as u16;
        }
    };
    MaridiaLargeSnail_Func_11 [] {
        set_fields! {[(*E)], 0; var_04, var_05}
        if ({ let (actor, distance) = (word!(game.cur_enemy_index), 0x20); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y, distance)) }) != 0
            && ({ let (actor, distance) = (word!(game.cur_enemy_index), 0x18); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
        {
            (*E).var_04 = 1;
            if (({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) & 0x8000) != 0 {
                (*E).var_05 = 1;
            }
        }
    };
}
pub unsafe fn MaridiaLargeSnail_Func_1(k: u16, game: &mut Game) {
    unsafe {
        MaridiaLargeSnail_Func_11(game);
        let E = Get_MaridiaLargeSnail(k as i32, game);
        (*E).var_06 = (*E).base.x_pos;
        (*E).var_08 = (*E).base.y_pos;
    }
}
pub unsafe fn MaridiaLargeSnail_Func_12(game: &mut Game) {
    unsafe {
        Enemy_NormalTouchAI_A2(game);
        MaridiaLargeSnail_Touch(game);
    }
}
unsafe fn HirisingSlowfalling_Func_1(k: u16, game: &mut Game) -> u16 {
    unsafe {
        let E = Get_HirisingSlowfalling(k as i32, game);
        Mult8x8((*E).parameter_1 as u8, (*E).var_F as u8)
    }
}
pub unsafe fn HirisingSlowfalling_Func_2(k: u16, r18: u16, game: &mut Game) {
    unsafe {
        let mut v: u32 = 0;
        let mut r24: u16 = 0;
        do_until! { (v >> 16i32).wrapping_sub(r18 as u32) & 0x8000 == 0 ;
            r24 = (r24).wrapping_add(512);
            let v1: i32 = 4 * (r24 as i32 >> 8);
            v = v.wrapping_add(words!((
                crate::rtl::rom_table_word!(game, 0xa2838f, (v1 + 1) as isize),
                crate::rtl::rom_table_word!(game, 0xa2838f, v1 as isize)
            )));
        }
        (*Get_HirisingSlowfalling(k as i32, game)).var_06 = r24;
    }
}
pub unsafe fn HirisingSlowfalling_Func_3(a: u16, game: &mut Game) {
    unsafe {
        let E = Get_HirisingSlowfalling(word!(game.cur_enemy_index) as i32, game);
        enemy_instruction! {[(*E).base]; instruction = a, timer = 1}
        (*E).base.timer = 0;
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallHirisingSlowfalling(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10674229] HirisingSlowfalling_Func_4 [] {
        if ({ let (actor, distance) = (word!(game.cur_enemy_index), 0x50); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0 {
            let E = Get_HirisingSlowfalling(word!(game.cur_enemy_index) as i32, game);
            (*E).var_07 = (*E).parameter_2;
            (*E).var_A = (0xa2e04fi32) as u16;
        }
    };
    [10674255] HirisingSlowfalling_Func_5 [] {
        let E = Get_HirisingSlowfalling(word!(game.cur_enemy_index) as i32, game);
        let v1: i16 = (((*E).var_07) as i16).wrapping_sub(1);
        (*E).var_07 = v1 as u16;
        if v1 < 0 {
            HirisingSlowfalling_Func_3(addr_kHirisingSlowfalling_Ilist_D834 as u16, game);
            (*E).var_A = (0xa2e06ai32) as u16;
        }
    };
    [10674282] HirisingSlowfalling_Func_6 [] {
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
            (*E).var_A = (0xa2e0cdi32) as u16;
        }
    };
    [10674381] HirisingSlowfalling_Func_7 [] {
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
                set_fields! {[(*E).base]; x_pos = (*E).var_00, x_subpos = 0, y_pos = (*E).var_01,
                    y_subpos = 0, };
                HirisingSlowfalling_Func_3(addr_kHirisingSlowfalling_Ilist_D82C as u16, game);
                (*E).var_A = (0xa2e035i32) as u16;
            }
        } else {
            (*E).base.x_pos = (*(v1 as *mut u16)).wrapping_add((*E).var_04);
            (*E).base.y_pos = (*(v1.offset(2) as *mut u16)).wrapping_add((*E).var_05);
            (*E).var_C = ((*E).var_C).wrapping_add(256);
        };
    };
}
pub unsafe fn LavaSeahorse_Func_6(game: &mut Game) {
    unsafe {
        let E = Get_LavaSeahorse(word!(game.cur_enemy_index) as i32, game);
        let lse_var_00: u16 = (*E).var_00;
        if lse_var_00 != (*E).var_01 {
            (*E).var_01 = lse_var_00;
            enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa2e5ef, lse_var_00 as isize), timer = 1}
            (*E).base.timer = 0;
        }
    }
}
pub unsafe fn LavaSeahorse_Func_7(game: &mut Game) {
    unsafe {
        let E = Get_LavaSeahorse(word!(game.cur_enemy_index) as i32 + 64, game);
        let lse_var_00: u16 = (*E).var_00;
        if lse_var_00 != (*E).var_01 {
            (*E).var_01 = lse_var_00;
            enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa2e5ef, lse_var_00 as isize), timer = 1}
            (*E).base.timer = 0;
        }
    }
}
pub unsafe fn LavaSeahorse_E7DA(game: &mut Game) {
    unsafe {
        let E = Get_LavaSeahorse(word!(game.cur_enemy_index) as i32, game);
        if (*E).base.health != 0 {
            let shake_timer: u16 = (*E).base.shake_timer;
            let E1 = Get_LavaSeahorse(word!(game.cur_enemy_index) as i32 + 64, game);
            set_fields! {[(*E1).base]; shake_timer = shake_timer,
            invincibility_timer = (*E).base.invincibility_timer,
            flash_timer = (*E).base.flash_timer, frozen_timer = (*E).base.frozen_timer,
            ai_handler_bits = (*E).base.ai_handler_bits, };
        } else {
            let E1_0 = Get_LavaSeahorse(word!(game.cur_enemy_index) as i32 + 64, game);
            (*E1_0).base.properties = ((*E1_0).base.properties) | (0x200u16);
        };
    }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    @prefix { let index = game.cur_enemy_index(); };
    [10652077] MiniCrocomire_Init [] {
        { let actor = Key::from_legacy(index); let value = addr_kSpritemap_Nothing_A2 as u16; Runtime::new(game).edit_actor(actor, Le, |actor| actor.spritemap = value); };
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
        restart_actor_animation(index, addr_kMiniCrocomire_Ilist_8932 as u16, game);
    };
    MiniCrocomire_Func2 [] {
        restart_actor_animation(index, addr_kMiniCrocomire_Ilist_8948 as u16, game);
    };
    MiniCrocomire_Func3 [] {
        restart_actor_animation(index, addr_kMiniCrocomire_Ilist_8958 as u16, game);
    };
    MiniCrocomire_Func4 [] {
        restart_actor_animation(index, addr_kMiniCrocomire_Ilist_896E as u16, game);
    };
    [10653036] MaridiaBeybladeTurtle_Init [] {
        let properties = Runtime::new(game).resolve_actor(Key::from_legacy(index), Le).properties;
        { let actor = Key::from_legacy(index); let value = properties | kEnemyProps_DisableSamusColl as u16; Runtime::new(game).edit_actor(actor, Le, |actor| actor.properties = value); };
        { let actor = Key::from_legacy(index); let value = addr_kSpritemap_Nothing_A2 as u16; Runtime::new(game).edit_actor(actor, Le, |actor| actor.spritemap = value); };
        { let actor = Key::from_legacy(index); let value = 1; Runtime::new(game).edit_actor(actor, Le, |actor| actor.instruction_delay = value); };
        { let actor = Key::from_legacy(index); let value = 0; Runtime::new(game).edit_actor(actor, Le, |actor| actor.timer = value); };
        { let actor = Key::from_legacy(index); let value = 0; Runtime::new(game).edit_actor(actor, Le, |actor| actor.radius.y = value); };
        { let actor = Key::from_legacy(index); let value = addr_kMaridiaBeybladeTurtle_Ilist_8C44 as u16; Runtime::new(game).edit_actor(actor, Le, |actor| actor.instruction = value); };
        let base = 0xf78 + usize::from(index);
        let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, var_A);
        game.g_ram[field..field + 2].copy_from_slice(&0x8dd8u16.to_le_bytes());
        let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, var_F);
        game.g_ram[field..field + 2].copy_from_slice(&g_word_A28D52.to_le_bytes());
    };
    [10653085] MiniMaridiaBeybladeTurtle_Init [] {
        let base = 0xf78 + usize::from(index);
        let x = Runtime::new(game).resolve_actor(Key::from_legacy(index), Le).x.whole;
        let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, var_C);
        game.g_ram[field..field + 2].copy_from_slice(&x.to_le_bytes());
        let y = Runtime::new(game).resolve_actor(Key::from_legacy(index), Le).y.whole;
        let radius = Runtime::new(game).resolve_actor(Key::from_legacy(index), Le).radius.y;
        let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, var_D);
        game.g_ram[field..field + 2].copy_from_slice(&y.wrapping_sub(radius).to_le_bytes());
        let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, var_A);
        game.g_ram[field..field + 2].copy_from_slice(&0x9142u16.to_le_bytes());
        { let actor = Key::from_legacy(index); let value = 1; Runtime::new(game).edit_actor(actor, Le, |actor| actor.instruction_delay = value); };
        { let actor = Key::from_legacy(index); let value = 0; Runtime::new(game).edit_actor(actor, Le, |actor| actor.timer = value); };
        let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, parameter_1);
        let parameter = u16::from_le_bytes([game.g_ram[field], game.g_ram[field + 1]]);
        let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, var_E);
        game.g_ram[field..field + 2].copy_from_slice(&parameter.to_le_bytes());
        let instruction = if (parameter as i16) >= 0 {
            addr_kMaridiaBeybladeTurtle_Ilist_8C72 as u16
        } else {
            addr_kMaridiaBeybladeTurtle_Ilist_8B80 as u16
        };
        { let actor = Key::from_legacy(index); let value = instruction; Runtime::new(game).edit_actor(actor, Le, |actor| actor.instruction = value); };
    };
    @functions [pub unsafe fn (game: &mut Game)];
    [10652144] MiniCrocomire_Main [] {
        let index = game.cur_enemy_index();
        let preinstruction = MiniCrocomireState::new(game, index).preinstruction();
        actor_species_pre_instruction(u32::from(preinstruction) | 0xa20000, game);
    };
    [10653138] MaridiaBeybladeTurtle_Main [] {
        let index = game.cur_enemy_index();
        let preinstruction_offset =
            0xf78 + usize::from(index) + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, var_A);
        let preinstruction = u16::from_le_bytes([
            game.g_ram[preinstruction_offset],
            game.g_ram[preinstruction_offset + 1],
        ]);
        let live_index = game.cur_enemy_index();
        CallMaridiaBeybladeTurtleFunc(0xa20000 | u32::from(preinstruction), live_index, game);
    };
    [10653998] MiniMaridiaBeybladeTurtle_Main [] {
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
    };
    [10654337] MaridiaBeybladeTurtle_Touch [] {
        let E = Get_MiniMaridiaTurtle(word!(game.cur_enemy_index) as i32, game);
        if ((*E).base.properties & 0x8000) == 0 {
            Enemy_NormalTouchAI_A2(game);
            (*E).var_A = (0xa290e1i32) as u16;
            (*E).var_04 = 2;
        }
    };
    [10654367] MiniMaridiaBeybladeTurtle_Touch [] {
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
            (*E).var_A = (0xa29142i32) as u16;
            ({ let (actor, amount) = (word!(game.cur_enemy_index), (words!(((*E).var_E, 0u16))) as i32); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 });
            MaridiaBeybladeTurtle_Func21(game);
        }
    };
    [10654479] MiniMaridiaBeybladeTurtle_Shot [] {
        actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
        MaridiaBeybladeTurtle_Func21(game);
    };
    [10658942] MaridiaSpikeyShell_Main [] {
        let e = Get_MaridiaSpikeyShell(word!(game.cur_enemy_index) as i32, game);
        off_A2A3D3[((*e).var_E as i16 as i32 + 2i32) as usize](game);
    };
    [10659396] GunshipTop_Init [] {
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
                (*E).var_F = (0xa2a80ci32) as u16;
            } else {
                let v3: u16 = ((*E).base.y_pos).wrapping_sub(25);
                (*E).base.y_pos = v3;
                (*E).var_E = v3;
                (*E).var_F = (0xa2a9bdi32) as u16;
            }
        } else {
            word!(game.samus_y_pos = 1138);
            let v5: u16 = ((*E).base.y_pos).wrapping_sub(25);
            (*E).base.y_pos = v5;
            (*E).var_E = v5;
            (*E).var_F = (0xa2ab60i32) as u16;
            (*Get_GunshipTop(0i32, game)).var_A = 144;
        }
        palette_effects::spawn(addr_stru_8DE1C0 as u16, game);
        (*Get_GunshipTop(word!(game.cur_enemy_index) as i32, game)).var_D = 1;
        (*Get_GunshipTop(0i32, game)).var_C = 0;
    };
    [10659538] GunshipBottom_Init [] {
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
        (*E).var_F = (0xa2a7d7i32) as u16;
    };
    [10659673] GunshipTop_Main [] {
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
        actor_species_pre_instruction((((*E0).var_F) as u32) | (0xa20000u32), game);
    };
    [10663280] LavaquakeRocks_Init [] {
        let E = Get_LavaquakeRocks(word!(game.cur_enemy_index) as i32, game);
        enemy_instruction! {[(*E).base]; instruction = addr_kNorfairErraticFireball_Ilist_B51A as u16, timer = 1}
        (*E).base.timer = 0;
        (*E).var_A = (0xa2b596i32) as u16;
        word!(game.random_number = 17);
    };
    [10663311] LavaquakeRocks_Main [] {
        let E = Get_LavaquakeRocks(word!(game.cur_enemy_index) as i32, game);
        CallLavaQuakeRocksFunc((((*E).var_A) as u32) | (0xa20000u32), game);
    };
    [10663426] Rinka_Init [] {
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
    };
    [10663876] Rinka_Main [] {
        let E = Get_Rinka(word!(game.cur_enemy_index) as i32, game);
        if (*E).parameter_1 != 0 && (*Get_Rinka(0i32, game)).var_1D != 0 {
            Rinka_6(word!(game.cur_enemy_index), game);
            Rinka_8(word!(game.cur_enemy_index), game);
            let mut v2: u16 = 0;
            ({ let animation = v2; actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Rinka { animation }) });
        } else {
            CallRinkaFunc(
                (((*E).var_A) as u32) | (0xa20000u32),
                word!(game.cur_enemy_index),
                game,
            );
        };
    };
    [10664263] Rinka_Touch [] {
        actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
        Rinka_B960(word!(game.cur_enemy_index), game);
    };
    [10664269] Rinka_Shot [] {
        actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::ReactionOnly);
        Rinka_B960(word!(game.cur_enemy_index), game);
    };
    @functions [pub fn (game: &mut Game)];
    [10664909] Rio_Init [] {
        crate::game::rio::initialize(game);
    };
    @functions [pub unsafe fn (game: &mut Game)];
    [10664931] Rio_Main [] {
        NextRandom(game);
        if !crate::game::rio::advance(game) {
            Unreachable(game);
        }
    };
    [10669331] MaridiaLargeSnail_Main [] {
        MaridiaLargeSnail_Func_1(word!(game.cur_enemy_index), game);
        let E = Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game);
        CallMaridiaLargeSnailFunc((((*E).var_A) as u32) | (0xa20000u32), game);
        MaridiaLargeSnail_Func_2(word!(game.cur_enemy_index), game);
        MaridiaLargeSnail_Func_3(word!(game.cur_enemy_index), game);
    };
    [10670988] MaridiaLargeSnail_Touch [] {
        let k: u16 = word!(game.cur_enemy_index);
        if ({ let actor = k; 0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(&Runtime::new(game), Key::from_legacy(actor)))) }) == 0 {
            if ((word!(game.samus_x_pos)) as i16).wrapping_sub(
                ((*Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game))
                    .base
                    .x_pos) as i16,
            ) < 0
            {
                word!(game.extra_samus_x_displacement -= 4);
            } else {
                word!(game.extra_samus_x_displacement += 4);
            }
        }
    };
    [10671028] MaridiaLargeSnail_Shot [] {
        actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
        QueueSfx2_Max6(0x57, game);
    };
    [10674038] HirisingSlowfalling_Init [] {
        HirisingSlowfalling_Func_3(addr_kHirisingSlowfalling_Ilist_D82C as u16, game);
        let E = Get_HirisingSlowfalling(word!(game.cur_enemy_index) as i32, game);
        (*E).var_A = (0xa2e035i32) as u16;
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
    };
    [10674222] HirisingSlowfalling_Main [] {
        let E = Get_HirisingSlowfalling(word!(game.cur_enemy_index) as i32, game);
        CallHirisingSlowfalling((((*E).var_A) as u32) | (0xa20000u32), game);
    };
    @functions [pub fn (game: &mut Game)];
    [10674643] Gripper_Init [] {
        rippers::initialize(Kind::Gripper, game);
    };
    [10674721] Gripper_Main [] {
        rippers::advance(Kind::Gripper, game);
    };
    [10674968] JetPowerRipper_Init [] {
        rippers::initialize(Kind::JetPowered, game);
    };
    [10675027] JetPowerRipper_Main [] {
        rippers::advance(Kind::JetPowered, game);
    };
    [10675113] JetPowerRipper_Shot [] {
        rippers::jet_shot(game);
    };
    [10675359] Ripper_Init [] {
        rippers::initialize(Kind::Ripper, game);
    };
    [10675418] Ripper_Main [] {
        rippers::advance(Kind::Ripper, game);
    };
    @functions [pub unsafe fn (game: &mut Game)];
    [10675718] LavaSeahorse_Init [] {
        let E = Get_LavaSeahorse(word!(game.cur_enemy_index) as i32, game);
        (*E).var_02 = 0;
        if (*E).parameter_1 != 0 {
            set_fields! {[(*E)], 2; var_00, var_01}
            (*E).base.current_instruction = addr_kLavaSeahorse_Ilist_E5A1 as u16;
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
            (*E).var_F = (0xa2e781i32) as u16;
        } else {
            set_fields! {[(*E)], 0; var_00, var_01}
            (*E).base.current_instruction = addr_kLavaSeahorse_Ilist_E59B as u16;
            (*E).var_F = (0xa2e654i32) as u16;
        };
    };
    [10675790] LavaSeahorse_Main [] {
        let E = Get_LavaSeahorse(word!(game.cur_enemy_index) as i32, game);
        actor_species_pre_instruction((((*E).var_F) as u32) | (0xa20000u32), game);
    };
    [10676168] LavaSeahorse_Touch [] {
        Enemy_NormalTouchAI_A2(game);
        LavaSeahorse_E7DA(game);
    };
    [10676174] LavaSeahorse_Shot [] {
        Enemy_NormalShotAI_A2(game);
        LavaSeahorse_E7DA(game);
    };
    [10676180] LavaSeahorse_Powerbomb [] {
        Enemy_NormalPowerBombAI_A2(game);
        LavaSeahorse_E7DA(game);
    };
    @functions [pub fn (game: &mut Game)];
    [10657608] SpikeShootingPlant_Init [] { crate::game::spike_plant::initialize(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10657715] SpikeShootingPlant_Main [] {
        let phase = {
            let runtime = Runtime::new(game);
            runtime.resolve_actor(Key::from_legacy(runtime.word(0xe54)), actors::access::AccessMode::Native).pre_instruction
        };
        actor_species_pre_instruction(0xa20000 | u32::from(phase), game);
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_MaridiaSpikeyShell(word!(game.cur_enemy_index) as i32, game); };
    [10658809] MaridiaSpikeyShell_Init [] {
        (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as u16;
        let mssl_parameter_1_low: u16 = byte!((*E).parameter_1; 0) as u16;
        (*E).var_E = mssl_parameter_1_low;
        off_A2A3D3[(mssl_parameter_1_low as i32 & 1i32) as usize](
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
    [10659193] MaridiaSpikeyShell_Shot [] {
        if (*E).var_E as i32 - 1i32 & 0x8000 != 0 {
            Enemy_NormalShotAI_A2(game);
        }
    };
    @functions [pub fn (game: &mut Game)];
    [10662880] NorfairErraticFireball_Init [] { crate::game::erratic_fireball::update(true, game); };
    [10662927] NorfairErraticFireball_Main [] { crate::game::erratic_fireball::update(false, game); };
    [10665625] NorfairLavajumpingEnemy_Init [] { crate::game::lava_jumper::initialize(game); };
    [10661995] Flies_Init [] { crate::game::flies::initialize(game); };
    [10666562] NorfairRio_Init [] { crate::game::norfair_rio::initialize(false, game); };
    [10667763] LowerNorfairRio_Init [] { crate::game::norfair_rio::initialize(true, game); };
    @forward [pub unsafe fn (game: &mut Game)];
    [10662175] Flies_Main => update_randomized_pre_instruction(game);
    [10665682] NorfairLavajumpingEnemy_Main => update_randomized_pre_instruction(game);
    [10666615] NorfairRio_Main => update_randomized_pre_instruction(game);
    [10667812] LowerNorfairRio_Main => update_randomized_pre_instruction(game);
    [10649615] Enemy_GrappleReact_CancelBeam_A2 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [10649610] Enemy_GrappleReact_KillEnemy_A2 => actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Grapple);
    [10649665] Enemy_NormalFrozenAI_A2 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
    Enemy_NormalPowerBombAI_A2 => actors::combat::power_bomb_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    [10649635] Enemy_NormalTouchAI_A2 => actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    [10649645] Enemy_NormalShotAI_A2 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
    @external {
        10651420 => crate::game::goofball::initialize;
        10651548 => crate::game::goofball::advance;
    }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub unsafe fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10652030] MiniCrocomire_Instr_897E [jp.offset(1)] {
        let parameter = jp.word(0);
        let index = game.cur_enemy_index();
        SpawnEprojWithGfx(parameter, index, addr_Eproj_DBF2 as u16, game);
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    @prefix { let index = game.cur_enemy_index(); };
    [10652048]
    MiniCrocomire_Instr_8990 [jp] {  set_mini_crocomire_preinstruction(index, 0x8a43, 0, game); };
    [10652061]
    MiniCrocomire_Instr_899D [jp] {  set_mini_crocomire_preinstruction(index, 0x8a5c, 1, game); };
    @functions [pub unsafe fn (
    k: u16,
    jp: EnemyScript,
    game: &mut Game,
) -> EnemyScript];
    [10654593] MaridiaBeybladeTurtle_Instr_9381 [jp] {
        let mut R48: u16 = 0;
        if ({ let actor = word!(game.cur_enemy_index); 0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(&Runtime::new(game), Key::from_legacy(actor)))) }) != 0 {
            word!(game.extra_samus_x_displacement += (*Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game)).var_E);
            R48 = 1;
        }
        let E = Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game);
        crate::update_place! { previous R50: u16 = (*E).base.y_pos => (*E).var_D; }
        ({ let (actor, amount) = (word!(game.cur_enemy_index), ((*E).var_E as i16 as i32) << 16); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 });
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
            ({ let (actor, amount) = (word!(game.cur_enemy_index), (v8 as i16 as i32) << 16); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 });
            if R48 != 0 {
                word!(checked_i32, game.extra_samus_y_displacement += ((*Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game))
                        .base
                        .y_pos as i32
                        - R50 as i32));
            }
        }
    };
    [10654738]
    MaridiaBeybladeTurtle_Instr_9412 [EnemyScript::jump(addr_kMaridiaBeybladeTurtle_Ilist_8C72 as u16)] {
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
            return EnemyScript::jump(addr_kMaridiaBeybladeTurtle_Ilist_8B80 as u16);
        }
    };
    [10654791] MaridiaBeybladeTurtle_Instr_9447 [jp] {
        (*Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game)).var_A =
            (0xa28f3fi32) as u16;
    };
    [10654801]
    MaridiaBeybladeTurtle_Instr_9451 [EnemyScript::jump(addr_kMaridiaBeybladeTurtle_Ilist_8C02 as u16)] {
        let E = Get_MaridiaTurtle(word!(game.cur_enemy_index) as i32, game);
        (*E).var_A = (0xa28f8di32) as u16;
        (*E).var_E = -1i32 as u16;
        (*E).var_00 = 16;
    };
    [10654827]
    MaridiaBeybladeTurtle_Instr_946B [EnemyScript::jump(addr_kMaridiaBeybladeTurtle_Ilist_8C02 as u16)] {
        let E = Get_MaridiaTurtle(word!(game.cur_enemy_index) as i32, game);
        (*E).var_A = (0xa28f8di32) as u16;
        (*E).var_E = 1;
        (*E).var_00 = 16;
    };
    [10654853] MaridiaBeybladeTurtle_Instr_9485 [jp] {
        if ({ let actor = word!(game.cur_enemy_index); 0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(&Runtime::new(game), Key::from_legacy(actor)))) }) != 0 {
            if ((*Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game)).var_E
                & 0x8000)
                == 0
            {
                return EnemyScript::jump(addr_kMaridiaBeybladeTurtle_Ilist_8D40 as u16);
            } else {
                return EnemyScript::jump(addr_kMaridiaBeybladeTurtle_Ilist_8C62 as u16);
            }
        }
    };
    [10654881]
    MaridiaBeybladeTurtle_Instr_94A1 [EnemyScript::jump(addr_kMaridiaBeybladeTurtle_Ilist_8B80 as u16)] {
        let E = Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game);
        if ({ let actor = word!(game.cur_enemy_index); 0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(&Runtime::new(game), Key::from_legacy(actor)))) }) != 0 {
            (*E).var_A = (0xa2925ei32) as u16;
        } else {
            (*E).var_A = (0xa29142i32) as u16;
        }
        if ((*E).var_E & 0x8000) == 0 {
            return EnemyScript::jump(addr_kMaridiaBeybladeTurtle_Ilist_8C72 as u16);
        }
    };
    [10654919] MaridiaBeybladeTurtle_Instr_94C7 [jp] {
        let E = Get_MaridiaBeybladeTurtle(word!(game.cur_enemy_index) as i32, game);
        (*E).var_A = (0xa29239i32) as u16;
    };
    [10654929] MaridiaBeybladeTurtle_Instr_94D1 [jp] { QueueSfx2_Max6(0x3a, game); };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10657578] SpikeShootingPlant_Instr_9F2A [jp] { QueueSfx2_Max6(0x34, game); };
    [10657941] SpikeShootingPlant_Instr_A095 [jp] { crate::game::spike_plant::resume(k, game); };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10657959] SpikeShootingPlant_Instr_A0A7 [jp.offset(1)] {
        SpawnEprojWithGfx(
            jp.word(0),
            word!(game.cur_enemy_index),
            addr_kEproj_SpikeShootingPlantSpikes as u16,
            game,
        );
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10659181]
    MaridiaSpikeyShell_Instr_A56D [jp] { (*Get_MaridiaSpikeyShell(k as i32, game)).var_E = 0; };
    [10659185]
    MaridiaSpikeyShell_Instr_A571 [jp] { (*Get_MaridiaSpikeyShell(k as i32, game)).var_E = 1; };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10664371] Rinka_Instr_B9B3 [jp] {
        let E = Get_Rinka(k as i32, game);
        (*E).base.properties = ((*E).base.properties as i32
            | (kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
            as u16;
    };
    [10664381] Rinka_Instr_B9BD [jp] {
        let E = Get_Rinka(k as i32, game);
        (*E).base.properties = ((*E).base.properties as i32
            | (kEnemyProps_ProcessedOffscreen as i32
                | kEnemyProps_Tangible as i32
                | kEnemyProps_Invisible as i32)) as u16;
    };
    [10664391] Rinka_Instr_B9C7 [jp] {
        let E = Get_Rinka(k as i32, game);
        (*E).base.properties = ((*E).base.properties as i32
            & !(kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
            as u16;
        (*E).var_A = (0xa2b7dfi32) as u16;
        let E0 = Get_Rinka(0, game);
        (*E0).var_1E = (*E0).var_1E.wrapping_add(1);
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10664899] EnemyInstr_Rio_Instr_1 [jp] {
        crate::game::rio::instruction_finished(game);
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10665614] NorfairLavajumpingEnemy_Instr_BE8E [jp] { crate::game::lava_jumper::instruction_finished(game); };
    [10666441] NorfairRio_Instr_C1C9 [jp] {
        crate::game::norfair_rio::script_value(1, 1, game);
    };
    [10666452] NorfairRio_Instr_C1D4 [jp] {
        crate::game::norfair_rio::script_value(2, 8, game);
    };
    [10666463] NorfairRio_Instr_C1DF [jp] {
        crate::game::norfair_rio::script_value(2, 8, game);
    };
    [10666474] NorfairRio_Instr_C1EA [jp] {
        crate::game::norfair_rio::script_value(2, 12, game);
    };
    [10666485] NorfairRio_Instr_C1F5 [jp] {
        crate::game::norfair_rio::script_value(2, -12i32 as u16, game);
    };
    [10666496] NorfairRio_Instr_C200 [jp] {
        crate::game::norfair_rio::script_value(2, 4, game);
    };
    [10666507] NorfairRio_Instr_C20B [jp] {
        crate::game::norfair_rio::script_value(2, 0, game);
    };
    [10666518] NorfairRio_Instr_C216 [jp] {
        crate::game::norfair_rio::script_value(2, -4i32 as u16, game);
    };
    [10666529] NorfairRio_Instr_C221 [jp] {
        crate::game::norfair_rio::script_value(2, -12i32 as u16, game);
    };
    [10666540] NorfairRio_Instr_C22C [jp] {
        crate::game::norfair_rio::script_value(2, -16i32 as u16, game);
    };
    [10666551] NorfairRio_Instr_C237 [jp] {
        crate::game::norfair_rio::script_value(2, 12, game);
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10667730] LowerNorfairRio_Instr_C6D2 [jp] {
        crate::game::norfair_rio::script_value(1, 1, game);
    };
    [10667741] LowerNorfairRio_Instr_C6DD [jp] {
        crate::game::norfair_rio::script_value(2, 0, game);
    };
    [10667752] LowerNorfairRio_Instr_C6E8 [jp] {
        crate::game::norfair_rio::script_value(2, 1, game);
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10668907] MaridiaLargeSnail_Instr_CB6B [jp] { QueueSfx2_Max6(0xe, game); };
    [10669235] MaridiaLargeSnail_Instr_CCB3 [jp] {
        (*Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game)).var_02 = 1;
    };
    [10669246] MaridiaLargeSnail_Instr_CCBE [jp] {
        (*Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game)).var_03 = 1;
    };
    [10669257] MaridiaLargeSnail_Instr_CCC9 [jp] {
        (*Get_MaridiaLargeSnail(word!(game.cur_enemy_index) as i32, game)).var_03 = 0;
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10675707] LavaSeahorse_Instr_E5FB [] {
        (*Get_LavaSeahorse(word!(game.cur_enemy_index) as i32, game)).var_02 = 1;
        jp
    };
    @external {
        10651846 => [crate::game::goofball::landed(j, game)];
    }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (_k: u16, game: &mut Game)];
    [10652227] MiniCrocomire_PreInstr5 [] {
        let index = game.cur_enemy_index();
        let high = MiniCrocomireState::new(game, index).var_d();
        let low = MiniCrocomireState::new(game, index).var_c();
        MiniCrocomire_Func7((u32::from(high) << 16 | u32::from(low)) as i32, game);
        if MiniCrocomire_Func9(game) != 0 {
            MiniCrocomire_Func2(game);
        }
    };
    [10652252] MiniCrocomire_PreInstr6 [] {
        let index = game.cur_enemy_index();
        let high = MiniCrocomireState::new(game, index).var_b();
        let low = MiniCrocomireState::new(game, index).var_a();
        MiniCrocomire_Func7((u32::from(high) << 16 | u32::from(low)) as i32, game);
        if MiniCrocomire_Func9(game) != 0 {
            MiniCrocomire_Func4(game);
        }
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [10657722] SpikeShootingPlant_2 [] { crate::game::spike_plant::patrol(false, game); SpikeShootingPlant_4(game); };
    [10657772] SpikeShootingPlant_3 [] { crate::game::spike_plant::patrol(true, game); SpikeShootingPlant_4(game); };
    [10659852] GunshipTop_3 [] {
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
                (*E).var_F = (0xa2a8d0i32) as u16;
                (*E).var_E = 0;
            }
        };
    };
    [10660048] GunshipTop_4 [] {
        let GunshipTop = Get_GunshipTop(k as i32, game);
        let mut v2: i32 = (*GunshipTop).var_E as i32;
        word!(game.samus_y_pos += crate::rtl::rom_table_word!(game, 0xa2a622, v2 as isize));
        let v3 = Get_GunshipTop(k as i32 + 128, game);
        (*v3).base.y_pos = ((*v3).base.y_pos).wrapping_add(crate::rtl::rom_table_word!(
            game,
            0xa2a622,
            v2 as isize
        ));
        let v4 = Get_GunshipTop(k as i32 + 64, game);
        (*v4).base.y_pos = ((*v4).base.y_pos).wrapping_add(crate::rtl::rom_table_word!(
            game,
            0xa2a622,
            v2 as isize
        ));
        (*GunshipTop).base.y_pos = ((*GunshipTop).base.y_pos)
            .wrapping_add(crate::rtl::rom_table_word!(game, 0xa2a622, v2 as isize));
        let mut v5: u16 = ((*GunshipTop).var_E).wrapping_add(1);
        (*GunshipTop).var_E = v5;
        if v5 as i32 - 17i32 & 0x8000 == 0 {
            set_fields! {[(*GunshipTop)]; var_F = (0xa2a942i32) as u16,
            var_E = (*GunshipTop).base.y_pos, var_D = 1, };
            let v6 = Get_GunshipTop(0, game);
            (*v6).var_C = 0;
            word!(game.samus_x_pos = ((*GunshipTop).base.x_pos).wrapping_add(1));
            word!(game.samus_prev_x_pos = word!(game.samus_x_pos));
            let v7 = Get_GunshipTop(k as i32 + 128, game);
            enemy_instruction! {[(*v7).base]; timer = 1, instruction = addr_kGunshipTop_Ilist_A5BE as u16}
            (*v6).var_A = 144;
            QueueSfx3_Max6(0x14, game);
        }
    };
    [10660176] GunshipTop_6 [] {
        let E = Get_GunshipTop(k as i32, game);
        let r18: u16 = ((*E).var_E).wrapping_sub(30);
        if word!(game.samus_y_pos -= 1, reread) as i32 - r18 as i32 & 0x8000 != 0 {
            (*E).var_F = (0xa2a987i32) as u16;
            let E2 = Get_GunshipTop(k as i32 + 128, game);
            enemy_instruction! {[(*E2).base]; timer = 1, instruction = addr_kGunshipTop_Ilist_A5EE as u16}
            (*Get_GunshipTop(0i32, game)).var_A = 144;
            QueueSfx3_Max6(0x15, game);
        }
    };
    [10660285] GunshipTop_8 [] {
        if word!(game.game_state) == 8 && word!(game.frame_handler_alfa) as i32 == 0x90e695 & 0xffff
        {
            let E = Get_GunshipTop(k as i32, game);
            if (((*E).base.x_pos as i32 - 8i32 - word!(game.samus_x_pos) as i32) as i16) < 0
                && (((*E).base.x_pos as i32 + 8 - word!(game.samus_x_pos) as i32) as i16) >= 0
                && (((*E).base.y_pos as i32 - 64i32 - word!(game.samus_y_pos) as i32) as i16) < 0
                && (((*E).base.y_pos) as i16).wrapping_sub((word!(game.samus_y_pos)) as i16) >= 0
                && *ram_ptr!(game; 0xa1f) == 0
                && (word!(game.joypad1_newkeys) as i32 & kButton_Down as i32) != 0
            {
                (*E).var_F = (0xa2aa4fi32) as u16;
                if word!(game.samus_x_pos) != 1152 {
                    word!(game.samus_x_pos = (*E).base.x_pos);
                    word!(game.samus_prev_x_pos = word!(game.samus_x_pos));
                }
                player_face_forward(game);
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
    };
    [10660445] GunshipTop_10 [] {
        let E = Get_GunshipTop(k as i32, game);
        let r18: u16 = ((*E).var_E).wrapping_add(18);
        word!(game.samus_y_pos += 2);
        if word!(game.samus_y_pos) as i32 - r18 as i32 & 0x8000 == 0 {
            (*E).var_F = (0xa2aa94i32) as u16;
            let E2 = Get_GunshipTop(k as i32 + 128, game);
            enemy_instruction! {[(*E2).base]; timer = 1, instruction = addr_kGunshipTop_Ilist_A5EE as u16}
            (*Get_GunshipTop(0i32, game)).var_A = 144;
            QueueSfx3_Max6(0x15, game);
        }
    };
    [10660514] GunshipTop_12 [] {
        if CheckEventHappened(0xe, game) != 0 {
            (*Get_GunshipTop(k as i32, game)).var_F = (0xa2abc7i32) as u16;
            let E = Get_GunshipTop(k as i32 + 64, game);
            set_fields! {[(*E)], 0; var_F, var_E}
            word! { game, 0; substate, suit_pickup_light_beam_pos }
            *(ram_ptr!(game, u16; 0xdf0)) = 0;
            *(ram_ptr!(game, u16; 0xdf2)) = 0;
            CallSomeSamusCode(0xa, game);
        } else {
            crate::game::item_pickups_runtime::restore(crate::game::item_pickups::Resource::Health, 2, game);
            crate::game::item_pickups_runtime::restore(crate::game::item_pickups::Resource::Missiles, 2, game);
            crate::game::item_pickups_runtime::restore(crate::game::item_pickups::Resource::SuperMissiles, 2, game);
            crate::game::item_pickups_runtime::restore(crate::game::item_pickups::Resource::PowerBombs, 2, game);
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
                (*Get_GunshipTop(k as i32, game)).var_F = (0xa2ab1fi32) as u16;
            }
        };
    };
    [10660639] GunshipTop_13 [] {
        let rv: i32 = message_boxes::poll(0x1c, game);
        if rv < 0 {
            return;
        }
        if rv != 2 {
            crate::update_place! { unaligned_word fresh3 = &raw mut (*(ram_ptr!(game, u16; 0xd8f8))) => (u16::from_le(fresh3.read_unaligned())) | (1u16); }
            word!(game.load_station_index = 0);
            save_slots::save(word!(game.selected_save_slot), game);
        }
        (*Get_GunshipTop(k as i32, game)).var_F = (0xa2ab60i32) as u16;
        let E = Get_GunshipTop(k as i32 + 128, game);
        enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kGunshipTop_Ilist_A5BE as u16}
        (*Get_GunshipTop(0i32, game)).var_A = 144;
        QueueSfx3_Max6(0x14, game);
    };
    [10660718] GunshipTop_15 [] {
        let E = Get_GunshipTop(k as i32, game);
        let r18: u16 = ((*E).var_E).wrapping_sub(30);
        word!(game.samus_y_pos -= 2);
        if word!(game.samus_y_pos) as i32 - r18 as i32 & 0x8000 != 0 {
            (*E).var_F = (0xa2aba5i32) as u16;
            let E2 = Get_GunshipTop(k as i32 + 128, game);
            enemy_instruction! {[(*E2).base]; timer = 1, instruction = addr_kGunshipTop_Ilist_A5EE as u16}
            (*Get_GunshipTop(0i32, game)).var_A = 144;
            QueueSfx3_Max6(0x15, game);
        }
    };
    [10660807] GunshipTop_17 [] {
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
        word!(game.substate += 2);
        if word!(game.substate) as i32 - 10i32 & 0x8000 == 0 {
            (*Get_GunshipTop(k as i32, game)).var_F = (0xa2ac1bi32) as u16;
            word!(game.substate = 0);
        }
    };
    [10660891] GunshipTop_18 [] {
        let E1 = Get_GunshipTop(k as i32 + 64, game);
        let gtp_var_E: u16 = (*E1).var_E;
        if gtp_var_E as i32 - 64i32 & 0x8000 != 0 {
            let mut v3: u16 = if (gtp_var_E & 1) != 0 {
                (word!(game.samus_y_pos)).wrapping_add(1)
            } else {
                (word!(game.samus_y_pos)).wrapping_sub(1)
            };
            word! {game; samus_y_pos = v3; samus_prev_y_pos = v3; }
        } else {
            let mut v4: u16 = if (gtp_var_E & 1) != 0 {
                (word!(game.samus_y_pos)).wrapping_add(2)
            } else {
                (word!(game.samus_y_pos)).wrapping_sub(2)
            };
            word! {game; samus_y_pos = v4; samus_prev_y_pos = v4; }
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
            (*E0).var_F = (0xa2acd7i32) as u16;
            (*Get_GunshipTop(0i32, game)).var_A = 0;
        };
    };
    [10661079] GunshipTop_19 [] {
        word!(game.samus_y_pos -= 2);
        let mut v1: i16 = ((word!(game.samus_y_pos)) as i16).wrapping_sub(17);
        let E0 = Get_GunshipTop(k as i32, game);
        (*E0).base.y_pos = (word!(game.samus_y_pos)).wrapping_sub(17);
        (*Get_GunshipTop(k as i32 + 128i32, game)).base.y_pos = ((v1) as u16).wrapping_sub(1);
        let E1 = Get_GunshipTop(k as i32 + 64, game);
        (*E1).base.y_pos = (word!(game.samus_y_pos)).wrapping_add(23);
        if (*E0).base.y_pos as i32 - 896i32 & 0x8000 != 0 {
            (*E0).var_F = (0xa2ad0ei32) as u16;
            (*E1).var_F = 512;
        }
    };
    [10661134] GunshipTop_20 [] {
        GunshipTop_21(k, game);
        let E = Get_GunshipTop(k as i32, game);
        if (*E).base.y_pos as i32 - 256i32 & 0x8000 != 0 {
            (*E).var_F = (0xa2ad2di32) as u16;
            word!(game.game_state = 38);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
        }
    };
    [10661165] GunshipTop_21 [] {
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
        word! {game; samus_y_subpos = v6 as u16; samus_y_pos = (v6 >> 16) as u16; }
        let v5: u16 = (word!(game.samus_y_pos)).wrapping_sub(17);
        (*Get_GunshipTop(k as i32, game)).base.y_pos = v5;
        (*Get_GunshipTop(k as i32 + 128i32, game)).base.y_pos = (v5).wrapping_sub(1);
        (*E).base.y_pos = (word!(game.samus_y_pos)).wrapping_add(23);
    };
    [10675953] LavaSeahorse_Func_3 [] {
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
                (*E).var_F = (0xa2e734i32) as u16;
            }
        }
    };
    [10676020] LavaSeahorse_Func_4 [] {
        let E = Get_LavaSeahorse(k as i32, game);
        crate::update_place! { previous fresh21 = (*E).var_D => (*E).var_D.wrapping_sub(1); }
        if fresh21 == 1 {
            (*E).var_D = 48;
            (*E).var_F = (0xa2e749i32) as u16;
            LavaSeahorse_Func_6(game);
        }
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E = Get_GunshipTop(0, game); };
    [10660162] GunshipTop_5 [] {
        let v2: bool = (*E).var_A == 1;
        (*E).var_A = (*E).var_A.wrapping_sub(1);
        let v3: bool = ((*E).var_A & 0x8000) != 0;
        if v2 as i32 != 0 || v3 as i32 != 0 {
            (*Get_GunshipTop(k as i32, game)).var_F = (0xa2a950i32) as u16;
        }
    };
    [10660231] GunshipTop_7 [] {
        let v2: bool = (*E).var_A == 1;
        (*E).var_A = (*E).var_A.wrapping_sub(1);
        let v3: bool = ((*E).var_A & 0x8000) != 0;
        if v2 as i32 != 0 || v3 as i32 != 0 {
            (*Get_GunshipTop(k as i32, game)).var_F = (0xa2a9bdi32) as u16;
            word!(game.frame_handler_alfa = (0x90e695i32) as u16);
            word!(game.frame_handler_beta = (0x90e725i32) as u16);
            word!(game.loading_game_state = kLoadingGameState_5_Main as u16);
            crate::update_place! { unaligned_word fresh11 = &raw mut (*(ram_ptr!(game, u16; 0xd8f8))) => (u16::from_le(fresh11.read_unaligned())) | (1u16); }
            word!(game.load_station_index = 0);
            save_slots::save(word!(game.selected_save_slot), game);
        }
    };
    [10660431] GunshipTop_9 [] {
        let v2: bool = (*E).var_A == 1;
        (*E).var_A = (*E).var_A.wrapping_sub(1);
        let v3: bool = ((*E).var_A & 0x8000) != 0;
        if v2 as i32 != 0 || v3 as i32 != 0 {
            (*Get_GunshipTop(k as i32, game)).var_F = (0xa2aa5di32) as u16;
        }
    };
    [10660500] GunshipTop_11 [] {
        let v2: bool = (*E).var_A == 1;
        (*E).var_A = (*E).var_A.wrapping_sub(1);
        let v3: bool = ((*E).var_A & 0x8000) != 0;
        if v2 as i32 != 0 || v3 as i32 != 0 {
            (*Get_GunshipTop(k as i32, game)).var_F = (0xa2aaa2i32) as u16;
        }
    };
    [10660704] GunshipTop_14 [] {
        let v2: bool = (*E).var_A == 1;
        (*E).var_A = (*E).var_A.wrapping_sub(1);
        let v3: bool = ((*E).var_A & 0x8000) != 0;
        if v2 as i32 != 0 || v3 as i32 != 0 {
            (*Get_GunshipTop(k as i32, game)).var_F = (0xa2ab6ei32) as u16;
        }
    };
    [10660773] GunshipTop_16 [] {
        let v2: bool = (*E).var_A == 1;
        (*E).var_A = (*E).var_A.wrapping_sub(1);
        let v3: bool = ((*E).var_A & 0x8000) != 0;
        if v2 as i32 != 0 || v3 as i32 != 0 {
            (*Get_GunshipTop(k as i32, game)).var_F = (0xa2a9bdi32) as u16;
            if word!(game.game_state) as i32 - kGameState_40_TransitionToDemo as i32 & 0x8000 != 0 {
                word!(game.frame_handler_alfa = (0x90e695i32) as u16);
                word!(game.frame_handler_beta = (0x90e725i32) as u16);
            }
        }
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let E = Get_MaridiaLargeSnail(k as i32, game); };
    MaridiaLargeSnail_Func_2 [] {
        (*E).var_0B = 0;
        if ({ let actor = k; 0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(&Runtime::new(game), Key::from_legacy(actor)))) }) != 0 {
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
            word!(game.extra_samus_x_displacement += r18);
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
    [10669926] MaridiaLargeSnail_Func_7 [] {
        let mut v2: u16 = ((*E).var_B).wrapping_add(384);
        if v2 as i32 - 0x4000i32 & 0x8000 == 0 {
            v2 = 0x4000;
        }
        (*E).var_B = v2;
        let v5: i32 = 8 * (v2 as i32 >> 8) >> 1;
        if ({ let (actor, amount) = (k, *(rom_ptr!(game, u32; 0xa0cbc7)).offset((v5 >> 1) as isize) as i32); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0
        {
            (*E).var_E = (*E).var_E.wrapping_sub(1);
            let v6: i16 = (((*E).var_B) as i16).wrapping_sub(4096);
            if v6 < 0 {
                (*E).var_E = 0;
            }
            (*E).var_B = v6 as u16;
            (*E).var_F = (0xa2cfa9i32) as u16;
        }
    };
    [10669993] MaridiaLargeSnail_Func_8 [] {
        let v2: i16 = (((*E).var_B) as i16).wrapping_sub(384);
        (*E).var_B = v2 as u16;
        if v2 >= 0 {
            let v3: u16 = ((v2 as i32 & 0x7f00) >> 8) as u16;
            let v5: i32 = 8 * v3 as i32 >> 1;
            ({ let (actor, amount) = (k, *(rom_ptr!(game, u32; 0xa0cbc7)).offset((v5 + 2 >> 1) as isize) as i32); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 });
        } else {
            (*E).var_B = 0;
            (*E).var_F = (0xa2cf66i32) as u16;
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
    @functions [pub fn (k: u16, game: &mut Game)];
    [10665692] NorfairLavajumpingEnemy_Func_1 [] { crate::game::lava_jumper::follow_parent(k, game); };
    [10665754] NorfairLavajumpingEnemy_Func_2 [] { crate::game::lava_jumper::launch(k, game); };
    [10665790] NorfairLavajumpingEnemy_Func_3 [] { crate::game::lava_jumper::advance(k, crate::game::lava_jumper::Flight::Rising, game); };
    [10665852] NorfairLavajumpingEnemy_Func_4 [] { crate::game::lava_jumper::advance(k, crate::game::lava_jumper::Flight::Turning, game); };
    [10665916] NorfairLavajumpingEnemy_Func_5 [] { crate::game::lava_jumper::advance(k, crate::game::lava_jumper::Flight::Falling, game); };
    [10662222] Flies_4 [] { crate::game::flies::circle(k, false, game); };
    [10662268] Flies_5 [] { crate::game::flies::circle(k, true, game); };
    [10662314] Flies_6 [] { crate::game::flies::dive(k, false, game); };
    [10662354] Flies_7 [] { crate::game::flies::dive(k, true, game); };
    [10666625] NorfairRio_Func_1 [] { crate::game::norfair_rio::follow_parent(k, false, game); };
    [10666727] NorfairRio_Func_2 [] { crate::game::norfair_rio::wait_to_dive(k, false, game); };
    [10666815] NorfairRio_Func_3 [] { crate::game::norfair_rio::finish_animation(k, false, false, game); };
    [10667014] NorfairRio_Func_6 [] { crate::game::norfair_rio::finish_animation(k, false, true, game); };
    [10667822] LowerNorfairRio_Func_1 [] { crate::game::norfair_rio::follow_parent(k, true, game); };
    [10667889] LowerNorfairRio_Func_2 [] { crate::game::norfair_rio::wait_to_dive(k, true, game); };
    [10667963] LowerNorfairRio_Func_3 [] { crate::game::norfair_rio::finish_animation(k, true, false, game); };
    [10668168] LowerNorfairRio_Func_6 [] { crate::game::norfair_rio::finish_animation(k, true, true, game); };
    [10666849] NorfairRio_Func_4 [] { crate::game::norfair_rio::dive(k, false, true, game); };
    [10666929] NorfairRio_Func_5 [] { crate::game::norfair_rio::dive(k, false, false, game); };
    [10667990] LowerNorfairRio_Func_4 [] { crate::game::norfair_rio::dive(k, true, true, game); };
    [10668077] LowerNorfairRio_Func_5 [] { crate::game::norfair_rio::dive(k, true, false, game); };
    @countdown [pub unsafe fn (k: u16, game: &mut Game)];
    before {
        let E = Get_LavaSeahorse(k as i32, game);
        (*E).var_D = (*E).var_D.wrapping_sub(1);
    }
    when ((*E).var_D & 0x8000) != 0;
    [10675796] LavaSeahorse_Func_1 {
        (*E).var_D = 48;
        (*E).var_F = (0xa2e6adi32) as u16;
        let t: u16 = { let actor = k; actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) };
        (*E).var_A = (*E).var_A & 0x7fff | t & 0x8000;
        if ((*E).var_A & 0x8000) != 0 {
            crate::update_place! { reference fresh16 = *(&raw mut (*E).var_00 as *mut u8) => *fresh16 & !(1); }
            let E1 = Get_LavaSeahorse(k as i32 + 64, game);
            crate::update_place! { reference fresh17 = *(&raw mut (*E1).var_00 as *mut u8) => *fresh17 & !(1); }
        } else {
            crate::update_place! { reference fresh18 = *(&raw mut (*E).var_00 as *mut u8) => (*fresh18) | (1u8); }
            let E1_0 = Get_LavaSeahorse(k as i32 + 64, game);
            crate::update_place! { reference fresh19 = *(&raw mut (*E1_0).var_00 as *mut u8) => (*fresh19) | (1u8); }
        }
        LavaSeahorse_Func_6(game);
        LavaSeahorse_Func_7(game);
    };
    [10675885] LavaSeahorse_Func_2 {
        (*E).var_00 = ((*E).var_00).wrapping_add(4);
        (*E).var_D = 3;
        (*E).var_F = (0xa2e6f1i32) as u16;
    } after {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        let E1 = Get_LavaSeahorse(k as i32 + 64, game);
        (*E1).base.y_pos = (*E1).base.y_pos.wrapping_sub(1);
    };
    [10676041] LavaSeahorse_Func_5 {
        (*E).var_D = 128;
        (*E).var_F = (0xa2e654i32) as u16;
    } after {
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
        let E1 = Get_LavaSeahorse(k as i32 + 64, game);
        (*E1).base.y_pos = (*E1).base.y_pos.wrapping_add(1);
    };
}
