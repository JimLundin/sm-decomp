use port::{
    WRAM_SIZE, Wram,
    enemy_projectiles::{ProjectileHost, spawn_room, spawn_room_with_host},
};
#[test]
fn host_required_initializers_refuse_without_mutating_wram() {
    let mut rom = vec![0; 0x38000];
    rom[0x30000..0x30002].copy_from_slice(&0x947fu16.to_le_bytes());
    let mut bytes = vec![0; WRAM_SIZE];
    let before = bytes.clone();
    assert!(spawn_room(&mut Wram::new(&mut bytes).unwrap(), &rom, 0x8000, 7).is_err());
    assert_eq!(bytes, before);
    let mut host = ProjectileHost { x: 0xffff, y: 12, rect_width: 123, r22: 1, ..Default::default() };
    let before = host;
    let mut r = Wram::new(&mut bytes).unwrap();
    spawn_room_with_host(&mut r, &rom, 0x8000, 7, &mut host).unwrap();
    assert_eq!(r.word(0x1a4b + 34), 0xffff);
    assert_eq!(r.word(0x1a93 + 34), 12);
    assert_eq!(r.word(0x19bb + 34), 2560);
    assert_eq!(r.word(0x1aff + 34), 7);
    assert_eq!(host, before);
}
#[test]
fn full_slots_preserve_host_and_only_set_parameter_without_rom() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    for k in (0..36).step_by(2) {
        r.set_word(0x1997 + k, 1);
    }
    let mut expected = vec![0; WRAM_SIZE];
    for k in (0..36).step_by(2) {
        expected[0x1997 + k] = 1;
    }
    expected[0x1993..0x1995].copy_from_slice(&99u16.to_le_bytes());
    let mut host = ProjectileHost::from_words([1, 2, 3, 4, 5, 6, 7, 8]);
    spawn_room_with_host(&mut r, &[], 0, 99, &mut host).unwrap();
    assert_eq!(r.word(0x1993), 99);
    assert_eq!(host.words(), [1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(bytes, expected);
}

#[test]
fn legacy_frame_rejects_host_required_slot_before_any_scheduler_writes() {
    let mut bytes = vec![0; WRAM_SIZE];
    {
        let mut r = Wram::new(&mut bytes).unwrap();
        r.set_word(0x198d, 0x8000);
        r.set_word(0x1997, 0x9642);
        r.set_word(0x1a03, 0x940e);
        r.set_word(0x1997 + 34, 1);
        r.set_word(0x1a03 + 34, 0x8170);
        r.set_word(0x1b8f + 34, 2);
    }
    let before = bytes.clone();
    assert!(port::enemy_projectile_instructions::run_all(&mut Wram::new(&mut bytes).unwrap(), &[], 100).is_err());
    assert_eq!(bytes, before);
}

#[test]
fn ridley_impact_updates_external_point_and_reuses_cleared_slot() {
    let mut rom = vec![0; 0x38000];
    rom[0x3165e..0x31660].copy_from_slice(&0x947fu16.to_le_bytes());
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x1997 + 34, 0x9642);
    r.set_word(0x1a4b + 34, 29);
    r.set_word(0x1a93 + 34, 24);
    r.set_word(0x1bb3 + 34, 0x0202);
    r.set_word(0x1ab7 + 34, 0x300);
    r.set_word(0x7a5, 16);
    r.set_word(0x10002 + 18 * 2, 0x8000);
    let mut host = ProjectileHost { r22: 99, ..Default::default() };
    port::enemy_projectile_instructions::preinstruction_with_host(&mut r, &rom, 0x940e, 34, &mut host).unwrap();
    assert_eq!((host.x, host.y), (30, 24));
    assert_eq!(host.r22, 99);
    assert_eq!(r.word(0x1997 + 34), 0x965e);
    assert_eq!(r.word(0x19bb + 34), 2560);
    assert_eq!(r.byte(0x666), 0x2b);
}

#[test]
fn drop_instruction_writes_external_context_even_when_no_slot_is_free() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    for k in (0..36).step_by(2) {
        r.set_word(0x1997 + k, 1);
    }
    r.set_word(0x1a4b + 2, 0xffff);
    r.set_word(0x1a93 + 2, 123);
    let mut expected = vec![0; WRAM_SIZE];
    for k in (0..36).step_by(2) {
        expected[0x1997 + k] = 1;
    }
    expected[0x1a4b + 2..0x1a4b + 4].copy_from_slice(&0xffffu16.to_le_bytes());
    expected[0x1a93 + 2..0x1a93 + 4].copy_from_slice(&123u16.to_le_bytes());
    let mut host = ProjectileHost { r22: 99, ..Default::default() };
    let result =
        port::enemy_projectile_instructions::instruction_with_host(&mut r, &[], 0x9270, 0x9000, 2, &mut host).unwrap();
    assert_eq!(result, 0x9000);
    assert_eq!((host.x, host.y, host.enemy_header), (0xffff, 123, 0xddbf));
    assert_eq!(host.r22, 99);
    assert_eq!(r.word(0x1993), 0);
    assert_eq!(bytes, expected);
}
#[test]
fn legacy_drop_instruction_refuses_before_point_or_wram_changes() {
    let mut bytes = vec![0; WRAM_SIZE];
    let before = bytes.clone();
    assert!(
        port::enemy_projectile_instructions::instruction(&mut Wram::new(&mut bytes).unwrap(), &[], 0x9270, 0x9000, 0)
            .is_err()
    );
    assert_eq!(bytes, before);
}
