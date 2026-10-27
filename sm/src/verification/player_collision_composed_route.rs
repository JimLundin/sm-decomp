//! Collision composition through initialized authored rooms and actual door completion.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::{
    game::{
        actors::{composed_checks as checks, composed_fixture as fixture},
        room_entry::{self, ArrivalRequest, EntryPoll, StartSite},
        session_runtime::Runtime,
    },
    game_state::Game,
    player_collision_composed_steps as steps,
};
fn put(g: &mut Game, at: usize, value: u16) {
    g.ram().copy_in(at, &value.to_le_bytes());
}
fn word(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes({
        let mut bytes = [0; 2];
        g.ram().copy_out(at, &mut bytes);
        bytes
    })
}
fn record(out: &mut Vec<u8>, phase: u8, g: &Game) {
    out.push(phase);
    out.extend((0..g.ram().len()).map(|at| g.ram().read_byte(at)));
    out.push(u8::from(g.g_fail));
}
struct Observe;
impl Drop for Observe {
    fn drop(&mut self) {
        checks::stop();
    }
}
pub(crate) unsafe fn run(id: u32, g: &mut Game) -> Vec<u8> {
    unsafe {
        assert!((14..16).contains(&id));
        assert!(!g.g_snes.is_null(), "composed.real-hardware-carrier");
        crate::rtl::RtlReset(2, g);
        crate::native_audio::upload_bank(0xcf8000, g);
        fixture::install(g);
        steps::install_surface_content(g);
        crate::game::session_runtime::initialize_player(g);
        for (at, value) in [(0x9c2, 99), (0x9c4, 99), (0x998, 8), (0x5f5, 0)] {
            put(g, at, value);
        }
        checks::begin();
        let _observe = Observe;
        crate::verify_probe::configure(&[
            "actors.population.installed.nonempty".into(),
            "actors.select.active".into(),
            "actors.advance.ai.completed".into(),
            "actors.contact.normal.completed".into(),
            "actors.animation.frame.committed".into(),
            "actors.scene.base.emitted".into(),
        ])
        .unwrap();
        let mut out = b"PCMP1".to_vec();
        out.extend_from_slice(&id.to_le_bytes());
        let poll = room_entry::begin(
            &mut Runtime::new(g),
            ArrivalRequest::Start(StartSite::Authored(crate::room_world_fixture::START_A)),
        );
        assert!(
            !matches!(poll, EntryPoll::Rejected),
            "composed.start.accepted"
        );
        let mut done = false;
        for step in 0..192 {
            crate::verify_probe::begin_frame(step);
            let returned = room_entry::protocol::resume(
                &mut Runtime::new(g),
                room_entry::EntrySignal::Initial,
            )
            .returned_byte();
            assert!(!g.g_fail, "composed.start.no-diagnostic-{step}");
            if returned == 0 {
                done = true;
                break;
            }
            crate::game::display_refresh::run(g);
            crate::cpu_infra::DrawFrameToPpu(g);
        }
        assert!(done, "composed.start.completed");
        assert_eq!(g.ram().read_byte(0x613), 0);
        assert_eq!(word(g, 0x99c), 0xe737);
        assert_eq!(word(g, 0x79b), crate::room_world_fixture::ROOM_A_KEY);
        // StartGameplay prepares assets; the retained GameState6 caller then drains
        // this exact transfer function with NMI between calls before gameplay begins.
        let mut transferred = false;
        for step in 0..16 {
            transferred = checks::source_transfer(&mut Runtime::new(g));
            crate::game::display_refresh::run(g);
            crate::cpu_infra::DrawFrameToPpu(g);
            assert!(!g.g_fail, "composed.source.transfer-no-diagnostic-{step}");
            if transferred {
                break;
            }
        }
        assert!(
            transferred,
            "composed.source.actual-transfer-and-install-completed"
        );
        checks::source(&mut Runtime::new(g), id == 15);
        crate::verify_probe::report().expect("composed.source.actual-producers-and-consumers");
        record(&mut out, 1, g);
        steps::run(g, id == 15, &mut out);
        // Explicit final fixture position/input; source actors remain installed.
        // Actual outer movement performs speed, actor, terrain and door admission.
        // Seed contact geometry before movement; no continuation/completion/IRQ outputs forced.
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
            (0xa1e, 8),
            (0xb4a, 0),
            (0xa46, 0),
            (0x79b, crate::room_world_fixture::ROOM_A_KEY),
            (0x7b5, 0xc000),
            (0x79f, 0),
            (0x99c, 0x7777),
        ] {
            put(g, at, value);
        }
        put(g, 0x10002, 0x9000);
        g.ram().write_byte(0x16402, 0);
        crate::game::player_collision::verification::move_x(65536, g);
        assert_eq!(
            word(g, 0xaf6),
            9,
            "collision.composed.outer-door-displacement"
        );
        assert_eq!(
            word(g, 0x78d),
            crate::room_world_fixture::EXIT_A_KEY,
            "composed.door.configured-exit"
        );
        assert_eq!(word(g, 0x99c), 0xe17d);
        assert_eq!(word(g, 0x998), 9);
        record(&mut out, 2, g);
        g.ram().write_byte(0x611, 0);
        let frames_before = g.snes_frame_counter;
        let mut done = false;
        for frame in 0..768 {
            crate::verify_probe::begin_frame(192 + frame);
            crate::rtl::RtlRunFrame(0, g);
            assert!(!g.g_fail, "composed.door.no-diagnostic-{frame}");
            if word(g, 0x998) == 8 {
                done = true;
                break;
            }
        }
        assert!(done, "composed.door.completed");
        assert!(g.snes_frame_counter > frames_before + 1);
        assert_ne!(
            word(g, 0x931) & 0x8000,
            0,
            "composed.door.actual-irq-completion"
        );
        assert!(word(g, 0x925) >= 64);
        assert_eq!(word(g, 0x79b), crate::room_world_fixture::ROOM_B_KEY);
        assert_eq!(
            word(g, 0x10002),
            0x0303,
            "composed.door.actual-destination-decompression"
        );
        checks::destination(&mut Runtime::new(g));
        assert!(!g.g_fail, "composed.final.no-diagnostic");
        record(&mut out, 3, g);
        out
    }
}
