use port::{WRAM_SIZE, Wram, enemy_projectile_contacts as contacts};

fn rom() -> Vec<u8> {
    let mut rom = vec![0; 0x200000];
    for (offset, value) in [(0x3000a, 0x8123u16), (0x3000c, 0x8456), (0x1a3db4, 0x8000), (0x1a0000, 7)] {
        rom[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    rom
}

fn enemy(r: &mut Wram<'_>, k: usize, properties: u16) {
    r.set_word(0x1997 + k, 0x8000);
    r.set_word(0x1bd7 + k, properties);
    r.set_word(0x1bb3 + k, 0x0404);
    r.set_word(0x1a4b + k, 128);
    r.set_word(0x1a93 + k, 128);
}

#[test]
fn samus_detection_keeps_descending_after_invincibility_and_accumulates_damage() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xaf6, 128);
    r.set_word(0xafa, 128);
    r.set_word(0xafe, 8);
    r.set_word(0xb00, 16);
    r.set_word(0x9c2, 100);
    enemy(&mut r, 34, 0x4014);
    enemy(&mut r, 0, 30);
    r.set_word(0x1a4b + 34, 129);
    r.set_word(0x1a4b, 127);
    contacts::detect_samus(&mut r, &rom()).unwrap();
    assert_eq!(r.word(0x9c2), 50);
    assert_eq!(r.word(0x18a6), 0);
    assert_eq!(r.word(0x18a8), 96);
    assert_eq!(r.word(0xa54), 1);
    assert_eq!(r.word(0x1997 + 34), 0x8000);
    assert_eq!(r.word(0x1997), 0);
    assert_eq!(r.word(0x1b47), 0x8123);
    assert_eq!(r.word(0x1b8f + 34), 1);
    assert_eq!(r.word(0xf378), 11);
}

#[test]
fn overlap_uses_c_signed_promotion_including_zero_axis_radius() {
    for (dx, hit) in [(0u16, true), (11, true), (12, false), (0x8000, false), (0xffff, true)] {
        let mut bytes = vec![0; WRAM_SIZE];
        let mut r = Wram::new(&mut bytes).unwrap();
        enemy(&mut r, 34, 0x4001);
        r.set_word(0xaf6, 128u16.wrapping_add(dx));
        r.set_word(0xafa, 128);
        r.set_word(0xafe, 8);
        r.set_word(0xb00, 1);
        r.set_word(0x1bb3 + 34, 4); // Zero y radius, still overlaps inside Samus.
        contacts::detect_samus(&mut r, &rom()).unwrap();
        assert_eq!(r.word(0x18a8) == 96, hit, "dx={dx}");
    }
}

#[test]
fn suit_division_precedes_reserved_damage_and_frozen_health_gate() {
    for (amount, items, frozen, health, expected) in [
        (1200, 0x21, 0, 1000, 1000), // Gravity takes precedence, producing reserved300.
        (600, 1, 0, 1000, 1000),
        (300, 0, 0, 1000, 1000),
        (301, 0, 1, 1000, 1000),
        (301, 0, 0, 100, 0),
        (0, 0, 0, 0xffff, 0), // Native signed-health clamp even with zero damage.
    ] {
        let mut bytes = vec![0; WRAM_SIZE];
        let mut r = Wram::new(&mut bytes).unwrap();
        enemy(&mut r, 0, amount);
        r.set_word(0x9a2, items);
        r.set_word(0xa78, frozen);
        r.set_word(0x9c2, health);
        contacts::hit_samus(&mut r, &rom(), 0).unwrap();
        assert_eq!(r.word(0x9c2), expected);
        assert_eq!(r.word(0x18aa), 5);
        assert_eq!(r.word(0x1997), 0);
    }
}

#[test]
fn zero_hit_list_preserves_instruction_state_and_entry_gates_preserve_collision_index() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    enemy(&mut r, 0, 0x4001);
    r.set_word(0x1b47, 0xabcd);
    r.set_word(0x1b8f, 0x1234);
    contacts::hit_samus(&mut r, &[0; 0x40000], 0).unwrap();
    assert_eq!(r.word(0x1b47), 0xabcd);
    assert_eq!(r.word(0x1b8f), 0x1234);
    r.set_word(0x18a6, 0xdead);
    contacts::detect_samus(&mut r, &[]).unwrap();
    assert_eq!(r.word(0x18a6), 0xdead);
    assert_eq!(r.word(0xf378), 11);
}

#[test]
fn projectile_detection_keeps_ascending_after_properties_are_cleared() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    enemy(&mut r, 34, 0xf123);
    r.set_word(0xcce, 1);
    for (j, kind) in [(0, 0x101), (2, 0x208), (4, 0x300), (6, 0x500), (8, 0x601)] {
        r.set_word(0xc18 + j, kind);
        r.set_word(0xb64 + j, 159); // Same32px cell, no radius collision required.
        r.set_word(0xb78 + j, 128);
    }
    contacts::detect_projectiles(&mut r, &rom()).unwrap();
    assert_eq!(r.word(0x1bfb + 34), 0x601);
    assert_eq!(r.word(0x1bd7 + 34), 0x123);
    assert_eq!(r.word(0x1a03 + 34), 0x84fb);
    assert_eq!(r.word(0x1b47 + 34), 0x8456);
    assert_eq!(r.word(0xc04), 0x10);
    assert_eq!(r.word(0xc04 + 2), 0);
    assert_eq!(r.word(0xc04 + 4), 0);
    assert_eq!(r.word(0xc04 + 6), 0);
    assert_eq!(r.word(0xc04 + 8), 0x10);
    assert_eq!(r.word(0xf378), 12);
}

#[test]
fn flag_two_blocks_detection_but_direct_handler_uses_normal_shot_branch() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    enemy(&mut r, 34, 0x8014);
    r.set_word(0xf380 + 34, 2);
    r.set_word(0xcce, 1);
    r.set_word(0xc18 + 8, 0x200);
    r.set_word(0xb64 + 8, 128);
    r.set_word(0xb78 + 8, 128);
    contacts::detect_projectiles(&mut r, &[]).unwrap();
    assert_eq!(r.word(0xc04 + 8), 0);
    assert_eq!(r.word(0x1bd7 + 34), 0x8014);
    contacts::hit_projectile(&mut r, &rom(), 34, 8).unwrap();
    assert_eq!(r.word(0xc04 + 8), 0x10);
    assert_eq!(r.word(0x1bfb + 34), 0x200);
}

#[test]
fn dud_sprite_uses_enemy_slot_wram_alias_and_sound_survives_full_sprite_pool() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    enemy(&mut r, 34, 0xfabc);
    r.set_word(0xf380 + 34, 1);
    r.set_word(0xc18 + 8, 0x108);
    r.set_word(0xb64 + 8, 777);
    r.set_word(0xb78 + 8, 888);
    r.set_word(0xb64 + 34, 0x1234);
    r.set_word(0xb78 + 34, 0xabcd);
    contacts::hit_projectile(&mut r, &rom(), 34, 8).unwrap();
    assert_eq!(r.word(0xf0f8 + 62), 0x1234);
    assert_eq!(r.word(0xf1f8 + 62), 0xabcd);
    assert_eq!(r.word(0xeff8 + 62), 7);
    assert_eq!(r.word(0xc04 + 8), 0);
    assert_eq!(r.word(0x1bd7 + 34), 0xfabc);
    assert_eq!(r.word(0x1997 + 34), 0x8000);
    assert_eq!(r.byte(0x653), 6);
    assert_eq!(r.byte(0x656), 0x3d);
    assert_eq!(r.byte(0x646), 1);
    for s in (0..64).step_by(2) {
        r.set_word(0xef78 + s, 0x8000);
    }
    contacts::hit_projectile(&mut r, &[], 34, 8).unwrap();
    assert_eq!(r.byte(0x657), 0x3d);
    assert_eq!(r.byte(0x646), 2);
}
