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
