use port::{WRAM_SIZE, Wram, samus_control as control};
fn rom() -> Vec<u8> {
    let mut rom = vec![0; 0x100000];
    for (address, bytes) in [(0x9b9400u32, vec![0x11; 32]), (0x9b9520, vec![0x22; 32]), (0x9b9800, vec![0x33; 32])] {
        let off = (((address >> 16) << 15) | (address & 0x7fff)) as usize & 0x3fffff;
        rom[off..off + bytes.len()].copy_from_slice(&bytes);
    }
    rom
}
#[test]
fn cancelling_momentum_restores_equipped_suit_palette_and_releases_echoes() {
    let rom = rom();
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xb3c, 1);
    ram.set_word(0xb3e, 0x401);
    ram.set_word(0x9a2, 0x20);
    ram.set_byte(0xa1e, 4);
    assert_eq!(control::dispatch_rom(&mut ram, &rom, "Samus_CancelSpeedBoost", 0, 0), Ok(0));
    assert_eq!(ram.word(0xb3c), 0);
    assert_eq!(ram.word(0xb3e), 0);
    assert_eq!(ram.word(0xc180), 0x3333);
    assert_eq!(ram.word(0xaae), 0xffff);
    assert_eq!(ram.word(0xac0), 0xfff8);
}
#[test]
fn transition_table_matches_configured_buttons_and_preserves_running_momentum() {
    let mut rom = rom();
    let off = |a: u32| (((a >> 16) << 15) | (a & 0x7fff)) as usize & 0x3fffff;
    rom[off(0x919ee4)..off(0x919ee4) + 2].copy_from_slice(&0x9000u16.to_le_bytes());
    for (n, v) in [0x80u16, 0x200, 3, 0xffff, 0, 0].iter().enumerate() {
        rom[off(0x919000) + n * 2..off(0x919000) + n * 2 + 2].copy_from_slice(&v.to_le_bytes());
    }
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xa1c, 1);
    ram.set_byte(0xa1f, 1);
    ram.set_word(0x8b, 0x200);
    ram.set_word(0x8f, 0x8000);
    ram.set_word(0x9b4, 0x8000);
    ram.set_word(0xa56, 7);
    assert_eq!(control::dispatch_rom(&mut ram, &rom, "Samus_LookupTransitionTable", 0, 0), Ok(0));
    assert_eq!(ram.word(0xa28), 3);
    assert_eq!(ram.word(0xa56), 0);
    ram.set_word(0xb46, 1);
    ram.set_word(0x8b, 0);
    assert_eq!(control::dispatch_rom(&mut ram, &rom, "Samus_Pose_CancelGrapple", 0, 0), Ok(0));
    assert_eq!(ram.word(0xa28), 1);
    assert_eq!(ram.word(0xa2e), 1);
}
#[test]
fn entering_water_spawns_splash_records_and_queues_the_entry_sound() {
    let mut rom = rom();
    let off = (((0x91b637u32 >> 16) << 15) | (0x91b637 & 0x7fff)) as usize & 0x3fffff;
    rom[off] = 21;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xa1c, 1);
    ram.set_word(0xafa, 100);
    ram.set_word(0xaf6, 50);
    ram.set_word(0x195e, 110);
    ram.set_word(0x5b6, 1);
    assert_eq!(control::dispatch_rom(&mut ram, &rom, "Samus_Animate_WaterFx", 0, 0), Ok(0));
    assert_eq!(ram.word(0xad2), 1);
    assert_eq!(ram.word(0xa9c), 3);
    assert_eq!(ram.word(0xaec), 0x100);
    assert_eq!(ram.word(0xadc), 54);
    assert_eq!(ram.word(0xae4), 116);
    assert_eq!(ram.byte(0x666), 13);
}
#[test]
fn animation_bytecode_changes_pose_and_loops_with_buffered_delay() {
    let mut rom = rom();
    let off = |a: u32| (((a >> 16) << 15) | (a & 0x7fff)) as usize & 0x3fffff;
    rom[off(0x91b012)..off(0x91b012) + 2].copy_from_slice(&0x9000u16.to_le_bytes());
    rom[off(0x919000)..off(0x919000) + 4].copy_from_slice(&[4, 0x8d, 0x42, 0]);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xa1c, 1);
    ram.set_word(0xa94, 1);
    assert_eq!(control::dispatch_rom(&mut ram, &rom, "Samus_Animate", 0, 0), Ok(0));
    assert_eq!(ram.word(0xa2c), 0x42);
    assert_eq!(ram.word(0xa32), 3);
    rom[off(0x919001)] = 0x8f;
    ram.set_word(0xa96, 0);
    ram.set_word(0xa94, 1);
    ram.set_word(0xa66, 2);
    assert_eq!(control::dispatch_rom(&mut ram, &rom, "Samus_Animate", 0, 0), Ok(0));
    assert_eq!(ram.word(0xa96), 0);
    assert_eq!(ram.word(0xa94), 6);
}
#[test]
fn speed_booster_animation_advances_stage_and_keeps_its_frame_timer() {
    let mut rom = rom();
    let off = |a: u32| (((a >> 16) << 15) | (a & 0x7fff)) as usize & 0x3fffff;
    for (address, value) in [(0x91b012u32, 0x9000u16), (0x91b5e0, 0x9100), (0x91b621, 2)] {
        rom[off(address)..off(address) + 2].copy_from_slice(&value.to_le_bytes());
    }
    rom[off(0x919001)] = 0x8f;
    rom[off(0x919100)] = 3;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0xa1c, 1);
    ram.set_byte(0xa1f, 1);
    ram.set_word(0x9a2, 0x2000);
    ram.set_word(0xb3c, 1);
    ram.set_word(0xb3e, 1);
    ram.set_word(0xa94, 1);
    ram.set_word(0x8b, 0x8000);
    ram.set_word(0x9b6, 0x8000);
    assert_eq!(control::dispatch_rom(&mut ram, &rom, "Samus_Animate", 0, 0), Ok(0));
    assert_eq!(ram.word(0xb3e), 0x102);
    assert_eq!(ram.word(0xa96), 0);
    assert_eq!(ram.word(0xa94), 3);
}
