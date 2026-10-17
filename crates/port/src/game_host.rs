//! Shared synchronous host interface for complete scheduled gameplay modules.
//!
//! Canonical WRAM remains private until completion. A rejected invocation with
//! no host request permits native fallback. Once a request starts, failure is
//! owned by Rust: the caller must terminate instead of repeating gameplay in C.
use crate::{WRAM_SIZE, Wram, projectile_special, room_scroll, samus_projectile_frame};

pub type Result<T> = core::result::Result<T, &'static str>;

/// Dependencies whose gameplay or hardware ownership remains with the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// Samus shot/bomb reactions have no collision displacement pointer.
    SpawnPlm { header: u16 },
    /// Native bank88 allocation; DMAP=0x40 and BBAD=0x28/0x29.
    SpawnHdma { script: u16 },
    /// Native bank80 $8C83, completed before preparing the next room column.
    FlushVram,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response {
    Plm(u8),
    Hdma(u16),
    VramFlushed,
}

/// A request runs synchronously against the supplied latest full WRAM.
/// Publish WRAM before entering native behavior and refresh it on return;
/// preserve ordered PPU/DMA effects and native callback write footprints.
/// A failing request may already have effects and always ends Rust ownership.
pub trait Host {
    fn request(&mut self, ram: &mut Wram<'_>, request: Request) -> Result<Response>;
}

/// Complete scheduled calls, with their internal routine dispatch kept private.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    SamusProjectiles,
    SamusPlayerDraw,
    SamusElevatedDraw,
    SamusExplosionDraw,
    RoomDisplay,
    RoomScroll,
    RoomLayer2Scroll,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Completed {
        value: u32,
        requests: usize,
    },
    /// Caller WRAM is unchanged and no host request started; C may own the call.
    Refused {
        reason: &'static str,
    },
    /// No native fallback is allowed, including when the first request failed.
    OwnedFailure {
        reason: &'static str,
        requests: usize,
    },
}

struct Bridge<'a, H> {
    host: &'a mut H,
    requests: usize,
}

impl<H: Host> Bridge<'_, H> {
    fn request(&mut self, ram: &mut Wram<'_>, request: Request) -> Result<Response> {
        // Count the attempt before entering the adapter: failure can follow
        // native publication or a hardware effect inside that first request.
        self.requests += 1;
        self.host.request(ram, request)
    }
}

impl<H: Host> projectile_special::Host for Bridge<'_, H> {
    fn spawn_plm(&mut self, ram: &mut Wram<'_>, header: u16) -> Result<u8> {
        match self.request(ram, Request::SpawnPlm { header })? {
            Response::Plm(value) => Ok(value),
            _ => Err("game host PLM response kind differs"),
        }
    }
}

impl<H: Host> samus_projectile_frame::Host for Bridge<'_, H> {
    fn spawn_hdma(&mut self, ram: &mut Wram<'_>, script: u16) -> Result<u16> {
        match self.request(ram, Request::SpawnHdma { script })? {
            Response::Hdma(value) => Ok(value),
            _ => Err("game host HDMA response kind differs"),
        }
    }
}

impl<H: Host> room_scroll::Host for Bridge<'_, H> {
    fn flush_vram(&mut self, ram: &mut Wram<'_>) -> Result<()> {
        match self.request(ram, Request::FlushVram)? {
            Response::VramFlushed => Ok(()),
            _ => Err("game host VRAM response kind differs"),
        }
    }
}

fn invoke(ram: &mut Wram<'_>, rom: &[u8], bridge: &mut impl samus_projectile_frame::Host, entry: Entry) -> Result<u32> {
    let name = match entry {
        Entry::SamusProjectiles => "HandleProjectile",
        Entry::SamusPlayerDraw => "DrawPlayerExplosions2",
        Entry::SamusElevatedDraw => "sub_9382FD",
        Entry::SamusExplosionDraw => "DrawBombAndProjectileExplosions",
        _ => return Err("game host entry is outside Samus module"),
    };
    samus_projectile_frame::dispatch_host(ram, rom, bridge, name, 0, 0)
}

/// Execute the existing module once; never run a speculative native callback.
/// Full WRAM copies preserve integer aliases and transactional refusal rules.
pub fn execute(ram: &mut Wram<'_>, rom: &[u8], host: &mut impl Host, entry: Entry) -> Outcome {
    let mut bytes = (0..WRAM_SIZE).map(|i| ram.byte(i)).collect::<Vec<_>>();
    let mut bridge = Bridge { host, requests: 0 };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut state = Wram::new(&mut bytes)?;
        match entry {
            Entry::RoomDisplay => {
                room_scroll::dispatch_host(&mut state, rom, &mut bridge, "DisplayViewablePartOfRoom", 0, 0)
            }
            Entry::RoomScroll => room_scroll::dispatch_rom(&mut state, rom, "UpdateScrollVarsUpdateMap", 0, 0),
            Entry::RoomLayer2Scroll => {
                room_scroll::dispatch_rom(&mut state, rom, "CalculateLayer2PosAndScrollsWhenScrolling", 0, 0)
            }
            _ => invoke(&mut state, rom, &mut bridge, entry),
        }
    }));
    match result {
        Ok(Ok(value)) => {
            for (i, byte) in bytes.into_iter().enumerate() {
                ram.set_byte(i, byte);
            }
            Outcome::Completed { value, requests: bridge.requests }
        }
        Ok(Err(reason)) if bridge.requests == 0 => Outcome::Refused { reason },
        Ok(Err(reason)) => Outcome::OwnedFailure { reason, requests: bridge.requests },
        Err(_) => Outcome::OwnedFailure { reason: "game host invocation panicked", requests: bridge.requests },
    }
}

/// Compatibility adapter for existing actual-C fixture declarations.
/// Live callers use typed `execute` and must distinguish refusal from failure.
pub fn dispatch_host(
    ram: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    symbol: &str,
    a: i32,
    b: i32,
) -> Result<u32> {
    if a != 0 || b != 0 {
        return Err("game host scheduled calls take no arguments");
    }
    let entry = match symbol {
        "HandleProjectile" => Entry::SamusProjectiles,
        "DrawPlayerExplosions2" => Entry::SamusPlayerDraw,
        "sub_9382FD" => Entry::SamusElevatedDraw,
        "DrawBombAndProjectileExplosions" => Entry::SamusExplosionDraw,
        "DisplayViewablePartOfRoom" => Entry::RoomDisplay,
        "UpdateScrollVarsUpdateMap" => Entry::RoomScroll,
        "CalculateLayer2PosAndScrollsWhenScrolling" => Entry::RoomLayer2Scroll,
        _ => return Err("unknown game host scheduled call"),
    };
    match execute(ram, rom, host, entry) {
        Outcome::Completed { value, .. } => Ok(value),
        Outcome::Refused { reason } | Outcome::OwnedFailure { reason, .. } => Err(reason),
    }
}
