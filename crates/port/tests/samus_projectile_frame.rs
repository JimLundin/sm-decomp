pub use port::{WRAM_SIZE, Wram, power_bomb, projectile_flight, projectile_special, projectiles};
#[path = "../src/samus_projectile_frame.rs"]
mod frame;
#[path = "../src/projectile_trails.rs"]
pub mod projectile_trails;

#[derive(Default)]
struct Host {
    callbacks: Vec<u16>,
}
impl projectile_special::Host for Host {
    fn spawn_plm(&mut self, _: &mut Wram<'_>, header: u16) -> Result<u8, &'static str> {
        self.callbacks.push(header);
        Ok(0)
    }
}
impl frame::Host for Host {
    fn spawn_hdma(&mut self, _: &mut Wram<'_>, script: u16) -> Result<u16, &'static str> {
        self.callbacks.push(script);
        Ok(0)
    }
}
fn put_word(rom: &mut [u8], bank: u8, pointer: u16, value: u16) {
    let at = (usize::from(bank & 0x7f) << 15) + usize::from(pointer & 0x7fff);
    rom[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn map(rom: &mut [u8], bank: u8, pointer: u16, entries: &[(u16, u8, u16)]) {
    put_word(rom, bank, pointer, entries.len() as u16);
    let at = (usize::from(bank & 0x7f) << 15) + usize::from(pointer & 0x7fff) + 2;
    for (i, &(x, y, tile)) in entries.iter().enumerate() {
        let at = at + i * 5;
        rom[at..at + 2].copy_from_slice(&x.to_le_bytes());
        rom[at + 2] = y;
        rom[at + 3..at + 5].copy_from_slice(&tile.to_le_bytes());
    }
}
fn active(r: &mut Wram<'_>, k: usize, kind: u16, map: u16, x: u16, y: u16) {
    r.set_word(0xc40 + k, 0x9000);
    r.set_word(0xc18 + k, kind);
    r.set_word(0xcb8 + k, map);
    r.set_word(0xb64 + k, x);
    r.set_word(0xb78 + k, y);
}

#[test]
fn ten_slot_scheduler_executes_delete_goto_and_data_in_reverse_order() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut rom = vec![0; 0xa0000];
    put_word(&mut rom, 0x93, 0x9000, 0x8239);
    put_word(&mut rom, 0x93, 0x9002, 0x9010);
    for (i, word) in [5, 0xa000, 0x1234, 0].into_iter().enumerate() {
        put_word(&mut rom, 0x93, 0x9010 + (i * 2) as u16, word);
    }
    put_word(&mut rom, 0x93, 0x9020, 0x822f);
    let mut r = Wram::new(&mut bytes).unwrap();
    for k in (0..20).step_by(2) {
        active(&mut r, k, 0, 0, 30, 40);
        r.set_word(0xc68 + k, 0xb169);
        r.set_word(0xc54 + k, 1);
    }
    r.set_word(0xc40 + 18, 0x9020);
    r.set_word(0xcd2, 1);
    frame::dispatch_host(&mut r, &rom, &mut Host::default(), "HandleProjectile", 0, 0).unwrap();
    assert_eq!(r.word(0xdde), 0xfffe);
    assert_eq!(r.word(0xc40 + 18), 0);
    assert_eq!(r.word(0xcd2), 0);
    for k in (0..18).step_by(2) {
        assert_eq!(r.word(0xc54 + k), 5);
        assert_eq!(r.word(0xc40 + k), 0x9018);
        assert_eq!(r.word(0xcb8 + k), 0xa000);
        assert_eq!((r.word(0xbb4 + k), r.word(0xbc8 + k)), (0x34, 0x12));
    }
}

#[test]
fn unsupported_later_preinstruction_refuses_before_publishing_earlier_work() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    active(&mut r, 18, 0, 0, 10, 20);
    active(&mut r, 0, 0, 0, 10, 20);
    r.set_word(0xc68 + 18, 0xb169);
    r.set_word(0xc54 + 18, 7);
    r.set_word(0xc68, 0xffff);
    let before = (0..WRAM_SIZE).map(|i| r.byte(i)).collect::<Vec<_>>();
    assert!(!frame::supports_call(&r, &[], "HandleProjectile", 0, 0).unwrap());
    assert!(frame::handle(&mut r, &[], &mut Host::default()).is_err());
    assert_eq!((0..WRAM_SIZE).map(|i| r.byte(i)).collect::<Vec<_>>(), before);
}

#[test]
fn native_plm_and_hdma_dependencies_do_not_enter_immutable_selection() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    active(&mut r, 10, 0x400, 0, 100, 100);
    r.set_word(0xc68 + 10, 0xb0ae);
    r.set_word(0xc54 + 10, 4);
    r.set_word(0xc7c + 10, 1);
    r.set_word(0x7a5, 16);
    r.set_word(0x7a7, 16);
    r.set_word(0x7a9, 1);
    r.set_word(0x7ab, 1);
    r.set_word(0xcd2, 1);
    let before = (0..WRAM_SIZE).map(|i| r.byte(i)).collect::<Vec<_>>();
    assert!(!frame::supports_call(&r, &vec![0; 0x100000], "HandleProjectile", 0, 0).unwrap());
    assert_eq!((0..WRAM_SIZE).map(|i| r.byte(i)).collect::<Vec<_>>(), before);
}

#[test]
fn plain_and_plasma_beams_follow_the_two_native_flicker_patterns() {
    let mut rom = vec![0; 0xa0000];
    map(&mut rom, 0x93, 0x9000, &[(0, 0, 0x1234)]);
    for kind in [0, 0xc] {
        for nmi in 0..4 {
            let mut bytes = vec![0; WRAM_SIZE];
            let mut r = Wram::new(&mut bytes).unwrap();
            r.set_word(0x5b6, nmi);
            for k in (0..10).step_by(2) {
                active(&mut r, k, kind, 0x9000, k as u16, 10);
            }
            frame::dispatch_rom(&mut r, &rom, "DrawPlayerExplosions2", 0, 0).unwrap();
            let expected: Vec<u8> = (0..10)
                .step_by(2)
                .rev()
                .filter(|&k| if kind == 0 { (k & 2 != 0) != (nmi & 1 != 0) } else { (k & 2 != 0) == (nmi & 2 != 0) })
                .map(|k| k as u8)
                .collect();
            assert_eq!(r.word(0x590), expected.len() as u16 * 4);
            assert_eq!((0..expected.len()).map(|i| r.byte(0x370 + i * 4)).collect::<Vec<_>>(), expected);
            assert_eq!(r.word(0xdde), 0xfffe);
        }
    }
}

#[test]
fn unaligned_spritemap_stores_extension_before_unsigned_vertical_and_tile() {
    let mut rom = vec![0; 0xa0000];
    map(&mut rom, 0x93, 0x9000, &[(0x8100, 0x80, 0xbeef), (2, 0xfe, 0x1234)]);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x590, 0x203);
    frame::draw_spritemap(&mut r, &rom, 0x93, 0x9000, 0x101, 0xff).unwrap();
    // The first extension word aliases OAM_NEXT itself. The local cursor then
    // wraps before the second sprite; final cursor replaces that aliased word.
    assert_eq!(r.byte(0x573), 1);
    assert_eq!(r.byte(0x574), 0x7f);
    assert_eq!(r.word(0x575), 0xbeef);
    assert_eq!(r.byte(0x377), 3);
    assert_eq!(r.byte(0x378), 0xfd);
    assert_eq!(r.word(0x379), 0x1234);
    assert_eq!(r.word(0x590), 11);
}

#[test]
fn oam_alias_can_activate_a_lower_projectile_in_the_same_pass() {
    let mut rom = vec![0; 0xa0000];
    map(&mut rom, 0x93, 0x9000, &[(1, 0x90, 0x9000)]);
    map(&mut rom, 0x93, 0x9010, &[(0, 0, 0x4321)]);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    active(&mut r, 8, 0x10, 0x9000, 0, 0);
    r.set_word(0xcb8 + 6, 0x9010);
    r.set_word(0xc18 + 6, 0x10);
    r.set_word(0xb64 + 6, 40);
    r.set_word(0xb78 + 6, 50);
    r.set_word(0x590, (0xc45 - 0x370) as u16);
    frame::dispatch_rom(&mut r, &rom, "DrawPlayerExplosions2", 0, 0).unwrap();
    assert_ne!(r.word(0xc40 + 6), 0);
    assert_eq!(r.byte(0x449), 40);
    assert_eq!(r.byte(0x44a), 50);
    assert_eq!(r.word(0x44b), 0x4321);
}

#[test]
fn player_pass_draws_echoes_before_the_frozen_trail_overlay() {
    let mut rom = vec![0; 0xa0000];
    map(&mut rom, 0x92, 0x9000, &[(0, 0, 0x1111)]);
    put_word(&mut rom, 0x92, 0x808d, 0x9000);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0x5b6, 1);
    r.set_word(0xac6, 1);
    r.set_word(0xab6, 20);
    r.set_word(0xabe, 30);
    r.set_word(0xa78, 1);
    r.set_word(0xd658, 5);
    r.set_word(0xd730, 40);
    r.set_word(0xd778, 50);
    r.set_word(0xd6e8, 0x2222);
    frame::dispatch_rom(&mut r, &rom, "DrawPlayerExplosions2", 0, 0).unwrap();
    assert_eq!((r.byte(0x370), r.byte(0x371), r.word(0x372)), (20, 30, 0x1111));
    assert_eq!((r.byte(0x374), r.byte(0x375), r.word(0x376)), (40, 50, 0x2222));
    assert_eq!(r.word(0x590), 8);
    assert_eq!(r.word(0xd658), 5);
}

#[test]
fn elevated_player_pass_subtracts_eight_and_skips_echo_flicker() {
    let mut rom = vec![0; 0xa0000];
    map(&mut rom, 0x93, 0x9000, &[(0, 0, 0x1234)]);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    active(&mut r, 0, 0x800, 0x9000, 40, 8);
    r.set_word(0xac6, 1);
    r.set_word(0x5b6, 1);
    frame::dispatch_rom(&mut r, &rom, "sub_9382FD", 0, 0).unwrap();
    assert_eq!((r.byte(0x370), r.byte(0x371)), (40, 0));
    assert_eq!(r.word(0x590), 4);
}

#[test]
fn ceres_player_uninitialized_draw_y_refuses_before_publishing() {
    let mut rom = vec![0; 0xa0000];
    map(&mut rom, 0x93, 0x9000, &[(0, 0, 0x1234)]);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    active(&mut r, 8, 0x10, 0x9000, 30, 40);
    r.set_word(0x93f, 0x8000);
    r.set_word(0x78, 0x100);
    let before = (0..WRAM_SIZE).map(|i| r.byte(i)).collect::<Vec<_>>();
    assert_eq!(
        frame::dispatch_rom(&mut r, &rom, "DrawPlayerExplosions2", 0, 0),
        Err("Ceres player oracle leaves draw Y uninitialized")
    );
    assert!(!frame::supports_call(&r, &rom, "DrawPlayerExplosions2", 0, 0).unwrap());
    assert_eq!((0..WRAM_SIZE).map(|i| r.byte(i)).collect::<Vec<_>>(), before);
}

#[test]
fn ceres_uninitialized_explosion_locals_refuse_without_wram_changes() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    active(&mut r, 18, 0x700, 0x9000, 30, 40);
    r.set_word(0x93f, 0x8000);
    let before = (0..WRAM_SIZE).map(|i| r.byte(i)).collect::<Vec<_>>();
    assert!(!frame::supports_call(&r, &[], "DrawBombAndProjectileExplosions", 0, 0).unwrap());
    assert_eq!(
        frame::dispatch_rom(&mut r, &[], "DrawBombAndProjectileExplosions", 0, 0),
        Err("Ceres explosion oracle discards Mode7 point")
    );
    assert_eq!((0..WRAM_SIZE).map(|i| r.byte(i)).collect::<Vec<_>>(), before);
}

#[test]
fn ceres_ignored_mode7_return_refuses_even_after_an_initialized_slot() {
    let mut rom = vec![0; 0xa0000];
    map(&mut rom, 0x93, 0x9000, &[(0, 0, 0x1111)]);
    map(&mut rom, 0x93, 0x9010, &[(0, 0, 0x2222)]);
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    active(&mut r, 18, 0x500, 0x9000, 30, 40);
    active(&mut r, 16, 0x700, 0x9010, 130, 140);
    r.set_word(0x93f, 0x8000);
    let before = (0..WRAM_SIZE).map(|i| r.byte(i)).collect::<Vec<_>>();
    assert!(!frame::supports_call(&r, &rom, "DrawBombAndProjectileExplosions", 0, 0).unwrap());
    assert!(frame::dispatch_rom(&mut r, &rom, "DrawBombAndProjectileExplosions", 0, 0).is_err());
    assert_eq!((0..WRAM_SIZE).map(|i| r.byte(i)).collect::<Vec<_>>(), before);
}

#[test]
fn mode7_rotation_preserves_signed_products_and_wrapped_camera_subtraction() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xb64, 140);
    r.set_word(0xb78, 90);
    r.set_word(0x80, 100);
    r.set_word(0x82, 100);
    r.set_word(0x78, 0);
    r.set_word(0x7a, 0x100);
    r.set_word(0x7c, 0xff00);
    r.set_word(0x911, 5);
    r.set_word(0x915, 7);
    assert_eq!(frame::dispatch_rom(&mut r, &[], "CalcExplosion_Mode7", 0, 0).unwrap(), 105 | (133 << 16));
}
