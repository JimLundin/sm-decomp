//! Shared byte-compatible gameplay types.

pub type uint8 = u8;
pub type uint16 = u16;
pub type VoidP = uint16;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct LongPtr {
    pub addr: VoidP,
    pub bank: uint8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct OamEnt {
    pub xcoord: uint8,
    pub ycoord: uint8,
    pub charnum: uint8,
    pub flags: uint8,
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct VramWriteEntry {
    pub size: uint16,
    pub src: LongPtr,
    pub vram_dst: uint16,
}

pub type Buttons = ::core::ffi::c_uint;

pub type CalculateAngleFromXYFunc = unsafe extern "C" fn(uint16) -> uint8;

pub type Consts_60 = ::core::ffi::c_uint;

pub type Consts_80 = ::core::ffi::c_uint;

pub type Consts_81 = ::core::ffi::c_uint;

pub type Consts_82 = ::core::ffi::c_uint;

pub type Consts_83 = ::core::ffi::c_uint;

pub type Consts_84 = ::core::ffi::c_uint;

pub type Consts_86 = ::core::ffi::c_uint;

pub type Consts_87 = ::core::ffi::c_uint;

pub type Consts_88 = ::core::ffi::c_uint;

pub type Consts_89 = ::core::ffi::c_uint;

pub type Consts_8B = ::core::ffi::c_uint;

pub type Consts_8C = ::core::ffi::c_uint;

pub type Consts_8D = ::core::ffi::c_uint;

pub type Consts_8F = ::core::ffi::c_uint;

pub type Consts_90 = ::core::ffi::c_uint;

pub type Consts_91 = ::core::ffi::c_uint;

pub type Consts_94 = ::core::ffi::c_uint;

pub type Consts_9B = ::core::ffi::c_uint;

pub type Consts_A0 = ::core::ffi::c_uint;

pub type Consts_A2 = ::core::ffi::c_uint;

pub type Consts_A3 = ::core::ffi::c_uint;

pub type Consts_A4 = ::core::ffi::c_uint;

pub type Consts_A5 = ::core::ffi::c_uint;

pub type Consts_A6 = ::core::ffi::c_uint;

pub type Consts_A7 = ::core::ffi::c_uint;

pub type Consts_A8 = ::core::ffi::c_uint;

pub type Consts_A9 = ::core::ffi::c_uint;

pub type Consts_AA = ::core::ffi::c_uint;

pub type Consts_AD = ::core::ffi::c_uint;

pub type Consts_B2 = ::core::ffi::c_uint;

pub type Consts_B3 = ::core::ffi::c_uint;

pub type Consts_B4 = ::core::ffi::c_uint;

pub type Consts_B7 = ::core::ffi::c_uint;

pub type CoroutineRet = uint8_t;

pub type CrocomireFunc = unsafe extern "C" fn(uint16, *const uint16) -> *const uint16;

pub type DspReg = ::core::ffi::c_uint;

pub type EnemyAiBits = ::core::ffi::c_uint;

pub type EnemyProps = ::core::ffi::c_uint;

pub type FuncXY_V = unsafe extern "C" fn(uint16, uint16) -> ();

pub type Func_AnimDelay = unsafe extern "C" fn(*const uint8) -> bool;

pub type Func_CalcHdma = unsafe extern "C" fn(uint16, uint16) -> ();

pub type Func_SpazerSba_V = unsafe extern "C" fn(uint16, uint16) -> ();

pub type Func_U8 = unsafe extern "C" fn() -> uint8;

pub type Func_V = unsafe extern "C" fn() -> ();

pub type Func_V_A = unsafe extern "C" fn() -> uint8;

pub type Func_V_Coroutine = unsafe extern "C" fn() -> CoroutineRet;

pub type Func_Y_V = unsafe extern "C" fn(uint16) -> ();

pub type Func_Y_Y = unsafe extern "C" fn(uint16) -> uint16;

pub type GLDEBUGPROC = Option<
    unsafe extern "C" fn(
        GLenum,
        GLenum,
        GLuint,
        GLenum,
        GLsizei,
        *const GLchar,
        *const ::core::ffi::c_void,
    ) -> (),
>;

pub type GLSLScaleType = ::core::ffi::c_uint;

pub type GLbitfield = ::core::ffi::c_uint;

pub type GLboolean = ::core::ffi::c_uchar;

pub type GLbyte = ::core::ffi::c_schar;

pub type GLchar = ::core::ffi::c_char;

pub type GLdouble = ::core::ffi::c_double;

pub type GLenum = ::core::ffi::c_uint;

pub type GLfloat = ::core::ffi::c_float;

pub type GLint = ::core::ffi::c_int;

pub type GLint64 = int64_t;

pub type GLintptr = ptrdiff_t;

pub type GLshort = ::core::ffi::c_short;

pub type GLsizei = ::core::ffi::c_int;

pub type GLsizeiptr = ptrdiff_t;

pub type GLubyte = ::core::ffi::c_uchar;

pub type GLuint = ::core::ffi::c_uint;

pub type GLushort = ::core::ffi::c_ushort;

pub type GameState = ::core::ffi::c_uint;

pub type HandlerFunc = unsafe extern "C" fn() -> ();

pub type LiquidPhysicsType = ::core::ffi::c_uint;

pub type LoadingGameState = ::core::ffi::c_uint;

pub type PFN_LOADFUNCPOINTERS = Option<unsafe extern "C" fn() -> ::core::ffi::c_int>;

pub type PpuZbufType = uint16_t;

pub type ProjectileType = ::core::ffi::c_uint;

pub type RunFrameFunc = unsafe extern "C" fn(uint16, ::core::ffi::c_int) -> ();

pub type RunMode = ::core::ffi::c_uint;

pub type SDL_AudioCallback =
    Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut Uint8, ::core::ffi::c_int) -> ()>;

pub type SDL_AudioDeviceID = Uint32;

pub type SDL_AudioFormat = Uint16;

pub type SDL_FingerID = Sint64;

pub type SDL_GLContext = *mut ::core::ffi::c_void;

pub type SDL_GLattr = ::core::ffi::c_uint;

pub type SDL_GestureID = Sint64;

pub type SDL_HitTestResult = ::core::ffi::c_uint;

pub type SDL_JoystickID = Sint32;

pub type SDL_JoystickPowerLevel = ::core::ffi::c_int;

pub type SDL_Keycode = Sint32;

pub type SDL_Keymod = ::core::ffi::c_uint;

pub type SDL_Scancode = ::core::ffi::c_uint;

pub type SDL_TouchID = Sint64;

pub type SDL_bool = ::core::ffi::c_uint;

pub type SamusMovementType = ::core::ffi::c_uint;

pub type SamusPose = ::core::ffi::c_uint;

pub type SaveLoadFunc =
    unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut ::core::ffi::c_void, size_t) -> ();

pub type Sint16 = int16_t;

pub type Sint32 = int32_t;

pub type Sint64 = int64_t;

pub type SnesRegs = ::core::ffi::c_uint;

pub type SyncAllFunc = unsafe extern "C" fn() -> ();

pub type Uint16 = uint16_t;

pub type Uint32 = uint32_t;

pub type Uint64 = uint64_t;

pub type Uint8 = uint8_t;

pub type _IO_lock_t = ();

pub type __int16_t = i16;

pub type __int32_t = i32;

pub type __int64_t = i64;

pub type __int8_t = i8;

pub type __mode_t = ::core::ffi::c_uint;

pub type __off64_t = ::core::ffi::c_long;

pub type __off_t = ::core::ffi::c_long;

pub type __time_t = ::core::ffi::c_long;

pub type __uint16_t = u16;

pub type __uint32_t = u32;

pub type __uint64_t = u64;

pub type __uint8_t = u8;

pub type int16 = int16_t;

pub type int16_t = __int16_t;

pub type int32 = int32_t;

pub type int32_t = __int32_t;

pub type int64_t = __int64_t;

pub type int8 = int8_t;

pub type int8_t = __int8_t;

pub type ogl_LoadStatus = ::core::ffi::c_uint;

pub type ptrdiff_t = isize;

pub type size_t = usize;

pub type stbi__uint16 = uint16_t;

pub type stbi__uint32 = uint32_t;

pub type stbi_uc = ::core::ffi::c_uchar;

pub type stbi_us = ::core::ffi::c_ushort;

pub type time_t = __time_t;

pub type uint = ::core::ffi::c_uint;

pub type uint16_t = __uint16_t;

pub type uint32 = uint32_t;

pub type uint32_t = __uint32_t;

pub type uint64 = uint64_t;

pub type uint64_t = __uint64_t;

pub type uint8_t = __uint8_t;

pub type uintptr_t = usize;
