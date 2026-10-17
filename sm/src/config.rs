extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    fn strtol(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long;
    fn realloc(__ptr: *mut ::core::ffi::c_void, __size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn Die(error: *const ::core::ffi::c_char);
    static mut stderr: *mut FILE;
    fn fprintf(
        __stream: *mut FILE,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn SDL_GetKeyFromName(name: *const ::core::ffi::c_char) -> SDL_Keycode;
    fn ReadWholeFile(name: *const ::core::ffi::c_char, length: *mut size_t) -> *mut uint8;
    fn NextDelim(
        s: *mut *mut ::core::ffi::c_char,
        sep: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char;
    fn NextLineStripComments(s: *mut *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn NextPossiblyQuotedString(s: *mut *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn SplitKeyValue(p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn StringEqualsNoCase(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn StringStartsWithNoCase(
        a: *const ::core::ffi::c_char,
        b: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn SkipPrefix(
        big: *const ::core::ffi::c_char,
        little: *const ::core::ffi::c_char,
    ) -> *const ::core::ffi::c_char;
    fn ReplaceFilenameWithNewPath(
        old_path: *const ::core::ffi::c_char,
        new_path: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
}
pub use crate::types::{
    __int32_t, __off64_t, __off_t, __uint16_t, __uint32_t, __uint8_t, int32_t, size_t, uint16,
    uint16_t, uint32, uint32_t, uint8, uint8_t,
};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    pub _flags2: ::core::ffi::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub __pad5: size_t,
    pub _mode: ::core::ffi::c_int,
    pub _unused2: [::core::ffi::c_char; 20],
}
pub use crate::types::_IO_lock_t;
pub type FILE = _IO_FILE;
pub use crate::types::{SDL_Keycode, Sint32};
pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const SDLK_ENDCALL: C2RustUnnamed = 1073742114;
pub const SDLK_CALL: C2RustUnnamed = 1073742113;
pub const SDLK_SOFTRIGHT: C2RustUnnamed = 1073742112;
pub const SDLK_SOFTLEFT: C2RustUnnamed = 1073742111;
pub const SDLK_AUDIOFASTFORWARD: C2RustUnnamed = 1073742110;
pub const SDLK_AUDIOREWIND: C2RustUnnamed = 1073742109;
pub const SDLK_APP2: C2RustUnnamed = 1073742108;
pub const SDLK_APP1: C2RustUnnamed = 1073742107;
pub const SDLK_SLEEP: C2RustUnnamed = 1073742106;
pub const SDLK_EJECT: C2RustUnnamed = 1073742105;
pub const SDLK_KBDILLUMUP: C2RustUnnamed = 1073742104;
pub const SDLK_KBDILLUMDOWN: C2RustUnnamed = 1073742103;
pub const SDLK_KBDILLUMTOGGLE: C2RustUnnamed = 1073742102;
pub const SDLK_DISPLAYSWITCH: C2RustUnnamed = 1073742101;
pub const SDLK_BRIGHTNESSUP: C2RustUnnamed = 1073742100;
pub const SDLK_BRIGHTNESSDOWN: C2RustUnnamed = 1073742099;
pub const SDLK_AC_BOOKMARKS: C2RustUnnamed = 1073742098;
pub const SDLK_AC_REFRESH: C2RustUnnamed = 1073742097;
pub const SDLK_AC_STOP: C2RustUnnamed = 1073742096;
pub const SDLK_AC_FORWARD: C2RustUnnamed = 1073742095;
pub const SDLK_AC_BACK: C2RustUnnamed = 1073742094;
pub const SDLK_AC_HOME: C2RustUnnamed = 1073742093;
pub const SDLK_AC_SEARCH: C2RustUnnamed = 1073742092;
pub const SDLK_COMPUTER: C2RustUnnamed = 1073742091;
pub const SDLK_CALCULATOR: C2RustUnnamed = 1073742090;
pub const SDLK_MAIL: C2RustUnnamed = 1073742089;
pub const SDLK_WWW: C2RustUnnamed = 1073742088;
pub const SDLK_MEDIASELECT: C2RustUnnamed = 1073742087;
pub const SDLK_AUDIOMUTE: C2RustUnnamed = 1073742086;
pub const SDLK_AUDIOPLAY: C2RustUnnamed = 1073742085;
pub const SDLK_AUDIOSTOP: C2RustUnnamed = 1073742084;
pub const SDLK_AUDIOPREV: C2RustUnnamed = 1073742083;
pub const SDLK_AUDIONEXT: C2RustUnnamed = 1073742082;
pub const SDLK_MODE: C2RustUnnamed = 1073742081;
pub const SDLK_RGUI: C2RustUnnamed = 1073742055;
pub const SDLK_RALT: C2RustUnnamed = 1073742054;
pub const SDLK_RSHIFT: C2RustUnnamed = 1073742053;
pub const SDLK_RCTRL: C2RustUnnamed = 1073742052;
pub const SDLK_LGUI: C2RustUnnamed = 1073742051;
pub const SDLK_LALT: C2RustUnnamed = 1073742050;
pub const SDLK_LSHIFT: C2RustUnnamed = 1073742049;
pub const SDLK_LCTRL: C2RustUnnamed = 1073742048;
pub const SDLK_KP_HEXADECIMAL: C2RustUnnamed = 1073742045;
pub const SDLK_KP_DECIMAL: C2RustUnnamed = 1073742044;
pub const SDLK_KP_OCTAL: C2RustUnnamed = 1073742043;
pub const SDLK_KP_BINARY: C2RustUnnamed = 1073742042;
pub const SDLK_KP_CLEARENTRY: C2RustUnnamed = 1073742041;
pub const SDLK_KP_CLEAR: C2RustUnnamed = 1073742040;
pub const SDLK_KP_PLUSMINUS: C2RustUnnamed = 1073742039;
pub const SDLK_KP_MEMDIVIDE: C2RustUnnamed = 1073742038;
pub const SDLK_KP_MEMMULTIPLY: C2RustUnnamed = 1073742037;
pub const SDLK_KP_MEMSUBTRACT: C2RustUnnamed = 1073742036;
pub const SDLK_KP_MEMADD: C2RustUnnamed = 1073742035;
pub const SDLK_KP_MEMCLEAR: C2RustUnnamed = 1073742034;
pub const SDLK_KP_MEMRECALL: C2RustUnnamed = 1073742033;
pub const SDLK_KP_MEMSTORE: C2RustUnnamed = 1073742032;
pub const SDLK_KP_EXCLAM: C2RustUnnamed = 1073742031;
pub const SDLK_KP_AT: C2RustUnnamed = 1073742030;
pub const SDLK_KP_SPACE: C2RustUnnamed = 1073742029;
pub const SDLK_KP_HASH: C2RustUnnamed = 1073742028;
pub const SDLK_KP_COLON: C2RustUnnamed = 1073742027;
pub const SDLK_KP_DBLVERTICALBAR: C2RustUnnamed = 1073742026;
pub const SDLK_KP_VERTICALBAR: C2RustUnnamed = 1073742025;
pub const SDLK_KP_DBLAMPERSAND: C2RustUnnamed = 1073742024;
pub const SDLK_KP_AMPERSAND: C2RustUnnamed = 1073742023;
pub const SDLK_KP_GREATER: C2RustUnnamed = 1073742022;
pub const SDLK_KP_LESS: C2RustUnnamed = 1073742021;
pub const SDLK_KP_PERCENT: C2RustUnnamed = 1073742020;
pub const SDLK_KP_POWER: C2RustUnnamed = 1073742019;
pub const SDLK_KP_XOR: C2RustUnnamed = 1073742018;
pub const SDLK_KP_F: C2RustUnnamed = 1073742017;
pub const SDLK_KP_E: C2RustUnnamed = 1073742016;
pub const SDLK_KP_D: C2RustUnnamed = 1073742015;
pub const SDLK_KP_C: C2RustUnnamed = 1073742014;
pub const SDLK_KP_B: C2RustUnnamed = 1073742013;
pub const SDLK_KP_A: C2RustUnnamed = 1073742012;
pub const SDLK_KP_BACKSPACE: C2RustUnnamed = 1073742011;
pub const SDLK_KP_TAB: C2RustUnnamed = 1073742010;
pub const SDLK_KP_RIGHTBRACE: C2RustUnnamed = 1073742009;
pub const SDLK_KP_LEFTBRACE: C2RustUnnamed = 1073742008;
pub const SDLK_KP_RIGHTPAREN: C2RustUnnamed = 1073742007;
pub const SDLK_KP_LEFTPAREN: C2RustUnnamed = 1073742006;
pub const SDLK_CURRENCYSUBUNIT: C2RustUnnamed = 1073742005;
pub const SDLK_CURRENCYUNIT: C2RustUnnamed = 1073742004;
pub const SDLK_DECIMALSEPARATOR: C2RustUnnamed = 1073742003;
pub const SDLK_THOUSANDSSEPARATOR: C2RustUnnamed = 1073742002;
pub const SDLK_KP_000: C2RustUnnamed = 1073742001;
pub const SDLK_KP_00: C2RustUnnamed = 1073742000;
pub const SDLK_EXSEL: C2RustUnnamed = 1073741988;
pub const SDLK_CRSEL: C2RustUnnamed = 1073741987;
pub const SDLK_CLEARAGAIN: C2RustUnnamed = 1073741986;
pub const SDLK_OPER: C2RustUnnamed = 1073741985;
pub const SDLK_OUT: C2RustUnnamed = 1073741984;
pub const SDLK_SEPARATOR: C2RustUnnamed = 1073741983;
pub const SDLK_RETURN2: C2RustUnnamed = 1073741982;
pub const SDLK_PRIOR: C2RustUnnamed = 1073741981;
pub const SDLK_CLEAR: C2RustUnnamed = 1073741980;
pub const SDLK_CANCEL: C2RustUnnamed = 1073741979;
pub const SDLK_SYSREQ: C2RustUnnamed = 1073741978;
pub const SDLK_ALTERASE: C2RustUnnamed = 1073741977;
pub const SDLK_KP_EQUALSAS400: C2RustUnnamed = 1073741958;
pub const SDLK_KP_COMMA: C2RustUnnamed = 1073741957;
pub const SDLK_VOLUMEDOWN: C2RustUnnamed = 1073741953;
pub const SDLK_VOLUMEUP: C2RustUnnamed = 1073741952;
pub const SDLK_MUTE: C2RustUnnamed = 1073741951;
pub const SDLK_FIND: C2RustUnnamed = 1073741950;
pub const SDLK_PASTE: C2RustUnnamed = 1073741949;
pub const SDLK_COPY: C2RustUnnamed = 1073741948;
pub const SDLK_CUT: C2RustUnnamed = 1073741947;
pub const SDLK_UNDO: C2RustUnnamed = 1073741946;
pub const SDLK_AGAIN: C2RustUnnamed = 1073741945;
pub const SDLK_STOP: C2RustUnnamed = 1073741944;
pub const SDLK_SELECT: C2RustUnnamed = 1073741943;
pub const SDLK_MENU: C2RustUnnamed = 1073741942;
pub const SDLK_HELP: C2RustUnnamed = 1073741941;
pub const SDLK_EXECUTE: C2RustUnnamed = 1073741940;
pub const SDLK_F24: C2RustUnnamed = 1073741939;
pub const SDLK_F23: C2RustUnnamed = 1073741938;
pub const SDLK_F22: C2RustUnnamed = 1073741937;
pub const SDLK_F21: C2RustUnnamed = 1073741936;
pub const SDLK_F20: C2RustUnnamed = 1073741935;
pub const SDLK_F19: C2RustUnnamed = 1073741934;
pub const SDLK_F18: C2RustUnnamed = 1073741933;
pub const SDLK_F17: C2RustUnnamed = 1073741932;
pub const SDLK_F16: C2RustUnnamed = 1073741931;
pub const SDLK_F15: C2RustUnnamed = 1073741930;
pub const SDLK_F14: C2RustUnnamed = 1073741929;
pub const SDLK_F13: C2RustUnnamed = 1073741928;
pub const SDLK_KP_EQUALS: C2RustUnnamed = 1073741927;
pub const SDLK_POWER: C2RustUnnamed = 1073741926;
pub const SDLK_APPLICATION: C2RustUnnamed = 1073741925;
pub const SDLK_KP_PERIOD: C2RustUnnamed = 1073741923;
pub const SDLK_KP_0: C2RustUnnamed = 1073741922;
pub const SDLK_KP_9: C2RustUnnamed = 1073741921;
pub const SDLK_KP_8: C2RustUnnamed = 1073741920;
pub const SDLK_KP_7: C2RustUnnamed = 1073741919;
pub const SDLK_KP_6: C2RustUnnamed = 1073741918;
pub const SDLK_KP_5: C2RustUnnamed = 1073741917;
pub const SDLK_KP_4: C2RustUnnamed = 1073741916;
pub const SDLK_KP_3: C2RustUnnamed = 1073741915;
pub const SDLK_KP_2: C2RustUnnamed = 1073741914;
pub const SDLK_KP_1: C2RustUnnamed = 1073741913;
pub const SDLK_KP_ENTER: C2RustUnnamed = 1073741912;
pub const SDLK_KP_PLUS: C2RustUnnamed = 1073741911;
pub const SDLK_KP_MINUS: C2RustUnnamed = 1073741910;
pub const SDLK_KP_MULTIPLY: C2RustUnnamed = 1073741909;
pub const SDLK_KP_DIVIDE: C2RustUnnamed = 1073741908;
pub const SDLK_NUMLOCKCLEAR: C2RustUnnamed = 1073741907;
pub const SDLK_UP: C2RustUnnamed = 1073741906;
pub const SDLK_DOWN: C2RustUnnamed = 1073741905;
pub const SDLK_LEFT: C2RustUnnamed = 1073741904;
pub const SDLK_RIGHT: C2RustUnnamed = 1073741903;
pub const SDLK_PAGEDOWN: C2RustUnnamed = 1073741902;
pub const SDLK_END: C2RustUnnamed = 1073741901;
pub const SDLK_DELETE: C2RustUnnamed = 127;
pub const SDLK_PAGEUP: C2RustUnnamed = 1073741899;
pub const SDLK_HOME: C2RustUnnamed = 1073741898;
pub const SDLK_INSERT: C2RustUnnamed = 1073741897;
pub const SDLK_PAUSE: C2RustUnnamed = 1073741896;
pub const SDLK_SCROLLLOCK: C2RustUnnamed = 1073741895;
pub const SDLK_PRINTSCREEN: C2RustUnnamed = 1073741894;
pub const SDLK_F12: C2RustUnnamed = 1073741893;
pub const SDLK_F11: C2RustUnnamed = 1073741892;
pub const SDLK_F10: C2RustUnnamed = 1073741891;
pub const SDLK_F9: C2RustUnnamed = 1073741890;
pub const SDLK_F8: C2RustUnnamed = 1073741889;
pub const SDLK_F7: C2RustUnnamed = 1073741888;
pub const SDLK_F6: C2RustUnnamed = 1073741887;
pub const SDLK_F5: C2RustUnnamed = 1073741886;
pub const SDLK_F4: C2RustUnnamed = 1073741885;
pub const SDLK_F3: C2RustUnnamed = 1073741884;
pub const SDLK_F2: C2RustUnnamed = 1073741883;
pub const SDLK_F1: C2RustUnnamed = 1073741882;
pub const SDLK_CAPSLOCK: C2RustUnnamed = 1073741881;
pub const SDLK_z: C2RustUnnamed = 122;
pub const SDLK_y: C2RustUnnamed = 121;
pub const SDLK_x: C2RustUnnamed = 120;
pub const SDLK_w: C2RustUnnamed = 119;
pub const SDLK_v: C2RustUnnamed = 118;
pub const SDLK_u: C2RustUnnamed = 117;
pub const SDLK_t: C2RustUnnamed = 116;
pub const SDLK_s: C2RustUnnamed = 115;
pub const SDLK_r: C2RustUnnamed = 114;
pub const SDLK_q: C2RustUnnamed = 113;
pub const SDLK_p: C2RustUnnamed = 112;
pub const SDLK_o: C2RustUnnamed = 111;
pub const SDLK_n: C2RustUnnamed = 110;
pub const SDLK_m: C2RustUnnamed = 109;
pub const SDLK_l: C2RustUnnamed = 108;
pub const SDLK_k: C2RustUnnamed = 107;
pub const SDLK_j: C2RustUnnamed = 106;
pub const SDLK_i: C2RustUnnamed = 105;
pub const SDLK_h: C2RustUnnamed = 104;
pub const SDLK_g: C2RustUnnamed = 103;
pub const SDLK_f: C2RustUnnamed = 102;
pub const SDLK_e: C2RustUnnamed = 101;
pub const SDLK_d: C2RustUnnamed = 100;
pub const SDLK_c: C2RustUnnamed = 99;
pub const SDLK_b: C2RustUnnamed = 98;
pub const SDLK_a: C2RustUnnamed = 97;
pub const SDLK_BACKQUOTE: C2RustUnnamed = 96;
pub const SDLK_UNDERSCORE: C2RustUnnamed = 95;
pub const SDLK_CARET: C2RustUnnamed = 94;
pub const SDLK_RIGHTBRACKET: C2RustUnnamed = 93;
pub const SDLK_BACKSLASH: C2RustUnnamed = 92;
pub const SDLK_LEFTBRACKET: C2RustUnnamed = 91;
pub const SDLK_AT: C2RustUnnamed = 64;
pub const SDLK_QUESTION: C2RustUnnamed = 63;
pub const SDLK_GREATER: C2RustUnnamed = 62;
pub const SDLK_EQUALS: C2RustUnnamed = 61;
pub const SDLK_LESS: C2RustUnnamed = 60;
pub const SDLK_SEMICOLON: C2RustUnnamed = 59;
pub const SDLK_COLON: C2RustUnnamed = 58;
pub const SDLK_9: C2RustUnnamed = 57;
pub const SDLK_8: C2RustUnnamed = 56;
pub const SDLK_7: C2RustUnnamed = 55;
pub const SDLK_6: C2RustUnnamed = 54;
pub const SDLK_5: C2RustUnnamed = 53;
pub const SDLK_4: C2RustUnnamed = 52;
pub const SDLK_3: C2RustUnnamed = 51;
pub const SDLK_2: C2RustUnnamed = 50;
pub const SDLK_1: C2RustUnnamed = 49;
pub const SDLK_0: C2RustUnnamed = 48;
pub const SDLK_SLASH: C2RustUnnamed = 47;
pub const SDLK_PERIOD: C2RustUnnamed = 46;
pub const SDLK_MINUS: C2RustUnnamed = 45;
pub const SDLK_COMMA: C2RustUnnamed = 44;
pub const SDLK_PLUS: C2RustUnnamed = 43;
pub const SDLK_ASTERISK: C2RustUnnamed = 42;
pub const SDLK_RIGHTPAREN: C2RustUnnamed = 41;
pub const SDLK_LEFTPAREN: C2RustUnnamed = 40;
pub const SDLK_QUOTE: C2RustUnnamed = 39;
pub const SDLK_AMPERSAND: C2RustUnnamed = 38;
pub const SDLK_DOLLAR: C2RustUnnamed = 36;
pub const SDLK_PERCENT: C2RustUnnamed = 37;
pub const SDLK_HASH: C2RustUnnamed = 35;
pub const SDLK_QUOTEDBL: C2RustUnnamed = 34;
pub const SDLK_EXCLAIM: C2RustUnnamed = 33;
pub const SDLK_SPACE: C2RustUnnamed = 32;
pub const SDLK_TAB: C2RustUnnamed = 9;
pub const SDLK_BACKSPACE: C2RustUnnamed = 8;
pub const SDLK_ESCAPE: C2RustUnnamed = 27;
pub const SDLK_RETURN: C2RustUnnamed = 13;
pub const SDLK_UNKNOWN: C2RustUnnamed = 0;
pub use crate::types::SDL_Keymod;
pub const KMOD_RESERVED: SDL_Keymod = 32768;
pub const KMOD_GUI: SDL_Keymod = 3072;
pub const KMOD_ALT: SDL_Keymod = 768;
pub const KMOD_SHIFT: SDL_Keymod = 3;
pub const KMOD_CTRL: SDL_Keymod = 192;
pub const KMOD_SCROLL: SDL_Keymod = 32768;
pub const KMOD_MODE: SDL_Keymod = 16384;
pub const KMOD_CAPS: SDL_Keymod = 8192;
pub const KMOD_NUM: SDL_Keymod = 4096;
pub const KMOD_RGUI: SDL_Keymod = 2048;
pub const KMOD_LGUI: SDL_Keymod = 1024;
pub const KMOD_RALT: SDL_Keymod = 512;
pub const KMOD_LALT: SDL_Keymod = 256;
pub const KMOD_RCTRL: SDL_Keymod = 128;
pub const KMOD_LCTRL: SDL_Keymod = 64;
pub const KMOD_RSHIFT: SDL_Keymod = 2;
pub const KMOD_LSHIFT: SDL_Keymod = 1;
pub const KMOD_NONE: SDL_Keymod = 0;
pub type C2RustUnnamed_0 = ::core::ffi::c_uint;
pub const kKeys_Total: C2RustUnnamed_0 = 130;
pub const kKeys_VolumeDown: C2RustUnnamed_0 = 129;
pub const kKeys_VolumeUp: C2RustUnnamed_0 = 128;
pub const kKeys_ToggleRenderer: C2RustUnnamed_0 = 127;
pub const kKeys_DisplayPerf: C2RustUnnamed_0 = 126;
pub const kKeys_WindowSmaller: C2RustUnnamed_0 = 125;
pub const kKeys_WindowBigger: C2RustUnnamed_0 = 124;
pub const kKeys_ReplayTurbo: C2RustUnnamed_0 = 123;
pub const kKeys_Turbo: C2RustUnnamed_0 = 122;
pub const kKeys_PauseDimmed: C2RustUnnamed_0 = 121;
pub const kKeys_Pause: C2RustUnnamed_0 = 120;
pub const kKeys_Reset: C2RustUnnamed_0 = 119;
pub const kKeys_Fullscreen: C2RustUnnamed_0 = 118;
pub const kKeys_StopReplay: C2RustUnnamed_0 = 117;
pub const kKeys_ClearKeyLog: C2RustUnnamed_0 = 116;
pub const kKeys_ToggleWhichFrame: C2RustUnnamed_0 = 115;
pub const kKeys_CheatJump: C2RustUnnamed_0 = 114;
pub const kKeys_CheatLife: C2RustUnnamed_0 = 113;
pub const kKeys_ReplayRef_Last: C2RustUnnamed_0 = 112;
pub const kKeys_ReplayRef: C2RustUnnamed_0 = 93;
pub const kKeys_LoadRef_Last: C2RustUnnamed_0 = 92;
pub const kKeys_LoadRef: C2RustUnnamed_0 = 73;
pub const kKeys_Replay_Last: C2RustUnnamed_0 = 72;
pub const kKeys_Replay: C2RustUnnamed_0 = 53;
pub const kKeys_Save_Last: C2RustUnnamed_0 = 52;
pub const kKeys_Save: C2RustUnnamed_0 = 33;
pub const kKeys_Load_Last: C2RustUnnamed_0 = 32;
pub const kKeys_Load: C2RustUnnamed_0 = 13;
pub const kKeys_Controls_Last: C2RustUnnamed_0 = 12;
pub const kKeys_Controls: C2RustUnnamed_0 = 1;
pub const kKeys_Null: C2RustUnnamed_0 = 0;
pub type C2RustUnnamed_1 = ::core::ffi::c_uint;
pub const kOutputMethod_OpenGL: C2RustUnnamed_1 = 2;
pub const kOutputMethod_SDLSoftware: C2RustUnnamed_1 = 1;
pub const kOutputMethod_SDL: C2RustUnnamed_1 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Config {
    pub window_width: ::core::ffi::c_int,
    pub window_height: ::core::ffi::c_int,
    pub enhanced_mode7: bool,
    pub new_renderer: bool,
    pub ignore_aspect_ratio: bool,
    pub fullscreen: uint8,
    pub window_scale: uint8,
    pub enable_audio: bool,
    pub linear_filtering: bool,
    pub output_method: uint8,
    pub audio_freq: uint16,
    pub audio_channels: uint8,
    pub audio_samples: uint16,
    pub autosave: bool,
    pub extended_aspect_ratio: uint8,
    pub extend_y: bool,
    pub no_sprite_limits: bool,
    pub display_perf_title: bool,
    pub enable_msu: uint8,
    pub resume_msu: bool,
    pub disable_frame_delay: bool,
    pub msuvolume: uint8,
    pub features0: uint32,
    pub link_graphics: *const ::core::ffi::c_char,
    pub memory_buffer: *mut ::core::ffi::c_char,
    pub shader: *const ::core::ffi::c_char,
    pub msu_path: *const ::core::ffi::c_char,
}
pub type C2RustUnnamed_2 = ::core::ffi::c_uint;
pub const kMsuEnabled_Opuz: C2RustUnnamed_2 = 4;
pub const kMsuEnabled_MsuDeluxe: C2RustUnnamed_2 = 2;
pub const kMsuEnabled_Msu: C2RustUnnamed_2 = 1;
pub type C2RustUnnamed_3 = ::core::ffi::c_int;
pub const kGamepadBtn_Count: C2RustUnnamed_3 = 17;
pub const kGamepadBtn_R2: C2RustUnnamed_3 = 16;
pub const kGamepadBtn_L2: C2RustUnnamed_3 = 15;
pub const kGamepadBtn_DpadRight: C2RustUnnamed_3 = 14;
pub const kGamepadBtn_DpadLeft: C2RustUnnamed_3 = 13;
pub const kGamepadBtn_DpadDown: C2RustUnnamed_3 = 12;
pub const kGamepadBtn_DpadUp: C2RustUnnamed_3 = 11;
pub const kGamepadBtn_R1: C2RustUnnamed_3 = 10;
pub const kGamepadBtn_L1: C2RustUnnamed_3 = 9;
pub const kGamepadBtn_R3: C2RustUnnamed_3 = 8;
pub const kGamepadBtn_L3: C2RustUnnamed_3 = 7;
pub const kGamepadBtn_Start: C2RustUnnamed_3 = 6;
pub const kGamepadBtn_Guide: C2RustUnnamed_3 = 5;
pub const kGamepadBtn_Back: C2RustUnnamed_3 = 4;
pub const kGamepadBtn_Y: C2RustUnnamed_3 = 3;
pub const kGamepadBtn_X: C2RustUnnamed_3 = 2;
pub const kGamepadBtn_B: C2RustUnnamed_3 = 1;
pub const kGamepadBtn_A: C2RustUnnamed_3 = 0;
pub const kGamepadBtn_Invalid: C2RustUnnamed_3 = -1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct GamepadMapEnt {
    pub modifiers: uint32,
    pub cmd: uint16,
    pub next: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyNameId {
    pub name: *const ::core::ffi::c_char,
    pub id: uint16,
    pub size: uint16,
}
pub const kKeyMod_ScanCode: C2RustUnnamed_4 = 512;
pub const kKeyMod_Shift: C2RustUnnamed_4 = 2048;
pub const kKeyMod_Ctrl: C2RustUnnamed_4 = 4096;
pub const kKeyMod_Alt: C2RustUnnamed_4 = 1024;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyMapHashEnt {
    pub key: uint16,
    pub cmd: uint16,
    pub next: uint16,
}
pub type C2RustUnnamed_4 = ::core::ffi::c_uint;
#[inline]
unsafe extern "C" fn atoi(mut __nptr: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    return strtol(
        __nptr,
        NULL as *mut *mut ::core::ffi::c_char,
        10 as ::core::ffi::c_int,
    ) as ::core::ffi::c_int;
}
pub const SDLK_SCANCODE_MASK: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 30 as ::core::ffi::c_int;
#[no_mangle]
pub static mut g_config: Config = Config {
    window_width: 0,
    window_height: 0,
    enhanced_mode7: false,
    new_renderer: false,
    ignore_aspect_ratio: false,
    fullscreen: 0,
    window_scale: 0,
    enable_audio: false,
    linear_filtering: false,
    output_method: 0,
    audio_freq: 0,
    audio_channels: 0,
    audio_samples: 0,
    autosave: false,
    extended_aspect_ratio: 0,
    extend_y: false,
    no_sprite_limits: false,
    display_perf_title: false,
    enable_msu: 0,
    resume_msu: false,
    disable_frame_delay: false,
    msuvolume: 0,
    features0: 0,
    link_graphics: ::core::ptr::null::<::core::ffi::c_char>(),
    memory_buffer: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
    shader: ::core::ptr::null::<::core::ffi::c_char>(),
    msu_path: ::core::ptr::null::<::core::ffi::c_char>(),
};
pub const N: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut kDefaultKbdControls: [uint16; 130] = [0; 130];
static mut kKeyNameId: [KeyNameId; 24] = [
    KeyNameId {
        name: b"Null\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_Null as ::core::ffi::c_int as uint16,
        size: 65535 as uint16,
    },
    KeyNameId {
        name: b"Controls\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_Controls as ::core::ffi::c_int as uint16,
        size: (kKeys_Controls_Last as ::core::ffi::c_int - kKeys_Controls as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16,
    },
    KeyNameId {
        name: b"Load\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_Load as ::core::ffi::c_int as uint16,
        size: (kKeys_Load_Last as ::core::ffi::c_int - kKeys_Load as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16,
    },
    KeyNameId {
        name: b"Save\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_Save as ::core::ffi::c_int as uint16,
        size: (kKeys_Save_Last as ::core::ffi::c_int - kKeys_Save as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16,
    },
    KeyNameId {
        name: b"Replay\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_Replay as ::core::ffi::c_int as uint16,
        size: (kKeys_Replay_Last as ::core::ffi::c_int - kKeys_Replay as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16,
    },
    KeyNameId {
        name: b"LoadRef\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_LoadRef as ::core::ffi::c_int as uint16,
        size: (kKeys_LoadRef_Last as ::core::ffi::c_int - kKeys_LoadRef as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16,
    },
    KeyNameId {
        name: b"ReplayRef\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_ReplayRef as ::core::ffi::c_int as uint16,
        size: (kKeys_ReplayRef_Last as ::core::ffi::c_int - kKeys_ReplayRef as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as uint16,
    },
    KeyNameId {
        name: b"CheatLife\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_CheatLife as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"CheatJump\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_CheatJump as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"ToggleWhichFrame\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_ToggleWhichFrame as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"ClearKeyLog\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_ClearKeyLog as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"StopReplay\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_StopReplay as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"Fullscreen\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_Fullscreen as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"Reset\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_Reset as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"Pause\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_Pause as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"PauseDimmed\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_PauseDimmed as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"Turbo\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_Turbo as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"ReplayTurbo\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_ReplayTurbo as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"WindowBigger\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_WindowBigger as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"WindowSmaller\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_WindowSmaller as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"VolumeUp\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_VolumeUp as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"VolumeDown\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_VolumeDown as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"DisplayPerf\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_DisplayPerf as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
    KeyNameId {
        name: b"ToggleRenderer\0" as *const u8 as *const ::core::ffi::c_char,
        id: kKeys_ToggleRenderer as ::core::ffi::c_int as uint16,
        size: 1 as uint16,
    },
];
static mut keymap_hash_first: [uint16; 255] = [0; 255];
static mut keymap_hash: *mut KeyMapHashEnt =
    ::core::ptr::null::<KeyMapHashEnt>() as *mut KeyMapHashEnt;
static mut keymap_hash_size: ::core::ffi::c_int = 0;
static mut has_keynameid: [bool; 24] = [false; 24];
unsafe extern "C" fn KeyMapHash_Add(mut key: uint16, mut cmd: uint16) -> bool {
    if keymap_hash_size & 0xff as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        if keymap_hash_size > 10000 as ::core::ffi::c_int {
            Die(b"Too many keys\0" as *const u8 as *const ::core::ffi::c_char);
        }
        keymap_hash = realloc(
            keymap_hash as *mut ::core::ffi::c_void,
            (::core::mem::size_of::<KeyMapHashEnt>() as size_t)
                .wrapping_mul((keymap_hash_size + 256 as ::core::ffi::c_int) as size_t),
        ) as *mut KeyMapHashEnt;
    }
    let fresh1 = keymap_hash_size;
    keymap_hash_size = keymap_hash_size + 1;
    let mut i: ::core::ffi::c_int = fresh1;
    let mut ent: *mut KeyMapHashEnt = keymap_hash.offset(i as isize) as *mut KeyMapHashEnt;
    (*ent).key = key;
    (*ent).cmd = cmd;
    (*ent).next = 0 as uint16;
    let mut j: ::core::ffi::c_int =
        (key as uint32).wrapping_rem(255 as uint32) as ::core::ffi::c_int;
    let mut cur: *mut uint16 =
        (&raw mut keymap_hash_first as *mut uint16).offset(j as isize) as *mut uint16;
    while *cur != 0 {
        let mut ent_0: *mut KeyMapHashEnt = keymap_hash
            .offset((*cur as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize)
            as *mut KeyMapHashEnt;
        if (*ent_0).key as ::core::ffi::c_int == key as ::core::ffi::c_int {
            return false_0 != 0;
        }
        cur = &raw mut (*ent_0).next;
    }
    *cur = (i + 1 as ::core::ffi::c_int) as uint16;
    return true_0 != 0;
}
unsafe extern "C" fn KeyMapHash_Find(mut key: uint16) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = keymap_hash_first
        [(key as ::core::ffi::c_int % 255 as ::core::ffi::c_int) as usize]
        as ::core::ffi::c_int;
    while i != 0 {
        let mut ent: *mut KeyMapHashEnt =
            keymap_hash.offset((i - 1 as ::core::ffi::c_int) as isize) as *mut KeyMapHashEnt;
        if (*ent).key as ::core::ffi::c_int == key as ::core::ffi::c_int {
            return (*ent).cmd as ::core::ffi::c_int;
        }
        i = (*ent).next as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn FindCmdForSdlKey(
    mut code: SDL_Keycode,
    mut mod_0: SDL_Keymod,
) -> ::core::ffi::c_int {
    if code & !(SDLK_SCANCODE_MASK as SDL_Keycode | 0x1ff as SDL_Keycode) != 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut key: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if code != SDLK_LALT as ::core::ffi::c_int as SDL_Keycode
        && code != SDLK_RALT as ::core::ffi::c_int as SDL_Keycode
    {
        key |= if mod_0 as ::core::ffi::c_uint
            & KMOD_ALT as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0
        {
            kKeyMod_Alt as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    }
    if code != SDLK_LCTRL as ::core::ffi::c_int as SDL_Keycode
        && code != SDLK_RCTRL as ::core::ffi::c_int as SDL_Keycode
    {
        key |= if mod_0 as ::core::ffi::c_uint
            & KMOD_CTRL as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0
        {
            kKeyMod_Ctrl as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    }
    if code != SDLK_LSHIFT as ::core::ffi::c_int as SDL_Keycode
        && code != SDLK_RSHIFT as ::core::ffi::c_int as SDL_Keycode
    {
        key |= if mod_0 as ::core::ffi::c_uint
            & KMOD_SHIFT as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0
        {
            kKeyMod_Shift as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    }
    key |= ((if code & SDLK_SCANCODE_MASK as SDL_Keycode != 0 {
        kKeyMod_ScanCode as ::core::ffi::c_int as SDL_Keycode
    } else {
        0 as SDL_Keycode
    }) | code & kKeyMod_ScanCode as ::core::ffi::c_int as SDL_Keycode - 1 as SDL_Keycode)
        as ::core::ffi::c_int;
    return KeyMapHash_Find(key as uint16);
}
unsafe extern "C" fn ParseKeyArray(
    mut value: *mut ::core::ffi::c_char,
    mut cmd: ::core::ffi::c_int,
    mut size: ::core::ffi::c_int,
) {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < size && {
        s = NextDelim(&raw mut value, ',' as i32);
        !s.is_null()
    } {
        if !(*s as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            let mut key_with_mod: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            loop {
                if !StringStartsWithNoCase(
                    s,
                    b"Shift+\0" as *const u8 as *const ::core::ffi::c_char,
                )
                .is_null()
                {
                    key_with_mod |= kKeyMod_Shift as ::core::ffi::c_int;
                    s = s.offset(6 as ::core::ffi::c_int as isize);
                } else if !StringStartsWithNoCase(
                    s,
                    b"Ctrl+\0" as *const u8 as *const ::core::ffi::c_char,
                )
                .is_null()
                {
                    key_with_mod |= kKeyMod_Ctrl as ::core::ffi::c_int;
                    s = s.offset(5 as ::core::ffi::c_int as isize);
                } else {
                    if StringStartsWithNoCase(
                        s,
                        b"Alt+\0" as *const u8 as *const ::core::ffi::c_char,
                    )
                    .is_null()
                    {
                        break;
                    }
                    key_with_mod |= kKeyMod_Alt as ::core::ffi::c_int;
                    s = s.offset(4 as ::core::ffi::c_int as isize);
                }
            }
            let mut key: SDL_Keycode = SDL_GetKeyFromName(s);
            if key == SDLK_UNKNOWN as ::core::ffi::c_int as SDL_Keycode {
                fprintf(
                    stderr,
                    b"Unknown key: '%s'\n\0" as *const u8 as *const ::core::ffi::c_char,
                    s,
                );
            } else if !KeyMapHash_Add(
                (key_with_mod as SDL_Keycode
                    | (if key & SDLK_SCANCODE_MASK as SDL_Keycode != 0 {
                        kKeyMod_ScanCode as ::core::ffi::c_int as SDL_Keycode
                    } else {
                        0 as SDL_Keycode
                    })
                    | key
                        & kKeyMod_ScanCode as ::core::ffi::c_int as SDL_Keycode - 1 as SDL_Keycode)
                    as uint16,
                cmd as uint16,
            ) {
                fprintf(
                    stderr,
                    b"Duplicate key: '%s'\n\0" as *const u8 as *const ::core::ffi::c_char,
                    s,
                );
            }
        }
        i += 1;
        cmd += (cmd != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
}
static mut joymap_first: [uint16; 17] = [0; 17];
static mut joymap_ents: *mut GamepadMapEnt =
    ::core::ptr::null::<GamepadMapEnt>() as *mut GamepadMapEnt;
static mut joymap_size: ::core::ffi::c_int = 0;
static mut has_joypad_controls: bool = false;
unsafe extern "C" fn CountBits32(mut n: uint32) -> ::core::ffi::c_int {
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while n != 0 as uint32 {
        n &= n.wrapping_sub(1 as uint32);
        count += 1;
    }
    return count;
}
unsafe extern "C" fn GamepadMap_Add(
    mut button: ::core::ffi::c_int,
    mut modifiers: uint32,
    mut cmd: uint16,
) {
    if joymap_size & 0xff as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        if joymap_size > 1000 as ::core::ffi::c_int {
            Die(b"Too many joypad keys\0" as *const u8 as *const ::core::ffi::c_char);
        }
        joymap_ents = realloc(
            joymap_ents as *mut ::core::ffi::c_void,
            (::core::mem::size_of::<GamepadMapEnt>() as size_t)
                .wrapping_mul((joymap_size + 64 as ::core::ffi::c_int) as size_t),
        ) as *mut GamepadMapEnt;
        if joymap_ents.is_null() {
            Die(b"realloc failure\0" as *const u8 as *const ::core::ffi::c_char);
        }
    }
    let mut p: *mut uint16 =
        (&raw mut joymap_first as *mut uint16).offset(button as isize) as *mut uint16;
    let mut cb: ::core::ffi::c_int = CountBits32(modifiers);
    while *p as ::core::ffi::c_int != 0
        && cb
            < CountBits32(
                (*joymap_ents
                    .offset((*p as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize))
                .modifiers,
            )
    {
        p = &raw mut (*joymap_ents
            .offset((*p as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize))
        .next;
    }
    let fresh0 = joymap_size;
    joymap_size = joymap_size + 1;
    let mut i: ::core::ffi::c_int = fresh0;
    let mut ent: *mut GamepadMapEnt = joymap_ents.offset(i as isize) as *mut GamepadMapEnt;
    (*ent).modifiers = modifiers;
    (*ent).cmd = cmd;
    (*ent).next = *p;
    *p = (i + 1 as ::core::ffi::c_int) as uint16;
}
#[no_mangle]
pub unsafe extern "C" fn FindCmdForGamepadButton(
    mut button: ::core::ffi::c_int,
    mut modifiers: uint32,
) -> ::core::ffi::c_int {
    let mut ent: *mut GamepadMapEnt = ::core::ptr::null_mut::<GamepadMapEnt>();
    let mut e: ::core::ffi::c_int = joymap_first[button as usize] as ::core::ffi::c_int;
    while e != 0 as ::core::ffi::c_int {
        ent = joymap_ents.offset((e - 1 as ::core::ffi::c_int) as isize) as *mut GamepadMapEnt;
        if modifiers & (*ent).modifiers == (*ent).modifiers {
            return (*ent).cmd as ::core::ffi::c_int;
        }
        e = (*ent).next as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn ParseGamepadButtonName(
    mut value: *mut *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut s: *const ::core::ffi::c_char = *value;
    static mut kGamepadKeyNames: [*const ::core::ffi::c_char; 19] = [
        b"Back\0" as *const u8 as *const ::core::ffi::c_char,
        b"Guide\0" as *const u8 as *const ::core::ffi::c_char,
        b"Start\0" as *const u8 as *const ::core::ffi::c_char,
        b"L3\0" as *const u8 as *const ::core::ffi::c_char,
        b"R3\0" as *const u8 as *const ::core::ffi::c_char,
        b"L1\0" as *const u8 as *const ::core::ffi::c_char,
        b"R1\0" as *const u8 as *const ::core::ffi::c_char,
        b"DpadUp\0" as *const u8 as *const ::core::ffi::c_char,
        b"DpadDown\0" as *const u8 as *const ::core::ffi::c_char,
        b"DpadLeft\0" as *const u8 as *const ::core::ffi::c_char,
        b"DpadRight\0" as *const u8 as *const ::core::ffi::c_char,
        b"L2\0" as *const u8 as *const ::core::ffi::c_char,
        b"R2\0" as *const u8 as *const ::core::ffi::c_char,
        b"Lb\0" as *const u8 as *const ::core::ffi::c_char,
        b"Rb\0" as *const u8 as *const ::core::ffi::c_char,
        b"A\0" as *const u8 as *const ::core::ffi::c_char,
        b"B\0" as *const u8 as *const ::core::ffi::c_char,
        b"X\0" as *const u8 as *const ::core::ffi::c_char,
        b"Y\0" as *const u8 as *const ::core::ffi::c_char,
    ];
    static mut kGamepadKeyIds: [uint8; 19] = [
        kGamepadBtn_Back as ::core::ffi::c_int as uint8,
        kGamepadBtn_Guide as ::core::ffi::c_int as uint8,
        kGamepadBtn_Start as ::core::ffi::c_int as uint8,
        kGamepadBtn_L3 as ::core::ffi::c_int as uint8,
        kGamepadBtn_R3 as ::core::ffi::c_int as uint8,
        kGamepadBtn_L1 as ::core::ffi::c_int as uint8,
        kGamepadBtn_R1 as ::core::ffi::c_int as uint8,
        kGamepadBtn_DpadUp as ::core::ffi::c_int as uint8,
        kGamepadBtn_DpadDown as ::core::ffi::c_int as uint8,
        kGamepadBtn_DpadLeft as ::core::ffi::c_int as uint8,
        kGamepadBtn_DpadRight as ::core::ffi::c_int as uint8,
        kGamepadBtn_L2 as ::core::ffi::c_int as uint8,
        kGamepadBtn_R2 as ::core::ffi::c_int as uint8,
        kGamepadBtn_L1 as ::core::ffi::c_int as uint8,
        kGamepadBtn_R1 as ::core::ffi::c_int as uint8,
        kGamepadBtn_A as ::core::ffi::c_int as uint8,
        kGamepadBtn_B as ::core::ffi::c_int as uint8,
        kGamepadBtn_X as ::core::ffi::c_int as uint8,
        kGamepadBtn_Y as ::core::ffi::c_int as uint8,
    ];
    let mut i: size_t = 0 as size_t;
    while i
        != (::core::mem::size_of::<[*const ::core::ffi::c_char; 19]>() as usize)
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>() as usize)
    {
        let mut r: *const ::core::ffi::c_char =
            StringStartsWithNoCase(s, kGamepadKeyNames[i as usize]);
        if !r.is_null() {
            *value = r;
            return kGamepadKeyIds[i as usize] as ::core::ffi::c_int;
        }
        i = i.wrapping_add(1);
    }
    return kGamepadBtn_Invalid as ::core::ffi::c_int;
}
static mut kDefaultGamepadCmds: [uint8; 12] = [
    kGamepadBtn_DpadUp as ::core::ffi::c_int as uint8,
    kGamepadBtn_DpadDown as ::core::ffi::c_int as uint8,
    kGamepadBtn_DpadLeft as ::core::ffi::c_int as uint8,
    kGamepadBtn_DpadRight as ::core::ffi::c_int as uint8,
    kGamepadBtn_Back as ::core::ffi::c_int as uint8,
    kGamepadBtn_Start as ::core::ffi::c_int as uint8,
    kGamepadBtn_B as ::core::ffi::c_int as uint8,
    kGamepadBtn_A as ::core::ffi::c_int as uint8,
    kGamepadBtn_Y as ::core::ffi::c_int as uint8,
    kGamepadBtn_X as ::core::ffi::c_int as uint8,
    kGamepadBtn_L1 as ::core::ffi::c_int as uint8,
    kGamepadBtn_R1 as ::core::ffi::c_int as uint8,
];
unsafe extern "C" fn ParseGamepadArray(
    mut value: *mut ::core::ffi::c_char,
    mut cmd: ::core::ffi::c_int,
    mut size: ::core::ffi::c_int,
) {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < size && {
        s = NextDelim(&raw mut value, ',' as i32);
        !s.is_null()
    } {
        if !(*s as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            let mut modifiers: uint32 = 0 as uint32;
            let mut ss: *const ::core::ffi::c_char = s;
            loop {
                let mut button: ::core::ffi::c_int = ParseGamepadButtonName(&raw mut ss);
                if !(button == kGamepadBtn_Invalid as ::core::ffi::c_int) {
                    while *ss as ::core::ffi::c_int == ' ' as i32
                        || *ss as ::core::ffi::c_int == '\t' as i32
                    {
                        ss = ss.offset(1);
                    }
                    if *ss as ::core::ffi::c_int == '+' as i32 {
                        ss = ss.offset(1);
                        modifiers |= ((1 as ::core::ffi::c_int) << button) as uint32;
                        continue;
                    } else if *ss as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                        GamepadMap_Add(button, modifiers, cmd as uint16);
                        break;
                    }
                }
                fprintf(
                    stderr,
                    b"Unknown gamepad button: '%s'\n\0" as *const u8 as *const ::core::ffi::c_char,
                    s,
                );
                break;
            }
        }
        i += 1;
        cmd += (cmd != 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn RegisterDefaultKeys() {
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while (i as usize)
        < (::core::mem::size_of::<[KeyNameId; 24]>() as usize)
            .wrapping_div(::core::mem::size_of::<KeyNameId>() as usize)
    {
        if !has_keynameid[i as usize] {
            let mut size: ::core::ffi::c_int = kKeyNameId[i as usize].size as ::core::ffi::c_int;
            let mut k: ::core::ffi::c_int = kKeyNameId[i as usize].id as ::core::ffi::c_int;
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while j < size {
                KeyMapHash_Add(kDefaultKbdControls[k as usize], k as uint16);
                j += 1;
                k += 1;
            }
        }
        i += 1;
    }
    if !has_joypad_controls {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (i_0 as usize)
            < (::core::mem::size_of::<[uint8; 12]>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
        {
            GamepadMap_Add(
                kDefaultGamepadCmds[i_0 as usize] as ::core::ffi::c_int,
                0 as uint32,
                (kKeys_Controls as ::core::ffi::c_int + i_0) as uint16,
            );
            i_0 += 1;
        }
    }
}
unsafe extern "C" fn GetIniSection(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if StringEqualsNoCase(s, b"[KeyMap]\0" as *const u8 as *const ::core::ffi::c_char) {
        return 0 as ::core::ffi::c_int;
    }
    if StringEqualsNoCase(
        s,
        b"[Graphics]\0" as *const u8 as *const ::core::ffi::c_char,
    ) {
        return 1 as ::core::ffi::c_int;
    }
    if StringEqualsNoCase(s, b"[Sound]\0" as *const u8 as *const ::core::ffi::c_char) {
        return 2 as ::core::ffi::c_int;
    }
    if StringEqualsNoCase(s, b"[General]\0" as *const u8 as *const ::core::ffi::c_char) {
        return 3 as ::core::ffi::c_int;
    }
    if StringEqualsNoCase(
        s,
        b"[Features]\0" as *const u8 as *const ::core::ffi::c_char,
    ) {
        return 4 as ::core::ffi::c_int;
    }
    if StringEqualsNoCase(
        s,
        b"[GamepadMap]\0" as *const u8 as *const ::core::ffi::c_char,
    ) {
        return 5 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn ParseBool(
    mut value: *const ::core::ffi::c_char,
    mut result: *mut bool,
) -> bool {
    let mut rv: bool = false_0 != 0;
    let fresh2 = value;
    value = value.offset(1);
    match *fresh2 as ::core::ffi::c_int | 32 as ::core::ffi::c_int {
        48 => {
            if !(*value as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
                return false_0 != 0;
            }
        }
        102 => {
            if !StringEqualsNoCase(value, b"alse\0" as *const u8 as *const ::core::ffi::c_char) {
                return false_0 != 0;
            }
        }
        110 => {
            if !StringEqualsNoCase(value, b"o\0" as *const u8 as *const ::core::ffi::c_char) {
                return false_0 != 0;
            }
        }
        111 => {
            rv = *value as ::core::ffi::c_int | 32 as ::core::ffi::c_int == 'n' as i32;
            if !StringEqualsNoCase(
                value,
                if rv as ::core::ffi::c_int != 0 {
                    b"n\0" as *const u8 as *const ::core::ffi::c_char
                } else {
                    b"ff\0" as *const u8 as *const ::core::ffi::c_char
                },
            ) {
                return false_0 != 0;
            }
        }
        49 => {
            rv = true_0 != 0;
            if !(*value as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
                return false_0 != 0;
            }
        }
        121 => {
            rv = true_0 != 0;
            if !StringEqualsNoCase(value, b"es\0" as *const u8 as *const ::core::ffi::c_char) {
                return false_0 != 0;
            }
        }
        116 => {
            rv = true_0 != 0;
            if !StringEqualsNoCase(value, b"rue\0" as *const u8 as *const ::core::ffi::c_char) {
                return false_0 != 0;
            }
        }
        _ => return false_0 != 0,
    }
    if !result.is_null() {
        *result = rv;
        return true_0 != 0;
    }
    return rv;
}
unsafe extern "C" fn HandleIniConfig(
    mut section: ::core::ffi::c_int,
    mut key: *const ::core::ffi::c_char,
    mut value: *mut ::core::ffi::c_char,
) -> bool {
    if section == 0 as ::core::ffi::c_int {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (i as usize)
            < (::core::mem::size_of::<[KeyNameId; 24]>() as usize)
                .wrapping_div(::core::mem::size_of::<KeyNameId>() as usize)
        {
            if StringEqualsNoCase(key, kKeyNameId[i as usize].name) {
                has_keynameid[i as usize] = true_0 != 0;
                ParseKeyArray(
                    value,
                    kKeyNameId[i as usize].id as ::core::ffi::c_int,
                    kKeyNameId[i as usize].size as ::core::ffi::c_int,
                );
                return true_0 != 0;
            }
            i += 1;
        }
    } else if section == 5 as ::core::ffi::c_int {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (i_0 as usize)
            < (::core::mem::size_of::<[KeyNameId; 24]>() as usize)
                .wrapping_div(::core::mem::size_of::<KeyNameId>() as usize)
        {
            if StringEqualsNoCase(key, kKeyNameId[i_0 as usize].name) {
                if i_0 == 1 as ::core::ffi::c_int {
                    has_joypad_controls = true_0 != 0;
                }
                ParseGamepadArray(
                    value,
                    kKeyNameId[i_0 as usize].id as ::core::ffi::c_int,
                    kKeyNameId[i_0 as usize].size as ::core::ffi::c_int,
                );
                return true_0 != 0;
            }
            i_0 += 1;
        }
    } else if section == 1 as ::core::ffi::c_int {
        if StringEqualsNoCase(
            key,
            b"WindowSize\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if StringEqualsNoCase(value, b"Auto\0" as *const u8 as *const ::core::ffi::c_char) {
                g_config.window_width = 0 as ::core::ffi::c_int;
                g_config.window_height = 0 as ::core::ffi::c_int;
                return true_0 != 0;
            }
            loop {
                s = NextDelim(&raw mut value, 'x' as i32);
                if s.is_null() {
                    break;
                }
                if g_config.window_width == 0 as ::core::ffi::c_int {
                    g_config.window_width = atoi(s);
                } else {
                    g_config.window_height = atoi(s);
                    return true_0 != 0;
                }
            }
        } else if StringEqualsNoCase(
            key,
            b"EnhancedMode7\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            return ParseBool(value, &raw mut g_config.enhanced_mode7);
        } else if StringEqualsNoCase(
            key,
            b"NewRenderer\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            return ParseBool(value, &raw mut g_config.new_renderer);
        } else if StringEqualsNoCase(
            key,
            b"IgnoreAspectRatio\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            return ParseBool(value, &raw mut g_config.ignore_aspect_ratio);
        } else if StringEqualsNoCase(
            key,
            b"Fullscreen\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            g_config.fullscreen = strtol(
                value,
                NULL_0 as *mut *mut ::core::ffi::c_char,
                10 as ::core::ffi::c_int,
            ) as uint8;
            return true_0 != 0;
        } else if StringEqualsNoCase(
            key,
            b"WindowScale\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            g_config.window_scale = strtol(
                value,
                NULL_0 as *mut *mut ::core::ffi::c_char,
                10 as ::core::ffi::c_int,
            ) as uint8;
            return true_0 != 0;
        } else if StringEqualsNoCase(
            key,
            b"OutputMethod\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            g_config.output_method = (if StringEqualsNoCase(
                value,
                b"SDL-Software\0" as *const u8 as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int
                != 0
            {
                kOutputMethod_SDLSoftware as ::core::ffi::c_int
            } else if StringEqualsNoCase(
                value,
                b"OpenGL\0" as *const u8 as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int
                != 0
            {
                kOutputMethod_OpenGL as ::core::ffi::c_int
            } else {
                kOutputMethod_SDL as ::core::ffi::c_int
            }) as uint8;
            return true_0 != 0;
        } else if StringEqualsNoCase(
            key,
            b"LinearFiltering\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            return ParseBool(value, &raw mut g_config.linear_filtering);
        } else if StringEqualsNoCase(
            key,
            b"NoSpriteLimits\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            return ParseBool(value, &raw mut g_config.no_sprite_limits);
        } else if StringEqualsNoCase(
            key,
            b"LinkGraphics\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            g_config.link_graphics = value;
            return true_0 != 0;
        } else if StringEqualsNoCase(key, b"Shader\0" as *const u8 as *const ::core::ffi::c_char) {
            g_config.shader = if *value as ::core::ffi::c_int != 0 {
                value
            } else {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            };
            return true_0 != 0;
        }
    } else if section == 2 as ::core::ffi::c_int {
        if StringEqualsNoCase(
            key,
            b"EnableAudio\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            return ParseBool(value, &raw mut g_config.enable_audio);
        } else if StringEqualsNoCase(
            key,
            b"AudioFreq\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            g_config.audio_freq = strtol(
                value,
                NULL_0 as *mut *mut ::core::ffi::c_char,
                10 as ::core::ffi::c_int,
            ) as uint16;
            return true_0 != 0;
        } else if StringEqualsNoCase(
            key,
            b"AudioChannels\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            g_config.audio_channels = strtol(
                value,
                NULL_0 as *mut *mut ::core::ffi::c_char,
                10 as ::core::ffi::c_int,
            ) as uint8;
            return true_0 != 0;
        } else if StringEqualsNoCase(
            key,
            b"AudioSamples\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            g_config.audio_samples = strtol(
                value,
                NULL_0 as *mut *mut ::core::ffi::c_char,
                10 as ::core::ffi::c_int,
            ) as uint16;
            return true_0 != 0;
        } else if StringEqualsNoCase(
            key,
            b"EnableMSU\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            if StringEqualsNoCase(value, b"opuz\0" as *const u8 as *const ::core::ffi::c_char) {
                g_config.enable_msu = kMsuEnabled_Opuz as ::core::ffi::c_int as uint8;
            } else if StringEqualsNoCase(
                value,
                b"deluxe\0" as *const u8 as *const ::core::ffi::c_char,
            ) {
                g_config.enable_msu = kMsuEnabled_MsuDeluxe as ::core::ffi::c_int as uint8;
            } else if StringEqualsNoCase(
                value,
                b"deluxe-opuz\0" as *const u8 as *const ::core::ffi::c_char,
            ) {
                g_config.enable_msu = (kMsuEnabled_MsuDeluxe as ::core::ffi::c_int
                    | kMsuEnabled_Opuz as ::core::ffi::c_int)
                    as uint8;
            } else {
                return ParseBool(value, &raw mut g_config.enable_msu as *mut bool);
            }
            return true_0 != 0;
        } else if StringEqualsNoCase(key, b"MSUPath\0" as *const u8 as *const ::core::ffi::c_char) {
            g_config.msu_path = value;
            return true_0 != 0;
        } else if StringEqualsNoCase(
            key,
            b"MSUVolume\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            g_config.msuvolume = atoi(value) as uint8;
            return true_0 != 0;
        } else if StringEqualsNoCase(
            key,
            b"ResumeMSU\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            return ParseBool(value, &raw mut g_config.resume_msu);
        }
    } else if section == 3 as ::core::ffi::c_int {
        if StringEqualsNoCase(
            key,
            b"Autosave\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            g_config.autosave = strtol(
                value,
                NULL_0 as *mut *mut ::core::ffi::c_char,
                10 as ::core::ffi::c_int,
            ) != 0;
            return true_0 != 0;
        } else if StringEqualsNoCase(
            key,
            b"DisplayPerfInTitle\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            return ParseBool(value, &raw mut g_config.display_perf_title);
        } else if StringEqualsNoCase(
            key,
            b"DisableFrameDelay\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            return ParseBool(value, &raw mut g_config.disable_frame_delay);
        }
    } else {
        section == 4 as ::core::ffi::c_int;
    }
    return false_0 != 0;
}
unsafe extern "C" fn ParseOneConfigFile(
    mut filename: *const ::core::ffi::c_char,
    mut depth: ::core::ffi::c_int,
) -> bool {
    let mut filedata: *mut ::core::ffi::c_char =
        ReadWholeFile(filename, ::core::ptr::null_mut::<size_t>()) as *mut ::core::ffi::c_char;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if filedata.is_null() {
        return false_0 != 0;
    }
    let mut section: ::core::ffi::c_int = -(2 as ::core::ffi::c_int);
    g_config.memory_buffer = filedata;
    let mut lineno: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    loop {
        p = NextLineStripComments(&raw mut filedata);
        if p.is_null() {
            break;
        }
        if !(*p as ::core::ffi::c_int == 0 as ::core::ffi::c_int) {
            if *p as ::core::ffi::c_int == '[' as i32 {
                section = GetIniSection(p);
                if section < 0 as ::core::ffi::c_int {
                    fprintf(
                        stderr,
                        b"%s:%d: Invalid .ini section %s\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        filename,
                        lineno,
                        p,
                    );
                }
            } else if *p as ::core::ffi::c_int == '!' as i32
                && !SkipPrefix(
                    p.offset(1 as ::core::ffi::c_int as isize),
                    b"include \0" as *const u8 as *const ::core::ffi::c_char,
                )
                .is_null()
            {
                let mut tt: *mut ::core::ffi::c_char = p.offset(8 as ::core::ffi::c_int as isize);
                let mut new_filename: *mut ::core::ffi::c_char =
                    ReplaceFilenameWithNewPath(filename, NextPossiblyQuotedString(&raw mut tt));
                if depth > 10 as ::core::ffi::c_int
                    || !ParseOneConfigFile(new_filename, depth + 1 as ::core::ffi::c_int)
                {
                    fprintf(
                        stderr,
                        b"Warning: Unable to read %s\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        new_filename,
                    );
                }
                free(new_filename as *mut ::core::ffi::c_void);
            } else if section == -(2 as ::core::ffi::c_int) {
                fprintf(
                    stderr,
                    b"%s:%d: Expecting [section]\n\0" as *const u8 as *const ::core::ffi::c_char,
                    filename,
                    lineno,
                );
            } else {
                let mut v: *mut ::core::ffi::c_char = SplitKeyValue(p);
                if v.is_null() {
                    fprintf(
                        stderr,
                        b"%s:%d: Expecting 'key=value'\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        filename,
                        lineno,
                    );
                } else if section >= 0 as ::core::ffi::c_int && !HandleIniConfig(section, p, v) {
                    fprintf(
                        stderr,
                        b"%s:%d: Can't parse '%s'\n\0" as *const u8 as *const ::core::ffi::c_char,
                        filename,
                        lineno,
                        p,
                    );
                }
            }
        }
        lineno += 1;
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn ParseConfigFile(mut filename: *const ::core::ffi::c_char) {
    g_config.msuvolume = 100 as uint8;
    if !filename.is_null()
        || !ParseOneConfigFile(
            b"sm.user.ini\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
        )
    {
        if filename.is_null() {
            filename = b"sm.ini\0" as *const u8 as *const ::core::ffi::c_char;
        }
        if !ParseOneConfigFile(filename, 0 as ::core::ffi::c_int) {
            fprintf(
                stderr,
                b"Warning: Unable to read config file %s\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                filename,
            );
        }
    }
    RegisterDefaultKeys();
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
unsafe extern "C" fn run_static_initializers() {
    kDefaultKbdControls = [
        0 as ::core::ffi::c_int as uint16,
        ((if SDLK_UP as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_UP as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_DOWN as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_DOWN as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_LEFT as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_LEFT as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_RIGHT as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_RIGHT as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_RSHIFT as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_RSHIFT as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_RETURN as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_RETURN as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_x as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_x as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_z as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_z as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_s as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_s as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_a as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_a as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_c as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_c as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_v as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_v as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_F1 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F1 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_F2 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F2 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_F3 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F3 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_F4 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F4 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_F5 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F5 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_F6 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F6 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_F7 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F7 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_F8 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F8 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_F9 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F9 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_F10 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F10 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        ((if SDLK_F1 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F1 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Shift as ::core::ffi::c_int) as uint16,
        ((if SDLK_F2 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F2 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Shift as ::core::ffi::c_int) as uint16,
        ((if SDLK_F3 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F3 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Shift as ::core::ffi::c_int) as uint16,
        ((if SDLK_F4 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F4 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Shift as ::core::ffi::c_int) as uint16,
        ((if SDLK_F5 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F5 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Shift as ::core::ffi::c_int) as uint16,
        ((if SDLK_F6 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F6 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Shift as ::core::ffi::c_int) as uint16,
        ((if SDLK_F7 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F7 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Shift as ::core::ffi::c_int) as uint16,
        ((if SDLK_F8 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F8 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Shift as ::core::ffi::c_int) as uint16,
        ((if SDLK_F9 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F9 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Shift as ::core::ffi::c_int) as uint16,
        ((if SDLK_F10 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F10 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Shift as ::core::ffi::c_int) as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        ((if SDLK_F1 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F1 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_F2 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F2 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_F3 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F3 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_F4 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F4 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_F5 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F5 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_F6 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F6 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_F7 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F7 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_F8 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F8 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_F9 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F9 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_F10 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_F10 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        ((if SDLK_1 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_1 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_2 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_2 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_3 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_3 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_4 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_4 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_5 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_5 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_6 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_6 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_7 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_7 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_8 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_8 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_9 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_9 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_0 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_0 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_MINUS as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_MINUS as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_EQUALS as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_EQUALS as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_BACKSPACE as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_BACKSPACE as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        ((if SDLK_1 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_1 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_2 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_2 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_3 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_3 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_4 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_4 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_5 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_5 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_6 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_6 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_7 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_7 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_8 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_8 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_9 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_9 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_0 as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_0 as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_MINUS as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_MINUS as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_EQUALS as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_EQUALS as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_BACKSPACE as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_BACKSPACE as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        N as uint16,
        ((if SDLK_w as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_w as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_q as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_q as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_r as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_r as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Shift as ::core::ffi::c_int) as uint16,
        ((if SDLK_k as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_k as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_l as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_l as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_RETURN as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_RETURN as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Alt as ::core::ffi::c_int) as uint16,
        ((if SDLK_r as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_r as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Ctrl as ::core::ffi::c_int) as uint16,
        ((if SDLK_p as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_p as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int
            | kKeyMod_Shift as ::core::ffi::c_int) as uint16,
        ((if SDLK_p as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_p as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_TAB as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_TAB as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_t as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_t as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        N as uint16,
        N as uint16,
        ((if SDLK_f as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_f as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        ((if SDLK_r as ::core::ffi::c_int & SDLK_SCANCODE_MASK != 0 {
            kKeyMod_ScanCode as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) | SDLK_r as ::core::ffi::c_int
            & kKeyMod_ScanCode as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16,
        0 as ::core::ffi::c_int as uint16,
        0 as ::core::ffi::c_int as uint16,
    ];
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [run_static_initializers];
