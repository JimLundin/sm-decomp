//! Prepared-world cases through the actual GameOwner and retained hardware.
//! Caller initializes this owner with SnesInit and verifier audio in a fresh child.
#![deny(unsafe_op_in_unsafe_fn)]
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
pub(crate) const CASES: u32 = 14;
#[cfg(not(sm_verify_dll))]
pub(crate) const CASES: u32 = 11;
fn put(g: &mut Game, at: usize, value: u16) {
    g.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn word(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes(g.g_ram[at..at + 2].try_into().unwrap())
}
fn record(out: &mut Vec<u8>, phase: u8, g: &Game) {
    out.push(phase);
    out.extend_from_slice(&g.g_ram);
    out.push(u8::from(g.g_fail));
}
/// Uses the real constructor route; exported DLL initializes hardware via owner.with.
pub(crate) fn owner(id: u32) -> Box<GameOwner> {
    #[cfg(sm_verify_dll)]
    if (12..14).contains(&id) {
        return GameOwner::with_world(crate::game::actors::composed_fixture::binding(id == 13));
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
        crate::game::sm_80::Vector_NMI(g);
        crate::cpu_infra::DrawFrameToPpu(g);
    }
}
unsafe fn run(id: u32, g: &mut Game) -> Vec<u8> {
    unsafe {
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
            assert_eq!(g.g_ram[0x611], 2, "room.world.actual-reset-frame-entered");
            assert!(Arc::ptr_eq(g.world(), &world), "room.world.frame-binding");
            assert!(!g.g_fail, "room.world.frame-no-diagnostic");
            record(&mut out, 0, g);
            return out;
        }
        // These focused routes begin after reset rather than running Vector_RESET.
        // Install its actual base audio program before any retained music command.
        crate::game::sm_80::APU_UploadBank(0xcf8000, g);
        assert!(!g.g_fail, "room.world.base-audio-upload-no-diagnostic");
        room_world_resources::install(g);
        assert!(!g.g_fail, "room.world.resources-no-diagnostic");
        // Actual player initialization supplies retained draw/movement prerequisites.
        crate::game::sm_91::Samus_Initialize(g);
        assert!(!g.g_fail, "room.world.player-initialize-no-diagnostic");
        put(g, 0x9c2, 99);
        put(g, 0x9c4, 99);
        put(g, 0x998, 8);
        put(g, 0x5f5, 0);
        if id >= 9 {
            let (site, room, exit, x, y) = if id == 9 {
                (
                    fixture::START_A_SITE,
                    fixture::ROOM_A_KEY,
                    fixture::EXIT_B_KEY,
                    0x1a0,
                    0x230,
                )
            } else {
                (
                    fixture::START_B_SITE,
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
            crate::game::sm_80::LoadFromLoadStation(g);
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
            assert_eq!(g.g_ram[0x178e], 0xa0, "room.world.source-draw-hook-bank");
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
            g.g_ram[0x16402] = 0;
            record(&mut out, 0, g);
            let moved = crate::game::samus_room_contact_ram::move_right(65536, g);
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
                g.g_ram[0x611] = 0;
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
                let returned = crate::game::sm_80::StartGameplay_Async(g);
                assert!(!g.g_fail, "room.world.initial-no-diagnostic-{step}");
                if returned == 0 {
                    done = true;
                    break;
                }
                nmi_and_draw(g);
            }
            assert!(done, "room.world.initial-completed");
            assert_eq!(
                g.g_ram[0x613], 0,
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
            crate::game::sm_80::Irq_FollowDoorTransition(g);
            assert_eq!(word(g, 0x925), 64, "room.world.actual-irq-frame");
            assert_eq!(word(g, 0x931), 0x8123, "room.world.actual-irq-completion");
            assert_eq!(
                (word(g, 0x911), word(g, 0x915)),
                (0x2345, 0x3456),
                "room.world.actual-irq-camera"
            );
        }
        assert!(
            Arc::ptr_eq(g.world(), &world),
            "room.world.route-binding-retained"
        );
        assert!(!g.g_fail, "room.world.route-no-diagnostic");
        record(&mut out, 2, g);
        out
    }
}
