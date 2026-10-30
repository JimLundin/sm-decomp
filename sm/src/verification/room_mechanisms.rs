//! Synthetic current-entry observations; shared with the later owned integration.
//! Synthetic dispatch, creation, drawing, real effects, consumers and suspension.
//! Eight command identities retain qualified native intervals with inherently
//! unaligned original loads (grey door, custom draw, turret, item frames and pose).
use crate::native_rom::NativeRomPointer;
use crate::room_background_support::{ram_bytes, ram_span};
use crate::{game_state::Game, rom::RomImage};
use std::panic::{AssertUnwindSafe, catch_unwind};
use {
    crate::game::room_mechanisms as mechanisms,
    sm_game::enemy_script::{EnemyScript, bank_base},
};
const COMMANDS: [u32; 14] = [
    8685236, 8685244, 8685249, 8685258, 8685390, 8685375, 8686138, 8686116, 8686126, 8686194,
    8685412, 8685541, 8686608, 8686913,
];
#[path = "room_mechanism_callback_edges.rs"]
mod callback_edges;
#[path = "room_mechanism_drawing_extended.rs"]
mod drawing_extended;
#[cfg(sm_verify_dll)]
pub(crate) use callback_edges::{
    display as observe_message_display, native_read as observe_native_draw_read,
};
#[path = "room_mechanism_getters.rs"]
mod getter_checks;
pub(crate) const CASES: u32 = 1038;
fn put(g: &mut Game, a: usize, v: u16) {
    g.ram().copy_in(a, &v.to_le_bytes());
}
fn rom_word(image: &RomImage, address: u32, value: u16) {
    let a = crate::rtl::lorom_offset(address) as usize;
    for (i, b) in value.to_le_bytes().into_iter().enumerate() {
        image.write(a + i, b);
    }
}
fn slot(g: &mut Game, k: usize, header: u16) {
    for (a, v) in [
        (0x1c37, header),
        (0x1c87, 34),
        (0x1cd7, 0x86d0),
        (0x1d27, 0x9000),
        (0x1d77, 2),
        (0x1dc7, 0x8000),
        (0x1e17, 0x1234),
        (0xde1c, 1),
        (0xde6c, 0xa000),
        (0xdebc, 0x9300),
        (0xdf0c, 0x5678),
    ] {
        put(g, a + 2 * k, v);
    }
}
fn seed(g: &mut Game, image: &RomImage) {
    for (a, v) in [
        (0x7a5, 16),
        (0x7a9, 8),
        (0x7ab, 8),
        (0x7b9, 1024),
        (0x1c23, 0x8005),
        (0x1c27, 0),
        (0xdc4, 17),
        (0xaf6, 24),
        (0xafa, 24),
    ] {
        put(g, a, v);
    }
    for i in 0..40 {
        slot(g, i, 0);
    }
    rom_word(image, 0x848000, 8698220u32 as u16);
    rom_word(image, 0x848002, 0x9000);
    for (a, v) in [
        (0x8f9000, 0x8000),
        (0x8f9002, 0x0203),
        (0x8f9004, 0x4567),
        (0x849000, 8685236u32 as u16),
        (0x849100, 0x9200),
        (0x849102, 0x1234),
        (0x849104, 0x5678),
        (0x849106, 0x9abc),
        (0x849108, 0xdef0),
        (0x84a000, 1),
        (0x84a002, 0x123),
        (0x84a004, 0),
    ] {
        rom_word(image, a, v);
    }
    for i in 0..4 {
        rom_word(image, 0x8487cd + 2 * i, 0x6000 + i as u16 * 0x80);
        rom_word(image, 0x8487d5 + 2 * i, 0x40 + i as u16 * 16);
        rom_word(image, 0x8487dd + 2 * i, 0x100 + i as u16 * 8);
    }
    for i in 0..2048 {
        put(g, 0xa000 + 2 * i, (i as u16) ^ 0x1357);
    }
}
fn result(script: EnemyScript) -> (u8, u16) {
    match script {
        EnemyScript::Stop => (0, 0),
        EnemyScript::Jump(a) => (1, a),
        s => (2, s.offset_from(bank_base(0x84)) as u16),
    }
}
// All unsafe calls below are actual current production entries, never modeled effects.
unsafe fn execute(id: u32, g: &mut Game, image: &RomImage) -> (u8, u16) {
    unsafe {
        match id {
            0..=11 => {
                let mode = id / 4;
                let source = id % 4;
                if mode != 0 {
                    for i in 0..40 {
                        put(g, 0x1c37 + 2 * i, 0x8000);
                    }
                    if mode == 2 {
                        put(g, 0x1c37 + 2 * 17, 0);
                    }
                }
                match source {
                    0 => {
                        mechanisms::spawn_hardcoded(
                            crate::types::SpawnHardcodedPlmArgs {
                                block_x: 3,
                                block_y: 2,
                                header: 0x8000,
                            },
                            g,
                        );
                        (0, 0)
                    }
                    1 => (0, u16::from(mechanisms::spawn_collision(0x8000, g))),
                    2 => {
                        mechanisms::spawn_room(0x9000, g);
                        (0, 0)
                    }
                    _ => {
                        for (a, v) in [(0x12, 0x8000), (0x14, 0x0203), (0x16, 0x4567)] {
                            put(g, a, v);
                        }
                        mechanisms::spawn_room(0x12, g);
                        (0, 0)
                    }
                }
            }
            12..=15 => {
                for i in 0..40 {
                    put(g, 0x1c37 + 2 * i, 0x8000);
                }
                put(g, 0x1c2d, 6);
                put(g, 0x1c2f, 0x9999);
                match id {
                    12 => mechanisms::clear(g),
                    13 => {
                        put(g, 0x1c23, 0x4567);
                        mechanisms::set_enabled(true, g)
                    }
                    14 => {
                        put(g, 0x1c23, 0xc567);
                        mechanisms::set_enabled(false, g)
                    }
                    _ => {
                        mechanisms::set_enabled(false, g);
                        mechanisms::set_enabled(true, g)
                    }
                };
                (0, 0)
            }
            16..=57 => {
                let choice = ((id - 16) / 3) as usize;
                let k = [0, 1, 80][((id - 16) % 3) as usize];
                slot(g, 0, 0x8000);
                put(g, 0x1c2d, 6);
                result(mechanisms::checks::command(
                    COMMANDS[choice],
                    EnemyScript::at(image.clone(), crate::rtl::lorom_offset(0x849100) as usize),
                    k,
                    g,
                ))
            }
            58..=65 => {
                let mode = id - 58;
                slot(g, 39, 0x8000);
                match mode {
                    0 => put(g, 0x1c23, 5),
                    1 => put(g, 0x1c37 + 78, 0),
                    2 => put(g, 0xde1c + 78, 0),
                    3 => put(g, 0xde1c + 78, 2),
                    4 => {
                        rom_word(image, 0x849000, 8685390u32 as u16);
                        image.write(crate::rtl::lorom_offset(0x849002) as usize, 7);
                        rom_word(image, 0x849003, 8685236u32 as u16);
                    }
                    5 => {
                        rom_word(image, 0x849000, 1);
                        rom_word(image, 0x849002, 0xa000);
                    }
                    6 => g.ram().write_byte(0x613, 2),
                    _ => g.ram().write_byte(0x613, 255),
                }
                (3, u16::from(mechanisms::run(g)))
            }
            66..=81 => {
                let mode = id - 66;
                slot(g, 0, 0x8000);
                let vertical = mode & 1 != 0;
                let flags = ((mode >> 1) & 3) as u16 * 0x400;
                rom_word(image, 0x84a000, if vertical { 0x8003 } else { 3 });
                for i in 0..3 {
                    rom_word(image, 0x84a002 + 2 * i, 0x123 | flags);
                }
                rom_word(image, 0x84a008, 0);
                if mode >= 8 {
                    for (a, v) in [
                        (0x1800, if vertical { 0x8003 } else { 3 }),
                        (0x1802, 0x123 | flags),
                        (0x1804, 0x124 | flags),
                        (0x1806, 0x125 | flags),
                        (0x1808, 0),
                    ] {
                        put(g, a, v);
                    }
                    put(g, 0xde6c, 0x1800);
                }
                if mode == 12 {
                    put(g, 0x1c87, 30);
                }
                if mode == 13 {
                    put(g, 0x911, 32);
                }
                if mode == 14 {
                    put(g, 0x915, 32);
                }
                if mode == 15 {
                    put(g, 0x1c25, 504);
                }
                mechanisms::checks::process_draw(0, g);
                mechanisms::calculate_coordinates(0, g);
                mechanisms::checks::draw(0, g);
                (0, 0)
            }
            82..=85 => {
                slot(g, 0, 0x8000);
                put(g, 0x1c2d, 6);
                let short = RomImage::zeroed(1);
                short.write(0, 7);
                result(mechanisms::checks::command(
                    [8685249, 8685390, 8685412, 8685541][(id - 82) as usize],
                    EnemyScript::at(short, 0),
                    0,
                    g,
                ))
            }
            86..=89 => {
                put(g, 0x7a5, [0, 16, 256, 0xffff][(id - 86) as usize]);
                mechanisms::calculate_coordinates(0, g);
                (0, 0)
            }
            _ => unreachable!(),
        }
    }
}
// Source census:115 setup identities. B3E3's fatal diagnostic is exercised
// separately; these114 defined setup dispatches have two synthetic input states.
const SETUPS: [u32; 114] = [
    0x84ee4d, 0x84ee52, 0x84ee57, 0x84ee5c, 0x84ee64, 0x84ee77, 0x84ee7c, 0x84ee81, 0x84ee86,
    0x84ee8e, 0x84eeab, 0x84b33a, 0x84b345, 0x84b350, 0x84b35b, 0x84b371, 0x84b382, 0x84b393,
    0x84b1c8, 0x84b1f0, 0x84b26d, 0x84b29d, 0x84b2d0, 0x84b300, 0x84b590, 0x84b5ee, 0x84b18b,
    0x84b21d, 0x84b245, 0x84b3eb, 0x84b408, 0x84b497, 0x84b4a8, 0x84b4b6, 0x84b541, 0x84b4c4,
    0x84d18f, 0x84d620, 0x84b5f8, 0x84b9f1, 0x84c794, 0x84c7bb, 0x84c7b1, 0x84c7e2, 0x84c54d,
    0x84c56c, 0x84c58b, 0x84c5af, 0x84c5d3, 0x84c5f1, 0x84c610, 0x84c627, 0x84c6be, 0x84c6cb,
    0x84c6d8, 0x84c6dc, 0x84c6e0, 0x84c73a, 0x84cdc2, 0x84cdea, 0x84ce37, 0x84ce6b, 0x84ce83,
    0x84ceda, 0x84cf2e, 0x84cf67, 0x84cfa0, 0x84b978, 0x84cfb5, 0x84b3c1, 0x84b3d4, 0x84de94,
    0x84deb9, 0x84def0, 0x84df15, 0x84df4c, 0x84b0dc, 0x84b113, 0x84b7c3, 0x84b89c, 0x84d67f,
    0x84d689, 0x84d693, 0x84d6cc, 0x84d606, 0x84d5f6, 0x84ab28, 0x84b04a, 0x84cfd5, 0x84d108,
    0x84d117, 0x84d12b, 0x84d616, 0x84b8dc, 0x84b551, 0x84b56f, 0x84b9c5, 0x84bb09, 0x84db1e,
    0x84da8c, 0x84dab9, 0x84ab27, 0x84b3cf, 0x84b8ef, 0x84bafa, 0x84d6f6, 0x84d6f7, 0x84d18e,
    0x84b3d0, 0x84b54f, 0x84b96c, 0x84b3d2, 0x84cfd1, 0x84cfcd,
];
unsafe fn setup_case(id: u32, g: &mut Game) -> (u8, i32) {
    let index = ((id - 90) / 2) as usize;
    let variant = (id - 90) % 2;
    for i in 0..40 {
        slot(g, i, 0x8000);
        put(g, 0x1c87 + 2 * i, 0x442);
        put(g, 0x1dc7 + 2 * i, 0);
        put(g, 0x1e17 + 2 * i, 0);
        put(g, 0xdf0c + 2 * i, 0);
    }
    // Native Eproj allocation exhaustion is explicit here. Actual successful
    // gate/debris initialization is covered by separate dedicated scenarios.
    for i in 0..18 {
        put(g, 0x1997 + 2 * i, 1);
    }
    for (a, v) in [
        (0x1c27, 0),
        (0x7a5, 16),
        (0x7a9, 8),
        (0x7ab, 8),
        (0xaf6, 24),
        (0xafa, 552),
        (0xa1c, 1),
        (0xb02, if variant == 0 { 0 } else { 3 }),
        (0x9a2, if variant == 0 { 0 } else { 0xffff }),
        (0x9a4, if variant == 0 { 0 } else { 0xffff }),
        (0xc18, if variant == 0 { 0x100 } else { 0x300 }),
        (0xdde, 0),
        (0x9c2, 99),
        (0x9c4, 99),
        (0x9c6, 10),
        (0x9c8, 10),
    ] {
        put(g, a, v);
    }
    let mut displacement = 0x12345i32;
    let returned = {
        mechanisms::checks::setup_entry(
            SETUPS[index],
            0,
            &mut crate::game::mechanism_displacement::Displacement::Local(&mut displacement),
            if variant == 0 { 0 } else { 0x8da0 },
            g,
        )
    };
    // Record transported displacement without hiding it in game RAM.
    (returned, displacement)
}

const PRE_ACTIONS: [u32; 57] = [
    0x84b927, 0x84b948, 0x84bb6b, 0x84bba4, 0x84bd50, 0x84bdd4, 0x84bde3, 0x84bdf2, 0x84be01,
    0x84be1f, 0x84db64, 0x84bd0f, 0x84bd26, 0x84bd88, 0x84bdb2, 0x84be1c, 0x84bb52, 0x84df89,
    0x84dfe6, 0x84d4bf, 0x84d33b, 0x84d15c, 0x84d1e6, 0x84ac89, 0x84d753, 0x84dade, 0x84daee,
    0x84dafe, 0x84db0e, 0x84b7ef, 0x84b82a, 0x84b846, 0x84be30, 0x84b8b0, 0x84b7dd, 0x848469,
    0x84dfa8, 0x8486d0, 0x8484e6, 0x84853d, 0x848aa6, 0x84bba3, 0x84d356, 0x84bafa, 0x84b0dc,
    0x84b113, 0x84bb6a, 0x84dad5, 0x84dad6, 0x84dad7, 0x84dad8, 0x84dad9, 0x84dada, 0x84dadb,
    0x84dadc, 0x84dadd, 0x848ae0,
];
const MORE_COMMANDS: [u32; 102] = [
    0x8486b4, 0x8486c1, 0x84874e, 0x848a72, 0x848d41, 0x84abd6, 0x84aebf, 0x84cd93, 0x84d79f,
    0x84db8e, 0x8486bc, 0x848a3a, 0x8486ca, 0x84873f, 0x8489fb, 0x848bdd, 0x84ba6f, 0x848764,
    0x8487e5, 0x848c8f, 0x84880e, 0x84882d, 0x84883e, 0x848848, 0x848865, 0x84887c, 0x848899,
    0x8488b0, 0x8488f3, 0x84891a, 0x848941, 0x848968, 0x848986, 0x8489a9, 0x8489d2, 0x848a24,
    0x848a2e, 0x848a91, 0x848acd, 0x848af1, 0x848b55, 0x848b93, 0x848bd1, 0x848c07, 0x848c10,
    0x848c19, 0x848c22, 0x848c2b, 0x848c34, 0x848c3d, 0x848c46, 0x848c4f, 0x848c58, 0x848c61,
    0x848c6a, 0x848c73, 0x848c7c, 0x848c85, 0x848caf, 0x848cd0, 0x848cf1, 0x84ae35, 0x84b024,
    0x84b030, 0x84bbdd, 0x84bbe1, 0x84bbf0, 0x84d3d7, 0x84d3f4, 0x84d77a, 0x84d790, 0x84d7b6,
    0x84dbf7, 0x84dc60, 0x84dc9f, 0x84e07f, 0x84e29d, 0x84e63b, 0x84d525, 0x84d52c, 0x84d536,
    0x84d543, 0x84d5e6, 0x84d5ee, 0x84d357, 0x84d3c7, 0x84d155, 0x84ad43, 0x84ad58, 0x84ab51,
    0x84ab00, 0x84ab59, 0x84d2f9, 0x84d30b, 0x84b9b9, 0x84bb25, 0x848724, 0x84ac9d, 0x84acb1,
    0x84d7c3, 0x84d7da, 0x84dc36,
];
const PRE_END: u32 = 660;

// Full effect allocation pools here deliberately exercise allocation exhaustion.
// Successful native effects and bus observations have separate scenarios.
fn dispatch_seed(g: &mut Game, image: &RomImage, variant: usize) {
    use crate::ram_map::words as ram;
    for i in 0..40 {
        slot(g, i, 0x8000);
        put(g, 0x1c87 + 2 * i, 0x442);
        put(g, 0x1dc7 + 2 * i, 0);
        put(g, 0x1e17 + 2 * i, 0x2000);
        put(g, 0xdf0c + 2 * i, 0);
    }
    for i in 0..18 {
        put(g, 0x1997 + 2 * i, 1);
    }
    let on = variant & 1 != 0;
    for (a, v) in [
        (ram::samus_pose, 1),
        (ram::samus_y_radius, 8),
        (ram::samus_x_pos, if on { 1024 } else { 24 }),
        (ram::samus_y_pos, if on { 2200 } else { 552 }),
        (ram::collected_items, if on { 0xffff } else { 0 }),
        (ram::equipped_items, 0),
        (ram::equipped_beams, 0),
        (ram::joypad1_newkeys, if on { 0xffff } else { 0 }),
        (ram::power_bomb_explosion_status, if on { 1 } else { 0 }),
        (
            ram::tourian_entrance_statue_finished,
            if on { 0x8000 } else { 0 },
        ),
        (0xe50, if on { 4 } else { 0 }),
        (0xe52, 4),
        (0x1978, 0x200),
        (0x197a, if on { 0xffff } else { 0 }),
        (0x9c2, if on { 99 } else { 20 }),
        (0x9c4, 99),
        (0x9c6, if on { 10 } else { 2 }),
        (0x9c8, 10),
    ] {
        put(g, a, v);
    }
    g.ram().fill(0xd820..0xd827, if on { 0xff } else { 0 });
    g.ram().fill(0xd828..0xd82f, if on { 0xff } else { 0 });
    g.ram().fill(0xd830..0xd831, if on { 0xff } else { 0 });
    g.ram().fill(0xd870..0xd871, if on { 0xff } else { 0 });
    g.ram().fill(0xd8b0..0xd8b1, if on { 0xff } else { 0 });
    // An aligned terminating scroll stream; script operands stay checked.
    rom_word(image, 0x8fa000, 0x0100);
    rom_word(image, 0x8fa002, 0x0080);
    for (a, v) in [
        (0x849100, 1),
        (0x849102, 0x9200),
        (0x849104, 0),
        (0x849106, 0),
        (0x849108, 0),
    ] {
        rom_word(image, a, v);
    }
    // Beam palette pointer and backing palette are synthetic, aligned, inbounds.
    rom_word(image, 0x90c3c9 + 2, 0xa000);
    for i in 0..16 {
        rom_word(image, 0x90a000 + 2 * i, 0x1234 + i as u16);
    }
}
unsafe fn pre_case(id: u32, g: &mut Game, image: &RomImage) -> (u8, u16) {
    let n = id - 318;
    let variant = (n % 6) as usize;
    dispatch_seed(g, image, variant);
    let ea = PRE_ACTIONS[(n / 6) as usize];
    put(g, 0x1d77, [0, 1, 0x100, 0x200, 0x300, 0xff][variant]);
    if ea == 0x84b846 {
        put(g, 0x1d77, [0, 6, 12, 18, 1, 7][variant]);
    }
    if ea == 0x84d15c {
        put(
            g,
            0x10002 + 2 * (8 * 16 + 4),
            if variant & 1 != 0 { 0xff } else { 0 },
        );
    }
    {
        mechanisms::checks::pre(ea, 0, g)
    };
    (0, 0)
}
unsafe fn command_case(id: u32, g: &mut Game, image: &RomImage) -> (u8, u16) {
    let n = id - PRE_END;
    let variant = (n % 2) as usize;
    dispatch_seed(g, image, variant);
    let ea = MORE_COMMANDS[(n / 2) as usize];
    put(g, 0x1d77, if variant == 0 { 0 } else { 1 });
    if matches!(ea, 0x848b55 | 0x848b93) {
        put(g, 0x1dc7, 0xa000);
    }
    if ea == 0x84bbf0 {
        put(g, 0x1aff + 34, 0x442);
    }
    if ea == 0x848cf1 && variant == 1 {
        put(g, 0x1c1f, 23);
        put(g, 0x5f9, 2);
    }
    // Common frame renderer uses its safe aligned ROM stream, not item-frame tables.
    if ea == 0x84e07f {
        put(g, 0xde6c, 0xa000);
    }
    result(unsafe {
        mechanisms::checks::command(
            ea,
            EnemyScript::at(image.clone(), (bank_base(0x84) + 0x9100) as usize),
            0,
            g,
        )
    })
}

unsafe fn native_effect_case(id: u32, g: &mut Game, image: &RomImage) -> (u8, u16) {
    let n = id - 864;
    dispatch_seed(g, image, (n & 1) as usize);
    for i in 0..18 {
        put(g, 0x1997 + 2 * i, 0);
    }
    put(g, 0x1c27, if n & 1 == 0 { 0 } else { 78 });
    put(g, 0x1c87 + 78, 0x482);
    let mut definition = |address: u32, init: u32| {
        for (offset, value) in [
            (0, init as u16),
            (2, 0x8000),
            (4, 0x9200),
            (6, 0x0808),
            (8, 0x3000),
            (10, 0),
            (12, 0),
        ] {
            rom_word(image, address + offset, value);
        }
    };
    if n < 4 {
        definition(0x86e659, 8840661);
        definition(0x86e675, 8840666);
        let value = {
            mechanisms::checks::setup_entry(
                if n < 2 { 8701630 } else { 8701643 },
                0,
                &mut crate::game::mechanism_displacement::Displacement::Absent,
                0x8da0,
                g,
            )
        };
        assert_ne!(
            u16::from_le_bytes([
                g.ram().read_byte(0x1997 + 34),
                g.ram().read_byte(0x1998 + 34)
            ]),
            0
        );
        return (4, u16::from(value));
    }
    let opcode = if n < 12 {
        definition(0x86a993, 8824676);
        for i in 0..16 {
            rom_word(image, 0x86a7ab + 2 * i, 0x9300 + 2 * i as u16);
        }
        rom_word(image, 0x849100, [0, 2, 14, 16][((n - 4) / 2) as usize]);
        8704855
    } else if n < 14 {
        definition(0x86b751, 8828547);
        rom_word(image, 0x849100, if n & 1 == 0 { 0 } else { 2 });
        8705936
    } else {
        definition(0x86e517, 8840358);
        for i in 0..48 {
            rom_word(image, 0x86e42c + 2 * i, 0x9400 + 2 * i as u16);
        }
        for i in 0..32 {
            rom_word(image, 0x86e47e + 2 * i, 3 + i as u16);
        }
        put(g, 0x5e5, 0x3456);
        if n & 1 == 0 { 8705951 } else { 8705974 }
    };
    let returned = result(unsafe {
        mechanisms::checks::command(
            opcode,
            EnemyScript::at(image.clone(), (bank_base(0x84) + 0x9100) as usize),
            0,
            g,
        )
    });
    assert_ne!(
        u16::from_le_bytes([
            g.ram().read_byte(0x1997 + 34),
            g.ram().read_byte(0x1998 + 34)
        ]),
        0
    );
    returned
}

// Actual message presentation and scheduler suspension with real bus objects.
// Each call records RAM, serialized PPU/DMA/SNES fields and the ordered APU queue.
unsafe fn message_case(id: u32, g: &mut Game, image: &RomImage, trace: &mut Vec<u8>) -> (u8, u16) {
    let door = (1009..=1011).contains(&id);
    let variant = if (1020..=1021).contains(&id) {
        id - 1014
    } else if door {
        0
    } else {
        id - 880
    };
    let previous_cpu = g.g_cpu;
    g.ensure_backend();
    let snes = g.g_snes;
    let ppu = unsafe { (*snes).ppu };
    unsafe {
        (*ppu).vramIncrement = 1;
    }
    g.input1_mut().type_0 = 0;
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        // Both configs point to synthetic, nonempty, aligned 64-byte message data.
        for index in 19..=21 {
            let at = 0x85869b + index * 6;
            rom_word(image, at, 8750134u32 as u16);
            rom_word(image, at + 2, 8749705u32 as u16);
            rom_word(image, at + 4, 0xa000 + ((index - 19) * 64) as u16);
        }
        for i in 0..96 {
            rom_word(image, 0x85a000 + 2 * i, 0x2800 + i as u16);
        }
        for i in 0..40 {
            slot(g, i, 0);
        }
        slot(g, 39, 0x8000);
        if variant == 6 {
            // Actual grey-door dud pre: emit sound for a hit, then clear Counter.
            put(g, 0x1cd7 + 78, 8699420u32 as u16);
        }
        rom_word(image, 0x849000, 8686735u32 as u16); // ActivateMapStation.
        rom_word(image, 0x849002, 8685390u32 as u16); // SetTimer 7; Sleep.
        rom_word(image, 0x849004, 0xb407);
        rom_word(image, 0x849006, 0x0086);
        assert_eq!(
            crate::ram_map::checked_word!(g.music_queue_read_pos),
            crate::ram_map::checked_word!(g.music_queue_write_pos)
        );
        if door {
            g.ram().write_byte(0x612, 2);
            put(g, crate::ram_map::words::door_direction, (id - 1009) as u16);
            put(g, crate::ram_map::words::door_transition_function, 0xe4a9);
        }
        if variant == 7 {
            callback_edges::begin_message();
        }
        let mut final_return = 0;
        for call in 0..80u32 {
            // Alternate released/held A so the real edge detector can dismiss.
            g.input1_mut().currentState = if call & 1 == 0 {
                0x0080u16.reverse_bits()
            } else {
                0
            };
            let returned = if door {
                let poll = crate::game::room_entry::protocol::dispatch(
                    &mut crate::game::session_runtime::Runtime::new(g),
                    8578217,
                );
                poll.returned_byte()
            } else {
                unsafe { mechanisms::run(g) }
            };
            if variant == 7 {
                assert_eq!(
                    g.ram().read_byte(0x613),
                    u8::from(returned != 0),
                    "only the scheduler return exports Pass"
                );
            }
            trace.push(returned);
            trace.extend_from_slice(&g.message_box_das0l_value.to_le_bytes());
            trace.extend_from_slice(&ram_bytes(&g));
            let mut hardware = crate::recording::RecordingBytes::default();
            let mut io = crate::snapshot_io::SnapshotIo::Save(&mut hardware);
            unsafe {
                io.state(&mut *ppu).unwrap();
                io.state(&mut g.dma_state()).unwrap();
                io.state(&mut *snes).unwrap();
            }
            trace.extend_from_slice(hardware.as_slice());
            trace
                .extend_from_slice(&crate::audio_snapshot::ApuQueueSnapshot::capture(g).to_bytes());
            if call == 0 {
                assert_ne!(returned, 0, "message did not suspend");
                if variant == 6 {
                    assert_eq!(
                        u16::from_le_bytes([
                            g.ram().read_byte(0x1d77 + 78),
                            g.ram().read_byte(0x1d78 + 78)
                        ]),
                        0
                    );
                }
            }
            if returned == 0 {
                final_return = returned;
                break;
            }
            if call == 0 {
                match variant {
                    1 => {
                        put(g, 0x1c23, 0);
                        put(g, 0x1c37 + 78, 0);
                    }
                    2 => {
                        put(g, 0x1c27, 0);
                        put(g, 0x1d27, 0x9002);
                    }
                    3 => {
                        put(g, 0xdc8, 21);
                    }
                    4 => {
                        let replaced = RomImage::zeroed(0x200000);
                        rom_word(&replaced, 0x849002, 8685390u32 as u16);
                        rom_word(&replaced, 0x849004, 0xb409);
                        rom_word(&replaced, 0x849006, 0x0086);
                        g.loaded_rom = Some(replaced);
                    }
                    5 => {
                        g.ram().write_byte(0x614, 255);
                    }
                    6 => {
                        // A repeated pre would clear this high byte; SetTimer writes only low.
                        put(g, 0x1d77 + 78, 0x5500);
                        put(g, 0xde1c + 78, 13);
                    }
                    _ => {}
                }
            }
            assert!(call < 79, "message exceeded bounded completion");
        }
        if variant == 7 {
            let edges = callback_edges::finish_message();
            trace.extend_from_slice(b"613E");
            trace.extend_from_slice(&(edges.len() as u32).to_le_bytes());
            trace.extend_from_slice(&edges);
        }
        if variant == 6 {
            assert_eq!(
                &ram_span::<2>(&g, 0x1d77 + 78),
                &0x5507u16.to_le_bytes(),
                "resumed pre-action changed counter"
            );
            assert_eq!(
                &ram_span::<2>(&g, 0xde1c + 78),
                &13u16.to_le_bytes(),
                "resumed scheduler changed delay"
            );
        }
        if door {
            assert_eq!(
                g.ram().read_byte(0x612),
                0,
                "door continuation retained after completion"
            );
            assert_eq!(
                crate::ram_map::checked_word!(g.door_transition_function),
                0xe659
            );
            assert_eq!(
                crate::ram_map::checked_word!(g.samus_x_pos),
                if id == 1009 { 31 } else { 24 }
            );
        }
        g.g_snes = core::ptr::null_mut();
        (3, u16::from(final_return))
    }));
    g.g_cpu = previous_cpu;
    let _hardware = unsafe { g.take_backend_for_shutdown() };
    match outcome {
        Ok(value) => value,
        Err(error) => std::panic::resume_unwind(error),
    }
}

unsafe fn consumer_case(id: u32, g: &mut Game, image: &RomImage) -> (u8, u16) {
    use crate::ram_map::words as ram;
    let n = id - 886;
    if n < 6 {
        for i in 0..40 {
            slot(g, i, if n == 4 { 0x8000 } else { 0 });
            put(g, 0x1c87 + 2 * i, 0);
        }
        put(g, ram::door_def_ptr, 0xa000);
        put(g, ram::door_direction, 0);
        rom_word(image, 0x83a004, 0x0203);
        rom_word(image, 0x8fe68a, if n == 5 { 0 } else { 0x8000 });
        rom_word(image, 0x848004, 0x9300);
        if matches!(n, 1 | 2 | 3) {
            slot(g, 3, 0x8000);
            put(g, 0x1c87 + 6, 70);
            slot(g, 39, if n == 2 { 0 } else { 0x8000 });
            put(g, 0x1c87 + 78, 70);
            if n == 3 {
                put(g, 0x1dc7 + 78, 0);
                g.ram().write_byte(0xd8b0, 1);
            }
        }
        unsafe { mechanisms::close_door(g) };
    } else if n < 10 {
        for i in 0..40 {
            slot(g, i, 0);
        }
        if n == 7 {
            put(g, 0x1c37, 0xd6ea);
        }
        if n == 8 {
            put(g, 0x1c37 + 78, 0xd6ea);
        }
        let actor = if n == 9 { 64 } else { 0 };
        put(
            g,
            0xf78
                + actor as usize
                + core::mem::offset_of!(crate::types::EnemyData, current_instruction),
            0xffff,
        );
        crate::game::sm_aa::CallTorizoFunc(11192006, actor, g);
    } else if n < 14 {
        let slot = if n & 1 == 0 { 0 } else { 39 };
        put(g, 0x1d77 + 2 * slot, 0xfffe);
        mechanisms::add_glass_hits(slot, [0, 1, 0xffff, 0x8000][(n - 10) as usize], g);
    } else if n < 22 {
        let v = n - 14;
        put(g, ram::area_index, if v & 2 == 0 { 0 } else { 3 });
        g.ram()
            .write_byte(0xd828 + 3, if v & 4 == 0 { 0 } else { 1 });
        put(g, 0xb2e, if v == 7 { 1 } else { 0 });
        mechanisms::carry(
            &mut crate::game::session_runtime::Runtime::new(g),
            if v & 1 == 0 {
                mechanisms::Direction::Right
            } else {
                mechanisms::Direction::Left
            },
        );
    } else {
        let v = n - 22;
        for i in 0..40 {
            slot(g, i, 0);
        }
        slot(
            g,
            39,
            [0, 0xdf88, 0xdf89, 0xffff, 0xdf89, 0xdf89][v as usize],
        );
        put(g, 0xdf0c + 78, 0);
        put(g, 0x1dc7 + 78, 0);
        put(g, ram::roomdefroomstate_ptr, 0xa100);
        if v == 4 {
            g.ram().write_byte(0xd870, 1);
        }
        rom_word(image, 0x84839d, 0xa000);
        rom_word(image, 0x84a002, if v == 5 { 0x923 } else { 0x123 });
        unsafe { crate::game::xray_background::reveal_room_items(g) };
    }
    (0, 0)
}

unsafe fn boundary_case(id: u32, g: &mut Game, image: &RomImage) -> (u8, u16) {
    let n = id - 914;
    if n < 84 {
        let selector = [78, 79, 80, 0x280, 0x8000, 0xffff][(n % 6) as usize];
        // Broad/odd selectors are accepted by these original entries. Their
        // destination words can alias another owned lane or neighboring RAM.
        result(unsafe {
            mechanisms::checks::command(
                COMMANDS[(n / 6) as usize],
                EnemyScript::at(image.clone(), (bank_base(0x84) + 0x9100) as usize),
                selector,
                g,
            )
        })
    } else {
        let target = [
            0x1c22, 0x1c24, 0x1c26, 0x1e66, 0x1e68, 0xde1a, 0xde1c, 0xc6c6, 0xc6c8, 0xc8c6, 0xc8c8,
        ][(n - 84) as usize];
        put(g, 0x1e17, target as u16);
        result(unsafe {
            mechanisms::checks::command(
                8706958,
                EnemyScript::at(image.clone(), (bank_base(0x84) + 0x9100) as usize),
                1,
                g,
            )
        })
    }
}

// Run only in separate bounded processes: 1012/1013 terminate through the
// native diagnostic,1014 never returns. The timeout observes nonreturn only;
// the cleared trigger's RAM prefix is established by source inspection.
unsafe fn nonreturn_case(id: u32, g: &mut Game) -> (u8, u16) {
    for i in 0..40 {
        slot(g, i, 0x8000);
        put(g, 0x1c87 + 2 * i, 0x444);
    }
    put(g, 0x1c87, 0x442);
    put(g, crate::ram_map::words::samus_collision_direction, 3);
    let address = match id {
        1012 => 8696803,
        1013 => 8697028,
        1014 => 8696723,
        _ => unreachable!(),
    };
    eprintln!("ROOM_MECHANISM_NONRETURN {id} setup={address:06x}");
    let value = {
        mechanisms::checks::setup_entry(
            address,
            0,
            &mut crate::game::mechanism_displacement::Displacement::Absent,
            0,
            g,
        )
    };
    panic!("nonreturn case {id} unexpectedly returned {value}");
}

unsafe fn displacement_alias_case(id: u32, g: &mut Game) -> (u8, u16) {
    let at = [0x1c25, 0x1d75, 0x1e65][(id - 1015) as usize];
    g.ram().copy_in(at, &0x12345678i32.to_ne_bytes());
    put(g, crate::ram_map::words::samus_collision_direction, 3);
    put(g, crate::ram_map::words::samus_y_dir, 0);
    put(g, 0xa6e, 0);
    let value = {
        mechanisms::checks::setup_entry(
            8697028,
            0,
            &mut crate::game::mechanism_displacement::Displacement::RamOffset(at),
            0,
            g,
        )
    };
    assert_eq!(&ram_span::<4>(&g, at), &0x12003078i32.to_ne_bytes());
    (4, u16::from(value))
}

// Opposite map bytes distinguish the two persistent-flag query identities.
// PrepareBitAccess is observable even though the queries do not change flags.
unsafe fn distinct_flag_query_case(id: u32, g: &mut Game, image: &RomImage) -> (u8, u16) {
    let object_query = id < 1035;
    let taken = id == 1033 || id == 1035;
    let selected_base = if object_query { 0xd830 } else { 0xd870 };
    let other_base = if object_query { 0xd870 } else { 0xd830 };
    // Bit21 selects byte2, mask20; the other map has exactly the opposite result.
    put(g, 0x1dc7, 21);
    put(g, 0x5e7, 0xa55a);
    g.ram()
        .write_byte(selected_base + 2, if taken { 0x20 } else { 0x40 });
    g.ram()
        .write_byte(other_base + 2, if taken { 0x40 } else { 0x20 });
    rom_word(image, 0x849100, 0x9234);
    let mut expected_ram = ram_bytes(&g);
    expected_ram[0x5e7..0x5e9].copy_from_slice(&0x20u16.to_le_bytes());
    let returned = result(unsafe {
        mechanisms::checks::command(
            if object_query { 8685640 } else { 8685692 },
            EnemyScript::at(image.clone(), (bank_base(0x84) + 0x9100) as usize),
            0,
            g,
        )
    });
    assert_eq!(
        returned,
        if taken { (1, 0x9234) } else { (2, 0x9102) },
        "persistent query selected wrong flag map"
    );
    assert_eq!(
        &ram_bytes(&g),
        &expected_ram[..],
        "persistent query must write only its two-byte scratch mask"
    );
    returned
}

pub(crate) fn capture_case(id: u32) -> Option<Vec<u8>> {
    if id == 1037 {
        return Some(getter_checks::capture(&mut Game::new()));
    }
    if id >= CASES {
        return None;
    }
    let mut g = Game::new();
    let image = RomImage::zeroed(0x200000);
    seed(&mut g, &image);
    g.loaded_rom = Some(image.clone());
    g.register_native_rom(image.clone());
    if (66..=81).contains(&id) || id == 63 || id >= 318 {
        assert_eq!(
            (image.as_mut_ptr() as usize + crate::rtl::lorom_offset(0x84a000) as usize) & 1,
            0,
            "native ROM stream alignment"
        );
    }
    let mut displacement = None;
    let mut trace = Vec::new();
    let outcome = catch_unwind(AssertUnwindSafe(|| unsafe {
        if id >= 1033 {
            distinct_flag_query_case(id, &mut g, &image)
        } else if id >= 1022 {
            drawing_extended::run(id, &mut g, &image, &mut trace)
        } else if id >= 1020 {
            message_case(id, &mut g, &image, &mut trace)
        } else if id == 1019 {
            assert!(mechanisms::qualified_checks(&mut g));
            (5, 2)
        } else if id == 1018 {
            assert!(mechanisms::ownership_checks(&mut g));
            (5, 1)
        } else if id >= 1015 {
            displacement_alias_case(id, &mut g)
        } else if id >= 1012 {
            nonreturn_case(id, &mut g)
        } else if id >= 1009 {
            message_case(id, &mut g, &image, &mut trace)
        } else if id >= 914 {
            boundary_case(id, &mut g, &image)
        } else if id >= 886 {
            consumer_case(id, &mut g, &image)
        } else if id >= 880 {
            message_case(id, &mut g, &image, &mut trace)
        } else if id >= 864 {
            native_effect_case(id, &mut g, &image)
        } else if id >= PRE_END {
            command_case(id, &mut g, &image)
        } else if id >= 318 {
            pre_case(id, &mut g, &image)
        } else if id >= 90 {
            let (returned, movement) = setup_case(id, &mut g);
            displacement = Some(movement);
            (4, u16::from(returned))
        } else {
            execute(id, &mut g, &image)
        }
    }));
    assert_eq!(
        outcome.is_err(),
        matches!(id, 82 | 84 | 85),
        "unexpected result for baseline case {id}"
    );
    assert!(!g.g_fail, "native diagnostic for baseline case {id}");
    let mut bytes = ram_bytes(&g);
    match outcome {
        Ok((kind, value)) => {
            bytes.push(kind);
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        Err(_) => bytes.extend_from_slice(&[0xff, 0, 0]),
    };
    bytes.push(u8::from(g.g_fail));
    if let Some(value) = displacement {
        bytes.extend_from_slice(&value.to_ne_bytes());
    }
    bytes.extend_from_slice(&trace);
    Some(bytes)
}
