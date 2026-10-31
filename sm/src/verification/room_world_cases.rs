//! Prepared-world cases through the actual GameOwner and retained hardware.
//! Caller initializes this owner with SnesInit and verifier audio in a fresh child.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::room_background_support::{ram_bytes, ram_span};
use crate::{
    game::{
        room_entry::{self, ArrivalRequest, EntryPoll, StartSite, world::Identity},
        session_runtime::Runtime,
    },
    game_state::{Game, GameOwner},
    room_world_fixture as fixture, room_world_resources,
};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
};

#[cfg(sm_verify_dll)]
pub(crate) const CASES: u32 = 107;
#[cfg(not(sm_verify_dll))]
pub(crate) const CASES: u32 = 11;
fn put(g: &mut Game, at: usize, value: u16) {
    g.ram().copy_in(at, &value.to_le_bytes());
}
fn word(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes(ram_span::<2>(&g, at))
}
fn record(out: &mut Vec<u8>, phase: u8, g: &Game) {
    out.push(phase);
    out.extend_from_slice(&ram_bytes(&g));
    out.push(u8::from(g.g_fail));
}
/// Uses the real constructor route; exported DLL initializes hardware via owner.with.
pub(crate) fn owner(id: u32) -> Box<GameOwner> {
    #[cfg(sm_verify_dll)]
    if (14..16).contains(&id) {
        return GameOwner::with_world(crate::game::actors::composed_fixture::binding(id == 15));
    }
    #[cfg(sm_verify_dll)]
    if (12..14).contains(&id) {
        return GameOwner::with_world(crate::game::actors::composed_fixture::binding(id == 13));
    }
    if (2..=5).contains(&id) {
        return GameOwner::with_world(room_world_resources::binding_with_tables(id & 1 != 0));
    }
    GameOwner::with_world(fixture::binding(id & 1 != 0))
}
pub(crate) fn capture(id: u32, owner: &GameOwner) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    catch_unwind(AssertUnwindSafe(|| owner.with(|g| unsafe { run(id, g) }))).ok()
}
unsafe fn nmi_and_draw(g: &mut Game) {
    unsafe {
        crate::game::display_refresh::run(g);
        crate::cpu_infra::draw_frame_to_ppu(g);
    }
}
unsafe fn run(id: u32, g: &mut Game) -> Vec<u8> {
    unsafe {
        #[cfg(sm_verify_dll)]
        if id == 105 {
            // Movies cannot supply divergent cached/per-layer routing and staged/effective PPU state.
            crate::rtl::rtl_reset(2, g);
            let snes = g.g_snes;
            assert!(!snes.is_null());
            let ppu = (*snes).ppu;
            assert!(ppu == (*snes).my_ppu || ppu == (*snes).snes_ppu);
            let other = if ppu == (*snes).my_ppu {
                (*snes).snes_ppu
            } else {
                (*snes).my_ppu
            };
            g.ram().copy_in(0x5a, &[0x11, 0x22]);
            g.ram().copy_in(0x69, &[0x33, 0x44]);
            g.ram().copy_in(0x6e, &[0x55, 0x66, 0x77, 0x88, 0x99, 0xaa]);
            g.ram().copy_in(0x84, &[0xbb, 0xcc]);
            (*ppu).screenEnabled[0] = 0xe5;
            for (layer, enabled) in (*ppu)
                .layer
                .iter_mut()
                .zip([false, true, false, false, true])
            {
                layer.mainScreenEnabled = enabled;
            }
            (*other).screenEnabled[0] = 0xc3;
            (*other).bgLayer[2].tilemapAdr = 0x1800;
            assert_eq!((*ppu).screenEnabled[0], 0xe5);
            assert_eq!(
                (*ppu).layer.map(|layer| layer.mainScreenEnabled),
                [false, true, false, false, true]
            );
            g.execution().write(0x2109, 0xe7);
            assert_eq!(
                (
                    (*ppu).bgLayer[2].tilemapAdr,
                    (*ppu).bgLayer[2].tilemapWider,
                    (*ppu).bgLayer[2].tilemapHigher
                ),
                (0xe400, true, true)
            );
            assert_eq!((*ppu).screenEnabled[0], 0xe5);
            assert_eq!(
                (*ppu).layer.map(|layer| layer.mainScreenEnabled),
                [false, true, false, false, true]
            );
            g.execution().write(0x2130, 0xdb);
            assert_eq!(((*ppu).clipMode, (*ppu).preventMathMode), (3, 1));
            assert!((*ppu).addSubscreen && (*ppu).directColor);
            g.execution().write(0x2131, 0xd5);
            assert_eq!((*ppu).mathEnabled, [true, false, true, false, true, false]);
            assert_eq!(((*ppu).subtractColor, (*ppu).halfColor), (true, true));
            g.execution().write(0x212c, 0xb6);
            assert_eq!((*ppu).screenEnabled[0], 0xb6);
            assert_eq!(
                (*ppu).layer.map(|layer| layer.mainScreenEnabled),
                [false, true, true, false, true]
            );
            g.execution().write(0x2100, 0x8b);
            assert_eq!(((*ppu).forcedBlank, (*ppu).brightness), (true, 11));
            g.execution().write(0x2109, 0x58);
            assert_eq!(
                (
                    (*ppu).bgLayer[2].tilemapAdr,
                    (*ppu).bgLayer[2].tilemapWider,
                    (*ppu).bgLayer[2].tilemapHigher
                ),
                (0x5800, false, false)
            );
            g.execution().write(0x2130, 0x6c);
            assert_eq!(((*ppu).clipMode, (*ppu).preventMathMode), (1, 2));
            assert!(!(*ppu).addSubscreen && !(*ppu).directColor);
            g.execution().write(0x2131, 0x2a);
            assert_eq!((*ppu).mathEnabled, [false, true, false, true, false, true]);
            assert_eq!(((*ppu).subtractColor, (*ppu).halfColor), (false, false));
            g.execution().write(0x2100, 0x07);
            assert_eq!(((*ppu).forcedBlank, (*ppu).brightness), (false, 7));
            assert_eq!(ram_span::<2>(g, 0x5a), [0x11, 0x22]);
            assert_eq!(ram_span::<2>(g, 0x69), [0x33, 0x44]);
            assert_eq!(ram_span::<6>(g, 0x6e), [0x55, 0x66, 0x77, 0x88, 0x99, 0xaa]);
            assert_eq!(ram_span::<2>(g, 0x84), [0xbb, 0xcc]);
            assert_eq!(
                ((*other).screenEnabled[0], (*other).bgLayer[2].tilemapAdr),
                (0xc3, 0x1800)
            );
            assert_eq!((*snes).ppu, ppu);
            assert!(!g.g_fail);
            let mut out = b"RMWD1".to_vec();
            out.extend_from_slice(&id.to_le_bytes());
            record(&mut out, 0, g);
            return out;
        }
        #[cfg(sm_verify_dll)]
        if id == 106 {
            // Movies cannot author odd beam positions, high-byte-first counter reads or OAM cursor wrap.
            crate::rtl::rtl_reset(2, g);
            let snes = g.g_snes;
            assert!(!snes.is_null());
            let ppu = (*snes).ppu;
            assert!(ppu == (*snes).my_ppu || ppu == (*snes).snes_ppu);
            let other = if ppu == (*snes).my_ppu {
                (*snes).snes_ppu
            } else {
                (*snes).my_ppu
            };
            (*snes).hPos = 0x52b;
            (*snes).vPos = 0x1d3;
            (*snes).openBus = 0xa6;
            (*snes).ppuLatch = false;
            (*ppu).hCount = 0xbeef;
            (*ppu).vCount = 0xdead;
            (*ppu).hCountSecond = true;
            (*ppu).vCountSecond = true;
            (*ppu).ppu2openBus = 0xa4;
            (*other).hCount = 0x1234;
            (*other).vCount = 0x5678;
            assert_eq!(g.execution().read(0x2137), 0xa6);
            assert_eq!(((*ppu).hCount, (*ppu).vCount), (0x014a, 0x01d3));
            assert!((*ppu).countersLatched && (*ppu).hCountSecond && (*ppu).vCountSecond);
            assert!(!(*snes).ppuLatch);
            assert_eq!(g.execution().read(0x213c), 0xa5);
            assert_eq!(g.execution().read(0x213c), 0x4a);
            assert_eq!(g.execution().read(0x213d), 0x4b);
            assert_eq!(g.execution().read(0x213d), 0xd3);
            assert_eq!(g.execution().read(0x213f), 0x43);
            assert!(!(*ppu).countersLatched && !(*ppu).hCountSecond && !(*ppu).vCountSecond);
            (*snes).hPos = 0x487;
            (*snes).vPos = 0x123;
            g.execution().write(0x4201, 0x80);
            assert!((*snes).ppuLatch && !(*ppu).countersLatched);
            assert_eq!(((*ppu).hCount, (*ppu).vCount), (0x014a, 0x01d3));
            g.execution().write(0x4201, 0);
            assert!(!(*snes).ppuLatch && (*ppu).countersLatched);
            assert_eq!(((*ppu).hCount, (*ppu).vCount), (0x0121, 0x0123));
            (*snes).hPos = 7;
            (*snes).vPos = 2;
            g.execution().write(0x4201, 0);
            assert_eq!(((*ppu).hCount, (*ppu).vCount), (0x0121, 0x0123));
            g.execution().write(0x2102, 0xff);
            g.execution().write(0x2103, 1);
            (*ppu).highOam[0x1e] = 0x5a;
            (*ppu).highOam[0x1f] = 0xc3;
            assert_eq!(((*ppu).oamAdr, (*ppu).oamAdrWritten), (0xff, 0xff));
            assert!((*ppu).oamInHigh && (*ppu).oamInHighWritten && !(*ppu).oamSecondWrite);
            assert_eq!(g.execution().read(0x2138), 0x5a);
            assert_eq!(g.execution().read(0x2138), 0xc3);
            assert_eq!(((*ppu).oamAdr, (*ppu).oamAdrWritten), (0, 0xff));
            assert!(!(*ppu).oamInHigh && (*ppu).oamInHighWritten && !(*ppu).oamSecondWrite);
            g.execution().write(0x2100, 0x80);
            (*snes).hPos = 0;
            (*snes).vPos = 225;
            g.execution().advance_beam();
            assert_eq!(((*ppu).oamAdr, (*ppu).oamAdrWritten), (0, 0xff));
            assert!(!(*ppu).oamInHigh && (*ppu).oamInHighWritten);
            g.execution().write(0x2100, 0);
            (*snes).hPos = 0;
            g.execution().advance_beam();
            assert_eq!(((*ppu).oamAdr, (*ppu).oamAdrWritten), (0xff, 0xff));
            assert!((*ppu).oamInHigh && (*ppu).oamInHighWritten && !(*ppu).oamSecondWrite);
            assert_eq!(((*other).hCount, (*other).vCount), (0x1234, 0x5678));
            assert_eq!((*snes).ppu, ppu);
            assert!(!g.g_fail);
            let mut out = b"RMWD1".to_vec();
            out.extend_from_slice(&id.to_le_bytes());
            record(&mut out, 0, g);
            return out;
        }
        #[cfg(sm_verify_dll)]
        if id == 103 {
            // Movies cannot make atmospheric OAM X/Y overwrite the camera used by this draw.
            use crate::{game::visual_host::Effects, rom::RomImage};
            crate::rtl::rtl_reset(2, g);
            let loaded = RomImage::zeroed(0x90000);
            let native = RomImage::zeroed(0x90000);
            for (image, address, value) in [
                (&loaded, 0x908c01, 0x9400u16),
                (&native, 0x908c01, 0x9600),
                (&loaded, 0x909402, 0xdead),
                (&native, 0x909402, 0xa55a),
                (&native, 0x909602, 0xbeef),
            ] {
                let at = crate::rtl::lorom_offset(address) as usize;
                for (lane, byte) in value.to_ne_bytes().into_iter().enumerate() {
                    image.write(at + lane, byte);
                }
            }
            g.register_rom(loaded);
            g.register_native_rom(native);
            for (at, value) in [
                (0xaec, 0x0101),
                (0xadc, 136),
                (0xae4, 200),
                (0x911, 0),
                (0x915, 0),
                (0x590, 0x5a5),
            ] {
                put(g, at, value);
            }
            g.ram().fill(0x917..0x919, 0xcc);
            Effects::draw_atmospheric_tile(&mut Runtime::new(g), 0, 1);
            assert_eq!(ram_span::<4>(g, 0x915), [0x84, 0x40, 0x5a, 0xa5]);
            assert_eq!(word(g, 0x590), 0x5a9, "atmosphere.saved-full-odd-cursor");
            assert!(!g.g_fail);
            let mut out = b"RMWD1".to_vec();
            out.extend_from_slice(&id.to_le_bytes());
            record(&mut out, 0, g);
            return out;
        }
        #[cfg(sm_verify_dll)]
        if id == 104 {
            // Movies cannot remove atmospheric sources or reach a whole-word WRAM-end failure.
            use crate::{
                game::{scene::atmosphere::Host, visual_host::Effects},
                rom::RomImage,
            };
            crate::rtl::rtl_reset(2, g);
            let table_at = crate::rtl::lorom_offset(0x908bff) as usize + 2;
            let tile_at = crate::rtl::lorom_offset(0x909400) as usize;
            for phase in 0..6 {
                g.loaded_rom = None;
                g.restore_native_rom_binding(None);
                g.g_fail = false;
                for (at, value) in [
                    (0xaec, 0x0100),
                    (0xadc, if phase == 0 { 3 } else { 136 }),
                    (0xae4, if phase == 1 { 3 } else { 100 }),
                    (0x911, 0),
                    (0x915, 0),
                    (0x590, 0),
                ] {
                    put(g, at, value);
                }
                g.ram().fill(0x370..0x374, 0xcc);
                if phase >= 3 {
                    let loaded = RomImage::zeroed(table_at + if phase == 3 { 1 } else { 2 });
                    loaded.write(table_at, 0);
                    if phase != 3 {
                        loaded.write(table_at + 1, 0x94);
                    }
                    g.register_rom(loaded);
                }
                if phase == 5 {
                    let native = RomImage::zeroed(tile_at + 1);
                    native.write(tile_at, 0x5a);
                    g.register_native_rom(native);
                }
                let result = catch_unwind(AssertUnwindSafe(|| {
                    Effects::draw_atmospheric_tile(&mut Runtime::new(g), 0, 1);
                }));
                assert_eq!(
                    result.is_err(),
                    phase >= 2,
                    "atmosphere.lazy-source-{phase}"
                );
                let expected = match phase {
                    0 => [0xcc; 4],
                    1 => [0x84, 0xcc, 0xcc, 0xcc],
                    _ => [0x84, 0x60, 0xcc, 0xcc],
                };
                assert_eq!(
                    ram_span::<4>(g, 0x370),
                    expected,
                    "atmosphere.prefix-{phase}"
                );
                assert_eq!(word(g, 0x590), 0, "atmosphere.unpublished-cursor-{phase}");
                assert!(!g.g_fail);
            }
            // The direct native Host accepts usize entries; a u16 draw cursor cannot reach this tail.
            g.ram().write_byte(0x1ffff, 0xcc);
            let result = catch_unwind(AssertUnwindSafe(|| {
                let mut runtime = Runtime::new(g);
                let mut tiles = sm_game::player_transitions::atmosphere::SessionTiles(&mut runtime);
                Host::write_tile(&mut tiles, 0x1fc8d, 0xa55a).unwrap();
            }));
            assert!(result.is_err(), "atmosphere.native-word-preflight");
            assert_eq!(
                g.ram().read_byte(0x1ffff),
                0xcc,
                "atmosphere.native-low-byte-retained"
            );
            assert_eq!(word(g, 0x590), 0);
            assert!(!g.g_fail);
            let mut out = b"RMWD1".to_vec();
            out.extend_from_slice(&id.to_le_bytes());
            record(&mut out, 0, g);
            return out;
        }
        // Movies cannot author same-bank cursor wrap or dormant flag variants.
        #[cfg(sm_verify_dll)]
        if id == 101 {
            crate::rtl::rtl_reset(2, g);
            let snes = g.g_snes;
            assert!(!snes.is_null());
            g.ram().write_byte(0xffff, 0x81);
            g.ram().copy_in(0, &[0xa1, 0xa2, 0]);
            g.ram().write_byte(0x10000, 0xb7);
            (*snes).ramAdr = 0x12340;
            let mut dma = g.dma_state();
            for c in &mut dma.channel {
                c.hdmaActive = false;
                c.doTransfer = true;
                c.terminated = true;
            }
            let c = &mut dma.channel[6];
            c.hdmaActive = true;
            c.dmaActive = true;
            c.offIndex = 5;
            c.indirect = false;
            c.fromB = false;
            c.mode = 2;
            c.aBank = 0x7e;
            c.aAdr = 0xffff;
            c.bAdr = 0x80;
            c.size = 0xbeef;
            dma.hdmaTimer = 0x2345;
            dma.dmaTimer = 0x8899aabb;
            dma.dmaBusy = true;
            g.set_dma_state(dma);
            g.execution().initialize_hdma();
            let initialized = g.dma_state();
            assert_eq!(initialized.hdmaTimer, 24);
            assert_eq!(
                (initialized.channel[6].tableAdr, initialized.channel[6].size),
                (0, 0xbeef)
            );
            assert!(!initialized.channel[6].dmaActive && initialized.channel[6].doTransfer);
            assert_eq!(initialized.channel[6].offIndex, 0);
            assert!(!initialized.channel[0].doTransfer && !initialized.channel[0].terminated);
            g.execution().step_hdma();
            let stepped = g.dma_state();
            assert_eq!(ram_span::<2>(&g, 0x12340), [0xa1, 0xa2]);
            assert_eq!((*snes).ramAdr, 0x12342);
            assert_eq!((stepped.hdmaTimer, stepped.channel[6].tableAdr), (40, 3));
            assert_eq!(stepped.channel[6].repCount, 0);
            assert!(stepped.channel[6].terminated && stepped.channel[6].doTransfer);
            assert_eq!((stepped.dmaTimer, stepped.dmaBusy), (0x8899aabb, true));
            g.execution().step_hdma();
            assert_eq!(g.dma_state().hdmaTimer, 0);
            assert_eq!(g.dma_state().channel[6].tableAdr, 3);
            assert_eq!((*snes).ramAdr, 0x12342);
            assert!(!g.g_fail);
            let mut out = b"RMWD1".to_vec();
            out.extend_from_slice(&id.to_le_bytes());
            record(&mut out, 0, g);
            return out;
        }
        // Movies cannot force a byte to rewrite its own mode/direction or invalid mode.
        #[cfg(sm_verify_dll)]
        if id == 102 {
            crate::rtl::rtl_reset(2, g);
            let snes = g.g_snes;
            assert!(!snes.is_null());
            (*snes).ramAdr = 0x12340;
            g.ram().write_byte(0x12340, 3);
            let mut dma = g.dma_state();
            for c in &mut dma.channel {
                c.hdmaActive = false;
            }
            let c = &mut dma.channel[6];
            c.hdmaActive = true;
            c.terminated = false;
            c.doTransfer = true;
            c.dmaActive = true;
            c.offIndex = 5;
            c.indirect = false;
            c.fromB = true;
            c.mode = 0;
            c.aBank = 0;
            c.aAdr = 0x2233;
            c.tableAdr = 0x4360;
            c.bAdr = 0x80;
            c.repCount = 0x81;
            g.set_dma_state(dma);
            g.execution().step_hdma();
            let stepped = g.dma_state();
            assert_eq!(
                (stepped.channel[6].mode, stepped.channel[6].fromB),
                (3, false)
            );
            assert_eq!(g.ram().read_byte(0x12341), 0x80);
            assert_eq!((*snes).ramAdr, 0x12322);
            assert_eq!(
                (stepped.hdmaTimer, stepped.channel[6].tableAdr),
                (56, 0x4365)
            );
            assert!(stepped.channel[6].terminated && stepped.channel[6].doTransfer);
            assert_eq!(stepped.channel[6].repCount, 0);
            let mut invalid = stepped;
            let c = &mut invalid.channel[6];
            c.mode = 8;
            c.terminated = false;
            c.doTransfer = true;
            c.dmaActive = true;
            c.offIndex = 7;
            c.tableAdr = 0x4321;
            c.repCount = 0x12;
            invalid.hdmaTimer = 0xabcd;
            g.set_dma_state(invalid);
            let failure = catch_unwind(AssertUnwindSafe(|| g.execution().step_hdma()))
                .expect_err("invalid native mode must fail at its original lookup");
            assert!(
                failure
                    .downcast_ref::<String>()
                    .unwrap()
                    // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
                    .contains("index out of bounds")
            );
            let failed = g.dma_state();
            assert_eq!(failed.hdmaTimer, 8);
            assert_eq!(
                (failed.channel[6].tableAdr, failed.channel[6].repCount),
                (0x4321, 0x12)
            );
            assert!(!failed.channel[6].dmaActive);
            assert_eq!(failed.channel[6].offIndex, 0);
            assert_eq!((*snes).ramAdr, 0x12322);
            assert!(!g.g_fail);
            let mut out = b"RMWD1".to_vec();
            out.extend_from_slice(&id.to_le_bytes());
            record(&mut out, 0, g);
            return out;
        }
        // Movies cannot supply invalid handlers or arbitrary pending/enable aliases.
        #[cfg(sm_verify_dll)]
        if id == 100 {
            use crate::game::scanline_irq;
            crate::rtl::rtl_reset(2, g);
            let snes = g.g_snes;
            assert!(!snes.is_null());
            let mut out = b"RMWD1".to_vec();
            out.extend_from_slice(&id.to_le_bytes());
            put(g, 0x84, 0xa5cf);
            (*snes).hTimer = 0x145;
            (*snes).vTimer = 0x1ee;
            scanline_irq::enable(false, g);
            assert_eq!(word(g, 0x84), 0xa5ff, "irq.full-enable-word");
            assert_eq!(((*snes).vTimer, (*snes).hTimer), (0, 152));
            assert!(!(*snes).hIrqEnabled && !(*snes).vIrqEnabled);
            scanline_irq::enable(true, g);
            assert!((*snes).hIrqEnabled && (*snes).vIrqEnabled);
            assert_eq!(word(g, 0x84), 0xa5ff);
            scanline_irq::disable(g);
            assert_eq!(word(g, 0x84), 0xa5cf, "irq.high-hdma-byte-preserved");
            assert!((*snes).hIrqEnabled && (*snes).vIrqEnabled);
            put(g, 0xa7, 0xfedc);
            put(g, 0xab, 1);
            scanline_irq::advance(g);
            assert_eq!((word(g, 0xab), word(g, 0xa7)), (0xfedc, 0));
            assert_eq!(((*snes).vTimer, (*snes).hTimer), (0, 0));
            put(g, 0x84, 0xa5ff);
            put(g, 0xa7, 0xcdef);
            put(g, 0xab, 3);
            scanline_irq::advance(g);
            assert_eq!((word(g, 0xab), word(g, 0xa7)), (0, 0xcdef));
            assert_eq!(word(g, 0x84), 0xa5cf);
            put(g, 0xab, 0xffff);
            put(g, 0xa7, 0xbeef);
            put(g, 0x84, 0xa577);
            (*snes).hTimer = 0x123;
            (*snes).vTimer = 0x134;
            let failure = catch_unwind(AssertUnwindSafe(|| scanline_irq::advance(g)))
                .expect_err("invalid IRQ must fail before effects");
            assert_eq!(
                failure.downcast_ref::<String>().map(String::as_str),
                Some("invalid scanline IRQ handler: 32767")
            );
            assert_eq!(
                (word(g, 0xab), word(g, 0xa7), word(g, 0x84)),
                (0xffff, 0xbeef, 0xa577)
            );
            assert_eq!(((*snes).hTimer, (*snes).vTimer), (0x123, 0x134));
            assert!(!g.g_fail);
            record(&mut out, 0, g);
            return out;
        }
        #[cfg(sm_verify_dll)]
        if (97..100).contains(&id) {
            return crate::player_collision_owned_cases::run(id - 97, g);
        }
        #[cfg(sm_verify_dll)]
        if (14..97).contains(&id) {
            return match id {
                14 | 15 => crate::player_collision_composed_route::run(id, g),
                16..=51 => crate::player_collision_movement_cases::run(id - 16, g),
                52..=75 => crate::player_collision_pose_cases::run(id - 52, g),
                76..=87 => crate::cpu_infra::player_collision_ownership::run(id - 76, g),
                88..=96 => crate::player_collision_walked_cases::run(id - 88, g),
                _ => unreachable!(),
            };
        }
        #[cfg(sm_verify_dll)]
        if (12..14).contains(&id) {
            return crate::actor_composed_route::run(id, g);
        }
        #[cfg(sm_verify_dll)]
        if id == 11 {
            return crate::cpu_infra::actor_mutable_cpu_checks::run(g);
        }
        let world = g.world().clone();
        assert_eq!(
            g.g_runmode,
            crate::cpu_infra::RM_MINE as u8,
            "room.world.constructor-play-mode"
        );
        assert!(!g.g_snes.is_null(), "room.world.real-hardware");
        let mut out = b"RMWD1".to_vec();
        out.extend_from_slice(&id.to_le_bytes());
        crate::rtl::rtl_reset(2, g);
        assert!(!g.g_fail, "room.world.reset-no-diagnostic");
        assert!(Arc::ptr_eq(g.world(), &world), "room.world.reset-binding");
        if id == 8 {
            let before = g.snes_frame_counter;
            crate::rtl::rtl_run_frame(0, g);
            assert_eq!(
                g.snes_frame_counter,
                before + 1,
                "room.world.actual-frame-readiness"
            );
            assert_eq!(
                g.ram().read_byte(0x611),
                2,
                "room.world.actual-reset-frame-entered"
            );
            assert!(Arc::ptr_eq(g.world(), &world), "room.world.frame-binding");
            assert!(!g.g_fail, "room.world.frame-no-diagnostic");
            record(&mut out, 0, g);
            return out;
        }
        // These focused routes begin after reset rather than running Vector_RESET.
        // Install its actual base audio program before any retained music command.
        crate::native_audio::upload_bank(0xcf8000, g);
        assert!(!g.g_fail, "room.world.base-audio-upload-no-diagnostic");
        room_world_resources::install(g);
        assert!(!g.g_fail, "room.world.resources-no-diagnostic");
        if g.world().prepared_room_assets().is_some() {
            if id == 2 {
                room_world_resources::check_assets(g);
                let selected = g.world().clone();
                let assets = selected.prepared_room_assets().unwrap();
                crate::room_background_support::check_prepared_vram(
                    selected.definition().unwrap(),
                    selected.compatibility_keys(),
                    &assets,
                );
                crate::room_background_support::check_prepared_dma(
                    selected.definition().unwrap(),
                    selected.compatibility_keys(),
                    &assets,
                );
            }
        } else if (2..=5).contains(&id) {
            room_world_resources::check_tables(g);
        }
        // Actual player initialization supplies retained draw/movement prerequisites.
        crate::game::session_runtime::initialize_player(g);
        assert!(!g.g_fail, "room.world.player-initialize-no-diagnostic");
        put(g, 0x9c2, 99);
        put(g, 0x9c4, 99);
        put(g, 0x998, 8);
        put(g, 0x5f5, 0);
        if id >= 9 {
            let (site, room, exit, x, y) = if id == 9 {
                (
                    if g.world().prepared_room_assets().is_some() {
                        sm_game::room_entry::world::LegacyStartSite { area: 6, index: 0 }
                    } else {
                        fixture::START_A_SITE
                    },
                    fixture::ROOM_A_KEY,
                    fixture::EXIT_B_KEY,
                    0x1a0,
                    0x230,
                )
            } else {
                (
                    if g.world().prepared_room_assets().is_some() {
                        sm_game::room_entry::world::LegacyStartSite { area: 0, index: 18 }
                    } else {
                        fixture::START_B_SITE
                    },
                    fixture::ROOM_B_KEY,
                    fixture::EXIT_A_KEY,
                    0x3c0,
                    0x450,
                )
            };
            put(g, 0x79f, site.area);
            put(g, 0x78b, site.index);
            put(g, 0x5f7, 0xa534);
            record(&mut out, 0, g);
            room_entry::begin(
                &mut Runtime::new(g),
                ArrivalRequest::Start(StartSite::CurrentLoadStation),
            );
            assert_eq!(word(g, 0x79b), room, "room.world.current-station-room");
            assert_eq!(word(g, 0x78d), exit, "room.world.current-station-exit");
            assert_eq!(
                (word(g, 0xaf6), word(g, 0xafa)),
                (x, y),
                "room.world.current-station-placement"
            );
            assert_eq!(word(g, 0x1e75), 1, "room.world.current-station-lockout");
            assert_eq!(
                word(g, 0x5f7),
                0xa500,
                "room.world.current-station-byte-prefix"
            );
        } else if id <= 1 || (4..=5).contains(&id) {
            // Enter contact from an initialized source room, as gameplay does.
            // Real asset loading installs the retained enemy draw hook and workspace.
            put(g, 0x79b, fixture::ROOM_A_KEY);
            {
                let mut h = Runtime::new(g);
                room_entry::installation::load_header(&mut h);
                room_entry::installation::load_state(&mut h);
            }
            assert_eq!(word(g, 0xe7a), 0, "room.world.source-tile-staging-empty");
            crate::game::actors::prepare_room_assets(&mut Runtime::new(g));
            assert!(!g.g_fail, "room.world.source-assets-no-diagnostic");
            assert_eq!(word(g, 0x178c), 0x804c, "room.world.source-draw-hook");
            assert_eq!(
                g.ram().read_byte(0x178e),
                0xa0,
                "room.world.source-draw-hook-bank"
            );
            // A real one-pixel player movement enters the first authored exit of roomA.
            for (at, value) in [
                (0xaf6, 8),
                (0xaf8, 0),
                (0xafa, 8),
                (0xafc, 0),
                (0xafe, 1),
                (0xb00, 1),
                (0x7a5, 96),
                (0x7a7, 96),
                (0x7a9, 6),
                (0x7ab, 6),
                (0x5b6, 0),
                (0xa1c, 8),
                (0x79b, fixture::ROOM_A_KEY),
                (0x7b5, 0xc000),
                (0x79f, 0),
                (0x99c, 0x7777),
            ] {
                put(g, at, value);
            }
            put(g, 0x10002, 0x9000);
            g.ram().write_byte(0x16402, 0);
            record(&mut out, 0, g);
            let moved = crate::game::player_collision::operations::displace(
                &mut Runtime::new(g),
                crate::game::player_collision::operations::DisplacementRequest::TerrainOnly {
                    axis: crate::game::player_collision::operations::Axis::X,
                    amount: 65536,
                },
            )
            .displacement;
            assert_eq!(moved, 65536, "room.world.contact-movement");
            assert_eq!(
                word(g, 0x78d),
                fixture::EXIT_A_KEY,
                "room.world.contact-configured-exit"
            );
            assert_eq!(word(g, 0x99c), 0xe17d, "room.world.contact-marker");
            assert_eq!(word(g, 0x998), 9, "room.world.contact-mode");
            record(&mut out, 1, g);
            if id >= 4 {
                // Actual outer dispatcher, NMI and physical IRQ progression. No completion
                // byte, wait result, music return or transition selector is forced here.
                g.ram().write_byte(0x611, 0);
                let frames_before = g.snes_frame_counter;
                let mut done = false;
                for frame in 0..768 {
                    crate::rtl::rtl_run_frame(0, g);
                    assert!(!g.g_fail, "room.world.door-no-diagnostic-{frame}");
                    if word(g, 0x998) == 8 {
                        done = true;
                        break;
                    }
                }
                assert!(done, "room.world.door-completed");
                assert!(
                    g.snes_frame_counter > frames_before + 1,
                    "room.world.door-advanced-frames"
                );
                assert_ne!(word(g, 0x931) & 0x8000, 0, "room.world.door-irq-completed");
                assert!(word(g, 0x925) >= 64, "room.world.door-scroll-frames");
                let expected_room = if id & 1 == 0 {
                    fixture::ROOM_B
                } else {
                    fixture::ROOM_A
                };
                let expected_key = if id & 1 == 0 {
                    fixture::ROOM_B_KEY
                } else {
                    fixture::ROOM_A_KEY
                };
                let expected_tile = if id & 1 == 0 { 0x0303 } else { 0x0101 };
                assert_eq!(
                    word(g, 0x79b),
                    expected_key,
                    "room.world.swapped-door-destination"
                );
                assert_eq!(
                    word(g, 0x10002),
                    expected_tile,
                    "room.world.swapped-door-level"
                );
                let h = Runtime::new(g);
                assert_eq!(
                    h.session().room.entry.room,
                    Identity::Known(expected_room),
                    "room.world.swapped-door-id"
                );
            }
        } else if id <= 3 {
            // Supplied authored start reaches the actual production Runtime/core.
            let result = room_entry::begin(
                &mut Runtime::new(g),
                ArrivalRequest::Start(StartSite::Authored(fixture::START_A)),
            );
            assert!(
                !matches!(result, EntryPoll::Rejected),
                "room.world.authored-start-accepted"
            );
            assert_eq!(
                word(g, 0x79b),
                fixture::ROOM_A_KEY,
                "room.world.authored-start-room"
            );
            assert_eq!(
                word(g, 0x78d),
                fixture::EXIT_B_KEY,
                "room.world.authored-start-exit"
            );
            assert_eq!(
                (word(g, 0xaf6), word(g, 0xafa)),
                (0x1a0, 0x230),
                "room.world.authored-start-placement"
            );
            record(&mut out, 0, g);
            let mut done = false;
            for step in 0..192 {
                let returned = room_entry::protocol::resume(
                    &mut Runtime::new(g),
                    room_entry::EntrySignal::Initial,
                )
                .returned_byte();
                assert!(!g.g_fail, "room.world.initial-no-diagnostic-{step}");
                if returned == 0 {
                    done = true;
                    break;
                }
                nmi_and_draw(g);
            }
            assert!(done, "room.world.initial-completed");
            assert_eq!(
                g.ram().read_byte(0x613),
                0,
                "room.world.initial-continuation-completed"
            );
            assert_eq!(word(g, 0x99c), 0xe737, "room.world.initial-final-stage");
            // Start exitB is intentionally affected by the swapped connection graph.
            assert_eq!(
                word(g, 0x79b),
                if id & 1 == 0 {
                    fixture::ROOM_A_KEY
                } else {
                    fixture::ROOM_B_KEY
                },
                "room.world.swapped-initial-destination"
            );
            assert_eq!(
                word(g, 0x10002),
                if id & 1 == 0 { 0x0101 } else { 0x0303 },
                "room.world.swapped-initial-level"
            );
        } else {
            // Actual terminal IRQ controller updates the shared entry and camera;
            // this is a focused terminal-frame fixture, not a claim of full scrolling.
            {
                let mut h = Runtime::new(g);
                let e = &mut h.session_mut().room.entry;
                e.direction = 0;
                e.frame = 63;
                e.arrival_x = 0x2345;
                e.arrival_y = 0x3456;
                e.speed_whole = 0;
                e.speed_fraction = 0;
                e.completion = 0x0123;
            }
            put(g, 0x7a5, 96);
            put(g, 0x7a7, 96);
            put(g, 0x7a9, 6);
            put(g, 0x7ab, 6);
            record(&mut out, 0, g);
            crate::game::scanline_irq::follow_transition(g);
            assert_eq!(word(g, 0x925), 64, "room.world.actual-irq-frame");
            assert_eq!(word(g, 0x931), 0x8123, "room.world.actual-irq-completion");
            assert_eq!(
                (word(g, 0x911), word(g, 0x915)),
                (0x2345, 0x3456),
                "room.world.actual-irq-camera"
            );
        }
        if (2..=5).contains(&id) {
            assert_eq!(
                (word(g, 0xa000), word(g, 0xa800)),
                (0, 0),
                "room.tables.actual-cre2-opening"
            );
            assert_eq!(g.world().revision(), world.revision());
        }
        assert!(
            Arc::ptr_eq(g.world(), &world),
            "room.world.route-binding-retained"
        );
        assert!(!g.g_fail, "room.world.route-no-diagnostic");
        #[cfg(sm_verify_dll)]
        if id == 2 && g.world().prepared_room_assets().is_some() {
            crate::verification_checkpoint::check_prepared_palette_checkpoint(g);
        }
        record(&mut out, 2, g);
        out
    }
}

// Movies cannot author portable lifecycle IDs or retained staging at the owned publication point.
#[cfg(sm_verify_dll)]
pub fn verify_owned_atmosphere_prefix(
    start: sm_game::scene::start::ValidatedDisplayStart,
    camera: [u16; 2],
    liquid_y: u16,
    context: crate::game::scene::records::ObjectContext,
    created_slot: Option<usize>,
) -> Result<
    (
        sm_game::scene::state::DisplayDescription,
        Result<(), sm_game::scene::state::DisplayError>,
    ),
    sm_game::scene::start::StartError,
> {
    use {
        crate::game::scene::records::ObjectContext,
        sm_game::scene::{
            atmosphere::Inputs,
            component::DisplayComponent,
            state::{
                DisplayControls, DisplayDescription, DisplayError, EffectiveDisplayDescription,
            },
        },
    };
    struct SuppliedInputs {
        camera: [u16; 2],
        liquid_y: u16,
        context: ObjectContext,
    }
    impl Inputs for SuppliedInputs {
        fn camera_x(&mut self, _: &DisplayDescription) -> Result<u16, DisplayError> {
            Ok(self.camera[0])
        }
        fn camera_y(&mut self, _: &DisplayDescription) -> Result<u16, DisplayError> {
            Ok(self.camera[1])
        }
        fn fx_y(&mut self, _: &DisplayDescription) -> Result<u16, DisplayError> {
            Ok(self.liquid_y)
        }
        fn object_context(
            &mut self,
            _: &DisplayControls,
            _: &EffectiveDisplayDescription,
        ) -> Result<ObjectContext, DisplayError> {
            Ok(self.context)
        }
    }
    let mut display = DisplayComponent::open(start)?.display;
    let owner = display.owner_mut();
    let result = owner.begin_interval(false).and_then(|()| {
        if let Some(slot) = created_slot {
            owner.atmospheric_created(slot)?;
        }
        owner.advance_atmosphere(&mut SuppliedInputs {
            camera,
            liquid_y,
            context,
        })
    });
    // Preserve actual successful publications and explicit-error prefixes before dropping the owner.
    Ok((owner.state.clone(), result))
}

// Movies cannot supply producer observations at the owned begin/publication/refresh seam.
#[cfg(sm_verify_dll)]
fn verification_settlement_produce(
    owner: &mut sm_game::scene::commit::OwnedDisplay,
    supplied: ([u16; 2], u16, crate::game::scene::records::ObjectContext),
    created_slot: Option<usize>,
) -> Result<(), sm_game::scene::state::DisplayError> {
    use {
        crate::game::scene::records::ObjectContext,
        sm_game::scene::{
            atmosphere::Inputs,
            state::{
                DisplayControls, DisplayDescription, DisplayError, EffectiveDisplayDescription,
            },
        },
    };
    struct Supplied {
        camera: [u16; 2],
        liquid: u16,
        context: ObjectContext,
    }
    impl Inputs for Supplied {
        fn camera_x(&mut self, _: &DisplayDescription) -> Result<u16, DisplayError> {
            Ok(self.camera[0])
        }
        fn camera_y(&mut self, _: &DisplayDescription) -> Result<u16, DisplayError> {
            Ok(self.camera[1])
        }
        fn fx_y(&mut self, _: &DisplayDescription) -> Result<u16, DisplayError> {
            Ok(self.liquid)
        }
        fn object_context(
            &mut self,
            _: &DisplayControls,
            _: &EffectiveDisplayDescription,
        ) -> Result<ObjectContext, DisplayError> {
            Ok(self.context)
        }
    }
    owner.begin_interval(false)?;
    if let Some(slot) = created_slot {
        owner.atmospheric_created(slot)?;
    }
    owner.advance_atmosphere(&mut Supplied {
        camera: supplied.0,
        liquid: supplied.1,
        context: supplied.2,
    })?;
    owner.refresh()
}

// Movies cannot supply immutable authored content bindings for a constructed numeric raster row.
#[cfg(sm_verify_dll)]
fn verification_settlement_backgrounds(
    state: &sm_game::scene::state::DisplayDescription,
    image: &sm_game::scene::commit::DisplayImage,
    media: &crate::game::scene::records::SceneMedia,
) -> Result<[crate::rendering::Background; 3], sm_game::scene::state::DisplayError> {
    use sm_game::scene::state::DisplayError;
    let compiled = media.image(image.image).map_err(|_| DisplayError::Scene)?;
    let backgrounds = state
        .backgrounds
        .iter()
        .map(|background| {
            let map = media
                .map(background.tilemap)
                .map_err(|_| DisplayError::Scene)?;
            Ok(crate::rendering::Background {
                horizontal_scroll: background.scroll[0],
                vertical_scroll: background.scroll[1],
                map_words: compiled
                    .maps
                    .iter()
                    .find(|binding| binding.map == background.tilemap)
                    .ok_or(DisplayError::Source)?
                    .first_word,
                tile_words: compiled
                    .atlases
                    .iter()
                    .find(|binding| binding.atlas == background.atlas)
                    .ok_or(DisplayError::Source)?
                    .first_word,
                wider: map.width == 64,
                higher: map.height == 64,
            })
        })
        .collect::<Result<Vec<_>, DisplayError>>()?;
    backgrounds.try_into().map_err(|_| DisplayError::Geometry)
}

// Movies cannot force portable semantic rejection after this real shared numeric raster completes.
#[cfg(sm_verify_dll)]
fn verification_settlement_raster(
    state: &sm_game::scene::state::DisplayDescription,
    image: &sm_game::scene::commit::DisplayImage,
    media: &crate::game::scene::records::SceneMedia,
    context: crate::game::scene::records::ObjectContext,
    row: u16,
) -> Result<crate::rendering::prepared::RenderedLine, sm_game::scene::state::DisplayError> {
    use crate::{
        game::scene::state::DisplayError,
        rendering::{self as r, prepared as p},
    };
    let indexed =
        p::IndexedImage::new(image.words.clone(), image.colors).map_err(|_| DisplayError::Scene)?;
    let compiled = media.image(image.image).map_err(|_| DisplayError::Scene)?;
    let sprites = p::PreparedSprites {
        words: core::array::from_fn(|word| {
            u16::from_le_bytes([image.objects[word * 2], image.objects[word * 2 + 1]])
        }),
        extension: image.objects[512..]
            .try_into()
            .map_err(|_| DisplayError::Geometry)?,
        tile_words: context.selection.atlases.map(|atlas| {
            compiled
                .atlases
                .iter()
                .find(|binding| binding.atlas == atlas)
                .unwrap()
                .first_word
        }),
        size: context.selection.size_set,
        rotation: context
            .selection
            .first_order
            .map(|order| order.wrapping_mul(2)),
        interlace: context.selection.interlaced,
        even_frame: context.selection.even_frame,
    };
    let mosaic = p::PreparedMosaic {
        native_enabled: state.controls.mosaic,
        background0_enabled: false,
        size: 1,
        start_line: state.raster.mosaic_origin,
        modulo: [0; 256],
    };
    let input = p::PreparedInput {
        media: &indexed,
        sprites: &sprites,
        mode: state.controls.mode,
        forced_blank: state.effective.forced_blank,
        backgrounds: verification_settlement_backgrounds(state, image, media)?,
        windows: r::Windows {
            first: [0; 2],
            second: [0; 2],
            native_flags: 0,
            layers: [r::Window {
                first_enabled: false,
                first_inverted: false,
                second_enabled: false,
                second_inverted: false,
                logic: 0,
            }; 6],
        },
        routing: r::Routing {
            enabled: [state.effective.main_layers, state.controls.sub_layers],
            windowed: [0; 2],
        },
        affine: r::Affine {
            matrix: [0; 8],
            horizontal_flip: false,
            vertical_flip: false,
            large_field: false,
            character_fill: false,
        },
        mosaic: &mosaic,
        color: r::ColorMath {
            clip: state.effective.color_clip,
            prevent: state.effective.color_prevent_math,
            enabled: state.effective.color_math_enabled,
            add_subscreen: state.effective.color_add_subscreen,
            subtract: state.effective.color_subtract,
            half: state.effective.color_half,
            fixed: state.controls.fixed_color,
            brightness: state.controls.brightness & 15,
        },
    };
    input
        .render(
            row,
            r::Overflow {
                range: state.raster.range_overflow,
                time: state.raster.time_overflow,
            },
        )
        .map_err(|_| DisplayError::Scene)
}

// Movies cannot supply portable semantic row histories or a deliberate post-raster context rejection.
#[cfg(sm_verify_dll)]
struct VerificationSettlement {
    template: crate::game::scene::records::SceneLine,
    lines: Vec<crate::game::scene::records::SceneLine>,
    rendered: Vec<crate::rendering::prepared::RenderedLine>,
    fail_row: Option<u16>,
}

// This quiesced fixture has no movie-reached room transition; unexpected callbacks must fail.
#[cfg(sm_verify_dll)]
impl sm_game::scene::commit::RoomEffects for VerificationSettlement {
    fn follow_transition(
        &mut self,
        _: &mut sm_game::scene::state::DisplayDescription,
        _: &mut sm_game::scene::commit::DisplayImage,
        _: &mut std::sync::Arc<crate::game::scene::records::SceneMedia>,
    ) -> Result<(), sm_game::scene::state::DisplayError> {
        Err(sm_game::scene::state::DisplayError::Source)
    }
}

// This authored fixture disables every channel; unexpected numeric destinations must fail at access.
#[cfg(sm_verify_dll)]
impl sm_game::scene::owned_hdma::Destinations for VerificationSettlement {
    fn read_destination(
        &mut self,
        _: &mut sm_game::scene::state::DisplayDescription,
        _: &mut sm_game::scene::commit::DisplayImage,
        _: u8,
    ) -> Result<u8, sm_game::scene::state::DisplayError> {
        Err(sm_game::scene::state::DisplayError::Source)
    }
    fn write_destination(
        &mut self,
        _: &mut sm_game::scene::state::DisplayDescription,
        _: &mut sm_game::scene::commit::DisplayImage,
        _: u8,
        _: u8,
    ) -> Result<(), sm_game::scene::state::DisplayError> {
        Err(sm_game::scene::state::DisplayError::Source)
    }
}

// Movies cannot force post-raster semantic failure while retaining completed numeric row effects.
#[cfg(sm_verify_dll)]
impl sm_game::scene::commit::SettlementEffects for VerificationSettlement {
    fn raster_row(
        &mut self,
        state: &mut sm_game::scene::state::DisplayDescription,
        image: &mut sm_game::scene::commit::DisplayImage,
        media: &mut std::sync::Arc<crate::game::scene::records::SceneMedia>,
        row: u16,
    ) -> Result<crate::game::scene::records::ObjectContext, sm_game::scene::state::DisplayError>
    {
        use {
            crate::game::scene::records::{ObjectContext, ObjectSelection},
            sm_game::scene::state::DisplayError,
        };
        let mut context = ObjectContext {
            selection: ObjectSelection {
                even_frame: state.raster.even_frame,
                ..self.template.object_selection
            },
            palette: image.palette,
        };
        let rendered = verification_settlement_raster(state, image, media, context, row)?;
        state.raster.range_overflow = rendered.sprites.overflow.range;
        state.raster.time_overflow = rendered.sprites.overflow.time;
        state.raster.has_objects = rendered.sprites.has_tiles;
        self.rendered.push(rendered);
        if self.fail_row == Some(row) {
            context.selection.size_set = 7;
            return Ok(context);
        }
        let mut line = self.template.clone();
        line.media = image.image;
        line.palette = context.palette;
        line.object_selection = context.selection;
        let mut objects = line.objects.to_vec();
        for record in &image.produced {
            let selected = record
                .for_line(media, context, &image.objects)
                .map_err(|_| DisplayError::Scene)?;
            let order = usize::from(selected.object.order);
            objects[order] = selected.object;
        }
        line.objects = std::sync::Arc::from(objects);
        self.lines.push(line);
        Ok(context)
    }
}

// Movies cannot author a complete owned producer-to-settlement commitment with supplied new incarnations.
#[cfg(sm_verify_dll)]
fn verification_settlement_run(
    owner: &mut sm_game::scene::commit::OwnedDisplay,
    supplied: ([u16; 2], u16, crate::game::scene::records::ObjectContext),
    incarnations: [crate::game::scene::records::MediaRef; 2],
    created_slot: Option<usize>,
    effects: &mut VerificationSettlement,
) -> Result<(), sm_game::scene::state::DisplayError> {
    use {crate::game::scene::records::DisplayStatus, sm_game::scene::state::DisplayError};
    let (_, resident, scene, media, _) = owner.checkpoint_parts()?;
    let bindings = media
        .image(resident.image)
        .map_err(|_| DisplayError::Scene)?;
    verification_settlement_produce(owner, supplied, created_slot)?;
    let media = owner.append_incarnation(
        incarnations[0],
        incarnations[1],
        bindings.atlases.clone(),
        bindings.maps.clone(),
    )?;
    effects.template.media = incarnations[0];
    effects.template.palette = incarnations[1];
    for plane in &mut effects.template.planes {
        plane.palette = incarnations[1];
    }
    for screen in &mut effects.template.routing {
        screen.visible = [false; 5];
        screen.masked = [false; 5];
    }
    let objects = std::sync::Arc::make_mut(&mut effects.template.objects);
    for object in objects {
        object.position = crate::game::scene::records::ScreenPosition { x: 0, y: 240 };
    }
    owner.begin_settlement()?;
    owner.settle(effects)?;
    let mut description = scene.description().clone();
    description.tick = description.tick.checked_add(1).ok_or(DisplayError::Phase)?;
    if !effects.lines.is_empty() {
        description.lines = std::sync::Arc::from(effects.lines.clone());
    }
    description.geometry.even_frame = owner.state.raster.even_frame;
    description.geometry.interlaced = owner.state.raster.frame_interlaced;
    description.status = DisplayStatus {
        range_overflow: owner.state.raster.range_overflow,
        time_overflow: owner.state.raster.time_overflow,
        has_objects: owner.state.raster.has_objects,
        even_frame: owner.state.raster.even_frame,
        interlaced: owner.state.raster.frame_interlaced,
        overscan: owner.state.raster.frame_overscan,
    };
    owner.commit_scene(description, media)?;
    Ok(())
}

// Movies cannot supply portable checkpoint/reset ownership or inspect explicit failure prefixes safely.
#[cfg(sm_verify_dll)]
pub fn verify_owned_atmosphere_settlement(
    mut display: sm_game::scene::component::DisplayComponent,
    supplied: ([u16; 2], u16, crate::game::scene::records::ObjectContext),
    incarnations: [crate::game::scene::records::MediaRef; 2],
    created_slot: Option<usize>,
    fail_row: Option<u16>,
) -> (
    sm_game::scene::component::DisplayComponent,
    sm_game::scene::state::DisplayDescription,
    crate::game::scene::ResidentDisplayDescription,
    Vec<crate::rendering::prepared::RenderedLine>,
    Result<(), sm_game::scene::state::DisplayError>,
) {
    use {crate::game::scene::ResidentDisplayDescription, sm_game::scene::state::DisplayError};
    let mut effects = VerificationSettlement {
        template: display.committed_scene().lines()[223].clone(),
        lines: Vec::new(),
        rendered: Vec::new(),
        fail_row,
    };
    let owner = display.owner_mut();
    let result =
        verification_settlement_run(owner, supplied, incarnations, created_slot, &mut effects);
    if result.is_err() && owner.state.phase == sm_game::scene::state::DisplayPhase::Failed {
        assert_eq!(owner.refresh(), Err(DisplayError::Poisoned));
        let mut next = owner.committed().description().clone();
        next.tick += 1;
        let media = owner.committed().media().clone();
        assert_eq!(
            owner.commit_scene(next, media).map(|_| ()),
            Err(DisplayError::Poisoned)
        );
    }
    let state = owner.state.clone();
    let image = owner.image();
    let resident = ResidentDisplayDescription {
        image: image.image,
        palette: image.palette,
        words: std::sync::Arc::from(image.words.clone()),
        colors: image.colors,
        objects: image.objects,
        produced: std::sync::Arc::from(image.produced.clone()),
    };
    (display, state, resident, effects.rendered, result)
}

// Movies cannot supply a portable catalogue/context at the actual common-owner publication seam.
#[cfg(sm_verify_dll)]
struct VerificationProducerInputs {
    catalog: std::sync::Arc<sm_game::scene::samus::SamusCatalog>,
    context: crate::game::scene::records::ObjectContext,
    camera: [u16; 2],
    liquid: u16,
}

// Movies cannot supply selected portable Samus media; unexpected invalid-map hooks must fail lazily.
#[cfg(sm_verify_dll)]
impl sm_game::scene::samus::Inputs for VerificationProducerInputs {
    fn content(
        &mut self,
        _: &sm_game::scene::state::DisplayDescription,
    ) -> Result<
        std::sync::Arc<sm_game::scene::samus::SamusCatalog>,
        sm_game::scene::state::DisplayError,
    > {
        Ok(self.catalog.clone())
    }
    fn invalid_map(
        &mut self,
        _: &mut sm_game::scene::state::DisplayDescription,
        _: u16,
    ) -> Result<(), sm_game::scene::state::DisplayError> {
        Err(sm_game::scene::state::DisplayError::Source)
    }
    fn object_context(
        &mut self,
        _: &sm_game::scene::state::DisplayControls,
        _: &sm_game::scene::state::EffectiveDisplayDescription,
    ) -> Result<crate::game::scene::records::ObjectContext, sm_game::scene::state::DisplayError>
    {
        Ok(self.context)
    }
}

// Movies cannot provide portable atmosphere observations during a constructed interleaved lifecycle.
#[cfg(sm_verify_dll)]
impl sm_game::scene::atmosphere::Inputs for VerificationProducerInputs {
    fn camera_x(
        &mut self,
        _: &sm_game::scene::state::DisplayDescription,
    ) -> Result<u16, sm_game::scene::state::DisplayError> {
        Ok(self.camera[0])
    }
    fn camera_y(
        &mut self,
        _: &sm_game::scene::state::DisplayDescription,
    ) -> Result<u16, sm_game::scene::state::DisplayError> {
        Ok(self.camera[1])
    }
    fn fx_y(
        &mut self,
        _: &sm_game::scene::state::DisplayDescription,
    ) -> Result<u16, sm_game::scene::state::DisplayError> {
        Ok(self.liquid)
    }
    fn object_context(
        &mut self,
        _: &sm_game::scene::state::DisplayControls,
        _: &sm_game::scene::state::EffectiveDisplayDescription,
    ) -> Result<crate::game::scene::records::ObjectContext, sm_game::scene::state::DisplayError>
    {
        Ok(self.context)
    }
}

// Movies cannot supply a common Samus birth while preserving the actual owning failure guard.
#[cfg(sm_verify_dll)]
fn verification_producer_allocate(
    owner: &mut sm_game::scene::commit::OwnedDisplay,
) -> Result<crate::game::scene::records::ProducerObjectId, sm_game::scene::state::DisplayError> {
    let mut guard = sm_game::scene::delivery::FaultGuard::new(&mut owner.state);
    let result = guard
        .state()
        .producer_ids
        .allocate()
        .map_err(|_| sm_game::scene::state::DisplayError::Identity);
    if result.is_ok() {
        guard.disarm();
    }
    result
}

// Movies cannot supply interleaved namespace births, Samus IDs and atmospheric retirement in one owner.
#[cfg(sm_verify_dll)]
pub fn verify_owned_namespace_interleaved(
    mut display: sm_game::scene::component::DisplayComponent,
    catalog: std::sync::Arc<sm_game::scene::samus::SamusCatalog>,
    supplied: ([u16; 2], u16, crate::game::scene::records::ObjectContext),
) -> (
    Vec<sm_game::scene::state::DisplayDescription>,
    Result<(), sm_game::scene::state::DisplayError>,
) {
    let media = display.committed_scene().media().clone();
    let owner = display.owner_mut();
    let birth = owner.state.atmospheric.slots[0].clone();
    let mut inputs = VerificationProducerInputs {
        catalog,
        camera: supplied.0,
        liquid: supplied.1,
        context: supplied.2,
    };
    let mut states = Vec::new();
    let result = (|| {
        owner.begin_interval(false)?;
        owner.atmospheric_created(0)?;
        states.push(owner.state.clone());
        let samus = verification_producer_allocate(owner)?;
        sm_game::scene::samus::draw_owned(
            &mut owner.state,
            &media,
            samus,
            0,
            (200, 60),
            &mut inputs,
        )?;
        states.push(owner.state.clone());
        owner.advance_atmosphere(&mut inputs)?;
        states.push(owner.state.clone());
        owner.advance_atmosphere(&mut inputs)?;
        states.push(owner.state.clone());
        owner.state.atmospheric.slots[0] = birth;
        owner.atmospheric_created(0)?;
        states.push(owner.state.clone());
        sm_game::scene::samus::draw_owned(
            &mut owner.state,
            &media,
            samus,
            0,
            (200, 60),
            &mut inputs,
        )?;
        owner.advance_atmosphere(&mut inputs)?;
        states.push(owner.state.clone());
        Ok(())
    })();
    (states, result)
}

// Movies cannot inject zero/exhausted counters or invalid slots inside an already admitted owned interval.
#[cfg(sm_verify_dll)]
pub fn verify_owned_namespace_birth_prefix(
    mut display: sm_game::scene::component::DisplayComponent,
    supplied_next: crate::game::scene::records::ProducerObjectId,
    slot: usize,
) -> (
    sm_game::scene::state::DisplayDescription,
    Result<(), sm_game::scene::state::DisplayError>,
) {
    let owner = display.owner_mut();
    let result = owner.begin_interval(false).and_then(|()| {
        owner.state.producer_ids.next = supplied_next;
        owner.atmospheric_created(slot)
    });
    if owner.state.phase == sm_game::scene::state::DisplayPhase::Failed {
        assert_eq!(
            owner.refresh(),
            Err(sm_game::scene::state::DisplayError::Poisoned)
        );
    }
    (owner.state.clone(), result)
}

// Movies cannot supply invalid Samus identity after the actual native-equivalent publication prefix.
#[cfg(sm_verify_dll)]
pub fn verify_owned_samus_id_prefix(
    mut display: sm_game::scene::component::DisplayComponent,
    catalog: std::sync::Arc<sm_game::scene::samus::SamusCatalog>,
    context: crate::game::scene::records::ObjectContext,
    producer: crate::game::scene::records::ProducerObjectId,
) -> (
    sm_game::scene::state::DisplayDescription,
    Result<(), sm_game::scene::state::DisplayError>,
) {
    let media = display.committed_scene().media().clone();
    let owner = display.owner_mut();
    let mut inputs = VerificationProducerInputs {
        catalog,
        context,
        camera: [0, 0],
        liquid: 0,
    };
    let result = owner.begin_interval(false).and_then(|()| {
        sm_game::scene::samus::draw_owned(
            &mut owner.state,
            &media,
            producer,
            0,
            (100, 76),
            &mut inputs,
        )
    });
    if owner.state.phase == sm_game::scene::state::DisplayPhase::Failed {
        assert_eq!(
            owner.refresh(),
            Err(sm_game::scene::state::DisplayError::Poisoned)
        );
    }
    (owner.state.clone(), result)
}

// Movies cannot author this prepared marker; these aliases serve only its constructed verification inputs.
#[cfg(sm_verify_dll)]
use sm_game::scene::{
    samus_body as vm_body, samus_position_tiles as vm_tiles,
    state::{
        DisplayControls as MarkerControls, DisplayDescription as MarkerState,
        DisplayError as MarkerError, EffectiveDisplayDescription as MarkerEffective,
    },
};

// Movies cannot supply body observations at a retained portable marker word/cursor prefix.
#[cfg(sm_verify_dll)]
struct VerificationMarkerInputs {
    base: VerificationProducerInputs,
    media: std::sync::Arc<crate::game::scene::records::SceneMedia>,
    player: [u16; 2],
    body: [u16; 2],
}

// Movies cannot supply marker-owned Samus content; the original lazy queries use the supplied catalogue.
#[cfg(sm_verify_dll)]
impl sm_game::scene::samus::Inputs for VerificationMarkerInputs {
    fn content(
        &mut self,
        state: &MarkerState,
    ) -> Result<std::sync::Arc<sm_game::scene::samus::SamusCatalog>, MarkerError> {
        sm_game::scene::samus::Inputs::content(&mut self.base, state)
    }
    fn invalid_map(&mut self, state: &mut MarkerState, cursor: u16) -> Result<(), MarkerError> {
        sm_game::scene::samus::Inputs::invalid_map(&mut self.base, state, cursor)
    }
    fn object_context(
        &mut self,
        controls: &MarkerControls,
        effective: &MarkerEffective,
    ) -> Result<crate::game::scene::records::ObjectContext, MarkerError> {
        sm_game::scene::samus::Inputs::object_context(&mut self.base, controls, effective)
    }
}

// Movies cannot supply these constructed marker observations without a common fallible result.
#[cfg(sm_verify_dll)]
type MarkerResult<T> = Result<T, MarkerError>;

// Movies cannot supply a frozen marker whose first selected tile table fails after publication.
#[cfg(sm_verify_dll)]
impl vm_body::Inputs for VerificationMarkerInputs {
    type BodyTable = vm_body::BodyTable;
    fn knockback(&mut self) -> MarkerResult<u16> {
        Err(MarkerError::Source)
    }
    fn invincibility(&mut self) -> MarkerResult<u16> {
        Err(MarkerError::Source)
    }
    fn shine(&mut self) -> MarkerResult<u16> {
        Err(MarkerError::Source)
    }
    fn frame_counter(&mut self) -> MarkerResult<u16> {
        Err(MarkerError::Source)
    }
    fn table(&mut self, _: vm_body::Half) -> MarkerResult<vm_body::BodyTable> {
        Ok(vm_body::BodyTable {
            bytes: std::sync::Arc::from([0, 0]),
            native_order: sm_game::scene::state::WordOrder::Little,
        })
    }
    fn set_sprite(&mut self, _: vm_body::Half, _: u16) -> MarkerResult<()> {
        Err(MarkerError::Source)
    }
}

// Movies cannot supply the marker's explicit full atlas byte source as a constructed position input.
#[cfg(sm_verify_dll)]
fn verification_marker_position_source(
    media: &crate::game::scene::records::SceneMedia,
    reference: crate::game::scene::records::AssetRef,
) -> MarkerResult<vm_tiles::SourceTable> {
    let crate::game::scene::records::SceneAsset::Atlas(atlas) =
        media.asset(reference).map_err(|_| MarkerError::Identity)?
    else {
        return Err(MarkerError::Source);
    };
    Ok(vm_tiles::SourceTable {
        source: sm_game::scene::state::SourceSpan {
            asset: reference,
            bytes: atlas
                .data
                .iter()
                .flat_map(|word| word.to_le_bytes())
                .collect::<Vec<_>>()
                .into(),
            offset: 0,
        },
        native_order: sm_game::scene::state::WordOrder::Little,
    })
}

// Movies cannot supply authored marker position inputs or fail the first selected tile table lazily.
#[cfg(sm_verify_dll)]
impl vm_tiles::Inputs for VerificationMarkerInputs {
    type Table = vm_tiles::SourceTable;
    fn ceres(&mut self) -> MarkerResult<u16> {
        Ok(0)
    }
    fn player_x(&mut self) -> MarkerResult<u16> {
        Ok(self.player[0])
    }
    fn player_y(&mut self) -> MarkerResult<u16> {
        Ok(self.player[1])
    }
    fn set_player_x(&mut self, _: &mut MarkerState, _: u16) -> MarkerResult<()> {
        Err(MarkerError::Source)
    }
    fn set_player_y(&mut self, _: &mut MarkerState, _: u16) -> MarkerResult<()> {
        Err(MarkerError::Source)
    }
    fn project(&mut self, _: &mut MarkerState, _: u16, _: u16) -> MarkerResult<(u16, u16)> {
        Err(MarkerError::Source)
    }
    fn animation_frame(&mut self) -> MarkerResult<u16> {
        Ok(0)
    }
    fn movement(&mut self) -> MarkerResult<u8> {
        Ok(0)
    }
    fn landing_order(&self) -> sm_game::scene::state::WordOrder {
        sm_game::scene::state::WordOrder::Little
    }
    fn position_source(&mut self) -> MarkerResult<vm_tiles::SourceTable> {
        verification_marker_position_source(&self.media, self.base.context.selection.atlases[0])
    }
    fn camera_x(&mut self) -> MarkerResult<u16> {
        Ok(self.base.camera[0])
    }
    fn camera_y(&mut self) -> MarkerResult<u16> {
        Ok(self.base.camera[1])
    }
    fn set_body_x(&mut self, _: &mut MarkerState, value: u16) -> MarkerResult<()> {
        self.body[0] = value;
        Ok(())
    }
    fn set_body_y(&mut self, _: &mut MarkerState, value: u16) -> MarkerResult<()> {
        self.body[1] = value;
        Ok(())
    }
    fn body_x(&mut self) -> MarkerResult<u16> {
        Ok(self.body[0])
    }
    fn body_y(&mut self) -> MarkerResult<u16> {
        Ok(self.body[1])
    }
    fn pose(&mut self) -> MarkerResult<u16> {
        Ok(0)
    }
    fn tile_table(&mut self, _: vm_tiles::TileTable) -> MarkerResult<vm_tiles::SourceTable> {
        Err(MarkerError::Source)
    }
    fn tile_definition(
        &mut self,
        _: &mut MarkerState,
        _: u16,
    ) -> MarkerResult<vm_tiles::SourceTable> {
        Err(MarkerError::Source)
    }
    fn request_word(&mut self, _: &mut MarkerState, _: vm_body::Half, _: u16) -> MarkerResult<()> {
        Err(MarkerError::Source)
    }
    fn request_flag(&mut self, _: &mut MarkerState, _: vm_body::Half) -> MarkerResult<()> {
        Err(MarkerError::Source)
    }
    fn tile_request(
        &mut self,
        _: &MarkerState,
        _: vm_body::Half,
        _: u16,
    ) -> MarkerResult<vm_tiles::TileRequest> {
        Err(MarkerError::Source)
    }
}

// Movies cannot inject owned marker cursor aliases or capture their original native low/high write prefixes.
#[cfg(sm_verify_dll)]
pub fn verify_owned_marker_prefix(
    mut display: sm_game::scene::component::DisplayComponent,
    catalog: std::sync::Arc<sm_game::scene::samus::SamusCatalog>,
    context: crate::game::scene::records::ObjectContext,
    marker: vm_body::MarkerPublication,
    cursor: u16,
) -> (MarkerState, Result<(), MarkerError>) {
    let media = display.committed_scene().media().clone();
    let owner = display.owner_mut();
    let mut inputs = VerificationMarkerInputs {
        base: VerificationProducerInputs {
            catalog,
            context,
            camera: [0, 0],
            liquid: 0,
        },
        media: media.clone(),
        player: [104, 200],
        body: [0, 0],
    };
    let result = owner.begin_interval(false).and_then(|()| {
        owner.state.objects.cursor = cursor;
        vm_body::draw_owned(
            &mut owner.state,
            &media,
            marker.producer,
            &marker,
            vm_body::Body::Frozen,
            &mut inputs,
        )
    });
    if owner.state.phase == sm_game::scene::state::DisplayPhase::Failed {
        assert_eq!(owner.refresh(), Err(MarkerError::Poisoned));
    }
    (owner.state.clone(), result)
}

// Movies cannot supply these injected owner-effect failures or explicit position observations.
#[cfg(sm_verify_dll)]
type VerificationPositionEvents = Vec<(&'static str, usize, usize)>;

// Movies cannot independently supply full frames, native order or temporary Ceres position owners.
#[cfg(sm_verify_dll)]
struct VerificationPositionHost {
    frame: u16,
    movement: u8,
    order: sm_game::scene::state::WordOrder,
    fallback: u8,
    ceres: u16,
    player: [u16; 2],
    camera: [u16; 2],
    projected: [u16; 2],
    body: [u16; 2],
    failure: usize,
    events: VerificationPositionEvents,
}

// Movies cannot inject a failure after an explicitly supplied owner's completed publication.
#[cfg(sm_verify_dll)]
impl VerificationPositionHost {
    fn event(&mut self, name: &'static str, first: usize, second: usize) -> Result<(), usize> {
        self.events.push((name, first, second));
        if self.events.len() == self.failure {
            Err(self.failure)
        } else {
            Ok(())
        }
    }
    fn read(&mut self, name: &'static str, value: u16) -> Result<u16, usize> {
        self.event(name, usize::from(value), 0)?;
        Ok(value)
    }
}

// Movies cannot supply injected query/write failures to the actual shared PositionHost core.
#[cfg(sm_verify_dll)]
impl vm_tiles::PositionHost for VerificationPositionHost {
    type Error = usize;
    fn ceres(&mut self) -> Result<u16, usize> {
        self.read("ceres", self.ceres)
    }
    fn player_x(&mut self) -> Result<u16, usize> {
        self.read("player_x", self.player[0])
    }
    fn player_y(&mut self) -> Result<u16, usize> {
        self.read("player_y", self.player[1])
    }
    fn set_player_x(&mut self, value: u16) -> Result<(), usize> {
        self.player[0] = value;
        self.event("set_player_x", usize::from(value), 0)
    }
    fn set_player_y(&mut self, value: u16) -> Result<(), usize> {
        self.player[1] = value;
        self.event("set_player_y", usize::from(value), 0)
    }
    fn project(&mut self, x: u16, y: u16) -> Result<(u16, u16), usize> {
        self.event("project", usize::from(x), usize::from(y))?;
        Ok((self.projected[0], self.projected[1]))
    }
    fn frame(&mut self) -> Result<u16, usize> {
        self.read("frame", self.frame)
    }
    fn movement(&mut self) -> Result<u8, usize> {
        self.event("movement", usize::from(self.movement), 0)?;
        Ok(self.movement)
    }
    fn landing_word(&self, bytes: [u8; 2]) -> u16 {
        self.order.decode(bytes)
    }
    fn fallback_byte(&mut self, offset: usize) -> Result<u8, usize> {
        self.event("fallback", offset, usize::from(self.fallback))?;
        Ok(self.fallback)
    }
    fn camera_x(&mut self) -> Result<u16, usize> {
        self.read("camera_x", self.camera[0])
    }
    fn camera_y(&mut self) -> Result<u16, usize> {
        self.read("camera_y", self.camera[1])
    }
    fn set_body_x(&mut self, value: u16) -> Result<(), usize> {
        self.body[0] = value;
        self.event("set_body_x", usize::from(value), 0)
    }
    fn set_body_y(&mut self, value: u16) -> Result<(), usize> {
        self.body[1] = value;
        self.event("set_body_y", usize::from(value), 0)
    }
}

// Movies cannot inject table-bound panics while retaining the actual full-frame query prefix.
#[cfg(sm_verify_dll)]
pub fn verify_samus_position_offset(
    doubled: u16,
    frame: u16,
    movement: u8,
    order: sm_game::scene::state::WordOrder,
    fallback: u8,
) -> (Result<Result<i32, usize>, ()>, VerificationPositionEvents) {
    let mut host = VerificationPositionHost {
        frame,
        movement,
        order,
        fallback,
        ceres: 0,
        player: [0, 0],
        camera: [0, 0],
        projected: [0, 0],
        body: [0, 0],
        failure: 0,
        events: Vec::new(),
    };
    let result = catch_unwind(AssertUnwindSafe(|| {
        vm_tiles::sprite_y_offset(&mut host, doubled)
    }))
    .map_err(|_| ());
    (result, host.events)
}

// Movies cannot inject Ceres query/publication failures into independent temporary position owners.
#[cfg(sm_verify_dll)]
pub fn verify_samus_ceres_position(
    failure: usize,
) -> (
    Result<(u16, u16), usize>,
    [u16; 2],
    [u16; 2],
    VerificationPositionEvents,
) {
    let mut host = VerificationPositionHost {
        frame: 2,
        movement: 0,
        order: sm_game::scene::state::WordOrder::Little,
        fallback: 0x80,
        ceres: 0x8000,
        player: [0x1234, 0x2345],
        camera: [0x30, 0x20],
        projected: [0xfff0, 0x10],
        body: [0x4567, 0x5678],
        failure,
        events: Vec::new(),
    };
    let result = vm_tiles::sprite_position(&mut host, 0);
    (result, host.player, host.body, host.events)
}

// Movies cannot author these independent Samus/resident observations inside the actual owned consumer.
#[cfg(sm_verify_dll)]
use {
    crate::game::scene::records as witness_records,
    sm_game::scene::{commit as witness_commit, state as witness_state},
};

// Movies cannot inspect an owned failed upload's completed resident prefix through a committed checkpoint.
#[cfg(sm_verify_dll)]
fn verification_resident_snapshot(
    owner: &witness_commit::OwnedDisplay,
) -> witness_commit::ResidentDisplayDescription {
    let image = owner.image();
    witness_commit::ResidentDisplayDescription {
        image: image.image,
        palette: image.palette,
        words: std::sync::Arc::from(image.words.clone()),
        colors: image.colors,
        objects: image.objects,
        produced: std::sync::Arc::from(image.produced.clone()),
    }
}

// Movies cannot inject transfer remap/source faults after a sealed component has adopted valid state.
#[cfg(sm_verify_dll)]
pub fn verify_owned_samus_upload(
    mut display: sm_game::scene::component::DisplayComponent,
    halves: [witness_state::SamusHalf; 2],
    latches: witness_state::DisplayLatches,
) -> (
    witness_state::DisplayDescription,
    witness_commit::ResidentDisplayDescription,
    Result<(), witness_state::DisplayError>,
) {
    let owner = display.owner_mut();
    let result = owner.begin_interval(false).and_then(|()| {
        owner.state.samus = halves;
        owner.state.latches = latches;
        witness_commit::RefreshHost::samus(owner)
    });
    if owner.state.phase == witness_state::DisplayPhase::Failed {
        assert_eq!(owner.refresh(), Err(witness_state::DisplayError::Poisoned));
    }
    (
        owner.state.clone(),
        verification_resident_snapshot(owner),
        result,
    )
}

// Movies cannot supply short independent resident extents or observe the exact pre-alias rejection.
#[cfg(sm_verify_dll)]
pub fn verify_resident_append_prefix(
    mut resident: witness_commit::ResidentDisplayDescription,
    media: std::sync::Arc<witness_records::SceneMedia>,
    incarnations: [witness_records::MediaRef; 2],
    aliases: [u16; 5],
) -> (
    witness_commit::ResidentDisplayDescription,
    usize,
    Result<std::sync::Arc<witness_records::SceneMedia>, witness_state::DisplayError>,
) {
    let words = &mut resident.words.to_vec();
    let calls = std::cell::Cell::new(0usize);
    let result = sm_game::scene::resident_media::append(
        sm_game::scene::resident_media::Resident {
            image: resident.image,
            palette: resident.palette,
            words,
            colors: &resident.colors,
        },
        &media,
        sm_game::scene::resident_media::Incarnation {
            image: incarnations[0],
            palette: incarnations[1],
            atlases: std::sync::Arc::from([]),
            maps: std::sync::Arc::from([]),
        },
        || {
            calls.set(calls.get() + 1);
            aliases
        },
    );
    resident.words = std::sync::Arc::from(words.clone());
    (resident, calls.get(), result)
}

// Movies cannot author new immutable identities or inspect actual append adoption after an alias prefix.
#[cfg(sm_verify_dll)]
pub fn verify_owned_resident_append(
    mut display: sm_game::scene::component::DisplayComponent,
    incarnations: [witness_records::MediaRef; 2],
    colors: [u16; 256],
    latches: witness_state::DisplayLatches,
    bindings: (
        std::sync::Arc<[witness_records::AtlasBinding]>,
        std::sync::Arc<[witness_records::MapBinding]>,
    ),
) -> (
    sm_game::scene::component::DisplayComponent,
    witness_commit::ResidentDisplayDescription,
    Result<std::sync::Arc<witness_records::SceneMedia>, witness_state::DisplayError>,
    Option<std::sync::Arc<witness_records::SceneMedia>>,
) {
    let owner = display.owner_mut();
    owner.state.visual.colors = colors;
    owner.state.latches = latches;
    let result = witness_commit::RefreshHost::palette_and_objects(owner).and_then(|()| {
        owner.append_incarnation(incarnations[0], incarnations[1], bindings.0, bindings.1)
    });
    let resident = verification_resident_snapshot(owner);
    let adopted = if result.is_ok() {
        owner
            .checkpoint_parts()
            .ok()
            .map(|(_, _, _, media, _)| media)
    } else {
        None
    };
    (display, resident, result, adopted)
}
