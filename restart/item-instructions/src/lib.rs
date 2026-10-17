//! Sealed Chozo missile, Morph and exposed-tank instruction families at pin578f90b.
//! Native code retains allocation, drawing services, HUD/music and presentation.
#![forbid(unsafe_code)]

mod memory;
mod morph;
mod tanks;
use memory::*;

#[cfg(any(test, feature = "fixture-support"))]
pub mod fixtures;
#[cfg(any(test, feature = "private-checks"))]
mod tests;

/// The same assertion bodies registered by ordinary Cargo tests.
#[cfg(feature = "private-checks")]
pub fn private_assertions() -> [(&'static str, fn()); 13] {
    tests::private_assertions()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Entry {
    Delete = 0,
    PreInstr = 1,
    ClearPreInstr = 2,
    Goto = 3,
    SetTimer = 4,
    GotoIfItemBitSet = 5,
    SetItemBit = 6,
    CollectAmmoMissileTank = 7,
    SetLinkReg = 8,
    Call = 9,
    Return = 10,
    ClearMusicQueueAndQueueTrack = 11,
    GotoLinkIfTriggered = 12,
    LoadItemPlmGfx = 13,
    DrawItemFrame0 = 14,
    DrawItemFrame1 = 15,
    PickupEquipmentAndShowMessage = 16,
    CollectHealthEnergyTank = 17,
    CollectHealthReserveTank = 18,
    CollectAmmoSuperMissileTank = 19,
    CollectAmmoPowerBombTank = 20,
}
impl Entry {
    pub const ALL: [Self; 21] = [
        Self::Delete,
        Self::PreInstr,
        Self::ClearPreInstr,
        Self::Goto,
        Self::SetTimer,
        Self::GotoIfItemBitSet,
        Self::SetItemBit,
        Self::CollectAmmoMissileTank,
        Self::SetLinkReg,
        Self::Call,
        Self::Return,
        Self::ClearMusicQueueAndQueueTrack,
        Self::GotoLinkIfTriggered,
        Self::LoadItemPlmGfx,
        Self::DrawItemFrame0,
        Self::DrawItemFrame1,
        Self::PickupEquipmentAndShowMessage,
        Self::CollectHealthEnergyTank,
        Self::CollectHealthReserveTank,
        Self::CollectAmmoSuperMissileTank,
        Self::CollectAmmoPowerBombTank,
    ];
    pub const fn address(self) -> u32 {
        0x840000
            | match self {
                Self::Delete => 0x86bc,
                Self::PreInstr => 0x86c1,
                Self::ClearPreInstr => 0x86ca,
                Self::Goto => 0x8724,
                Self::SetTimer => 0x874e,
                Self::GotoIfItemBitSet => 0x887c,
                Self::SetItemBit => 0x8899,
                Self::CollectAmmoMissileTank => 0x89a9,
                Self::SetLinkReg => 0x8a24,
                Self::Call => 0x8a2e,
                Self::Return => 0x8a3a,
                Self::ClearMusicQueueAndQueueTrack => 0x8bdd,
                Self::GotoLinkIfTriggered => 0xdf89,
                Self::LoadItemPlmGfx => 0x8764,
                Self::DrawItemFrame0 => 0xe04f,
                Self::DrawItemFrame1 => 0xe067,
                Self::PickupEquipmentAndShowMessage => 0x88f3,
                Self::CollectHealthEnergyTank => 0x8968,
                Self::CollectHealthReserveTank => 0x8986,
                Self::CollectAmmoSuperMissileTank => 0x89d2,
                Self::CollectAmmoPowerBombTank => 0x89fb,
            }
    }
    fn admits(self, operand: Option<u16>) -> bool {
        match self {
            Self::Delete => operand == Some(0xe4ae),
            Self::PreInstr => matches!(operand, Some(0xe48e | 0xdfb1)),
            Self::ClearPreInstr => operand == Some(0xdfc9),
            Self::Goto => matches!(operand, Some(0xe49d | 0xdfc5)),
            Self::SetTimer => operand == Some(0xe492),
            Self::GotoIfItemBitSet => operand == Some(0xe47e),
            Self::SetItemBit => operand == Some(0xe4a1),
            Self::CollectAmmoMissileTank => operand == Some(0xe4a6),
            Self::SetLinkReg => operand == Some(0xe48a),
            Self::Call => matches!(operand, Some(0xe482 | 0xe486)),
            Self::Return => operand == Some(0xdfd7),
            Self::ClearMusicQueueAndQueueTrack => operand == Some(0xe4a3),
            Self::GotoLinkIfTriggered => operand.is_none(),
            _ => false,
        }
    }
}
impl TryFrom<u32> for Entry {
    type Error = String;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::ALL.get(value as usize).copied().ok_or_else(|| format!("unknown item instruction entry {value}"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    PreinstructionDone,
    Stop,
    Continue(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Service {
    AddMissilesToHud = 0,
    PlayRoomMusicAfter = 1,
    QueueMusicDelayed8 = 2,
    ProcessPlmDrawInstruction = 3,
    CalculatePlmBlockCoords = 4,
    DrawPlm = 5,
    AddSuperMissilesToHud = 6,
    AddPowerBombsToHud = 7,
}

/// Availability checks are nonmutating. Each service receives the same private
/// invocation WRAM after all preceding body/service effects. A service error is
/// fatal; the caller must never retry the owned body through C.
pub trait Services {
    fn available(&self, service: Service) -> bool;
    fn add_missiles_to_hud(&mut self, ram: &mut [u8]) -> Result<(), String>;
    fn play_room_music_after(&mut self, ram: &mut [u8], frames: u16) -> Result<(), String>;
    fn queue_music_delayed8(&mut self, ram: &mut [u8], track: u16) -> Result<(), String>;
    fn process_plm_draw_instruction(&mut self, ram: &mut [u8], slot: u16) -> Result<(), String>;
    fn calculate_plm_block_coords(&mut self, ram: &mut [u8], slot: u16) -> Result<(), String>;
    fn draw_plm(&mut self, ram: &mut [u8], slot: u16) -> Result<(), String>;
    fn add_super_missiles_to_hud(&mut self, ram: &mut [u8]) -> Result<(), String>;
    fn add_power_bombs_to_hud(&mut self, ram: &mut [u8]) -> Result<(), String>;
}

/// Execute the selected exact header/argument domains at sealed sites. Safe foreign
/// identities refuse without effects; malformed identity-local requests fail
/// before any write/service. The native adapter owns transaction commit.
pub fn execute(
    entry: Entry,
    slot: u16,
    operand: Option<u16>,
    ram: &mut [u8],
    rom: &[u8],
    services: &mut dyn Services,
) -> Result<Option<Output>, String> {
    if ram.len() != WRAM_SIZE {
        return Err("item instruction requires complete WRAM".into());
    }
    if slot > 78 || slot & 1 != 0 {
        return Err(format!("invalid PLM byte slot {slot}"));
    }
    let k = usize::from(slot);
    let header = word(ram, HEADER + k);
    let argument = word(ram, ARGUMENT + k);
    let tank = tanks::Domain::from_header(header);
    if tank.is_some() {
        if argument & 0x8000 != 0 {
            return Ok(None);
        }
        if argument > 0x1ff {
            return Err("tank item id exceeds permanent item map".into());
        }
    }
    let is_morph = match (header, argument) {
        (0xef2f, 0x22) => false,
        (0xef23, 0x1a) => true,
        _ if tank.is_some() => false,
        _ => return Ok(None),
    };
    let admitted = if let Some(domain) = tank {
        domain.admits(entry, operand)
    } else if is_morph {
        morph::admits(entry, operand)
    } else {
        entry.admits(operand)
    };
    if !admitted {
        return Err(format!("unadmitted item instruction site {entry:?}/{operand:?}"));
    }
    if let Some(domain) = tank {
        domain.validate_rom(rom)?;
    } else if is_morph {
        morph::validate_rom(rom)?;
    } else {
        validate_rom(rom)?;
    }
    if entry == Entry::Return && word(ram, LINK + k) != 0xe488 {
        return Err("item burst return link is not E488".into());
    }
    if entry == Entry::GotoLinkIfTriggered {
        let valid_pair = if let Some(domain) = tank {
            domain.valid_trigger(word(ram, LINK + k), word(ram, IP + k))
        } else if is_morph {
            word(ram, LINK + k) == 0xe40f && matches!(word(ram, IP + k), 0xe409 | 0xe40b)
        } else {
            matches!(
                (word(ram, LINK + k), word(ram, IP + k)),
                (0xe484, 0xdfb7 | 0xdfbb | 0xdfbf | 0xdfc3) | (0xe49f, 0xe497 | 0xe49b)
            )
        };
        if word(ram, PRE + k) != 0xdf89 || !valid_pair {
            return Err("item preinstruction/link/instruction identity differs".into());
        }
    }
    let required: &[Service] = match entry {
        Entry::CollectAmmoMissileTank => &[Service::AddMissilesToHud, Service::PlayRoomMusicAfter],
        Entry::ClearMusicQueueAndQueueTrack => &[Service::QueueMusicDelayed8],
        Entry::PickupEquipmentAndShowMessage => &[Service::PlayRoomMusicAfter],
        Entry::CollectHealthEnergyTank | Entry::CollectHealthReserveTank => &[Service::PlayRoomMusicAfter],
        Entry::CollectAmmoSuperMissileTank => &[Service::AddSuperMissilesToHud, Service::PlayRoomMusicAfter],
        Entry::CollectAmmoPowerBombTank => &[Service::AddPowerBombsToHud, Service::PlayRoomMusicAfter],
        Entry::DrawItemFrame0 | Entry::DrawItemFrame1 => {
            &[Service::ProcessPlmDrawInstruction, Service::CalculatePlmBlockCoords, Service::DrawPlm]
        }
        _ => &[],
    };
    if matches!(
        entry,
        Entry::CollectAmmoMissileTank
            | Entry::ClearMusicQueueAndQueueTrack
            | Entry::PickupEquipmentAndShowMessage
            | Entry::CollectHealthEnergyTank
            | Entry::CollectHealthReserveTank
            | Entry::CollectAmmoSuperMissileTank
            | Entry::CollectAmmoPowerBombTank
    ) {
        for at in [0x639, 0x63b] {
            let position = word(ram, at);
            if position > 14 || position & 1 != 0 {
                return Err("invalid native music queue position".into());
            }
        }
    }
    for &service in required {
        if !services.available(service) {
            return Err(format!("required native item service unavailable: {service:?}"));
        }
    }
    morph::preflight(entry, slot, ram)?;
    // Every admission, ROM read extent and service requirement is now checked.
    let p = operand.unwrap_or(0);
    let (item_byte, item_mask) = tanks::item_bit(argument);
    let result = match entry {
        Entry::Delete => {
            set_word(ram, HEADER + k, 0);
            Output::Stop
        }
        Entry::PreInstr => {
            set_word(ram, PRE + k, word(rom, rom_index(p)));
            Output::Continue(p + 2)
        }
        Entry::ClearPreInstr => {
            set_word(ram, PRE + k, 0x86d0);
            Output::Continue(p)
        }
        Entry::Goto => Output::Continue(word(rom, rom_index(p))),
        Entry::SetTimer => {
            ram[TIMER + k] = rom[rom_index(p)];
            Output::Continue(p + 1)
        }
        Entry::GotoIfItemBitSet => {
            set_word(ram, 0x5e7, u16::from(item_mask));
            Output::Continue(if ram[item_byte] & item_mask != 0 { word(rom, rom_index(p)) } else { p + 2 })
        }
        Entry::SetItemBit => {
            set_word(ram, 0x5e7, u16::from(item_mask));
            ram[item_byte] |= item_mask;
            Output::Continue(p)
        }
        Entry::CollectAmmoMissileTank => {
            let amount = word(rom, rom_index(p));
            let capacity = word(ram, 0x9c8).wrapping_add(amount);
            let ammo = word(ram, 0x9c6).wrapping_add(amount);
            set_word(ram, 0x9c8, capacity);
            set_word(ram, 0x9c6, ammo);
            services.add_missiles_to_hud(ram)?;
            services.play_room_music_after(ram, 0x168)?;
            set_word(ram, 0xdc8, 2);
            Output::Continue(p + 2)
        }
        Entry::SetLinkReg => {
            set_word(ram, LINK + k, word(rom, rom_index(p)));
            Output::Continue(p + 2)
        }
        Entry::Call => {
            set_word(ram, LINK + k, p + 2);
            Output::Continue(word(rom, rom_index(p)))
        }
        Entry::Return => Output::Continue(word(ram, LINK + k)),
        Entry::ClearMusicQueueAndQueueTrack => {
            for i in (0..8).rev() {
                set_word(ram, 0x619 + 2 * i, 0);
                set_word(ram, 0x629 + 2 * i, 0);
            }
            let write_position = word(ram, 0x639);
            set_word(ram, 0x63b, write_position);
            set_word(ram, 0x63f, 0);
            set_word(ram, 0x63d, 0);
            services.queue_music_delayed8(ram, u16::from(rom[rom_index(p)]))?;
            Output::Continue(p + 1)
        }
        Entry::GotoLinkIfTriggered => {
            if ram[TIMER + k] == 0xff {
                let link = word(ram, LINK + k);
                set_word(ram, PRE + k, 0xdfa8);
                set_word(ram, IP + k, link);
                set_word(ram, INSTRUCTION_TIMER + k, 1);
            }
            Output::PreinstructionDone
        }
        Entry::LoadItemPlmGfx => morph::load_gfx(slot, p, ram, rom),
        Entry::DrawItemFrame0 | Entry::DrawItemFrame1 => morph::draw_frame(entry, slot, p, ram, rom, services)?,
        Entry::PickupEquipmentAndShowMessage => {
            let equipment = word(rom, rom_index(p));
            let equipped = word(ram, 0x9a2) | equipment;
            set_word(ram, 0x9a2, equipped);
            let collected = word(ram, 0x9a4) | equipment;
            set_word(ram, 0x9a4, collected);
            services.play_room_music_after(ram, 0x168)?;
            set_word(ram, 0xdc8, u16::from(rom[rom_index(p + 2)]));
            Output::Continue(p + 3)
        }
        Entry::CollectHealthEnergyTank
        | Entry::CollectHealthReserveTank
        | Entry::CollectAmmoSuperMissileTank
        | Entry::CollectAmmoPowerBombTank => tanks::collect(entry, p, ram, rom, services)?,
    };
    Ok(Some(result))
}
