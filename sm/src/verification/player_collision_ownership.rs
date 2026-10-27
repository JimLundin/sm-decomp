//! Predecessor ownership-sensitive observations; child of cpu_infra for real snapshots.
//! Uses actual owned transition entries; retains the original expected RAM observations.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::{
    game::{
        player_projectiles::access::Store, player_transitions as pt, session_access::SessionAccess,
        session_runtime::Runtime, sm_a9,
    },
    game_state::Game,
    rom::{RomImage, lorom_offset},
};
use std::panic::{AssertUnwindSafe, catch_unwind};

pub(crate) const COUNT: u32 = 12;
// This is a future migration address set, NOT a claim that the predecessor owns it.
const LANES: [usize; 29] = [
    0x5e9, 0x5eb, 0xa20, 0xa22, 0xa26, 0xa34, 0xa36, 0xa38, 0xa3a, 0xa3c, 0xa3e, 0xa40, 0xa46,
    0xa94, 0xa96, 0xafe, 0xb10, 0xb12, 0xb14, 0xb16, 0xb4a, 0xdba, 0xdbc, 0xdbe, 0xdc6, 0xdce,
    0xdd0, 0xdec, 0xdee,
];
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn word(g: &Game, at: usize) -> u16 {
    {
        let _ = &[(); 0x20000][at..at + 2];
        let mut bytes = [0; 2];
        g.ram().copy_out(at, &mut bytes);
        u16::from_le_bytes(bytes)
    }
}
fn assert_ram(g: &Game, expected: &[u8], label: &str) {
    assert_eq!(
        (0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<_>>()
            .as_slice(),
        expected,
        "{label}"
    );
    assert!(!g.g_fail, "{label}.no_diagnostic");
}
fn record(out: &mut Vec<u8>, phase: u8, result: u32, g: &Game) {
    out.push(phase);
    out.extend_from_slice(&result.to_le_bytes());
    out.extend((0..g.ram().len()).map(|at| g.ram().read_byte(at)));
    out.push(u8::from(g.g_fail));
}
fn seed_lanes(g: &mut Game) {
    for (index, &at) in LANES.iter().enumerate() {
        g.ram()
            .copy_in(at, &(0x4100 + 13 * index as u16).to_le_bytes());
    }
}
fn synthetic_radius_image(g: &mut Game, pose: u16, radius: u8) -> RomImage {
    let image = RomImage::zeroed(0x200000);
    image.write(
        lorom_offset(0x91b629) as usize + usize::from(pose) * 8 + 6,
        radius,
    );
    g.register_native_rom(image.clone());
    g.register_rom(image.clone());
    image
}
fn history(g: &mut Game, out: &mut Vec<u8>) {
    g.ram().copy_in(
        0xa1c,
        &[
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc,
        ],
    );
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    record(out, 0, 0, g);
    expected[0xa20..0xa28].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]);
    pt::record_pose_history(Runtime::new(g).session_mut());
    assert_ram(g, &expected, "collision.owner.history.first_rotation");
    record(out, 1, 0, g);
    expected[0xa24..0xa28].copy_from_slice(&[0x11, 0x22, 0x33, 0x44]);
    {
        let mut rt = Runtime::new(g);
        rt.foreign(|g| pt::record_pose_history(Runtime::new(g).session_mut()));
        assert_eq!(
            rt.word(0xa24),
            0x2211,
            "collision.owner.history.foreign_reread"
        );
    }
    assert_ram(g, &expected, "collision.owner.history.second_rotation");
    record(out, 2, 0, g);
}
fn radius(g: &mut Game, out: &mut Vec<u8>) {
    let pose = 7;
    let image = synthetic_radius_image(g, pose, 19);
    g.ram().copy_in(0xa1c, &(pose as u16).to_le_bytes());
    g.ram().copy_in(0xafe, &(0x4567 as u16).to_le_bytes());
    g.ram().copy_in(0xb00, &(0x89ab as u16).to_le_bytes());
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    record(out, 0, 0, g);
    crate::game::session_runtime::set_player_radius(g);
    put(&mut expected, 0xafe, 5);
    put(&mut expected, 0xb00, 19);
    assert_ram(g, &expected, "collision.owner.radius.first_live_content");
    record(out, 1, 0, g);
    image.write(
        lorom_offset(0x91b629) as usize + usize::from(pose) * 8 + 6,
        31,
    );
    {
        let mut rt = Runtime::new(g);
        rt.foreign(crate::game::session_runtime::set_player_radius);
        assert_eq!(rt.word(0xb00), 31, "collision.owner.radius.foreign_reload");
        assert_eq!(rt.word(0xafe), 5, "collision.owner.radius.x_reread");
    }
    put(&mut expected, 0xb00, 31);
    assert_ram(g, &expected, "collision.owner.radius.mutated_content");
    record(out, 2, 0, g);
}
fn mixed_native(g: &mut Game, out: &mut Vec<u8>) {
    g.ram().copy_in(0xa20, &[1, 2, 3, 4, 5, 6, 7, 8]);
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    record(out, 0, 0, g);
    expected[0xa23..0xa25].copy_from_slice(&0x3412u16.to_ne_bytes());
    {
        let mut rt = Runtime::new(g);
        // Odd native word spans predecessor Game byte A23 and owned A24.
        rt.store(0xa23, 0x3412, Store::SliceNative);
        for at in 0xa20..0xa28 {
            assert_eq!(
                rt.byte(at),
                expected[at],
                "collision.owner.mixed_native.live"
            );
        }
    }
    assert_ram(g, &expected, "collision.owner.mixed_native.drop");
    record(out, 1, 0, g);
}
unsafe fn animation(g: &mut Game, out: &mut Vec<u8>) {
    g.ram().copy_in(0x196e, &(8 as u16).to_le_bytes()); // FX kind4: deliberately no FX callback.
    g.ram().copy_in(0xa1c, &(0x4d as u16).to_le_bytes());
    g.ram().copy_in(0xa94, &(1 as u16).to_le_bytes());
    g.ram().copy_in(0xa96, &(1 as u16).to_le_bytes());
    g.ram().copy_in(0xb36, &(1 as u16).to_le_bytes());
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    record(out, 0, 0, g);
    pt::advance_animation(&mut Runtime::new(g), pt::state::AnimationStep::Gameplay);
    put(&mut expected, 0xa94, 3);
    assert_ram(g, &expected, "collision.owner.animation.held_jump_timer");
    record(out, 1, 0, g);
    {
        let mut rt = Runtime::new(g);
        rt.foreign(|g| {
            pt::advance_animation(&mut Runtime::new(g), pt::state::AnimationStep::Gameplay)
        });
        assert_eq!(
            rt.word(0xa94),
            2,
            "collision.owner.animation.foreign_reread"
        );
        assert_eq!(
            rt.word(0xa96),
            1,
            "collision.owner.animation.frame_preserved"
        );
    }
    put(&mut expected, 0xa94, 2);
    assert_ram(g, &expected, "collision.owner.animation.normal_countdown");
    record(out, 2, 0, g);
}
unsafe fn vertical_carry(g: &mut Game, out: &mut Vec<u8>) {
    g.ram().copy_in(0xafa, &(60 as u16).to_le_bytes());
    g.ram().copy_in(0xafc, &(0xf0a5 as u16).to_le_bytes());
    g.ram().copy_in(0xb14, &(0x6677 as u16).to_le_bytes());
    g.ram().copy_in(0xb16, &(0x995a as u16).to_le_bytes());
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    record(out, 0, 0, g);
    let result = unsafe { sm_a9::MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(0x0120, g) };
    assert_eq!(result, 0, "collision.owner.vertical.unclamped");
    put(&mut expected, 0xafa, 62);
    put(&mut expected, 0xb14, 62);
    put(&mut expected, 0xafc, 0x10a5);
    put(&mut expected, 0xb16, 0x105a);
    assert_ram(
        g,
        &expected,
        "collision.owner.vertical.partial_fraction_bytes",
    );
    record(out, 1, u32::from(result), g);
    g.ram().copy_in(0xafa, &(190 as u16).to_le_bytes());
    g.ram().copy_in(0xafc, &(0xf0a5 as u16).to_le_bytes());
    expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    let result = unsafe { sm_a9::MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(0x0220, g) };
    assert_eq!(result, 1, "collision.owner.vertical.clamped");
    for at in [0xafa, 0xb14] {
        put(&mut expected, at, 192);
    }
    for at in [0xafc, 0xb16] {
        put(&mut expected, at, 0);
    }
    assert_ram(
        g,
        &expected,
        "collision.owner.vertical.clamp_clears_fractions",
    );
    record(out, 2, u32::from(result), g);
}
unsafe fn horizontal_carry(g: &mut Game, out: &mut Vec<u8>) {
    g.ram().copy_in(0xaf6, &(100 as u16).to_le_bytes());
    g.ram().copy_in(0xaf8, &(0xf0a5 as u16).to_le_bytes());
    g.ram().copy_in(0xb10, &(0x6677 as u16).to_le_bytes());
    g.ram().copy_in(0xb12, &(0x995a as u16).to_le_bytes());
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    record(out, 0, 0, g);
    let result = unsafe { sm_a9::MotherBrain_MoveSamusHorizTowardsWall(0x0120, g) };
    assert_eq!(result, 0, "collision.owner.horizontal.unclamped");
    for at in [0xaf6, 0xb10] {
        put(&mut expected, at, 102);
    }
    put(&mut expected, 0xaf8, 0x10a5);
    put(&mut expected, 0xb12, 0x105a);
    assert_ram(
        g,
        &expected,
        "collision.owner.horizontal.partial_fraction_bytes",
    );
    record(out, 1, u32::from(result), g);
    g.ram().copy_in(0xaf6, &(234 as u16).to_le_bytes());
    g.ram().copy_in(0xaf8, &(0xf0a5 as u16).to_le_bytes());
    expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    let result = unsafe { sm_a9::MotherBrain_MoveSamusHorizTowardsWall(0x0120, g) };
    assert_eq!(result, 1, "collision.owner.horizontal.clamped");
    for at in [0xaf6, 0xb10] {
        put(&mut expected, at, 235);
    }
    for at in [0xaf8, 0xb12] {
        put(&mut expected, at, 0);
    }
    assert_ram(
        g,
        &expected,
        "collision.owner.horizontal.clamp_clears_fractions",
    );
    record(out, 2, u32::from(result), g);
}
unsafe fn sequence(g: &mut Game, out: &mut Vec<u8>) {
    g.ram().copy_in(0xdec, &(3 as u16).to_le_bytes());
    g.ram().copy_in(0xdee, &(0x8070 as u16).to_le_bytes());
    g.ram().write_byte(0xddd, 0x20);
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    record(out, 0, 0, g);
    let result = {
        let mut rt = Runtime::new(g);
        crate::game::player_transitions::suit::expand(&mut rt, Runtime::suit_window_word);
        let result = 1u8;
        assert_eq!(rt.word(0xdec), 4, "collision.owner.sequence.stage_reread");
        assert_eq!(
            rt.word(0xdee),
            0x846c,
            "collision.owner.sequence.value_reread"
        );
        result
    };
    assert_eq!(result, 1, "collision.owner.sequence.return");
    put(&mut expected, 0xdec, 4);
    put(&mut expected, 0xdee, 0x846c);
    for at in (0x9800..0x9a00).step_by(2) {
        put(&mut expected, at, 0xa050);
    }
    assert_ram(
        g,
        &expected,
        "collision.owner.sequence.hdma_observes_before_stage_override",
    );
    record(out, 1, u32::from(result), g);
}
unsafe fn multiply(g: &mut Game, out: &mut Vec<u8>) {
    seed_lanes(g);
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    record(out, 0, 0, g);
    let result = g.multiply_with_scratch(0xf123, 0x0123);
    assert_eq!(result, 0x01121ac9, "collision.owner.multiply.product");
    put(&mut expected, 0x5e9, 0xf123);
    put(&mut expected, 0x5eb, 0x0123);
    assert_ram(g, &expected, "collision.owner.multiply.shared_operands");
    record(out, 1, result, g);
}
unsafe fn reset(g: &mut Game, out: &mut Vec<u8>) {
    seed_lanes(g);
    g.ram().write_byte(0xa01, 0x92);
    g.ram().write_byte(0xe0c, 0x37);
    g.ram().copy_in(0x1e77, &(0x2a5c as u16).to_le_bytes());
    g.ram().copy_in(0x998, &(8 as u16).to_le_bytes());
    g.ram().copy_in(0x9c2, &(99 as u16).to_le_bytes());
    let multiply = [word(g, 0x5e9), word(g, 0x5eb)];
    record(out, 0, 0, g);
    crate::game::session_runtime::initialize_player(g);
    for &at in &LANES {
        let expected = match at {
            0x5e9 => multiply[0],
            0x5eb => multiply[1],
            0xa46 => 3,
            _ => 0,
        };
        assert_eq!(
            word(g, at),
            expected,
            "collision.owner.reset.samus_lane at={at:04x}"
        );
    }
    assert_eq!(
        (g.ram().read_byte(0xa01), g.ram().read_byte(0xe0c)),
        (0x92, 0x37),
        "collision.owner.reset.outside_workspace"
    );
    assert_eq!(
        word(g, 0x1e77),
        0x2a5c,
        "collision.owner.reset.samus_retains_room_slope"
    );
    assert!(!g.g_fail, "collision.owner.reset.samus_no_diagnostic");
    record(out, 1, 0, g);
    let snes = g.g_snes;
    let world = g.world().clone();
    unsafe { crate::rtl::RtlReset(2, g) };
    assert_eq!(g.g_snes, snes, "collision.owner.reset.backend_identity");
    assert!(
        std::sync::Arc::ptr_eq(g.world(), &world),
        "collision.owner.reset.world_identity"
    );
    for &at in &LANES {
        assert_eq!(
            word(g, at),
            0,
            "collision.owner.reset.full_lane at={at:04x}"
        );
    }
    assert_eq!(
        g.ram().read_byte(0x611),
        1,
        "collision.owner.reset.boot_resume"
    );
    assert!(!g.g_fail, "collision.owner.reset.full_no_diagnostic");
    record(out, 2, 0, g);
}
unsafe fn cpu(g: &mut Game, out: &mut Vec<u8>) {
    let snes = g.g_snes;
    seed_lanes(g);
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    record(out, 0, 0, g);
    let before_ops = unsafe { (*snes).cpuMemOps };
    for (index, &at) in LANES.iter().enumerate() {
        let value = 0x6100 + 7 * index as u16;
        let [low, high] = value.to_le_bytes();
        unsafe {
            let mut execution = g.execution();
            execution.cpu_write(0x800000 | at as u32, low);
            execution.cpu_write((at + 1) as u32, high);
        }
        put(&mut expected, at, value);
    }
    unsafe {
        assert_eq!(
            (*snes).cpuMemOps,
            before_ops.wrapping_add(58),
            "collision.owner.cpu.actual_bus_entry_count"
        )
    };
    assert_ram(g, &expected, "collision.owner.cpu.mirrored_writes");
    {
        let rt = Runtime::new(g);
        for &at in &LANES {
            assert_eq!(
                rt.word(at),
                u16::from_le_bytes(expected[at..at + 2].try_into().unwrap()),
                "collision.owner.cpu.imported_reread"
            );
        }
    }
    record(out, 1, 58, g);
    unsafe {
        {
            let mut execution = g.execution();
            execution.write_bbus(0x81, 0xec);
            execution.write_bbus(0x82, 0x0d);
            execution.write_bbus(0x83, 0);
            execution.write_bbus(0x80, 0xa1);
            execution.write_bbus(0x80, 0xb2);
        }
        assert_eq!(
            (*snes).ramAdr,
            0xdee,
            "collision.owner.cpu.wram_port_increment"
        );
    }
    put(&mut expected, 0xdec, 0xb2a1);
    assert_ram(g, &expected, "collision.owner.cpu.wram_port_bytes");
    record(out, 2, 0xdee, g);
}
unsafe fn snapshot(g: &mut Game, out: &mut Vec<u8>) {
    seed_lanes(g);
    let before = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    let world = g.world().clone();
    record(out, 0, 0, g);
    let snes = g.g_snes;
    unsafe {
        {
            let snapshot_snes = snes;
            let snapshot_cpu = g.g_cpu;
            let snapshot_player = g.g_spc_player;
            let (backend, ram, _, _, before) = g.ram_and_snapshots();
            super::MakeSnapshot(
                before,
                &*snapshot_snes,
                &*snapshot_cpu,
                snapshot_player,
                ram,
                backend,
            )
        }
    };
    g.actor_snapshots.before = Some(g.capture_actor_snapshot());
    let queue = crate::audio_snapshot::ApuQueueSnapshot::capture(g);
    for &at in &LANES {
        g.ram().copy_in(at, &(0xfeed as u16).to_le_bytes());
    }
    record(out, 1, 0, g);
    queue.restore(g);
    unsafe { g.g_snapshot_before.player.restore(g.g_spc_player) };
    let restore = g
        .actor_snapshots
        .before
        .as_ref()
        .unwrap()
        .preflight(g.actor_identity(), g.world())
        .expect("collision.owner.snapshot.preflight");
    unsafe {
        {
            let snapshot_snes = snes;
            let snapshot_cpu = g.g_cpu;
            let (backend, ram, _, _, before) = g.ram_and_snapshots();
            super::RestoreSnapshot(
                before,
                &mut *snapshot_snes,
                &mut *snapshot_cpu,
                ram,
                backend,
            )
        }
    };
    restore.commit(g.actor_identity_mut());
    assert_ram(g, &before, "collision.owner.snapshot.whole_ram_restored");
    assert_eq!(
        g.capture_actor_snapshot(),
        g.actor_snapshots.before.as_ref().unwrap().clone(),
        "collision.owner.snapshot.actor_payload_restored"
    );
    assert!(
        std::sync::Arc::ptr_eq(g.world(), &world),
        "collision.owner.snapshot.world_binding"
    );
    {
        let rt = Runtime::new(g);
        for &at in &LANES {
            assert_eq!(
                rt.word(at),
                u16::from_le_bytes(before[at..at + 2].try_into().unwrap()),
                "collision.owner.snapshot.fresh_import"
            );
        }
    }
    assert_ram(
        g,
        &before,
        "collision.owner.snapshot.import_drop_preserves_restore",
    );
    record(out, 2, 0, g);
}
fn foreign_unwind(g: &mut Game, out: &mut Vec<u8>) {
    seed_lanes(g);
    let mut expected = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    record(out, 0, 0, g);
    let error = catch_unwind(AssertUnwindSafe(|| {
        let mut rt = Runtime::new(g);
        rt.foreign(|g| {
            g.multiply_with_scratch(0x1234, 0x5678);
            g.ram().write_byte(0xdc7, 0xe1);
            panic!("collision ownership controlled foreign unwind");
        });
    }))
    .expect_err("collision.owner.foreign.expected_unwind");
    assert_eq!(
        error.downcast_ref::<&str>().copied(),
        Some("collision ownership controlled foreign unwind"),
        "collision.owner.foreign.exact_panic"
    );
    put(&mut expected, 0x5e9, 0x1234);
    put(&mut expected, 0x5eb, 0x5678);
    expected[0xdc7] = 0xe1;
    assert!(
        !g.actor_identity_is_with_session(),
        "collision.owner.foreign.owner_returned"
    );
    assert_ram(
        g,
        &expected,
        "collision.owner.foreign.partial_writes_survive",
    );
    {
        let rt = Runtime::new(g);
        assert_eq!(
            rt.word(0x5e9),
            0x1234,
            "collision.owner.foreign.fresh_import"
        );
        assert_eq!(
            rt.byte(0xdc7),
            0xe1,
            "collision.owner.foreign.high_byte_reread"
        );
    }
    assert_ram(g, &expected, "collision.owner.foreign.no_stale_drop");
    record(out, 1, 0, g);
}
/// Root's room-world export supplies an initialized real-ROM backend in a fresh process.
pub(crate) unsafe fn run(id: u32, g: &mut Game) -> Vec<u8> {
    assert!(id < COUNT, "collision.owner.case_id");
    assert!(
        !g.g_snes.is_null() && !g.g_cpu.is_null() && !g.g_spc_player.is_null(),
        "collision.owner.initialized_backend"
    );
    // Fixture setup, not a claimed reset. Backend allocations and cartridge remain real.
    g.ram().fill(0..g.ram().len(), 0);
    g.g_fail = false;
    let mut out = b"PCOW1".to_vec();
    out.extend_from_slice(&id.to_le_bytes());
    match id {
        0 => history(g, &mut out),
        1 => radius(g, &mut out),
        2 => mixed_native(g, &mut out),
        3 => unsafe { animation(g, &mut out) },
        4 => unsafe { vertical_carry(g, &mut out) },
        5 => unsafe { horizontal_carry(g, &mut out) },
        6 => unsafe { sequence(g, &mut out) },
        7 => unsafe { multiply(g, &mut out) },
        8 => unsafe { reset(g, &mut out) },
        9 => unsafe { cpu(g, &mut out) },
        10 => unsafe { snapshot(g, &mut out) },
        11 => foreign_unwind(g, &mut out),
        _ => unreachable!(),
    }
    out
}
