use crate::game::callback_tables::callback_table;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a0::*;
use crate::game::sm_80::{
    NextRandom, PrepareBitAccess, QueueMusic_Delayed8, QueueSfx1_Max1, QueueSfx1_Max15,
    QueueSfx1_Max3, QueueSfx1_Max6, QueueSfx1_Max9, QueueSfx2_Max1, QueueSfx2_Max15,
    QueueSfx2_Max3, QueueSfx2_Max6, QueueSfx2_Max9, QueueSfx3_Max1, QueueSfx3_Max15,
    QueueSfx3_Max3, QueueSfx3_Max6, QueueSfx3_Max9,
};
use crate::game::sm_81::{DrawEprojSpritemapWithBaseTile, DrawEprojSpritemapWithBaseTileOffscreen};
use crate::game::sm_84::CalculatePlmBlockCoords;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_91::{
    Samus_DealDamage, Samus_RestoreHealth, Samus_RestoreMissiles, Samus_RestorePowerBombs,
    Samus_RestoreSuperMissiles,
};
use crate::game::sm_94::{
    kSinCosTable8bit_Sext, scale_sine_word, square_slope_occupied, square_slope_single_block,
};
use crate::game::sm_a0::{
    CalculateAngleFromXY, CalculateAngleOfSamusFromEproj, CallEnemyAi, ConvertAngleToXy,
    SignExtend8, SpawnEnemyDrops, SuitDamageDivision,
};
use crate::game::sm_a6::Ridley_Func_103;
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::word;
use crate::rtl::abs16;
use crate::rtl::rom_ptr;
use crate::rtl::words;
use crate::rtl::AddToHiLo;
use crate::rtl::{Load24, Mult8x8, RomPtr, SnesDivide, SnesModulus, Unreachable};
use crate::set_fields;
use ::core::ffi::{c_long, c_void};
use ::core::ptr::{null, null_mut};

pub use crate::types::{
    EnemyBase, EnemyData, EnemyDef, EnemySpawnData, EprojDef, ExpandingSquareTransitionHdma,
    ExtraEnemyRam7800, ExtraEnemyRam8000, Func_V, Func_Y_V, LongPtr, Point16U, Ram7800_Kraid,
    Rect16U, VoidP, VramWriteEntry,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ExtraEnemyRam8800 {
    pub pad: [u8; 64],
}
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct Ram7800_Default {
        pub var_00, var_01, var_02, var_03, var_04, var_05, var_06, var_07, var_08, var_09, var_0A,
            var_0B, var_0C, var_0D, var_0E, var_0F, var_10, var_11, var_12, var_13, var_14, var_15,
            var_16, var_17, var_18, var_19, var_1A, var_1B, var_1C, var_1D, var_1E, var_1F: u16;
    }
}
pub use crate::types::Consts_86;
pub use crate::types::Consts_A0;
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct EprojCollInfo {
        pub eci_r20, eci_r24, eci_r26, eci_r28, eci_r30, eci_r32, eci_r34: u16;
    }
}
pub type Func_EprojCollInfo_U8 = unsafe fn(*mut EprojCollInfo, &mut Game) -> u8;
pub use crate::types::{Enemy_MotherBrain, Point32};
pub const NULL: *mut c_void = null_mut::<c_void>();
const RomPtr_86: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x86>;
const RomPtr_A0: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa0>;
const RomPtr_A1: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa1>;
const RomPtr_B4: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xb4>;
#[inline]
unsafe fn RomBankBase(bank: u8, game: &mut Game) -> *const u8 {
    RomPtr(
        (((bank as i32) << 16i32) as u32).wrapping_add(0x8000u32),
        game,
    )
    .offset(-(0x8000i32 as isize))
}
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
#[inline]
unsafe fn get_EnemyDef_A2(a: u16, game: &mut Game) -> *mut EnemyDef {
    RomPtr((0xa00000u32) | ((a) as u32), game) as *mut EnemyDef
}
#[inline]
unsafe fn get_EprojDef(a: u16, game: &mut Game) -> *mut EprojDef {
    RomPtr((0x860000u32) | ((a) as u32), game) as *mut EprojDef
}
crate::ram_map::ram_pointer_functions! {
    gRam7800_Default(a: u16) -> Ram7800_Default = [(0x7800 + a as i32) as isize];
}
crate::game::constants::typed_constants! { u32;
    fnEprojPreInstr_nullsub_83 = 8815867,
    fnEproj_Init_0x8bde_SkreeDownLeft = 8817429,
}
crate::game::constants::typed_constants! { i32;
    fnEprojPreInstr_Empty = 0x86efdf,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    gEnemySpawnData(o: i32) -> EnemySpawnData = [0x7000, o as isize];
    gExtraEnemyRam7800(o: i32) -> ExtraEnemyRam7800 = [0x7800, o as isize];
    gExtraEnemyRam8000(o: i32) -> ExtraEnemyRam8000 = [0x8000, o as isize];
    Get_MotherBrain(o: i32) -> Enemy_MotherBrain = [0xf78, o as isize];
}

// Shared projectile state stays backed by the original little-endian RAM bytes.
struct EprojState<'a>(&'a [u8]);
impl EprojState<'_> {
    fn enabled(&self) -> bool {
        u16::from_le_bytes([self.0[0x198d], self.0[0x198e]]) & 0x8000 != 0
    }
    crate::forward_calls! { fn (&self) -> u16;
        index => u16::from_le_bytes([self.0[0x1991], self.0[0x1992]]);
        init_param_1 => u16::from_le_bytes([self.0[0x1993], self.0[0x1994]]);
    }
    fn slot_word(&self, base: usize, word_index: usize) -> u16 {
        let address = base + word_index * 2;
        u16::from_le_bytes([self.0[address], self.0[address + 1]])
    }
    fn id(&self, word_index: usize) -> u16 {
        self.slot_word(0x1997, word_index)
    }
    fn preinstruction(&self, word_index: usize) -> u16 {
        self.slot_word(0x1a03, word_index)
    }
    fn instruction(&self, word_index: usize) -> u16 {
        self.slot_word(0x1b47, word_index)
    }
    fn properties(&self, word_index: usize) -> u16 {
        self.slot_word(0x1bd7, word_index)
    }
    fn spritemap(&self, word_index: usize) -> u16 {
        self.slot_word(0x1b6b, word_index)
    }
    fn gfx_low(&self, word_index: usize) -> u8 {
        self.0[0x19bb + word_index * 2]
    }
    fn x_radius(&self, word_index: usize) -> u8 {
        self.0[0x1bb3 + word_index * 2]
    }
    fn y_radius(&self, word_index: usize) -> u8 {
        self.0[0x1bb4 + word_index * 2]
    }
}

struct EprojStateMut<'a>(&'a mut [u8]);
impl EprojStateMut<'_> {
    fn decrement_byte(&mut self, base: usize, byte_index: usize) -> u8 {
        let byte = &mut self.0[base + byte_index];
        *byte = byte.wrapping_sub(1);
        *byte
    }
    fn write(&mut self, base: usize, word_index: usize, value: u16) {
        let address = base + word_index * 2;
        self.0[address..address + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn add_split_words(
        &mut self,
        high_base: usize,
        low_base: usize,
        word_index: usize,
        amount: u32,
    ) {
        let low = u32::from(EprojState(self.0).slot_word(low_base, word_index));
        let high = u32::from(EprojState(self.0).slot_word(high_base, word_index));
        let sum = (low | (high << 16)).wrapping_add(amount);
        self.write(low_base, word_index, sum as u16);
        self.write(high_base, word_index, (sum >> 16) as u16);
    }
    fn add_word(&mut self, base: usize, word_index: usize, delta: u16) -> u16 {
        let value = EprojState(self.0)
            .slot_word(base, word_index)
            .wrapping_add(delta);
        self.write(base, word_index, value);
        value
    }
    fn decrement(&mut self, base: usize, word_index: usize) -> u16 {
        let timer = EprojState(self.0).slot_word(base, word_index);
        self.write(base, word_index, timer.wrapping_sub(1));
        timer
    }
    fn decrement_instruction_timer(&mut self, word_index: usize) -> u16 {
        self.decrement(0x1b8f, word_index)
    }
    fn decrement_timer(&mut self, word_index: usize) -> u16 {
        self.decrement(0x19df, word_index)
    }
    fn set_timer(&mut self, word_index: usize, value: u16) {
        self.write(0x19df, word_index, value);
    }
    fn set_instruction_timer(&mut self, word_index: usize, value: u16) {
        self.write(0x1b8f, word_index, value);
    }
    fn set_spritemap(&mut self, word_index: usize, value: u16) {
        self.write(0x1b6b, word_index, value);
    }
    fn set_properties(&mut self, word_index: usize, value: u16) {
        self.write(0x1bd7, word_index, value);
    }
    fn set_property_bits(&mut self, word_index: usize, mask: u16) {
        let properties = EprojState(self.0).properties(word_index);
        self.set_properties(word_index, properties | mask);
    }
    fn set_preinstruction(&mut self, word_index: usize, value: u16) {
        self.write(0x1a03, word_index, value);
    }
    fn start_instruction(&mut self, word_index: usize, instruction: u16) {
        self.set_instruction(word_index, instruction);
        self.set_instruction_timer(word_index, 1);
    }
    fn set_instruction(&mut self, word_index: usize, value: u16) {
        self.write(0x1b47, word_index, value);
    }
    fn finish_pickup(&mut self, word_index: usize) {
        self.set_instruction(word_index, addr_word_86ECA3 as u16);
        self.set_instruction_timer(word_index, 1);
        self.set_properties(word_index, 0x3000);
        self.set_preinstruction(word_index, fnEprojPreInstr_Empty as u16);
    }
    fn clear_spawn_variables(&mut self, word_index: usize) {
        // E/F, timer, X/Y subpositions, then G, in the original store order.
        for base in [0x1aff, 0x1b23, 0x19df, 0x1a27, 0x1a6f, 0x1bfb] {
            self.write(base, word_index, 0);
        }
    }
    fn set_id(&mut self, word_index: usize, value: u16) {
        self.write(0x1997, word_index, value);
    }
    fn set_index(&mut self, value: u16) {
        self.write(0x1991, 0, value);
    }
}

pub fn EnableEprojs(game: &mut Game) {
    let flags = EprojState(&game.g_ram).slot_word(0x198d, 0) | 0x8000;
    EprojStateMut(&mut game.g_ram).write(0x198d, 0, flags);
}
pub unsafe fn DisableEprojs(game: &mut Game) {
    word!(game.cinematic_var5, |fresh17| (fresh17 & !(0x8000)));
}
pub fn ClearEprojs(game: &mut Game) {
    for word_index in (0..18).rev() {
        EprojStateMut(&mut game.g_ram).set_id(word_index, 0);
    }
}
unsafe fn SpawnEprojInner(j: u16, gfx_idx: u16, game: &mut Game) -> Option<u16> {
    let mut v3: u16 = 34;
    while EprojState(&game.g_ram).id(usize::from(v3 >> 1)) != 0 {
        v3 = v3.wrapping_sub(2);
        if (v3 & 0x8000) != 0 {
            return None;
        }
    }
    let v4 = usize::from(v3 >> 1);
    EprojStateMut(&mut game.g_ram).write(0x19bb, v4, gfx_idx);
    EprojStateMut(&mut game.g_ram).set_id(v4, j);
    let Edef = get_EprojDef(j, game);
    let value = (*Edef).pre_instr_ptr;
    EprojStateMut(&mut game.g_ram).set_preinstruction(v4, value);
    let value = (*Edef).instr_list;
    EprojStateMut(&mut game.g_ram).set_instruction(v4, value);
    let radius = (*Edef).radius;
    EprojStateMut(&mut game.g_ram).write(0x1bb3, v4, radius);
    let value = (*Edef).properties;
    EprojStateMut(&mut game.g_ram).set_properties(v4, value);
    EprojStateMut(&mut game.g_ram).set_instruction_timer(v4, 1);
    EprojStateMut(&mut game.g_ram).set_spritemap(v4, 0x8000);
    EprojStateMut(&mut game.g_ram).clear_spawn_variables(v4);
    CallEprojInit((((*Edef).init_code_ptr) as u32) | (0x860000u32), v3, game);
    Some(v3)
}
pub unsafe fn SpawnEprojWithGfx(a: u16, k: u16, j: u16, game: &mut Game) -> u16 {
    EprojStateMut(&mut game.g_ram).write(0x1993, 0, a);
    let enemy_index = k;
    let gfx_idx = (game.enemy_state(enemy_index).vram_tiles_index())
        | (game.enemy_state(enemy_index).palette_index());
    SpawnEprojInner(j, gfx_idx, game).unwrap_or(gfx_idx)
}
pub unsafe fn SpawnEprojWithRoomGfx(j: u16, a: u16, game: &mut Game) -> u16 {
    EprojStateMut(&mut game.g_ram).write(0x1993, 0, a);
    SpawnEprojInner(j, 0, game)
        .unwrap_or_else(|| u16::from_le_bytes([game.g_ram[0x1997], game.g_ram[0x1998]]))
}
unsafe fn EprojRunOne(k: u16, game: &mut Game) {
    CallEprojPreInstr(
        0x860000 | u32::from(EprojState(&game.g_ram).preinstruction(usize::from(k >> 1))),
        k,
        game,
    );
    let v1 = EprojState(&game.g_ram).index();
    let v2 = usize::from(EprojState(&game.g_ram).index() >> 1);
    let fresh20 = EprojStateMut(&mut game.g_ram).decrement_instruction_timer(v2);
    if fresh20 == 1 {
        let base = RomBankBase(0x86, game);
        let mut p = base.add(usize::from(EprojState(&game.g_ram).instruction(v2)));
        while *(p as *const u16) & 0x8000 != 0 {
            p = CallEprojInstr(
                0x860000 | u32::from(*(p as *const u16)),
                v1,
                p.offset(2),
                game,
            );
            if (p as usize) < 0x10000 {
                if p.is_null() {
                    return;
                }
                p = base.add(p as usize);
            }
        }
        let v7 = usize::from(v1 >> 1);
        let value = *(p as *mut u16);
        EprojStateMut(&mut game.g_ram).set_instruction_timer(v7, value);
        let value = *(p.offset(2) as *mut u16);
        EprojStateMut(&mut game.g_ram).set_spritemap(v7, value);
        let value = p.offset(4).offset_from(base) as c_long as u16;
        EprojStateMut(&mut game.g_ram).set_instruction(v7, value);
    }
}
pub unsafe fn EprojRunAll(game: &mut Game) {
    if EprojState(&game.g_ram).enabled() {
        let mut index = Some(34);
        while let Some(mut slot) = index {
            EprojStateMut(&mut game.g_ram).set_index(slot);
            if EprojState(&game.g_ram).id(usize::from(slot >> 1)) != 0 {
                EprojRunOne(slot, game);
                slot = EprojState(&game.g_ram).index();
            }
            index = slot.checked_sub(2);
        }
    }
}
crate::instruction_callbacks! { pub fn (k: u16, epjp: *const u8, game: &mut Game) -> *const u8;
    EprojInstr_Delete [null()] { EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0); };
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, epjp: *const u8, game: &mut Game) -> *const u8;
    EprojInstr_Sleep [null()] {
        let value = (epjp.offset_from(RomBankBase(0x86, game)) as c_long - 2 as c_long) as u16;
        EprojStateMut(&mut game.g_ram).set_instruction(usize::from(k >> 1), value);
    };
    EprojInstr_SetPreInstr_ [epjp.offset(2)] {
        let value = *(epjp as *mut u16);
        EprojStateMut(&mut game.g_ram).set_preinstruction(usize::from(k >> 1), value);
    };
}
pub fn EprojInstr_ClearPreInstr(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    EprojStateMut(&mut game.g_ram).set_preinstruction(usize::from(k >> 1), 0x8170);
    epjp
}
crate::game::address_dispatch! {
    pub unsafe fn CallEprojFunc(ea: u32, k: u32, game: &mut Game) => ea;
        call (); default { Unreachable(game); };
    {
        8833019 => [{
                    SpawnMotherBrainDeathBeam(k as u16, game);
                }];
    }
}
pub unsafe fn EprojInstr_CallFunc(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    CallEprojFunc(Load24(epjp as *mut LongPtr), k as u32, game);
    epjp.offset(3)
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, epjp: *const u8) -> *const u8;
    EprojInstr_Goto [*(epjp as *mut u16) as usize as *const u8] { };
    EprojInstr_GotoRel [epjp.offset(*epjp as i8 as i32 as isize)] { };
}
crate::instruction_callbacks! { pub unsafe fn (
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8;
    EprojInstr_DecTimerAndGotoIfNonZero [] {
        if EprojStateMut(&mut game.g_ram).decrement_timer(usize::from(k >> 1)) == 1 {
            epjp.offset(2)
        } else {
            EprojInstr_Goto(k, epjp)
        }
    };
    EprojInstr_DecTimerAndGotoRelIfNonZero [] {
        let v2: i32 = k as i32 >> 1;
        crate::update_place! { reference fresh46 [fresh47] = *(ram_ptr!(game, u16; 0x19df)).offset(v2 as isize) => (*fresh46).wrapping_sub(1); }
        if fresh47 == 1 {
            epjp.offset(1)
        } else {
            EprojInstr_GotoRel(k, epjp)
        }
    };
}
pub unsafe fn EprojInstr_SetTimer(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let value = *(epjp as *mut u16);
    EprojStateMut(&mut game.g_ram).set_timer(usize::from(k >> 1), value);
    epjp.offset(2)
}
crate::instruction_callbacks! { pub unsafe fn (
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8;
    EprojInstr_MoveRandomlyWithinRadius [epjp.offset(4)] {
        let mut v4: i8 = 0;
        let mut v6: i16 = 0;
        let r18: u16 = NextRandom(game);
        loop {
            let mut Random: i8 = NextRandom(game) as i8;
            v4 = ((*epjp as i32 & Random as i32) - *epjp.offset(1) as i32) as i8;
            if (v4 >= 0) {
                break;
            }
        }
        word!(game.eproj_x_pos[(k >> 1)], |fresh44| (fresh44 as i32
            + if r18 & 0x8000 != 0 {
                -(v4 as u8 as i32)
            } else {
                v4 as u8 as i32
            }) as u16);
        loop {
            *(&raw mut v6 as *mut u8) = NextRandom(game) as u8;
            *(&raw mut v6 as *mut u8) =
                ((*epjp.offset(2) as i32 & v6 as i32) - *epjp.offset(3) as i32) as u8;
            if ((v6 as u16 & 0x80) == 0) {
                break;
            }
        }
        v6 = v6 as u8 as i16;
        if (r18 & 0x4000) != 0 {
            v6 = -(v6 as u8 as i32) as i16;
        }
        word!(game.eproj_y_pos[(k >> 1)], wrapping_add((v6) as u16));
    };
    EprojInstr_SetProjectileProperties [epjp.offset(2)] {
        let slot = usize::from(k >> 1);
        let properties = EprojState(&game.g_ram).slot_word(0x1bd7, slot);
        let mask = *(epjp as *mut u16);
        EprojStateMut(&mut game.g_ram).set_properties(slot, properties | mask);
    };
    EprojInstr_ClearProjectileProperties [epjp.offset(2)] {
        let slot = usize::from(k >> 1);
        let properties = EprojState(&game.g_ram).slot_word(0x1bd7, slot);
        let mask = *(epjp as *mut u16);
        EprojStateMut(&mut game.g_ram).set_properties(slot, properties & mask);
    };
    EprojInstr_EnableCollisionWithSamusProj [epjp] {
        word!(game.eproj_properties[(k >> 1)], |fresh41| (fresh41
            | 0x8000));
    };
    EprojInstr_DisableCollisionWithSamusProj [epjp] {
        word!(game.eproj_properties[(k >> 1)], |fresh40| (fresh40
            & !(0x8000)));
    };
    EprojInstr_DisableCollisionWithSamus [epjp] {
        word!(game.eproj_properties[(k >> 1)], |fresh39| (fresh39
            | 0x2000));
    };
    EprojInstr_EnableCollisionWithSamus [epjp] {
        word!(game.eproj_properties[(k >> 1)], |fresh38| (fresh38
            & !(0x2000)));
    };
    EprojInstr_SetToNotDieOnContact [epjp] {
        word!(game.eproj_properties[(k >> 1)], |fresh37| (fresh37
            | 0x4000));
    };
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, epjp: *const u8, game: &mut Game) -> *const u8;
    EprojInstr_SetToDieOnContact [epjp] {
        word!(game.eproj_properties[(k >> 1)], |fresh36| (fresh36
            & !(0x4000)));
    };
    EprojInstr_SetLowPriority [epjp] {
        word!(game.eproj_properties[(k >> 1)], |fresh35| (fresh35
            | 0x1000));
    };
    EprojInstr_SetHighPriority [epjp] {
        word!(game.eproj_properties[(k >> 1)], |fresh34| (fresh34
            & !(0x1000)));
    };
    EprojInstr_SetXyRadius [epjp.offset(2)] {
        word!(game.eproj_radius[(k >> 1)] = *(epjp as *mut u16));
    };
    EprojInstr_SetXyRadiusZero [epjp] { word!(game.eproj_radius[(k >> 1)] = 0); };
}
crate::instruction_callbacks! { pub unsafe fn (
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8;
    EprojInstr_CalculateDirectionTowardsSamus [epjp] {
        let v2: i32 = k as i32 >> 1;
        let x: u16 = (word!(game.samus_x_pos)).wrapping_sub(word!(game.eproj_x_pos[v2]));
        let y: u16 = (word!(game.samus_y_pos)).wrapping_sub(word!(game.eproj_y_pos[v2]));
        let v3: u16 = (2u16).wrapping_mul(CalculateAngleFromXY(x, y, game));
        let v4: i32 = word!(game.cinematic_var7) as i32 >> 1;
        word!(game.eproj_E[v4] = v3);
        let v5: i32 = v3 as i32 >> 1;
        word!(game.eproj_x_vel[v4] = kSinCosTable8bit_Sext[(v5 + 64) as usize] as u16);
        word!(game.eproj_y_vel[v4] = kSinCosTable8bit_Sext[v5 as usize] as u16);
    };
    EprojInstr_WriteColorsToPalette [epjp.offset(5)] {
        let mut v3: u16 = *(epjp.offset(2) as *mut u16);
        let mut n: i32 = *epjp.offset(4) as i32;
        let mut v4: u16 = *(epjp as *mut u16);
        loop {
            word!(game.palette_buffer[(v3 >> 1)] = *(RomPtr_86(v4, game) as *mut u16));
            v4 = (v4 as i32 + 2) as u16;
            v3 = (v3 as i32 + 2) as u16;
            n -= 1;
            if ((n & 0x8000) != 0) {
                break;
            }
        }
    };
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, epjp: *const u8, game: &mut Game) -> *const u8;
    EprojInstr_QueueMusic [epjp.offset(1)] { QueueMusic_Delayed8(*epjp as u16, game); };
    EprojInstr_QueueSfx1_Max6 [epjp.offset(1)] { QueueSfx1_Max6(*epjp as u16, game); };
    EprojInstr_QueueSfx2_Max6 [epjp.offset(1)] { QueueSfx2_Max6(*epjp as u16, game); };
    EprojInstr_QueueSfx3_Max6 [epjp.offset(1)] { QueueSfx3_Max6(*epjp as u16, game); };
    EprojInstr_QueueSfx1_Max15 [epjp.offset(1)] { QueueSfx1_Max15(*epjp as u16, game); };
    EprojInstr_QueueSfx2_Max15 [epjp.offset(1)] { QueueSfx2_Max15(*epjp as u16, game); };
    EprojInstr_QueueSfx3_Max15 [epjp.offset(1)] { QueueSfx3_Max15(*epjp as u16, game); };
    EprojInstr_QueueSfx1_Max3 [epjp.offset(1)] { QueueSfx1_Max3(*epjp as u16, game); };
    EprojInstr_QueueSfx2_Max3 [epjp.offset(1)] { QueueSfx2_Max3(*epjp as u16, game); };
    EprojInstr_QueueSfx3_Max3 [epjp.offset(1)] { QueueSfx3_Max3(*epjp as u16, game); };
    EprojInstr_QueueSfx1_Max9 [epjp.offset(1)] { QueueSfx1_Max9(*epjp as u16, game); };
    EprojInstr_QueueSfx2_Max9 [epjp.offset(1)] { QueueSfx2_Max9(*epjp as u16, game); };
    EprojInstr_QueueSfx3_Max9 [epjp.offset(1)] { QueueSfx3_Max9(*epjp as u16, game); };
    EprojInstr_QueueSfx1_Max1 [epjp.offset(1)] { QueueSfx1_Max1(*epjp as u16, game); };
    EprojInstr_QueueSfx2_Max1 [epjp.offset(1)] { QueueSfx2_Max1(*epjp as u16, game); };
    EprojInstr_QueueSfx3_Max1 [epjp.offset(1)] { QueueSfx3_Max1(*epjp as u16, game); };
}
pub enum EprojPriority {
    Low,
    High,
}

pub unsafe fn DrawEprojsWithPriority(priority: EprojPriority, game: &mut Game) {
    let pt = GetValuesForScreenShaking(game);
    let low_priority = matches!(priority, EprojPriority::Low);
    for slot in (0..18).rev() {
        if EprojState(&game.g_ram).id(slot) != 0
            && (EprojState(&game.g_ram).properties(slot) & 0x1000 != 0) == low_priority
        {
            DrawEprojs((slot * 2) as u16, pt, game);
        }
    }
}
pub unsafe fn DrawEprojs(k: u16, pt: Point16U, game: &mut Game) {
    let v1 = usize::from(k >> 1);
    let v2: u16 = EprojState(&game.g_ram).spritemap(v1);
    let r26: u16 = EprojState(&game.g_ram).gfx_low(v1) as u16;
    let R28: u16 = EprojState(&game.g_ram).slot_word(0x19bb, v1) & 0xff00;
    let r20: u16 =
        (pt.x as i32 + game.enemy_projectile_x_pos(k) as i32 - game.layer1_x_pos() as i32) as u16;
    if (r20 as i32 + 128 & 0xfe00) == 0 {
        let v3: u16 = (pt.y as i32 + game.enemy_projectile_y_pos(k) as i32
            - game.layer1_y_pos() as i32) as u16;
        let r18: u16 = v3;
        if (v3 & 0xff00) != 0 {
            if (v3 as i32 + 128 & 0xfe00) == 0 {
                DrawEprojSpritemapWithBaseTileOffscreen(0x8d, v2, r20, r18, r26, R28, game);
            }
        } else {
            DrawEprojSpritemapWithBaseTile(0x8d, v2, r20, r18, r26, R28, game);
        }
    }
}
pub unsafe fn GetValuesForScreenShaking(game: &mut Game) -> Point16U {
    let mut R34: u16 = 0;
    let mut R36: u16 = 0;
    if word!(game.earthquake_timer) != 0
        && game.time_is_frozen_flag() == 0
        && word!(game.earthquake_type) as i32 - 36i32 & 0x8000 != 0
    {
        let v0: i32 = ((4u16).wrapping_mul(word!(game.earthquake_type))) as i32 >> 1;
        if (word!(game.earthquake_timer) & 2) != 0 {
            R36 = -(crate::rtl::rom_table_word!(game, 0x86846b, v0 as isize) as i32) as u16;
            R34 = -(crate::rtl::rom_table_word!(game, 0x86846b, (v0 + 1) as isize) as i32) as u16;
        } else {
            R36 = crate::rtl::rom_table_word!(game, 0x86846b, v0 as isize);
            R34 = crate::rtl::rom_table_word!(game, 0x86846b, (v0 + 1) as isize);
        }
    } else {
        R34 = 0;
        R36 = 0;
    }
    Point16U { x: R36, y: R34 }
}
unsafe fn EprojColl_8506(eci: *mut EprojCollInfo, game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) != 0 {
        word!(game.cur_block_index, |fresh1| (fresh1 as i32
            + *ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) as i8 as i32)
            as u16);
        return 0xff;
    }
    0
}
unsafe fn EprojColl_Unknown8536(eci: *mut EprojCollInfo, game: &mut Game) -> u8 {
    if *ram_ptr!(game; 0x16402, game.cur_block_index() as isize) != 0 {
        word!(game.cur_block_index, |fresh0| (fresh0 as i32
            + *ram_ptr!(game; 0x16402, word!(game.cur_block_index) as isize) as i8 as i32
                * word!(game.room_width_in_blocks) as i32)
            as u16);
        return 0xff;
    }
    0
}
unsafe fn EprojColl_858E(eci: *mut EprojCollInfo, game: &mut Game) -> u8 {
    let v0: u16 = ((*ram_ptr!(game; 0x16402, game.cur_block_index() as isize)) as u16) & (0x1fu16);
    if v0 < 5 {
        return EprojColl_85C2(eci, v0, word!(game.cur_block_index), game);
    }
    word!(
        game.current_slope_bts = *ram_ptr!(game; 0x16402, game.cur_block_index() as isize) as u16
    );
    EprojColl_873D()
}
unsafe fn EprojColl_85AD(eci: *mut EprojCollInfo, game: &mut Game) -> u8 {
    let v0: u16 = ((*ram_ptr!(game; 0x16402, game.cur_block_index() as isize)) as u16) & (0x1fu16);
    if v0 >= 5 {
        EprojColl_874E(eci, game)
    } else {
        EprojColl_8676(eci, v0, game.cur_block_index(), 0, game)
    }
}
#[derive(Copy, Clone)]
enum EprojCollisionAxis {
    Horizontal,
    Vertical,
}
impl EprojCollisionAxis {
    unsafe fn radius(self, eci: *const EprojCollInfo) -> u16 {
        match self {
            Self::Horizontal => (*eci).eci_r28,
            Self::Vertical => (*eci).eci_r30,
        }
    }
    unsafe fn perpendicular_radius(self, eci: *const EprojCollInfo) -> u16 {
        match self {
            Self::Horizontal => (*eci).eci_r30,
            Self::Vertical => (*eci).eci_r28,
        }
    }
    fn position(self, k: u16, game: &Game) -> u16 {
        match self {
            Self::Horizontal => game.enemy_projectile_x_pos(k),
            Self::Vertical => game.enemy_projectile_y_pos(k),
        }
    }
    fn perpendicular_position(self, k: u16, game: &Game) -> u16 {
        match self {
            Self::Horizontal => game.enemy_projectile_y_pos(k),
            Self::Vertical => game.enemy_projectile_x_pos(k),
        }
    }
    fn position_base(self) -> usize {
        match self {
            Self::Horizontal => 0x1a4b,
            Self::Vertical => 0x1a93,
        }
    }
    fn subposition_base(self) -> usize {
        match self {
            Self::Horizontal => 0x1a27,
            Self::Vertical => 0x1a6f,
        }
    }
    fn other_half(self) -> u16 {
        match self {
            Self::Horizontal => 2,
            Self::Vertical => 1,
        }
    }
}
// Keep context reads raw: the collision context may alias a destination store.
unsafe fn eproj_square_slope(
    eci: *mut EprojCollInfo,
    slope: u16,
    block: u16,
    axis: EprojCollisionAxis,
    game: &mut Game,
) -> u8 {
    crate::verify_probe::hit(match axis {
        EprojCollisionAxis::Horizontal => "square-slope.eproj.horizontal",
        EprojCollisionAxis::Vertical => "square-slope.eproj.vertical",
    });
    // Only the vertical original captures this index before the BTS/context reads.
    let captured_index = match axis {
        EprojCollisionAxis::Horizontal => None,
        EprojCollisionAxis::Vertical => Some(EprojState(&game.g_ram).index()),
    };
    let orientation = (*ram_ptr!(game; 0x16402, block as isize) >> 6) as u16;
    let shift = match axis {
        EprojCollisionAxis::Horizontal => 3,
        EprojCollisionAxis::Vertical => 2,
    };
    let index = slope
        .wrapping_mul(4)
        .wrapping_add(orientation ^ (((*eci).eci_r34 & 8) >> shift));
    let other = axis.other_half();
    // The original bank's table has the same solidity bits as bank 94.
    // Sampling performs no stores; retain raw context reads after the first store.
    let k = captured_index.unwrap_or_else(|| EprojState(&game.g_ram).index());
    let position = axis.perpendicular_position(k, game);
    let radius = axis.perpendicular_radius(eci);
    let occupied = if (*eci).eci_r32 == 0 {
        square_slope_single_block(index, position, radius, other)
    } else {
        square_slope_occupied(
            index,
            (*eci).eci_r26,
            (*eci).eci_r32,
            position,
            radius,
            other,
        )
    };
    if !occupied {
        return 0;
    }
    let slot = usize::from(EprojState(&game.g_ram).index() >> 1);
    EprojStateMut(&mut game.g_ram).write(axis.subposition_base(), slot, 0);
    let position = if (*eci).eci_r20 & 0x8000 != 0 {
        axis.radius(eci)
            .wrapping_add((*eci).eci_r34 | 7)
            .wrapping_add(1)
    } else {
        ((*eci).eci_r34 & 0xfff8).wrapping_sub(axis.radius(eci))
    };
    EprojStateMut(&mut game.g_ram).write(axis.position_base(), slot, position);
    1
}
unsafe fn EprojColl_85C2(eci: *mut EprojCollInfo, a: u16, k: u16, game: &mut Game) -> u8 {
    eproj_square_slope(eci, a, k, EprojCollisionAxis::Horizontal, game)
}
unsafe fn EprojColl_8676(eci: *mut EprojCollInfo, a: u16, k: u16, j: u16, game: &mut Game) -> u8 {
    eproj_square_slope(eci, a, k, EprojCollisionAxis::Vertical, game)
}
pub fn EprojColl_873D() -> u8 {
    0
}
unsafe fn EprojColl_874E(eci: *mut EprojCollInfo, game: &mut Game) -> u8 {
    let k = EprojState(&game.g_ram).index();
    let negative = (*eci).eci_r20 & 0x8000 != 0;
    let block = word!(game.cur_block_index);
    let column = SnesModulus(
        word!(game.cur_block_index),
        word!(game.room_width_in_blocks) as u8,
    );
    if game.enemy_projectile_x_pos(k) >> 4 != column {
        return 0;
    }
    let edge_offset = if negative {
        ((*eci).eci_r24.wrapping_sub((*eci).eci_r30) & 15) ^ 15
    } else {
        (*eci).eci_r30.wrapping_add((*eci).eci_r24).wrapping_sub(1) & 15
    };
    let row = u16::from(*ram_ptr!(game; 0x16402, block as isize) & 0x1f) * 16;
    let bts = *ram_ptr!(game; 0x16402, block as isize);
    if (bts & 0x80 != 0) != negative {
        return 0;
    }
    let x = if bts & 0x40 != 0 {
        game.enemy_projectile_x_pos(k) ^ 15
    } else {
        game.enemy_projectile_x_pos(k)
    };
    let index = row.wrapping_add(x & 15);
    // Both original ROM reads remain separate and occur before the landing stores.
    let height = *(rom_ptr!(game; 0x948b2b)).offset(index as isize) & 0x1f;
    let adjustment = i16::from(height)
        .wrapping_sub(edge_offset as i16)
        .wrapping_sub(1);
    let height = *(rom_ptr!(game; 0x948b2b)).offset(index as isize) & 0x1f;
    if i32::from(height) - i32::from(edge_offset) != 1 && adjustment >= 0 {
        return 0;
    }
    let position = if negative {
        (*eci).eci_r24.wrapping_sub(adjustment as u16)
    } else {
        (*eci).eci_r24.wrapping_add(adjustment as u16)
    };
    EprojStateMut(&mut game.g_ram).write(0x1a93, usize::from(k >> 1), position);
    EprojStateMut(&mut game.g_ram).write(
        0x1a6f,
        usize::from(k >> 1),
        if negative { 0 } else { 0xffff },
    );
    1
}
unsafe fn react_to_enemy_projectile_block(
    eci: *mut EprojCollInfo,
    k: u16,
    slope: Func_EprojCollInfo_U8,
    game: &mut Game,
) -> u8 {
    word!(game.cur_block_index = (k >> 1));
    game.react_to_block(|block_type, game| match block_type {
        0 | 2 | 3 | 4 | 6 | 7 | 14 => 0,
        1 => slope(eci, game),
        5 => EprojColl_8506(eci, game),
        8 | 9 | 10 | 11 | 12 | 15 => 1,
        13 => EprojColl_Unknown8536(eci, game),
        _ => unreachable!(),
    })
}
crate::forward_calls! { pub unsafe fn ( eci: *mut EprojCollInfo, k: u16, game: &mut Game, ) -> u8;
    EprojBlockCollisition_CheckHorizontal =>
        react_to_enemy_projectile_block(eci, k, EprojColl_858E, game);
    EprojBlockCollisition_CheckVertical =>
        react_to_enemy_projectile_block(eci, k, EprojColl_85AD, game);
}
unsafe fn eproj_block_collision(k: u16, axis: EprojCollisionAxis, game: &mut Game) -> u8 {
    let slot = usize::from(k >> 1);
    let velocity_base = match axis {
        EprojCollisionAxis::Horizontal => 0x1ab7,
        EprojCollisionAxis::Vertical => 0x1adb,
    };
    let amount = i32::from(EprojState(&game.g_ram).slot_word(velocity_base, slot) as i16) << 8;
    let y_radius = u16::from(EprojState(&game.g_ram).y_radius(slot));
    let x_radius = u16::from(EprojState(&game.g_ram).x_radius(slot));
    let (radius, perpendicular_radius) = match axis {
        EprojCollisionAxis::Horizontal => (x_radius, y_radius),
        EprojCollisionAxis::Vertical => (y_radius, x_radius),
    };
    let first_edge = axis
        .perpendicular_position(k, game)
        .wrapping_sub(perpendicular_radius)
        & 0xfff0;
    let span = (perpendicular_radius
        .wrapping_add(axis.perpendicular_position(k, game))
        .wrapping_sub(1)
        .wrapping_sub(first_edge))
        >> 4;
    // The horizontal path reads room width and perpendicular position before movement.
    let horizontal_product = match axis {
        EprojCollisionAxis::Horizontal => Mult8x8(
            (game.enemy_projectile_y_pos(k).wrapping_sub(y_radius) >> 4) as u8,
            game.room_width_in_blocks() as u8,
        ),
        EprojCollisionAxis::Vertical => 0,
    };
    let position = ((amount as u32).wrapping_add(
        (u32::from(axis.position(k, game)) << 16)
            | u32::from(EprojState(&game.g_ram).slot_word(axis.subposition_base(), slot)),
    ) >> 16) as u16;
    let subposition = (amount
        + i32::from(EprojState(&game.g_ram).slot_word(axis.subposition_base(), slot)))
        as u16;
    let edge = if amount < 0 {
        position.wrapping_sub(radius)
    } else {
        radius.wrapping_add(position).wrapping_sub(1)
    };
    let mut collision = EprojCollInfo {
        eci_r20: (amount >> 16) as u16,
        eci_r24: position,
        eci_r26: span,
        eci_r28: x_radius,
        eci_r30: y_radius,
        eci_r32: span,
        eci_r34: edge,
    };
    let mut cursor = match axis {
        EprojCollisionAxis::Horizontal => {
            i32::from((2 * (i32::from(horizontal_product) + i32::from(edge >> 4))) as u16)
        }
        EprojCollisionAxis::Vertical => {
            let product = Mult8x8((edge >> 4) as u8, game.room_width_in_blocks() as u8);
            let column = (game.enemy_projectile_x_pos(k).wrapping_sub(x_radius) >> 4) as i16;
            2 * (i32::from(product) + i32::from(column))
        }
    };
    loop {
        let hit = match axis {
            EprojCollisionAxis::Horizontal => {
                EprojBlockCollisition_CheckHorizontal(&raw mut collision, cursor as u16, game)
            }
            EprojCollisionAxis::Vertical => {
                EprojBlockCollisition_CheckVertical(&raw mut collision, cursor as u16, game)
            }
        };
        if hit & 1 != 0 {
            break;
        }
        if matches!(axis, EprojCollisionAxis::Horizontal) {
            cursor = i32::from(
                (cursor as u16).wrapping_add((i32::from(game.room_width_in_blocks()) * 2) as u16),
            );
        }
        collision.eci_r26 = collision.eci_r26.wrapping_sub(1);
        if collision.eci_r26 & 0x8000 != 0 {
            EprojStateMut(&mut game.g_ram).write(axis.subposition_base(), slot, subposition);
            EprojStateMut(&mut game.g_ram).write(axis.position_base(), slot, position);
            return 0;
        }
        if matches!(axis, EprojCollisionAxis::Vertical) {
            cursor += 2;
        }
    }
    EprojStateMut(&mut game.g_ram).write(axis.subposition_base(), slot, 0);
    if amount < 0 {
        let corrected = axis
            .radius(&raw const collision)
            .wrapping_add(collision.eci_r34 | 15)
            .wrapping_add(1);
        if corrected <= axis.position(k, game) {
            EprojStateMut(&mut game.g_ram).write(axis.position_base(), slot, corrected);
        }
    } else {
        let corrected =
            (collision.eci_r34 & 0xfff0).wrapping_sub(axis.radius(&raw const collision));
        if corrected >= axis.position(k, game) {
            EprojStateMut(&mut game.g_ram).write(axis.position_base(), slot, corrected);
        }
    }
    1
}
crate::forward_calls! { pub unsafe fn (k: u16, game: &mut Game) -> u8;
    EprojBlockCollisition_Horiz => eproj_block_collision(k, EprojCollisionAxis::Horizontal, game);
    EprojBlockCollisition_Vertical => eproj_block_collision(k, EprojCollisionAxis::Vertical, game);
}
unsafe fn Eproj_Init_0x8aaf(j: u16, game: &mut Game) {
    NextRandom(game);
    let v1: *mut EnemyData = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    let v2: u16 = ((*v1).y_pos as i32 + 12) as u16;
    let v3: i32 = j as i32 >> 1;
    word! { game;
        eproj_y_pos[v3] = v2;
        eproj_timers[v3] = (v2 as i32 + 72) as u16;
        eproj_y_vel[v3] = (*v1).ai_var_D;
        eproj_x_pos[v3] =
            ((*v1).x_pos as i32 + (word!(game.random_number) as i32 & 0x1f) - 16i32) as u16;
        eproj_instr_list_ptr[v3] = crate::rtl::rom_table_word!(
            game,
            0x868a75,
            ((*((ram_ptr!(game, u16; 0x1adb)).offset(v3 as isize) as *mut u8).offset(1,) as u16
                & 6)
                >> 1) as isize,
        );
    }
}
pub unsafe fn Eproj_PreInit_0x8aaf(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a93)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a6f)).offset(v1 as isize),
        ((word!(game.eproj_y_vel[v1]) as i16 as i32) << 8) as u32,
    );
    if word!(game.eproj_y_pos[v1]) >= word!(game.eproj_timers[v1]) {
        word!(game.eproj_id[v1] = 0);
    }
}
fn init_skree_particle(j: u16, x_offset: i16, x_velocity: i16, y_velocity: i16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let word_index = usize::from(j >> 1);
    let y = game.enemy_state(enemy_index).y_pos();
    EprojStateMut(&mut game.g_ram).write(0x1a93, word_index, y);
    EprojStateMut(&mut game.g_ram).write(0x1adb, word_index, y_velocity as u16);
    let x = game
        .enemy_state(enemy_index)
        .x_pos()
        .wrapping_add_signed(x_offset);
    EprojStateMut(&mut game.g_ram).write(0x1a4b, word_index, x);
    EprojStateMut(&mut game.g_ram).write(0x1ab7, word_index, x_velocity as u16);
}

pub fn Eproj_PreInstr_SkreeParticle(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let split_amount =
        ((EprojState(&game.g_ram).slot_word(0x1ab7, v1 as usize) as i16 as i32) << 8) as u32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a4b, 0x1a27, v1 as usize, split_amount);
    let split_amount =
        ((EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as i16 as i32) << 8) as u32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a93, 0x1a6f, v1 as usize, split_amount);
    EprojStateMut(&mut game.g_ram).add_word(0x1adb, v1 as usize, 80);
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        EprojStateMut(&mut game.g_ram).set_id(v1 as usize, 0);
    }
}
pub unsafe fn EprojInstr_SpawnEnemyDropsWithDraygonsEyeDrops(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    let v1: i32 = k as i32 >> 1;
    game.eproj_spawn_pt = Point16U {
        x: word!(game.eproj_x_pos[v1]),
        y: word!(game.eproj_y_pos[v1]),
    };
    SpawnEnemyDrops(addr_kEnemyDef_DE7F as u16, k, 0, game);
    epjp
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, epjp: *const u8, game: &mut Game) -> *const u8;
    EprojInstr_SetPreInstrA [epjp] {
        word!(game.eproj_pre_instr[(k >> 1)] = (0x868dff & 0xffff) as u16);
    };
    EprojInstr_SetPreInstrB [epjp] {
        word!(game.eproj_pre_instr[(k >> 1)] = (0x868dca & 0xffff) as u16);
    };
}
unsafe fn EprojInit_DraygonsGunk(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v1] = game.eproj_spawn_pt.x);
    word!(game.eproj_y_pos[v1] = game.eproj_spawn_pt.y);
    word!(game.g_word_7E97DC[v1] = word!(game.cinematic_var9));
    let pt: Point32 =
        ConvertAngleToXy(word!(game.cinematic_var9), word!(game.cinematic_var8), game);
    word!(game.eproj_x_vel[v1] = (pt.x >> 16) as u16);
    word!(game.eproj_E[v1] = pt.x as u16);
    word!(game.eproj_y_vel[v1] = (pt.y >> 16) as u16);
    word!(game.eproj_F[v1] = pt.y as u16);
    word!(game.eproj_gfx_idx[v1] = 1024);
}
unsafe fn EprojInit_DraygonsWallTurretProjs(j: u16, game: &mut Game) {
    Eproj_AngleToSamus(j, game.eproj_spawn_pt.x, game.eproj_spawn_pt.y, game);
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_gfx_idx[v1] = 2560);
    word!(game.eproj_pre_instr[v1] = (0x868d54 & 0xffff) as u16);
}
pub unsafe fn EprojPowerBombCollision(k: u16, game: &mut Game) {
    let r18: u16 = byte!(@pointer ram_ptr!(game, u16; 0xcea); 1) as u16;
    if byte!(@pointer ram_ptr!(game, u16; 0xcea); 1) != 0 {
        let r20: u16 = ((r18 as i32
            + (byte!(@pointer ram_ptr!(game, u16; 0xcea); 1) as i32 & 1)
            + (word!(game.power_bomb_explosion_radius) as i32 >> 9))
            as u16
            >> 1);
        let v1: i32 = k as i32 >> 1;
        if abs16(
            (word!(game.power_bomb_explosion_x_pos) as i32 - word!(game.eproj_x_pos[v1]) as i32)
                as u16,
        ) < r18
            && abs16(
                (word!(game.power_bomb_explosion_y_pos) as i32 - word!(game.eproj_y_pos[v1]) as i32)
                    as u16,
            ) < r20
        {
            word! { game, 0; eproj_id[v1], samus_x_speed_divisor }
        }
    }
}
pub unsafe fn EprojInstr_868D99(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    EprojPowerBombCollision(k, game);
    let v2: u16 = (word!(game.samus_x_speed_divisor) as i32 + 1) as u16;
    if word!(game.samus_x_speed_divisor) as i32 - 5i32 & 0x8000 != 0 {
        word!(game.samus_x_speed_divisor, wrapping_add(1));
        let v3: i32 = k as i32 >> 1;
        word!(game.eproj_F[v3] = v2);
        word!(game.eproj_E[v3] = 256);
        word!(game.eproj_pre_instr[v3] = (0x868dca & 0xffff) as u16);
        word!(game.eproj_properties[v3] = (word!(game.eproj_properties[v3]) & 0x5fff | 0x2000));
        word! { game, 0; samus_invincibility_timer, samus_knockback_timer }
    }
    epjp
}
pub unsafe fn EprojPreInstr_8DCA(k: u16, game: &mut Game) {
    EprojPowerBombCollision(k, game);
    if word!(game.samus_contact_damage_index) != 0 || {
        let mut v1: i32 = k as i32 >> 1;
        word! { game;
            eproj_x_pos[v1] = word!(game.samus_x_pos);
            eproj_y_pos[v1] = (word!(game.samus_y_pos) as i32 + 4 * word!(game.eproj_F[v1]) as i32
                - 12i32) as u16;
        }
        word!(game.eproj_E[v1], wrapping_sub(1));
        word!(game.eproj_E[v1]) == 0
    } {
        word!(game.eproj_id[(k >> 1)] = 0);
        if (word!(game.samus_x_speed_divisor, wrapping_sub(1), reread) & 0x8000) != 0 {
            word!(game.samus_x_speed_divisor = 0);
        }
    }
}
pub unsafe fn EprojPreInstr_DraygonsTurret_8DFF(k: u16, game: &mut Game) {
    EprojPowerBombCollision(k, game);
    Eproj_FuncE73E_MoveXY(k, game);
    let mut v1: i16 = Eproj_FuncE722(k, game) as i16;
    if v1 != 0 {
        word!(game.eproj_id[(k >> 1)] = 0);
    }
}
pub unsafe fn EprojPreInstr_DraygonsGunk_8E0F(k: u16, game: &mut Game) {
    EprojPowerBombCollision(k, game);
    Eproj_FuncE73E_MoveXY(k, game);
    let v1: i32 = k as i32 >> 1;
    let v2: u16 =
        abs16((word!(game.samus_x_pos) as i32 - word!(game.eproj_x_pos[v1]) as i32) as u16);
    if v2 as i32 - 16i32 & 0x8000 != 0 && {
        let mut v3: u16 =
            abs16((word!(game.samus_y_pos) as i32 - word!(game.eproj_y_pos[v1]) as i32) as u16);
        v3 as i32 - 20i32 & 0x8000 != 0
    } {
        word!(game.eproj_instr_list_ptr[v1] = 0x8c38);
        word!(game.eproj_instr_timers[v1] = 1);
    } else if Eproj_FuncE722(k, game) != 0 {
        word!(game.eproj_id[v1] = 0);
    }
}
unsafe fn EprojInit_CrocomireProjectile(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_vel[v2] = -512i32 as u16);
    word!(game.eproj_y_vel[v2] = 1);
    let v3 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    word!(game.eproj_x_pos[v2] = ((*v3).x_pos as i32 - 32i32) as u16);
    word!(game.eproj_y_pos[v2] = ((*v3).y_pos as i32 - 16i32) as u16);
    word! { game, 0; eproj_timers[v2], eproj_x_subpos[v2], eproj_y_subpos[v2] }
    word!(game.eproj_gfx_idx[v2] = 2560);
}
pub unsafe fn EprojPreInstr_CrocomireProjectile(k: u16, game: &mut Game) {
    EprojBlockCollisition_Horiz(k, game);
    word!(game.eproj_gfx_idx[0] = 2560);
    word!(game.eproj_timers[(k >> 1)], |fresh142| (fresh142 as i32
        + word!(game.eproj_x_vel[(k >> 1)]) as i32)
        as u16);
    let x: u16 = -64i32 as u16;
    let y: u16 = crate::rtl::rom_table_word!(
        game,
        0x869059,
        ((*(ram_ptr!(game, EnemyData; 0xf78))).ai_preinstr >> 1) as isize,
    );
    let v1: i32 = CalculateAngleFromXY(x, y, game) as i32;
    let v2: i32 = k as i32 >> 1;
    word!(game.eproj_x_vel[v2] = (4 * kSinCosTable8bit_Sext[(v1 + 64) as usize] as i32) as u16);
    word!(game.eproj_y_vel[v2] = (4 * kSinCosTable8bit_Sext[v1 as usize] as i32) as u16);
    word!(game.eproj_pre_instr[v2] = (0x8690b3 & 0xffff) as u16);
}
pub unsafe fn sub_8690B3(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) & 1 != 0
        || EprojBlockCollisition_Vertical(k, game) & 1 != 0
    {
        word!(game.eproj_id[(k >> 1)] = 0);
    }
}
unsafe fn EprojInit_CrocomireSpikeWallPieces(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(
        game.eproj_y_pos[v1] =
            crate::rtl::rom_table_word!(game, 0x869105, ((j as i32 - 20i32) as u16 >> 1) as isize,)
    );
    word!(game.eproj_x_pos[v1] = 528);
    word! { game, 0;
        eproj_x_vel[v1], eproj_y_vel[v1], eproj_E[v1], eproj_F[v1], eproj_x_subpos[v1],
        eproj_y_subpos[v1],
    }
    word!(game.eproj_y_vel[v1] = -5i32 as u16);
    word!(game.eproj_F[v1] = -30720i32 as u16);
}
static CrocomireSpikeWallPieces_Tab2: [u16; 18] = [
    0, 0, 0xff0, 0xee0, 0xcc0, 0xaa0, 0x880, 0x660, 0x440, 0x220, 0xff0, 0xee0, 0xcc0, 0xaa0,
    0x880, 0x660, 0x440, 0x220,
];
static CrocomireSpikeWallPieces_Tab1: [u16; 18] = [
    0, 0, 0xff00, 0xee00, 0xcc00, 0xaa00, 0x8800, 0x6600, 0x4400, 0x2200, 0xff00, 0xee00, 0xcc00,
    0xaa00, 0x8800, 0x6600, 0x4400, 0x2200,
];
static CrocomireSpikeWallPieces_Tab3: [i8; 36] = [
    0, 0, 0, 0, 4, 0, 4, 0, 3, 0, 3, 0, 2, 0, 2, 0, 1, 0, 1, 0, 6, 0, 5, 0, 4, 0, 3, 0, 2, 0, 2, 0,
    1, 0, 1, 0,
];
pub unsafe fn EprojPreInstr_CrocomireSpikeWallPieces(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let mut v2: u16 = word!(game.eproj_E[v1]);
    if v2 != CrocomireSpikeWallPieces_Tab1[v1 as usize] {
        v2 = (v2 as i32 + CrocomireSpikeWallPieces_Tab2[v1 as usize] as i32) as u16;
        if v2 >= CrocomireSpikeWallPieces_Tab1[v1 as usize] {
            v2 = CrocomireSpikeWallPieces_Tab1[v1 as usize];
        }
    }
    word!(game.eproj_E[v1] = v2);
    let mut v3: i8 = *(ram_ptr!(game, u16; 0x1aff) as *mut u8)
        .offset(k as i32 as isize)
        .offset(1) as i8;
    let mut v4: bool = u8::overflowing_add(
        byte!(@pointer ram_ptr!(game, u16; 0x1ab7); k as i32 as isize),
        v3 as u8,
    )
    .1;
    crate::update_place! { reference fresh138 = *(ram_ptr!(game, u16; 0x1ab7) as *mut u8).offset(k as i32 as isize) => (*fresh138 as i32 + v3 as i32) as u8; }
    let mut v5: u8 = (v4 as i32
        + *(ram_ptr!(game, u16; 0x1ab7) as *mut u8)
            .offset(k as i32 as isize)
            .offset(1) as i32) as u8;
    if ((v5 as i32 - CrocomireSpikeWallPieces_Tab3[k as usize] as i32) as i8) >= 0 {
        v5 = CrocomireSpikeWallPieces_Tab3[k as usize] as u8;
    }
    *(ram_ptr!(game, u16; 0x1ab7) as *mut u8)
        .offset(k as i32 as isize)
        .offset(1) = v5;
    let mut v6: i8 = byte!(@pointer ram_ptr!(game, u16; 0x1ab7); k as i32 as isize) as i8;
    v4 = u8::overflowing_add(
        *(ram_ptr!(game, u16; 0x1a27) as *mut u8)
            .offset(k as i32 as isize)
            .offset(1),
        v6 as u8,
    )
    .1;
    crate::update_place! { reference fresh139 = *(ram_ptr!(game, u16; 0x1a27) as *mut u8)
    .offset(k as i32 as isize)
    .offset(1) => (*fresh139 as i32 + v6 as i32) as u8; }
    let mut v7: i8 = *(ram_ptr!(game, u16; 0x1ab7) as *mut u8)
        .offset(k as i32 as isize)
        .offset(1) as i8;
    let v9: bool = v4;
    v4 = u8::overflowing_add(v4 as u8, v7 as u8).1;
    let mut v8: i8 = (v9 as i32 + v7 as i32) as i8;
    v4 = (v4 as i32
        | u8::overflowing_add(
            byte!(@pointer ram_ptr!(game, u16; 0x1a4b); k as i32 as isize),
            v8 as u8,
        )
        .1 as i32)
        != 0;
    crate::update_place! { reference fresh140 = *(ram_ptr!(game, u16; 0x1a4b) as *mut u8).offset(k as i32 as isize) => (*fresh140 as i32 + v8 as i32) as u8; }
    crate::update_place! { reference fresh141 = *(ram_ptr!(game, u16; 0x1a4b) as *mut u8)
    .offset(k as i32 as isize)
    .offset(1) => (*fresh141 as i32 + v4 as i32) as u8; }
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1adb)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1b23)).offset(v1 as isize),
        0x3000,
    );
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a93)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a6f)).offset(v1 as isize),
        words!((word!(game.eproj_y_vel[v1]), word!(game.eproj_F[v1]))),
    );
    if word!(game.eproj_y_pos[v1]) >= 0xa8 {
        word!(game.eproj_id[v1] = 0);
        if (k & 2) == 0 {
            QueueSfx2_Max6(0x29, game);
        }
        let v12: i32 = k as i32 >> 1;
        game.eproj_spawn_pt = Point16U {
            x: word!(game.eproj_x_pos[v12]),
            y: word!(game.eproj_y_pos[v12]),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15, game);
        QueueSfx2_Max6(0x25, game);
    }
}
pub unsafe fn EprojInstr_9270(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v1: i32 = k as i32 >> 1;
    game.eproj_spawn_pt = Point16U {
        x: word!(game.eproj_x_pos[v1]),
        y: word!(game.eproj_y_pos[v1]),
    };
    SpawnEnemyDrops(addr_kEnemyDef_DDBF as u16, k, 0, game);
    epjp
}
unsafe fn EprojInit_CrocomireBridgeCrumbling(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v1] = word!(game.cinematic_var8));
    word!(game.eproj_y_pos[v1] = 187);
    word! { game, 0; eproj_y_subpos[v1], eproj_x_subpos[v1], eproj_x_vel[v1] }
    word!(game.eproj_y_vel[v1] = ((word!(game.random_number) as i32 & 0x3f) + 64) as u16);
    word!(game.eproj_gfx_idx[v1] = 1024);
}
pub unsafe fn EprojPreInstr_CrocomireBridgeCrumbling(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) & 1 != 0 {
        word!(game.eproj_id[(k >> 1)] = 0);
    } else {
        word!(
            game.eproj_y_vel[(k >> 1)] =
                (word!(game.eproj_y_vel[(k >> 1)]) as i32 + 24 & 0x3fff) as u16
        );
    };
}
pub unsafe fn MoveEprojWithVelocity(k: u16, game: &mut Game) -> u16 {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = word!(game.eproj_x_vel[v1]);
    let carry: i32 = *(ram_ptr!(game, u16; 0x1a27) as *mut u8)
        .offset(k as i32 as isize)
        .offset(1) as i32
        + (v2 as i32 & 0xff);
    *(ram_ptr!(game, u16; 0x1a27) as *mut u8)
        .offset(k as i32 as isize)
        .offset(1) = carry as u8;
    word!(
        game.eproj_x_pos[v1],
        |fresh7| (fresh7 as i32 + ((v2 >> 8) as i8 as i32 + (carry >> 8))) as u16
    );
    MoveEprojWithVelocityY(k, game)
}
pub unsafe fn MoveEprojWithVelocityY(k: u16, game: &mut Game) -> u16 {
    let v1 = usize::from(k >> 1);
    let v2: u16 = word!(game.eproj_y_vel[v1]);
    let (fraction, carry) =
        (*(ram_ptr!(game; 0x1a6f)).offset(k as isize).offset(1)).overflowing_add(v2 as u8);
    *(ram_ptr!(game; 0x1a6f)).offset(k as isize).offset(1) = fraction;
    let result = game
        .enemy_projectile_y_pos(k)
        .wrapping_add_signed(((v2 as i16) >> 8) + carry as i16);
    word!(game.eproj_y_pos[v1] = result);
    result
}
pub unsafe fn MoveEprojWithVelocityX(k: u16, game: &mut Game) -> u16 {
    let v1 = usize::from(k >> 1);
    let v2: u16 = word!(game.eproj_x_vel[v1]);
    let (fraction, carry) =
        (*(ram_ptr!(game; 0x1a27)).offset(k as isize).offset(1)).overflowing_add(v2 as u8);
    *(ram_ptr!(game; 0x1a27)).offset(k as isize).offset(1) = fraction;
    let result = game
        .enemy_projectile_x_pos(k)
        .wrapping_add_signed(((v2 as i16) >> 8) + carry as i16);
    word!(game.eproj_x_pos[v1] = result);
    result
}
pub unsafe fn SetAreaDependentEprojPropertiesEx(k: u16, j: u16, game: &mut Game) {
    let p = RomPtr_86(k, game) as *mut u16;
    let mut v2: u16 = if game.area_index() == 2 {
        *p.offset(1)
    } else if game.area_index() == 5 {
        *p.offset(2)
    } else {
        *p
    };
    EprojStateMut(&mut game.g_ram).set_properties(usize::from(j >> 1), v2);
}
static kEprojInit_9634_Xvel: [i16; 4] = [-0x200i16, -0x1f0i16, -0x1bci16, -0x16ai16];
static kEprojInit_9634_Yvel: [i16; 4] = [0, 0x88, 0xfc, 0x16a];
unsafe fn EprojInit_9634(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word! { game, 0; eproj_E[v1], eproj_F[v1] }
    word! { game;
        eproj_x_pos[v1] = ((*(ram_ptr!(game, EnemyData; 0xf78))).x_pos as i32 - 29i32) as u16;
        eproj_y_pos[v1] = ((*(ram_ptr!(game, EnemyData; 0xf78))).y_pos as i32 - 35i32) as u16;
        eproj_gfx_idx[v1] = 2560;
    }
    let v2: i32 = (*(ram_ptr!(game, EnemyData; 0xf78))).parameter_1 as i32;
    word!(game.eproj_x_vel[v1] = kEprojInit_9634_Xvel[v2 as usize] as u16);
    word!(game.eproj_y_vel[v1] = kEprojInit_9634_Yvel[v2 as usize] as u16);
}
pub unsafe fn EprojPreInstr_9634(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    if word!(game.eproj_E[v1]) >= 8 {
        MoveEprojWithVelocity(k, game);
        if EprojBlockCollisition_Vertical(k, game) & 1 != 0 {
            word!(game.eproj_instr_list_ptr[v1] = addr_off_869574 as u16);
            word!(game.eproj_E[v1], wrapping_add(1));
            word!(game.eproj_instr_timers[v1] = 1);
            word! { game, 0; eproj_x_vel[v1], eproj_y_vel[v1] }
            word!(game.eproj_gfx_idx[v1] = 2560);
            QueueSfx2_Max6(0x2b, game);
        }
    } else {
        word!(game.eproj_E[v1], wrapping_add(1));
    };
}
unsafe fn EprojInit_9642_RidleysFireball(j: u16, game: &mut Game) {
    let slot = usize::from(j >> 1);
    let parameter = EprojState(&game.g_ram).slot_word(0x1995, 0);
    EprojStateMut(&mut game.g_ram).write(0x1b23, slot, parameter);
    let v2: i16 = (if EprojState(&game.g_ram).init_param_1() != 0 {
        25
    } else {
        -25i32
    }) as i16;
    let x = (game.enemy_state(0).x_pos()).wrapping_add((v2) as u16);
    EprojStateMut(&mut game.g_ram).write(0x1a4b, slot, x);
    let y = (game.enemy_state(0).y_pos()).wrapping_sub(43);
    EprojStateMut(&mut game.g_ram).write(0x1a93, slot, y);
    EprojStateMut(&mut game.g_ram).write(0x19bb, slot, 2560);
    let x_velocity = EprojState(&game.g_ram)
        .slot_word(0x7800 + core::mem::offset_of!(Ram7800_Default, var_19), 0);
    EprojStateMut(&mut game.g_ram).write(0x1ab7, slot, x_velocity);
    let y_velocity = EprojState(&game.g_ram)
        .slot_word(0x7800 + core::mem::offset_of!(Ram7800_Default, var_1A), 0);
    EprojStateMut(&mut game.g_ram).write(0x1adb, slot, y_velocity);
    SetAreaDependentEprojPropertiesEx(addr_kRidleysFireball_Tab0 as u16, j, game);
}
pub unsafe fn SetAreaDependentEprojProperties(j: u16, game: &mut Game) {
    SetAreaDependentEprojPropertiesEx(addr_kRidleysFireball_Tab0 as u16, j, game);
}
fn SetEprojSpawnPoint(k: u16, game: &mut Game) {
    let x = game.enemy_projectile_x_pos(k);
    let y = game.enemy_projectile_y_pos(k);
    game.eproj_spawn_pt = Point16U { x, y };
}
pub unsafe fn EprojPreInstr_9642_RidleysFireball(k: u16, game: &mut Game) {
    let mut v1: u16 = if EprojBlockCollisition_Horiz(k, game) & 1 != 0 {
        -27042i32 as u16
    } else {
        if EprojBlockCollisition_Vertical(k, game) & 1 == 0 {
            return;
        }
        -27056i32 as u16
    };
    let v2: i32 = k as i32 >> 1;
    EprojStateMut(&mut game.g_ram).set_id(v2 as usize, 0);
    if EprojState(&game.g_ram).slot_word(0x1b23, v2 as usize) == 0 {
        SetEprojSpawnPoint(k, game);
        SpawnEprojWithRoomGfx(v1, 3, game);
        QueueSfx2_Max6(0x2b, game);
    }
}
pub unsafe fn EprojInstr_DisableCollisionsWithSamus(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    word!(game.eproj_properties[(k >> 1)], |fresh32| (fresh32
        | 0x2000));
    epjp
}
fn EprojInit_9660_FireballExplosion(j: u16, game: &mut Game) {
    let slot = usize::from(j >> 1);
    EprojStateMut(&mut game.g_ram).write(0x19bb, slot, 2560);
    let x = game.eproj_spawn_pt.x;
    EprojStateMut(&mut game.g_ram).write(0x1a4b, slot, x);
    let y = game.eproj_spawn_pt.y;
    EprojStateMut(&mut game.g_ram).write(0x1a93, slot, y);
    let parameter = EprojState(&game.g_ram).init_param_1();
    EprojStateMut(&mut game.g_ram).write(0x1aff, slot, parameter);
    EprojStateMut(&mut game.g_ram).write(0x1b23, slot, 0);
    EprojStateMut(&mut game.g_ram).write(0x1ab7, slot, 0);
    EprojStateMut(&mut game.g_ram).write(0x1adb, slot, 0);
}
unsafe fn EprojInit_Common(j: u16, k: u16, a: u16, game: &mut Game) {
    let slot = usize::from(j >> 1);
    EprojStateMut(&mut game.g_ram).write(0x1b23, slot, a);
    let x = game.enemy_projectile_x_pos(k);
    EprojStateMut(&mut game.g_ram).write(0x1a4b, slot, x);
    let y = game.enemy_projectile_y_pos(k);
    EprojStateMut(&mut game.g_ram).write(0x1a93, slot, y);
    let e = EprojState(&game.g_ram).slot_word(0x1aff, usize::from(k >> 1));
    EprojStateMut(&mut game.g_ram).write(0x1aff, slot, e);
    EprojStateMut(&mut game.g_ram).write(0x19bb, slot, 2560);
    SetAreaDependentEprojProperties(j, game);
}
unsafe fn EprojInit_FireballChild(j: u16, x: u16, y: u16, handler: u16, game: &mut Game) {
    let parent = EprojState(&game.g_ram).init_param_1();
    let slot = usize::from(j >> 1);
    EprojStateMut(&mut game.g_ram).write(0x1ab7, slot, x);
    EprojStateMut(&mut game.g_ram).write(0x1adb, slot, y);
    EprojInit_Common(j, parent, handler, game);
}
crate::forward_calls! { unsafe fn (j: u16, game: &mut Game);
    EprojInit_9688 => EprojInit_FireballChild(j, 0, (-3584i16) as u16, 0x9688, game);
    EprojInit_9696 => EprojInit_FireballChild(j, 0, 3584, 0x9696, game);
    EprojInit_966C => EprojInit_FireballChild(j, 3584, 0, 0x966c, game);
    EprojInit_967A => EprojInit_FireballChild(j, (-3584i16) as u16, 0, 0x967a, game);
}
pub unsafe fn EprojPreInstr_966C(k: u16, game: &mut Game) {
    MoveEprojWithVelocityX(k, game);
    if EprojBlockCollisition_Vertical(k, game) & 1 != 0 {
        let slot = usize::from(k >> 1);
        EprojStateMut(&mut game.g_ram).set_instruction(slot, addr_off_869574 as u16);
        EprojStateMut(&mut game.g_ram).set_instruction_timer(slot, 1);
    }
}
pub unsafe fn EprojPreInstr_9688(k: u16, game: &mut Game) {
    MoveEprojWithVelocityY(k, game);
    if EprojBlockCollisition_Horiz(k, game) & 1 != 0 {
        let slot = usize::from(k >> 1);
        EprojStateMut(&mut game.g_ram).set_instruction(slot, addr_off_869574 as u16);
        EprojStateMut(&mut game.g_ram).set_instruction_timer(slot, 1);
    }
}
pub unsafe fn EprojPreInstr_96A4(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) & 1 != 0 {
        word!(game.eproj_id[(k >> 1)] = 0);
    }
}
pub unsafe fn EprojPreInstr_96C0(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) & 1 != 0 {
        word!(game.eproj_id[(k >> 1)] = 0);
    }
}
pub use self::EprojPreInstr_96C0 as EprojPreInstr_96CE;
crate::instruction_callbacks! { pub unsafe fn (k: u16, epjp: *const u8, game: &mut Game) -> *const u8;
    EprojInstr_95BA [epjp] {
        EprojStateMut(&mut game.g_ram).write(0x1995, 0, 0);
        SpawnEprojWithRoomGfx(addr_stru_86966C as u16, k, game);
        EprojStateMut(&mut game.g_ram).write(0x1995, 0, 0);
        SpawnEprojWithRoomGfx(addr_stru_86967A as u16, k, game);
    };
    EprojInstr_95ED [epjp] {
        EprojStateMut(&mut game.g_ram).write(0x1995, 0, 0);
        SpawnEprojWithRoomGfx(addr_stru_869688 as u16, k, game);
        EprojStateMut(&mut game.g_ram).write(0x1995, 0, 0);
        SpawnEprojWithRoomGfx(addr_stru_869696 as u16, k, game);
    };
    EprojInstr_9620 [epjp] {
        let count = EprojStateMut(&mut game.g_ram).decrement_byte(0x1aff, usize::from(k));
        if count as i8 >= 0 {
            let handler = EprojState(&game.g_ram).slot_word(0x1b23, usize::from(k >> 1));
            SpawnEprojWithRoomGfx(handler, k, game);
        }
    };
}
fn EprojInit_9734_CeresFallingDebris(j: u16, game: &mut Game) {
    let slot = usize::from(j >> 1);
    EprojStateMut(&mut game.g_ram).write(0x1aff, slot, 0);
    EprojStateMut(&mut game.g_ram).write(0x1b23, slot, 0);
    EprojStateMut(&mut game.g_ram).write(0x1ab7, slot, 0);
    EprojStateMut(&mut game.g_ram).write(0x19bb, slot, 3584);
    let x = EprojState(&game.g_ram).init_param_1();
    EprojStateMut(&mut game.g_ram).write(0x1a4b, slot, x);
    EprojStateMut(&mut game.g_ram).write(0x1a93, slot, 42);
    EprojStateMut(&mut game.g_ram).write(0x1adb, slot, 16);
}
pub unsafe fn EprojPreInstr_9734_CeresFallingDebris(k: u16, game: &mut Game) {
    let slot = usize::from(k >> 1);
    EprojStateMut(&mut game.g_ram).add_word(0x1adb, slot, 16);
    if EprojBlockCollisition_Vertical(k, game) & 1 != 0 {
        EprojStateMut(&mut game.g_ram).set_id(slot, 0);
        SetEprojSpawnPoint(k, game);
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, game);
        QueueSfx2_Max6(0x6d, game);
    }
}
pub unsafe fn EprojInstr_980E(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v2: i32 = k as i32 >> 1;
    game.eproj_spawn_pt = Point16U {
        x: word!(game.eproj_x_pos[v2]),
        y: word!(game.eproj_y_pos[v2]),
    };
    SpawnEnemyDrops(addr_kEnemyDef_E4FF as u16, k, 0, game);
    epjp
}
static byte_8698B4: [u8; 16] = [
    0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0xf0, 0xe0, 0xd0, 0xc0, 0xb0, 0xa0, 0x90, 0x80,
];
static byte_8698F7: [u8; 9] = [0x30, 0x44, 0x58, 0x6c, 0x80, 0x94, 0xa8, 0xbc, 0xd0];
static byte_869979: [u8; 8] = [0, 0x20, 0x40, 0x60, 0x80, 0xa0, 0xc0, 0xe0];
unsafe fn EprojInit_PhantoonDestroyableFireballs(j: u16, game: &mut Game) {
    let mut v1: i8 = ((word!(game.cinematic_var8) & 0xff00) >> 8) as i8;
    match v1 as i32 {
        0 => {
            let v2: i32 = j as i32 >> 1;
            word! { game, 0;
                eproj_x_subpos[v2], eproj_y_subpos[v2], eproj_x_vel[v2], eproj_y_vel[v2],
            }
            word! { game;
                eproj_x_pos[v2] = (*(ram_ptr!(game, EnemyData; 0xf78))).x_pos;
                eproj_y_pos[v2] = ((*(ram_ptr!(game, EnemyData; 0xf78))).y_pos as i32 + 32) as u16;
                eproj_instr_list_ptr[v2] = addr_off_8697B4 as u16;
                eproj_properties[v2] = (word!(game.eproj_properties[v2]) & 0xfff | 0x2000);
            }
        }
        2 => {
            let v3: i32 = j as i32 >> 1;
            word! { game, 0; eproj_x_subpos[v3], eproj_y_subpos[v3], eproj_y_vel[v3] }
            let v4: u16 = word!(game.cinematic_var8) as u8 as u16;
            if ((word!(game.cinematic_var8) as u8 as i32 - 8i32) as i16) >= 0 {
                word!(game.eproj_x_vel[v3] = -2i32 as u16);
            } else {
                word!(game.eproj_x_vel[v3] = 2);
            }
            word! { game;
                eproj_E[v3] = byte_8698B4[v4 as usize] as u16;
                eproj_x_pos[v3] = (*(ram_ptr!(game, EnemyData; 0xf78))).x_pos;
                eproj_y_pos[v3] = ((*(ram_ptr!(game, EnemyData; 0xf78))).y_pos as i32 + 32) as u16;
                eproj_pre_instr[v3] = (0x869a45 & 0xffff) as u16;
            }
        }
        4 => {
            let v5: i32 = j as i32 >> 1;
            word! { game, 0; eproj_x_subpos[v5], eproj_y_subpos[v5], eproj_y_vel[v5] }
            word!(game.eproj_x_vel[v5] = ((word!(game.cinematic_var8) as i32 & 0xf0) >> 1) as u16);
            word!(
                game.eproj_x_pos[v5] =
                    byte_8698F7[(word!(game.cinematic_var8) as i32 & 0xf) as usize] as u16
            );
            word!(game.eproj_y_pos[v5] = 40);
            word!(game.eproj_pre_instr[v5] = (0x869a94 & 0xffff) as u16);
        }
        6 => {
            let v7: i32 = j as i32 >> 1;
            word! { game, 0; eproj_x_subpos[v7], eproj_y_subpos[v7], eproj_y_vel[v7] }
            word! { game;
                eproj_x_vel[v7] = 128;
                eproj_E[v7] = byte_869979[word!(game.cinematic_var8) as u8 as usize] as u16;
                eproj_x_pos[v7] = (*(ram_ptr!(game, EnemyData; 0xf78))).x_pos;
                eproj_y_pos[v7] = ((*(ram_ptr!(game, EnemyData; 0xf78))).y_pos as i32 + 16) as u16;
                eproj_pre_instr[v7] = (0x869ada & 0xffff) as u16;
            }
        }
        _ => {
            Unreachable(game);
        }
    };
}
unsafe fn EprojInit_PhantoonStartingFireballs(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word! { game, 0;
        eproj_x_subpos[v1], eproj_y_subpos[v1], eproj_x_vel[v1], eproj_y_vel[v1],
    }
    let v2: u16 = byte_869979[word!(game.cinematic_var8) as usize] as u16;
    word!(game.eproj_E[v1] = v2);
    let pt: Point16U = Eproj_PhantomFireballs_Func1(v2, 0x30);
    word! { game;
        eproj_x_pos[v1] = (pt.x as i32 + (*(ram_ptr!(game, EnemyData; 0xf78))).x_pos as i32) as u16;
        eproj_y_pos[v1] =
            (pt.y as i32 + (*(ram_ptr!(game, EnemyData; 0xf78))).y_pos as i32 + 16) as u16;
    }
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballs(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word!(game.eproj_y_vel[v1], |fresh133| (fresh133 as i32 + 16)
        as u16);
    if EprojBlockCollisition_Vertical(k, game) & 1 != 0 {
        word!(game.eproj_properties[v1] = (word!(game.eproj_properties[v1]) & 0xfff | 0x8000));
        word!(game.eproj_pre_instr[v1] = (0x8699bf & 0xffff) as u16);
        word!(game.eproj_instr_list_ptr[v1] = addr_word_86976C as u16);
        word!(game.eproj_instr_timers[v1] = 1);
        word!(game.eproj_E[v1] = 8);
        word!(game.eproj_y_pos[v1], |fresh134| (fresh134 as i32 + 8)
            as u16);
    }
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballs2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let v2: bool = word!(game.eproj_E[v1]) == 1;
    let v3: bool = (word!(game.eproj_E[v1], wrapping_sub(1), reread) & 0x8000) != 0;
    if v2 || v3 {
        word!(game.eproj_pre_instr[v1] = (0x869a01 & 0xffff) as u16);
        word!(game.eproj_instr_list_ptr[v1] = addr_word_869772 as u16);
        word!(game.eproj_instr_timers[v1] = 1);
        word!(game.eproj_y_pos[v1], |fresh132| (fresh132 as i32 - 8i32)
            as u16);
        word!(game.eproj_y_vel[v1] = -768i32 as u16);
        word!(game.eproj_E[v1] = 0);
        if (word!(game.nmi_frame_counter_word) & 1) != 0 {
            word!(game.eproj_x_vel[v1] = -128i32 as u16);
        } else {
            word!(game.eproj_x_vel[v1] = 128);
        }
    }
}
static word_869A3E: [u16; 3] = [0xfd00, 0xfe00, 0xff00];
pub unsafe fn EprojPreInstr_PhantoonStartingFireballs3(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word!(game.eproj_y_vel[v1], |fresh129| (fresh129 as i32 + 16)
        as u16);
    let switch_fireball_phase: bool = if EprojBlockCollisition_Vertical(k, game) & 1 != 0 {
        let v2: u16 = (word!(game.eproj_E[v1]) as i32 + 1) as u16;
        word!(game.eproj_E[v1] = v2);
        if v2 as i32 - 3i32 & 0x8000 != 0 {
            word!(game.eproj_y_vel[v1] = word_869A3E[v2 as usize]);
            return;
        }
        true
    } else {
        (EprojBlockCollisition_Horiz(k, game) & 1 != 0)
    };
    if switch_fireball_phase {
        word!(game.eproj_instr_list_ptr[v1] = addr_word_869782 as u16);
        word!(game.eproj_instr_timers[v1] = 1);
        word!(game.eproj_pre_instr[v1] = (0x869a44 & 0xffff) as u16);
    };
}

macro_rules! move_phantoon_orbiting_fireball {
    ($game:ident, $slot:ident, $angle:ident, $y_position:ident) => {
        word!($game.eproj_E[$slot] = $angle);
        let pt: Point16U =
            Eproj_PhantomFireballs_Func1($angle, word!($game.eproj_y_vel[$slot]));
        let mut v3: bool = ((pt.x as i32
            + (*(ram_ptr!($game, EnemyData; 0xf78))).x_pos as i32)
            as i16)
            < 0;
        let mut v4: i16 = (pt.x as i32
            + (*(ram_ptr!($game, EnemyData; 0xf78))).x_pos as i32)
            as i16;
        word!(
            $game.eproj_x_pos[$slot] = (pt.x as i32
                + (*(ram_ptr!($game, EnemyData; 0xf78))).x_pos as i32)
                as u16
        );
        if v3
            || v4 as i32 - 256i32 & 0x8000 == 0
            || {
                v3 = ((pt.y as i32
                    + (*(ram_ptr!($game, EnemyData; 0xf78))).y_pos as i32
                    + 16) as i16)
                    < 0;
                $y_position = (pt.y as i32
                    + (*(ram_ptr!($game, EnemyData; 0xf78))).y_pos as i32
                    + 16) as u16;
                word!($game.eproj_y_pos[$slot] = $y_position);
                v3 as i32 != 0
            }
            || $y_position as i32 - 256i32 & 0x8000 == 0
        {
            word!($game.eproj_instr_list_ptr[$slot] = addr_off_8697F8 as u16);
            word!($game.eproj_instr_timers[$slot] = 1);
        }
    };
}

pub unsafe fn EprojPreInstr_PhantoonDestroyableFireballs(k: u16, game: &mut Game) {
    let mut v5: u16 = 0;
    let v1: i32 = k as i32 >> 1;
    word!(game.eproj_y_vel[v1], |fresh128| (fresh128 as i32 + 4)
        as u16);
    let v2: u16 = (byte!(@pointer (ram_ptr!(game, u16; 0x1ab7)).offset(v1 as isize); 0) as i32
        + word!(game.eproj_E[v1]) as i32) as u8 as u16;
    move_phantoon_orbiting_fireball!(game, v1, v2, v5);
}
pub unsafe fn EprojPreInstr_PhantoonDestroyableFireballs_2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let apply_gravity: bool = (word!(game.eproj_x_vel[v1]) == 0)
        || ({
            let mut v2: bool = word!(game.eproj_x_vel[v1]) == 1;
            let mut v3: bool = (word!(game.eproj_x_vel[v1], wrapping_sub(1), reread) & 0x8000) != 0;
            (v2 || v3)
                && ({
                    QueueSfx3_Max6(0x1d, game);
                    true
                })
        });
    if apply_gravity {
        word!(game.eproj_y_vel[v1], |fresh126| (fresh126 as i32 + 16)
            as u16);
        if EprojBlockCollisition_Vertical(k, game) & 1 != 0 {
            word!(game.eproj_instr_list_ptr[v1] = addr_word_8697AC as u16);
            word!(game.eproj_instr_timers[v1] = 1);
            word!(game.eproj_y_pos[v1], |fresh127| (fresh127 as i32 + 8)
                as u16);
            word!(game.eproj_pre_instr[v1] = (0x869a44 & 0xffff) as u16);
            QueueSfx3_Max6(0x1d, game);
        }
    };
}
pub unsafe fn EprojPreInstr_PhantoonDestroyableFireballs_3(k: u16, game: &mut Game) {
    let mut v5: u16 = 0;
    let v1: i32 = k as i32 >> 1;
    word!(game.eproj_y_vel[v1], |fresh123| (fresh123 as i32 + 2)
        as u16);
    let v2: u16 = (word!(game.eproj_E[v1]) as i32 + 2) as u8 as u16;
    move_phantoon_orbiting_fireball!(game, v1, v2, v5);
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballsB(k: u16, game: &mut Game) {
    if (*(ram_ptr!(game, EnemyData; 0xf78))).ai_var_B != 0 {
        let v1: i32 = k as i32 >> 1;
        word!(game.eproj_pre_instr[v1] = (0x869b41 & 0xffff) as u16);
        word!(game.eproj_x_vel[v1] = 180);
        word!(game.eproj_y_vel[v1] = 48);
    }
}
pub unsafe fn EprojPreInstr_PhantoonStartingFireballsB_2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = word!(game.eproj_x_vel[v1]);
    if v2 != 0 {
        word!(game.eproj_x_vel[v1] = (v2 as i32 - 1i32) as u16);
    } else if ((word!(game.nmi_frame_counter_word) & 1) != 0) {
        let v3: u16 = (word!(game.eproj_y_vel[v1]) as i32 - 1i32) as u16;
        word!(game.eproj_y_vel[v1] = v3);
        if (v3 == 0) {
            word! { game;
                eproj_x_pos[v1] = (*(ram_ptr!(game, EnemyData; 0xf78))).x_pos;
                eproj_y_pos[v1] = ((*(ram_ptr!(game, EnemyData; 0xf78))).y_pos as i32 + 16) as u16;
                eproj_instr_timers[v1] = 1;
                eproj_instr_list_ptr[v1] = addr_off_8697F8 as u16;
            }
            return;
        }
    }
    let v4: u8 = (word!(game.eproj_E[v1]) as i32 + 1) as u8;
    word!(game.eproj_E[v1] = v4 as u16);
    let pt: Point16U = Eproj_PhantomFireballs_Func1(v4 as u16, word!(game.eproj_y_vel[v1]));
    word! { game;
        eproj_x_pos[v1] = (pt.x as i32 + (*(ram_ptr!(game, EnemyData; 0xf78))).x_pos as i32) as u16;
        eproj_y_pos[v1] =
            (pt.y as i32 + (*(ram_ptr!(game, EnemyData; 0xf78))).y_pos as i32 + 16) as u16;
    }
}
pub unsafe fn Eproj_PhantomFireballs_Func1(j: u16, a: u16) -> Point16U {
    let r24: u16 = a;
    let r26: u16 = j;
    let mut v2: u16 = if j as i32 - 128i32 & 0x8000 != 0 {
        scale_sine_word((2 * j as i32) as u16, r24)
    } else {
        -(scale_sine_word((2i32 * (j as i32 + 0x80i32) as u8 as i32) as u16, r24) as i32) as u16
    };
    let r20: u16 = v2;
    let mut v3: i16 = (r26 as i32 - 64i32) as u8 as i16;
    let mut v4: u16 = if v3 as i32 - 128i32 & 0x8000 != 0 {
        scale_sine_word((2 * v3 as i32) as u16, r24)
    } else {
        -(scale_sine_word((2i32 * (v3 as i32 + 0x80i32) as u8 as i32) as u16, r24) as i32) as u16
    };
    let r22: u16 = v4;
    Point16U { x: r20, y: r22 }
}
unsafe fn EprojInit_RocksKraidSpits(j: u16, game: &mut Game) {
    let enemy_index = word!(game.cur_enemy_index);
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v2] = (game.enemy_state(enemy_index).x_pos()).wrapping_add(16));
    word!(game.eproj_y_pos[v2] = (game.enemy_state(enemy_index).y_pos()).wrapping_sub(96));
    word! { game, 0; eproj_y_subpos[v2], eproj_x_subpos[v2] }
    word!(game.eproj_x_vel[v2] = word!(game.cinematic_var8));
    word!(game.eproj_y_vel[v2] = -1024i32 as u16);
    word!(game.eproj_gfx_idx[v2] = 1536);
}
unsafe fn EprojInit_RocksFallingKraidCeiling(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v1] = word!(game.cinematic_var8));
    word!(game.eproj_y_pos[v1] = 312);
    word! { game, 0; eproj_y_subpos[v1], eproj_x_subpos[v1], eproj_x_vel[v1] }
    word!(game.eproj_y_vel[v1] = (word!(game.random_number) & 0x3f).wrapping_add(64));
    word!(game.eproj_gfx_idx[v1] = 1536);
}
unsafe fn EprojInit_RocksWhenKraidRises(j: u16, game: &mut Game) {
    let mut v1: i16 = ((word!(game.random_number)) as i16) & (0x3fi16);
    if (word!(game.random_number) & 1) == 0 {
        v1 = !(v1 as i32) as i16;
    }
    let v2: i32 = j as i32 >> 1;
    word!(
        game.eproj_x_pos[v2] = ({
            let enemy_index = word!(game.cur_enemy_index);
            game.enemy_state(enemy_index).x_pos()
        })
        .wrapping_add((v1) as u16)
    );
    word!(game.eproj_y_pos[v2] = 432);
    word! { game, 0; eproj_y_subpos[v2], eproj_x_subpos[v2] }
    word!(game.eproj_x_vel[v2] = word!(game.cinematic_var8));
    word!(game.eproj_y_vel[v2] = -1280i32 as u16);
    word!(game.eproj_gfx_idx[v2] = 1536);
    QueueSfx3_Max6(0x1e, game);
}
pub unsafe fn EprojPreInstr_KraidRocks(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) & 1 != 0
        || EprojBlockCollisition_Vertical(k, game) & 1 != 0
    {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    } else {
        let v1: i32 = k as i32 >> 1;
        let mut v2: u16 = EprojStateMut(&mut game.g_ram).add_word(0x1ab7, v1 as usize, 8);
        if v2 as i32 - 256i32 & 0x8000 == 0 {
            v2 = -256i32 as u16;
        }
        word!(game.eproj_x_vel[v1] = v2);
        EprojStateMut(&mut game.g_ram).add_word(0x1adb, v1 as usize, 64);
    };
}
pub unsafe fn EprojPreInstr_RocksFallingKraidCeiling(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) & 1 != 0 {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    } else {
        word!(
            game.eproj_y_vel[(k >> 1)] =
                ((word!(game.eproj_y_vel[(k >> 1)]) as i32 + 24) as u16) & (0x3fffu16)
        );
    };
}
pub unsafe fn sub_869DA5(k: u16, game: &mut Game) {
    word!(game.eproj_gfx_idx[(k >> 1)] = 0);
}
unsafe fn EprojInit_MiniKraidSpit(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let extra_index = word!(game.cur_enemy_index);
    let v3: i32 = j as i32 >> 1;
    word!(
        game.eproj_x_pos[v3] = (game.mini_kraid_spit_state(extra_index).x_offset() as i32
            + game.enemy_state(enemy_index).x_pos() as i32) as u16
    );
    word!(game.eproj_y_pos[v3] = (game.enemy_state(enemy_index).y_pos() as i32 - 16i32) as u16);
    word! { game, 0; eproj_y_subpos[v3], eproj_x_subpos[v3] }
    word!(game.eproj_x_vel[v3] = game.mini_kraid_spit_state(extra_index).x_velocity());
    word!(game.eproj_y_vel[v3] = game.mini_kraid_spit_state(extra_index).y_velocity());
}
pub unsafe fn EprojPreInit_MiniKraidSpit(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) & 1 != 0
        || EprojBlockCollisition_Vertical(k, game) & 1 != 0
    {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    } else {
        let v1: i32 = k as i32 >> 1;
        let mut v2: i16 = ((word!(game.eproj_y_vel[v1])) as i16).wrapping_add(64);
        if v2 >= 0 && word!(game.eproj_y_vel[v1]) as i32 - 960i32 & 0x8000 == 0 {
            v2 = 1024;
        }
        word!(game.eproj_y_vel[v1] = v2 as u16);
    };
}
static kEprojInit_MiniKraidSpikes_Tab0: [i16; 3] = [-2i16, 12, 24];
unsafe fn EprojInit_MiniKraidSpikes(j: u16, a: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_vel[v2] = a);
    let r18: u16 =
        kEprojInit_MiniKraidSpikes_Tab0[((*gExtraEnemyRam7800(game.cur_enemy_index() as i32, game))
            .kraid
            .kraid_healths_8ths[0i32 as usize]
            >> 1) as usize] as u16;
    let enemy_index = game.cur_enemy_index();
    word!(game.eproj_x_pos[v2] = game.enemy_state(enemy_index).x_pos());
    word!(game.eproj_y_pos[v2] = (r18).wrapping_add(game.enemy_state(enemy_index).y_pos()));
    word! { game, 0; eproj_y_subpos[v2], eproj_x_subpos[v2], eproj_y_vel[v2] }
}
crate::forward_calls! { unsafe fn (j: u16, game: &mut Game);
    EprojInit_MiniKraidSpikesLeft => EprojInit_MiniKraidSpikes(j, -0x200i32 as u16, game);
    EprojInit_MiniKraidSpikesRight => EprojInit_MiniKraidSpikes(j, 0x200, game);
}
pub unsafe fn EprojPreInstr_MiniKraidSpikes(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) & 1 != 0 {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    }
}
unsafe fn EprojInit_WalkingLavaSeahorseFireball(j: u16, game: &mut Game) {
    static word_869EF9: [i16; 3] = [-0x100i16, 0, 0x100];
    let v1 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_y_pos[v2] = ((*v1).y_pos as i32 - 12i32) as u16);
    word!(game.eproj_x_vel[v2] = -1024i32 as u16);
    word!(game.eproj_x_pos[v2] = ((*v1).x_pos as i32 - 16i32) as u16);
    if ((*v1).ai_var_D & 0x8000) == 0 {
        word!(game.eproj_x_vel[v2] = 1024);
        word!(game.eproj_x_pos[v2] = ((*v1).x_pos as i32 + 16) as u16);
    }
    word!(game.eproj_y_vel[v2] = word_869EF9[(word!(game.cinematic_var8) >> 1) as usize] as u16);
    word! { game, 0; eproj_y_subpos[v2], eproj_x_subpos[v2] }
}
pub unsafe fn EprojPreInstr_WalkingLavaSeahorseFireball(k: u16, game: &mut Game) {
    let mut v2: u16 = 0;
    if EprojBlockCollisition_Vertical(k, game) & 1 != 0
        || EprojBlockCollisition_Horiz(k, game) & 1 != 0
    {
        word!(game.eproj_id[(k >> 1)] = 0);
    } else {
        let v1: i32 = k as i32 >> 1;
        if (word!(game.eproj_x_vel[v1]) & 0x8000) == 0 {
            v2 = (word!(game.eproj_x_vel[v1]) as i32 - 64i32) as u16;
            word!(game.eproj_x_vel[v1] = v2);
            if v2 as i32 - 512i32 & 0x8000 != 0 {
                v2 = 512;
            }
        } else {
            v2 = (word!(game.eproj_x_vel[v1]) as i32 + 64) as u16;
            word!(game.eproj_x_vel[v1] = v2);
            if v2 as i32 + 512 & 0x8000 == 0 {
                v2 = -512i32 as u16;
            }
        }
        word!(game.eproj_x_vel[v1] = v2);
    };
}
unsafe fn EprojInit_PirateMotherBrainLaser(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    word!(
        game.eproj_instr_list_ptr[v2] = (if game.eproj_spawn_r22 != 0 {
            addr_word_869F7D as i32
        } else {
            addr_word_869F41 as i32
        }) as u16
    );
    word!(game.eproj_pre_instr[v2] = (0x86a05b & 0xffff) as u16);
    word!(game.eproj_x_pos[v2] = game.eproj_spawn_pt.x);
    word!(game.eproj_y_pos[v2] = game.eproj_spawn_pt.y);
    let enemy_index = game.cur_enemy_index();
    word!(
        game.eproj_properties[v2] =
            (*(RomPtr_A0(game.enemy_state(enemy_index).enemy_ptr(), game) as *mut u16).offset(3)
                | 0x1000)
    );
    word!(game.eproj_E[v2] = game.enemy_state(enemy_index).parameter_1());
    QueueSfx2_Max6(0x67, game);
}
pub unsafe fn EprojInstr_SetPreInstrAndRun(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    word!(game.eproj_pre_instr[(k >> 1)] = *(epjp as *mut u16));
    epjp
}
pub unsafe fn EprojPreInstr_PirateMotherBrainLaser_MoveLeft(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    EprojStateMut(&mut game.g_ram).add_word(0x1a4b, v1 as usize, (-2i16) as u16);
    if (word!(game.eproj_E[v1]) & 0x8000) == 0 {
        word!(game.eproj_x_pos[v1], |fresh22| (fresh22 as i32 - 2i32)
            as u16);
    }
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        EprojStateMut(&mut game.g_ram).set_id(v1 as usize, 0);
    }
}
pub unsafe fn EprojPreInstr_PirateMotherBrainLaser_MoveRight(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    EprojStateMut(&mut game.g_ram).add_word(0x1a4b, v1 as usize, 2);
    if (word!(game.eproj_E[v1]) & 0x8000) == 0 {
        word!(game.eproj_x_pos[v1], |fresh24| (fresh24 as i32 + 2) as u16);
    }
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        EprojStateMut(&mut game.g_ram).set_id(v1 as usize, 0);
    }
}
unsafe fn EprojInit_PirateClaw(j: u16, game: &mut Game) {
    let r: Rect16U = game.eproj_spawn_rect;
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_y_pos[v1] = (r.y as i32 + r.h as i32) as u16);
    word!(game.eproj_x_pos[v1] = (r.x as i32 + r.w as i32) as u16);
    word!(
        game.eproj_instr_list_ptr[v1] = (if word!(game.cinematic_var8) != 0 {
            addr_off_869FE1 as i32
        } else {
            addr_off_869FB9 as i32
        }) as u16
    );
    word!(game.eproj_pre_instr[v1] = (0x86a05b & 0xffff) as u16);
    word!(game.eproj_E[v1] = 2048);
    word!(game.eproj_F[v1] = 1);
}
pub unsafe fn EprojPreInstr_PirateClawThrownLeft(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    if word!(game.eproj_F[v1]) != 0 {
        word!(game.eproj_x_pos[v1], |fresh118| (fresh118 as i32
            - *((ram_ptr!(game, u16; 0x1aff)).offset(v1 as isize) as *mut u8).offset(1,) as i32)
            as u16);
        let v2: u16 = (word!(game.eproj_E[v1]) as i32 - 32i32) as u16;
        word!(game.eproj_E[v1] = v2);
        if v2 == 0 {
            word!(game.eproj_F[v1] = 0);
        }
    } else {
        word!(game.eproj_x_pos[v1], |fresh119| (fresh119 as i32
            + *((ram_ptr!(game, u16; 0x1aff)).offset(v1 as isize) as *mut u8).offset(1,) as i32)
            as u16);
        word!(game.eproj_E[v1], |fresh120| (fresh120 as i32 + 32) as u16);
    }
    word!(game.eproj_y_pos[v1], wrapping_add(1));
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        word!(game.eproj_id[v1] = 0);
    }
}
pub unsafe fn EprojPreInstr_PirateClawThrownRight(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    if word!(game.eproj_F[v1]) != 0 {
        word!(game.eproj_x_pos[v1], |fresh114| (fresh114 as i32
            + *((ram_ptr!(game, u16; 0x1aff)).offset(v1 as isize) as *mut u8).offset(1,) as i32)
            as u16);
        let v2: u16 = (word!(game.eproj_E[v1]) as i32 - 32i32) as u16;
        word!(game.eproj_E[v1] = v2);
        if v2 == 0 {
            word!(game.eproj_F[v1] = 0);
        }
    } else {
        word!(game.eproj_x_pos[v1], |fresh115| (fresh115 as i32
            - *((ram_ptr!(game, u16; 0x1aff)).offset(v1 as isize) as *mut u8).offset(1,) as i32)
            as u16);
        word!(game.eproj_E[v1], |fresh116| (fresh116 as i32 + 32) as u16);
    }
    word!(game.eproj_y_pos[v1], wrapping_add(1));
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        word!(game.eproj_id[v1] = 0);
    }
}
static word_86A2D6: [i16; 6] = [64, 72, 80, -64i16, -72i16, -80i16];
unsafe fn EprojInit_A379(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word! { game, 0;
        eproj_x_subpos[v1], eproj_y_subpos[v1], eproj_x_vel[v1], eproj_y_vel[v1],
    }
    let v2: u16 = word!(game.cinematic_var8);
    let v3: i32 = word!(game.cinematic_var8) as i32 >> 1;
    word! { game;
        eproj_x_pos[v1] = (word_86A2D6[v3 as usize] as i32 + word!(game.samus_x_pos) as i32) as u16;
        eproj_y_pos[v1] = (word!(game.samus_y_pos) as i32 + 80) as u16;
        eproj_instr_list_ptr[v1] = crate::rtl::rom_table_word!(game, 0x86a2e2, v3 as isize);
        eproj_E[v1] = v2;
    }
}
fn EprojInit_CeresElevatorPad(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    let eproj_y_pos: u16 = game.samus_y_pos().wrapping_add(28);
    EprojStateMut(&mut game.g_ram).write(0x1a93, v1 as usize, eproj_y_pos);
    EprojStateMut(&mut game.g_ram).write(0x1aff, v1 as usize, 60);
    sub_86A301(j, game);
}
pub fn sub_86A301(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    EprojStateMut(&mut game.g_ram).write(0x1a27, v1 as usize, 0);
    EprojStateMut(&mut game.g_ram).write(0x1a6f, v1 as usize, 0);
    EprojStateMut(&mut game.g_ram).write(0x1ab7, v1 as usize, 0);
    EprojStateMut(&mut game.g_ram).write(0x1adb, v1 as usize, 0);
    EprojStateMut(&mut game.g_ram).write(0x19bb, v1 as usize, 0);
    let eproj_x_pos: u16 = game.samus_x_pos();
    EprojStateMut(&mut game.g_ram).write(0x1a4b, v1 as usize, eproj_x_pos);
}
fn EprojInit_CeresElevatorPlatform(j: u16, game: &mut Game) {
    let eproj_word_index = (j >> 1) as usize;
    EprojStateMut(&mut game.g_ram).write(0x1a93, eproj_word_index, 97);
    sub_86A301(j, game);
}
pub unsafe fn EprojPreInstr_CeresElevatorPad(k: u16, game: &mut Game) {
    let mut v3: bool = false;
    let v1: i32 = k as i32 >> 1;
    if word!(game.eproj_E[v1]) == 0
        || {
            let mut v2: bool = word!(game.eproj_E[v1]) == 1;
            v3 = ((word!(game.eproj_E[v1])) as i16).wrapping_sub(1) < 0;
            word!(game.eproj_E[v1], wrapping_sub(1));
            v2 as i32 != 0
        }
        || v3
    {
        word!(game.eproj_y_pos[v1] = game.samus_y_pos().wrapping_add(28));
        let fresh113 = EprojStateMut(&mut game.g_ram).add_word(0xafa, 0, 1);
        if fresh113 as i32 - 73i32 & 0x8000 == 0 {
            word!(game.samus_y_pos = 72);
            word!(game.eproj_instr_timers[v1] = 1);
            word!(game.eproj_instr_list_ptr[v1] = addr_off_86A28B as u16);
            CallSomeSamusCode(0xe, game);
        }
    }
}
pub unsafe fn EprojPreInstr_CeresElevatorPlatform(k: u16, game: &mut Game) {
    if game.samus_y_pos() == 72 {
        let v1: i32 = k as i32 >> 1;
        word!(game.eproj_instr_timers[v1] = 1);
        word!(game.eproj_instr_list_ptr[v1] = addr_off_86A28B as u16);
    }
}
pub unsafe fn EprojPreInstr_PrePhantomRoom(j: u16, game: &mut Game) {
    word!(game.bg2_y_scroll = 0);
}
pub unsafe fn EprojInstr_A3BE(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v1: i32 = k as i32 >> 1;
    word!(game.eproj_x_pos[v1] = word!(game.eproj_E[v1]));
    word!(game.eproj_y_pos[v1] = word!(game.eproj_F[v1]));
    epjp
}
pub unsafe fn EprojInstr_GotoWithProbability25(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    if (NextRandom(game) & 0xc000) == 0xc000 {
        *(epjp as *mut u16) as usize as *const u8
    } else {
        epjp.offset(2)
    }
}
unsafe fn EprojInit_BombTorizoLowHealthDrool(j: u16, game: &mut Game) {
    let mut v5: u16 = 0;
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_gfx_idx[v1] = 0);
    word!(
        game.eproj_instr_list_ptr[v1] = crate::rtl::rom_table_word!(
            game,
            0x86a64d,
            (((NextRandom(game) >> 1) & (0xeu16)) >> 1) as isize,
        )
    );
    NextRandom(game);
    let v2 = gEnemyData(game.cur_enemy_index() as i32, game);
    word!(game.eproj_y_pos[v1] = ((*v2).y_pos).wrapping_sub(5));
    let mut parameter_1: i16 = (*v2).parameter_1 as i16;
    let mut v4: u16 = if (parameter_1 as u16 & 0x4000) != 0 {
        (word!(game.random_number) & 0x1fe)
    } else {
        if parameter_1 < 0 {
            v5 = 32;
        } else {
            v5 = 224;
        }
        (2 * (v5 as i32 + (game.random_number() as i32 & 0xf) - 8i32)) as u16
    };
    let v6: i32 = v4 as i32 >> 1;
    word!(game.eproj_x_vel[v1] = kSinCosTable8bit_Sext[(v6 + 64) as usize] as u16);
    word!(game.eproj_y_vel[v1] = kSinCosTable8bit_Sext[v6 as usize] as u16);
    let v7 = gEnemyData(game.cur_enemy_index() as i32, game);
    let mut v8: u16 = if ((*v7).parameter_1 & 0x8000) != 0 {
        ((*v7).x_pos).wrapping_add(8)
    } else {
        ((*v7).x_pos as i32 - 8i32) as u16
    };
    word!(game.eproj_x_pos[v1] = v8);
}
unsafe fn EprojInit_BombTorizoLowHealthInitialDrool(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_gfx_idx[v1] = 0);
    NextRandom(game);
    let v2 = gEnemyData(game.cur_enemy_index() as i32, game);
    // Avoid a shared Game reference while the returned enemy pointer remains live.
    word! { game;
        eproj_y_pos[v1] = (((*v2).y_pos as i32
            + (u16::from_le(
                (&raw const game.g_ram)
                    .cast::<u8>()
                    .add(0x5e5)
                    .cast::<u16>()
                    .read_unaligned(),
            ) as i32
                & 3)) as u16)
            .wrapping_sub(5);
        eproj_y_vel[v1] = (u16::from_le(
            (&raw const game.g_ram)
                .cast::<u8>()
                .add(0x5e5)
                .cast::<u16>()
                .read_unaligned(),
        ) & 0x1f)
            .wrapping_add(48);
    }
    NextRandom(game);
    let mut v3: i16 = ((u16::from_le(
        (&raw const game.g_ram)
            .cast::<u8>()
            .add(0x5e5)
            .cast::<u16>()
            .read_unaligned(),
    )) as i16)
        & (3i16);
    let mut parameter_1: i16 = (*v2).parameter_1 as i16;
    if (parameter_1 as u16 & 0x4000) != 0 {
        word!(game.eproj_x_pos[v1] = ((*v2).x_pos as i32 + v3 as i32) as u16);
        word!(game.eproj_x_vel[v1] = 0);
    } else {
        if parameter_1 < 0 {
            word!(game.eproj_x_pos[v1] = (((*v2).x_pos as i32 + v3 as i32) as u16).wrapping_add(8));
        } else {
            word!(game.eproj_x_pos[v1] = (((*v2).x_pos as i32 + v3 as i32) as u16).wrapping_sub(8));
        }
        word!(game.eproj_x_vel[v1] = 0);
    };
}
unsafe fn EprojInit_A977(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_gfx_idx[v1] = 0);
    CalculatePlmBlockCoords(word!(game.plm_id), game);
    word!(game.eproj_x_pos[v1] = (16 * word!(game.plm_x_block) as i32) as u16);
    word!(game.eproj_y_pos[v1] = (16 * word!(game.plm_y_block) as i32 - 4i32) as u16);
    word! { game, 0; eproj_x_subpos[v1], eproj_y_subpos[v1] }
}
static kEprojInit_BombTorizoExplosiveSwipe_Tab0: [i16; 11] = [
    -30i16, -40i16, -47i16, -31i16, -21i16, -1i16, -28i16, -43i16, -48i16, -31i16, -21i16,
];
static kEprojInit_BombTorizoExplosiveSwipe_Tab1: [i16; 11] = [
    -52i16, -28i16, -11i16, 9, 21, 20, -52i16, -27i16, -10i16, 9, 20,
];
static kEprojInit_BombTorizoStatueBreaking_Xpos: [i16; 16] = [
    8, 0x18, -8i16, 8, 0x18, -8i16, 8, 0x18, 8, -8i16, 0x18, 8, -8i16, 0x18, 8, -8i16,
];
static kEprojInit_BombTorizoStatueBreaking_Ypos: [i16; 8] =
    [-8i16, -8i16, 8, 8, 8, 0x18, 0x18, 0x18];
static kEprojInit_BombTorizoStatueBreaking_Yvel: [i16; 8] =
    [256, 256, 256, 256, 256, 256, 256, 256];
static kEprojInit_BombTorizoStatueBreaking_F: [i16; 8] = [16, 16, 16, 16, 16, 16, 16, 16];
static kEprojInit_BombTorizoLowHealthExplode_X: [i16; 6] = [0, 12, -12i16, 0, 16, -16i16];
static kEprojInit_BombTorizoLowHealthExplode_Y: [i16; 6] =
    [-8i16, -8i16, -8i16, -20i16, -20i16, -20i16];
unsafe fn EprojInit_BombTorizoExplosiveSwipe(j: u16, game: &mut Game) {
    let v1 = gEnemyData(game.cur_enemy_index() as i32, game);
    if ((*v1).parameter_1 & 0x8000) != 0 {
        let v4: i32 = word!(game.cinematic_var8) as i32 >> 1;
        let v5: i32 = j as i32 >> 1;
        word! { game;
            eproj_x_pos[v5] = ((*v1).x_pos)
                .wrapping_sub((kEprojInit_BombTorizoExplosiveSwipe_Tab0[v4 as usize]) as u16);
            eproj_y_pos[v5] = ((*v1).y_pos)
                .wrapping_add((kEprojInit_BombTorizoExplosiveSwipe_Tab1[v4 as usize]) as u16);
        }
    } else {
        let v2: i32 = word!(game.cinematic_var8) as i32 >> 1;
        let v3: i32 = j as i32 >> 1;
        word! { game;
            eproj_x_pos[v3] = ((*v1).x_pos as i32
                + kEprojInit_BombTorizoExplosiveSwipe_Tab0[v2 as usize] as i32)
                as u16;
            eproj_y_pos[v3] = ((*v1).y_pos as i32
                + kEprojInit_BombTorizoExplosiveSwipe_Tab1[v2 as usize] as i32)
                as u16;
        }
    };
}
unsafe fn EprojInit_BombTorizoStatueBreaking(j: u16, game: &mut Game) {
    CalculatePlmBlockCoords(word!(game.plm_id), game);
    let mut v1: i8 = EprojState(&game.g_ram).init_param_1() as i8;
    let v2: i32 = EprojState(&game.g_ram).init_param_1() as i32 >> 1;
    let v3: i32 = j as i32 >> 1;
    word!(game.eproj_instr_list_ptr[v3] = crate::rtl::rom_table_word!(game, 0x86a7ab, v2 as isize));
    word!(
        game.eproj_x_pos[v3] = (kEprojInit_BombTorizoStatueBreaking_Xpos[v2 as usize] as i32
            + 16 * word!(game.plm_x_block) as i32) as u16
    );
    let v4: i32 = (v1 as i32 & 0xf) >> 1;
    word!(
        game.eproj_y_pos[v3] = (kEprojInit_BombTorizoStatueBreaking_Ypos[v4 as usize] as i32
            + 16 * word!(game.plm_y_block) as i32) as u16
    );
    word!(game.eproj_y_vel[v3] = kEprojInit_BombTorizoStatueBreaking_Yvel[v4 as usize] as u16);
    word!(game.eproj_F[v3] = kEprojInit_BombTorizoStatueBreaking_F[v4 as usize] as u16);
    EprojStateMut(&mut game.g_ram).set_property_bits(v3 as usize, 0x1000);
}
unsafe fn EprojInit_BombTorizoLowHealthExplode(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v2] = game.enemy_state(enemy_index).x_pos());
    word!(game.eproj_y_pos[v2] = game.enemy_state(enemy_index).y_pos());
    if (game.enemy_state(enemy_index).parameter_1() & 0x8000) == 0 {
        word!(game.cinematic_var8, |fresh13| (fresh13 as i32 + 2) as u16);
    }
    EprojStateMut(&mut game.g_ram).add_word(0x1993, 0, 2);
    let v3: i32 = EprojState(&game.g_ram).init_param_1() as i32 >> 1;
    let v4: u16 = (kEprojInit_BombTorizoLowHealthExplode_X[v3 as usize] as i32
        + word!(game.eproj_x_pos[v2]) as i32) as u16;
    word!(game.eproj_x_pos[v2] = v4);
    word!(game.eproj_E[v2] = v4);
    let v5: u16 = (kEprojInit_BombTorizoLowHealthExplode_Y[v3 as usize] as i32
        + word!(game.eproj_y_pos[v2]) as i32) as u16;
    word!(game.eproj_y_pos[v2] = v5);
    word!(game.eproj_F[v2] = v5);
}
unsafe fn EprojInit_BombTorizoDeathExplosion(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let x_pos: u16 = game.enemy_state(enemy_index).x_pos();
    let v3: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v3] = x_pos);
    word!(game.eproj_E[v3] = x_pos);
    let y_pos: u16 = game.enemy_state(enemy_index).y_pos();
    word!(game.eproj_y_pos[v3] = y_pos);
    word!(game.eproj_F[v3] = y_pos);
}
pub unsafe fn sub_86A887(v0: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(v0, game) & 1 != 0 {
        let v6: i32 = v0 as i32 >> 1;
        word!(game.eproj_instr_list_ptr[v6] = 0xa48a);
        word!(game.eproj_instr_timers[v6] = 1);
    } else {
        let v1: i32 = v0 as i32 >> 1;
        let mut v2: i16 = word!(game.eproj_x_vel[v1]) as i16;
        if v2 >= 0 {
            let mut v4: i16 = (v2).wrapping_sub(4);
            if v4 < 0 {
                v4 = 3;
            }
            word!(game.eproj_x_vel[v1] = v4 as u16);
        } else {
            let mut v3: i16 = (v2 as i32 + 4) as i16;
            if v3 >= 0 {
                v3 = 3;
            }
            word!(game.eproj_x_vel[v1] = v3 as u16);
        }
        let carry: u8 = EprojBlockCollisition_Vertical(v0, game);
        if (word!(game.eproj_y_vel[v1]) & 0x8000) != 0 || carry == 0 {
            let v5: u16 = EprojStateMut(&mut game.g_ram).add_word(0x1adb, v1 as usize, 16);
            if (v5 & 0xf000) == 4096 {
                word!(game.eproj_id[v1] = 0);
            }
        } else {
            let v7: i32 = v0 as i32 >> 1;
            EprojStateMut(&mut game.g_ram).add_word(0x1a93, v7 as usize, (-3i16) as u16);
            EprojStateMut(&mut game.g_ram).start_instruction(v7 as usize, 0xa48e);
        }
    };
}
pub unsafe fn EprojPreInstr_A977(k: u16, game: &mut Game) {
    let carry: u8 = EprojBlockCollisition_Vertical(k, game);
    let v1: i32 = k as i32 >> 1;
    if (word!(game.eproj_y_vel[v1]) & 0x8000) != 0 || carry == 0 {
        let v2: u16 = (word!(game.eproj_F[0])).wrapping_add(word!(game.eproj_y_vel[v1]));
        word!(game.eproj_y_vel[v1] = v2);
        if (v2 & 0xf000) == 4096 {
            word!(game.eproj_y_vel[v1] = 4096);
        }
    } else {
        word!(game.eproj_pre_instr[v1] = 0xa918);
    };
}
unsafe fn EprojInit_AB07(j: u16, game: &mut Game) {
    let mut v1: u16 = word!(game.vram_write_queue_tail);
    let v2 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
    (*v2).size = 64;
    (*v2).src.addr = addr_kEprojInit_AB07_Tile0 as i32 as VoidP;
    *(&raw mut (*v2).src.bank as *mut u16) = 134;
    (*v2).vram_dst = 28160;
    v1 = (v1 as i32 + 7) as u16;
    let v3 = gVramWriteEntry(v1 as i32, game);
    (*v3).size = 64;
    (*v3).src.addr = addr_kEprojInit_AB07_Tile1 as i32 as VoidP;
    *(&raw mut (*v3).src.bank as *mut u16) = 134;
    (*v3).vram_dst = 28416;
    word!(game.vram_write_queue_tail = (v1 as i32 + 7) as u16);
    let v4: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v4] = word!(game.samus_x_pos));
    word!(game.eproj_y_pos[v4] = (word!(game.samus_y_pos) as i32 - 36i32) as u16);
}
pub unsafe fn EprojPreInstr_AB07(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let mut amt: i32 = ((8 * (word!(game.samus_x_pos) as i32 - word!(game.eproj_x_pos[v1]) as i32))
        as i16 as i32)
        << 8;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a4b)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a27)).offset(v1 as isize),
        amt as u32,
    );
    amt = ((8 * (word!(game.samus_y_pos) as i32 - 36i32 - word!(game.eproj_y_pos[v1]) as i32))
        as i16 as i32)
        << 8;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a93)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a6f)).offset(v1 as isize),
        amt as u32,
    );
}
pub unsafe fn EprojInstr_SpawnEnemyDrops(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v2: i32 = k as i32 >> 1;
    game.eproj_spawn_pt = Point16U {
        x: word!(game.eproj_x_pos[v2]),
        y: word!(game.eproj_y_pos[v2]),
    };
    SpawnEnemyDrops(
        (if word!(game.area_index) != 0 {
            *(epjp.offset(2) as *mut u16) as i32
        } else {
            *(epjp as *mut u16) as i32
        }) as u16,
        k,
        0,
        game,
    );
    epjp.offset(4)
}
unsafe fn Eproj_InitXYVelRandom(j: u16, k: u16, pt: Point16U, game: &mut Game) {
    let v2 = RomPtr_86(k, game) as *const u16;
    let v3: i32 = j as i32 >> 1;
    word! { game;
        eproj_instr_list_ptr[v3] = *v2;
        eproj_x_pos[v3] = (*v2.offset(1)).wrapping_add(pt.x);
        eproj_x_vel[v3] = (*v2.offset(2) as i32 + NextRandom(game) as u8 as i32 - 128i32) as u16;
        eproj_y_pos[v3] = (*v2.offset(3)).wrapping_add(pt.y);
        eproj_y_vel[v3] = (*v2.offset(4) as i32 + NextRandom(game) as u8 as i32 - 128i32) as u16;
    }
}
unsafe fn EprojInit_BombTorizosChozoOrbs(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v2: u16 = (if (game.enemy_state(enemy_index).parameter_1() & 0x8000) != 0 {
        addr_kEprojInit_BombTorizosChozoOrbs_init0 as i32
    } else {
        addr_kEprojInit_BombTorizosChozoOrbs_init1 as i32
    }) as u16;
    Eproj_InitXYVelRandom(
        j,
        v2,
        Point16U {
            x: game.enemy_state(enemy_index).x_pos(),
            y: game.enemy_state(enemy_index).y_pos(),
        },
        game,
    );
}
unsafe fn EprojInit_GoldenTorizosChozoOrbs(j: u16, game: &mut Game) {
    let E = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    let v2: u16 = (if ((*E).parameter_1 & 0x8000) != 0 {
        addr_kEprojInit_GoldenTorizosChozoOrbs_init0 as i32
    } else {
        addr_kEprojInit_GoldenTorizosChozoOrbs_init1 as i32
    }) as u16;
    Eproj_InitXYVelRandom(
        j,
        v2,
        Point16U {
            x: (*E).x_pos,
            y: (*E).y_pos,
        },
        game,
    );
}
pub unsafe fn EprojPreInstr_BombTorizosChozoOrbs(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) != 0 {
        let v3: i32 = k as i32 >> 1;
        EprojStateMut(&mut game.g_ram).start_instruction(v3 as usize, addr_off_86AB25 as u16);
    } else {
        let carry: u8 = EprojBlockCollisition_Vertical(k, game);
        let v1: i32 = k as i32 >> 1;
        if (word!(game.eproj_y_vel[v1]) & 0x8000) != 0 || carry == 0 {
            let v2: u16 = EprojStateMut(&mut game.g_ram).add_word(0x1adb, v1 as usize, 18);
            if (v2 & 0xf000) == 4096 {
                word!(game.eproj_id[v1] = 0);
            }
        } else {
            let v4: i32 = k as i32 >> 1;
            word!(
                game.eproj_y_pos[v4] =
                    (game.enemy_projectile_y_pos(k) & 0xfff0 | 8).wrapping_sub(2)
            );
            EprojStateMut(&mut game.g_ram).start_instruction(v4 as usize, addr_off_86AB41 as u16);
        }
    };
}
pub unsafe fn EprojPreInstr_GoldenTorizosChozoOrbs(k: u16, game: &mut Game) {
    let mut v1: i32 = 0;
    let mut v3: u16 = 0;
    if EprojBlockCollisition_Horiz(k, game) & 1 != 0 {
        word!(game.eproj_x_vel[(k >> 1)] = -(word!(game.eproj_x_vel[(k >> 1)]) as i32) as u16);
    }
    if EprojBlockCollisition_Vertical(k, game) & 1 != 0
        && {
            v1 = k as i32 >> 1;
            (word!(game.eproj_y_vel[v1]) & 0x8000) == 0
        }
        && {
            let mut v2: i16 = word!(game.eproj_x_vel[v1]) as i16;
            if v2 >= 0 {
                v3 = (v2 as i32 - 64i32) as u16;
            } else {
                v3 = (v2 as i32 + 64) as u16;
            };
            word!(game.eproj_x_vel[v1] = v3);
            let mut v4: u16 = -(word!(game.eproj_y_vel[v1]) as i32 >> 1i32) as u16;
            word!(game.eproj_y_vel[v1] = v4);
            (v4 & 0xff80) == 0xff80
        }
    {
        let v5: i32 = k as i32 >> 1;
        word! { game;
            eproj_y_pos[v5] = ((word!(game.eproj_y_pos[v5]) as i32 & 0xfff0 | 8) - 2i32) as u16;
            eproj_instr_list_ptr[v5] = addr_off_86AB41 as u16;
            eproj_instr_timers[v5] = 1;
        }
    } else {
        word!(
            game.eproj_y_vel[(k >> 1)],
            |fresh110| (fresh110 as i32 + 24) as u16
        );
    };
}
pub unsafe fn EprojInstr_GotoDependingOnXDirection(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    let v2: i32 = k as i32 >> 1;
    let amt: i32 = (word!(game.eproj_x_vel[v2]) as i16 as i32) << 8;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a4b)).offset(v2 as isize),
        (ram_ptr!(game, u16; 0x1a27)).offset(v2 as isize),
        amt as u32,
    );
    if (word!(game.eproj_x_vel[v2]) & 0x8000) == 0 {
        *(epjp.offset(2) as *mut u16) as usize as *const u8
    } else {
        *(epjp as *mut u16) as usize as *const u8
    }
}
unsafe fn EprojInit_TorizoSonicBoom(j: u16, game: &mut Game) {
    let v1: i32 = if (NextRandom(game) & 1) != 0 {
        -12i32
    } else {
        20
    };
    let v2 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    let v3: i32 = j as i32 >> 1;
    word!(game.eproj_y_pos[v3] = ((*v2).y_pos as i32 + v1) as u16);
    word!(game.eproj_y_vel[v3] = 0);
    if ((*v2).parameter_1 & 0x8000) != 0 {
        word!(game.eproj_x_pos[v3] = ((*v2).x_pos as i32 + 32) as u16);
        word!(game.eproj_x_vel[v3] = 624);
        word!(game.eproj_instr_list_ptr[v3] = addr_off_86ADD2 as u16);
    } else {
        word!(game.eproj_x_pos[v3] = ((*v2).x_pos as i32 - 32i32) as u16);
        word!(game.eproj_x_vel[v3] = -624i32 as u16);
        word!(game.eproj_instr_list_ptr[v3] = addr_off_86ADBF as u16);
    };
}
pub unsafe fn EprojPreInstr_TorizoSonicBoom(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) & 1 != 0 {
        let v4: i32 = k as i32 >> 1;
        word!(game.eproj_instr_list_ptr[v4] = addr_off_86ADE5 as u16);
        word!(game.eproj_instr_timers[v4] = 1);
        word!(game.eproj_E[v4] = word!(game.eproj_x_pos[v4]));
        word!(game.eproj_F[v4] = word!(game.eproj_y_pos[v4]));
    } else {
        let v1: i32 = k as i32 >> 1;
        word!(game.eproj_x_vel[v1], |fresh109| (fresh109 as i32
            + if word!(game.eproj_x_vel[v1]) & 0x8000 != 0 {
                -16i32
            } else {
                16
            }) as u16);
        if (word!(game.eproj_x_vel[v1]) & 0xf000) == 4096 {
            word!(game.eproj_id[v1] = 0);
        }
    };
}
unsafe fn EprojInit_WreckedShipChozoSpikeFootsteps(j: u16, game: &mut Game) {
    let v1 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v2] = (word!(game.cinematic_var8) as i32 + (*v1).x_pos as i32) as u16);
    word!(game.eproj_y_pos[v2] = ((*v1).y_pos as i32 + 28) as u16);
}
pub unsafe fn EprojInstr_ResetXYpos1(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v2: i32 = k as i32 >> 1;
    word!(game.eproj_x_pos[v2] = word!(game.eproj_E[v2]));
    word!(game.eproj_y_pos[v2] = word!(game.eproj_F[v2]));
    epjp
}
unsafe fn EprojInit_TourianStatueDustClouds(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_E[v1] = 128);
    word!(game.eproj_F[v1] = 188);
}
unsafe fn EprojInit_TourianLandingDustCloudsRightFoot(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_y_pos[v2] = (game.enemy_state(enemy_index).y_pos()).wrapping_add(48));
    word!(game.eproj_x_pos[v2] = (game.enemy_state(enemy_index).x_pos()).wrapping_add(24));
}
pub fn EprojInstr_MoveY_Minus4(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    EprojStateMut(&mut game.g_ram).add_word(0x1a93, usize::from(k >> 1), (-4i16) as u16);
    epjp
}
unsafe fn EprojInit_TorizoLandingDustCloudLeftFoot(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_y_pos[v2] = (game.enemy_state(enemy_index).y_pos()).wrapping_add(48));
    word!(game.eproj_x_pos[v2] = (game.enemy_state(enemy_index).x_pos()).wrapping_sub(24));
}
unsafe fn EprojInit_GoldenTorizoEgg(j: u16, game: &mut Game) {
    let v1 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_F[v2] = ((0xe2 & 0x1f) + 64) as u16);
    word!(game.eproj_E[v2] = (*v1).parameter_1);
    let v4: u16 = (if (*v1).parameter_1 & 0x8000 != 0 {
        addr_kEprojInit_GoldenTorizoEgg0 as i32
    } else {
        addr_kEprojInit_GoldenTorizoEgg1 as i32
    }) as u16;
    Eproj_InitXYVelRandom(
        j,
        v4,
        Point16U {
            x: (*v1).x_pos,
            y: (*v1).y_pos,
        },
        game,
    );
}
pub unsafe fn EprojPreInstr_GoldenTorizoEgg(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    if (word!(game.eproj_F[v1], wrapping_sub(1), reread) & 0x8000) != 0 {
        word!(
            game.eproj_instr_list_ptr[v1],
            |fresh105| (fresh105 as i32 + 2) as u16
        );
        word!(game.eproj_instr_timers[v1] = 1);
        word!(
            game.eproj_x_vel[v1] = (if (word!(game.eproj_E[v1]) & 0x8000) != 0 {
                256
            } else {
                -256i32
            }) as u16
        );
    } else {
        if EprojBlockCollisition_Horiz(k, game) & 1 != 0 {
            word!(game.eproj_x_vel[v1] = -(word!(game.eproj_x_vel[v1]) as i32) as u16);
            word!(game.eproj_E[v1], |fresh106| (fresh106 ^ 0x8000));
        }
        if EprojBlockCollisition_Vertical(k, game) & 1 != 0
            && (word!(game.eproj_y_vel[v1]) & 0x8000) == 0
        {
            word!(game.eproj_x_vel[v1], |fresh107| (fresh107 as i32
                + if word!(game.eproj_x_vel[v1]) & 0x8000 != 0 {
                    32
                } else {
                    -32i32
                }) as u16);
            word!(game.eproj_y_vel[v1] = -(word!(game.eproj_y_vel[v1]) as i32) as u16);
        }
        word!(game.eproj_y_vel[v1], |fresh108| (fresh108 as i32 + 48)
            as u16);
        if (word!(game.eproj_y_vel[v1]) & 0xf000) == 4096 {
            word!(game.eproj_id[v1] = 0);
        }
    };
}
pub unsafe fn sub_86B0B9(k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    if EprojBlockCollisition_Horiz(k, game) & 1 != 0 {
        let v3: i32 = k as i32 >> 1;
        word!(game.eproj_pre_instr[v3] = 0xb0dd);
        word!(game.eproj_y_vel[v3] = 0);
    } else {
        let v1: i32 = k as i32 >> 1;
        if (word!(game.eproj_E[v1]) & 0x8000) != 0 {
            v2 = 48;
        } else {
            v2 = -48i16;
        }
        word!(
            game.eproj_x_vel[v1],
            |fresh103| (fresh103 as i32 + v2 as i32) as u16
        );
    };
}
pub unsafe fn sub_86B0DD(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Vertical(k, game) & 1 != 0 {
        let v1: i32 = k as i32 >> 1;
        word!(
            game.eproj_instr_list_ptr[v1] = (if (word!(game.eproj_E[v1]) & 0x8000) != 0 {
                addr_off_86B1A8 as i32
            } else {
                addr_off_86B190 as i32
            }) as u16
        );
        word!(game.eproj_instr_timers[v1] = 1);
    } else {
        word!(
            game.eproj_y_vel[(k >> 1)],
            |fresh102| (fresh102 as i32 + 48) as u16
        );
    };
}
pub unsafe fn sub_86B13E(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    ((if (word!(game.eproj_E[(k >> 1)]) & 0x8000) != 0 {
        addr_off_86B166 as i32
    } else {
        addr_off_86B14B as i32
    }) as usize as *const u8)
}
static word_86B205: [i16; 2] = [-0x1ei16, 0x1e];
unsafe fn EprojInit_GoldenTorizoSuperMissile(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_E[v2] = word!(game.cur_enemy_index));
    let v3 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    let v4: i32 = if ((*v3).parameter_1 & 0x8000) != 0 {
        1
    } else {
        0
    };
    word!(game.eproj_x_pos[v2] = ((*v3).x_pos as i32 + word_86B205[v4 as usize] as i32) as u16);
    word!(game.eproj_y_pos[v2] = ((*v3).y_pos as i32 - 52i32) as u16);
    word!(game.eproj_instr_list_ptr[v2] = crate::rtl::rom_table_word!(game, 0x86b209, v4 as isize));
}
pub unsafe fn EprojPreInstr_GoldenTorizoSuperMissile(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let v2 = gEnemyData(word!(game.eproj_E[v1]) as i32, game);
    let v3: i32 = if ((*v2).parameter_1 & 0x8000) != 0 {
        32
    } else {
        -32i32
    };
    word!(game.eproj_x_pos[v1] = ((*v2).x_pos as i32 + v3) as u16);
    word!(game.eproj_y_pos[v1] = ((*v2).y_pos as i32 - 52i32) as u16);
}
pub unsafe fn EprojPreInstr_B237(k: u16, game: &mut Game) {
    let mut v1: i32 = 0;
    if EprojBlockCollisition_Horiz(k, game) != 0 || {
        let mut carry: u8 = EprojBlockCollisition_Vertical(k, game);
        v1 = k as i32 >> 1;
        (word!(game.eproj_y_vel[v1]) & 0x8000) == 0 && carry != 0
    } {
        let v3: i32 = k as i32 >> 1;
        word!(game.eproj_instr_list_ptr[v3] = addr_off_86B2EF as u16);
        word!(game.eproj_instr_timers[v3] = 1);
    } else {
        let v2: u16 = (word!(game.eproj_y_vel[v1]) as i32 + 16) as u16;
        word!(game.eproj_y_vel[v1] = v2);
        if (v2 & 0xf000) == 4096 {
            word!(game.eproj_id[v1] = 0);
        }
    };
}
unsafe fn sub_86B279(k: u16, a: u16, game: &mut Game) {
    let v3: i32 = k as i32 >> 1;
    word!(
        game.eproj_x_vel[v3] = (4 * kSinCosTable8bit_Sext[(a as i32 + 64) as usize] as i32) as u16
    );
    word!(game.eproj_y_vel[v3] = (4 * kSinCosTable8bit_Sext[a as usize] as i32) as u16);
}
crate::instruction_callbacks! { pub unsafe fn (
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8;
    EprojInstr_SetVelTowardsSamus1 [epjp] {
        sub_86B279(k, (CalculateAngleOfSamusFromEproj(k, game) & 0x7f), game);
    };
    EprojInstr_SetVelTowardsSamus2 [epjp] {
        sub_86B279(k, (CalculateAngleOfSamusFromEproj(k, game) | 0x80), game);
    };
}
unsafe fn EprojInit_GoldenTorizoEyeBeam(j: u16, game: &mut Game) {
    let E = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    let v2: u16 = (if ((*E).parameter_1 & 0x8000) != 0 {
        addr_stru_86B376 as i32
    } else {
        addr_stru_86B380 as i32
    }) as u16;
    Eproj_InitXYVelRandom(
        j,
        v2,
        Point16U {
            x: (*E).x_pos,
            y: (*E).y_pos,
        },
        game,
    );
    let mut v3: u16 = ((NextRandom(game) as i32 & 0x1e) - 16i32 + 192) as u16;
    if ((*E).parameter_1 & 0x8000) == 0 {
        v3 = (v3 as i32 + 128) as u16;
    }
    let v4: i32 = v3 as i32 >> 1;
    let v5: i32 = j as i32 >> 1;
    word!(game.eproj_x_vel[v5] = (8 * kSinCosTable8bit_Sext[(v4 + 64) as usize] as i32) as u16);
    word!(game.eproj_y_vel[v5] = (8 * kSinCosTable8bit_Sext[v4 as usize] as i32) as u16);
}
pub unsafe fn EprojPreInstr_GoldenTorizoEyeBeam(k: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(k, game) & 1 != 0 {
        word!(game.eproj_instr_list_ptr[(k >> 1)] = addr_off_86B3CD as u16);
        word!(game.eproj_instr_timers[(k >> 1)] = 1);
        return;
    }
    if EprojBlockCollisition_Vertical(k, game) & 1 != 0 {
        let v1: i32 = k as i32 >> 1;
        word! { game;
            eproj_y_pos[v1] = ((word!(game.eproj_y_pos[v1]) as i32 & 0xfff0 | 8) - 2i32) as u16;
            eproj_instr_list_ptr[v1] = addr_off_86B3E5 as u16;
            eproj_instr_timers[v1] = 1;
        }
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, epjp: *const u8, game: &mut Game) -> *const u8;
    EprojInstr_GotoIfFunc1 [] {
        if ((*gExtraEnemyRam7800(word!(game.eproj_F[(k >> 1)]) as i32, game))
            .kraid
            .kraid_healths_8ths[0i32 as usize]
            & 0x8000)
            == 0
        {
            *(epjp as *mut u16) as usize as *const u8
        } else {
            epjp.offset(2)
        }
    };
    EprojInstr_ResetXYpos2 [epjp] {
        let v2: i32 = k as i32 >> 1;
        word!(game.eproj_x_pos[v2] = word!(game.eproj_E[v2]));
        word!(game.eproj_y_pos[v2] = word!(game.eproj_F[v2]));
    };
}
unsafe fn EprojInit_TourianEscapeShaftFakeWallExplode(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word! { game, 272; eproj_x_pos[v1], eproj_E[v1] }
    word! { game, 2184; eproj_y_pos[v1], eproj_F[v1] }
}
unsafe fn EprojInit_LavaSeahorseFireball(j: u16, game: &mut Game) {
    let v1 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_y_pos[v2] = ((*v1).y_pos).wrapping_sub(28));
    word!(game.eproj_y_vel[v2] = -961i32 as u16);
    if ((*v1).ai_var_A & 0x8000) == 0 {
        word!(game.eproj_x_pos[v2] = ((*v1).x_pos as i32 + 12) as u16);
        word!(game.eproj_x_vel[v2] = 704);
        word!(game.eproj_instr_list_ptr[v2] = addr_word_86B4CB as u16);
    } else {
        word!(game.eproj_x_pos[v2] = ((*v1).x_pos).wrapping_sub(12));
        word!(game.eproj_x_vel[v2] = -704i32 as u16);
        word!(game.eproj_instr_list_ptr[v2] = addr_word_86B4BF as u16);
    };
}
pub unsafe fn sub_86B535(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let split_amount =
        ((EprojState(&game.g_ram).slot_word(0x1ab7, v1 as usize) as i16 as i32) << 8) as u32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a4b, 0x1a27, v1 as usize, split_amount);
    let split_amount =
        ((EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as i16 as i32) << 8) as u32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a93, 0x1a6f, v1 as usize, split_amount);
    let v6: i16 = EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as i16;
    if v6 >= 0 {
        EprojStateMut(&mut game.g_ram).write(0x1adb, v1 as usize, (v6 as u16).wrapping_add(32));
        Eproj_DeleteIfYposOutside(k, game);
    } else {
        EprojStateMut(&mut game.g_ram).write(0x1adb, v1 as usize, (v6 as u16).wrapping_add(32));
        if (v6).wrapping_add(32) >= 0 {
            {
                let instruction: u16 = (if (word!(game.eproj_x_vel[v1]) & 0x8000) == 0 {
                    addr_word_86B4E3 as i32
                } else {
                    addr_word_86B4D7 as i32
                }) as u16;
                EprojStateMut(&mut game.g_ram).start_instruction(v1 as usize, instruction);
            }
        }
    };
}
pub unsafe fn Eproj_DeleteIfYposOutside(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    if ((word!(game.eproj_y_pos[v1])) as i16).wrapping_sub((word!(game.layer1_y_pos)) as i16) >= 288
    {
        EprojStateMut(&mut game.g_ram).set_id(v1 as usize, 0);
    }
}
static word_86B65B: [i16; 20] = [
    -16i16, 16, -96i16, -64i16, -128i16, -32i16, -96i16, 64, -128i16, 32, 16, 16, 96, -64i16, 112,
    -64i16, 128, -64i16, 144, -64i16,
];
unsafe fn EprojInit_EyeDoorProjectile(j: u16, game: &mut Game) {
    let v1: u16 = word!(game.plm_id);
    let v2: i32 = j as i32 >> 1;
    word!(
        game.eproj_F[v2] =
            *(ram_ptr!(game, u16; 0x1dc7)).offset((word!(game.plm_id) >> 1) as isize,)
    );
    CalculatePlmBlockCoords(v1, game);
    let v3: i32 = word!(game.cinematic_var8) as i32 >> 1;
    word! { game;
        eproj_x_pos[v2] =
            (word_86B65B[v3 as usize] as i32 + 8 * (2 * word!(game.plm_x_block) as i32 + 1)) as u16;
        eproj_y_pos[v2] =
            (word_86B65B[(v3 + 1) as usize] as i32 + 16 * word!(game.plm_y_block) as i32) as u16;
    }
}
static word_86B6B1: [i16; 4] = [-64i16, 512, 64, 512];
unsafe fn EprojInit_EyeDoorSweat(j: u16, game: &mut Game) {
    CalculatePlmBlockCoords(word!(game.plm_id), game);
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v1] = (8 * (2 * (word!(game.plm_x_block) as i32 - 1i32) + 1)) as u16);
    word!(game.eproj_y_pos[v1] = (16 * (word!(game.plm_y_block) as i32 + 1)) as u16);
    let v2: i32 = word!(game.cinematic_var8) as i32 >> 1;
    word!(game.eproj_x_vel[v1] = word_86B6B1[v2 as usize] as u16);
    word!(game.eproj_y_vel[v1] = word_86B6B1[(v2 + 1) as usize] as u16);
}
pub unsafe fn EprojPreInstr_EyeDoorProjectile(k: u16, game: &mut Game) {
    let v3: i32 = k as i32 >> 1;
    if !(EprojBlockCollisition_Horiz(k, game) & 1 != 0
        || EprojBlockCollisition_Vertical(k, game) & 1 != 0)
    {
        let v1: i32 = word!(game.eproj_E[(k >> 1)]) as i32 >> 1;
        word!(
            game.eproj_x_vel[v3],
            wrapping_add((kSinCosTable8bit_Sext[(v1 + 64) as usize] >> 4) as u16)
        );
        word!(game.eproj_y_vel[v3], |fresh101| (fresh101 as i32
            + (kSinCosTable8bit_Sext[v1 as usize] as i32 >> 4))
            as u16);
        let t: i32 = PrepareBitAccess(word!(game.eproj_F[v3]), game) as i32;
        if ((word!(game.bitmask) & *ram_ptr!(game; 0xd8b0, t as isize) as u16) == 0) {
            return;
        }
    }
    EprojStateMut(&mut game.g_ram).start_instruction(v3 as usize, addr_off_86B5F3 as u16);
}
pub unsafe fn EprojPreInstr_EyeDoorSweat(k: u16, game: &mut Game) {
    EprojBlockCollisition_Horiz(k, game);
    let carry: u8 = EprojBlockCollisition_Vertical(k, game);
    let v1: i32 = k as i32 >> 1;
    if (word!(game.eproj_y_vel[v1]) & 0x8000) != 0 || carry == 0 {
        EprojStateMut(&mut game.g_ram).add_word(0x1adb, v1 as usize, 12);
    } else {
        EprojStateMut(&mut game.g_ram).add_word(0x1a93, v1 as usize, (-4i16) as u16);
        EprojStateMut(&mut game.g_ram).start_instruction(v1 as usize, addr_off_86B61D as u16);
    };
}
pub unsafe fn EprojInstr_SpawnTourianStatueUnlockingParticle(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueUnlockingParticle as u16, k, game);
    epjp
}
pub unsafe fn EprojInstr_Earthquake(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    word!(game.earthquake_type = 1);
    word!(game.earthquake_timer, |fresh29| (fresh29 | 0x20));
    epjp
}
pub unsafe fn EprojInstr_SpawnTourianStatueUnlockingParticleTail(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_TourianStatueUnlockingParticleTail as u16,
        k,
        game,
    );
    epjp
}
pub unsafe fn EprojInstr_AddToYpos(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    word!(
        game.eproj_y_pos[(k >> 1)],
        |fresh28| (fresh28 as i32 + *(epjp as *mut u16) as i32) as u16
    );
    epjp.offset(2)
}
unsafe fn EprojInit_TourianStatueUnlockingParticleWaterSplash(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(
        game.eproj_x_pos[v1] =
            *(ram_ptr!(game, u16; 0x1a4b)).offset((word!(game.cinematic_var8) >> 1) as isize,)
    );
    word!(game.eproj_y_pos[v1] = (word!(game.fx_y_pos) as i32 - 4i32) as u16);
}
static kEprojInit_TourianStatueEyeGlow_X: [u16; 4] = [0x84, 0x7a, 0x9e, 0x68];
static kEprojInit_TourianStatueEyeGlow_Y: [u16; 4] = [0x90, 0x51, 0x80, 0x72];
static kEprojInit_TourianStatueEyeGlow_Colors: [u16; 16] = [
    0x6bff, 0x33b, 0x216, 0x173, 0x7f5f, 0x7c1f, 0x5816, 0x300c, 0x7f5a, 0x7ec0, 0x6de0, 0x54e0,
    0x6bfa, 0x3be0, 0x2680, 0x1580,
];
unsafe fn EprojInit_TourianStatueEyeGlow(j: u16, game: &mut Game) {
    let v1: u16 = word!(game.cinematic_var8);
    let v2: i32 = word!(game.cinematic_var8) as i32 >> 1;
    let v3: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v3] = kEprojInit_TourianStatueEyeGlow_X[v2 as usize]);
    word!(game.eproj_y_pos[v3] = kEprojInit_TourianStatueEyeGlow_Y[v2 as usize]);
    let mut v4: u16 = (4 * v1 as i32) as u16;
    let mut i: i32 = 498;
    while i != 506 {
        word!(
            game.palette_buffer[(i >> 1)] =
                kEprojInit_TourianStatueEyeGlow_Colors[(v4 >> 1) as usize]
        );
        v4 = (v4 as i32 + 2) as u16;
        i += 2;
    }
}
unsafe fn EprojInit_TourianStatueUnlockingParticle(j: u16, game: &mut Game) {
    let v1: i32 = word!(game.cinematic_var8) as i32 >> 1;
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v2] = word!(game.eproj_x_pos[v1]));
    word!(game.eproj_y_pos[v2] = word!(game.eproj_y_pos[v1]));
    let v3: u16 = (2 * ((NextRandom(game) as i32 & 0x3f) - 32i32) as u8 as i32) as u16;
    word!(game.eproj_E[v2] = v3);
    let v4: i32 = v3 as i32 >> 1;
    word!(game.eproj_x_vel[v2] = kSinCosTable8bit_Sext[(v4 + 64) as usize] as u16);
    word!(game.eproj_y_vel[v2] = (4 * kSinCosTable8bit_Sext[v4 as usize] as i32) as u16);
}
pub unsafe fn EprojIni_TourianStatueUnlockingParticleTail(v0: u16, game: &mut Game) {
    let v1: i32 = word!(game.cinematic_var8) as i32 >> 1;
    let v2: i32 = v0 as i32 >> 1;
    word!(game.eproj_x_pos[v2] = word!(game.eproj_x_pos[v1]));
    word!(game.eproj_y_pos[v2] = word!(game.eproj_y_pos[v1]));
}
unsafe fn EprojInit_TourianStatueSoul(j: u16, game: &mut Game) {
    let v1: i32 = word!(game.cinematic_var8) as i32 >> 1;
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v2] = kEprojInit_TourianStatueEyeGlow_X[v1 as usize]);
    word!(game.eproj_y_pos[v2] = kEprojInit_TourianStatueEyeGlow_Y[v1 as usize]);
    word!(game.eproj_y_vel[v2] = -1024i32 as u16);
}
unsafe fn EprojInit_TourianStatueBaseDecoration(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word! { game, 120; eproj_E[v1], eproj_x_pos[v1] }
    word! { game, 184; eproj_F[v1], eproj_y_pos[v1] }
}
unsafe fn EprojInit_TourianStatueRidley(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word! { game, 142; eproj_E[v1], eproj_x_pos[v1] }
    word! { game, 85; eproj_F[v1], eproj_y_pos[v1] }
}
unsafe fn EprojInit_TourianStatuePhantoon(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word! { game, 132; eproj_E[v1], eproj_x_pos[v1] }
    word! { game, 136; eproj_F[v1], eproj_y_pos[v1] }
}
pub unsafe fn EprojPreInstr_TourianStatueUnlockingParticleWaterSplash(k: u16, game: &mut Game) {
    word!(game.eproj_y_pos[(k >> 1)] = (word!(game.fx_y_pos) as i32 - 4i32) as u16);
}
pub unsafe fn EprojPreInstr_TourianStatueUnlockingParticle(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let mut amt: i32 = (word!(game.eproj_x_vel[v1]) as i16 as i32) << 8;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a4b)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a27)).offset(v1 as isize),
        amt as u32,
    );
    let v7: u16 = (word!(game.fx_y_pos) as i32 - word!(game.eproj_y_pos[v1]) as i32) as u16;
    amt = (word!(game.eproj_y_vel[v1]) as i16 as i32) << 8;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a93)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a6f)).offset(v1 as isize),
        amt as u32,
    );
    if ((v7 as i32 ^ word!(game.fx_y_pos) as i32 - word!(game.eproj_y_pos[v1]) as i32) & 0x8000)
        != 0
    {
        SpawnEprojWithRoomGfx(addr_stru_86BA5C as u16, k, game);
    }
    if (word!(game.eproj_y_pos[v1]) & 0xff00) == 256 {
        word!(game.eproj_instr_list_ptr[v1] = addr_off_86B79F as u16);
        word!(game.eproj_instr_timers[v1] = 1);
    } else {
        word!(game.eproj_y_vel[v1], |fresh97| (fresh97 as i32 + 16) as u16);
    };
}
pub unsafe fn EprojPreInstr_TourianStatueSoul(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let amt: i32 = (word!(game.eproj_y_vel[v1]) as i16 as i32) << 8;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a93)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a6f)).offset(v1 as isize),
        amt as u32,
    );
    if (word!(game.eproj_y_pos[v1]) & 0x100) != 0 {
        word!(game.eproj_instr_list_ptr[v1] = addr_off_86B79F as u16);
        word!(game.eproj_instr_timers[v1] = 1);
    }
    word!(game.eproj_y_vel[v1], |fresh96| (fresh96 as i32 - 128i32)
        as u16);
}
unsafe fn EprojPreInstr_BA42(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word!(game.eproj_x_pos[v1] = word!(game.eproj_E[v1]));
    word!(
        game.eproj_y_pos[v1] = (word!(game.eproj_F[v1]) as i32 + word!(game.layer1_y_pos) as i32
            - *(&raw mut (*(ram_ptr!(game, ExpandingSquareTransitionHdma; 0x9e00))).field_0
                as *mut u16) as i32) as u16
    );
}
pub unsafe fn EprojPreInstr_TourianStatueStuff(k: u16, game: &mut Game) {
    if word!(game.tourian_entrance_statue_animstate) == 0 {
        word!(game.tourian_entrance_statue_finished, |fresh95| (fresh95
            | 0x8000));
    }
    EprojPreInstr_BA42(k, game);
}
pub unsafe fn sub_86BB30(j: u16, game: &mut Game) {
    let v1 = gExtraEnemyRam8000(word!(game.cur_enemy_index) as i32, game);
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v2] = *((&raw mut (*v1).pad as *mut u8).offset(34) as *mut u16));
    word!(game.eproj_y_pos[v2] = *((&raw mut (*v1).pad as *mut u8).offset(36) as *mut u16));
    word!(
        game.eproj_instr_list_ptr[v2] =
            crate::rtl::rom_table_word!(game, 0x86bb1e, word!(game.cinematic_var8) as isize)
    );
}
unsafe fn EprojInit_NuclearWaffleBody(j: u16, game: &mut Game) {
    let v2 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    let v3: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v3] = (*v2).x_pos);
    word!(game.eproj_x_subpos[v3] = (*v2).x_subpos);
    word!(game.eproj_y_pos[v3] = (*v2).y_pos);
    word!(game.eproj_y_subpos[v3] = (*v2).y_subpos);
    let v4 = gExtraEnemyRam8000(word!(game.cur_enemy_index) as i32, game);
    (*gExtraEnemyRam7800(
        word!(game.cur_enemy_index) as i32
            + *((&raw mut (*v4).pad as *mut u8).offset(20) as *mut u16) as i32,
        game,
    ))
    .kraid
    .kraid_next = j;
    word!(game.eproj_flags[v3] = 1);
}
unsafe fn EprojInit_NorfairLavaquakeRocks(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_instr_list_ptr[v1] = addr_word_86BBD5 as u16);
    word!(game.eproj_E[v1] = (0x86bc16 & 0xffff) as u16);
    word!(game.eproj_y_vel[v1] = word!(game.cinematic_var8));
    word!(game.eproj_x_vel[v1] = word!(game.cinematic_var9));
    let v2 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    word!(game.eproj_x_pos[v1] = (*v2).x_pos);
    word!(game.eproj_x_subpos[v1] = (*v2).x_subpos);
    word!(game.eproj_y_pos[v1] = (*v2).y_pos);
    word!(game.eproj_y_subpos[v1] = (*v2).y_subpos);
}
crate::game::address_dispatch! {
    pub unsafe fn CallNorfairLavaquakeRocksFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        8829974 => Eproj_NorfairLavaquakeRocks_Func1; 8830095 => Eproj_NorfairLavaquakeRocks_Func2;
    }
}
pub unsafe fn EprojPreInstr_NorfairLavaquakeRocks_Inner2(k: u16, game: &mut Game) -> u16 {
    let v1: i32 = k as i32 >> 1;
    (((word!(game.eproj_x_pos[v1]) as i32 - word!(game.layer1_x_pos) as i32) as i16) < 0
        || ((word!(game.layer1_x_pos) as i32 + 256 - word!(game.eproj_x_pos[v1]) as i32) as i16)
            < 0
        || ((word!(game.eproj_y_pos[v1]) as i32 - word!(game.layer1_y_pos) as i32) as i16) < 0
        || ((word!(game.layer1_y_pos) as i32 + 256 - word!(game.eproj_y_pos[v1]) as i32) as i16)
            < 0) as u16
}
pub unsafe fn EprojPreInstr_NorfairLavaquakeRocks_Inner(k: u16, game: &mut Game) {
    if EprojPreInstr_NorfairLavaquakeRocks_Inner2(k, game) != 0 {
        word!(game.eproj_id[(k >> 1)] = 0);
    }
}
pub unsafe fn EprojPreInstr_NorfairLavaquakeRocks(k: u16, game: &mut Game) {
    CallNorfairLavaquakeRocksFunc(
        (word!(game.eproj_E[(k >> 1)]) as i32 | 0x860000) as u32,
        k,
        game,
    );
    EprojPreInstr_NorfairLavaquakeRocks_Inner(k, game);
}
pub unsafe fn Eproj_NorfairLavaquakeRocks_Func1(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let mut v2: i16 = (word!(game.eproj_y_vel[v1]) as i32 - 2i32) as i16;
    word!(game.eproj_y_vel[v1] = v2 as u16);
    if v2 >= 0 {
        let mut n: i32 = 2;
        loop {
            let mut v3: i16 = (n + word!(game.eproj_y_vel[v1]) as i32 - 1i32) as i16;
            if v3 < 0 {
                v3 = 0;
            }
            let t: i32 = 8 * v3 as i32 + 4 >> 1;
            AddToHiLo(
                (ram_ptr!(game, u16; 0x1a93)).offset(v1 as isize),
                (ram_ptr!(game, u16; 0x1a6f)).offset(v1 as isize),
                *(rom_ptr!(game, u32; 0xa0cbc7)).offset((t >> 1) as isize),
            );
            word!(game.eproj_F[v1] = crate::rtl::rom_table_word!(game, 0xa0cbc7, (t + 1) as isize));
            n -= 1;
            if (n == 0) {
                break;
            }
        }
        Eproj_NorfairLavaquakeRocks_Func3(k, game);
    } else {
        word!(game.eproj_y_vel[v1] = 0);
        word!(game.eproj_E[v1] = (0x86bc8f & 0xffff) as u16);
    };
}
pub unsafe fn Eproj_NorfairLavaquakeRocks_Func2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = (word!(game.eproj_y_vel[v1]) as i32 + 2) as u16;
    word!(game.eproj_y_vel[v1] = v2);
    if v2 as i32 - 64i32 & 0x8000 == 0 {
        word!(game.eproj_y_vel[v1] = 64);
    }
    let mut n: i32 = 2;
    loop {
        let t: i32 = 8 * (word!(game.eproj_y_vel[v1]) as i32 - n + 1) >> 1;
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a93)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1a6f)).offset(v1 as isize),
            *(rom_ptr!(game, u32; 0xa0cbc7)).offset((t >> 1) as isize),
        );
        word!(game.eproj_F[v1] = crate::rtl::rom_table_word!(game, 0xa0cbc7, (t + 1) as isize));
        n -= 1;
        if (n == 0) {
            break;
        }
    }
    Eproj_NorfairLavaquakeRocks_Func3(k, game);
}
pub unsafe fn Eproj_NorfairLavaquakeRocks_Func3(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a4b)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a27)).offset(v1 as isize),
        ((word!(game.eproj_x_vel[v1]) as i16 as i32) << 8) as u32,
    );
}
static kEprojInit_ShaktoolAttackMiddleBackCircle_X: [i16; 8] =
    [0, 12, 16, 12, 0, -12i16, -16i16, -12i16];
static kEprojInit_ShaktoolAttackMiddleBackCircle_Y: [i16; 8] =
    [-16i16, -12i16, 0, 12, 16, 12, 0, -12i16];
unsafe fn EprojInit_BDA2(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    let v2 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    word!(game.eproj_x_pos[v1] = (*v2).x_pos);
    word!(game.eproj_y_pos[v1] = (*v2).y_pos);
    let v3: u16 = (2 * byte!((*v2).ai_var_D; 0) as i32) as u16;
    let v4: i32 = v3 as i32 >> 1;
    word!(game.eproj_x_vel[v1] = kSinCosTable8bit_Sext[(v4 + 64) as usize] as u16);
    word!(game.eproj_y_vel[v1] = kSinCosTable8bit_Sext[v4 as usize] as u16);
    let v5: i32 = (v3 >> 5) as i32 >> 1;
    word!(game.eproj_x_pos[v1], |fresh11| (fresh11 as i32
        + kEprojInit_ShaktoolAttackMiddleBackCircle_X[v5 as usize] as i32)
        as u16);
    word!(game.eproj_y_pos[v1], |fresh12| (fresh12 as i32
        + kEprojInit_ShaktoolAttackMiddleBackCircle_Y[v5 as usize] as i32)
        as u16);
}
unsafe fn EprojInit_ShaktoolAttackMiddleBackCircle(j: u16, game: &mut Game) {
    word!(game.eproj_E[(j >> 1)] = word!(game.cinematic_var8));
    EprojInit_BDA2(j, game);
}
unsafe fn EprojInit_ShaktoolAttackFrontCircle(v1: u16, game: &mut Game) {
    if EprojBlockCollisition_Horiz(v1, game) & 1 != 0
        || EprojBlockCollisition_Vertical(v1, game) & 1 != 0
    {
        word!(game.eproj_id[(v1 >> 1)] = 0);
    }
}
pub unsafe fn EprojPreInstr_BE12(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    if *(ram_ptr!(game, u16; 0x1997)).offset((word!(game.eproj_E[v1]) >> 1) as isize) != 0 {
        EprojBlockCollisition_Horiz(k, game);
        EprojBlockCollisition_Vertical(k, game);
    } else {
        word!(game.eproj_id[v1] = 0);
    };
}
static kEprojInit_MotherBrainRoomTurrets_X: [i16; 12] = [
    0x398, 0x348, 0x328, 0x2d8, 0x288, 0x268, 0x218, 0x1c8, 0x1a8, 0x158, 0x108, 0xe8,
];
static EprojInit_MotherBrainRoomTurrets_Y: [i16; 12] = [
    0x30, 0x40, 0x40, 0x30, 0x40, 0x40, 0x30, 0x40, 0x40, 0x30, 0x40, 0x40,
];
unsafe fn EprojInit_MotherBrainRoomTurrets(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_gfx_idx[v1] = 1024);
    let v2: u16 = (2 * word!(game.cinematic_var8) as i32) as u16;
    let v3: u16 =
        (crate::rtl::rom_table_word!(game, 0x86bee1, word!(game.cinematic_var8) as isize) | 0x100);
    word!(game.eproj_y_subpos[v1] = v3);
    word!(
        game.eproj_instr_list_ptr[v1] = crate::rtl::rom_table_word!(
            game,
            0x86beb9,
            ((2 * v3 as u8 as i32) as u16 >> 1) as isize,
        )
    );
    let v4: i32 = v2 as i32 >> 1;
    word!(game.eproj_x_pos[v1] = kEprojInit_MotherBrainRoomTurrets_X[v4 as usize] as u16);
    word!(game.eproj_y_pos[v1] = EprojInit_MotherBrainRoomTurrets_Y[v4 as usize] as u16);
    word!(game.eproj_x_subpos[v1] = crate::rtl::rom_table_word!(game, 0x86bec9, v4 as isize));
    Eproj_SetXvelRandom(j, game);
    Eproj_SetYvelRandom(j, game);
}
static kEproj_MotherBrainRoomTurretBullets_X: [i16; 8] = [-17i16, -12i16, 0, 12, 17, 12, 0, -12i16];
static kEproj_MotherBrainRoomTurretBullets_Y: [i16; 8] =
    [-9i16, 3, 7, 3, -9i16, -19i16, -21i16, -19i16];
static kEproj_MotherBrainRoomTurretBullets_Xvel: [i16; 8] =
    [-704i16, -498i16, 0, 498, 704, 498, 0, -498i16];
static kEproj_MotherBrainRoomTurretBullets_Yvel: [i16; 8] =
    [0, 498, 704, 498, 0, -498i16, -704i16, -498i16];
unsafe fn EprojInit_MotherBrainRoomTurretBullets(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_F[v1] = 0);
    word!(game.eproj_gfx_idx[v1] = 1024);
    let v2: u16 = (2 * byte!(@pointer (ram_ptr!(game, u16; 0x1a6f))
        .offset((word!(game.cinematic_var8) >> 1) as isize); 0) as i32) as u16;
    word!(game.eproj_E[v1] = v2);
    let v3: i32 = v2 as i32 >> 1;
    let pt: Point16U = Point16U {
        x: kEproj_MotherBrainRoomTurretBullets_X[v3 as usize] as u16,
        y: kEproj_MotherBrainRoomTurretBullets_Y[v3 as usize] as u16,
    };
    word!(game.eproj_x_vel[v1] = kEproj_MotherBrainRoomTurretBullets_Xvel[v3 as usize] as u16);
    word!(game.eproj_y_vel[v1] = kEproj_MotherBrainRoomTurretBullets_Yvel[v3 as usize] as u16);
    let v4: i32 = word!(game.cinematic_var8) as i32 >> 1;
    word!(game.eproj_x_pos[v1] = (pt.x as i32 + word!(game.eproj_x_pos[v4]) as i32) as u16);
    word!(game.eproj_y_pos[v1] = (pt.y as i32 + word!(game.eproj_y_pos[v4]) as i32) as u16);
}
pub unsafe fn EprojPreInstr_MotherBrainRoomTurrets(k: u16, game: &mut Game) {
    if Eproj_MotherBrainRoomTurretBullets_CheckIfTurretOnScreen(k, game) & 1 != 0 {
        if (*gRam7800_Default(0u16, game)).var_1D != 0 {
            *((ram_ptr!(game, u16; 0x1997) as *mut u8).offset(k as i32 as isize) as *mut u16) = 0;
        }
    } else if (*gRam7800_Default(0u16, game)).var_1D != 0 {
        *((ram_ptr!(game, u16; 0x1997) as *mut u8).offset(k as i32 as isize) as *mut u16) = 0;
        let v5: i32 = k as i32 >> 1;
        game.eproj_spawn_pt = Point16U {
            x: word!(game.eproj_x_pos[v5]),
            y: word!(game.eproj_y_pos[v5]),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0xc, game);
    } else {
        let v1: i32 = k as i32 >> 1;
        crate::update_place! { reference fresh88 [fresh89] = *(ram_ptr!(game, u16; 0x1ab7)).offset(v1 as isize) => (*fresh88).wrapping_sub(1); }
        let mut v2: bool = fresh89 == 1;
        if v2 {
            Eproj_SetXvelRandom(k, game);
            Eproj_MotherBrainRoomTurretBullets_Func2(k, game);
            let v3: i32 = k as i32 >> 1;
            word!(
                game.eproj_instr_list_ptr[v3] = *(rom_ptr!(game, u16; 0x86c040)).offset(
                    *((ram_ptr!(game, u16; 0x1a6f)).offset(v3 as isize) as *mut u8) as isize,
                )
            );
            word!(game.eproj_instr_timers[v3] = 1);
        }
        let v4: i32 = k as i32 >> 1;
        crate::update_place! { reference fresh92 [fresh93] = *(ram_ptr!(game, u16; 0x1adb)).offset(v4 as isize) => (*fresh92).wrapping_sub(1); }
        v2 = fresh93 == 1;
        if v2 {
            Eproj_SetYvelRandom(k, game);
            SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainRoomTurretBullets as u16, k, game);
        }
    };
}
pub unsafe fn Eproj_MotherBrainRoomTurretBullets_Func2(v0: u16, game: &mut Game) {
    let r20: u8 = (*(ram_ptr!(game, u16; 0x1a6f) as *mut u8)
        .offset(v0 as i32 as isize)
        .offset(1) as i32
        + byte!(@pointer ram_ptr!(game, u16; 0x1a6f); v0 as i32 as isize) as i32
        & 7) as u8;
    let r18: u16 = word!(game.eproj_x_subpos[(v0 >> 1)]);
    if *RomPtr_86(r18, game).offset((r20 as i32 & 7) as isize) != 0 {
        byte!(@pointer ram_ptr!(game, u16; 0x1a6f); v0 as i32 as isize) = r20;
    } else {
        let v1: i8 = -(*(ram_ptr!(game, u16; 0x1a6f) as *mut u8)
            .offset(v0 as i32 as isize)
            .offset(1) as i32) as i8;
        *(ram_ptr!(game, u16; 0x1a6f) as *mut u8)
            .offset(v0 as i32 as isize)
            .offset(1) = v1 as u8;
        crate::update_place! { reference fresh94 = *(ram_ptr!(game, u16; 0x1a6f) as *mut u8).offset(v0 as i32 as isize) => (*fresh94 as i32 + v1 as i32) as u8; }
    };
}
pub unsafe fn Eproj_SetXvelRandom(v0: u16, game: &mut Game) {
    let mut Random: u16 = NextRandom(game) as u8 as u16;
    if Random as u8 as i32 - 32i32 & 0x8000 != 0 {
        Random = 32;
    }
    word!(game.eproj_x_vel[(v0 >> 1)] = Random);
}
pub unsafe fn Eproj_SetYvelRandom(v0: u16, game: &mut Game) {
    let mut Random: u16 = NextRandom(game) as u8 as u16;
    if Random as u8 as i32 - 128i32 & 0x8000 != 0 {
        Random = 128;
    }
    word!(game.eproj_y_vel[(v0 >> 1)] = Random);
}
pub unsafe fn Eproj_MotherBrainRoomTurretBullets_CheckIfTurretOnScreen(
    k: u16,
    game: &mut Game,
) -> u8 {
    let v1: i32 = k as i32 >> 1;
    let mut v2: i16 = word!(game.eproj_y_pos[v1]) as i16;
    let result: u8 = 1;
    if v2 >= 0 {
        let mut v3: i16 = (v2 as i32 + 16 - word!(game.layer1_y_pos) as i32) as i16;
        if v3 >= 0 && v3 as i32 - 256i32 & 0x8000 != 0 {
            let mut v4: i16 = word!(game.eproj_x_pos[v1]) as i16;
            if v4 >= 0 {
                let mut v5: i16 = (v4 as i32 + 4 - word!(game.layer1_x_pos) as i32) as i16;
                if v5 >= 0 && v5 as i32 - 264i32 & 0x8000 != 0 {
                    return 0;
                }
            }
        }
    }
    result
}
pub unsafe fn EprojPreInstr_MotherBrainRoomTurretBullets(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word!(game.eproj_properties[v1], |fresh85| (fresh85 ^ 0x8000));
    MoveEprojWithVelocity(k, game);
    if Ridley_Func_103(
        word!(game.eproj_x_pos[v1]),
        word!(game.eproj_y_pos[v1]),
        game,
    ) & 1
        != 0
    {
        *(ram_ptr!(game, u16; 0x1997)).offset((word!(game.cinematic_var7) >> 1) as isize) = 0;
    }
}
static kMotherBrainsBomb_Yaccel: [i16; 10] =
    [7, 0x10, 0x20, 0x40, 0x70, 0xb0, 0xf0, 0x130, 0x170, 0];
pub unsafe fn EprojPreInstr_MotherBrainBomb(k: u16, game: &mut Game) {
    if MotherBrainBomb_Bomb_CollDetect_DoubleRet(k, game) != 0 {
        return;
    }
    let v1: i32 = k as i32 >> 1;

    let advance_bomb_phase: bool = if word!(game.eproj_F[v1]) != 0 {
        let v4: i32 = k as i32 >> 1;
        let v5: u16 = kMotherBrainsBomb_Yaccel[(word!(game.eproj_F[v4]) >> 1) as usize] as u16;
        if v5 == 0 {
            word! { game, 0; eproj_x_vel[v4], eproj_y_vel[v4] }
            let fresh81 = &raw mut (*(ram_ptr!(game, ExtraEnemyRam7800; 0x7800)).offset(1))
                .kraid
                .kraid_mouth_flags;
            fresh81.write_unaligned(fresh81.read_unaligned().wrapping_sub(1));
            word!(game.eproj_id[v4] = 0);
            game.eproj_spawn_pt = Point16U {
                x: word!(game.eproj_x_pos[v4]),
                y: word!(game.eproj_y_pos[v4]),
            };
            SpawnEprojWithRoomGfx(
                addr_stru_869650 as u16,
                byte!(@pointer (ram_ptr!(game, u16; 0x1a27)).offset(v4 as isize); 0) as u16,
                game,
            );
            game.eproj_spawn_pt = Point16U {
                x: word!(game.eproj_x_pos[v4]),
                y: word!(game.eproj_y_pos[v4]),
            };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3, game);
            QueueSfx3_Max6(0x13, game);
            return;
        }
        (MoveMotherBrainBomb(k, v5, game) & 1 != 0)
    } else {
        let mut v2: i16 = (abs16(word!(game.eproj_x_vel[v1])) as i32 - 2i32) as i16;
        if v2 < 0 {
            v2 = 0;
        }
        word!(
            game.eproj_x_vel[v1] = (if word!(game.eproj_x_vel[v1]) & 0x8000 != 0 {
                -(v2 as i32)
            } else {
                v2 as i32
            }) as u16
        );
        (MoveMotherBrainBomb(k, 7, game) & 1 != 0)
    };
    if advance_bomb_phase {
        let v3: i32 = k as i32 >> 1;
        word!(game.eproj_F[v3], |fresh82| (fresh82 as i32 + 2) as u16);
    };
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, epjp: *const u8, game: &mut Game) -> *const u8;
    EprojInstr_SwitchJump [*(epjp.offset(word!(game.eproj_E[(k >> 1)]) as i32 as isize) as *mut u16) as usize
            as *const u8] {
    };
    EprojInstr_UserPalette0 [epjp] { word!(game.eproj_gfx_idx[(k >> 1)] = 0); };
}
unsafe fn Eproj_CheckForBombCollisionWithRect(rect: Rect16U, game: &mut Game) -> u8 {
    if word!(game.bomb_counter) == 0 {
        return 0;
    }
    let mut v1: i32 = 10;
    while v1 < 20 {
        let v2: i32 = v1 >> 1;
        if (word!(game.projectile_type[v2]) & 0xf00) == 1280
            && word!(game.projectile_variables[v2]) == 0
        {
            let x: u16 = abs16((word!(game.projectile_x_pos[v2]) as i32 - rect.x as i32) as u16);
            if x < word!(game.projectile_x_radius[v2])
                || ((x as i32 - word!(game.projectile_x_radius[v2]) as i32) as u16) < rect.w
            {
                let y: u16 =
                    abs16((word!(game.projectile_y_pos[v2]) as i32 - rect.y as i32) as u16);
                if y < word!(game.projectile_y_radius[v2])
                    || ((y as i32 - word!(game.projectile_y_radius[v2]) as i32) as u16) < rect.h
                {
                    return 1;
                }
            }
        }
        v1 += 2;
    }
    0
}
unsafe fn Eproj_CheckForEnemyCollisionWithRect(k: u16, rect: Rect16U, game: &mut Game) -> u8 {
    let v1 = gEnemyData(k as i32, game);
    ((abs16(((*v1).x_pos as i32 - rect.x as i32) as u16) as i32 - (*v1).x_width as i32)
        < rect.w as i32
        && (abs16(((*v1).y_pos as i32 - rect.y as i32) as u16) as i32 - (*v1).y_height as i32)
            < rect.h as i32) as u8
}
unsafe fn Eproj_CheckForCollisionWithSamus(v0: u16, game: &mut Game) -> u8 {
    let rect: Rect16U = Eproj_GetCollDetectRect(v0, game);
    let v1: u16 = abs16((word!(game.samus_x_pos) as i32 - rect.x as i32) as u16);
    let mut v2: bool = v1 < word!(game.samus_x_radius);
    let v3: u16 = (v1 as i32 - word!(game.samus_x_radius) as i32) as u16;
    let result: u8 = 0;
    if v2 || v3 < rect.w {
        let v4: u16 = abs16((word!(game.samus_y_pos) as i32 - rect.y as i32) as u16);
        v2 = v4 < word!(game.samus_y_radius);
        let v5: u16 = (v4 as i32 - word!(game.samus_y_radius) as i32) as u16;
        if v2 || v5 < rect.h {
            return 1;
        }
    }
    result
}
crate::forward_calls! { pub unsafe fn (a: u16, r18: u16) -> u16;
    Math_MultBySin => Math_MultBySinCos(a, r18);
    Math_MultByCos => Math_MultBySinCos(a, r18.wrapping_add(64));
}
pub unsafe fn Math_MultBySinCos(r38: u16, a: u16) -> u16 {
    let r46: u16 = kSinCosTable8bit_Sext[((a as i32 & 0xff) + 64) as usize] as u16;
    let prod: u16 = ((r38 as u32).wrapping_mul(abs16(r46) as u32) >> 8) as u16;
    ((if r46 & 0x8000 != 0 {
        -(prod as i32)
    } else {
        prod as i32
    }) as u16)
}
pub unsafe fn Eproj_MotherBrainsBlueRingLasers(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_E[v1] = 8);
    word!(game.eproj_F[v1] = 0);
    word!(game.eproj_gfx_idx[v1] = 1024);
    let r18: u16 = word!(game.cinematic_var8);
    word! { game;
        eproj_x_vel[v1] = Math_MultBySin(0x450, r18);
        eproj_y_vel[v1] = Math_MultByCos(0x450, r18);
        eproj_x_pos[v1] =
            ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).x_pos as i32 + 10) as u16;
        eproj_y_pos[v1] =
            ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).y_pos as i32 + 16) as u16;
    }
    sub_86C320(j, game);
}
pub unsafe fn sub_86C320(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word! { game;
        eproj_x_pos[v1] =
            ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).x_pos as i32 + 10) as u16;
        eproj_y_pos[v1] =
            ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).y_pos as i32 + 16) as u16;
    }
}
pub unsafe fn Eproj_MoveToBlueRingSpawnPosition(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    if word!(game.eproj_E[v1]) != 0 {
        word!(game.eproj_E[v1], wrapping_sub(1));
        sub_86C320(k, game);
    } else {
        MoveEprojWithVelocity(k, game);
        let t: u8 = CheckForCollisionWithShitroid_DoubleRet(k, game);
        if t & 0x80 != 0 {
            return;
        }
        if t != 0 {
            let fresh84 = &raw mut (*(ram_ptr!(game, ExtraEnemyRam7800; 0x7800)))
                .kraid
                .field_28;
            fresh84.write_unaligned(fresh84.read_unaligned().wrapping_add(1));
            BlueRingContactExplosion(k, game);
            let v3: u16 = (*(ram_ptr!(game, ExtraEnemyRam7800; 0x7800)).offset(1))
                .kraid
                .kraid_healths_8ths[4i32 as usize];
            (*gExtraEnemyRam7800(
                (*(ram_ptr!(game, ExtraEnemyRam7800; 0x7800)).offset(1))
                    .kraid
                    .kraid_healths_8ths[4i32 as usize] as i32,
                game,
            ))
            .kraid
            .kraid_healths_8ths[0i32 as usize] = 16;
            let v4 = gEnemyData(v3 as i32, game);
            let mut v5: i16 = ((*v4).health as i32 - 80i32) as i16;
            if v5 < 0 {
                v5 = 0;
            }
            (*v4).health = v5 as u16;
        } else if Eproj_CheckForCollisionWithSamus(k, game) & 1 != 0 {
            BlueRingContactExplosion(k, game);
            Samus_DealDamage(SuitDamageDivision(0x50, game), game);
            word!(game.samus_invincibility_timer = 96);
            word!(game.samus_knockback_timer = 5);
            word!(
                game.knockback_x_dir = (((word!(game.samus_x_pos) as i32
                    - word!(game.eproj_x_pos[(k >> 1)]) as i32)
                    as i16)
                    >= 0) as u16
            );
        } else if CheckForBlueRingCollisionWithRoom(k, game) & 1 != 0 {
            Eproj_Earthqhake5(k, game);
        }
    };
}
pub unsafe fn CheckForCollisionWithShitroid_DoubleRet(v0: u16, game: &mut Game) -> u8 {
    if (*(ram_ptr!(game, ExtraEnemyRam7800; 0x7800)).offset(1))
        .kraid
        .kraid_healths_8ths[4i32 as usize]
        == 0
    {
        return 0;
    }
    if (*gEnemyData(
        (*(ram_ptr!(game, ExtraEnemyRam7800; 0x7800)).offset(1))
            .kraid
            .kraid_healths_8ths[4i32 as usize] as i32,
        game,
    ))
    .health
        != 0
    {
        let rect: Rect16U = Eproj_GetCollDetectRect(v0, game);
        return Eproj_CheckForEnemyCollisionWithRect(
            (*(ram_ptr!(game, ExtraEnemyRam7800; 0x7800)).offset(1))
                .kraid
                .kraid_healths_8ths[4i32 as usize],
            rect,
            game,
        );
    }
    word!(game.eproj_id[(v0 >> 1)] = 0);
    0xff
}
pub unsafe fn CheckForBlueRingCollisionWithRoom(k: u16, game: &mut Game) -> u8 {
    let v1: i32 = k as i32 >> 1;
    let result: u8 = 1;
    if word!(game.eproj_y_pos[v1]) as i32 - 32i32 & 0x8000 == 0
        && word!(game.eproj_y_pos[v1]) < 0xd8
    {
        let mut v2: i16 = word!(game.eproj_x_pos[v1]) as i16;
        if v2 >= 0 {
            let mut v3: i16 = (v2 as i32 - word!(game.layer1_x_pos) as i32) as i16;
            if v3 >= 0 && v3 as i32 - 248i32 & 0x8000 != 0 {
                return 0;
            }
        }
    }
    result
}
unsafe fn Eproj_GetCollDetectRect(k: u16, game: &mut Game) -> Rect16U {
    let v1: i32 = k as i32 >> 1;
    let rect: Rect16U = Rect16U {
        x: word!(game.eproj_x_pos[v1]),
        y: word!(game.eproj_y_pos[v1]),
        w: byte!(@pointer (ram_ptr!(game, u16; 0x1bb3)).offset(v1 as isize); 0) as u16,
        h: byte!(@pointer (ram_ptr!(game, u16; 0x1bb3)).offset(v1 as isize); 1) as u16,
    };
    rect
}
pub unsafe fn Eproj_Earthqhake5(k: u16, game: &mut Game) {
    word!(game.earthquake_timer = 10);
    word!(game.earthquake_type = 5);
    BlueRingContactExplosion(k, game);
}
pub unsafe fn BlueRingContactExplosion(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word!(game.eproj_id[v1] = 0);
    game.eproj_spawn_pt = Point16U {
        x: word!(game.eproj_x_pos[v1]),
        y: word!(game.eproj_y_pos[v1]),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3, game);
    QueueSfx3_Max6(0x13, game);
    sub_86C42E(k, game);
}
pub use self::sub_869DA5 as sub_86C42E;
unsafe fn EprojInit_MotherBrainBomb(j: u16, game: &mut Game) {
    byte!(@pointer ram_ptr!(game, u16; 0x1a27); j as i32 as isize) =
        word!(game.cinematic_var8) as u8;
    let v1: i32 = j as i32 >> 1;
    word! { game;
        eproj_y_vel[v1] = 256;
        eproj_x_vel[v1] = 224;
        eproj_x_pos[v1] =
            ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).x_pos as i32 + 12) as u16;
        eproj_y_pos[v1] =
            ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).y_pos as i32 + 16) as u16;
        eproj_gfx_idx[v1] = 1024;
        eproj_E[v1] = 112;
        eproj_F[v1] = 0;
    }
    let fresh10 = &raw mut (*(ram_ptr!(game, ExtraEnemyRam7800; 0x7800)).offset(1))
        .kraid
        .kraid_mouth_flags;
    fresh10.write_unaligned(fresh10.read_unaligned().wrapping_add(1));
}
pub unsafe fn MotherBrainBomb_Bomb_CollDetect_DoubleRet(k: u16, game: &mut Game) -> u8 {
    let v1: i32 = k as i32 >> 1;
    let rect: Rect16U = Rect16U {
        x: word!(game.eproj_x_pos[v1]),
        y: word!(game.eproj_y_pos[v1]),
        w: byte!(@pointer (ram_ptr!(game, u16; 0x1bb3)).offset(v1 as isize); 0) as u16,
        h: byte!(@pointer (ram_ptr!(game, u16; 0x1bb3)).offset(v1 as isize); 1) as u16,
    };
    if Eproj_CheckForBombCollisionWithRect(rect, game) == 0 {
        return 0;
    }
    let fresh2 = &raw mut (*(ram_ptr!(game, ExtraEnemyRam7800; 0x7800)).offset(1))
        .kraid
        .kraid_mouth_flags;
    fresh2.write_unaligned(fresh2.read_unaligned().wrapping_sub(1));
    word! { game, 0; eproj_x_vel[v1], eproj_y_vel[v1], eproj_id[v1] }
    game.eproj_spawn_pt = Point16U {
        x: word!(game.eproj_x_pos[v1]),
        y: word!(game.eproj_y_pos[v1]),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, game);
    game.eproj_spawn_pt = Point16U {
        x: word!(game.eproj_x_pos[v1]),
        y: word!(game.eproj_y_pos[v1]),
    };
    SpawnEnemyDrops(addr_kEnemyDef_EC3F as u16, k, 0, game);
    1
}
pub unsafe fn MoveMotherBrainBomb(k: u16, a: u16, game: &mut Game) -> u8 {
    let v2: i32 = k as i32 >> 1;
    word!(game.eproj_y_vel[v2], |fresh16| (fresh16 as i32 + a as i32)
        as u16);
    MoveEprojWithVelocity(k, game);
    if word!(game.eproj_x_pos[v2]) as i32 - 240i32 & 0x8000 == 0 {
        word!(game.eproj_x_vel[v2] = -(word!(game.eproj_x_vel[v2]) as i32) as u16);
    }
    if word!(game.eproj_y_pos[v2]) as i32 - 208i32 & 0x8000 != 0 {
        return 0;
    }
    word!(game.eproj_y_pos[v2] = 208);
    word!(
        game.eproj_x_vel[v2] = (if word!(game.eproj_x_vel[v2]) & 0x8000 != 0 {
            -(word!(game.eproj_E[v2]) as i32)
        } else {
            word!(game.eproj_E[v2]) as i32
        }) as u16
    );
    word!(game.eproj_y_vel[v2] = -512i32 as u16);
    1
}
pub unsafe fn sub_86C605(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word! { game, 0;
        eproj_E[v1], eproj_F[v1], eproj_x_vel[v1], eproj_y_vel[v1], eproj_x_subpos[v1],
        eproj_y_subpos[v1],
    }
    let E = Get_MotherBrain(0, game);
    set_fields! {[(*E)], 0; var_23, var_25}
    word!(game.eproj_gfx_idx[v1] = 1024);
    crate::update_place! { reference fresh8 = *(ram_ptr!(game, u16; 0x1a4b)).offset(v1 as isize) => ((*E).base.x_pos as i32 + 64) as u16; }
    (*E).var_24 = *fresh8;
    let x: u16 = (word!(game.samus_x_pos) as i32 - (*E).var_24 as i32) as u16;
    crate::update_place! { reference fresh9 = *(ram_ptr!(game, u16; 0x1a93)).offset(v1 as isize) => ((*E).base.y_pos as i32 - 48i32) as u16; }
    (*E).var_26 = *fresh9;
    let y: u16 = (word!(game.samus_y_pos) as i32 - (*E).var_26 as i32) as u16;
    let r18: i32 = -(CalculateAngleFromXY(x, y, game) as i32 + 0x80i32) as u8 as i32;
    (*E).var_29 = r18 as u16;
    (*E).var_27 = Math_MultBySin(0xc00, r18 as u16);
    (*E).var_28 = Math_MultByCos(0xc00, r18 as u16);
}
unsafe fn EprojInit_MotherBrainDeathBeemFired(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    let E = Get_MotherBrain(0, game);
    word!(game.eproj_x_pos[v1] = (*E).var_24);
    word!(game.eproj_x_subpos[v1] = (*E).var_23);
    word!(game.eproj_y_pos[v1] = (*E).var_26);
    word!(game.eproj_y_subpos[v1] = (*E).var_25);
    word!(game.eproj_x_vel[v1] = (*E).var_27);
    word!(game.eproj_y_vel[v1] = (*E).var_28);
    MoveEprojWithVelocity(j, game);
    (*E).var_24 = word!(game.eproj_x_pos[v1]);
    (*E).var_23 = word!(game.eproj_x_subpos[v1]);
    (*E).var_26 = word!(game.eproj_y_pos[v1]);
    (*E).var_25 = word!(game.eproj_y_subpos[v1]);
    let r18: u16 = (byte!((*E).var_29; 0) as i32 + NextRandom(game) as i32) as u8 as u16;
    let rv: u16 = NextRandom(game);
    word!(game.eproj_x_vel[v1] = Math_MultBySin((rv & 0x700), r18,));
    word!(game.eproj_y_vel[v1] = Math_MultByCos((word!(game.random_number) & 0x700), r18,));
    MoveEprojWithVelocity(j, game);
    if word!(game.eproj_y_pos[v1]) as i32 - 34i32 & 0x8000 != 0
        || word!(game.eproj_y_pos[v1]) as i32 - 206i32 & 0x8000 == 0
        || word!(game.eproj_x_pos[v1]) as i32 - 2i32 & 0x8000 != 0
        || word!(game.eproj_x_pos[v1]) as i32 - 238i32 & 0x8000 == 0
    {
        word!(game.eproj_id[v1] = 0);
        game.eproj_spawn_pt = Point16U {
            x: word!(game.eproj_x_pos[v1]),
            y: word!(game.eproj_y_pos[v1]),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x1d, game);
        QueueSfx3_Max6(0x13, game);
        word!(game.earthquake_timer = 10);
        word!(game.earthquake_type = 5);
    } else {
        word!(game.eproj_E[v1] = (word!(game.eproj_E[v1]) as i32 + 1 & 3) as u16);
        word! { game, 0; eproj_F[v1], eproj_x_vel[v1], eproj_y_vel[v1] }
    };
}
pub unsafe fn SpawnMotherBrainDeathBeam(x: u16, game: &mut Game) {
    SpawnEprojWithRoomGfx(
        addr_kEproj_MotherBrainDeathBeamFired as u16,
        word!(game.eproj_E[(x >> 1)]),
        game,
    );
}
pub unsafe fn EprojPreInstr_MotherBrainRainbowBeam(k: u16, game: &mut Game) {
    let v2: i32 = k as i32 >> 1;
    word!(game.eproj_x_pos[v2] = (*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).x_pos);
    word!(game.eproj_y_pos[v2] = (*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).y_pos);
}
unsafe fn EprojInit_MotherBrainRainbowBeam(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word! { game, 0; eproj_gfx_idx[v1], eproj_x_vel[v1], eproj_y_vel[v1] }
    EprojPreInstr_MotherBrainRainbowBeam(j, game);
}
static kEprojInit_MotherBrainsDrool: [i16; 12] =
    [6, 0x14, 0xe, 0x12, 8, 0x17, 0xa, 0x13, 0xb, 0x19, 0xc, 0x12];
pub unsafe fn EprojPreInstr_C84D(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let v2: i32 = (4 * word!(game.eproj_E[v1]) as i32) as u16 as i32 >> 1;
    word! { game;
        eproj_x_pos[v1] = ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).x_pos as i32
            + kEprojInit_MotherBrainsDrool[v2 as usize] as i32) as u16;
        eproj_y_pos[v1] = ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).y_pos as i32
            + kEprojInit_MotherBrainsDrool[(v2 + 1) as usize] as i32)
            as u16;
    }
    word! { game, 0; eproj_x_vel[v1], eproj_y_vel[v1] }
}
unsafe fn EprojInit_MotherBrainsDrool(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_gfx_idx[v1] = 0);
    word!(game.eproj_E[v1] = word!(game.cinematic_var8));
    EprojPreInstr_C84D(j, game);
}
unsafe fn EprojInit_MotherBrainsDrool_Falling(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word!(game.eproj_y_vel[v1], |fresh79| (fresh79 as i32 + 12) as u16);
    if MoveEprojWithVelocityY(k, game) >= 0xd7 {
        word!(game.eproj_y_pos[v1], |fresh80| (fresh80 as i32 - 4i32)
            as u16);
        word!(game.eproj_instr_list_ptr[v1] = addr_off_86C8E1 as u16);
        word!(game.eproj_instr_timers[v1] = 1);
    }
}
pub unsafe fn EprojInstr_Add12ToY(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    word!(game.eproj_y_pos[(k >> 1)], |fresh27| (fresh27 as i32 + 12)
        as u16);
    epjp
}
pub unsafe fn EprojPreInit_MotherBrainsDeathExplosion_0(k: u16, game: &mut Game) {
    let v2: i32 = k as i32 >> 1;
    word! { game;
        eproj_x_pos[v2] = ((*(ram_ptr!(game, EnemyData; 0xf78))).x_pos as i32
            + word!(game.eproj_x_vel[v2]) as i32) as u16;
        eproj_y_pos[v2] = ((*(ram_ptr!(game, EnemyData; 0xf78))).y_pos as i32
            + word!(game.eproj_y_vel[v2]) as i32) as u16;
    }
}
unsafe fn EprojInit_MotherBrainsDeathExplosion(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(
        game.eproj_instr_list_ptr[v1] =
            crate::rtl::rom_table_word!(game, 0x86c929, word!(game.cinematic_var8) as isize)
    );
    word!(game.eproj_instr_timers[v1] = 1);
    word!(game.eproj_gfx_idx[v1] = 0);
    word!(game.eproj_x_vel[v1] = game.eproj_spawn_pt.x);
    word!(game.eproj_y_vel[v1] = game.eproj_spawn_pt.y);
    EprojPreInit_MotherBrainsDeathExplosion_0(j, game);
}
unsafe fn EprojInit_MotherBrainsRainbowBeamExplosion(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word! { game;
        eproj_gfx_idx[v1] = 0;
        eproj_x_vel[v1] = game.eproj_spawn_pt.x;
        eproj_x_pos[v1] = (word!(game.samus_x_pos) as i32 + game.eproj_spawn_pt.x as i32) as u16;
        eproj_y_vel[v1] = game.eproj_spawn_pt.y;
        eproj_y_pos[v1] = (word!(game.samus_y_pos) as i32 + game.eproj_spawn_pt.y as i32) as u16;
    }
}
pub unsafe fn EprojPreInstr_MotherBrainsRainbowBeamExplosion(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word! { game;
        eproj_x_pos[v1] =
            (word!(game.samus_x_pos) as i32 + word!(game.eproj_x_vel[v1]) as i32) as u16;
        eproj_y_pos[v1] =
            (word!(game.samus_y_pos) as i32 + word!(game.eproj_y_vel[v1]) as i32) as u16;
    }
}
unsafe fn EprojInit_MotherBrainEscapeDoorParticles(j: u16, game: &mut Game) {
    static kEprojInit_MotherBrainEscapeDoorParticles_X: [i16; 16] = [
        0, -0x20i16, 0, -0x18i16, 0, -0x10i16, 0, -8i16, 0, 0, 0, 8, 0, 0x10, 0, 0x18,
    ];
    static kEprojInit_MotherBrainEscapeDoorParticles_Xvel: [i16; 16] = [
        0x500, -0x200i16, 0x500, -0x100i16, 0x500, -0x100i16, 0x500, -0x80i16, 0x500, -0x80i16,
        0x500, 0x80, 0x500, -0x100i16, 0x500, 0x200,
    ];
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_gfx_idx[v1] = 0);
    let v2: i32 = 4 * word!(game.cinematic_var8) as i32 >> 1;
    word! { game;
        eproj_x_pos[v1] =
            (kEprojInit_MotherBrainEscapeDoorParticles_X[v2 as usize] as i32 + 16) as u16;
        eproj_y_pos[v1] =
            (kEprojInit_MotherBrainEscapeDoorParticles_X[(v2 + 1) as usize] as i32 + 128) as u16;
        eproj_x_vel[v1] = kEprojInit_MotherBrainEscapeDoorParticles_Xvel[v2 as usize] as u16;
        eproj_y_vel[v1] = kEprojInit_MotherBrainEscapeDoorParticles_Xvel[(v2 + 1) as usize] as u16;
        eproj_E[v1] = 32;
    }
}
pub unsafe fn EprojPreInstr_MotherBrainsExplodedDoorParticles(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let mut v2: i16 = (abs16(word!(game.eproj_x_vel[v1])) as i32 - 16i32) as i16;
    if v2 < 0 {
        v2 = 0;
    }
    word!(
        game.eproj_x_vel[v1] = (if word!(game.eproj_x_vel[v1]) & 0x8000 != 0 {
            -(v2 as i32)
        } else {
            v2 as i32
        }) as u16
    );
    word!(game.eproj_y_vel[v1], |fresh76| (fresh76 as i32 + 32) as u16);
    MoveEprojWithVelocity(k, game);
    if (word!(game.eproj_E[v1], wrapping_sub(1), reread) & 0x8000) != 0 {
        word!(game.eproj_id[v1] = 0);
        word!(game.eproj_y_pos[v1], |fresh78| (fresh78 as i32 - 4i32)
            as u16);
        game.eproj_spawn_pt = Point16U {
            x: word!(game.eproj_x_pos[v1]),
            y: word!(game.eproj_y_pos[v1]),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, game);
    }
}
unsafe fn EprojInit_MotherBrainPurpleBreathBig(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word! { game;
        eproj_gfx_idx[v1] = 0;
        eproj_x_pos[v1] = ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).x_pos as i32 + 6) as u16;
        eproj_y_pos[v1] =
            ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).y_pos as i32 + 16) as u16;
    }
}
unsafe fn EprojInit_MotherBrainPurpleBreathSmall(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word! { game;
        eproj_gfx_idx[v1] = 0;
        eproj_x_pos[v1] = ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).x_pos as i32 + 6) as u16;
        eproj_y_pos[v1] =
            ((*(ram_ptr!(game, EnemyData; 0xf78)).offset(1)).y_pos as i32 + 16) as u16;
    }
    (*(ram_ptr!(game, ExtraEnemyRam7800; 0x7800)).offset(1))
        .kraid
        .kraid_hurt_frame = 1;
}
pub unsafe fn EprojInstr_MotherBrainPurpleBreathIsActive(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    (*(ram_ptr!(game, ExtraEnemyRam7800; 0x7800)).offset(1))
        .kraid
        .kraid_hurt_frame = 0;
    epjp
}
pub unsafe fn EprojPreInstr_TimeBombSetJapaneseText(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word! { game, 0; eproj_x_vel[v1], eproj_y_vel[v1] }
    word!(game.eproj_x_pos[v1] = 128);
    word!(game.eproj_y_pos[v1] = 192);
}
unsafe fn EprojInit_TimeBombSetJapaneseText(j: u16, game: &mut Game) {
    word!(game.eproj_gfx_idx[(j >> 1)] = 0);
    EprojPreInstr_TimeBombSetJapaneseText(j, game);
}
unsafe fn EprojInit_MotherBrainTubeFalling(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_gfx_idx[v1] = 3584);
    word! { game, 0; eproj_x_vel[v1], eproj_y_vel[v1] }
    word!(game.eproj_x_pos[v1] = game.eproj_spawn_pt.x);
    word!(game.eproj_y_pos[v1] = game.eproj_spawn_pt.y);
    word!(game.eproj_E[v1] = (0x86cbea & 0xffff) as u16);
}
crate::game::address_dispatch! {
    pub unsafe fn CallMotherBrainTubeFallingFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        8834026 => MotherBrainTubeFallingFunc_GenerateExplosion;
        8834056 => MotherBrainTubeFallingFunc_Falling;
    }
}
pub unsafe fn EprojPreInstr_MotherBrainTubeFalling(k: u16, game: &mut Game) {
    CallMotherBrainTubeFallingFunc(
        (word!(game.eproj_E[(k >> 1)]) as i32 | 0x860000) as u32,
        k,
        game,
    );
}
pub unsafe fn MotherBrainTubeFallingFunc_GenerateExplosion(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    game.eproj_spawn_pt = Point16U {
        x: word!(game.eproj_x_pos[v1]),
        y: (word!(game.eproj_y_pos[v1]) as i32 + 8) as u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, game);
    word!(game.eproj_E[v1] = (0x86cc08 & 0xffff) as u16);
    MotherBrainTubeFallingFunc_Falling(k, game);
}
pub unsafe fn MotherBrainTubeFallingFunc_Falling(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word!(game.eproj_y_vel[v1], |fresh75| (fresh75 as i32 + 6) as u16);
    let mut v2: i16 = MoveEprojWithVelocityY(k, game) as i16;
    if v2 as i32 - 208i32 & 0x8000 == 0 {
        word!(game.eproj_id[v1] = 0);
        game.eproj_spawn_pt = Point16U {
            x: word!(game.eproj_x_pos[v1]),
            y: word!(game.eproj_y_pos[v1]),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0xc, game);
    }
}
static kEprojInit_MotherBrainGlassShatteringShard_X: [i16; 3] = [8, -40i16, -16i16];
static kEprojInit_MotherBrainGlassShatteringShard_Y: [i16; 3] = [32, 32, 32];
unsafe fn EprojInit_MotherBrainGlassShatteringShard(j: u16, game: &mut Game) {
    let v1: u16 = (2 * NextRandom(game) as i32 & 0x1fe) as u16;
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_E[v2] = v1);
    let v3: i32 = v1 as i32 >> 1;
    word!(game.eproj_x_vel[v2] = kSinCosTable8bit_Sext[(v3 + 64) as usize] as u16);
    word!(game.eproj_y_vel[v2] = (4 * kSinCosTable8bit_Sext[v3 as usize] as i32) as u16);
    word!(
        game.eproj_instr_list_ptr[v2] = crate::rtl::rom_table_word!(
            game,
            0x86ce41,
            ((v1 as i32 >> 4 & 0x1e) as u16 >> 1) as isize,
        )
    );
    word!(game.eproj_gfx_idx[v2] = 1600);
    CalculatePlmBlockCoords(word!(game.plm_id), game);
    let v4: i32 = word!(game.cinematic_var8) as i32 >> 1;
    word! { game;
        eproj_x_pos[v2] = (kEprojInit_MotherBrainGlassShatteringShard_X[v4 as usize] as i32
            + 16 * word!(game.plm_x_block) as i32) as u16;
        eproj_y_pos[v2] = (kEprojInit_MotherBrainGlassShatteringShard_Y[v4 as usize] as i32
            + 16 * word!(game.plm_y_block) as i32) as u16;
    }
    word!(
        game.eproj_x_pos[v2],
        |fresh5| (fresh5 as i32 + ((NextRandom(game) as i32 & 0xf) - 8i32)) as u16
    );
    word!(
        game.eproj_y_pos[v2],
        |fresh6| (fresh6 as i32 + ((NextRandom(game) as i32 & 0xf) - 8i32)) as u16
    );
}
unsafe fn EprojInit_MotherBrainGlassShatteringSparkle(j: u16, game: &mut Game) {
    let v1: i32 = word!(game.cinematic_var8) as i32 >> 1;
    let v2: i32 = j as i32 >> 1;
    word! { game;
        eproj_x_pos[v2] =
            (word!(game.eproj_x_pos[v1]) as i32 + (NextRandom(game) as i32 & 0x1f) - 16i32) as u16;
        eproj_y_pos[v2] =
            (word!(game.eproj_y_pos[v1]) as i32 + (NextRandom(game) as i32 & 0x1f) - 16i32) as u16;
        eproj_gfx_idx[v2] = 1600;
    }
}

macro_rules! integrate_shard_velocity_xy {
    (
        $slot:ident, $projectile_index:ident, $game:ident, $x_high_ram:literal,
        $x_low_ram:literal $(,)?
    ) => {
        let $slot: i32 = $projectile_index as i32 >> 1;
        let mut amt: i32 = (word!($game.eproj_x_vel[$slot]) as i16 as i32) << 8;
        AddToHiLo(
            (ram_ptr!($game, u16; $x_high_ram)).offset($slot as isize),
            (ram_ptr!($game, u16; $x_low_ram)).offset($slot as isize),
            amt as u32,
        );
        amt = (word!($game.eproj_y_vel[$slot]) as i16 as i32) << 8;
        AddToHiLo(
            (ram_ptr!($game, u16; 0x1a93)).offset($slot as isize),
            (ram_ptr!($game, u16; 0x1a6f)).offset($slot as isize),
            amt as u32,
        );
    };
}

pub unsafe fn EprojPreInstr_MotherBrainGlassShatteringShard(k: u16, game: &mut Game) {
    integrate_shard_velocity_xy!(v1, k, game, 0x1a4b, 0x1a27);
    let v6: u16 = word!(game.eproj_y_subpos[v1]);
    if (word!(game.eproj_y_pos[v1]) & 0xff00) != 0 {
        word!(game.eproj_id[v1] = 0);
    } else {
        word!(game.eproj_y_vel[v1], |fresh74| (fresh74 as i32 + 32) as u16);
        if (NextRandom(game) & 0x420) == 0 {
            SpawnEprojWithRoomGfx(
                addr_kEproj_MotherBrainGlassShatteringSparkle as u16,
                k,
                game,
            );
        }
    };
}
unsafe fn EprojInit_KiHunterAcidSpitCommon(k: u16, j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_y_vel[v2] = 0);
    word!(game.eproj_y_pos[v2] = ((*gEnemyData(k as i32, game)).y_pos as i32 - 16i32) as u16);
    word! { game, 0; eproj_y_subpos[v2], eproj_x_subpos[v2] }
}
unsafe fn EprojInit_KiHunterAcidSpitLeft(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_vel[v2] = -768i32 as u16);
    word!(
        game.eproj_x_pos[v2] =
            ((*gEnemyData(word!(game.cur_enemy_index) as i32, game)).x_pos as i32 - 22i32) as u16
    );
    EprojInit_KiHunterAcidSpitCommon(word!(game.cur_enemy_index), j, game);
}
unsafe fn EprojInit_KiHunterAcidSpitRight(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_vel[v2] = 768);
    word!(
        game.eproj_x_pos[v2] =
            ((*gEnemyData(word!(game.cur_enemy_index) as i32, game)).x_pos as i32 + 22) as u16
    );
    EprojInit_KiHunterAcidSpitCommon(word!(game.cur_enemy_index), j, game);
}
pub unsafe fn sub_86CFD5(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word!(game.eproj_pre_instr[v1] = (0x86cff8 & 0xffff) as u16);
    word!(game.eproj_x_pos[v1], |fresh73| (fresh73 as i32 - 19i32)
        as u16);
}
pub unsafe fn sub_86CFE6(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word!(game.eproj_pre_instr[v1] = (0x86cff8 & 0xffff) as u16);
    word!(game.eproj_x_pos[v1], |fresh72| (fresh72 as i32 + 19) as u16);
}
pub unsafe fn sub_86CFF8(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    if EprojBlockCollisition_Vertical(k, game) & 1 != 0 {
        word!(game.eproj_instr_list_ptr[v1] = addr_off_86CF56 as u16);
        word!(game.eproj_instr_timers[v1] = 1);
    } else if EprojBlockCollisition_Horiz(k, game) & 1 != 0 {
        word!(game.eproj_x_vel[(k >> 1)] = 0);
    } else {
        let mut v2: u16 = (word!(game.eproj_y_vel[v1]) as i32 + 16) as u16;
        word!(game.eproj_y_vel[v1] = v2);
        if v2 as i32 - 512i32 & 0x8000 == 0 {
            v2 = 512;
        }
        word!(game.eproj_y_vel[v1] = v2);
    };
}
unsafe fn EprojInit_KagosBugs(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_F[v2] = game.cur_enemy_index());
    let enemy_index = game.cur_enemy_index();
    word!(game.eproj_x_pos[v2] = game.enemy_state(enemy_index).x_pos());
    word!(game.eproj_y_pos[v2] = game.enemy_state(enemy_index).y_pos());
    let v4: u16 = ((game.random_number() as i32 & 7) + 1) as u16;
    word!(game.eproj_G[v2] = v4);
    word!(game.eproj_E[v2] = v4.wrapping_add(4));
    word!(game.eproj_pre_instr[v2] = ((0x86d0cai32) as u16));
}
pub unsafe fn EprojPreInstr_KagosBugs_Func1(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = EprojState(&game.g_ram).slot_word(0x1aff, v1 as usize);
    if v2 != 0 {
        let v3: u16 = v2.wrapping_sub(1);
        EprojStateMut(&mut game.g_ram).write(0x1aff, v1 as usize, v3);
        if v3 == 0 {
            QueueSfx2_Max6(0x6c, game);
        }
    }
}
pub unsafe fn EprojPreInstr_KagosBugs_Func2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let enemy_index = word!(game.eproj_F[v1]);
    let v3: u16 =
        abs16((game.enemy_state(enemy_index).x_pos()).wrapping_sub(game.enemy_projectile_x_pos(k)));
    if v3 as i32 - 23i32 & 0x8000 == 0 {
        EprojStateMut(&mut game.g_ram).set_property_bits(v1 as usize, 0x8000);
    }
}
pub unsafe fn EprojPreInstr_KagosBugs(k: u16, game: &mut Game) {
    EprojPreInstr_KagosBugs_Func1(k, game);
    EprojPreInstr_KagosBugs_Func2(k, game);
    let v1: i32 = k as i32 >> 1;
    if word!(game.eproj_G[v1]) != 0 {
        let fresh71 = EprojStateMut(&mut game.g_ram).add_word(0x1bfb, v1 as usize, u16::MAX);
    } else {
        EprojStateMut(&mut game.g_ram).start_instruction(v1 as usize, addr_word_86D052 as u16);
        word!(game.eproj_pre_instr[v1] = ((0x86d0ebi32) as u16));
    };
}
static word_86D082: u16 = 0xe0;
pub unsafe fn EprojPreInstr_D0EC(k: u16, game: &mut Game) {
    EprojPreInstr_KagosBugs_Func1(k, game);
    EprojPreInstr_KagosBugs_Func2(k, game);
    let reset_vertical_speed: bool = if EprojBlockCollisition_Horiz(k, game) != 0 {
        word!(game.eproj_x_vel[(k >> 1)] = 0);
        true
    } else if EprojBlockCollisition_Vertical(k, game) != 0 {
        true
    } else {
        let v2: u16 = word!(game.eproj_y_vel[(k >> 1)]);
        let v3: bool = ((word_86D082) as i16).wrapping_add((v2) as i16) < 0;
        word!(game.eproj_y_vel[(k >> 1)] = (word_86D082).wrapping_add(v2));
        if !v3 {
            false
        } else {
            return;
        }
    };
    if reset_vertical_speed {
        word!(game.eproj_y_vel[(k >> 1)] = 256);
    }
    word!(game.eproj_pre_instr[(k >> 1)] = ((0x86d128i32) as u16));
    EprojStateMut(&mut game.g_ram).start_instruction(usize::from(k >> 1), addr_word_86D04A as u16);
}
pub unsafe fn EprojPreInstr_D128(v0: u16, game: &mut Game) {
    EprojPreInstr_KagosBugs_Func1(v0, game);
    EprojPreInstr_KagosBugs_Func2(v0, game);
    if EprojBlockCollisition_Horiz(v0, game) & 1 != 0 {
        word!(game.eproj_x_vel[(v0 >> 1)] = 0);
    } else if EprojBlockCollisition_Vertical(v0, game) & 1 != 0 {
        let v1: i32 = v0 as i32 >> 1;
        word!(game.eproj_pre_instr[v1] = ((0x86d0ebi32) as u16));
        EprojStateMut(&mut game.g_ram).start_instruction(v1 as usize, addr_word_86D03C as u16);
    } else {
        word!(game.eproj_y_vel[(v0 >> 1)], wrapping_add(word_86D082));
    };
}
static g_word_86D086: u16 = 0x200;
static g_word_86D084: u16 = 0x30;
crate::instruction_callbacks! { pub unsafe fn (k: u16, epjp: *const u8, game: &mut Game) -> *const u8;
    EprojInstr_D15C [epjp] {
        EprojPreInstr_KagosBugs_Func1(k, game);
        EprojPreInstr_KagosBugs_Func2(k, game);
        let v2: i32 = k as i32 >> 1;
        let mut t: u16 = ((game.random_number() as i32 & 0x300) + 2048) as u16;
        word!(game.eproj_y_vel[v2] = -(t as i32) as u16);
        let enemy_index = word!(game.eproj_F[v2]);
        let t2: u16 =
            (game.enemy_state(enemy_index).x_pos()).wrapping_sub(game.enemy_projectile_x_pos(k));
        if ((abs16(t2)) as i16).wrapping_sub((g_word_86D084) as i16) >= 0 {
            t = (if t2 & 0x8000 != 0 { -1i32 } else { 0 }) as u16;
        }
        let mut v4: u16 = g_word_86D086;
        if t & 0x100 != 0 {
            v4 = -(v4 as i32) as u16;
        }
        word!(game.eproj_x_vel[v2] = v4);
        word!(game.eproj_pre_instr[v2] = ((0x86d0eci32) as u16));
    };
    EprojInstr_D1B6 [epjp] {
        let v1: i32 = k as i32 >> 1;
        word!(game.eproj_G[v1] = (game.random_number() & 0x1f).wrapping_add(1));
        word!(game.eproj_pre_instr[v1] = ((0x86d0cai32) as u16));
    };
    EprojInstr_D1C7 [epjp] { word!(game.eproj_gfx_idx[(k >> 1)] = 0); };
    EprojInstr_D1CE [epjp] {
        game.eproj_spawn_pt = Point16U {
            x: word!(game.eproj_x_pos[(k >> 1)]),
            y: word!(game.eproj_y_pos[(k >> 1)]),
        };
        SpawnEnemyDrops(addr_kEnemyDef_E7FF as u16, k, 0, game);
    };
}
unsafe fn EprojInit_MaridiaFloatersSpikes(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word! { game, 0;
        eproj_x_subpos[v1], eproj_y_subpos[v1], eproj_x_vel[v1], eproj_y_vel[v1],
    }
    let v2 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    word!(game.eproj_x_pos[v1] = (*v2).x_pos);
    word!(game.eproj_y_pos[v1] = (*v2).y_pos);
    word!(game.eproj_E[v1] = word!(game.cinematic_var8));
}
static word_86D21A: [i16; 8] = [0, 32, 32, 32, 0, -32i16, -32i16, -32i16];
static word_86D22A: [i16; 8] = [-32i16, -32i16, 0, 32, 32, 32, 0, -32i16];
pub unsafe fn EprojPreInstr_MaridiaFloatersSpikes(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word!(game.eproj_x_vel[v1], |fresh68| (fresh68 as i32
        + word_86D21A[word!(game.eproj_E[v1]) as usize] as i32)
        as u16);
    if EprojBlockCollisition_Horiz(k, game) & 1 != 0 || {
        word!(game.eproj_y_vel[v1], |fresh69| (fresh69 as i32
            + word_86D22A[word!(game.eproj_E[v1]) as usize] as i32)
            as u16);
        EprojBlockCollisition_Vertical(k, game) & 1 != 0
    } {
        word!(game.eproj_instr_list_ptr[v1] = addr_off_86D218 as u16);
        word!(game.eproj_instr_timers[v1] = 1);
    }
}
unsafe fn EprojInit_WreckedShipRobotLaserCommon(k: u16, j: u16, game: &mut Game) {
    let v2 = gEnemyData(k as i32, game);
    let v3: i32 = j as i32 >> 1;
    word!(game.eproj_y_pos[v3] = ((*v2).y_pos as i32 - 16i32) as u16);
    word!(
        game.eproj_x_pos[v3] = ((*v2).x_pos as i32
            + (if word!(game.eproj_x_vel[v3]) & 0x8000 != 0 {
                -4i32
            } else {
                4
            })) as u16
    );
    word! { game, 0; eproj_y_subpos[v3], eproj_x_subpos[v3] }
    if (((*v2).x_width as i32 + (*v2).x_pos as i32 - word!(game.layer1_x_pos) as i32) as i16) >= 0
        && (((*v2).x_pos as i32 - (*v2).x_width as i32 - 257i32 - word!(game.layer1_x_pos) as i32)
            as i16)
            < 0
        && (((*v2).y_height as i32 + (*v2).y_pos as i32 - word!(game.layer1_y_pos) as i32) as i16)
            >= 0
        && (((*v2).y_pos as i32 - (*v2).y_height as i32 - 224i32) as i16) < 0
    {
        QueueSfx2_Max6(0x67, game);
    }
}
unsafe fn EprojInit_WreckedShipRobotLaserDown(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_vel[v2] = (*gEnemyData(word!(game.cur_enemy_index) as i32, game)).ai_var_A);
    word!(game.eproj_y_vel[v2] = 128);
    word!(game.eproj_gfx_idx[v2] = 0);
    EprojInit_WreckedShipRobotLaserCommon(word!(game.cur_enemy_index), j, game);
}
unsafe fn EprojInit_WreckedShipRobotLaserHorizontal(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_vel[v2] = (*gEnemyData(word!(game.cur_enemy_index) as i32, game)).ai_var_A);
    word!(game.eproj_y_vel[v2] = 0);
    EprojInit_WreckedShipRobotLaserCommon(word!(game.cur_enemy_index), j, game);
}
unsafe fn EprojInit_WreckedShipRobotLaserUp(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_vel[v2] = (*gEnemyData(word!(game.cur_enemy_index) as i32, game)).ai_var_A);
    word!(game.eproj_y_vel[v2] = -128i32 as u16);
    EprojInit_WreckedShipRobotLaserCommon(word!(game.cur_enemy_index), j, game);
}
pub unsafe fn EprojPreInstr_WreckedShipRobotLaser(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word!(game.eproj_gfx_idx[v1] = 0);
    if EprojBlockCollisition_Horiz(k, game) & 1 != 0
        || EprojBlockCollisition_Vertical(k, game) & 1 != 0
    {
        word!(game.eproj_id[v1] = 0);
    }
}
pub unsafe fn EprojInstr_AssignNewN00bTubeShardVelocity(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    NextRandom(game);
    let v2: i32 = k as i32 >> 1;
    word!(game.eproj_x_vel[v2] = *((ram_ptr!(game, u16; 0x5e5) as *mut u8).offset(1) as *mut u16));
    word!(game.eproj_y_vel[v2] = 192);
    epjp
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, epjp: *const u8, game: &mut Game) -> *const u8;
    EprojInstr_SetN00bTubeShardX [null::<u8>()] {
        let v2: i32 = k as i32 >> 1;
        if word!(game.nmi_frame_counter_word) & 1 != 0 {
            word!(game.eproj_x_pos[v2] = word!(game.eproj_F[v2]));
            word!(game.eproj_spritemap_ptr[v2] = *(epjp as *mut u16));
        } else {
            word!(
                game.eproj_x_pos[v2] = (128 - word!(game.eproj_F[v2]) as i32 + 128) as u16
            );
            word!(game.eproj_spritemap_ptr[v2] = *(epjp.offset(2) as *mut u16));
        }
        word!(
            game.eproj_instr_list_ptr[v2] =
                (epjp.offset_from(RomBankBase(0x86, game)) as c_long + 4 as c_long) as u16
        );
        word!(game.eproj_instr_timers[v2] = 1);
    };
    EprojInstr_D62A [null::<u8>()] {
        if word!(game.nmi_frame_counter_word) & 1 != 0 {
            word!(game.eproj_x_pos[(k >> 1)] = word!(game.eproj_F[(k >> 1)]));
        } else {
            word!(game.eproj_x_pos[(k >> 1)] = -4608i32 as u16);
        }
        let v2: i32 = k as i32 >> 1;
        word!(game.eproj_spritemap_ptr[v2] = *(epjp as *mut u16));
        word!(
            game.eproj_instr_list_ptr[v2] =
                (epjp.offset_from(RomBankBase(0x86, game)) as c_long + 2 as c_long) as u16
        );
        word!(game.eproj_instr_timers[v2] = 1);
    };
    EprojInstr_SetXvelRandom [epjp] {
        NextRandom(game);
        word!(
            game.eproj_x_vel[(k >> 1)] =
                *((ram_ptr!(game, u16; 0x5e5) as *mut u8).offset(1) as *mut u16)
        );
    };
}
unsafe fn EprojInit_N00bTubeCrack(j: u16, game: &mut Game) {
    CalculatePlmBlockCoords(word!(game.plm_id), game);
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v1] = (16 * word!(game.plm_x_block) as i32 + 96) as u16);
    word!(game.eproj_y_pos[v1] = (16 * word!(game.plm_y_block) as i32 + 48) as u16);
}
static kEprojInit_N00bTubeShards_X: [i16; 10] = [
    -56i16, -64i16, -20i16, -40i16, -64i16, -48i16, -24i16, -40i16, 0, -8i16,
];
static kEprojInit_N00bTubeShards_Y: [i16; 10] =
    [8, -12i16, -26i16, -24i16, -32i16, 28, 16, -8i16, -24i16, 16];
static kEprojInit_N00bTubeShards_Xvel: [i16; 10] = [
    -384i16, -384i16, -160i16, -288i16, -288i16, -320i16, -96i16, -352i16, 0, -64i16,
];
static kEprojInit_N00bTubeShards_Yvel: [i16; 10] = [
    320, -256i16, -416i16, -288i16, -288i16, 448, 576, -96i16, -288i16, 384,
];
unsafe fn EprojInit_N00bTubeShards(j: u16, game: &mut Game) {
    CalculatePlmBlockCoords(word!(game.plm_id), game);
    let v1: i32 = word!(game.cinematic_var8) as i32 >> 1;
    let v2: i32 = j as i32 >> 1;
    word! { game;
        eproj_F[v2] = (kEprojInit_N00bTubeShards_X[v1 as usize] as i32
            + 16 * word!(game.plm_x_block) as i32
            + 96) as u16;
        eproj_E[v2] = 0;
        eproj_y_pos[v2] = (kEprojInit_N00bTubeShards_Y[v1 as usize] as i32
            + 16 * word!(game.plm_y_block) as i32
            + 48) as u16;
        eproj_instr_list_ptr[v2] = crate::rtl::rom_table_word!(game, 0x86d760, v1 as isize);
        eproj_x_vel[v2] = kEprojInit_N00bTubeShards_Xvel[v1 as usize] as u16;
        eproj_y_vel[v2] = kEprojInit_N00bTubeShards_Yvel[v1 as usize] as u16;
    }
}
static kEprojInit_N00bTubeReleasedAirBubbles_X: [u16; 6] = [40, 80, 104, 120, 152, 184];
static kEprojInit_N00bTubeReleasedAirBubbles_Y: [u16; 6] = [80, 72, 84, 32, 64, 84];
unsafe fn EprojInit_N00bTubeReleasedAirBubbles(j: u16, game: &mut Game) {
    CalculatePlmBlockCoords(word!(game.plm_id), game);
    let v1: i32 = word!(game.cinematic_var8) as i32 >> 1;
    let v2: i32 = j as i32 >> 1;
    word! { game;
        eproj_F[v2] = (kEprojInit_N00bTubeReleasedAirBubbles_X[v1 as usize] as i32
            + 16 * word!(game.plm_x_block) as i32) as u16;
        eproj_E[v2] = 0;
        eproj_y_pos[v2] = (kEprojInit_N00bTubeReleasedAirBubbles_Y[v1 as usize] as i32
            + 16 * word!(game.plm_y_block) as i32) as u16;
        eproj_y_vel[v2] = -1280i32 as u16;
    }
}
pub unsafe fn sub_86D7BF(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    if word!(game.eproj_x_pos[v1]) != 0xee00 {
        word!(game.eproj_E[v1] = word!(game.eproj_x_pos[v1]));
    }
    if word!(game.nmi_frame_counter_word) & 1 != 0 {
        word!(game.eproj_x_pos[v1] = -4608i32 as u16);
    } else {
        word!(game.eproj_x_pos[v1] = word!(game.eproj_E[v1]));
    };
}
pub unsafe fn sub_86D7DE(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let amt: i32 = (192i16 as i32) << 8;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a93)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a6f)).offset(v1 as isize),
        amt as u32,
    );
}
pub unsafe fn EprojPreInstr_N00bTubeShards(k: u16, game: &mut Game) {
    integrate_shard_velocity_xy!(v1, k, game, 0x1b23, 0x1aff);
    Eproj_DeleteIfYposOutside(k, game);
}
pub unsafe fn sub_86D83D(k: u16, game: &mut Game) {
    let v2: i32 = (word!(game.eproj_x_vel[(k >> 1)]) as i32 & 0x17e | 0x80) >> 1;
    let v6: i32 = word!(game.cinematic_var7) as i32 >> 1;
    let mut amt: i32 = ((kSinCosTable8bit_Sext[(v2 + 64) as usize] >> 2) as i32) << 8;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1b23)).offset(v6 as isize),
        (ram_ptr!(game, u16; 0x1aff)).offset(v6 as isize),
        amt as u32,
    );
    word!(game.eproj_x_vel[v6], |fresh67| (fresh67 as i32 + 2) as u16);
    amt = (word!(game.eproj_y_vel[v6]) as i16 as i32) << 8;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a93)).offset(v6 as isize),
        (ram_ptr!(game, u16; 0x1a6f)).offset(v6 as isize),
        amt as u32,
    );
    Eproj_DeleteIfYposOutside(word!(game.cinematic_var7), game);
}
pub unsafe fn sub_86D89F(k: u16, game: &mut Game) {
    let v1: u16 = (word!(game.eproj_x_vel[(k >> 1)]) & 0x17e | 0x80);
    let v2: i32 = v1 as i32 >> 1;
    let amt: i32 = ((kSinCosTable8bit_Sext[(v2 + 64) as usize] >> 2) as i32) << 8;
    let v5: i32 = word!(game.cinematic_var7) as i32 >> 1;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1b23)).offset(v5 as isize),
        (ram_ptr!(game, u16; 0x1aff)).offset(v5 as isize),
        amt as u32,
    );
    word!(game.eproj_x_vel[v5], |fresh66| (fresh66 as i32 + 4) as u16);
    sub_86D8DF(word!(game.cinematic_var7), game);
}
pub unsafe fn sub_86D8DF(k: u16, game: &mut Game) {
    let v5: i32 = k as i32 >> 1;
    let amt: i32 = (word!(game.eproj_y_vel[v5]) as i16 as i32) << 8;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a93)).offset(v5 as isize),
        (ram_ptr!(game, u16; 0x1a6f)).offset(v5 as isize),
        amt as u32,
    );
    word!(game.eproj_x_pos[v5] = word!(game.eproj_F[v5]));
}
pub unsafe fn sub_86D992(v0: u16, game: &mut Game) {
    let v1: u16 = EprojState(&game.g_ram).init_param_1();
    let v2: i32 = v0 as i32 >> 1;
    word! { game;
        eproj_E[v2] = EprojState(&game.g_ram).init_param_1();
        eproj_instr_list_ptr[v2] = crate::rtl::rom_table_word!(game, 0x86d96a, (v1 >> 1) as isize);
    }
    let enemy_index = game.cur_enemy_index();
    word!(game.eproj_x_pos[v2] = game.enemy_state(enemy_index).x_pos());
    word!(game.eproj_x_subpos[v2] = game.enemy_state(enemy_index).x_subpos());
    word!(game.eproj_y_pos[v2] = game.enemy_state(enemy_index).y_pos());
    word!(game.eproj_y_subpos[v2] = game.enemy_state(enemy_index).y_subpos());
    word!(game.eproj_y_vel[v2] = -512i32 as u16);
    word!(game.eproj_x_vel[v2] = 512);
    if EprojState(&game.g_ram).init_param_1() as i32 - 12i32 & 0x8000 == 0 {
        word!(game.eproj_y_vel[v2] = -384i32 as u16);
        word!(game.eproj_x_vel[v2] = 384);
    }
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let split_amount =
        ((EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as i16 as i32) << 8) as u32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a93, 0x1a6f, v1 as usize, split_amount);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a93)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a6f)).offset(v1 as isize),
        ((word!(game.eproj_x_vel[v1]) as i16 as i32) << 8) as u32,
    );
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let split_amount =
        ((EprojState(&game.g_ram).slot_word(0x1adb, v1 as usize) as i16 as i32) << 8) as u32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a4b, 0x1a27, v1 as usize, split_amount);
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let split_amount =
        ((EprojState(&game.g_ram).slot_word(0x1ab7, v1 as usize) as i16 as i32) << 8) as u32;
    EprojStateMut(&mut game.g_ram).add_split_words(0x1a4b, 0x1a27, v1 as usize, split_amount);
}
crate::forward_calls! { pub fn (k: u16, game: &mut Game);
    EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1 =>
        EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2 =>
        EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1 =>
        EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2(k: u16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, game);
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1(k: u16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2(k: u16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX1(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, game);
}
pub fn EprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1(k: u16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY1(k, game);
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2(k: u16, game: &mut Game) {
    EprojPreInstr_SpikeShootingPlantSpikes_MoveX2(k, game);
    EprojPreInstr_SpikeShootingPlantSpikes_MoveY2(k, game);
}
static kEprojPreInstr_SpikeShootingPlantSpikes: [Option<Func_Y_V>; 10] = callback_table! { unsafe Func_Y_V;
    EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1,
    EprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1,
    EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2,
    EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1,
    EprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2,
    EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2,
    EprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1,
    EprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1,
    EprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2,
    EprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2,
};
pub fn EprojPreInstrHelper_SpikeShootingPlantSpikes_Func1(k: u16, game: &mut Game) {
    if EprojPreInstrHelper_SpikeShootingPlantSpikes_Func2(k, game) != 0 {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    }
}
pub unsafe fn EprojPreInstr_SpikeShootingPlantSpikes(k: u16, game: &mut Game) {
    kEprojPreInstr_SpikeShootingPlantSpikes[(word!(game.eproj_E[(k >> 1)]) >> 1) as usize]
        .expect("non-null function pointer")(k, game);
    EprojPreInstrHelper_SpikeShootingPlantSpikes_Func1(k, game);
}
pub fn EprojPreInstrHelper_SpikeShootingPlantSpikes_Func2(k: u16, game: &mut Game) -> u16 {
    ((game
        .enemy_projectile_x_pos(k)
        .wrapping_sub(game.layer1_x_pos()) as i16)
        < 0
        || (game
            .layer1_x_pos()
            .wrapping_add(256)
            .wrapping_sub(game.enemy_projectile_x_pos(k)) as i16)
            < 0
        || (game
            .enemy_projectile_y_pos(k)
            .wrapping_sub(game.layer1_y_pos()) as i16)
            < 0
        || (game
            .layer1_y_pos()
            .wrapping_add(256)
            .wrapping_sub(game.enemy_projectile_y_pos(k)) as i16)
            < 0) as u16
}
unsafe fn EprojInit_DBF2(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_instr_list_ptr[v2] = addr_word_86DB0C as u16);
    word!(game.eproj_E[v2] = (0x86db62 & 0xffff) as u16);
    if word!(game.cinematic_var8) != 0 {
        word!(game.eproj_E[v2] = (0x86db8c & 0xffff) as u16);
    }
    let v3 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    word!(game.eproj_x_pos[v2] = (*v3).x_pos);
    word!(game.eproj_x_subpos[v2] = (*v3).x_subpos);
    word!(game.eproj_y_pos[v2] = ((*v3).y_pos as i32 + 2) as u16);
    word!(game.eproj_y_subpos[v2] = (*v3).y_subpos);
    word!(game.eproj_y_vel[v2] = -256i32 as u16);
    word!(game.eproj_x_vel[v2] = 256);
}
pub unsafe fn EprojPreInstr_DBF2_Func1(k: u16, game: &mut Game) {
    if EprojPreInstrHelper_DBF2_Func2(k, game) != 0 {
        word!(game.eproj_id[(k >> 1)] = 0);
    }
}
pub unsafe fn EprojPreInstr_DBF2(k: u16, game: &mut Game) {
    CallEprojPreInstr(
        (word!(game.eproj_E[(k >> 1)]) as i32 | 0x860000) as u32,
        k,
        game,
    );
    EprojPreInstr_DBF2_Func1(k, game);
}
pub unsafe fn EprojPreInstr_DBF2_MoveX1(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a4b)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a27)).offset(v1 as isize),
        ((word!(game.eproj_y_vel[v1]) as i16 as i32) << 8) as u32,
    );
}
pub use self::EprojPreInstr_NorfairLavaquakeRocks_Inner2 as EprojPreInstrHelper_DBF2_Func2;
pub use self::Eproj_NorfairLavaquakeRocks_Func3 as EprojPreInstr_DBF2_MoveX2;
crate::instruction_callbacks! { pub unsafe fn (k: u16, epjp: *const u8, game: &mut Game) -> *const u8;
    EprojInstr_DC5A [epjp] { word!(game.eproj_properties[(k >> 1)] = 12288); };
    EprojInstr_SpawnEnemyDrops_0 [epjp] {
        let v2: i32 = k as i32 >> 1;
        game.eproj_spawn_pt = Point16U {
            x: word!(game.eproj_x_pos[v2]),
            y: word!(game.eproj_y_pos[v2]),
        };
        SpawnEnemyDrops(addr_kEnemyDef_DF7F as u16, k, 0, game);
    };
    EprojInstr_SpawnSporesEproj [epjp] {
        let v2: i32 = k as i32 >> 1;
        game.eproj_spawn_pt = Point16U {
            x: word!(game.eproj_x_pos[v2]),
            y: word!(game.eproj_y_pos[v2]),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_Spores as u16, 0, game);
    };
}
unsafe fn EprojInit_Spores(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    crate::update_place! { reference fresh4 = *(ram_ptr!(game, u16; 0x1a4b)).offset(v2 as isize) => game.eproj_spawn_pt.x; }
    word!(game.eproj_F[v2] = *fresh4);
    word!(game.eproj_y_pos[v2] = game.eproj_spawn_pt.y);
    word!(game.eproj_gfx_idx[v2] = 512);
}
static word_86DCB9: [i16; 5] = [-64i16, -56i16, -48i16, -40i16, -32i16];
unsafe fn EprojInit_SporeSpawnStalk(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(
        game.eproj_y_pos[v1] = ((*(ram_ptr!(game, EnemyData; 0xf78))).y_pos as i32
            + word_86DCB9[word!(game.cinematic_var8) as usize] as i32)
            as u16
    );
    word!(game.eproj_x_pos[v1] = (*(ram_ptr!(game, EnemyData; 0xf78))).x_pos);
}
static kEprojInit_SporeSpawners_X: [i16; 4] = [0x20, 0x60, 0xa0, 0xe0];
unsafe fn EprojInit_SporeSpawners(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word! { game;
        eproj_x_pos[v1] = kEprojInit_SporeSpawners_X[word!(game.cinematic_var8) as usize] as u16;
        eproj_y_pos[v1] = 520;
    }
}
pub unsafe fn EprojPreInstr_Spores(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 = byte!(@pointer (ram_ptr!(game, u16; 0x1aff)).offset(v1 as isize); 0) as u16;
    let mut r18: u16 = SignExtend8(
        byte!(@pointer rom_ptr!(game; 0x86dd6c); *((ram_ptr!(game, u16; 0x1aff)).offset(v1 as isize) as *mut u8) as isize)
            as u16,
    );
    if (word!(game.eproj_F[v1]) & 0x80) != 0 {
        r18 = -(r18 as i32) as u16;
    }
    word!(
        game.eproj_x_pos[v1],
        |fresh65| (fresh65 as i32 + r18 as i32) as u16
    );
    r18 = SignExtend8(byte!(@pointer rom_ptr!(game; 0x86dd6c); (v2 as i32 + 1) as isize) as u16);
    let v3: u16 = (r18 as i32 + word!(game.eproj_y_pos[v1]) as i32 + r18 as i32) as u16;
    word!(game.eproj_y_pos[v1] = v3);
    if v3 as i32 - 768i32 & 0x8000 == 0 {
        word!(game.eproj_id[v1] = 0);
    }
    word!(
        game.eproj_E[v1] = (*((ram_ptr!(game, u16; 0x1aff)).offset(v1 as isize) as *mut u8) as i32
            + 2) as u8 as u16
    );
}
pub unsafe fn EprojPreInstr_SporeSpawners(k: u16, game: &mut Game) {
    if word!(game.kraid_unk9000) == 0 {
        let v1: i32 = k as i32 >> 1;
        if word!(game.eproj_F[v1]) == 0 {
            word!(game.eproj_instr_list_ptr[v1] = addr_word_86DC06 as u16);
            word!(game.eproj_instr_timers[v1] = 1);
            word!(game.eproj_F[v1] = (NextRandom(game) & 0x1ff));
        }
        word!(game.eproj_F[v1], wrapping_sub(1));
    }
}
unsafe fn EprojInit_NamiFuneFireball(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_instr_list_ptr[v2] = addr_word_86DE96 as u16);
    word!(game.eproj_E[v2] = (0x86df40 & 0xffff) as u16);
    if word!(game.cinematic_var8) != 0 {
        word!(game.eproj_instr_list_ptr[v2] = addr_word_86DEA6 as u16);
        word!(game.eproj_E[v2] = (0x86df6a & 0xffff) as u16);
    }
    let v3 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    word!(game.eproj_x_pos[v2] = (*v3).x_pos);
    word!(game.eproj_x_subpos[v2] = (*v3).x_subpos);
    word!(game.eproj_y_pos[v2] = (*v3).y_pos);
    word!(game.eproj_y_subpos[v2] = (*v3).y_subpos);
    if ((*v3).parameter_1 & 0xf) != 0 {
        word!(game.eproj_y_pos[v2], |fresh3| (fresh3 as i32 + 4) as u16);
    }
    let v4: i32 = (4 * byte!((*v3).parameter_2; 0) as i32) as u16 as i32 >> 1;
    word!(game.eproj_y_vel[v2] = crate::rtl::rom_table_word!(game, 0x86deb6, v4 as isize));
    word!(game.eproj_x_vel[v2] = crate::rtl::rom_table_word!(game, 0x86deb6, (v4 + 1) as isize));
}
unsafe fn EprojInit_NamiFuneFireball_MoveX1(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a4b)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a27)).offset(v1 as isize),
        ((word!(game.eproj_y_vel[v1]) as i16 as i32) << 8) as u32,
    );
}
unsafe fn EprojInit_NamiFuneFireball_MoveX2(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a4b)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a27)).offset(v1 as isize),
        ((word!(game.eproj_x_vel[v1]) as i16 as i32) << 8) as u32,
    );
}
crate::game::address_dispatch! {
    pub unsafe fn CallNamiFuneFireballFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        8838976 => EprojInit_NamiFuneFireball_MoveX1; 8839018 => EprojInit_NamiFuneFireball_MoveX2;
    }
}
pub unsafe fn EprojPreInstr_NamiFuneFireball(v0: u16, game: &mut Game) {
    CallNamiFuneFireballFunc(
        (word!(game.eproj_E[(v0 >> 1)]) as i32 | 0x860000) as u32,
        v0,
        game,
    );
    Eproj_NamiFuneFireball_After(v0, game);
}
pub unsafe fn Eproj_NamiFuneFireball_After(v0: u16, game: &mut Game) {
    if EprojPreInstrHelper_DBF2_Func2(v0, game) != 0 {
        word!(game.eproj_id[(v0 >> 1)] = 0);
    }
}
pub unsafe fn EprojInstr_DFEA(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    let v1: i32 = k as i32 >> 1;
    game.eproj_spawn_pt = Point16U {
        x: word!(game.eproj_x_pos[v1]),
        y: word!(game.eproj_y_pos[v1]),
    };
    SpawnEnemyDrops(addr_kEnemyDef_E83F as u16, k, 0, game);
    epjp
}
unsafe fn EprojInit_LavaThrownByLavaman(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_instr_list_ptr[v2] = addr_word_86DFD8 as u16);
    word!(game.eproj_E[v2] = (0x86e050 & 0xffff) as u16);
    if word!(game.cinematic_var8) != 0 {
        word!(game.eproj_instr_list_ptr[v2] = addr_word_86DFDE as u16);
        word!(game.eproj_E[v2] = (0x86e07a & 0xffff) as u16);
    }
    let v3 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    word!(game.eproj_x_pos[v2] = (*v3).x_pos);
    word!(game.eproj_x_subpos[v2] = (*v3).x_subpos);
    word!(game.eproj_y_pos[v2] = ((*v3).y_pos as i32 + 2) as u16);
    word!(game.eproj_y_subpos[v2] = (*v3).y_subpos);
    word!(game.eproj_y_vel[v2] = -768i32 as u16);
    word!(game.eproj_x_vel[v2] = 768);
}
crate::game::address_dispatch! {
    pub unsafe fn CallLavamanFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        8839248 => Eproj_LavaThrownByLavaman_MoveX1; 8839290 => Eproj_LavaThrownByLavaman_MoveX2;
    }
}
pub unsafe fn sub_86E049(v0: u16, game: &mut Game) {
    CallLavamanFunc(
        (word!(game.eproj_E[(v0 >> 1)]) as i32 | 0x860000) as u32,
        v0,
        game,
    );
    sub_86E0A4(v0, game);
}
pub use self::EprojPreInstr_DBF2_MoveX1 as Eproj_LavaThrownByLavaman_MoveX1;
pub use self::Eproj_NorfairLavaquakeRocks_Func3 as Eproj_LavaThrownByLavaman_MoveX2;
pub unsafe fn sub_86E0A4(v0: u16, game: &mut Game) {
    if sub_86E0B0(v0, game) != 0 {
        word!(game.eproj_id[(v0 >> 1)] = 0);
    }
}
pub use self::EprojPreInstr_NorfairLavaquakeRocks_Inner2 as sub_86E0B0;
unsafe fn EprojInit_DustCloudOrExplosion(v0: u16, game: &mut Game) {
    let v1: i32 = v0 as i32 >> 1;
    word!(
        game.eproj_instr_list_ptr[v1] = crate::rtl::rom_table_word!(
            game,
            0x86e42c,
            EprojState(&game.g_ram).init_param_1() as isize
        )
    );
    word!(game.eproj_x_pos[v1] = game.eproj_spawn_pt.x);
    word!(game.eproj_y_pos[v1] = game.eproj_spawn_pt.y);
}
unsafe fn EprojInit_EyeDoorSmoke(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(
        game.eproj_instr_list_ptr[v1] =
            crate::rtl::rom_table_word!(game, 0x86e42c, word!(game.cinematic_var8) as u8 as isize)
    );
    let v2: i32 = 8 * byte!(@pointer ram_ptr!(game, u16; 0x1993); 1) as i32 >> 1;
    let x: u16 = (crate::rtl::rom_table_word!(game, 0x86e47e, (v2 + 2) as isize) as i32
        + (crate::rtl::rom_table_word!(game, 0x86e47e, v2 as isize) as i32
            & word!(game.random_number) as i32)) as u16;
    let y: u16 = (crate::rtl::rom_table_word!(game, 0x86e47e, (v2 + 3) as isize) as i32
        + (crate::rtl::rom_table_word!(game, 0x86e47e, (v2 + 1) as isize) as i32
            & word!(game.random_number) as i32 >> 8)) as u16;
    CalculatePlmBlockCoords(word!(game.plm_id), game);
    word!(game.eproj_x_pos[v1] = (x as i32 + 8 * (2 * word!(game.plm_x_block) as i32 + 1)) as u16);
    word!(game.eproj_y_pos[v1] = (y as i32 + 8 * (2 * word!(game.plm_y_block) as i32 + 1)) as u16);
    NextRandom(game);
}
pub fn EprojPreInstr_DustCloudOrExplosion(k: u16, game: &mut Game) {
    if CheckIfEprojIsOffScreen(k, game) != 0 {
        EprojStateMut(&mut game.g_ram).set_id(usize::from(k >> 1), 0);
    }
}
pub unsafe fn EprojInstr_SetYVel(k: u16, epjp: *const u8, game: &mut Game) -> *const u8 {
    word!(game.eproj_y_vel[(k >> 1)] = *(epjp as *mut u16));
    epjp.offset(2)
}
unsafe fn EprojInit_SpawnedGate_Common(j: u16, v1: u16, game: &mut Game) {
    let v2: u16 = word!(game.plm_id);
    CalculatePlmBlockCoords(word!(game.plm_id), game);
    let v3: i32 = j as i32 >> 1;
    word!(game.eproj_E[v3] = word!(game.plm_block_indices[(v2 >> 1)]));
    word!(game.eproj_x_pos[v3] = (16u16).wrapping_mul(word!(game.plm_x_block)));
    word!(game.eproj_y_pos[v3] = (v1 as i32 + 16 * word!(game.plm_y_block) as i32) as u16);
}
crate::forward_calls! { unsafe fn (j: u16, game: &mut Game);
    EprojInit_SpawnedShotGate => EprojInit_SpawnedGate_Common(j, 0, game);
    EprojInit_ClosedDownwardsShotGate => EprojInit_SpawnedGate_Common(j, 0x40, game);
    EprojInit_ClosedUpwardsShotGate => EprojInit_SpawnedGate_Common(j, -0x40i32 as u16, game);
}
pub unsafe fn EprojPreInstr_E605(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let mut v2: u16 =
        (word!(game.eproj_timers[v1])).wrapping_add(abs16(word!(game.eproj_y_vel[v1])));
    if v2 >= 0x1000 {
        word!(game.eproj_instr_timers[v1] = 1);
        let fresh60 = EprojStateMut(&mut game.g_ram).add_word(0x1b47, v1 as usize, 1);
        let fresh61 = EprojStateMut(&mut game.g_ram).add_word(0x1b47, v1 as usize, 1);
        v2 = 0;
    }
    word!(game.eproj_timers[v1] = v2);
    word!(game.eproj_y_subpos[v1], |fresh62| (fresh62 as i32
        + ((*((ram_ptr!(game, u16; 0x1adb)).offset(v1 as isize) as *mut u8) as i32) << 8))
        as u16);
    word!(
        game.eproj_y_pos[v1],
        wrapping_add(
            (*((ram_ptr!(game, u16; 0x1adb)).offset(v1 as isize) as *mut u8).offset(1,) as i8)
                as u16
        )
    );
}
unsafe fn EprojInit_SaveStationElectricity(j: u16, game: &mut Game) {
    CalculatePlmBlockCoords(word!(game.plm_id), game);
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v1] = (16 * (word!(game.plm_x_block) as i32 + 1)) as u16);
    word!(game.eproj_y_pos[v1] = (16 * (word!(game.plm_y_block) as i32 - 2i32)) as u16);
}
fn CheckIfEprojIsOffScreen(k: u16, game: &mut Game) -> u16 {
    if (game
        .enemy_projectile_x_pos(k)
        .wrapping_sub(game.layer1_x_pos()) as i16)
        >= 0
        && ((game.enemy_projectile_x_pos(k) as i32 - (game.layer1_x_pos() as i32 + 256)) as i16) < 0
        && (game
            .enemy_projectile_y_pos(k)
            .wrapping_sub(game.layer1_y_pos()) as i16)
            >= 0
        && ((game.enemy_projectile_y_pos(k) as i32 - (game.layer1_y_pos() as i32 + 256)) as i16) < 0
    {
        return 0;
    }
    1
}
pub unsafe fn Eproj_FuncE722(k: u16, game: &mut Game) -> u16 {
    let v1: i32 = k as i32 >> 1;
    let mut v2: i16 = word!(game.eproj_x_pos[v1]) as i16;
    let result: u16 = 1;
    if v2 >= 0 && v2 as i32 - 512i32 & 0x8000 != 0 {
        let mut v3: i16 = word!(game.eproj_y_pos[v1]) as i16;
        if v3 >= 0 && v3 as i32 - 512i32 & 0x8000 != 0 {
            return 0;
        }
    }
    result
}
pub unsafe fn Eproj_FuncE73E_MoveXY(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    if (word!(game.g_word_7E97DC[v1]) as i32 + 64 & 0x80) != 0 {
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a4b)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1a27)).offset(v1 as isize),
            -(words!(i32, (word!(game.eproj_x_vel[v1]), word!(game.eproj_E[v1])))) as u32,
        );
    } else {
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a4b)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1a27)).offset(v1 as isize),
            words!((word!(game.eproj_x_vel[v1]), word!(game.eproj_E[v1]))),
        );
    }
    if (word!(game.g_word_7E97DC[v1]) as i32 + 128 & 0x80) != 0 {
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a93)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1a6f)).offset(v1 as isize),
            -(words!(i32, (word!(game.eproj_y_vel[v1]), word!(game.eproj_F[v1])))) as u32,
        );
    } else {
        AddToHiLo(
            (ram_ptr!(game, u16; 0x1a93)).offset(v1 as isize),
            (ram_ptr!(game, u16; 0x1a6f)).offset(v1 as isize),
            words!((word!(game.eproj_y_vel[v1]), word!(game.eproj_F[v1]))),
        );
    };
}
pub unsafe fn Eproj_AngleToSamus(j: u16, r18: u16, r20: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v1] = r18);
    word!(game.eproj_y_pos[v1] = r20);
    let v2: u16 = (64
        - CalculateAngleFromXY(
            (word!(game.samus_x_pos) as i32 - word!(game.eproj_x_pos[v1]) as i32) as u16,
            (word!(game.samus_y_pos) as i32 - word!(game.eproj_y_pos[v1]) as i32) as u16,
            game,
        ) as i32) as u8 as u16;
    word!(game.g_word_7E97DC[v1] = v2);
    let pt: Point32 = ConvertAngleToXy(v2, word!(game.cinematic_var8), game);
    word!(game.eproj_x_vel[v1] = (pt.x >> 16) as u16);
    word!(game.eproj_E[v1] = pt.x as u16);
    word!(game.eproj_y_vel[v1] = (pt.y >> 16) as u16);
    word!(game.eproj_F[v1] = pt.y as u16);
}
unsafe fn EprojInit_BotwoonsBody(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v1] = (*(ram_ptr!(game, EnemyData; 0xf78))).x_pos);
    word!(game.eproj_y_pos[v1] = (*(ram_ptr!(game, EnemyData; 0xf78))).y_pos);
    word!(game.eproj_y_vel[v1] = 0);
    let v2: u16 = (if (*(ram_ptr!(game, EnemyData; 0xf78))).ai_var_A != 0 {
        16
    } else {
        48
    }) as u16;
    let v3: u16 = crate::rtl::rom_table_word!(game, 0x86e9f1, (v2 >> 1) as isize);
    word!(game.eproj_instr_list_ptr[v1] = v3);
    word!(game.eproj_F[v1] = v3);
    word!(game.eproj_E[v1] = v2);
    word!(game.eproj_x_vel[v1] = (0x86ea98 & 0xffff) as u16);
    let Ex = gExtraEnemyRam7800((*(ram_ptr!(game, EnemyData; 0xf78))).ai_var_A as i32, game);
    (*Ex).kraid.kraid_next = j;
    word!(game.eproj_flags[v1] = 2);
    (*Ex).kraid.kraid_healths_4ths[0i32 as usize] = 1;
}
crate::game::address_dispatch! {
    pub unsafe fn CallBotwoonEprojFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        8841880 => Eproj_BotwoonsBody_Main; 8841972 => Eproj_BotwonsBodyFunction_Dying;
        8841988 => Eproj_BotwonsBodyFunction_Dying2;
        8842015 => Eproj_BotwonsBodyFunction_DyingFalling; 8842131 => [()];
    }
}
pub unsafe fn EprojPreInstr_BotwoonsBody(k: u16, game: &mut Game) {
    if *((&raw mut (*(ram_ptr!(game, ExtraEnemyRam8000; 0x8000))).pad as *mut u8).offset(32)
        as *mut u16)
        != 0
    {
        let v1: i32 = k as i32 >> 1;
        if word!(game.eproj_x_vel[v1]) as i32 == 0x86ea98 & 0xffff {
            word!(game.eproj_x_vel[v1] = (0x86eaf4 & 0xffff) as u16);
        }
    }
    CallBotwoonEprojFunc(
        (word!(game.eproj_x_vel[(k >> 1)]) as i32 | 0x860000) as u32,
        k,
        game,
    );
}
pub unsafe fn Eproj_BotwoonsBody_Main(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let v2: u16 =
        crate::rtl::rom_table_word!(game, 0x86e9f1, (word!(game.eproj_E[v1]) >> 1) as isize);
    if v2 != word!(game.eproj_F[v1]) {
        word!(game.eproj_instr_list_ptr[v1] = v2);
        word!(game.eproj_F[v1] = v2);
        word!(game.eproj_instr_timers[v1] = 1);
    }
    Eproj_BotwoonsBodyHurtFlashHandling1(k, game);
}
pub unsafe fn Eproj_BotwoonsBodyHurtFlashHandling1(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    word!(game.eproj_gfx_idx[v1], |fresh58| (fresh58 | 0xe00));
    if (*(ram_ptr!(game, EnemyData; 0xf78))).flash_timer != 0
        && (word!(game.random_enemy_counter) & 2) != 0
    {
        word!(game.eproj_gfx_idx[v1], |fresh59| (fresh59 & 0xf1ff));
    }
}
pub unsafe fn Eproj_BotwoonsBodyHurtFlashHandling2(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1;
    word!(game.eproj_gfx_idx[v1], |fresh55| (fresh55 | 0xe00));
    if (*(ram_ptr!(game, EnemyData; 0xf78))).flash_timer != 0
        && (word!(game.random_enemy_counter) & 2) != 0
    {
        word!(game.eproj_gfx_idx[v1], |fresh56| (fresh56 & 0xf1ff));
    }
}
pub unsafe fn Eproj_BotwonsBodyFunction_Dying(v0: u16, game: &mut Game) {
    let v1: i32 = v0 as i32 >> 1;
    word!(game.eproj_E[v1] = (4 * v0 as i32 + 96) as u16);
    word!(game.eproj_x_vel[v1] = 0xeb04);
    Eproj_BotwonsBodyFunction_Dying2(v0, game);
}
pub unsafe fn Eproj_BotwonsBodyFunction_Dying2(v0: u16, game: &mut Game) {
    let v1: i32 = v0 as i32 >> 1;
    if word!(game.eproj_E[v1], wrapping_add(1), reread) as i32 - 256i32 & 0x8000 == 0 {
        word!(game.eproj_x_vel[v1] = (0x86eb1f & 0xffff) as u16);
    }
    word!(game.eproj_instr_timers[v1] = 0);
    Eproj_BotwoonsBodyHurtFlashHandling1(v0, game);
}
pub unsafe fn Eproj_BotwonsBodyFunction_DyingFalling(v0: u16, game: &mut Game) {
    let v1: i32 = v0 as i32 >> 1;
    let v3: i32 = 8 * ((word!(game.eproj_y_vel[v1]) as i32 & 0xff00) >> 8) >> 1;
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a93)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a6f)).offset(v1 as isize),
        *(rom_ptr!(game, u32; 0xa0cbc7)).offset((v3 >> 1) as isize),
    );
    if word!(game.eproj_y_pos[v1]) as i32 - 200i32 & 0x8000 != 0 {
        let v7: i32 = v0 as i32 >> 1;
        word!(game.eproj_y_vel[v7], |fresh54| (fresh54 as i32 + 192)
            as u16);
        word!(game.eproj_instr_timers[v7] = 0);
        Eproj_BotwoonsBodyHurtFlashHandling2(v0, game);
    } else {
        word!(game.eproj_y_pos[v1] = 200);
        word!(game.eproj_x_vel[v1] = (0x86eb93 & 0xffff) as u16);
        word!(game.eproj_instr_list_ptr[v1] = addr_word_86E208 as u16);
        word!(game.eproj_gfx_idx[v1] = 2560);
        word!(game.eproj_instr_timers[v1] = 1);
        QueueSmallExplosionSfx(game);
        if v0 == 10 {
            *((&raw mut (*(ram_ptr!(game, ExtraEnemyRam8800; 0x8800))).pad as *mut u8).offset(62)
                as *mut u16) = 1;
        }
    };
}
pub unsafe fn QueueSmallExplosionSfx(game: &mut Game) {
    QueueSfx2_Max6(0x24, game);
}
unsafe fn EprojInit_BotwoonsSpit(j: u16, game: &mut Game) {
    let v1 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v2] = (*v1).x_pos);
    word!(game.eproj_y_pos[v2] = (*v1).y_pos);
    word!(game.eproj_instr_list_ptr[v2] = addr_word_86EBAE as u16);
    word!(game.g_word_7E97DC[v2] = *ram_ptr!(game; 0x60b) as u16);
    let pt: Point32 = ConvertAngleToXy(
        *ram_ptr!(game; 0x60b) as u16,
        word!(game.cinematic_var8),
        game,
    );
    word!(game.eproj_x_vel[v2] = (pt.x >> 16) as u16);
    word!(game.eproj_E[v2] = pt.x as u16);
    word!(game.eproj_y_vel[v2] = (pt.y >> 16) as u16);
    word!(game.eproj_F[v2] = pt.y as u16);
}
pub unsafe fn EprojPreInstr_BotwoonsSpit(k: u16, game: &mut Game) {
    Eproj_FuncE73E_MoveXY(k, game);
    sub_86EC0C(k, game);
}
pub unsafe fn sub_86EC0C(k: u16, game: &mut Game) {
    if sub_86EC18(k, game) != 0 {
        word!(game.eproj_id[(k >> 1)] = 0);
    }
}
pub use self::EprojPreInstr_NorfairLavaquakeRocks_Inner2 as sub_86EC18;
unsafe fn EprojInit_YappingMawsBody(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v3: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v3] = game.enemy_state(enemy_index).x_pos());
    word!(game.eproj_y_pos[v3] = game.enemy_state(enemy_index).y_pos());
    word!(game.eproj_instr_list_ptr[v3] = addr_word_86EC5C as u16);
    if game.enemy_state(enemy_index).parameter_2() == 0 {
        word!(game.eproj_instr_list_ptr[v3] = addr_word_86EC56 as u16);
    }
    *((&raw mut (*(ram_ptr!(game, ExtraEnemyRam8800; 0x8800))).pad as *mut u8).offset(
        (game.cur_enemy_index() as i32
            + 2 * *((&raw mut (*(ram_ptr!(game, ExtraEnemyRam8800; 0x8800))).pad as *mut u8)
                .offset((game.cur_enemy_index() as i32 + 8) as isize)
                as *mut u16) as i32) as u16 as isize,
    ) as *mut u16) = j;
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, epjp: *const u8, game: &mut Game) -> *const u8;
    EprojInstr_ECE3 [epjp.offset(2)] {
        let x: u16 = (game.enemy_projectile_x_pos(k) as i32 + (NextRandom(game) as i32 & 0x3f)
            - 32i32) as u16;
        let y: u16 = ((game.enemy_projectile_y_pos(k) as i32
            + (((game.random_number()) & (0x3f00u16)) as i32 >> 8)) as u16)
            .wrapping_sub(32);
        CreateSpriteAtPos(x, y, *(epjp as *mut u16), 0, game);
    };
    EprojInstr_ED17 [epjp.offset(2)] {
        let x: u16 = (game.enemy_projectile_x_pos(k) as i32 + (NextRandom(game) as i32 & 0x1f)
            - 16i32) as u16;
        let y: u16 = ((game.enemy_projectile_y_pos(k) as i32
            + (((game.random_number()) & (0x1f00u16)) as i32 >> 8)) as u16)
            .wrapping_sub(16);
        CreateSpriteAtPos(x, y, *(epjp as *mut u16), 0, game);
    };
    EprojInstr_QueueSfx2_9 [epjp] { QueueSfx2_Max1(9, game); };
    EprojInstr_QueueSfx2_24 [epjp] { QueueSfx2_Max1(0x24, game); };
    EprojInstr_QueueSfx2_B [epjp] { QueueSfx2_Max1(0xb, game); };
    EprojInstr_EEAF [] {
        let v2: u16 = RandomDropRoutine(k, game);
        if k != 0 && v2 as i32 - 6i32 & 0x8000 != 0 {
            let v3: u16 = (2u16).wrapping_mul(v2);
            let v4: i32 = k as i32 >> 1;
            word!(game.eproj_E[v4] = v3);
            word!(
                game.eproj_instr_list_ptr[v4] =
                    crate::rtl::rom_table_word!(game, 0x86ef04, (v3 >> 1) as isize)
            );
            word!(game.eproj_instr_timers[v4] = 1);
            word!(game.eproj_F[v4] = 400);
            word!(game.eproj_pre_instr[v4] = ((0x86efe0i32) as u16));
            word!(game.eproj_properties[v4], |fresh25| (fresh25)
                & ((!(0x4000i32)) as u16));
            word!(game.eproj_instr_list_ptr[v4]) as usize as *const u8
        } else {
            let v6: i32 = k as i32 >> 1;
            word!(game.eproj_instr_timers[v6] = 1);
            word!(game.eproj_properties[v6] = 12288);
            word!(game.eproj_pre_instr[v6] = ((0x86efdfi32) as u16));
            word!(game.eproj_instr_list_ptr[v6] = 0xeca3);
            0xeca3i32 as usize as *const u8
        }
    };
}
pub unsafe fn EprojInstr_HandleRespawningEnemy(
    k: u16,
    epjp: *const u8,
    game: &mut Game,
) -> *const u8 {
    if (word!(game.eproj_killed_enemy_index[(k >> 1)]) as i16) <= -2i16 {
        RespawnEnemy(
            (word!(game.eproj_killed_enemy_index[(k >> 1)])) & (0x7fffu16),
            game,
        );
    }
    epjp
}
unsafe fn EprojInit_F337(j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v2] = game.eproj_spawn_pt.x);
    word!(game.eproj_y_pos[v2] = game.eproj_spawn_pt.y);
    word!(game.eproj_gfx_idx[v2] = 0);
    word!(game.eproj_enemy_header_ptr[v2] = game.eproj_spawn_varE24);
    let v3: u16 = RandomDropRoutine(j, game);
    if v3 != 0 && v3 as i32 - 6i32 & 0x8000 != 0 {
        let v4: u16 = (2u16).wrapping_mul(v3);
        word!(game.eproj_E[v2] = v4);
        word!(
            game.eproj_instr_list_ptr[v2] =
                crate::rtl::rom_table_word!(game, 0x86ef04, (v4 >> 1) as isize)
        );
        word!(game.eproj_instr_timers[v2] = 1);
        word!(game.eproj_F[v2] = 400);
        word!(game.eproj_killed_enemy_index[(j >> 1)] = -1i32 as u16);
    } else {
        let v5: i32 = j as i32 >> 1;
        word!(game.eproj_instr_list_ptr[v5] = addr_word_86ECA3 as u16);
        word!(game.eproj_instr_timers[v5] = 1);
        word!(game.eproj_properties[v5] = 12288);
        word!(game.eproj_pre_instr[v5] = (0x86efdf & 0xffff) as u16);
    };
}
unsafe fn EprojInit_EnemyDeathExplosion(j: u16, game: &mut Game) {
    let enemy_index = game.cur_enemy_index();
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_x_pos[v2] = game.enemy_state(enemy_index).x_pos());
    word!(game.eproj_y_pos[v2] = game.enemy_state(enemy_index).y_pos());
    word!(game.eproj_killed_enemy_index[v2] = game.cur_enemy_index());
    if (game.enemy_state(enemy_index).properties() & 0x4000) != 0 {
        word!(game.eproj_killed_enemy_index[v2] = (game.cur_enemy_index()) | (0x8000u16));
    }
    word!(game.eproj_enemy_header_ptr[v2] = game.enemy_state(enemy_index).enemy_ptr());
    word!(game.eproj_gfx_idx[v2] = 0);
    {
        let instruction: u16 = crate::rtl::rom_table_word!(
            game,
            0x86efd5,
            EprojState(&game.g_ram).init_param_1() as isize
        );
        EprojStateMut(&mut game.g_ram).start_instruction(v2 as usize, instruction);
    }
}
static off_86F0AD: [Option<Func_V>; 7] = callback_table! { unsafe Func_V;
    None, Eproj_Pickup_SmallHealth, Eproj_Pickup_BigHealth, Eproj_Pickup_PowerBombs,
    Eproj_Pickup_Missiles, Eproj_Pickup_SuperMissiles, None,
};
pub unsafe fn EprojPreInstr_Pickup(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    let fresh53 = EprojStateMut(&mut game.g_ram).add_word(0x1b23, v1 as usize, u16::MAX);
    if (fresh53 != 0) {
        let skip_pickup_collision: bool = (CallSomeSamusCode(0xd, game) != 0)
            && ((word!(game.eproj_F[v1]) as i32 - 384i32 & 0x8000 != 0)
                && ({
                    let mut v2: u16 = abs16(
                        (word!(game.eproj_x_pos[v1]) as i32
                            - word!(game.grapple_beam_end_x_pos) as i32)
                            as u16,
                    );
                    (v2 as i32 - 16i32 & 0x8000 != 0)
                        && ({
                            let mut v3: u16 = abs16(
                                (word!(game.eproj_y_pos[v1]) as i32
                                    - word!(game.grapple_beam_end_y_pos) as i32)
                                    as u16,
                            );
                            (v3 as i32 - 16i32 & 0x8000 != 0)
                                && ({
                                    off_86F0AD[(word!(game.eproj_E[v1]) >> 1) as usize]
                                        .expect("non-null function pointer")(
                                        game
                                    );
                                    true
                                })
                        })
                }));
        if skip_pickup_collision {
        } else {
            let v5: i32 = k as i32 >> 1;
            let varE20: u16 = EprojState(&game.g_ram).x_radius(v5 as usize) as u16;
            let varE22: u16 = EprojState(&game.g_ram).y_radius(v5 as usize) as u16;
            let v6: u16 = abs16(
                game.samus_x_pos()
                    .wrapping_sub(game.enemy_projectile_x_pos(k)),
            );
            let mut v7: bool = v6 < game.samus_x_radius();
            let v8: u16 = v6.wrapping_sub(game.samus_x_radius());
            if v7 || v8 < varE20 {
                let v9: u16 = abs16(
                    game.samus_y_pos()
                        .wrapping_sub(game.enemy_projectile_y_pos(k)),
                );
                v7 = v9 < game.samus_y_radius();
                let v10: u16 = v9.wrapping_sub(game.samus_y_radius());
                if v7 || v10 < varE22 {
                    off_86F0AD[(word!(game.eproj_E[v5]) >> 1) as usize]
                        .expect("non-null function pointer")(game);
                    let v11: i32 = k as i32 >> 1;
                    EprojStateMut(&mut game.g_ram).finish_pickup(v11 as usize);
                }
            }
            return;
        }
    }
    let v4: i32 = k as i32 >> 1;
    EprojStateMut(&mut game.g_ram).finish_pickup(v4 as usize);
}
pub unsafe fn Eproj_Pickup_SmallHealth(game: &mut Game) {
    Samus_RestoreHealth(5, game);
    QueueSfx2_Max1(1, game);
}
pub unsafe fn Eproj_Pickup_BigHealth(game: &mut Game) {
    Samus_RestoreHealth(0x14, game);
    QueueSfx2_Max1(2, game);
}
pub unsafe fn Eproj_Pickup_PowerBombs(game: &mut Game) {
    Samus_RestorePowerBombs(1, game);
    QueueSfx2_Max1(5, game);
}
pub unsafe fn Eproj_Pickup_Missiles(game: &mut Game) {
    Samus_RestoreMissiles(2, game);
    QueueSfx2_Max1(3, game);
}
pub unsafe fn Eproj_Pickup_SuperMissiles(game: &mut Game) {
    Samus_RestoreSuperMissiles(1, game);
    QueueSfx2_Max1(4, game);
}
static byte_86F25E: [u8; 6] = [1, 2, 4, 6, 5, 3];
pub unsafe fn RandomDropRoutine(k: u16, game: &mut Game) -> u16 {
    let mut i: i32 = 0;
    let mut v9: i8 = 0;
    let mut r18: u8 = 0;
    let mut r22: u8 = 0;
    let v1: i32 = k as i32 >> 1;
    let varE28: u16 = word!(game.eproj_enemy_header_ptr[v1]);
    if (varE28 != 0) {
        let mut v2: u16 = (*get_EnemyDef_A2(varE28, game)).item_drop_chances_ptr;
        if (v2 != 0) {
            let mut Random: u8 = 0;
            loop {
                Random = NextRandom(game) as u8;
                if (Random != 0) {
                    break;
                }
            }
            let mut r20: u8 = 255;
            let mut r24: u16 = 0;
            let mut v5: u16 = 1;
            let update_health_bias: bool = !(game
                .samus_reserve_health()
                .wrapping_add(game.samus_health())
                >= 0x1e)
                || (!(game
                    .samus_reserve_health()
                    .wrapping_add(game.samus_health())
                    < 0x32)
                    && ({
                        v5 = 0;
                        true
                    }));
            if update_health_bias {
                word!(game.health_drop_bias_flag = v5);
            }
            let mut v7 = RomPtr_B4(v2, game);
            if word!(game.health_drop_bias_flag) as u8 != 0 {
                r18 = (*v7.offset(1)).wrapping_add(*v7);
                r22 = 3;
            } else {
                r18 = *v7.offset(3);
                r22 = 8;
                if game.samus_health() != game.samus_max_health()
                    || game.samus_reserve_health() != game.samus_max_reserve_health()
                {
                    r18 = ((*v7.offset(1) as i32 + *v7 as i32) as u8).wrapping_add(r18);
                    r22 = (r22) | (3u8);
                }
                if game.samus_missiles() != game.samus_max_missiles() {
                    r18 = (*v7.offset(2)).wrapping_add(r18);
                    r22 = (r22) | (4u8);
                }
                if game.samus_super_missiles() != game.samus_max_super_missiles() {
                    r20 = (r20).wrapping_sub(*v7.offset(4));
                    r22 = (r22) | (0x10u8);
                }
                if game.samus_power_bombs() != game.samus_max_power_bombs() {
                    r20 = (r20 as i32 - *v7.offset(5) as i32) as u8;
                    r22 = (r22 | 0x20);
                }
            }
            i = 0;
            i = 0;
            while i != 4 {
                if r18 == 0 {
                    r22 = (r22 >> 4);
                    v2 = v2.wrapping_add(4);
                    i = 4;
                    break;
                } else {
                    v9 = ((r22) as i8) & (1i8);
                    r22 = (r22 >> 1);
                    if v9 != 0 {
                        let v10 = RomPtr_B4(v2, game);
                        let RegWord: u16 = Mult8x8(r20, *v10);
                        let divved: i32 = SnesDivide(RegWord, r18) as i32;
                        let v12: u16 = r24;
                        let v13: u16 = (divved + v12 as i32) as u16;
                        if v13 >= ((Random) as u16) {
                            return byte_86F25E[i as usize] as u16;
                        }
                        r24 = v13;
                    }
                    v2 = v2.wrapping_add(1);
                    i += 1;
                }
            }
            loop {
                v9 = ((r22) as i8) & (1i8);
                r22 = (r22 >> 1);
                if v9 != 0 {
                    let v14 = RomPtr_B4(v2, game);
                    if (r24).wrapping_add((*v14) as u16) >= ((Random) as u16) {
                        return byte_86F25E[i as usize] as u16;
                    }
                    r24 = (r24 as i32 + *v14 as i32) as u16;
                }
                v2 = v2.wrapping_add(1);
                i += 1;
                if (i == 6) {
                    break;
                }
            }
        }
    }
    i = 3;
    byte_86F25E[i as usize] as u16
}
pub unsafe fn RespawnEnemy(v0: u16, game: &mut Game) {
    word!(game.cur_enemy_index = v0);
    let v1 = RomPtr_A1(
        (word!(game.room_enemy_population_ptr)).wrapping_add((v0 >> 2)),
        game,
    );
    let E = gEnemyData(v0 as i32, game);
    (*E).enemy_ptr = *(v1 as *mut u16);
    (*E).x_pos = *(v1.offset(2) as *mut u16);
    (*E).y_pos = *(v1.offset(4) as *mut u16);
    (*E).current_instruction = *(v1.offset(6) as *mut u16);
    (*E).properties = *(v1.offset(8) as *mut u16);
    (*E).extra_properties = *(v1.offset(10) as *mut u16);
    (*E).parameter_1 = *(v1.offset(12) as *mut u16);
    (*E).parameter_2 = *(v1.offset(14) as *mut u16);
    let ESD = gEnemySpawnData(v0 as i32, game);
    (*E).palette_index = (*ESD).palette_index;
    (*E).vram_tiles_index = (*ESD).vram_tiles_index;
    set_fields! {[(*E)], 0; frozen_timer, flash_timer, invincibility_timer, timer, frame_counter, ai_var_A, ai_var_B, ai_var_C, ai_var_D, ai_var_E, ai_preinstr}
    (*E).instruction_timer = 1;
    let Edef = get_EnemyDef_A2((*E).enemy_ptr, game);
    (*E).x_width = (*Edef).x_radius;
    (*E).y_height = (*Edef).y_radius;
    (*E).health = (*Edef).health;
    (*E).layer = (*Edef).layer as u16;
    *(&raw mut (*E).bank as *mut u16) = *(&raw mut (*Edef).bank as *mut u16);
    CallEnemyAi((words!(i32, (*Edef), bank, ai_init)) as u32, game);
}
unsafe fn EprojInit_Sparks(j: u16, game: &mut Game) {
    static word_86F3D4: [u16; 14] = [
        0xffff, 0xb800, 0xffff, 0xc000, 0xffff, 0xe000, 0xffff, 0xff00, 0, 0x100, 0, 0x2000, 0,
        0x4000,
    ];
    let v2: i32 = j as i32 >> 1;
    word!(game.eproj_instr_list_ptr[v2] = addr_kEnemyDef_F353 as u16);
    let v3 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    word!(game.eproj_x_pos[v2] = (*v3).x_pos);
    word!(game.eproj_x_subpos[v2] = (*v3).x_subpos);
    word!(game.eproj_y_pos[v2] = ((*v3).y_pos as i32 + 8) as u16);
    word!(game.eproj_y_subpos[v2] = (*v3).y_subpos);
    word! { game, 0; eproj_x_vel[v2], eproj_y_vel[v2] }
    let v4: u16 = ((NextRandom(game) as i32 & 0x1c) >> 1) as u16;
    word!(game.eproj_F[v2] = crate::rtl::rom_table_word!(game, 0x86f3d4, (v4 as i32 + 0) as isize));
    word!(game.eproj_E[v2] = crate::rtl::rom_table_word!(game, 0x86f3d4, (v4 as i32 + 1) as isize));
}
pub unsafe fn EprojPreInstr_Sparks(k: u16, game: &mut Game) {
    let v1: i32 = k as i32 >> 1;
    if (word!(game.eproj_y_vel[v1]) & 0x8000) == 0 {
        if EprojBlockCollisition_Vertical(k, game) & 1 != 0 {
            word! { game;
                eproj_instr_list_ptr[v1] = 0xf363;
                eproj_instr_timers[v1] = 1;
                eproj_F[v1] = (word!(game.eproj_F[v1]) as i32 * 2
                    + (word!(game.eproj_E[v1]) as i32 >> 15)) as u16;
            }
            word!(game.eproj_E[v1], |fresh50| (fresh50 as i32 * 2) as u16);
            word!(
                game.eproj_F[v1] = (word!(game.eproj_F[v1]) as i32 * 2
                    + (word!(game.eproj_E[v1]) as i32 >> 15))
                    as u16
            );
            word!(game.eproj_E[v1], |fresh51| (fresh51 as i32 * 2) as u16);
            word!(game.eproj_x_vel[v1] = 0x8000);
            word!(game.eproj_y_vel[v1] = -1i32 as u16);
            word!(game.eproj_y_pos[v1], |fresh52| (fresh52 as i32 - 2i32)
                as u16);
            return;
        }
        let v2: u16 = word!(game.eproj_x_vel[v1]);
        word!(game.eproj_x_vel[v1] = (v2 as i32 + 0x4000) as u16);
        let v3: u16 = (u16::overflowing_add(v2, 0x4000u16).1 as i32
            + word!(game.eproj_y_vel[v1]) as i32) as u16;
        if v3 < 4 {
            word!(game.eproj_y_vel[v1] = v3);
        }
    }
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a93)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a6f)).offset(v1 as isize),
        words!((word!(game.eproj_y_vel[v1]), word!(game.eproj_x_vel[v1]))),
    );
    AddToHiLo(
        (ram_ptr!(game, u16; 0x1a4b)).offset(v1 as isize),
        (ram_ptr!(game, u16; 0x1a27)).offset(v1 as isize),
        words!((word!(game.eproj_F[v1]), word!(game.eproj_E[v1]))),
    );
    if (*ram_ptr!(game; 0x5b5) & 3) == 0 {
        CreateSpriteAtPos(
            word!(game.eproj_x_pos[v1]),
            word!(game.eproj_y_pos[v1]),
            48,
            word!(game.eproj_gfx_idx[v1]),
            game,
        );
    }
}
crate::game::address_dispatch! {
    unsafe fn CallEprojInit(ea: u32, j: u16, game: &mut Game) => ea;
        call (j, game); default { Unreachable(game); };
    {
        8817209 => Eproj_Init_0x8aaf;
        8817357 => [{
                    init_skree_particle(j, 6, 320, -769, game);
                }];
        8817393 => [{
                    init_skree_particle(j, 6, 96, -1025, game);
                }];
        8817429 => [{
                    init_skree_particle(j, -6, -320, -769, game);
                }];
        8817465 => [{
                    init_skree_particle(j, -6, -96, -1025, game);
                }];
        8817924 => EprojInit_DraygonsGunk; 8817984 => EprojInit_DraygonsWallTurretProjs;
        8818723 => EprojInit_CrocomireProjectile; 8818895 => EprojInit_CrocomireSpikeWallPieces;
        8819334 => EprojInit_CrocomireBridgeCrumbling; 8819533 => EprojInit_9634;
        8819658 => EprojInit_9642_RidleysFireball; 8819839 => EprojInit_9660_FireballExplosion;
        8819872 => EprojInit_9688; 8819892 => EprojInit_9696; 8819912 => EprojInit_966C;
        8819932 => EprojInit_967A; 8820444 => EprojInit_9734_CeresFallingDebris;
        8820772 => EprojInit_PhantoonDestroyableFireballs;
        8821050 => EprojInit_PhantoonStartingFireballs; 8821923 => EprojInit_RocksKraidSpits;
        8821976 => EprojInit_RocksFallingKraidCeiling; 8822028 => EprojInit_RocksWhenKraidRises;
        8822252 => EprojInit_MiniKraidSpit; 8822342 => EprojInit_MiniKraidSpikesLeft;
        8822347 => EprojInit_MiniKraidSpikesRight; 8822450 => EprojInit_WalkingLavaSeahorseFireball;
        8822793 => EprojInit_PirateMotherBrainLaser; 8822936 => EprojInit_PirateClaw;
        8823457 => EprojInit_A379; 8823534 => EprojInit_CeresElevatorPad;
        8823579 => EprojInit_CeresElevatorPlatform; 8823715 => EprojPreInstr_PrePhantomRoom;
        8824275 => EprojInit_BombTorizoLowHealthDrool;
        8824413 => EprojInit_BombTorizoLowHealthInitialDrool; 8824519 => EprojInit_A977;
        8824566 => EprojInit_BombTorizoExplosiveSwipe;
        8824676 => EprojInit_BombTorizoStatueBreaking;
        8824859 => EprojInit_BombTorizoLowHealthExplode;
        8824945 => EprojInit_BombTorizoDeathExplosion; 8825405 => EprojInit_AB07;
        8825835 => EprojInit_BombTorizosChozoOrbs; 8825980 => EprojInit_GoldenTorizosChozoOrbs;
        8826389 => EprojInit_TorizoSonicBoom; 8826620 => EprojInit_WreckedShipChozoSpikeFootsteps;
        8826691 => EprojInit_TourianStatueDustClouds;
        8826704 => EprojInit_TourianLandingDustCloudsRightFoot;
        8826829 => EprojInit_TorizoLandingDustCloudLeftFoot; 8826881 => EprojInit_GoldenTorizoEgg;
        8827342 => EprojInit_GoldenTorizoSuperMissile; 8827688 => EprojInit_GoldenTorizoEyeBeam;
        8828061 => EprojInit_TourianEscapeShaftFakeWallExplode;
        8828143 => EprojInit_LavaSeahorseFireball; 8828461 => EprojInit_EyeDoorProjectile;
        8828547 => EprojInit_EyeDoorSweat;
        8829050 => EprojInit_TourianStatueUnlockingParticleWaterSplash;
        8829070 => EprojInit_TourianStatueEyeGlow;
        8829109 => EprojInit_TourianStatueUnlockingParticle;
        8829160 => EprojIni_TourianStatueUnlockingParticleTail;
        8829176 => EprojInit_TourianStatueSoul; 8829246 => EprojInit_TourianStatueBaseDecoration;
        8829265 => EprojInit_TourianStatueRidley; 8829284 => EprojInit_TourianStatuePhantoon;
        8829744 => sub_86BB30; 8829842 => EprojInit_NuclearWaffleBody;
        8829915 => EprojInit_NorfairLavaquakeRocks;
        8830364 => EprojInit_ShaktoolAttackMiddleBackCircle; 8830370 => EprojInit_BDA2;
        8830543 => EprojInit_MotherBrainRoomTurrets;
        8830809 => EprojInit_MotherBrainRoomTurretBullets;
        8831731 => Eproj_MotherBrainsBlueRingLasers; 8832130 => EprojInit_MotherBrainBomb;
        8832517 => sub_86C605; 8832644 => EprojInit_MotherBrainDeathBeemFired;
        8833034 => EprojInit_MotherBrainRainbowBeam; 8833091 => EprojInit_MotherBrainsDrool;
        8833269 => EprojInit_MotherBrainsDeathExplosion;
        8833327 => EprojInit_MotherBrainsRainbowBeamExplosion;
        8833377 => EprojInit_MotherBrainEscapeDoorParticles;
        8833642 => EprojInit_MotherBrainPurpleBreathBig;
        8833667 => EprojInit_MotherBrainPurpleBreathSmall;
        8833782 => EprojInit_TimeBombSetJapaneseText; 8833993 => EprojInit_MotherBrainTubeFalling;
        8834501 => EprojInit_MotherBrainGlassShatteringShard;
        8834669 => EprojInit_MotherBrainGlassShatteringSparkle;
        8834960 => EprojInit_KiHunterAcidSpitLeft; 8834982 => EprojInit_KiHunterAcidSpitRight;
        8835208 => EprojInit_KagosBugs; 8835642 => EprojInit_MaridiaFloatersSpikes;
        8835852 => EprojInit_WreckedShipRobotLaserDown;
        8835886 => EprojInit_WreckedShipRobotLaserHorizontal;
        8835905 => EprojInit_WreckedShipRobotLaserUp; 8836773 => EprojInit_N00bTubeCrack;
        8836809 => EprojInit_N00bTubeShards; 8836980 => EprojInit_N00bTubeReleasedAirBubbles;
        8837522 => sub_86D992; 8837912 => EprojInit_DBF2; 8838285 => EprojInit_Spores;
        8838307 => EprojInit_SporeSpawnStalk; 8838356 => EprojInit_SporeSpawners;
        8838870 => EprojInit_NamiFuneFireball; 8839168 => EprojInit_LavaThrownByLavaman;
        8840296 => EprojInit_DustCloudOrExplosion; 8840358 => EprojInit_EyeDoorSmoke;
        8840656 => EprojInit_SpawnedShotGate; 8840661 => EprojInit_ClosedDownwardsShotGate;
        8840666 => EprojInit_ClosedUpwardsShotGate; 8840877 => EprojInit_SaveStationElectricity;
        8841777 => EprojInit_BotwoonsBody; 8842182 => EprojInit_BotwoonsSpit;
        8842338 => EprojInit_YappingMawsBody; 8843049 => EprojInit_F337;
        8843145 => EprojInit_EnemyDeathExplosion; 8844177 => EprojInit_Sparks;
    }
    unsafe fn CallEprojPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        8825112 => [()]; 8814960 => [()]; 8815867 => [()]; 8817277 => Eproj_PreInit_0x8aaf;
        8817501 => Eproj_PreInstr_SkreeParticle; 8818004 => [()]; 8818122 => EprojPreInstr_8DCA;
        8818175 => EprojPreInstr_DraygonsTurret_8DFF; 8818191 => EprojPreInstr_DraygonsGunk_8E0F;
        8818795 => EprojPreInstr_CrocomireProjectile; 8818867 => sub_8690B3;
        8818965 => EprojPreInstr_CrocomireSpikeWallPieces;
        8819386 => EprojPreInstr_CrocomireBridgeCrumbling; 8819602 => EprojPreInstr_9634;
        8819726 => EprojPreInstr_9642_RidleysFireball; 8819980 => [()];
        8819981 => EprojPreInstr_966C; 8820002 => EprojPreInstr_9688; 8820023 => EprojPreInstr_96A4;
        8820032 => EprojPreInstr_96C0; 8820041 => EprojPreInstr_96CE;
        8820481 => EprojPreInstr_9734_CeresFallingDebris;
        8821121 => EprojPreInstr_PhantoonStartingFireballs;
        8821183 => EprojPreInstr_PhantoonStartingFireballs2;
        8821249 => EprojPreInstr_PhantoonStartingFireballs3; 8821316 => [()];
        8821317 => EprojPreInstr_PhantoonDestroyableFireballs;
        8821396 => EprojPreInstr_PhantoonDestroyableFireballs_2;
        8821466 => EprojPreInstr_PhantoonDestroyableFireballs_3;
        8821545 => EprojPreInstr_PhantoonStartingFireballsB;
        8821569 => EprojPreInstr_PhantoonStartingFireballsB_2; 8822102 => EprojPreInstr_KraidRocks;
        8822153 => EprojPreInstr_RocksFallingKraidCeiling; 8822181 => sub_869DA5;
        8822302 => EprojPreInit_MiniKraidSpit; 8822403 => EprojPreInstr_MiniKraidSpikes;
        8822527 => EprojPreInstr_WalkingLavaSeahorseFireball; 8822875 => [()];
        8822876 => EprojPreInstr_PirateMotherBrainLaser_MoveLeft;
        8822906 => EprojPreInstr_PirateMotherBrainLaser_MoveRight;
        8822993 => EprojPreInstr_PirateClawThrownLeft;
        8823076 => EprojPreInstr_PirateClawThrownRight; 8823591 => [()];
        8823592 => EprojPreInstr_CeresElevatorPad; 8823652 => EprojPreInstr_CeresElevatorPlatform;
        8823715 => EprojPreInstr_PrePhantomRoom; 8824967 => sub_86A887;
        8825071 => EprojPreInstr_A977; 8825113 => [()]; 8825484 => EprojPreInstr_AB07;
        8826029 => EprojPreInstr_BombTorizosChozoOrbs;
        8826106 => EprojPreInstr_GoldenTorizosChozoOrbs; 8826476 => EprojPreInstr_TorizoSonicBoom;
        8826947 => EprojPreInstr_GoldenTorizoEgg; 8827065 => sub_86B0B9; 8827101 => sub_86B0DD;
        8827405 => EprojPreInstr_GoldenTorizoSuperMissile; 8827447 => EprojPreInstr_B237;
        8827786 => EprojPreInstr_GoldenTorizoEyeBeam; 8828080 => [()]; 8828213 => sub_86B535;
        8828601 => EprojPreInstr_EyeDoorProjectile; 8828692 => EprojPreInstr_EyeDoorSweat;
        8829303 => EprojPreInstr_TourianStatueUnlockingParticleWaterSplash;
        8829314 => EprojPreInstr_TourianStatueUnlockingParticle;
        8829437 => EprojPreInstr_TourianStatueSoul; 8829495 => EprojPreInstr_TourianStatueStuff;
        8829506 => EprojPreInstr_BA42; 8829775 => [()]; 8829894 => [()];
        8829967 => EprojPreInstr_NorfairLavaquakeRocks;
        8830238 => EprojPreInstr_NorfairLavaquakeRocks_Inner;
        8830250 => [{
                    EprojPreInstr_NorfairLavaquakeRocks_Inner2(k, game);
                }];
        8830467 => EprojInit_ShaktoolAttackFrontCircle; 8830482 => EprojPreInstr_BE12;
        8830943 => EprojPreInstr_MotherBrainRoomTurrets;
        8831200 => EprojPreInstr_MotherBrainRoomTurretBullets;
        8831797 => Eproj_MoveToBlueRingSpawnPosition; 8832200 => EprojPreInstr_MotherBrainBomb;
        8833044 => EprojPreInstr_MotherBrainRainbowBeam; 8833101 => EprojPreInstr_C84D;
        8833158 => EprojInit_MotherBrainsDrool_Falling;
        8833300 => EprojPreInit_MotherBrainsDeathExplosion_0;
        8833356 => EprojPreInstr_MotherBrainsRainbowBeamExplosion;
        8833490 => EprojPreInstr_MotherBrainsExplodedDoorParticles; 8833699 => [()];
        8833786 => EprojPreInstr_TimeBombSetJapaneseText;
        8834023 => EprojPreInstr_MotherBrainTubeFalling;
        8834715 => EprojPreInstr_MotherBrainGlassShatteringShard; 8835029 => sub_86CFD5;
        8835046 => sub_86CFE6; 8835063 => [()]; 8835064 => sub_86CFF8;
        8835251 => EprojPreInstr_KagosBugs_Func1; 8835274 => EprojPreInstr_KagosBugs;
        8835307 => [()]; 8835308 => EprojPreInstr_D0EC; 8835368 => EprojPreInstr_D128;
        8835556 => EprojPreInstr_KagosBugs_Func2; 8835683 => EprojPreInstr_MaridiaFloatersSpikes;
        8836031 => EprojPreInstr_WreckedShipRobotLaser; 8837055 => sub_86D7BF;
        8837086 => sub_86D7DE; 8837117 => EprojPreInstr_N00bTubeShards; 8837181 => sub_86D83D;
        8837279 => sub_86D89F; 8837343 => sub_86D8DF;
        8837595 => EprojPreInstr_SpikeShootingPlantSpikes;
        8837606 => EprojPreInstr_SpikeShootingPlantSpikes_MoveY1;
        8837648 => EprojPreInstr_SpikeShootingPlantSpikes_MoveY2;
        8837690 => EprojPreInstr_SpikeShootingPlantSpikes_MoveX1;
        8837732 => EprojPreInstr_SpikeShootingPlantSpikes_MoveX2;
        8837774 => EprojPreInstr_SpikeShootingPlantSpikes_0_MoveX1;
        8837779 => EprojPreInstr_SpikeShootingPlantSpikes_2_MoveX2;
        8837784 => EprojPreInstr_SpikeShootingPlantSpikes_1_MoveY1;
        8837789 => EprojPreInstr_SpikeShootingPlantSpikes_4_MoveY2;
        8837794 => EprojPreInstr_SpikeShootingPlantSpikes_6_MoveX1Y1;
        8837802 => EprojPreInstr_SpikeShootingPlantSpikes_8_MoveX1Y2;
        8837810 => EprojPreInstr_SpikeShootingPlantSpikes_7_MoveX2Y1;
        8837818 => EprojPreInstr_SpikeShootingPlantSpikes_9_MoveX2Y2; 8837979 => EprojPreInstr_DBF2;
        8837986 => EprojPreInstr_DBF2_MoveX1; 8838028 => EprojPreInstr_DBF2_MoveX2;
        8838070 => EprojPreInstr_DBF2_Func1; 8838382 => EprojPreInstr_Spores; 8838468 => [()];
        8838470 => EprojPreInstr_SporeSpawners; 8838969 => EprojPreInstr_NamiFuneFireball;
        8839241 => sub_86E049; 8840446 => EprojPreInstr_DustCloudOrExplosion; 8840456 => [()];
        8840708 => [()]; 8840709 => EprojPreInstr_E605; 8840913 => [()];
        8841856 => EprojPreInstr_BotwoonsBody; 8842245 => EprojPreInstr_BotwoonsSpit;
        8842388 => [()]; 8843231 => [()]; 8843232 => EprojPreInstr_Pickup;
        8844272 => EprojPreInstr_Sparks; 8832877 => [()];
    }
}
crate::game::address_dispatch! {
    unsafe fn CallEprojInstr(ea: u32, k: u16, j: *const u8, game: &mut Game) -> *const u8 => ea;
        call (k, j, game); default {
            Unreachable(game);
            null::<u8>()
        };
    {
        8814932 => EprojInstr_Delete; 8814937 => EprojInstr_Sleep;
        8814945 => EprojInstr_SetPreInstr_; 8814954 => EprojInstr_ClearPreInstr;
        8814961 => EprojInstr_CallFunc; 8815019 => [EprojInstr_Goto(k, j)];
        8815024 => [EprojInstr_GotoRel(k, j)]; 8815046 => EprojInstr_DecTimerAndGotoIfNonZero;
        8815054 => EprojInstr_DecTimerAndGotoRelIfNonZero; 8815061 => EprojInstr_SetTimer;
        8815071 => EprojInstr_MoveRandomlyWithinRadius;
        8815152 => EprojInstr_SetProjectileProperties;
        8815164 => EprojInstr_ClearProjectileProperties;
        8815176 => EprojInstr_EnableCollisionWithSamusProj;
        8815186 => EprojInstr_DisableCollisionWithSamusProj;
        8815196 => EprojInstr_DisableCollisionWithSamus;
        8815206 => EprojInstr_EnableCollisionWithSamus; 8815216 => EprojInstr_SetToNotDieOnContact;
        8815226 => EprojInstr_SetToDieOnContact; 8815236 => EprojInstr_SetLowPriority;
        8815246 => EprojInstr_SetHighPriority; 8815256 => EprojInstr_SetXyRadius;
        8815265 => EprojInstr_SetXyRadiusZero; 8815269 => EprojInstr_CalculateDirectionTowardsSamus;
        8815317 => EprojInstr_WriteColorsToPalette; 8815357 => EprojInstr_QueueMusic;
        8815369 => EprojInstr_QueueSfx1_Max6; 8815378 => EprojInstr_QueueSfx2_Max6;
        8815387 => EprojInstr_QueueSfx3_Max6; 8815396 => EprojInstr_QueueSfx1_Max15;
        8815405 => EprojInstr_QueueSfx2_Max15; 8815414 => EprojInstr_QueueSfx3_Max15;
        8815423 => EprojInstr_QueueSfx1_Max3; 8815432 => EprojInstr_QueueSfx2_Max3;
        8815441 => EprojInstr_QueueSfx3_Max3; 8815450 => EprojInstr_QueueSfx1_Max9;
        8815459 => EprojInstr_QueueSfx2_Max9; 8815468 => EprojInstr_QueueSfx3_Max9;
        8815477 => EprojInstr_QueueSfx1_Max1; 8815486 => EprojInstr_QueueSfx2_Max1;
        8815495 => EprojInstr_QueueSfx3_Max1;
        8817768 => EprojInstr_SpawnEnemyDropsWithDraygonsEyeDrops; 8818073 => EprojInstr_868D99;
        8819829 => EprojInstr_DisableCollisionsWithSamus; 8820154 => EprojInstr_95BA;
        8820205 => EprojInstr_95ED; 8820256 => EprojInstr_9620; 8820750 => EprojInstr_980E;
        8822864 => EprojInstr_SetPreInstrAndRun; 8823894 => EprojInstr_GotoWithProbability25;
        8825738 => EprojInstr_SpawnEnemyDrops; 8826258 => EprojInstr_GotoDependingOnXDirection;
        8826678 => EprojInstr_ResetXYpos1; 8826770 => EprojInstr_MoveY_Minus4;
        8827497 => EprojInstr_SetVelTowardsSamus1; 8827506 => EprojInstr_SetVelTowardsSamus2;
        8827832 => EprojInstr_GotoIfFunc1; 8827958 => EprojInstr_ResetXYpos2;
        8828906 => EprojInstr_SpawnTourianStatueUnlockingParticle; 8828917 => EprojInstr_Earthquake;
        8828952 => EprojInstr_SpawnTourianStatueUnlockingParticleTail;
        8828993 => EprojInstr_AddToYpos; 8831347 => EprojInstr_SwitchJump;
        8831412 => EprojInstr_UserPalette0; 8833232 => EprojInstr_Add12ToY;
        8833774 => EprojInstr_MotherBrainPurpleBreathIsActive; 8835420 => EprojInstr_D15C;
        8835510 => EprojInstr_D1B6; 8835527 => EprojInstr_D1C7; 8835534 => EprojInstr_D1CE;
        8836577 => EprojInstr_AssignNewN00bTubeShardVelocity;
        8836594 => EprojInstr_SetN00bTubeShardX; 8836650 => EprojInstr_D62A;
        8836762 => EprojInstr_SetXvelRandom; 8838234 => EprojInstr_DC5A;
        8838241 => EprojInstr_SpawnEnemyDrops_0; 8838263 => EprojInstr_SpawnSporesEproj;
        8839146 => EprojInstr_DFEA; 8840499 => EprojInstr_SetYVel; 8842467 => EprojInstr_ECE3;
        8842519 => EprojInstr_ED17; 8842891 => EprojInstr_QueueSfx2_9;
        8842903 => EprojInstr_QueueSfx2_24; 8842915 => EprojInstr_QueueSfx2_B;
        8842927 => EprojInstr_EEAF; 8843024 => EprojInstr_HandleRespawningEnemy;
        8817910 => EprojInstr_SetPreInstrA; 8817917 => EprojInstr_SetPreInstrB;
        8822906 => [{
                    EprojPreInstr_PirateMotherBrainLaser_MoveRight(k, game);
                    j
                }];
        8822876 => [{
                    EprojPreInstr_PirateMotherBrainLaser_MoveLeft(k, game);
                    j
                }];
        8823742 => EprojInstr_A3BE; 8819312 => EprojInstr_9270; 8815070 => [j];
        8827198 => sub_86B13E;
    }
}
