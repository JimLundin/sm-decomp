use port::{WRAM_SIZE, Wram};

#[test]
fn body_allocation_wraps_the_link_index_and_selects_orientation() {
    let mut rom = vec![0; 0x38000];
    let definition = 0xec95;
    let offset = 0x30000 + (definition & 0x7fff);
    rom[offset..offset + 2].copy_from_slice(&0xec62u16.to_le_bytes());
    let mut bytes = vec![0; WRAM_SIZE];
    let mut r = Wram::new(&mut bytes).unwrap();
    r.set_word(0xe54, 64);
    r.set_word(0xf7a + 64, 120);
    r.set_word(0xf7e + 64, 300);
    r.set_word(0xfb6 + 64, 1);
    r.set_word(0x8808 + 64, 0xffff);
    r.set_word(0x1c27, 0xffff);
    port::enemy_projectiles::spawn_room(&mut r, &rom, definition as u16, 3).unwrap();
    assert_eq!(r.word(0x1a4b + 34), 120);
    assert_eq!(r.word(0x1a93 + 34), 300);
    assert_eq!(r.word(0x1b47 + 34), 0xec5c);
    assert_eq!(r.word(0x883e), 34);
    r.set_word(0xfb6 + 64, 0);
    r.set_word(0x8808 + 64, 2);
    port::enemy_projectiles::spawn_room(&mut r, &rom, definition as u16, 2).unwrap();
    assert_eq!(r.word(0x1b47 + 32), 0xec56);
    assert_eq!(r.word(0x8844), 32);
}
