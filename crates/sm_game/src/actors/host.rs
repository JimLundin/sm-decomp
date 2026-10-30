//! What bank-derived actor code needs from every host: named and raw work
//! RAM, enemy slots, and native and loaded cartridge reads.
#![forbid(unsafe_code)]
use crate::cartridge::{LoadedImage, NativeDiagnostics};
use crate::ram::{CompatibilityRam, WorkRamAccess};

pub trait ActorHost: WorkRamAccess + CompatibilityRam + NativeDiagnostics + LoadedImage {}

impl<T: WorkRamAccess + CompatibilityRam + NativeDiagnostics + LoadedImage + ?Sized> ActorHost
    for T
{
}
