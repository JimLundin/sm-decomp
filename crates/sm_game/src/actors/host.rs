//! What bank-derived actor code needs from every host: named and raw work
//! RAM, enemy slots, native and loaded cartridge reads, and the RNG.
#![forbid(unsafe_code)]
use crate::cartridge::{LoadedImage, NativeDiagnostics};
use crate::ram::{CompatibilityRam, WorkRamAccess};

/// The game's random number generator.
pub trait Random {
    fn next_random(&mut self) -> u16;
}

pub trait ActorHost:
    WorkRamAccess + CompatibilityRam + NativeDiagnostics + LoadedImage + Random
{
}

impl<T> ActorHost for T where
    T: WorkRamAccess + CompatibilityRam + NativeDiagnostics + LoadedImage + Random + ?Sized
{
}
