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
        crate::cpu_infra::DrawFrameToPpu(g);
    }
}
unsafe fn run(id: u32, g: &mut Game) -> Vec<u8> {
    unsafe {
        #[cfg(sm_verify_dll)]
        if id == 105 {
            // Movies cannot supply divergent cached/per-layer routing and staged/effective PPU state.
            crate::rtl::RtlReset(2, g);
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
            crate::rtl::RtlReset(2, g);
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
            crate::rtl::RtlReset(2, g);
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
            crate::rtl::RtlReset(2, g);
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
                Host::write_tile(&mut Runtime::new(g), 0x1fc8d, 0xa55a).unwrap();
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
            crate::rtl::RtlReset(2, g);
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
            crate::rtl::RtlReset(2, g);
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
            crate::rtl::RtlReset(2, g);
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
        crate::rtl::RtlReset(2, g);
        assert!(!g.g_fail, "room.world.reset-no-diagnostic");
        assert!(Arc::ptr_eq(g.world(), &world), "room.world.reset-binding");
        if id == 8 {
            let before = g.snes_frame_counter;
            crate::rtl::RtlRunFrame(0, g);
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
                        crate::game::room_entry::world::LegacyStartSite { area: 6, index: 0 }
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
                        crate::game::room_entry::world::LegacyStartSite { area: 0, index: 18 }
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
                    crate::rtl::RtlRunFrame(0, g);
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
