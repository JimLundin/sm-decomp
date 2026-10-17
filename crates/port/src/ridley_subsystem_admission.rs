//! Immutable native Ridley ownership and reachable-state validation.
use super::*;

// Compact mutation guards for a separately SHA256-authenticated original ROM.
// These FNV-1a values are not cryptographic evidence authentication. Pinning the
// complete A6 bank also covers graphics, palettes, scripts and branch tables.
const ROM_RANGES: &[(u32, usize, u64)] = &[
    (0xa68000, 32768, 0xd303ba9f559fcda8),
    (0xa0b3c3, 640, 0x79b13f548af9d4de),
    (0xa0e13f, 192, 0x361a764217fb1447),
    (0x90a83a, 110, 0x5929238e99f4c39d),
    (0x86e42c, 60, 0xb7a4447d055eed81),
    (0x86e509, 14, 0xcdd858ddb93da815),
];

pub(super) fn pinned_rom(rom: &[u8]) -> bool {
    ROM_RANGES.iter().all(|(address, size, expected)| {
        let offset = rom_offset(*address);
        rom.get(offset..offset + size).is_some_and(|bytes| {
            bytes
                .iter()
                .fold(0xcbf29ce484222325_u64, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3))
                == *expected
        })
    })
}

pub(super) fn boss_record(ram: &Wram<'_>, slot: u16) -> bool {
    record(slot).is_ok_and(|enemy| ram.byte(enemy + 0x2e) == 0xa6 && matches!(ram.word(enemy), 0xe13f | 0xe17f))
}

fn native_entry(ram: &Wram<'_>, rom: &[u8], symbol: &str) -> Result<bool> {
    let slot = ram.word(CURRENT);
    let Ok(enemy) = record(slot) else { return Ok(false) };
    if ram.byte(enemy + 0x2e) != 0xa6 {
        return Ok(false);
    }
    let header = ram.word(enemy);
    let (expected, field, pointer) = match symbol {
        "CeresRidley_Init" if matches!(header, 0xe13f | 0xe17f) => (header, 0x12, 0xa0f5),
        "CeresRidley_Main" => (0xe13f, 0x18, 0xa288),
        "CeresRidley_Hurt" => (0xe13f, 0x1c, 0xa2d3),
        "Ridley_Main" => (0xe17f, 0x18, 0xb227),
        "Ridley_Hurt" => (0xe17f, 0x1c, 0xb297),
        "Ridley_Func_2" => (0xe17f, 0x20, 0xb28a),
        "RidleysExplosion_Init" => (0xe1bf, 0x12, 0xc696),
        "RidleysExplosion_Main" => (0xe1bf, 0x18, 0xc8d4),
        "Ridley_Shot" if matches!(header, 0xe13f | 0xe17f) => (header, 0x32, 0xdf8a),
        "Ridley_Powerbomb" if matches!(header, 0xe13f | 0xe17f) => (header, 0x28, 0xdfb2),
        "sub_A6DF59" | "sub_A6DF60" | "Ridley_Func_70" | "Ridley_A2F2" | "HandleCeresRidleyGetawayCutscene" => {
            return Ok(boss_record(ram, slot) && boss_record(ram, 0));
        }
        _ => return Ok(false),
    };
    Ok(header == expected
        && rom_word(rom, 0xa00000 | (u32::from(header) + field))? == pointer
        && (header == 0xe1bf || boss_record(ram, 0)))
}

fn state(address: u16) -> bool {
    matches!(
        address,
        0xa354
            | 0xa35b
            | 0xa377
            | 0xa389
            | 0xa3df
            | 0xa455
            | 0xa478
            | 0xa6af
            | 0xa6c8
            | 0xa6e8
            | 0xa782
            | 0xa7f9
            | 0xa83c
            | 0xa84e
            | 0xa88d
            | 0xa8a4
            | 0xa8d4
            | 0xa8f8
            | 0xa923
            | 0xa947
            | 0xa971
            | 0xa9a0
            | 0xaa11
            | 0xaa4f
            | 0xaa50
            | 0xb2f3
            | 0xb321
            | 0xb3ec
            | 0xb3f8
            | 0xb441
            | 0xb455
            | 0xb493
            | 0xb4d1
            | 0xb516
            | 0xb554
            | 0xb594
            | 0xb5c4
            | 0xb5e5
            | 0xb613
            | 0xb6a7
            | 0xb6dd
            | 0xb70e
            | 0xb7b9
            | 0xbab7
            | 0xbb8f
            | 0xbbc4
            | 0xbbf1
            | 0xbc2e
            | 0xbd4e
            | 0xbd9a
            | 0xbdbc
            | 0xbdf2
            | 0xbe03
            | 0xc04e
            | 0xc538
            | 0xc53e
            | 0xc551
            | 0xc588
            | 0xc5a8
            | 0xc5c8
            | 0xc5da
            | 0xc600
    )
}

fn sound(ram: &Wram<'_>) -> bool {
    (0..3).all(|channel| ram.byte(0x643 + channel) < 16 && ram.byte(0x646 + channel) < 16)
}

fn music(ram: &Wram<'_>) -> bool {
    ram.word(0x998) >= 40 || [0x639, 0x63b].into_iter().all(|at| ram.word(at) <= 14 && ram.word(at) & 1 == 0)
}

fn graphics_pointer(pc: u16) -> bool {
    matches!(pc, 0xda71 | 0xda77 | 0xda7d | 0xda83 | 0xda89)
}

fn baby_pointer(rom: &[u8], pc: u16) -> Result<bool> {
    if !negative(pc) {
        return Ok(true);
    }
    let mut cursor = 0xbf31;
    while cursor < 0xbfc9 {
        if cursor == pc {
            return Ok(true);
        }
        let word = rom_word(rom, 0xa60000 | u32::from(cursor))?;
        cursor += 4;
        if negative(word) && !matches!(word, 0xbfc9 | 0xbfe1 | 0xbff2 | 0xbff8) {
            return Ok(false);
        }
    }
    Ok(false)
}

// Private execution does not validate an arbitrary foreign service. The real
// host must preserve native SamusCode 0/1 writes: words A28..A32 and A42/A44.
// Those pose/handler fields cannot affect subsequent Ridley branches or overlap
// its output inside the protected queue/OAM envelope. Contact and death services
// are terminal. Population init and item-drop services have nested allocation,
// queue and RNG effects, so their frames conservatively refuse until immutable
// admission can model the exact foreign continuation contract.
struct PreflightHost;
impl Host for PreflightHost {
    fn service(&mut self, ram: &mut Wram<'_>, _: &[u8], _: &mut ProjectileHost, service: Service) -> Result<u32> {
        match service {
            Service::SpawnEnemy { .. } | Service::EnemyItemDrop { .. } => {
                Err("Ridley foreign allocation continuation requires separate admission")
            }
            Service::SamusCode { code: 0 | 1 }
                if ram.word(0x330) <= 0x100 && ram.word(0x334) <= 0x100 && ram.word(0x590) <= 0x200 =>
            {
                Ok(0)
            }
            Service::SamusCode { .. } => Err("Ridley foreign Samus code outside reviewed contract"),
            Service::NormalTouch
            | Service::NormalShot
            | Service::NormalPowerbomb
            | Service::EnemyDeath { animation: 0, .. } => Ok(0),
            _ => Err("Ridley foreign service outside reviewed contract"),
        }
    }
    fn mode7_multiply(&mut self, sine: i16, factor: u8) -> Result<u16> {
        Ok(((i32::from(sine) * i32::from(factor as i8)) >> 8) as u16)
    }
    fn ordered_audio(&mut self, _: u8, _: u16, _: u8) -> Result<()> {
        Ok(())
    }
    fn ordered_music(&mut self, _: u16, _: u16) -> Result<()> {
        Ok(())
    }
    fn ordered_projectile(&mut self, _: u16, _: u16, _: Option<u16>) -> Result<()> {
        Ok(())
    }
}

fn envelope(ram: &Wram<'_>, rom: &[u8], symbol: &str) -> Result<bool> {
    if matches!(symbol, "RidleysExplosion_Init" | "RidleysExplosion_Main") {
        let enemy = record(ram.word(CURRENT))?;
        let parameter = ram.word(enemy + 0x3c);
        return Ok(symbol == "RidleysExplosion_Main" || parameter <= 22 && parameter & 1 == 0);
    }
    if matches!(symbol, "Ridley_Shot" | "Ridley_Powerbomb" | "sub_A6DF59" | "sub_A6DF60") {
        return Ok(true);
    }
    if symbol == "CeresRidley_Init" {
        return Ok(ram.word(0x79f) < 8 && ram.word(0x7a1) < 64 && ram.word(0x7a3) < 32 && music(ram));
    }
    if ram.word(0x79f) >= 8
        || ram.word(0x330) > 0x100
        || ram.word(0x334) > 0x100
        || ram.word(0x590) > 0x200
        || !sound(ram)
        || !music(ram)
    {
        return Ok(false);
    }
    if symbol == "HandleCeresRidleyGetawayCutscene" {
        return Ok(ram.word(0x8026) & 1 == 0 && ram.word(0x8026) <= 224);
    }
    if !state(ram.word(ENEMY + 0x30))
        || ram.word(0x7820) > 2
        || ram.word(0x780e) >= 10
        || ram.byte(0xa1f) >= 28
        || ram.word(0x7824) > 3
        || ram.word(0x2000) > 8
        || ram.word(0x783a) > 4
        || ram.word(0x783a) & 1 != 0
        || !graphics_pointer(ram.word(0x780a))
        || !baby_pointer(rom, ram.word(0x7806))?
        || !matches!(ram.word(0x8800), 0xbe9c | 0xbeb3 | 0xbeca | 0xbedc | 0xbf19)
    {
        return Ok(false);
    }
    Ok(true)
}

pub(super) fn supports(ram: &Wram<'_>, rom: &[u8], symbol: &str) -> Result<bool> {
    if symbol == "ProcessEnemyInstructions" {
        return instructions::admits(ram, rom, 4096);
    }
    if !pinned_rom(rom) || !native_entry(ram, rom, symbol)? || !envelope(ram, rom, symbol)? {
        return Ok(false);
    }
    // Replay only translated Rust on a private state with no external service,
    // hardware transaction or effect publication. This accounts for raw WRAM
    // aliases in the exact execution order after the static domain checks.
    let mut bytes = (0..crate::WRAM_SIZE).map(|at| ram.byte(at)).collect::<Vec<_>>();
    let mut copied = Wram::new(&mut bytes)?;
    let mut host = PreflightHost;
    let mut projectile = ProjectileHost::default();
    Ok(execute(&mut copied, rom, &mut projectile, &mut host, symbol).is_ok())
}
