use port::{WRAM_SIZE, Wram, enemy_projectile_collision as c};
#[test]
fn solid_wall_stops_motion_at_projectile_radius() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 29);
    r.set_word(0x1a93, 24);
    r.set_word(0x1bb3, 0x0202);
    r.set_word(0x1ab7, 0x0300);
    r.set_word(0x1a27, 0x1234);
    r.set_word(0x10002 + 18 * 2, 0x8000);
    assert_eq!(c::horizontal(&mut r, &[], 0).unwrap(), 1);
    assert_eq!(r.word(0x1a4b), 30);
    assert_eq!(r.word(0x1a27), 0);
    assert_eq!(r.word(0xdc4), 18);
}
#[test]
fn extension_chain_resolves_target_and_leaves_final_block_index() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 29);
    r.set_word(0x1a93, 24);
    r.set_word(0x1bb3, 0x0202);
    r.set_word(0x1ab7, 0x0300);
    r.set_word(0x10002 + 36, 0x5000);
    r.set_byte(0x16402 + 18, 1);
    r.set_word(0x10002 + 38, 0xd000);
    r.set_byte(0x16402 + 19, 1);
    r.set_word(0x10002 + 70, 0x8000);
    assert_eq!(c::horizontal(&mut r, &[], 0).unwrap(), 1);
    assert_eq!(r.word(0xdc4), 35);
    assert_eq!(r.word(0x1a4b), 30);
}
#[test]
fn floor_slope_uses_global_slot_and_outer_collision_clamp() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0xa1000];
    r.set_word(0x7a5, 16);
    r.set_word(0x1991, 2);
    r.set_word(0x1a4b, 24);
    r.set_word(0x1a93, 29);
    r.set_word(0x1bb3, 0x0202);
    r.set_word(0x1adb, 0x0300);
    r.set_word(0x1a4b + 2, 24);
    r.set_word(0x1a93 + 2, 200);
    r.set_word(0x10002 + 33 * 2, 0x1000);
    r.set_byte(0x16402 + 33, 5);
    rom[0xa0b2b + 5 * 16 + 8] = 1;
    assert_eq!(c::vertical(&mut r, &rom, 0).unwrap(), 1);
    assert_eq!(r.word(0x1a93), 30);
    assert_eq!(r.word(0x1a93 + 2), 31);
    assert_eq!(r.word(0x1a6f + 2), 0xffff);
    assert_eq!(r.word(0x1a6f), 0);
}
#[test]
fn sweat_splash_and_robot_laser_delete_on_terrain() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 24);
    r.set_word(0x1a93, 29);
    r.set_word(0x1bb3, 0x0202);
    r.set_word(0x1adb, 0x0300);
    r.set_word(0x10002 + 33 * 2, 0x8000);
    e::preinstruction(&mut r, &[], 0xb714, 0).unwrap();
    assert_eq!(r.word(0x1a93), 26);
    assert_eq!(r.word(0x1b47), 0xb61d);
    assert_eq!(r.word(0x1b8f), 1);
    r.set_word(0x1a4b, 29);
    r.set_word(0x1a93, 24);
    r.set_word(0x1ab7, 0x0300);
    r.set_word(0x10002 + 18 * 2, 0x8000);
    r.set_word(0x19bb, 1600);
    r.set_word(0x1997, 0x9000);
    e::preinstruction(&mut r, &[], 0xd3bf, 0).unwrap();
    assert_eq!(r.word(0x19bb), 0);
    assert_eq!(r.word(0x1997), 0);
}

#[test]
fn delayed_fireball_waits_then_hits_floor_and_queues_explosion_sound() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 24);
    r.set_word(0x1a93, 27);
    r.set_word(0x1bb3, 0x0202);
    r.set_word(0x1adb, 0x0200);
    r.set_word(0x1aff, 7);
    r.set_word(0x10002 + 33 * 2, 0x8000);
    e::preinstruction(&mut r, &[], 0x9392, 0).unwrap();
    assert_eq!(r.word(0x1a93), 27);
    assert_eq!(r.word(0x1aff), 8);
    e::preinstruction(&mut r, &[], 0x9392, 0).unwrap();
    assert_eq!(r.word(0x1a93), 30);
    assert_eq!(r.word(0x1aff), 9);
    assert_eq!(r.word(0x1b47), 0x9574);
    assert_eq!(r.word(0x1b8f), 1);
    assert_eq!(r.word(0x1adb), 0);
    assert_eq!(r.word(0x19bb), 2560);
    assert_eq!(r.byte(0x666), 0x2b);
    assert_eq!(r.byte(0x647), 1);
    assert_eq!(r.byte(0x654), 6);
}

#[test]
fn bridge_debris_acceleration_wraps_to_fourteen_bits() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 24);
    r.set_word(0x1a93, 24);
    r.set_word(0x1bb3, 0x0202);
    r.set_word(0x1adb, 0x3ff8);
    e::preinstruction(&mut r, &[], 0x92ba, 0).unwrap();
    assert_eq!(r.word(0x1adb), 16);
}

#[test]
fn random_radius_rejects_negative_bytes_before_advancing_each_axis() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    rom[0x37000..0x37004].copy_from_slice(&[15, 8, 31, 16]);
    r.set_word(0x5e5, 0);
    r.set_word(0x1a4b, 65535);
    r.set_word(0x1a93, 1);
    assert_eq!(e::instruction(&mut r, &rom, 0x81df, 0xf000, 0).unwrap(), 0xf004);
    // Direction 0111: positive X/Y. 0666 rejects X=-2, 210F accepts
    // X=7, and A65C accepts Y=12, preserving all four RNG calls.
    assert_eq!(r.word(0x1a4b), 6);
    assert_eq!(r.word(0x1a93), 13);
    assert_eq!(r.word(0x5e5), 0xa65c);
}

#[test]
fn impossible_random_radius_is_rejected_before_rng_or_position_writes() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    rom[0x37000..0x37004].copy_from_slice(&[0, 1, 0, 0]);
    r.set_word(0x5e5, 0x1234);
    let before = r.word(0x5e5);
    assert!(e::instruction(&mut r, &rom, 0x81df, 0xf000, 0).is_err());
    assert_eq!(r.word(0x5e5), before);
    assert_eq!(r.word(0x1a4b), 0);
}

#[test]
fn fireball_repeat_decrements_only_low_timer_byte_when_pool_is_full() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    for k in (0..=34).step_by(2) {
        r.set_word(0x1997 + k, 1);
    }
    r.set_word(0x1aff, 0x1201);
    r.set_word(0x1b23, 0x966c);
    assert_eq!(e::instruction(&mut r, &[], 0x9620, 0xf000, 0).unwrap(), 0xf000);
    assert_eq!(r.word(0x1aff), 0x1200);
    e::instruction(&mut r, &[], 0x9620, 0xf000, 0).unwrap();
    assert_eq!(r.word(0x1aff), 0x12ff);
}

#[test]
fn directional_fireball_keeps_velocity_after_impact_script_transition() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 24);
    r.set_word(0x1a93, 29);
    r.set_word(0x1bb3, 0x202);
    r.set_word(0x1ab7, 0x100);
    r.set_word(0x1adb, 0x300);
    r.set_word(0x10002 + 33 * 2, 0x8000);
    e::preinstruction(&mut r, &[], 0x950d, 0).unwrap();
    assert_eq!(r.word(0x1a4b), 25);
    assert_eq!(r.word(0x1a93), 30);
    assert_eq!(r.word(0x1b47), 0x9574);
    assert_eq!(r.word(0x1b8f), 1);
    assert_eq!(r.word(0x1ab7), 0x100);
    assert_eq!(r.word(0x1adb), 0x300);
}

#[test]
fn phantoon_fireball_lands_waits_and_launches_with_frame_parity() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 24);
    r.set_word(0x1a93, 29);
    r.set_word(0x1bb3, 0x202);
    r.set_word(0x1adb, 0x300);
    r.set_word(0x1bd7, 0x7345);
    r.set_word(0x10002 + 33 * 2, 0x8000);
    e::preinstruction(&mut r, &[], 0x9981, 0).unwrap();
    assert_eq!(r.word(0x1a93), 38);
    assert_eq!(r.word(0x1aff), 8);
    assert_eq!(r.word(0x1bd7), 0x8345);
    assert_eq!(r.word(0x1a03), 0x99bf);
    for _ in 0..7 {
        e::preinstruction(&mut r, &[], 0x99bf, 0).unwrap();
    }
    assert_eq!(r.word(0x1aff), 1);
    r.set_word(0x5b6, 1);
    e::preinstruction(&mut r, &[], 0x99bf, 0).unwrap();
    assert_eq!(r.word(0x1a93), 30);
    assert_eq!(r.word(0x1a03), 0x9a01);
    assert_eq!(r.word(0x1adb), 0xfd00);
    assert_eq!(r.word(0x1ab7), 0xff80);
    assert_eq!(r.word(0x1aff), 0);
}

#[test]
fn phantoon_third_bounce_stops_preinstruction() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 24);
    r.set_word(0x1a93, 29);
    r.set_word(0x1bb3, 0x202);
    r.set_word(0x1adb, 0x300);
    r.set_word(0x1aff, 2);
    r.set_word(0x10002 + 33 * 2, 0x8000);
    e::preinstruction(&mut r, &[], 0x9a01, 0).unwrap();
    assert_eq!(r.word(0x1aff), 3);
    assert_eq!(r.word(0x1a03), 0x9a44);
    assert_eq!(r.word(0x1b47), 0x9782);
}

#[test]
fn phantoon_orbit_offscreen_x_preserves_previous_y() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xf7a, 300);
    r.set_word(0x1a93, 123);
    e::preinstruction(&mut r, &[], 0x9ada, 0).unwrap();
    assert_eq!(r.word(0x1a93), 123);
    assert_eq!(r.word(0x1b47), 0x97f8);
    assert_eq!(r.word(0x1aff), 2);
}

#[test]
fn phantoon_orbit_contracts_only_after_delay_on_odd_frames() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xf7a, 128);
    r.set_word(0xf7e, 64);
    r.set_word(0x1adb, 1);
    r.set_word(0x1ab7, 1);
    r.set_word(0x5b6, 1);
    e::preinstruction(&mut r, &[], 0x9b41, 0).unwrap();
    assert_eq!(r.word(0x1adb), 1);
    assert_eq!(r.word(0x1aff), 1);
    e::preinstruction(&mut r, &[], 0x9b41, 0).unwrap();
    assert_eq!(r.word(0x1adb), 0);
    assert_eq!(r.word(0x1aff), 1);
    assert_eq!(r.word(0x1a4b), 128);
    assert_eq!(r.word(0x1a93), 80);
    assert_eq!(r.word(0x1b47), 0x97f8);
}

#[test]
fn kraid_rock_acceleration_wraps_at_positive_threshold() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 24);
    r.set_word(0x1a93, 24);
    r.set_word(0x1bb3, 0x202);
    r.set_word(0x1ab7, 248);
    r.set_word(0x1adb, 0);
    e::preinstruction(&mut r, &[], 0x9d56, 0).unwrap();
    assert_eq!(r.word(0x1ab7), 0xff00);
    assert_eq!(r.word(0x1adb), 64);
}

#[test]
fn kraid_wall_collision_skips_vertical_motion_and_acceleration() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 29);
    r.set_word(0x1a93, 24);
    r.set_word(0x1bb3, 0x202);
    r.set_word(0x1ab7, 0x300);
    r.set_word(0x1adb, 0x200);
    r.set_word(0x1997, 0x9c45);
    r.set_word(0x10002 + 18 * 2, 0x8000);
    e::preinstruction(&mut r, &[], 0x9d56, 0).unwrap();
    assert_eq!(r.word(0x1997), 0);
    assert_eq!(r.word(0x1a93), 24);
    assert_eq!(r.word(0x1ab7), 0x300);
    assert_eq!(r.word(0x1adb), 0x200);
}

#[test]
fn mini_kraid_spit_clamps_positive_speed_but_preserves_signed_wrap() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 24);
    r.set_word(0x1a93, 24);
    r.set_word(0x1bb3, 0x202);
    r.set_word(0x1adb, 960);
    e::preinstruction(&mut r, &[], 0x9e1e, 0).unwrap();
    assert_eq!(r.word(0x1adb), 1024);
    r.set_word(0x1a93, 24);
    r.set_word(0x1adb, 0x7ff0);
    e::preinstruction(&mut r, &[], 0x9e1e, 0).unwrap();
    assert_eq!(r.word(0x1adb), 0x8030);
}

#[test]
fn seahorse_fireball_floor_contact_skips_horizontal_motion() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 24);
    r.set_word(0x1a93, 29);
    r.set_word(0x1bb3, 0x202);
    r.set_word(0x1ab7, 1024);
    r.set_word(0x1adb, 0x300);
    r.set_word(0x1997, 0x9e90);
    r.set_word(0x10002 + 33 * 2, 0x8000);
    e::preinstruction(&mut r, &[], 0x9eff, 0).unwrap();
    assert_eq!(r.word(0x1997), 0);
    assert_eq!(r.word(0x1a4b), 24);
    assert_eq!(r.word(0x1ab7), 1024);
}

#[test]
fn pirate_claw_reverses_after_zero_speed_and_preserves_camera_wrap() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x911, 65520);
    r.set_word(0x1a4b, 16);
    r.set_word(0x1a93, 50);
    r.set_word(0x1aff, 32);
    r.set_word(0x1b23, 1);
    r.set_word(0x1997, 0xa189);
    e::preinstruction(&mut r, &[], 0xa0d1, 0).unwrap();
    assert_eq!(r.word(0x1b23), 0);
    assert_eq!(r.word(0x1aff), 0);
    assert_eq!(r.word(0x1997), 0xa189);
    r.set_word(0x1aff, 256);
    e::preinstruction(&mut r, &[], 0xa0d1, 0).unwrap();
    assert_eq!(r.word(0x1a4b), 17);
    assert_eq!(r.word(0x1a93), 52);
}

#[test]
fn torizo_probability_branch_consumes_exactly_one_rng_call() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    rom[0x37000..0x37002].copy_from_slice(&0xf100u16.to_le_bytes());
    r.set_word(0x5e5, 0x5c15);
    assert_eq!(e::instruction(&mut r, &rom, 0xa456, 0xf000, 0).unwrap(), 0xf100);
    assert_eq!(r.word(0x5e5), 0xcd7a);
    r.set_word(0x5e5, 0);
    assert_eq!(e::instruction(&mut r, &rom, 0xa456, 0xf000, 0).unwrap(), 0xf002);
    assert_eq!(r.word(0x5e5), 0x0111);
}

#[test]
fn torizo_drool_near_zero_motion_preserves_three_velocity_quirk() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 24);
    r.set_word(0x1a93, 24);
    r.set_word(0x1bb3, 0x202);
    r.set_word(0x1ab7, 0);
    e::preinstruction(&mut r, &[], 0xa887, 0).unwrap();
    assert_eq!(r.word(0x1ab7), 3);
    assert_eq!(r.word(0x1adb), 16);
}

#[test]
fn golden_torizo_orb_reflects_wall_then_settles_small_floor_bounce() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 29);
    r.set_word(0x1a93, 29);
    r.set_word(0x1bb3, 0x202);
    r.set_word(0x1ab7, 0x300);
    r.set_word(0x1adb, 128);
    for j in 0..256 {
        r.set_word(0x10002 + j * 2, 0x8000);
    }
    e::preinstruction(&mut r, &[], 0xacfa, 0).unwrap();
    assert_eq!(r.word(0x1ab7), 0xfd40);
    assert_eq!(r.word(0x1adb), 0xffc0);
    assert_eq!(r.word(0x1a93), 22);
    assert_eq!(r.word(0x1b47), 0xab41);
}

#[test]
fn torizo_sonic_impact_saves_position_without_acceleration() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1a4b, 29);
    r.set_word(0x1a93, 24);
    r.set_word(0x1bb3, 0x202);
    r.set_word(0x1ab7, 624);
    r.set_word(0x10002 + 18 * 2, 0x8000);
    e::preinstruction(&mut r, &[], 0xae6c, 0).unwrap();
    assert_eq!(r.word(0x1aff), 30);
    assert_eq!(r.word(0x1b23), 24);
    assert_eq!(r.word(0x1b47), 0xade5);
    assert_eq!(r.word(0x1ab7), 624);
}

#[test]
fn torizo_egg_timer_underflow_launches_without_moving() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1a4b, 123);
    r.set_word(0x1a93, 45);
    r.set_word(0x1b47, 0x9000);
    r.set_word(0x1aff, 0x8000);
    e::preinstruction(&mut r, &[], 0xb043, 0).unwrap();
    assert_eq!(r.word(0x1b23), 65535);
    assert_eq!(r.word(0x1ab7), 256);
    assert_eq!(r.word(0x1b47), 0x9002);
    assert_eq!(r.word(0x1a4b), 123);
    assert_eq!(r.word(0x1a93), 45);
}

#[test]
fn torizo_super_missile_tracks_saved_enemy_slot() {
    use port::enemy_projectile_instructions as e;
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1aff, 64);
    r.set_word(0xe54, 0);
    r.set_word(0xf7a + 64, 100);
    r.set_word(0xf7e + 64, 200);
    r.set_word(0xfb4 + 64, 0x8000);
    e::preinstruction(&mut r, &[], 0xb20d, 0).unwrap();
    assert_eq!(r.word(0x1a4b), 132);
    assert_eq!(r.word(0x1a93), 148);
}

#[test]
fn tourian_soul_removal_tests_bit_eight_and_still_accelerates() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1a93, 768);
    r.set_word(0x1adb, 0);
    port::enemy_projectile_instructions::preinstruction(&mut r, &[], 0xb9fd, 0).unwrap();
    assert_eq!(r.word(0x1b47), 0xb79f);
    assert_eq!(r.word(0x1b8f), 1);
    assert_eq!(r.word(0x1adb), 0xff80);
    // Particles use the complete high byte, unlike souls.
    r.set_word(0x1b47, 0x9754);
    r.set_word(0x1adb, 0);
    port::enemy_projectile_instructions::preinstruction(&mut r, &[], 0xb982, 0).unwrap();
    assert_eq!(r.word(0x1b47), 0x9754);
    assert_eq!(r.word(0x1adb), 16);
}

#[test]
fn tourian_statue_tracks_hdma_scroll_and_preserves_finished_flags() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1aff, 142);
    r.set_word(0x1b23, 85);
    r.set_word(0x915, 32);
    r.set_word(0x9e00, 128);
    r.set_word(0x1e6d, 3);
    port::enemy_projectile_instructions::preinstruction(&mut r, &[], 0xba37, 0).unwrap();
    assert_eq!(r.word(0x1a4b), 142);
    assert_eq!(r.word(0x1a93), 0xfff5);
    assert_eq!(r.word(0x1e6d), 0x8003);
}

#[test]
fn lavaquake_apex_defers_horizontal_motion_and_keeps_inclusive_edges() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1997, 1);
    r.set_word(0x1aff, 0xbc16);
    r.set_word(0x1adb, 1);
    r.set_word(0x1ab7, 0x100);
    r.set_word(0x1a4b, 256);
    r.set_word(0x1a93, 256);
    r.set_word(0x1b23, 0xdead);
    port::enemy_projectile_instructions::preinstruction(&mut r, &[], 0xbc0f, 0).unwrap();
    assert_eq!(r.word(0x1aff), 0xbc8f);
    assert_eq!(r.word(0x1adb), 0);
    assert_eq!(r.word(0x1a4b), 256);
    assert_eq!(r.word(0x1b23), 0xdead);
    assert_eq!(r.word(0x1997), 1);
    r.set_word(0x1aff, 0xbc16);
    r.set_word(0x1adb, 1);
    r.set_word(0x1a4b, 257);
    port::enemy_projectile_instructions::preinstruction(&mut r, &[], 0xbc0f, 0).unwrap();
    assert_eq!(r.word(0x1997), 0);
}

#[test]
fn lavaquake_arc_consumes_both_rom_half_steps_and_retains_last_high_word() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x105000];
    // Falling speed 2 reads rows 1 and 2, downward halves.
    rom[0x104bcf..0x104bd3].copy_from_slice(&0x00018000u32.to_le_bytes());
    rom[0x104bd7..0x104bdb].copy_from_slice(&0x00028000u32.to_le_bytes());
    r.set_word(0x1aff, 0xbc8f);
    r.set_word(0x1a93, 100);
    r.set_word(0x1a6f, 0x1234);
    port::enemy_projectile_instructions::preinstruction(&mut r, &rom, 0xbc0f, 0).unwrap();
    assert_eq!(r.word(0x1adb), 2);
    assert_eq!(r.word(0x1a93), 104);
    assert_eq!(r.word(0x1a6f), 0x1234);
    assert_eq!(r.word(0x1b23), 2);
}

#[test]
fn shaktool_front_stops_before_vertical_check_but_linked_circle_checks_both() {
    for (address, expected_y, alive) in [(0xbe03, 29, 0), (0xbe12, 31, 1)] {
        let mut bytes = vec![0; WRAM_SIZE];
        let mut r = Wram::new(&mut bytes).unwrap();
        r.set_word(0x7a5, 16);
        r.set_word(0x1997, 1);
        r.set_word(0x1aff, 1); // Odd offset aliases the current slot.
        r.set_word(0x1a4b, 27);
        r.set_word(0x1a93, 29);
        r.set_word(0x1bb3, 0x0404);
        r.set_word(0x1ab7, 0x0400);
        r.set_word(0x1adb, 0x0200);
        r.set_word(0x10002 + 18 * 2, 0x8000);
        r.set_word(0x10002 + 34 * 2, 0x8000);
        port::enemy_projectile_instructions::preinstruction(&mut r, &[], address, 0).unwrap();
        assert_eq!(r.word(0x1a4b), 28);
        assert_eq!(r.word(0x1a93), expected_y);
        assert_eq!(r.word(0x1997), alive);
    }
}

#[test]
fn shaktool_linked_circle_expires_without_movement_when_reference_is_dead() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1997, 1);
    r.set_word(0x1aff, 35);
    r.set_word(0x1a4b, 100);
    r.set_word(0x1ab7, 0x400);
    port::enemy_projectile_instructions::preinstruction(&mut r, &[], 0xbe12, 0).unwrap();
    assert_eq!(r.word(0x1997), 0);
    assert_eq!(r.word(0x1a4b), 100);
}

#[test]
fn turret_bullet_hits_nonair_tiles_and_deletes_global_active_slot() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x7a5, 16);
    r.set_word(0x1991, 2);
    r.set_word(0x1997, 1);
    r.set_word(0x1999, 1);
    r.set_word(0x1a4b, 32);
    r.set_word(0x1a93, 32);
    r.set_word(0x1bd7, 0x1234);
    r.set_word(0x10002 + 34 * 2, 0x5000);
    port::enemy_projectile_instructions::preinstruction(&mut r, &[], 0xc0e0, 0).unwrap();
    assert_eq!(r.word(0x1997), 1);
    assert_eq!(r.word(0x1999), 0);
    assert_eq!(r.word(0x1bd7), 0x9234);
}

#[test]
fn turret_rotation_reverses_direction_at_disallowed_octant() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    r.set_word(0x1a27, 0xbec9);
    r.set_word(0x1a6f, 0x0103);
    port::enemy_projectile_instructions::dispatch_rom(&mut r, &rom, "Eproj_MotherBrainRoomTurretBullets_Func2", 0, 0)
        .unwrap();
    assert_eq!(r.word(0x1a6f), 0xff02);
    rom[0x33ec9 + 1] = 1;
    port::enemy_projectile_instructions::dispatch_rom(&mut r, &rom, "Eproj_MotherBrainRoomTurretBullets_Func2", 0, 0)
        .unwrap();
    assert_eq!(r.word(0x1a6f), 0xff01);
}

#[test]
fn nuclear_waffle_body_records_allocated_slot_in_linked_enemy_ram() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    rom[0x33bc7..0x33bc9].copy_from_slice(&0xbb92u16.to_le_bytes());
    r.set_word(0xe54, 64);
    r.set_word(0x8054, 128);
    r.set_word(0xfba, 100);
    r.set_word(0xfbc, 0x1234);
    r.set_word(0xfbe, 200);
    r.set_word(0xfc0, 0x5678);
    r.set_word(0x78c0, 0xdead);
    port::enemy_projectiles::spawn_room(&mut r, &rom, 0xbbc7, 0).unwrap();
    assert_eq!(r.word(0x78c0), 34);
    assert_eq!(r.word(0xf3a2), 1);
    assert_eq!(r.word(0x1a6d), 100);
    assert_eq!(r.word(0x1a49), 0x1234);
    assert_eq!(r.word(0x1ab5), 200);
    assert_eq!(r.word(0x1a91), 0x5678);
}

#[test]
fn spore_motion_sign_extends_rom_deltas_and_wraps_phase_as_a_word() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    let mut rom = vec![0; 0x38000];
    rom[0x35e6a] = 0xfe;
    rom[0x35e6b] = 1;
    r.set_word(0x1997, 1);
    r.set_word(0x1aff, 0xa5fe);
    r.set_word(0x1b23, 0x80);
    r.set_word(0x1a4b, 100);
    r.set_word(0x1a93, 767);
    port::enemy_projectile_instructions::preinstruction(&mut r, &rom, 0xdcee, 0).unwrap();
    assert_eq!(r.word(0x1a4b), 102);
    assert_eq!(r.word(0x1a93), 769);
    assert_eq!(r.word(0x1997), 0);
    assert_eq!(r.word(0x1aff), 0);
}

#[test]
fn spore_spawner_freezes_countdown_and_rng_when_boss_flag_is_set() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x9000, 1);
    r.set_word(0x5e5, 0xface);
    r.set_word(0x1b47, 0x9754);
    port::enemy_projectile_instructions::preinstruction(&mut r, &[], 0xdd46, 0).unwrap();
    assert_eq!(r.word(0x1b23), 0);
    assert_eq!(r.word(0x5e5), 0xface);
    assert_eq!(r.word(0x1b47), 0x9754);
}

#[test]
fn nami_fune_left_state_moves_x_using_y_velocity_and_leaves_y_unchanged() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1997, 1);
    r.set_word(0x1aff, 0xdf40);
    r.set_word(0x1a4b, 100);
    r.set_word(0x1a27, 0x1234);
    r.set_word(0x1a93, 256);
    r.set_word(0x1a6f, 0xabcd);
    r.set_word(0x1ab7, 0x200);
    r.set_word(0x1adb, 0xff80);
    port::enemy_projectile_instructions::preinstruction(&mut r, &[], 0xdf39, 0).unwrap();
    assert_eq!(r.word(0x1a4b), 99);
    assert_eq!(r.word(0x1a27), 0x9234);
    assert_eq!(r.word(0x1a93), 256);
    assert_eq!(r.word(0x1a6f), 0xabcd);
    assert_eq!(r.word(0x1997), 1);
}

#[test]
fn shot_gate_fraction_wrap_does_not_carry_into_y_and_threshold_advances_script() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1adb, 1);
    r.set_word(0x1a93, 100);
    r.set_word(0x1a6f, 0xffa5);
    r.set_word(0x19df, 0xfff);
    r.set_word(0x1b47, 0x9752);
    port::enemy_projectile_instructions::preinstruction(&mut r, &[], 0xe605, 0).unwrap();
    assert_eq!(r.word(0x1a93), 100);
    assert_eq!(r.word(0x1a6f), 0xa5);
    assert_eq!(r.word(0x19df), 0);
    assert_eq!(r.word(0x1b47), 0x9754);
    assert_eq!(r.word(0x1b8f), 1);
}
