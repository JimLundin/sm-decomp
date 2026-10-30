//! Compatibility transport for one active contact; RAM aliases are offsets, never references.
#![forbid(unsafe_code)]
pub enum Displacement<'a> {
    Absent,
    Local(&'a mut i32),
    RamOffset(usize),
}
