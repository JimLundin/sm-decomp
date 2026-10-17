extern "C" {
    static mut g_ram: [uint8; 131072];
    static mut g_rom: *const uint8;
    fn Unreachable() -> bool;
    fn RomPtr(addr: uint32_t) -> *const uint8;
    fn InvalidInterrupt_Crash();
    fn QueueSfx2_Max6(a: uint16);
    fn DrawProjectileSpritemap(k: uint16, x_r20: uint16, y_r18: uint16);
    fn CalcExplosion_Mode7(k: uint16) -> Point16U;
    fn ClearProjectile(k: uint16);
    fn HandleProjectileTrails();
    fn Samus_DrawShinesparkCrashEchoProjectiles();
}
pub use crate::types::{
    __int16_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, uint16, uint16_t, uint32,
    uint32_t, uint8, uint8_t, Point16U, VoidP,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ProjectileDamagesAndInstrPtr {
    pub damages: ::core::ffi::c_uint,
    pub instr_ptr: VoidP,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ProjectileDataTable {
    pub damage: uint16,
    pub instr_ptrs: [uint16; 10],
}
pub use crate::types::{ProjectileInstr, ProjectileType};
pub const kProjectileType_DontInteractWithSamus: ProjectileType = 32768;
pub const kProjectileType_TypeMask: ProjectileType = 3840;
pub const kProjectileType_MissileExplosion: ProjectileType = 2048;
pub const kProjectileType_BeamExplosion: ProjectileType = 1792;
pub const kProjectileType_Bomb: ProjectileType = 1280;
pub const kProjectileType_PowerBomb: ProjectileType = 768;
pub const kProjectileType_SuperMissile: ProjectileType = 512;
pub const kProjectileType_Missile: ProjectileType = 256;
#[inline]
unsafe extern "C" fn RomPtr_93(mut addr: uint16_t) -> *const uint8 {
    return RomPtr((0x930000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t);
}
#[inline]
unsafe extern "C" fn get_ProjectileDataTable(mut a: uint16) -> *mut ProjectileDataTable {
    return RomPtr((0x930000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t)
        as *mut ProjectileDataTable;
}
#[inline]
unsafe extern "C" fn get_ProjectileInstr(mut a: uint16) -> *mut ProjectileInstr {
    return RomPtr((0x930000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t)
        as *mut ProjectileInstr;
}
pub const fnProj93Instr_Delete: uint32 = 9667119 as uint32;
pub const fnProj93Instr_Goto: uint32 = 9667129 as uint32;
#[no_mangle]
pub unsafe extern "C" fn InitializeProjectile(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut r18: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0xc04 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int;
    let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let mut v3: uint16 = 0;
    if v2 as ::core::ffi::c_int & 0xf00 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        v3 = *(g_rom.offset(
            (((0x9383f1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x9383f1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*(&raw mut v2 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as isize,
            );
    } else if v2 as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
        v3 = *(g_rom.offset(
            (((0x9383d9 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x9383d9 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as isize,
            );
    } else {
        v3 = *(g_rom.offset(
            (((0x9383c1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x9383c1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as isize,
            );
    }
    let mut PD: *mut ProjectileDataTable = get_ProjectileDataTable(v3);
    if (*PD).damage as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        InvalidInterrupt_Crash();
    }
    *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = (*PD).damage;
    let mut v7: uint16 = (*PD).instr_ptrs[r18 as usize];
    *((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = v7;
    let mut v8: *const uint8 = RomPtr_93(v7 as uint16_t);
    *((&raw mut g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = *v8.offset(4 as ::core::ffi::c_int as isize) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = *v8.offset(5 as ::core::ffi::c_int as isize) as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc54 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 1 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn InitializeInstrForSuperMissile(mut v0: uint16) {
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v3: *const uint8 = RomPtr_93(
        *(g_rom.offset(
            (((0x93842b as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x93842b as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*(((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as isize,
            ) as uint16_t,
    );
    let mut v4: uint16 = *(v3 as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = v4;
    if v4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        InvalidInterrupt_Crash();
    }
    *((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = *(v3.offset(2 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xc54 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 1 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn InitializeInstrForMissile(mut v0: uint16) {
    let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v3: *const uint8 = RomPtr_93(
        *(g_rom.offset(
            (((0x9383f1 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x9383f1 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*(((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16 as *mut uint8)
                    .offset(
                        (::core::mem::size_of::<uint16>() as usize)
                            .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                            .wrapping_sub(1 as usize) as isize,
                    ) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as isize,
            ) as uint16_t,
    );
    let mut v4: uint16 = *(v3 as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = *(v3 as *mut uint16);
    if v4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        InvalidInterrupt_Crash();
    }
    *((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = *(v3.offset(2 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xc54 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 1 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn KillProjectileInner(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf00 as ::core::ffi::c_int
        != 0 as ::core::ffi::c_int
    {
        if *((&raw mut g_ram as *mut uint8).offset(0x1f51 as ::core::ffi::c_int as isize)
            as *mut uint16)
            == 0
        {
            QueueSfx2_Max6(7 as uint16);
        }
        let mut v2: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (v2 as ::core::ffi::c_int & 0xf0ff as ::core::ffi::c_int
            | 0x800 as ::core::ffi::c_int) as uint16;
        if v2 as ::core::ffi::c_int & 0x200 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            *((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = *(&raw mut (*(g_rom.offset(
                (((0x938691 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x938691 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8
                as *mut ProjectileDamagesAndInstrPtr))
                .damages as *mut uint16)
                .offset(
                    (::core::mem::size_of::<::core::ffi::c_uint>() as usize)
                        .wrapping_div(::core::mem::size_of::<uint16>() as usize)
                        .wrapping_sub(1 as usize) as isize,
                );
            *((&raw mut g_ram as *mut uint8).offset(0x183e as ::core::ffi::c_int as isize)
                as *mut uint16) = 20 as uint16;
            *((&raw mut g_ram as *mut uint8).offset(0x1840 as ::core::ffi::c_int as isize)
                as *mut uint16) = 30 as uint16;
        } else {
            *((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) = *(&raw mut (*(g_rom.offset(
                (((0x938679 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x938679 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8
                as *mut ProjectileDamagesAndInstrPtr))
                .instr_ptr)
                .offset(1 as ::core::ffi::c_int as isize)
                as uint16;
        }
        if *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            - 21 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            *((&raw mut g_ram as *mut uint8).offset(0xccc as ::core::ffi::c_int as isize)
                as *mut uint16) = 20 as uint16;
        }
    } else {
        *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (*((&raw mut g_ram as *mut uint8)
            .offset(0xc18 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) as ::core::ffi::c_int
            & 0xf0ff as ::core::ffi::c_int
            | 0x700 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = *(&raw mut (*(g_rom.offset(
            (((0x938679 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x938679 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8
            as *mut ProjectileDamagesAndInstrPtr))
            .damages as *mut uint16)
            .offset(
                (::core::mem::size_of::<::core::ffi::c_uint>() as usize)
                    .wrapping_div(::core::mem::size_of::<uint16>() as usize)
                    .wrapping_sub(1 as usize) as isize,
            );
        QueueSfx2_Max6(0xc as uint16);
    }
    *((&raw mut g_ram as *mut uint8).offset(0xc54 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 1 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 8 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn InitializeBombExplosion(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = *(&raw mut (*(g_rom.offset(
        (((0x938681 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0x938681 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8
        as *mut ProjectileDamagesAndInstrPtr))
        .damages as *mut uint16)
        .offset(
            (::core::mem::size_of::<::core::ffi::c_uint>() as usize)
                .wrapping_div(::core::mem::size_of::<uint16>() as usize)
                .wrapping_sub(1 as usize) as isize,
        );
    *((&raw mut g_ram as *mut uint8).offset(0xc54 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 1 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn InitializeShinesparkEchoOrSpazerSba(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut r18: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0xc04 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int;
    let mut PD: *mut ProjectileDataTable = get_ProjectileDataTable(
        *(g_rom.offset(
            (((0x938403 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x938403 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*(((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as *mut uint16 as *mut uint8)
                    .offset(0 as ::core::ffi::c_int as isize)
                    as ::core::ffi::c_int
                    - 34 as ::core::ffi::c_int) as isize,
            ),
    );
    *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = (*PD).damage;
    if (*PD).damage as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        InvalidInterrupt_Crash();
    }
    *((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = (*PD).instr_ptrs[r18 as usize];
    *((&raw mut g_ram as *mut uint8).offset(0xc54 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 1 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn InitializeSbaProjectile(mut k: uint16) {
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: *const uint8 = RomPtr_93(
        *(g_rom.offset(
            (((0x938413 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x938413 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(
                (*((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as isize,
            ) as uint16_t,
    );
    let mut v3: uint16 = *(v2 as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xc2c as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = v3;
    if v3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        InvalidInterrupt_Crash();
    }
    *((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = *(v2.offset(2 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut g_ram as *mut uint8).offset(0xc54 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize) = 1 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn ProjectileInsts_GetValue(mut k: uint16) -> uint16 {
    let mut ip: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0xc40 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int;
    let mut delta: ::core::ffi::c_int = if *((&raw mut g_ram as *mut uint8)
        .offset(0xc54 as ::core::ffi::c_int as isize)
        as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
        && (*get_ProjectileInstr(ip as uint16)).timer as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        0 as ::core::ffi::c_int
    } else {
        -(8 as ::core::ffi::c_int)
    };
    return (*get_ProjectileInstr((ip + delta) as uint16)).field_6;
}
#[no_mangle]
pub unsafe extern "C" fn CallProj93Instr(mut ea: uint32, mut j: uint16, mut k: uint16) -> uint16 {
    match ea {
        9667119 => return Proj93Instr_Delete(j, k),
        9667129 => return Proj93Instr_Goto(j, k),
        _ => return Unreachable() as uint16,
    };
}
#[no_mangle]
pub unsafe extern "C" fn RunProjectileInstructions() {
    let mut PI: *mut ProjectileInstr = ::core::ptr::null_mut::<ProjectileInstr>();
    let mut v0: uint16 = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16);
    let mut v1: ::core::ffi::c_int = *((&raw mut g_ram as *mut uint8)
        .offset(0xdde as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let ref mut fresh0 = *((&raw mut g_ram as *mut uint8)
        .offset(0xc54 as ::core::ffi::c_int as isize) as *mut uint16)
        .offset(v1 as isize);
    let fresh1 = *fresh0;
    *fresh0 = (*fresh0).wrapping_sub(1);
    if fresh1 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
        let mut v3: uint16 = *((&raw mut g_ram as *mut uint8)
            .offset(0xc40 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize);
        loop {
            PI = get_ProjectileInstr(v3);
            if (*PI).timer as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                break;
            }
            v3 = CallProj93Instr(
                ((*PI).timer as ::core::ffi::c_int | 0x930000 as ::core::ffi::c_int) as uint32,
                v0,
                (v3 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
            );
            if v3 == 0 {
                return;
            }
        }
        *((&raw mut g_ram as *mut uint8).offset(0xc54 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (*PI).timer;
        *((&raw mut g_ram as *mut uint8).offset(0xcb8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (*PI).spritemap_ptr as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xbb4 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (*PI).x_radius as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xbc8 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (*PI).y_radius as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize) = (v3 as ::core::ffi::c_int + 8 as ::core::ffi::c_int) as uint16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Proj93Instr_Delete(mut k: uint16, mut j: uint16) -> uint16 {
    ClearProjectile(k);
    return 0 as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn Proj93Instr_Goto(mut k: uint16, mut j: uint16) -> uint16 {
    return *(RomPtr_93(j as uint16_t) as *mut uint16);
}
#[no_mangle]
pub unsafe extern "C" fn DrawPlayerExplosions2() {
    let mut v2: uint16 = 0;
    let mut v3: uint16 = 0;
    let mut current_block: u64;
    let mut v0: uint16 = 8 as uint16;
    let mut r18: uint16 = 0;
    let mut r20: uint16 = 0;
    *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16) =
        8 as uint16;
    loop {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if !(*((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize)
            == 0)
        {
            v2 = 0;
            v3 = 0;
            v2 = *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize);
            if v2 as ::core::ffi::c_int
                & (kProjectileType_TypeMask as ::core::ffi::c_int | 0x10 as ::core::ffi::c_int)
                != 0 as ::core::ffi::c_int
            {
                if (v2 as ::core::ffi::c_int & 0xf00 as ::core::ffi::c_int)
                    - 768 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    current_block = 10261563330130345889;
                } else {
                    current_block = 7149356873433890176;
                }
            } else if v2 as ::core::ffi::c_int & 0xc as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int
            {
                if v0 as ::core::ffi::c_int & 2 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                    if *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & 2 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        current_block = 10261563330130345889;
                    } else {
                        current_block = 7149356873433890176;
                    }
                } else if *((&raw mut g_ram as *mut uint8)
                    .offset(0x5b6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 2 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    current_block = 10261563330130345889;
                } else {
                    current_block = 7149356873433890176;
                }
            } else if v0 as ::core::ffi::c_int & 2 as ::core::ffi::c_int != 0 as ::core::ffi::c_int
            {
                if *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    current_block = 10261563330130345889;
                } else {
                    current_block = 7149356873433890176;
                }
            } else if *((&raw mut g_ram as *mut uint8).offset(0x5b6 as ::core::ffi::c_int as isize)
                as *mut uint16) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                == 0 as ::core::ffi::c_int
            {
                current_block = 10261563330130345889;
            } else {
                current_block = 7149356873433890176;
            }
            match current_block {
                10261563330130345889 => {}
                _ => {
                    if *((&raw mut g_ram as *mut uint8).offset(0x93f as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        r20 = (*((&raw mut g_ram as *mut uint8)
                            .offset(0xb64 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v1 as isize)
                            as ::core::ffi::c_int
                            - *((&raw mut g_ram as *mut uint8)
                                .offset(0x911 as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int)
                            as uint16;
                        v3 = (*((&raw mut g_ram as *mut uint8)
                            .offset(0xb78 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v1 as isize)
                            as ::core::ffi::c_int
                            - *((&raw mut g_ram as *mut uint8)
                                .offset(0x915 as ::core::ffi::c_int as isize)
                                as *mut uint16) as ::core::ffi::c_int)
                            as uint16;
                        r18 = v3;
                    } else {
                        let mut pt: Point16U = CalcExplosion_Mode7(v0);
                        v3 = pt.y;
                        r20 = pt.x;
                    }
                    if v3 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                        && *((&raw mut g_ram as *mut uint8)
                            .offset(0xcb8 as ::core::ffi::c_int as isize)
                            as *mut uint16)
                            .offset(v1 as isize) as ::core::ffi::c_int
                            & 0x8000 as ::core::ffi::c_int
                            != 0 as ::core::ffi::c_int
                    {
                        DrawProjectileSpritemap(v0, r20, r18);
                    }
                    v0 = *((&raw mut g_ram as *mut uint8)
                        .offset(0xdde as ::core::ffi::c_int as isize)
                        as *mut uint16);
                }
            }
        }
        v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16) = v0;
        if !(v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
    Samus_DrawShinesparkCrashEchoProjectiles();
    HandleProjectileTrails();
}
#[no_mangle]
pub unsafe extern "C" fn sub_9382FD() {
    let mut v0: uint16 = 8 as uint16;
    *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16) =
        8 as uint16;
    loop {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if *((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize)
            != 0
        {
            let mut r20: uint16 =
                (*((&raw mut g_ram as *mut uint8).offset(0xb64 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x911 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16;
            let mut r18: uint16 =
                (*((&raw mut g_ram as *mut uint8).offset(0xb78 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    - 8 as ::core::ffi::c_int
                    - *((&raw mut g_ram as *mut uint8).offset(0x915 as ::core::ffi::c_int as isize)
                        as *mut uint16) as ::core::ffi::c_int) as uint16;
            if !(r18 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int)
            {
                if *((&raw mut g_ram as *mut uint8).offset(0xcb8 as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize) as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int
                {
                    DrawProjectileSpritemap(v0, r20, r18);
                }
            }
            v0 = *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
                as *mut uint16);
        }
        v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16) = v0;
        if !(v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
    HandleProjectileTrails();
}
#[no_mangle]
pub unsafe extern "C" fn DrawBombAndProjectileExplosions() {
    let mut current_block: u64;
    let mut v0: uint16 = 18 as uint16;
    let mut v2: uint16 = 0;
    let mut r18: uint16 = 0;
    let mut r20: uint16 = 0;
    *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize) as *mut uint16) =
        18 as uint16;
    loop {
        let mut v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if !(*((&raw mut g_ram as *mut uint8).offset(0xc40 as ::core::ffi::c_int as isize)
            as *mut uint16)
            .offset(v1 as isize)
            == 0
            || (((*((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0xf00 as ::core::ffi::c_int)
                - 768 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int)
        {
            if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0xf00 as ::core::ffi::c_int
                == 768 as ::core::ffi::c_int
            {
                if *((&raw mut g_ram as *mut uint8).offset(0xc7c as ::core::ffi::c_int as isize)
                    as *mut uint16)
                    .offset(v1 as isize)
                    == 0
                {
                    current_block = 2612600445040741303;
                } else {
                    current_block = 11875828834189669668;
                }
            } else if *((&raw mut g_ram as *mut uint8).offset(0xc18 as ::core::ffi::c_int as isize)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0xf00 as ::core::ffi::c_int
                == 1280 as ::core::ffi::c_int
                || *((&raw mut g_ram as *mut uint8).offset(0x93f as ::core::ffi::c_int as isize)
                    as *mut uint16) as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0 as ::core::ffi::c_int
            {
                current_block = 11875828834189669668;
            } else {
                CalcExplosion_Mode7(v0);
                v2 = r18;
                current_block = 2311960645627505036;
            }
            match current_block {
                2612600445040741303 => {}
                _ => {
                    match current_block {
                        11875828834189669668 => {
                            let mut v3: uint16 = (*((&raw mut g_ram as *mut uint8)
                                .offset(0xb64 as ::core::ffi::c_int as isize)
                                as *mut uint16)
                                .offset(v1 as isize)
                                as ::core::ffi::c_int
                                - *((&raw mut g_ram as *mut uint8)
                                    .offset(0x911 as ::core::ffi::c_int as isize)
                                    as *mut uint16)
                                    as ::core::ffi::c_int)
                                as uint16;
                            r20 = v3;
                            if v3 as ::core::ffi::c_int - 304 as ::core::ffi::c_int
                                & 0x8000 as ::core::ffi::c_int
                                == 0
                                || v3 as ::core::ffi::c_int + 48 as ::core::ffi::c_int
                                    & 0x8000 as ::core::ffi::c_int
                                    != 0
                            {
                                current_block = 2612600445040741303;
                            } else {
                                v2 = (*((&raw mut g_ram as *mut uint8)
                                    .offset(0xb78 as ::core::ffi::c_int as isize)
                                    as *mut uint16)
                                    .offset(v1 as isize)
                                    as ::core::ffi::c_int
                                    - *((&raw mut g_ram as *mut uint8)
                                        .offset(0x915 as ::core::ffi::c_int as isize)
                                        as *mut uint16)
                                        as ::core::ffi::c_int)
                                    as uint16;
                                r18 = v2;
                                current_block = 2311960645627505036;
                            }
                        }
                        _ => {}
                    }
                    match current_block {
                        2612600445040741303 => {}
                        _ => {
                            if !(v2 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int
                                != 0 as ::core::ffi::c_int)
                            {
                                DrawProjectileSpritemap(v0, r20, r18);
                            }
                            v0 = *((&raw mut g_ram as *mut uint8)
                                .offset(0xdde as ::core::ffi::c_int as isize)
                                as *mut uint16);
                        }
                    }
                }
            }
        }
        v0 = (v0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        *((&raw mut g_ram as *mut uint8).offset(0xdde as ::core::ffi::c_int as isize)
            as *mut uint16) = v0;
        if !(v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            break;
        }
    }
}
