use port::{WRAM_SIZE, Wram, enemy_projectiles::ProjectileHost, mother_brain_bomb};

#[test]
fn allocation_preserves_scratch_high_byte_and_wraps_head_position_and_counter() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1a27 + 34, 0xa555);
    r.set_word(0xfba, 0xfff8);
    r.set_word(0xfbe, 0xfff0);
    r.set_word(0x784a, 0xffff);
    mother_brain_bomb::initialize(&mut r, 34, 0xbbaa).unwrap();
    assert_eq!(r.word(0x1a27 + 34), 0xa5aa);
    assert_eq!(r.word(0x1a4b + 34), 4);
    assert_eq!(r.word(0x1a93 + 34), 0);
    assert_eq!(r.word(0x784a), 0);
    assert_eq!(r.word(0x1ab7 + 34), 224);
    assert_eq!(r.word(0x1adb + 34), 256);
}

#[test]
fn wall_reversal_precedes_floor_bounce_and_keeps_low_fraction_bytes() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1a4b, 239);
    r.set_word(0x1a93, 207);
    r.set_word(0x1a27, 0xfe37);
    r.set_word(0x1a6f, 0xfe73);
    r.set_word(0x1ab7, 224);
    r.set_word(0x1adb, 256);
    r.set_word(0x1aff, 112);
    assert_eq!(mother_brain_bomb::move_bomb(&mut r, 0, 7).unwrap(), 1);
    assert_eq!(r.word(0x1a4b), 240);
    assert_eq!(r.word(0x1a93), 208);
    assert_eq!(r.word(0x1ab7), 0xff90);
    assert_eq!(r.word(0x1adb), 0xfe00);
    assert_eq!(r.word(0x1a27), 0xde37);
    assert_eq!(r.word(0x1a6f), 0x0573);
}

#[test]
fn damping_matches_signed_minimum_velocity_and_odd_acceleration_stage() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1a4b, 128);
    r.set_word(0x1a93, 100);
    r.set_word(0x1ab7, 0x8000);
    let mut host = ProjectileHost::from_words([1, 2, 3, 4, 5, 6, 7, 8]);
    mother_brain_bomb::update(&mut r, &[], 0, &mut host).unwrap();
    assert_eq!(r.word(0x1ab7), 0x8002);
    assert_eq!(r.word(0x1adb), 7);
    assert_eq!(r.word(0x1a4b), 0);
    r.set_word(0x1b23, 3);
    r.set_word(0x1ab7, 0);
    mother_brain_bomb::update(&mut r, &[], 0, &mut host).unwrap();
    assert_eq!(r.word(0x1adb), 23);
    assert_eq!(host.words(), [1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn explosion_allocation_reuses_slot_before_the_native_second_spawn() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    // Real initializer identities with a minimal dust script table.
    rom[0x31650..0x3165a].copy_from_slice(&[0x7f, 0x94, 0x03, 0x80, 0x00, 0x90, 1, 1, 0, 0]);
    rom[0x36509..0x36513].copy_from_slice(&[0x68, 0xe4, 0x03, 0x80, 0x00, 0x90, 1, 1, 0, 0]);
    rom[0x36432..0x36434].copy_from_slice(&0x9000u16.to_le_bytes());
    for slot in (0..36).step_by(2) {
        r.set_word(0x1997 + slot, 0xcb59);
    }
    r.set_word(0x1a4b + 34, 128);
    r.set_word(0x1a93 + 34, 208);
    r.set_word(0x1a27 + 34, 0xfe37);
    r.set_word(0x1b23 + 34, 18);
    r.set_word(0x784a, 0);
    let mut host = ProjectileHost::from_words([1, 2, 3, 4, 5, 6, 7, 8]);
    mother_brain_bomb::update(&mut r, &rom, 34, &mut host).unwrap();
    assert_eq!(r.word(0x1997 + 34), 0x9650);
    assert_eq!(r.word(0x1aff + 34), 0x37);
    assert_eq!(r.word(0x1993), 3);
    assert_eq!(r.word(0x784a), 0xffff);
    assert_eq!(host.words(), [128, 208, 3, 4, 5, 6, 7, 8]);
    assert_eq!(r.byte(0x676), 0x13);
}

#[test]
fn active_bombs_require_overlap_on_both_axes_and_zero_projectile_variable() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1a4b, 128);
    r.set_word(0x1a93, 128);
    r.set_word(0x1bb3, 0x0606);
    r.set_word(0xcd2, 1);
    r.set_word(0xc18 + 18, 0xa500);
    r.set_word(0xb64 + 18, 142);
    r.set_word(0xb78 + 18, 128);
    r.set_word(0xbb4 + 18, 8);
    r.set_word(0xbc8 + 18, 8);
    let mut host = ProjectileHost::default();
    assert_eq!(mother_brain_bomb::collide_bombs(&mut r, &[], 0, &mut host).unwrap(), 0);
    r.set_word(0xb64 + 18, 141);
    r.set_word(0xc7c + 18, 1);
    assert_eq!(mother_brain_bomb::collide_bombs(&mut r, &[], 0, &mut host).unwrap(), 0);
    r.set_word(0xc7c + 18, 0);
    r.set_word(0xb78 + 18, 142);
    assert_eq!(mother_brain_bomb::collide_bombs(&mut r, &[], 0, &mut host).unwrap(), 0);
}

#[test]
fn scheduled_final_explosion_runs_reused_and_lower_slots_in_the_same_frame() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    rom[0x31650..0x3165a].copy_from_slice(&[0x7f, 0x94, 0x70, 0x81, 0x00, 0x90, 1, 1, 0, 0]);
    rom[0x36509..0x36513].copy_from_slice(&[0x68, 0xe4, 0x70, 0x81, 0x04, 0x90, 1, 1, 0, 0]);
    rom[0x36432..0x36434].copy_from_slice(&0x9004u16.to_le_bytes());
    rom[0x31000..0x31008].copy_from_slice(&[7, 0, 0x34, 0x92, 9, 0, 0x78, 0x96]);
    r.set_word(0x198d, 0x8000);
    r.set_word(0x1997 + 34, 0xcb59);
    r.set_word(0x1a03 + 34, 0xc4c8);
    r.set_word(0x1a4b + 34, 128);
    r.set_word(0x1a93 + 34, 208);
    r.set_word(0x1a27 + 34, 0xfe37);
    r.set_word(0x1b23 + 34, 18);
    r.set_word(0x784a, 1);
    let mut host = ProjectileHost::from_words([1, 2, 3, 4, 5, 6, 7, 8]);
    port::enemy_projectile_instructions::run_all_with_host(&mut r, &rom, 4096, &mut host).unwrap();
    assert_eq!(r.word(0x1997 + 34), 0x9650);
    assert_eq!(r.word(0x1997 + 32), 0xe509);
    assert_eq!(r.word(0x1b8f + 34), 7);
    assert_eq!(r.word(0x1b6b + 34), 0x9234);
    assert_eq!(r.word(0x1b8f + 32), 9);
    assert_eq!(r.word(0x1b6b + 32), 0x9678);
    assert_eq!(r.word(0x784a), 0);
    assert_eq!(host.words(), [128, 208, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn hostless_scheduler_rejects_before_writing_frame_index() {
    let mut bytes = vec![0; WRAM_SIZE];
    {
        let mut r = Wram::new(&mut bytes).unwrap();
        r.set_word(0x198d, 0x8000);
        r.set_word(0x1997 + 34, 0xcb59);
        r.set_word(0x1a03 + 34, 0xc4c8);
        r.set_word(0x1991, 0xa55a);
    }
    let before = bytes.clone();
    assert!(port::enemy_projectile_instructions::run_all(&mut Wram::new(&mut bytes).unwrap(), &[], 100).is_err());
    assert_eq!(bytes, before);
}
