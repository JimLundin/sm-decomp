//! Exact exposed-tank domains and collectors; drawing reuses the item helpers.
use crate::{Entry, Output, Services, memory::*};

#[derive(Clone, Copy)]
pub(crate) struct Domain {
    pub(crate) start: u16,
    pub(crate) collector: Entry,
    reserve: bool,
}

impl Domain {
    pub(crate) fn from_header(header: u16) -> Option<Self> {
        let (start, collector, reserve) = match header {
            0xeed7 => (0xe099, Entry::CollectHealthEnergyTank, false),
            0xeedb => (0xe0be, Entry::CollectAmmoMissileTank, false),
            0xeedf => (0xe0e3, Entry::CollectAmmoSuperMissileTank, false),
            0xeee3 => (0xe108, Entry::CollectAmmoPowerBombTank, false),
            0xef27 => (0xe41d, Entry::CollectHealthReserveTank, true),
            _ => return None,
        };
        Some(Self { start, collector, reserve })
    }

    pub(crate) fn admits(self, entry: Entry, operand: Option<u16>) -> bool {
        let s = self.start;
        if entry == Entry::GotoLinkIfTriggered {
            return operand.is_none();
        }
        let Some(p) = operand else {
            return false;
        };
        if entry == Entry::Delete {
            return p == 0xdfaf;
        }
        if self.reserve {
            match entry {
                Entry::LoadItemPlmGfx => p == s + 2,
                Entry::GotoIfItemBitSet => p == s + 14,
                Entry::SetLinkReg => p == s + 18,
                Entry::PreInstr => p == s + 22,
                Entry::DrawItemFrame0 => p == s + 26,
                Entry::DrawItemFrame1 => p == s + 28,
                Entry::Goto => p == s + 30 || p == s + 43,
                Entry::SetItemBit => p == s + 34,
                Entry::ClearMusicQueueAndQueueTrack => p == s + 36,
                Entry::CollectHealthReserveTank => p == s + 39,
                _ => false,
            }
        } else {
            match entry {
                Entry::GotoIfItemBitSet => p == s + 2,
                Entry::SetLinkReg => p == s + 6,
                Entry::PreInstr => p == s + 10,
                Entry::Goto => p == s + 22 || p == s + 35,
                Entry::SetItemBit => p == s + 26,
                Entry::ClearMusicQueueAndQueueTrack => p == s + 28,
                _ => entry == self.collector && p == s + 31,
            }
        }
    }

    pub(crate) fn valid_trigger(self, link: u16, ip: u16) -> bool {
        let (pickup, first, second) = if self.reserve { (32, 26, 28) } else { (24, 16, 20) };
        link == self.start + pickup && (ip == self.start + first || ip == self.start + second)
    }

    pub(crate) fn validate_rom(self, rom: &[u8]) -> Result<(), String> {
        let script: Vec<u8> = if self.reserve {
            vec![
                0x64, 0x87, 0x00, 0x90, 0, 0, 0, 0, 0, 0, 0, 0, 0x7c, 0x88, 0x46, 0xe4, 0x24, 0x8a, 0x3d, 0xe4, 0xc1,
                0x86, 0x89, 0xdf, 0x4f, 0xe0, 0x67, 0xe0, 0x24, 0x87, 0x35, 0xe4, 0x99, 0x88, 0xdd, 0x8b, 0x02, 0x86,
                0x89, 0x64, 0, 0x24, 0x87, 0xa9, 0xdf,
            ]
        } else {
            // Exact common layout with only source-bound header parameters.
            let mut bytes = Vec::with_capacity(37);
            let draw = match self.start {
                0xe099 => 0xa2df_u16,
                0xe0be => 0xa2eb,
                0xe0e3 => 0xa2f7,
                _ => 0xa303,
            };
            for value in [
                0x887c,
                self.start + 33,
                0x8a24,
                self.start + 24,
                0x86c1,
                0xdf89,
                4,
                draw,
                4,
                draw + 6,
                0x8724,
                self.start + 12,
                0x8899,
                0x8bdd,
            ] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.push(2);
            bytes.extend((self.collector.address() as u16).to_le_bytes());
            bytes.extend((if self.start == 0xe099 { 100_u16 } else { 5 }).to_le_bytes());
            bytes.extend([0x24, 0x87, 0xa9, 0xdf]);
            bytes
        };
        let start = rom_index(self.start);
        if rom.get(start..start + script.len()) != Some(script.as_slice()) {
            return Err(format!("tank bank84 script {:04x} differs or is truncated", self.start));
        }
        let cleanup = rom_index(0xdfa9);
        if rom.get(cleanup..cleanup + 6) != Some(&[1, 0, 0xb5, 0xa2, 0xbc, 0x86][..]) {
            return Err("tank cleanup script differs or is truncated".into());
        }
        if self.reserve {
            crate::morph::validate_graphics_rom(rom)?;
        }
        Ok(())
    }
}

/// Admission has bounded the argument to the 64-byte permanent item map.
pub(crate) fn item_bit(argument: u16) -> (usize, u8) {
    (0xd870 + usize::from(argument >> 3), 1_u8 << (argument & 7))
}

pub(crate) fn collect(
    entry: Entry,
    operand: u16,
    ram: &mut [u8],
    rom: &[u8],
    services: &mut dyn Services,
) -> Result<Output, String> {
    let amount = word(rom, rom_index(operand));
    let message = match entry {
        Entry::CollectHealthEnergyTank => {
            let maximum = word(ram, 0x9c4).wrapping_add(amount);
            set_word(ram, 0x9c4, maximum);
            set_word(ram, 0x9c2, maximum);
            1
        }
        Entry::CollectHealthReserveTank => {
            let maximum = word(ram, 0x9d4).wrapping_add(amount);
            set_word(ram, 0x9d4, maximum);
            if word(ram, 0x9c0) == 0 {
                set_word(ram, 0x9c0, 1);
            }
            0x19
        }
        Entry::CollectAmmoSuperMissileTank | Entry::CollectAmmoPowerBombTank => {
            let (current, capacity) =
                if entry == Entry::CollectAmmoSuperMissileTank { (0x9ca, 0x9cc) } else { (0x9ce, 0x9d0) };
            let new_capacity = word(ram, capacity).wrapping_add(amount);
            set_word(ram, capacity, new_capacity);
            let new_current = word(ram, current).wrapping_add(amount);
            set_word(ram, current, new_current);
            if entry == Entry::CollectAmmoSuperMissileTank {
                services.add_super_missiles_to_hud(ram)?;
                3
            } else {
                services.add_power_bombs_to_hud(ram)?;
                4
            }
        }
        _ => unreachable!("only new tank collectors call collect"),
    };
    services.play_room_music_after(ram, 0x168)?;
    set_word(ram, 0xdc8, message);
    Ok(Output::Continue(operand + 2))
}
