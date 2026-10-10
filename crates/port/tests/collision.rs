use port::{WRAM_SIZE, Wram, audio, samus, samus_control};
#[path = "../src/collision.rs"]
mod collision;
struct NoPlm;
impl collision::Host for NoPlm {
    fn spawn_plm(&mut self, _: &mut Wram<'_>, _: u16, _: &mut i32) -> Result<u8, &'static str> {
        Err("unexpected PLM")
    }
}
#[test]
fn solid_wall_clamps_horizontal_movement_and_retains_contact_subposition() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x7a5, 16);
    ram.set_word(0xaf6, 24);
    ram.set_word(0xafa, 24);
    ram.set_word(0xafe, 5);
    ram.set_word(0xb00, 5);
    ram.set_word(0x10002 + 18 * 2, 0x8000);
    assert_eq!(
        collision::dispatch_host(&mut ram, &[], &mut NoPlm, "Samus_MoveRight_NoSolidColl", 0x40000, 0),
        Ok(0x30000)
    );
    assert_eq!(ram.word(0xaf6), 27);
    assert_eq!(ram.word(0xaf8), 0xffff);
    assert_eq!(ram.word(0xdd0), 1);
}
struct ClampPlm;
impl collision::Host for ClampPlm {
    fn spawn_plm(&mut self, ram: &mut Wram<'_>, _: u16, amount: &mut i32) -> Result<u8, &'static str> {
        *amount = 0x10000;
        ram.set_word(0x1e71, 1);
        Ok(0)
    }
}
#[test]
fn special_tile_setup_can_synchronously_change_displacement() {
    let mut rom = vec![0; 0x100000];
    let off = (((0x949139u32 >> 16) << 15) | (0x949139 & 0x7fff)) as usize & 0x3fffff;
    rom[off..off + 2].copy_from_slice(&0xb6ffu16.to_le_bytes());
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x7a5, 16);
    ram.set_word(0xaf6, 24);
    ram.set_word(0xafa, 24);
    ram.set_word(0xafe, 5);
    ram.set_word(0xb00, 5);
    ram.set_word(0x10002 + 18 * 2, 0x3000);
    assert_eq!(
        collision::dispatch_host(&mut ram, &rom, &mut ClampPlm, "Samus_MoveRight_NoSolidColl", 0x40000, 0),
        Ok(0x10000)
    );
    assert_eq!(ram.word(0xaf6), 25);
    assert_eq!(ram.word(0x1e71), 1);
}
#[test]
fn solid_enemy_limits_movement_and_records_directional_contact() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x17a6, 2);
    ram.set_word(0x17ec, 0);
    ram.set_word(0x17ee, 0xffff);
    ram.set_word(0xaf6, 24);
    ram.set_word(0xafa, 24);
    ram.set_word(0xafe, 5);
    ram.set_word(0xb00, 5);
    ram.set_word(0xb02, 1);
    ram.set_word(0xf78 + 2, 34);
    ram.set_word(0xf78 + 6, 24);
    ram.set_word(0xf78 + 10, 5);
    ram.set_word(0xf78 + 12, 5);
    ram.set_word(0xf78 + 14, 0x8000);
    let result = collision::check_solid_enemy(&mut ram, 0x40000).unwrap();
    assert!(result.0);
    assert_eq!(result.1, 0);
    assert_eq!(ram.word(0x182e), 0);
    assert_eq!(ram.word(0x18a6), 0);
}
#[test]
fn landing_on_floor_preserves_the_original_ffff_enemy_collision_flag() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x7a5, 16);
    ram.set_word(0xaf6, 24);
    ram.set_word(0xafa, 24);
    ram.set_word(0xafe, 5);
    ram.set_word(0xb00, 5);
    ram.set_byte(0xa1f, 2);
    ram.set_word(0x17a6, 2);
    ram.set_word(0x17ec, 0);
    ram.set_word(0x17ee, 0xffff);
    ram.set_word(0xf7a, 24);
    ram.set_word(0xf7e, 34);
    ram.set_word(0xf82, 5);
    ram.set_word(0xf84, 5);
    ram.set_word(0xf86, 0x8000);
    assert_eq!(collision::dispatch_host(&mut ram, &[], &mut NoPlm, "Samus_MoveDown", 0x40000, 0), Ok(0));
    assert_eq!(ram.word(0xdd0), 0xffff);
    assert_eq!(ram.word(0xdc6), 1);
    assert_eq!(ram.word(0xafa), 24);
}
