extern "C" {
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn apu_cpuRead(apu: *mut Apu, adr: uint16_t) -> uint8_t;
    fn apu_cpuWrite(apu: *mut Apu, adr: uint16_t, val: uint8_t);
}
pub use crate::types::{
    __int16_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16_t, int8_t, size_t, uint16_t,
    uint32_t, uint8_t, Apu, ApuHistory as C2RustUnnamed, Dsp, DspChannel, DspRegWriteHistory,
    SaveLoadFunc, Spc, Timer,
};
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut cyclesPerOpcode: [::core::ffi::c_int; 256] = [
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    12 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    9 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    7 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    8 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    6 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    5 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    2 as ::core::ffi::c_int,
    4 as ::core::ffi::c_int,
    3 as ::core::ffi::c_int,
];
unsafe extern "C" fn spc_read(mut spc: *mut Spc, mut adr: uint16_t) -> uint8_t {
    return apu_cpuRead((*spc).apu, adr);
}
unsafe extern "C" fn spc_write(mut spc: *mut Spc, mut adr: uint16_t, mut val: uint8_t) {
    apu_cpuWrite((*spc).apu, adr, val);
}
#[no_mangle]
pub unsafe extern "C" fn spc_init(mut apu: *mut Apu) -> *mut Spc {
    let mut spc: *mut Spc = malloc(::core::mem::size_of::<Spc>() as size_t) as *mut Spc;
    (*spc).apu = apu;
    return spc;
}
#[no_mangle]
pub unsafe extern "C" fn spc_free(mut spc: *mut Spc) {
    free(spc as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn spc_reset(mut spc: *mut Spc) {
    (*spc).a = 0 as uint8_t;
    (*spc).x = 0 as uint8_t;
    (*spc).y = 0 as uint8_t;
    (*spc).sp = 0 as uint8_t;
    (*spc).pc = (spc_read(spc, 0xfffe as uint16_t) as ::core::ffi::c_int
        | (spc_read(spc, 0xffff as uint16_t) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as uint16_t;
    (*spc).c = false_0 != 0;
    (*spc).z = false_0 != 0;
    (*spc).v = false_0 != 0;
    (*spc).n = false_0 != 0;
    (*spc).i = false_0 != 0;
    (*spc).h = false_0 != 0;
    (*spc).p = false_0 != 0;
    (*spc).b = false_0 != 0;
    (*spc).stopped = false_0 != 0;
    (*spc).cyclesUsed = 0 as uint8_t;
}
#[no_mangle]
pub unsafe extern "C" fn spc_saveload(
    mut spc: *mut Spc,
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
) {
    func.expect("non-null function pointer")(
        ctx,
        &raw mut (*spc).a as *mut ::core::ffi::c_void,
        (23 as size_t).wrapping_sub(8 as size_t),
    );
}
#[no_mangle]
pub unsafe extern "C" fn spc_runOpcode(mut spc: *mut Spc) -> ::core::ffi::c_int {
    (*spc).cyclesUsed = 0 as uint8_t;
    if (*spc).stopped {
        return 1 as ::core::ffi::c_int;
    }
    let mut opcode: uint8_t = spc_readOpcode(spc);
    (*spc).cyclesUsed = cyclesPerOpcode[opcode as usize] as uint8_t;
    spc_doOpcode(spc, opcode);
    return (*spc).cyclesUsed as ::core::ffi::c_int;
}
unsafe extern "C" fn spc_readOpcode(mut spc: *mut Spc) -> uint8_t {
    let fresh0 = (*spc).pc;
    (*spc).pc = (*spc).pc.wrapping_add(1);
    return spc_read(spc, fresh0);
}
unsafe extern "C" fn spc_readOpcodeWord(mut spc: *mut Spc) -> uint16_t {
    let mut low: uint8_t = spc_readOpcode(spc);
    return (low as ::core::ffi::c_int
        | (spc_readOpcode(spc) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as uint16_t;
}
unsafe extern "C" fn spc_getFlags(mut spc: *mut Spc) -> uint8_t {
    let mut val: uint8_t = (((*spc).n as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as uint8_t;
    val = (val as ::core::ffi::c_int | ((*spc).v as ::core::ffi::c_int) << 6 as ::core::ffi::c_int)
        as uint8_t;
    val = (val as ::core::ffi::c_int | ((*spc).p as ::core::ffi::c_int) << 5 as ::core::ffi::c_int)
        as uint8_t;
    val = (val as ::core::ffi::c_int | ((*spc).b as ::core::ffi::c_int) << 4 as ::core::ffi::c_int)
        as uint8_t;
    val = (val as ::core::ffi::c_int | ((*spc).h as ::core::ffi::c_int) << 3 as ::core::ffi::c_int)
        as uint8_t;
    val = (val as ::core::ffi::c_int | ((*spc).i as ::core::ffi::c_int) << 2 as ::core::ffi::c_int)
        as uint8_t;
    val = (val as ::core::ffi::c_int | ((*spc).z as ::core::ffi::c_int) << 1 as ::core::ffi::c_int)
        as uint8_t;
    val = (val as ::core::ffi::c_int | (*spc).c as ::core::ffi::c_int) as uint8_t;
    return val;
}
unsafe extern "C" fn spc_setFlags(mut spc: *mut Spc, mut val: uint8_t) {
    (*spc).n = val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
    (*spc).v = val as ::core::ffi::c_int & 0x40 as ::core::ffi::c_int != 0;
    (*spc).p = val as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int != 0;
    (*spc).b = val as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int != 0;
    (*spc).h = val as ::core::ffi::c_int & 8 as ::core::ffi::c_int != 0;
    (*spc).i = val as ::core::ffi::c_int & 4 as ::core::ffi::c_int != 0;
    (*spc).z = val as ::core::ffi::c_int & 2 as ::core::ffi::c_int != 0;
    (*spc).c = val as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn spc_setZN(mut spc: *mut Spc, mut value: uint8_t) {
    (*spc).z = value as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
    (*spc).n = value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn spc_doBranch(mut spc: *mut Spc, mut value: uint8_t, mut check: bool) {
    if check {
        (*spc).cyclesUsed =
            ((*spc).cyclesUsed as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint8_t;
        (*spc).pc =
            ((*spc).pc as ::core::ffi::c_int + value as int8_t as ::core::ffi::c_int) as uint16_t;
    }
}
unsafe extern "C" fn spc_pullByte(mut spc: *mut Spc) -> uint8_t {
    (*spc).sp = (*spc).sp.wrapping_add(1);
    return spc_read(
        spc,
        (0x100 as ::core::ffi::c_int | (*spc).sp as ::core::ffi::c_int) as uint16_t,
    );
}
unsafe extern "C" fn spc_pushByte(mut spc: *mut Spc, mut value: uint8_t) {
    spc_write(
        spc,
        (0x100 as ::core::ffi::c_int | (*spc).sp as ::core::ffi::c_int) as uint16_t,
        value,
    );
    (*spc).sp = (*spc).sp.wrapping_sub(1);
}
unsafe extern "C" fn spc_pullWord(mut spc: *mut Spc) -> uint16_t {
    let mut value: uint8_t = spc_pullByte(spc);
    return (value as ::core::ffi::c_int
        | (spc_pullByte(spc) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as uint16_t;
}
unsafe extern "C" fn spc_pushWord(mut spc: *mut Spc, mut value: uint16_t) {
    spc_pushByte(
        spc,
        (value as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8_t,
    );
    spc_pushByte(
        spc,
        (value as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t,
    );
}
unsafe extern "C" fn spc_readWord(
    mut spc: *mut Spc,
    mut adrl: uint16_t,
    mut adrh: uint16_t,
) -> uint16_t {
    let mut value: uint8_t = spc_read(spc, adrl);
    return (value as ::core::ffi::c_int
        | (spc_read(spc, adrh) as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as uint16_t;
}
unsafe extern "C" fn spc_writeWord(
    mut spc: *mut Spc,
    mut adrl: uint16_t,
    mut adrh: uint16_t,
    mut value: uint16_t,
) {
    spc_write(
        spc,
        adrl,
        (value as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t,
    );
    spc_write(
        spc,
        adrh,
        (value as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8_t,
    );
}
unsafe extern "C" fn spc_adrDp(mut spc: *mut Spc) -> uint16_t {
    return (spc_readOpcode(spc) as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t;
}
unsafe extern "C" fn spc_adrAbs(mut spc: *mut Spc) -> uint16_t {
    return spc_readOpcodeWord(spc);
}
unsafe extern "C" fn spc_adrInd(mut spc: *mut Spc) -> uint16_t {
    return ((*spc).x as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t;
}
unsafe extern "C" fn spc_adrIdx(mut spc: *mut Spc) -> uint16_t {
    let mut pointer: uint8_t = spc_readOpcode(spc);
    return spc_readWord(
        spc,
        (pointer as ::core::ffi::c_int + (*spc).x as ::core::ffi::c_int
            & 0xff as ::core::ffi::c_int
            | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t,
        (pointer as ::core::ffi::c_int + (*spc).x as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 0xff as ::core::ffi::c_int
            | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t,
    );
}
unsafe extern "C" fn spc_adrImm(mut spc: *mut Spc) -> uint16_t {
    let fresh1 = (*spc).pc;
    (*spc).pc = (*spc).pc.wrapping_add(1);
    return fresh1;
}
unsafe extern "C" fn spc_adrDpx(mut spc: *mut Spc) -> uint16_t {
    return (spc_readOpcode(spc) as ::core::ffi::c_int + (*spc).x as ::core::ffi::c_int
        & 0xff as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t;
}
unsafe extern "C" fn spc_adrDpy(mut spc: *mut Spc) -> uint16_t {
    return (spc_readOpcode(spc) as ::core::ffi::c_int + (*spc).y as ::core::ffi::c_int
        & 0xff as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t;
}
unsafe extern "C" fn spc_adrAbx(mut spc: *mut Spc) -> uint16_t {
    return (spc_readOpcodeWord(spc) as ::core::ffi::c_int + (*spc).x as ::core::ffi::c_int
        & 0xffff as ::core::ffi::c_int) as uint16_t;
}
unsafe extern "C" fn spc_adrAby(mut spc: *mut Spc) -> uint16_t {
    return (spc_readOpcodeWord(spc) as ::core::ffi::c_int + (*spc).y as ::core::ffi::c_int
        & 0xffff as ::core::ffi::c_int) as uint16_t;
}
unsafe extern "C" fn spc_adrIdy(mut spc: *mut Spc) -> uint16_t {
    let mut pointer: uint8_t = spc_readOpcode(spc);
    let mut adr: uint16_t = spc_readWord(
        spc,
        (pointer as ::core::ffi::c_int
            | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t,
        (pointer as ::core::ffi::c_int + 1 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
            | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t,
    );
    return (adr as ::core::ffi::c_int + (*spc).y as ::core::ffi::c_int
        & 0xffff as ::core::ffi::c_int) as uint16_t;
}
unsafe extern "C" fn spc_adrDpDp(mut spc: *mut Spc, mut src: *mut uint16_t) -> uint16_t {
    *src = (spc_readOpcode(spc) as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t;
    return (spc_readOpcode(spc) as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t;
}
unsafe extern "C" fn spc_adrDpImm(mut spc: *mut Spc, mut src: *mut uint16_t) -> uint16_t {
    let fresh3 = (*spc).pc;
    (*spc).pc = (*spc).pc.wrapping_add(1);
    *src = fresh3;
    return (spc_readOpcode(spc) as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t;
}
unsafe extern "C" fn spc_adrIndInd(mut spc: *mut Spc, mut src: *mut uint16_t) -> uint16_t {
    *src = ((*spc).y as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t;
    return ((*spc).x as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t;
}
unsafe extern "C" fn spc_adrAbsBit(mut spc: *mut Spc, mut adr: *mut uint16_t) -> uint8_t {
    let mut adrBit: uint16_t = spc_readOpcodeWord(spc);
    *adr = (adrBit as ::core::ffi::c_int & 0x1fff as ::core::ffi::c_int) as uint16_t;
    return (adrBit as ::core::ffi::c_int >> 13 as ::core::ffi::c_int) as uint8_t;
}
unsafe extern "C" fn spc_adrDpWord(mut spc: *mut Spc, mut low: *mut uint16_t) -> uint16_t {
    let mut adr: uint8_t = spc_readOpcode(spc);
    *low = (adr as ::core::ffi::c_int | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
        as uint16_t;
    return (adr as ::core::ffi::c_int + 1 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t;
}
unsafe extern "C" fn spc_adrIndP(mut spc: *mut Spc) -> uint16_t {
    let fresh2 = (*spc).x;
    (*spc).x = (*spc).x.wrapping_add(1);
    return (fresh2 as ::core::ffi::c_int
        | ((*spc).p as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16_t;
}
unsafe extern "C" fn spc_and(mut spc: *mut Spc, mut adr: uint16_t) {
    (*spc).a =
        ((*spc).a as ::core::ffi::c_int & spc_read(spc, adr) as ::core::ffi::c_int) as uint8_t;
    spc_setZN(spc, (*spc).a);
}
unsafe extern "C" fn spc_andm(mut spc: *mut Spc, mut dst: uint16_t, mut src: uint16_t) {
    let mut value: uint8_t = spc_read(spc, src);
    let mut result: uint8_t =
        (spc_read(spc, dst) as ::core::ffi::c_int & value as ::core::ffi::c_int) as uint8_t;
    spc_write(spc, dst, result);
    spc_setZN(spc, result);
}
unsafe extern "C" fn spc_or(mut spc: *mut Spc, mut adr: uint16_t) {
    (*spc).a =
        ((*spc).a as ::core::ffi::c_int | spc_read(spc, adr) as ::core::ffi::c_int) as uint8_t;
    spc_setZN(spc, (*spc).a);
}
unsafe extern "C" fn spc_orm(mut spc: *mut Spc, mut dst: uint16_t, mut src: uint16_t) {
    let mut value: uint8_t = spc_read(spc, src);
    let mut result: uint8_t =
        (spc_read(spc, dst) as ::core::ffi::c_int | value as ::core::ffi::c_int) as uint8_t;
    spc_write(spc, dst, result);
    spc_setZN(spc, result);
}
unsafe extern "C" fn spc_eor(mut spc: *mut Spc, mut adr: uint16_t) {
    (*spc).a =
        ((*spc).a as ::core::ffi::c_int ^ spc_read(spc, adr) as ::core::ffi::c_int) as uint8_t;
    spc_setZN(spc, (*spc).a);
}
unsafe extern "C" fn spc_eorm(mut spc: *mut Spc, mut dst: uint16_t, mut src: uint16_t) {
    let mut value: uint8_t = spc_read(spc, src);
    let mut result: uint8_t =
        (spc_read(spc, dst) as ::core::ffi::c_int ^ value as ::core::ffi::c_int) as uint8_t;
    spc_write(spc, dst, result);
    spc_setZN(spc, result);
}
unsafe extern "C" fn spc_adc(mut spc: *mut Spc, mut adr: uint16_t) {
    let mut value: uint8_t = spc_read(spc, adr);
    let mut result: ::core::ffi::c_int = (*spc).a as ::core::ffi::c_int
        + value as ::core::ffi::c_int
        + (*spc).c as ::core::ffi::c_int;
    (*spc).v = (*spc).a as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
        == value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
        && value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
            != result & 0x80 as ::core::ffi::c_int;
    (*spc).h = ((*spc).a as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
        + (value as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
        + (*spc).c as ::core::ffi::c_int
        > 0xf as ::core::ffi::c_int;
    (*spc).c = result > 0xff as ::core::ffi::c_int;
    (*spc).a = result as uint8_t;
    spc_setZN(spc, (*spc).a);
}
unsafe extern "C" fn spc_adcm(mut spc: *mut Spc, mut dst: uint16_t, mut src: uint16_t) {
    let mut value: uint8_t = spc_read(spc, src);
    let mut applyOn: uint8_t = spc_read(spc, dst);
    let mut result: ::core::ffi::c_int = applyOn as ::core::ffi::c_int
        + value as ::core::ffi::c_int
        + (*spc).c as ::core::ffi::c_int;
    (*spc).v = applyOn as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
        == value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
        && value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
            != result & 0x80 as ::core::ffi::c_int;
    (*spc).h = (applyOn as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
        + (value as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
        + (*spc).c as ::core::ffi::c_int
        > 0xf as ::core::ffi::c_int;
    (*spc).c = result > 0xff as ::core::ffi::c_int;
    spc_write(spc, dst, result as uint8_t);
    spc_setZN(spc, result as uint8_t);
}
unsafe extern "C" fn spc_sbc(mut spc: *mut Spc, mut adr: uint16_t) {
    let mut value: uint8_t =
        (spc_read(spc, adr) as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int) as uint8_t;
    let mut result: ::core::ffi::c_int = (*spc).a as ::core::ffi::c_int
        + value as ::core::ffi::c_int
        + (*spc).c as ::core::ffi::c_int;
    (*spc).v = (*spc).a as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
        == value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
        && value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
            != result & 0x80 as ::core::ffi::c_int;
    (*spc).h = ((*spc).a as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
        + (value as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
        + (*spc).c as ::core::ffi::c_int
        > 0xf as ::core::ffi::c_int;
    (*spc).c = result > 0xff as ::core::ffi::c_int;
    (*spc).a = result as uint8_t;
    spc_setZN(spc, (*spc).a);
}
unsafe extern "C" fn spc_sbcm(mut spc: *mut Spc, mut dst: uint16_t, mut src: uint16_t) {
    let mut value: uint8_t =
        (spc_read(spc, src) as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int) as uint8_t;
    let mut applyOn: uint8_t = spc_read(spc, dst);
    let mut result: ::core::ffi::c_int = applyOn as ::core::ffi::c_int
        + value as ::core::ffi::c_int
        + (*spc).c as ::core::ffi::c_int;
    (*spc).v = applyOn as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
        == value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
        && value as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int
            != result & 0x80 as ::core::ffi::c_int;
    (*spc).h = (applyOn as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
        + (value as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
        + (*spc).c as ::core::ffi::c_int
        > 0xf as ::core::ffi::c_int;
    (*spc).c = result > 0xff as ::core::ffi::c_int;
    spc_write(spc, dst, result as uint8_t);
    spc_setZN(spc, result as uint8_t);
}
unsafe extern "C" fn spc_cmp(mut spc: *mut Spc, mut adr: uint16_t) {
    let mut value: uint8_t =
        (spc_read(spc, adr) as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int) as uint8_t;
    let mut result: ::core::ffi::c_int =
        (*spc).a as ::core::ffi::c_int + value as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    (*spc).c = result > 0xff as ::core::ffi::c_int;
    spc_setZN(spc, result as uint8_t);
}
unsafe extern "C" fn spc_cmpx(mut spc: *mut Spc, mut adr: uint16_t) {
    let mut value: uint8_t =
        (spc_read(spc, adr) as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int) as uint8_t;
    let mut result: ::core::ffi::c_int =
        (*spc).x as ::core::ffi::c_int + value as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    (*spc).c = result > 0xff as ::core::ffi::c_int;
    spc_setZN(spc, result as uint8_t);
}
unsafe extern "C" fn spc_cmpy(mut spc: *mut Spc, mut adr: uint16_t) {
    let mut value: uint8_t =
        (spc_read(spc, adr) as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int) as uint8_t;
    let mut result: ::core::ffi::c_int =
        (*spc).y as ::core::ffi::c_int + value as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    (*spc).c = result > 0xff as ::core::ffi::c_int;
    spc_setZN(spc, result as uint8_t);
}
unsafe extern "C" fn spc_cmpm(mut spc: *mut Spc, mut dst: uint16_t, mut src: uint16_t) {
    let mut value: uint8_t =
        (spc_read(spc, src) as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int) as uint8_t;
    let mut result: ::core::ffi::c_int = spc_read(spc, dst) as ::core::ffi::c_int
        + value as ::core::ffi::c_int
        + 1 as ::core::ffi::c_int;
    (*spc).c = result > 0xff as ::core::ffi::c_int;
    spc_setZN(spc, result as uint8_t);
}
unsafe extern "C" fn spc_mov(mut spc: *mut Spc, mut adr: uint16_t) {
    (*spc).a = spc_read(spc, adr);
    spc_setZN(spc, (*spc).a);
}
unsafe extern "C" fn spc_movx(mut spc: *mut Spc, mut adr: uint16_t) {
    (*spc).x = spc_read(spc, adr);
    spc_setZN(spc, (*spc).x);
}
unsafe extern "C" fn spc_movy(mut spc: *mut Spc, mut adr: uint16_t) {
    (*spc).y = spc_read(spc, adr);
    spc_setZN(spc, (*spc).y);
}
unsafe extern "C" fn spc_movs(mut spc: *mut Spc, mut adr: uint16_t) {
    spc_read(spc, adr);
    spc_write(spc, adr, (*spc).a);
}
unsafe extern "C" fn spc_movsx(mut spc: *mut Spc, mut adr: uint16_t) {
    spc_read(spc, adr);
    spc_write(spc, adr, (*spc).x);
}
unsafe extern "C" fn spc_movsy(mut spc: *mut Spc, mut adr: uint16_t) {
    spc_read(spc, adr);
    spc_write(spc, adr, (*spc).y);
}
unsafe extern "C" fn spc_asl(mut spc: *mut Spc, mut adr: uint16_t) {
    let mut val: uint8_t = spc_read(spc, adr);
    (*spc).c = val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
    val = ((val as ::core::ffi::c_int) << 1 as ::core::ffi::c_int) as uint8_t;
    spc_write(spc, adr, val);
    spc_setZN(spc, val);
}
unsafe extern "C" fn spc_lsr(mut spc: *mut Spc, mut adr: uint16_t) {
    let mut val: uint8_t = spc_read(spc, adr);
    (*spc).c = val as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0;
    val = (val as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint8_t;
    spc_write(spc, adr, val);
    spc_setZN(spc, val);
}
unsafe extern "C" fn spc_rol(mut spc: *mut Spc, mut adr: uint16_t) {
    let mut val: uint8_t = spc_read(spc, adr);
    let mut newC: bool = val as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
    val = ((val as ::core::ffi::c_int) << 1 as ::core::ffi::c_int | (*spc).c as ::core::ffi::c_int)
        as uint8_t;
    (*spc).c = newC;
    spc_write(spc, adr, val);
    spc_setZN(spc, val);
}
unsafe extern "C" fn spc_ror(mut spc: *mut Spc, mut adr: uint16_t) {
    let mut val: uint8_t = spc_read(spc, adr);
    let mut newC: bool = val as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0;
    val = (val as ::core::ffi::c_int >> 1 as ::core::ffi::c_int
        | ((*spc).c as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as uint8_t;
    (*spc).c = newC;
    spc_write(spc, adr, val);
    spc_setZN(spc, val);
}
unsafe extern "C" fn spc_inc(mut spc: *mut Spc, mut adr: uint16_t) {
    let mut val: uint8_t =
        (spc_read(spc, adr) as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint8_t;
    spc_write(spc, adr, val);
    spc_setZN(spc, val);
}
unsafe extern "C" fn spc_dec(mut spc: *mut Spc, mut adr: uint16_t) {
    let mut val: uint8_t =
        (spc_read(spc, adr) as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint8_t;
    spc_write(spc, adr, val);
    spc_setZN(spc, val);
}
unsafe extern "C" fn spc_doOpcode(mut spc: *mut Spc, mut opcode: uint8_t) {
    match opcode as ::core::ffi::c_int {
        1 | 17 | 33 | 49 | 65 | 81 | 97 | 113 | 129 | 145 | 161 | 177 | 193 | 209 | 225 | 241 => {
            spc_pushWord(spc, (*spc).pc);
            let mut adr: uint16_t = (0xffde as ::core::ffi::c_int
                - 2 as ::core::ffi::c_int
                    * (opcode as ::core::ffi::c_int >> 4 as ::core::ffi::c_int))
                as uint16_t;
            (*spc).pc = spc_readWord(
                spc,
                adr,
                (adr as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as uint16_t,
            );
        }
        2 | 34 | 66 | 98 | 130 | 162 | 194 | 226 => {
            let mut adr_0: uint16_t = spc_adrDp(spc);
            spc_write(
                spc,
                adr_0,
                (spc_read(spc, adr_0) as ::core::ffi::c_int
                    | (1 as ::core::ffi::c_int)
                        << (opcode as ::core::ffi::c_int >> 5 as ::core::ffi::c_int))
                    as uint8_t,
            );
        }
        18 | 50 | 82 | 114 | 146 | 178 | 210 | 242 => {
            let mut adr_1: uint16_t = spc_adrDp(spc);
            spc_write(
                spc,
                adr_1,
                (spc_read(spc, adr_1) as ::core::ffi::c_int
                    & !((1 as ::core::ffi::c_int)
                        << (opcode as ::core::ffi::c_int >> 5 as ::core::ffi::c_int)))
                    as uint8_t,
            );
        }
        3 | 35 | 67 | 99 | 131 | 163 | 195 | 227 => {
            let mut val: uint8_t = spc_read(spc, spc_adrDp(spc));
            spc_doBranch(
                spc,
                spc_readOpcode(spc),
                val as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int)
                        << (opcode as ::core::ffi::c_int >> 5 as ::core::ffi::c_int)
                    != 0,
            );
        }
        19 | 51 | 83 | 115 | 147 | 179 | 211 | 243 => {
            let mut val_0: uint8_t = spc_read(spc, spc_adrDp(spc));
            spc_doBranch(
                spc,
                spc_readOpcode(spc),
                val_0 as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int)
                        << (opcode as ::core::ffi::c_int >> 5 as ::core::ffi::c_int)
                    == 0 as ::core::ffi::c_int,
            );
        }
        4 => {
            spc_or(spc, spc_adrDp(spc));
        }
        5 => {
            spc_or(spc, spc_adrAbs(spc));
        }
        6 => {
            spc_or(spc, spc_adrInd(spc));
        }
        7 => {
            spc_or(spc, spc_adrIdx(spc));
        }
        8 => {
            spc_or(spc, spc_adrImm(spc));
        }
        9 => {
            let mut src: uint16_t = 0 as uint16_t;
            let mut dst: uint16_t = spc_adrDpDp(spc, &raw mut src);
            spc_orm(spc, dst, src);
        }
        10 => {
            let mut adr_2: uint16_t = 0 as uint16_t;
            let mut bit: uint8_t = spc_adrAbsBit(spc, &raw mut adr_2);
            (*spc).c = (*spc).c as ::core::ffi::c_int
                | spc_read(spc, adr_2) as ::core::ffi::c_int >> bit as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                != 0;
        }
        11 => {
            spc_asl(spc, spc_adrDp(spc));
        }
        12 => {
            spc_asl(spc, spc_adrAbs(spc));
        }
        13 => {
            spc_pushByte(spc, spc_getFlags(spc));
        }
        14 => {
            let mut adr_3: uint16_t = spc_adrAbs(spc);
            let mut val_1: uint8_t = spc_read(spc, adr_3);
            let mut result: uint8_t = ((*spc).a as ::core::ffi::c_int
                + (val_1 as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int) as uint8_t;
            spc_setZN(spc, result);
            spc_write(
                spc,
                adr_3,
                (val_1 as ::core::ffi::c_int | (*spc).a as ::core::ffi::c_int) as uint8_t,
            );
        }
        15 => {
            spc_pushWord(spc, (*spc).pc);
            spc_pushByte(spc, spc_getFlags(spc));
            (*spc).i = false_0 != 0;
            (*spc).b = true_0 != 0;
            (*spc).pc = spc_readWord(spc, 0xffde as uint16_t, 0xffdf as uint16_t);
        }
        16 => {
            spc_doBranch(spc, spc_readOpcode(spc), !(*spc).n);
        }
        20 => {
            spc_or(spc, spc_adrDpx(spc));
        }
        21 => {
            spc_or(spc, spc_adrAbx(spc));
        }
        22 => {
            spc_or(spc, spc_adrAby(spc));
        }
        23 => {
            spc_or(spc, spc_adrIdy(spc));
        }
        24 => {
            let mut src_0: uint16_t = 0 as uint16_t;
            let mut dst_0: uint16_t = spc_adrDpImm(spc, &raw mut src_0);
            spc_orm(spc, dst_0, src_0);
        }
        25 => {
            let mut src_1: uint16_t = 0 as uint16_t;
            let mut dst_1: uint16_t = spc_adrIndInd(spc, &raw mut src_1);
            spc_orm(spc, dst_1, src_1);
        }
        26 => {
            let mut low: uint16_t = 0 as uint16_t;
            let mut high: uint16_t = spc_adrDpWord(spc, &raw mut low);
            let mut value: uint16_t = (spc_readWord(spc, low, high) as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as uint16_t;
            (*spc).z = value as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
            (*spc).n = value as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0;
            spc_writeWord(spc, low, high, value);
        }
        27 => {
            spc_asl(spc, spc_adrDpx(spc));
        }
        28 => {
            (*spc).c = (*spc).a as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
            (*spc).a = (((*spc).a as ::core::ffi::c_int) << 1 as ::core::ffi::c_int) as uint8_t;
            spc_setZN(spc, (*spc).a);
        }
        29 => {
            (*spc).x = (*spc).x.wrapping_sub(1);
            spc_setZN(spc, (*spc).x);
        }
        30 => {
            spc_cmpx(spc, spc_adrAbs(spc));
        }
        31 => {
            let mut pointer: uint16_t = spc_readOpcodeWord(spc);
            (*spc).pc = spc_readWord(
                spc,
                (pointer as ::core::ffi::c_int + (*spc).x as ::core::ffi::c_int
                    & 0xffff as ::core::ffi::c_int) as uint16_t,
                (pointer as ::core::ffi::c_int
                    + (*spc).x as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    & 0xffff as ::core::ffi::c_int) as uint16_t,
            );
        }
        32 => {
            (*spc).p = false_0 != 0;
        }
        36 => {
            spc_and(spc, spc_adrDp(spc));
        }
        37 => {
            spc_and(spc, spc_adrAbs(spc));
        }
        38 => {
            spc_and(spc, spc_adrInd(spc));
        }
        39 => {
            spc_and(spc, spc_adrIdx(spc));
        }
        40 => {
            spc_and(spc, spc_adrImm(spc));
        }
        41 => {
            let mut src_2: uint16_t = 0 as uint16_t;
            let mut dst_2: uint16_t = spc_adrDpDp(spc, &raw mut src_2);
            spc_andm(spc, dst_2, src_2);
        }
        42 => {
            let mut adr_4: uint16_t = 0 as uint16_t;
            let mut bit_0: uint8_t = spc_adrAbsBit(spc, &raw mut adr_4);
            (*spc).c = (*spc).c as ::core::ffi::c_int
                | !(spc_read(spc, adr_4) as ::core::ffi::c_int >> bit_0 as ::core::ffi::c_int)
                    & 1 as ::core::ffi::c_int
                != 0;
        }
        43 => {
            spc_rol(spc, spc_adrDp(spc));
        }
        44 => {
            spc_rol(spc, spc_adrAbs(spc));
        }
        45 => {
            spc_pushByte(spc, (*spc).a);
        }
        46 => {
            let mut val_2: uint8_t = (spc_read(spc, spc_adrDp(spc)) as ::core::ffi::c_int
                ^ 0xff as ::core::ffi::c_int) as uint8_t;
            let mut result_0: uint8_t = ((*spc).a as ::core::ffi::c_int
                + val_2 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint8_t;
            spc_doBranch(
                spc,
                spc_readOpcode(spc),
                result_0 as ::core::ffi::c_int != 0 as ::core::ffi::c_int,
            );
        }
        47 => {
            (*spc).pc = ((*spc).pc as ::core::ffi::c_int
                + spc_readOpcode(spc) as int8_t as ::core::ffi::c_int)
                as uint16_t;
        }
        48 => {
            spc_doBranch(spc, spc_readOpcode(spc), (*spc).n);
        }
        52 => {
            spc_and(spc, spc_adrDpx(spc));
        }
        53 => {
            spc_and(spc, spc_adrAbx(spc));
        }
        54 => {
            spc_and(spc, spc_adrAby(spc));
        }
        55 => {
            spc_and(spc, spc_adrIdy(spc));
        }
        56 => {
            let mut src_3: uint16_t = 0 as uint16_t;
            let mut dst_3: uint16_t = spc_adrDpImm(spc, &raw mut src_3);
            spc_andm(spc, dst_3, src_3);
        }
        57 => {
            let mut src_4: uint16_t = 0 as uint16_t;
            let mut dst_4: uint16_t = spc_adrIndInd(spc, &raw mut src_4);
            spc_andm(spc, dst_4, src_4);
        }
        58 => {
            let mut low_0: uint16_t = 0 as uint16_t;
            let mut high_0: uint16_t = spc_adrDpWord(spc, &raw mut low_0);
            let mut value_0: uint16_t = (spc_readWord(spc, low_0, high_0) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint16_t;
            (*spc).z = value_0 as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
            (*spc).n = value_0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0;
            spc_writeWord(spc, low_0, high_0, value_0);
        }
        59 => {
            spc_rol(spc, spc_adrDpx(spc));
        }
        60 => {
            let mut newC: bool = (*spc).a as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0;
            (*spc).a = (((*spc).a as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
                | (*spc).c as ::core::ffi::c_int) as uint8_t;
            (*spc).c = newC;
            spc_setZN(spc, (*spc).a);
        }
        61 => {
            (*spc).x = (*spc).x.wrapping_add(1);
            spc_setZN(spc, (*spc).x);
        }
        62 => {
            spc_cmpx(spc, spc_adrDp(spc));
        }
        63 => {
            let mut dst_5: uint16_t = spc_readOpcodeWord(spc);
            spc_pushWord(spc, (*spc).pc);
            (*spc).pc = dst_5;
        }
        64 => {
            (*spc).p = true_0 != 0;
        }
        68 => {
            spc_eor(spc, spc_adrDp(spc));
        }
        69 => {
            spc_eor(spc, spc_adrAbs(spc));
        }
        70 => {
            spc_eor(spc, spc_adrInd(spc));
        }
        71 => {
            spc_eor(spc, spc_adrIdx(spc));
        }
        72 => {
            spc_eor(spc, spc_adrImm(spc));
        }
        73 => {
            let mut src_5: uint16_t = 0 as uint16_t;
            let mut dst_6: uint16_t = spc_adrDpDp(spc, &raw mut src_5);
            spc_eorm(spc, dst_6, src_5);
        }
        74 => {
            let mut adr_5: uint16_t = 0 as uint16_t;
            let mut bit_1: uint8_t = spc_adrAbsBit(spc, &raw mut adr_5);
            (*spc).c = (*spc).c as ::core::ffi::c_int
                & (spc_read(spc, adr_5) as ::core::ffi::c_int >> bit_1 as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int)
                != 0;
        }
        75 => {
            spc_lsr(spc, spc_adrDp(spc));
        }
        76 => {
            spc_lsr(spc, spc_adrAbs(spc));
        }
        77 => {
            spc_pushByte(spc, (*spc).x);
        }
        78 => {
            let mut adr_6: uint16_t = spc_adrAbs(spc);
            let mut val_3: uint8_t = spc_read(spc, adr_6);
            let mut result_1: uint8_t = ((*spc).a as ::core::ffi::c_int
                + (val_3 as ::core::ffi::c_int ^ 0xff as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int) as uint8_t;
            spc_setZN(spc, result_1);
            spc_write(
                spc,
                adr_6,
                (val_3 as ::core::ffi::c_int & !((*spc).a as ::core::ffi::c_int)) as uint8_t,
            );
        }
        79 => {
            let mut dst_7: uint8_t = spc_readOpcode(spc);
            spc_pushWord(spc, (*spc).pc);
            (*spc).pc = (0xff00 as ::core::ffi::c_int | dst_7 as ::core::ffi::c_int) as uint16_t;
        }
        80 => {
            spc_doBranch(spc, spc_readOpcode(spc), !(*spc).v);
        }
        84 => {
            spc_eor(spc, spc_adrDpx(spc));
        }
        85 => {
            spc_eor(spc, spc_adrAbx(spc));
        }
        86 => {
            spc_eor(spc, spc_adrAby(spc));
        }
        87 => {
            spc_eor(spc, spc_adrIdy(spc));
        }
        88 => {
            let mut src_6: uint16_t = 0 as uint16_t;
            let mut dst_8: uint16_t = spc_adrDpImm(spc, &raw mut src_6);
            spc_eorm(spc, dst_8, src_6);
        }
        89 => {
            let mut src_7: uint16_t = 0 as uint16_t;
            let mut dst_9: uint16_t = spc_adrIndInd(spc, &raw mut src_7);
            spc_eorm(spc, dst_9, src_7);
        }
        90 => {
            let mut low_1: uint16_t = 0 as uint16_t;
            let mut high_1: uint16_t = spc_adrDpWord(spc, &raw mut low_1);
            let mut value_1: uint16_t = (spc_readWord(spc, low_1, high_1) as ::core::ffi::c_int
                ^ 0xffff as ::core::ffi::c_int) as uint16_t;
            let mut ya: uint16_t = ((*spc).a as ::core::ffi::c_int
                | ((*spc).y as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16_t;
            let mut result_2: ::core::ffi::c_int =
                ya as ::core::ffi::c_int + value_1 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
            (*spc).c = result_2 > 0xffff as ::core::ffi::c_int;
            (*spc).z = result_2 & 0xffff as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
            (*spc).n = result_2 & 0x8000 as ::core::ffi::c_int != 0;
        }
        91 => {
            spc_lsr(spc, spc_adrDpx(spc));
        }
        92 => {
            (*spc).c = (*spc).a as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0;
            (*spc).a = ((*spc).a as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint8_t;
            spc_setZN(spc, (*spc).a);
        }
        93 => {
            (*spc).x = (*spc).a;
            spc_setZN(spc, (*spc).x);
        }
        94 => {
            spc_cmpy(spc, spc_adrAbs(spc));
        }
        95 => {
            (*spc).pc = spc_readOpcodeWord(spc);
        }
        96 => {
            (*spc).c = false_0 != 0;
        }
        100 => {
            spc_cmp(spc, spc_adrDp(spc));
        }
        101 => {
            spc_cmp(spc, spc_adrAbs(spc));
        }
        102 => {
            spc_cmp(spc, spc_adrInd(spc));
        }
        103 => {
            spc_cmp(spc, spc_adrIdx(spc));
        }
        104 => {
            spc_cmp(spc, spc_adrImm(spc));
        }
        105 => {
            let mut src_8: uint16_t = 0 as uint16_t;
            let mut dst_10: uint16_t = spc_adrDpDp(spc, &raw mut src_8);
            spc_cmpm(spc, dst_10, src_8);
        }
        106 => {
            let mut adr_7: uint16_t = 0 as uint16_t;
            let mut bit_2: uint8_t = spc_adrAbsBit(spc, &raw mut adr_7);
            (*spc).c = (*spc).c as ::core::ffi::c_int
                & (!(spc_read(spc, adr_7) as ::core::ffi::c_int >> bit_2 as ::core::ffi::c_int)
                    & 1 as ::core::ffi::c_int)
                != 0;
        }
        107 => {
            spc_ror(spc, spc_adrDp(spc));
        }
        108 => {
            spc_ror(spc, spc_adrAbs(spc));
        }
        109 => {
            spc_pushByte(spc, (*spc).y);
        }
        110 => {
            let mut adr_8: uint16_t = spc_adrDp(spc);
            let mut result_3: uint8_t =
                (spc_read(spc, adr_8) as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint8_t;
            spc_write(spc, adr_8, result_3);
            spc_doBranch(
                spc,
                spc_readOpcode(spc),
                result_3 as ::core::ffi::c_int != 0 as ::core::ffi::c_int,
            );
        }
        111 => {
            (*spc).pc = spc_pullWord(spc);
        }
        112 => {
            spc_doBranch(spc, spc_readOpcode(spc), (*spc).v);
        }
        116 => {
            spc_cmp(spc, spc_adrDpx(spc));
        }
        117 => {
            spc_cmp(spc, spc_adrAbx(spc));
        }
        118 => {
            spc_cmp(spc, spc_adrAby(spc));
        }
        119 => {
            spc_cmp(spc, spc_adrIdy(spc));
        }
        120 => {
            let mut src_9: uint16_t = 0 as uint16_t;
            let mut dst_11: uint16_t = spc_adrDpImm(spc, &raw mut src_9);
            spc_cmpm(spc, dst_11, src_9);
        }
        121 => {
            let mut src_10: uint16_t = 0 as uint16_t;
            let mut dst_12: uint16_t = spc_adrIndInd(spc, &raw mut src_10);
            spc_cmpm(spc, dst_12, src_10);
        }
        122 => {
            let mut low_2: uint16_t = 0 as uint16_t;
            let mut high_2: uint16_t = spc_adrDpWord(spc, &raw mut low_2);
            let mut value_2: uint16_t = spc_readWord(spc, low_2, high_2);
            let mut ya_0: uint16_t = ((*spc).a as ::core::ffi::c_int
                | ((*spc).y as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16_t;
            let mut result_4: ::core::ffi::c_int =
                ya_0 as ::core::ffi::c_int + value_2 as ::core::ffi::c_int;
            (*spc).v = ya_0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == value_2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                && value_2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != result_4 & 0x8000 as ::core::ffi::c_int;
            (*spc).h = (ya_0 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int)
                + (value_2 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int
                > 0xfff as ::core::ffi::c_int;
            (*spc).c = result_4 > 0xffff as ::core::ffi::c_int;
            (*spc).z = result_4 & 0xffff as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
            (*spc).n = result_4 & 0x8000 as ::core::ffi::c_int != 0;
            (*spc).a = (result_4 & 0xff as ::core::ffi::c_int) as uint8_t;
            (*spc).y = (result_4 >> 8 as ::core::ffi::c_int) as uint8_t;
        }
        123 => {
            spc_ror(spc, spc_adrDpx(spc));
        }
        124 => {
            let mut newC_0: bool = (*spc).a as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0;
            (*spc).a = ((*spc).a as ::core::ffi::c_int >> 1 as ::core::ffi::c_int
                | ((*spc).c as ::core::ffi::c_int) << 7 as ::core::ffi::c_int)
                as uint8_t;
            (*spc).c = newC_0;
            spc_setZN(spc, (*spc).a);
        }
        125 => {
            (*spc).a = (*spc).x;
            spc_setZN(spc, (*spc).a);
        }
        126 => {
            spc_cmpy(spc, spc_adrDp(spc));
        }
        127 => {
            spc_setFlags(spc, spc_pullByte(spc));
            (*spc).pc = spc_pullWord(spc);
        }
        128 => {
            (*spc).c = true_0 != 0;
        }
        132 => {
            spc_adc(spc, spc_adrDp(spc));
        }
        133 => {
            spc_adc(spc, spc_adrAbs(spc));
        }
        134 => {
            spc_adc(spc, spc_adrInd(spc));
        }
        135 => {
            spc_adc(spc, spc_adrIdx(spc));
        }
        136 => {
            spc_adc(spc, spc_adrImm(spc));
        }
        137 => {
            let mut src_11: uint16_t = 0 as uint16_t;
            let mut dst_13: uint16_t = spc_adrDpDp(spc, &raw mut src_11);
            spc_adcm(spc, dst_13, src_11);
        }
        138 => {
            let mut adr_9: uint16_t = 0 as uint16_t;
            let mut bit_3: uint8_t = spc_adrAbsBit(spc, &raw mut adr_9);
            (*spc).c = (*spc).c as ::core::ffi::c_int
                ^ spc_read(spc, adr_9) as ::core::ffi::c_int >> bit_3 as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                != 0;
        }
        139 => {
            spc_dec(spc, spc_adrDp(spc));
        }
        140 => {
            spc_dec(spc, spc_adrAbs(spc));
        }
        141 => {
            spc_movy(spc, spc_adrImm(spc));
        }
        142 => {
            spc_setFlags(spc, spc_pullByte(spc));
        }
        143 => {
            let mut src_12: uint16_t = 0 as uint16_t;
            let mut dst_14: uint16_t = spc_adrDpImm(spc, &raw mut src_12);
            let mut val_4: uint8_t = spc_read(spc, src_12);
            spc_read(spc, dst_14);
            spc_write(spc, dst_14, val_4);
        }
        144 => {
            spc_doBranch(spc, spc_readOpcode(spc), !(*spc).c);
        }
        148 => {
            spc_adc(spc, spc_adrDpx(spc));
        }
        149 => {
            spc_adc(spc, spc_adrAbx(spc));
        }
        150 => {
            spc_adc(spc, spc_adrAby(spc));
        }
        151 => {
            spc_adc(spc, spc_adrIdy(spc));
        }
        152 => {
            let mut src_13: uint16_t = 0 as uint16_t;
            let mut dst_15: uint16_t = spc_adrDpImm(spc, &raw mut src_13);
            spc_adcm(spc, dst_15, src_13);
        }
        153 => {
            let mut src_14: uint16_t = 0 as uint16_t;
            let mut dst_16: uint16_t = spc_adrIndInd(spc, &raw mut src_14);
            spc_adcm(spc, dst_16, src_14);
        }
        154 => {
            let mut low_3: uint16_t = 0 as uint16_t;
            let mut high_3: uint16_t = spc_adrDpWord(spc, &raw mut low_3);
            let mut value_3: uint16_t = (spc_readWord(spc, low_3, high_3) as ::core::ffi::c_int
                ^ 0xffff as ::core::ffi::c_int) as uint16_t;
            let mut ya_1: uint16_t = ((*spc).a as ::core::ffi::c_int
                | ((*spc).y as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16_t;
            let mut result_5: ::core::ffi::c_int = ya_1 as ::core::ffi::c_int
                + value_3 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int;
            (*spc).v = ya_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == value_3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                && value_3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                    != result_5 & 0x8000 as ::core::ffi::c_int;
            (*spc).h = (ya_1 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int)
                + (value_3 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int)
                + 1 as ::core::ffi::c_int
                > 0xfff as ::core::ffi::c_int;
            (*spc).c = result_5 > 0xffff as ::core::ffi::c_int;
            (*spc).z = result_5 & 0xffff as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
            (*spc).n = result_5 & 0x8000 as ::core::ffi::c_int != 0;
            (*spc).a = (result_5 & 0xff as ::core::ffi::c_int) as uint8_t;
            (*spc).y = (result_5 >> 8 as ::core::ffi::c_int) as uint8_t;
        }
        155 => {
            spc_dec(spc, spc_adrDpx(spc));
        }
        156 => {
            (*spc).a = (*spc).a.wrapping_sub(1);
            spc_setZN(spc, (*spc).a);
        }
        157 => {
            (*spc).x = (*spc).sp;
            spc_setZN(spc, (*spc).x);
        }
        158 => {
            let mut value_4: uint16_t = ((*spc).a as ::core::ffi::c_int
                | ((*spc).y as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
                as uint16_t;
            let mut result_6: ::core::ffi::c_int = 0xffff as ::core::ffi::c_int;
            let mut mod_0: ::core::ffi::c_int = (*spc).a as ::core::ffi::c_int;
            if (*spc).x as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                result_6 = value_4 as ::core::ffi::c_int / (*spc).x as ::core::ffi::c_int;
                mod_0 = value_4 as ::core::ffi::c_int % (*spc).x as ::core::ffi::c_int;
            }
            (*spc).v = result_6 > 0xff as ::core::ffi::c_int;
            (*spc).h = (*spc).x as ::core::ffi::c_int & 0xf as ::core::ffi::c_int
                <= (*spc).y as ::core::ffi::c_int & 0xf as ::core::ffi::c_int;
            (*spc).a = result_6 as uint8_t;
            (*spc).y = mod_0 as uint8_t;
            spc_setZN(spc, (*spc).a);
        }
        159 => {
            (*spc).a = ((*spc).a as ::core::ffi::c_int >> 4 as ::core::ffi::c_int
                | ((*spc).a as ::core::ffi::c_int) << 4 as ::core::ffi::c_int)
                as uint8_t;
            spc_setZN(spc, (*spc).a);
        }
        160 => {
            (*spc).i = true_0 != 0;
        }
        164 => {
            spc_sbc(spc, spc_adrDp(spc));
        }
        165 => {
            spc_sbc(spc, spc_adrAbs(spc));
        }
        166 => {
            spc_sbc(spc, spc_adrInd(spc));
        }
        167 => {
            spc_sbc(spc, spc_adrIdx(spc));
        }
        168 => {
            spc_sbc(spc, spc_adrImm(spc));
        }
        169 => {
            let mut src_15: uint16_t = 0 as uint16_t;
            let mut dst_17: uint16_t = spc_adrDpDp(spc, &raw mut src_15);
            spc_sbcm(spc, dst_17, src_15);
        }
        170 => {
            let mut adr_10: uint16_t = 0 as uint16_t;
            let mut bit_4: uint8_t = spc_adrAbsBit(spc, &raw mut adr_10);
            (*spc).c = spc_read(spc, adr_10) as ::core::ffi::c_int >> bit_4 as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0;
        }
        171 => {
            spc_inc(spc, spc_adrDp(spc));
        }
        172 => {
            spc_inc(spc, spc_adrAbs(spc));
        }
        173 => {
            spc_cmpy(spc, spc_adrImm(spc));
        }
        174 => {
            (*spc).a = spc_pullByte(spc);
        }
        175 => {
            let mut adr_11: uint16_t = spc_adrIndP(spc);
            spc_write(spc, adr_11, (*spc).a);
        }
        176 => {
            spc_doBranch(spc, spc_readOpcode(spc), (*spc).c);
        }
        180 => {
            spc_sbc(spc, spc_adrDpx(spc));
        }
        181 => {
            spc_sbc(spc, spc_adrAbx(spc));
        }
        182 => {
            spc_sbc(spc, spc_adrAby(spc));
        }
        183 => {
            spc_sbc(spc, spc_adrIdy(spc));
        }
        184 => {
            let mut src_16: uint16_t = 0 as uint16_t;
            let mut dst_18: uint16_t = spc_adrDpImm(spc, &raw mut src_16);
            spc_sbcm(spc, dst_18, src_16);
        }
        185 => {
            let mut src_17: uint16_t = 0 as uint16_t;
            let mut dst_19: uint16_t = spc_adrIndInd(spc, &raw mut src_17);
            spc_sbcm(spc, dst_19, src_17);
        }
        186 => {
            let mut low_4: uint16_t = 0 as uint16_t;
            let mut high_4: uint16_t = spc_adrDpWord(spc, &raw mut low_4);
            let mut val_5: uint16_t = spc_readWord(spc, low_4, high_4);
            (*spc).a = (val_5 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t;
            (*spc).y = (val_5 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8_t;
            (*spc).z = val_5 as ::core::ffi::c_int == 0 as ::core::ffi::c_int;
            (*spc).n = val_5 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0;
        }
        187 => {
            spc_inc(spc, spc_adrDpx(spc));
        }
        188 => {
            (*spc).a = (*spc).a.wrapping_add(1);
            spc_setZN(spc, (*spc).a);
        }
        189 => {
            (*spc).sp = (*spc).x;
        }
        190 => {
            if (*spc).a as ::core::ffi::c_int > 0x99 as ::core::ffi::c_int || !(*spc).c {
                (*spc).a = ((*spc).a as ::core::ffi::c_int - 0x60 as ::core::ffi::c_int) as uint8_t;
                (*spc).c = false_0 != 0;
            }
            if (*spc).a as ::core::ffi::c_int & 0xf as ::core::ffi::c_int > 9 as ::core::ffi::c_int
                || !(*spc).h
            {
                (*spc).a = ((*spc).a as ::core::ffi::c_int - 6 as ::core::ffi::c_int) as uint8_t;
            }
            spc_setZN(spc, (*spc).a);
        }
        191 => {
            let mut adr_12: uint16_t = spc_adrIndP(spc);
            (*spc).a = spc_read(spc, adr_12);
            spc_setZN(spc, (*spc).a);
        }
        192 => {
            (*spc).i = false_0 != 0;
        }
        196 => {
            spc_movs(spc, spc_adrDp(spc));
        }
        197 => {
            spc_movs(spc, spc_adrAbs(spc));
        }
        198 => {
            spc_movs(spc, spc_adrInd(spc));
        }
        199 => {
            spc_movs(spc, spc_adrIdx(spc));
        }
        200 => {
            spc_cmpx(spc, spc_adrImm(spc));
        }
        201 => {
            spc_movsx(spc, spc_adrAbs(spc));
        }
        202 => {
            let mut adr_13: uint16_t = 0 as uint16_t;
            let mut bit_5: uint8_t = spc_adrAbsBit(spc, &raw mut adr_13);
            let mut result_7: uint8_t = (spc_read(spc, adr_13) as ::core::ffi::c_int
                & !((1 as ::core::ffi::c_int) << bit_5 as ::core::ffi::c_int)
                | ((*spc).c as ::core::ffi::c_int) << bit_5 as ::core::ffi::c_int)
                as uint8_t;
            spc_write(spc, adr_13, result_7);
        }
        203 => {
            spc_movsy(spc, spc_adrDp(spc));
        }
        204 => {
            spc_movsy(spc, spc_adrAbs(spc));
        }
        205 => {
            spc_movx(spc, spc_adrImm(spc));
        }
        206 => {
            (*spc).x = spc_pullByte(spc);
        }
        207 => {
            let mut result_8: uint16_t =
                ((*spc).a as ::core::ffi::c_int * (*spc).y as ::core::ffi::c_int) as uint16_t;
            (*spc).a = (result_8 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int) as uint8_t;
            (*spc).y = (result_8 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8_t;
            spc_setZN(spc, (*spc).y);
        }
        208 => {
            spc_doBranch(spc, spc_readOpcode(spc), !(*spc).z);
        }
        212 => {
            spc_movs(spc, spc_adrDpx(spc));
        }
        213 => {
            spc_movs(spc, spc_adrAbx(spc));
        }
        214 => {
            spc_movs(spc, spc_adrAby(spc));
        }
        215 => {
            spc_movs(spc, spc_adrIdy(spc));
        }
        216 => {
            spc_movsx(spc, spc_adrDp(spc));
        }
        217 => {
            spc_movsx(spc, spc_adrDpy(spc));
        }
        218 => {
            let mut low_5: uint16_t = 0 as uint16_t;
            let mut high_5: uint16_t = spc_adrDpWord(spc, &raw mut low_5);
            spc_read(spc, low_5);
            spc_write(spc, low_5, (*spc).a);
            spc_write(spc, high_5, (*spc).y);
        }
        219 => {
            spc_movsy(spc, spc_adrDpx(spc));
        }
        220 => {
            (*spc).y = (*spc).y.wrapping_sub(1);
            spc_setZN(spc, (*spc).y);
        }
        221 => {
            (*spc).a = (*spc).y;
            spc_setZN(spc, (*spc).a);
        }
        222 => {
            let mut val_6: uint8_t = (spc_read(spc, spc_adrDpx(spc)) as ::core::ffi::c_int
                ^ 0xff as ::core::ffi::c_int) as uint8_t;
            let mut result_9: uint8_t = ((*spc).a as ::core::ffi::c_int
                + val_6 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as uint8_t;
            spc_doBranch(
                spc,
                spc_readOpcode(spc),
                result_9 as ::core::ffi::c_int != 0 as ::core::ffi::c_int,
            );
        }
        223 => {
            if (*spc).a as ::core::ffi::c_int > 0x99 as ::core::ffi::c_int
                || (*spc).c as ::core::ffi::c_int != 0
            {
                (*spc).a = ((*spc).a as ::core::ffi::c_int + 0x60 as ::core::ffi::c_int) as uint8_t;
                (*spc).c = true_0 != 0;
            }
            if (*spc).a as ::core::ffi::c_int & 0xf as ::core::ffi::c_int > 9 as ::core::ffi::c_int
                || (*spc).h as ::core::ffi::c_int != 0
            {
                (*spc).a = ((*spc).a as ::core::ffi::c_int + 6 as ::core::ffi::c_int) as uint8_t;
            }
            spc_setZN(spc, (*spc).a);
        }
        224 => {
            (*spc).v = false_0 != 0;
            (*spc).h = false_0 != 0;
        }
        228 => {
            spc_mov(spc, spc_adrDp(spc));
        }
        229 => {
            spc_mov(spc, spc_adrAbs(spc));
        }
        230 => {
            spc_mov(spc, spc_adrInd(spc));
        }
        231 => {
            spc_mov(spc, spc_adrIdx(spc));
        }
        232 => {
            spc_mov(spc, spc_adrImm(spc));
        }
        233 => {
            spc_movx(spc, spc_adrAbs(spc));
        }
        234 => {
            let mut adr_14: uint16_t = 0 as uint16_t;
            let mut bit_6: uint8_t = spc_adrAbsBit(spc, &raw mut adr_14);
            let mut result_10: uint8_t = (spc_read(spc, adr_14) as ::core::ffi::c_int
                ^ (1 as ::core::ffi::c_int) << bit_6 as ::core::ffi::c_int)
                as uint8_t;
            spc_write(spc, adr_14, result_10);
        }
        235 => {
            spc_movy(spc, spc_adrDp(spc));
        }
        236 => {
            spc_movy(spc, spc_adrAbs(spc));
        }
        237 => {
            (*spc).c = !(*spc).c;
        }
        238 => {
            (*spc).y = spc_pullByte(spc);
        }
        239 => {
            (*spc).stopped = true_0 != 0;
        }
        240 => {
            spc_doBranch(spc, spc_readOpcode(spc), (*spc).z);
        }
        244 => {
            spc_mov(spc, spc_adrDpx(spc));
        }
        245 => {
            spc_mov(spc, spc_adrAbx(spc));
        }
        246 => {
            spc_mov(spc, spc_adrAby(spc));
        }
        247 => {
            spc_mov(spc, spc_adrIdy(spc));
        }
        248 => {
            spc_movx(spc, spc_adrDp(spc));
        }
        249 => {
            spc_movx(spc, spc_adrDpy(spc));
        }
        250 => {
            let mut src_18: uint16_t = 0 as uint16_t;
            let mut dst_20: uint16_t = spc_adrDpDp(spc, &raw mut src_18);
            let mut val_7: uint8_t = spc_read(spc, src_18);
            spc_write(spc, dst_20, val_7);
        }
        251 => {
            spc_movy(spc, spc_adrDpx(spc));
        }
        252 => {
            (*spc).y = (*spc).y.wrapping_add(1);
            spc_setZN(spc, (*spc).y);
        }
        253 => {
            (*spc).y = (*spc).a;
            spc_setZN(spc, (*spc).y);
        }
        254 => {
            (*spc).y = (*spc).y.wrapping_sub(1);
            spc_doBranch(
                spc,
                spc_readOpcode(spc),
                (*spc).y as ::core::ffi::c_int != 0 as ::core::ffi::c_int,
            );
        }
        255 => {
            (*spc).stopped = true_0 != 0;
        }
        0 | _ => {}
    };
}
