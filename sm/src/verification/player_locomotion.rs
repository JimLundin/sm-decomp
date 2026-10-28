//! Actual normal-locomotion dispatch; initialized existing transition carrier.
//! Authored content only; full RAM and existing effect observations are compared.
use super::{Game, MakeSnapshot, RestoreSnapshot, Snapshot};
use crate::ram_map::{word_arrays as a, words as w};
use crate::{
    initial_state::InitialState, player_transitions_rust_observer as observer, rom::RomImage,
};
pub const FIRST: u32 = 4000;
pub const LAST: u32 = 4082;
fn put(g: &mut Game, at: usize, v: u16) {
    g.ram().copy_in(at, &v.to_le_bytes());
}
fn get(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes([g.ram().read_byte(at), g.ram().read_byte(at + 1)])
}
fn expect(g: &Game, at: usize, v: u16, why: &str) {
    assert_eq!(get(g, at), v, "{why} at {at:04x}");
}
fn record(out: &mut Vec<u8>, tag: u8, data: &[u8]) {
    out.push(tag);
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(data);
}
fn author(r: &RomImage, address: u32, data: &[u8], manifest: &mut Vec<u8>) {
    let at = crate::rtl::lorom_offset(address) as usize;
    assert!(at + data.len() <= r.len());
    manifest.extend_from_slice(&address.to_le_bytes());
    manifest.extend_from_slice(&(data.len() as u32).to_le_bytes());
    manifest.extend_from_slice(data);
    for (i, b) in data.iter().copied().enumerate() {
        r.write(at + i, b);
    }
}
fn seed(id: u32, g: &mut Game) -> (RomImage, RomImage, Vec<u8>, Vec<u8>) {
    g.ram().fill(0..g.ram().len(), 0);
    g.ram().write_byte(0x1ffff, 1);
    g.g_fail = false;
    let loaded = RomImage::zeroed(0x200000);
    let native = RomImage::zeroed(0x200000);
    let mut lm = Vec::new();
    let mut nm = Vec::new();
    let mut poses = vec![0; 256 * 8];
    for pose in poses.chunks_exact_mut(8) {
        pose[0] = 8;
        pose[6] = 3;
    }
    for (image, manifest, max) in [(&loaded, &mut lm, 6u16), (&native, &mut nm, 8u16)] {
        author(image, 0x91b629, &poses, manifest);
        for start in [0x90e000, 0x909f49, 0x90a08d, 0x90a1dd] {
            let mut table = Vec::new();
            for _ in 0..28 {
                for v in [0u16, 0, max, 0, 2, 0] {
                    table.extend_from_slice(&v.to_le_bytes());
                }
            }
            author(image, start, &table, manifest);
        }
        let mut palette = Vec::new();
        for color in 0..16u16 {
            palette.extend_from_slice(&(0x4100 + color).to_le_bytes());
        }
        author(image, 0x9b9800, &palette, manifest);
    }
    // Raw bank90 spin-splash table is Native; Loaded intentionally disagrees.
    author(&native, 0x90a514, &[1; 10], &mut nm);
    author(&loaded, 0x90a514, &[0; 10], &mut lm);
    g.register_rom(loaded.clone());
    g.register_native_rom(native.clone());
    let seed_id = match id {
        4049 => 4030,
        4050 => 4031,
        4051..=4082 => 4001,
        _ => id,
    };
    let local = seed_id - FIRST;
    let selector = if local < 28 {
        local as u16
    } else {
        match local {
            29 => 0,
            39 | 40 => 3,
            41 => 2,
            42 | 43 => 8,
            44 => 6,
            45 => 15,
            46 => 20,
            47 => 22,
            _ => 1,
        }
    };
    let pose = match selector {
        4 => 0x1d,
        15 => 0x35,
        17 => 0x79,
        _ => 1,
    };
    for (at, v) in [
        (w::room_width_in_blocks, 32),
        (w::room_height_in_blocks, 32),
        (w::samus_x_pos, 136),
        (w::samus_y_pos, 136),
        (w::samus_x_radius, 3),
        (w::samus_y_radius, 3),
        (w::samus_x_subpos, 0x1111),
        (w::samus_y_subpos, 0x2222),
        (w::samus_prev_x_pos, 136),
        (w::samus_prev_y_pos, 136),
        (w::equipped_items, 0x20),
        (w::speed_echoes_index, 0xffff),
        (w::samus_x_speed_table_pointer, 0xe000),
        (w::button_config_jump_a, 0x80),
        (w::button_config_run_b, 0x40),
        (w::joypad1_lastkeys, 0x180),
        (w::samus_pose, pose),
        (0xa1e, (selector << 8) | 8),
        (w::samus_anim_frame_timer, 2),
        (w::samus_y_dir, 2),
        (w::samus_y_speed, 1),
        (w::samus_y_subspeed, 0x2000),
        (w::samus_y_subaccel, 0x1c00),
        (w::samus_x_base_speed, 1),
        (w::samus_x_base_subspeed, 0x1234),
        (w::input_to_pose_calc, 0x7777),
        (w::fx_y_pos, 0xffff),
        (w::lava_acid_y_pos, 0xffff),
        (w::samus_suit_palette_index, 4),
    ] {
        put(g, at, v);
    }
    match local {
        28 => {
            put(g, w::time_is_frozen_flag, 1);
            put(g, w::speed_boost_counter, 0x400);
            put(g, w::speed_echoes_index, 2);
        }
        29 => {
            put(g, w::samus_pose, 0);
            put(g, w::elevator_status, 1);
        }
        30 | 31 => {
            put(g, w::samus_anim_frame_timer, 1);
            put(g, w::samus_anim_frame, if local == 30 { 2 } else { 7 });
            put(g, w::area_index, if local == 30 { 4 } else { 1 });
            if local == 31 {
                put(g, w::speed_boost_counter, 0x400);
                put(g, w::speed_echoes_index, 2);
                put(g, w::game_time_frames, 4);
            }
        }
        32..=35 => {
            put(g, w::equipped_items, if local == 35 { 0x20 } else { 0 });
            put(g, w::fx_y_pos, if local == 33 { 0xffff } else { 100 });
            put(g, w::lava_acid_y_pos, 100);
            if local == 34 {
                put(g, w::fx_liquid_options, 4);
            }
        }
        36 | 37 => {
            put(g, w::samus_x_base_speed, 8);
            put(
                g,
                w::samus_x_base_subspeed,
                if local == 36 { 0x8000 } else { 1 },
            );
        }
        38 => {
            put(g, w::samus_x_accel_mode, 1);
            put(g, w::samus_x_base_speed, 1);
            put(g, w::samus_x_base_subspeed, 0);
        }
        39 => {
            put(g, w::equipped_items, 0x220);
            put(g, w::samus_y_speed, 3);
            put(g, w::samus_y_subspeed, 0);
            put(g, w::joypad1_newkeys, 0x80);
        }
        40 => {
            put(g, w::samus_suit_palette_index, 0);
            put(g, w::equipped_items, 0);
            put(g, w::fx_y_pos, 100);
            put(g, w::samus_anim_frame_timer, 1);
        }
        41 => {
            put(g, w::samus_y_dir, 1);
            put(g, w::samus_y_speed, 2);
            put(g, w::joypad1_lastkeys, 0x100);
        }
        42 | 43 => {
            put(g, w::used_for_ball_bounce_on_landing, 1);
            put(g, w::extra_samus_y_displacement, 2);
            if local == 43 {
                put(g, w::knockback_dir, 1);
            }
        }
        44 => {
            put(g, w::samus_y_dir, 1);
            put(g, w::samus_y_speed, 0xffff);
        }
        45 => {
            put(g, w::samus_pose, 0x39);
            put(g, w::input_to_pose_calc, 1025);
            put(g, w::used_for_ball_bounce_on_landing, 2);
            put(g, w::samus_y_pos, 140);
            put(g, w::samus_y_speed, 2);
            put(g, 0xaa4, 0x5678);
            put(g, 0x10002 + 2 * 296, 0x8000);
            put(g, 0x10002 + 2 * 297, 0x8000);
        }
        46 => put(g, w::samus_anim_frame, 23),
        47 => put(g, w::input_to_pose_calc, 5),
        48 => {
            put(g, w::samus_x_accel_mode, 1);
            put(g, w::samus_x_base_speed, 3);
            put(g, w::samus_x_base_subspeed, 0);
            put(g, 0xb4c, 0x80);
        }
        _ => {}
    }
    if id >= 4049 {
        atmosphere_seed(id, g, &loaded, &native, &mut lm, &mut nm);
    }
    (loaded, native, lm, nm)
}
fn check(id: u32, g: &Game, before: &[u8]) {
    let local = id - FIRST;
    if matches!(local, 9 | 13 | 28) {
        assert_eq!(
            (0..g.ram().len())
                .map(|at| g.ram().read_byte(at))
                .collect::<Vec<_>>()
                .as_slice(),
            before,
            "unused/frozen normal dispatch is byte-exact no-op"
        );
    }
    if matches!(local, 7 | 11 | 12 | 26 | 27) {
        let mut expected = before.to_vec();
        expected[w::input_to_pose_calc..w::input_to_pose_calc + 2].fill(0);
        assert_eq!(
            (0..g.ram().len())
                .map(|at| g.ram().read_byte(at))
                .collect::<Vec<_>>()
                .as_slice(),
            expected.as_slice(),
            "unused normal selector clears only input result"
        );
    }
    match local {
        22 => expect(
            g,
            w::input_to_pose_calc,
            0,
            "grapple clears ordinary input result",
        ),
        29 => expect(
            g,
            w::input_to_pose_calc,
            0,
            "front-facing elevator clears input result",
        ),
        30 => {
            expect(
                g,
                a::atmospheric_gfx_frame_and_type,
                0x100,
                "Maridia ordinary dust",
            );
            expect(g, a::atmospheric_gfx_anim_timer, 0x8002, "dust first timer");
        }
        31 => {
            expect(
                g,
                a::atmospheric_gfx_frame_and_type,
                0x700,
                "default speed dust",
            );
            expect(g, w::speed_echoes_index, 0, "echo wraps after slot1");
            expect(
                g,
                a::speed_echo_xpos + 2,
                get(g, w::samus_x_pos),
                "echo samples final X",
            );
            expect(
                g,
                a::speed_echo_ypos + 2,
                get(g, w::samus_y_pos),
                "echo samples final Y",
            );
        }
        32 => expect(g, w::samus_x_speed_table_pointer, 0xa08d, "water table"),
        33 => expect(g, w::samus_x_speed_table_pointer, 0xa1dd, "lava table"),
        34 | 35 => expect(
            g,
            w::samus_x_speed_table_pointer,
            0xe000,
            "disabled water/gravity keeps table",
        ),
        36 => {
            expect(
                g,
                w::samus_x_base_speed,
                8,
                "quirked signed fraction avoids clamp",
            );
            expect(
                g,
                w::samus_x_base_subspeed,
                0x8000,
                "quirked signed fraction retained",
            );
        }
        37 => {
            expect(g, w::samus_x_base_speed, 8, "Native speed table clamp");
            expect(
                g,
                w::samus_x_base_subspeed,
                0,
                "positive excess fraction clamps",
            );
        }
        38 => {
            expect(
                g,
                w::samus_x_base_speed,
                0,
                "deceleration underflow clears whole",
            );
            expect(
                g,
                w::samus_x_accel_mode,
                0,
                "deceleration underflow clears mode",
            );
        }
        39 => {
            expect(g, w::samus_y_dir, 1, "space jump restarts upward");
            expect(g, 0xdfa, 1, "space jump flag low byte");
        }
        41 | 44 => expect(g, w::samus_y_dir, 2, "release/sign starts falling"),
        42 => {
            expect(g, w::samus_y_pos, 139, "positive bounce extra adds one");
            expect(g, w::samus_y_speed, 0, "bounce extra clears speed");
        }
        43 => expect(
            g,
            w::samus_y_pos,
            136,
            "knockback skips bounce vertical movement",
        ),
        45 => {
            expect(g, w::input_to_pose_calc, 0, "crouch input result consumed");
            expect(g, w::samus_y_dir, 0, "crouch landing clears direction");
            expect(g, w::samus_y_pos, 141, "crouch lands on authored floor");
            expect(g, w::samus_y_speed, 0, "crouch landing clears speed");
            expect(g, w::samus_y_subspeed, 0, "crouch landing clears fraction");
            expect(
                g,
                w::used_for_ball_bounce_on_landing,
                0,
                "crouch landing clears bounce",
            );
            expect(g, 0xaa4, 0, "crouch producer clears work");
        }
        46 => expect(
            g,
            w::samus_contact_damage_index,
            3,
            "late wall jump contact damage",
        ),
        47 => expect(
            g,
            w::input_to_pose_calc,
            5,
            "grapple preserves special input result",
        ),
        48 => expect(
            g,
            w::samus_x_base_speed,
            2,
            "scaled deceleration uses high product byte",
        ),
        _ => {}
    }
}

// Authored atmosphere lifecycle/content. No copied ROM data or expected algorithm.
fn atmosphere_seed(
    id: u32,
    g: &mut Game,
    loaded: &RomImage,
    native: &RomImage,
    lm: &mut Vec<u8>,
    nm: &mut Vec<u8>,
) {
    for (image, manifest, decoy) in [(loaded, lm, false), (native, nm, true)] {
        for kind in 0..8u16 {
            for (address, value) in [
                (0x908b93, if decoy { 0x9040 } else { 0x9000 }),
                (0x908bef, if decoy { 1 } else { 3 }),
                (0x908bff, if decoy { 0x9400 } else { 0x9200 + kind * 0x20 }),
            ] {
                author(
                    image,
                    address + u32::from(kind) * 2,
                    &value.to_le_bytes(),
                    manifest,
                );
            }
            for frame in 0..4u16 {
                author(
                    image,
                    0x909200 + u32::from(kind * 0x20 + frame * 2),
                    &(if decoy {
                        0x4000 + kind * 0x100 + frame
                    } else {
                        0x7000
                    })
                    .to_le_bytes(),
                    manifest,
                );
            }
        }
        for frame in 0..256u16 {
            author(
                image,
                0x909000 + u32::from(frame) * 2,
                &(if decoy { 3 + frame } else { 9 }).to_le_bytes(),
                manifest,
            );
        }
        for index in 390..=402u16 {
            author(
                image,
                0x92808d + u32::from(index) * 2,
                &0xa000u16.to_le_bytes(),
                manifest,
            );
        }
        author(
            image,
            0x92a000,
            &[1, 0, 0, 0, 0, if decoy { 0x99 } else { 0x21 }, 0x43],
            manifest,
        );
    }
    g.ram().fill(0x370..0x390, 0xcc);
    if id <= 4050 {
        return;
    }
    if matches!(id, 4051..=4053) {
        put(g, w::fx_type, if id == 4053 { 2 } else { 6 });
        put(g, w::fx_y_pos, 100);
        put(g, w::lava_acid_y_pos, 135);
        put(g, w::equipped_items, 0);
        put(g, w::nmi_frame_counter_word, u16::from(id == 4051));
        return;
    }
    if id == 4054 {
        put(g, w::area_index, 2);
        put(g, w::input_to_pose_calc, 0x401);
        return;
    }
    let kind = if id <= 4061 { (id - 4054) as u16 } else { 1 };
    put(g, a::atmospheric_gfx_frame_and_type, kind << 8);
    put(
        g,
        a::atmospheric_gfx_anim_timer,
        match id {
            4062 => 0,
            4063 | 4067 | 4068 => 1,
            4064 => 0x8000,
            4065 => 0x8001,
            4066 => 0xffff,
            _ => 2,
        },
    );
    put(g, a::atmospheric_gfx_x_pos, 136);
    put(g, a::atmospheric_gfx_y_pos, 100);
    match id {
        4067 => put(g, a::atmospheric_gfx_frame_and_type, 0x102),
        4068 => put(g, a::atmospheric_gfx_frame_and_type, 0x1ff),
        4069..=4072 => put(
            g,
            a::atmospheric_gfx_x_pos,
            [3, 4, 259, 260][(id - 4069) as usize],
        ),
        4073..=4076 => put(
            g,
            a::atmospheric_gfx_y_pos,
            [3, 4, 259, 260][(id - 4073) as usize],
        ),
        4077 => put(g, w::oam_next_ptr, 0x220), // Output overwrites its live cursor.
        4078 => put(g, w::oam_next_ptr, 0x774), // X store changes the following Y read.
        4079 => put(g, w::oam_next_ptr, 0x786), // Output aliases owned Samus X/fraction.
        4080 => {
            for slot in 0..4 {
                put(g, a::atmospheric_gfx_frame_and_type + slot * 2, 0x100);
                put(g, a::atmospheric_gfx_anim_timer + slot * 2, 2);
                put(
                    g,
                    a::atmospheric_gfx_x_pos + slot * 2,
                    10 + slot as u16 * 10,
                );
                put(g, a::atmospheric_gfx_y_pos + slot * 2, 100);
            }
        }
        4081 => {
            put(g, w::oam_next_ptr, 0x77c);
            put(g, a::atmospheric_gfx_y_pos, 5);
        }
        4082 => {
            put(g, a::atmospheric_gfx_x_pos, 1);
            put(g, w::layer1_x_pos, 0xfffd);
        }
        _ => {}
    }
}
unsafe fn atmosphere_invoke(id: u32, g: &mut Game) {
    use crate::game::{player_transitions as transitions, session_runtime::Runtime};
    if id <= 4050 {
        unsafe {
            crate::game::session_runtime::advance_special_movement(0x90a337, g);
        }
        assert_ne!(
            get(g, a::atmospheric_gfx_frame_and_type),
            0,
            "atmosphere.lifecycle.footstep_producer"
        );
    } else if id <= 4053 {
        transitions::advance_animation(
            &mut Runtime::new(g),
            transitions::state::AnimationStep::Gameplay,
        );
        assert_ne!(
            get(g, a::atmospheric_gfx_frame_and_type),
            0,
            "atmosphere.lifecycle.liquid_producer"
        );
    } else if id == 4054 {
        transitions::verify_momentum_action(
            &mut Runtime::new(g),
            transitions::state::MomentumAction::from_raw(5),
        );
        assert_eq!(
            get(g, a::atmospheric_gfx_frame_and_type + 4),
            0x600,
            "atmosphere.lifecycle.landing_producer"
        );
    }
    for tick in 0..if id <= 4054 { 4 } else { 1 } {
        transitions::draw_atmospheric(&mut Runtime::new(g));
        observer::observe(450 + tick, g);
    }
    match id {
        4062 => expect(
            g,
            a::atmospheric_gfx_anim_timer,
            0xffff,
            "zero timer wraps and waits",
        ),
        4063 => expect(
            g,
            a::atmospheric_gfx_frame_and_type,
            0x101,
            "timer expiry advances frame",
        ),
        4064 => expect(
            g,
            a::atmospheric_gfx_anim_timer,
            0x7fff,
            "signed timer boundary draws",
        ),
        4065 => expect(
            g,
            a::atmospheric_gfx_frame_and_type,
            0x100,
            "delayed activation retains frame",
        ),
        4067 => expect(
            g,
            a::atmospheric_gfx_frame_and_type,
            0,
            "terminal frame retires",
        ),
        4068 => expect(
            g,
            a::atmospheric_gfx_frame_and_type,
            0x200,
            "frame carry retains captured dispatch",
        ),
        4073 | 4076 => {
            assert_eq!(g.ram().read_byte(0x370), 132, "Y clipping retains X write");
            assert_eq!(g.ram().read_byte(0x371), 0xcc);
            expect(g, w::oam_next_ptr, 0, "Y clipping leaves cursor");
        }
        4077 => expect(
            g,
            w::oam_next_ptr,
            0x224,
            "captured cursor survives its own alias",
        ),
        4078 => expect(
            g,
            a::atmospheric_gfx_y_pos,
            0x8084,
            "X output precedes live Y reread",
        ),
        4079 => expect(g, w::samus_x_pos, 0x6084, "OAM writes publish into Session"),
        4080 => assert_eq!(
            [
                g.ram().read_byte(0x370),
                g.ram().read_byte(0x374),
                g.ram().read_byte(0x378),
                g.ram().read_byte(0x37c)
            ],
            [36, 26, 16, 6],
            "descending slots preserve render order"
        ),
        4081 => expect(
            g,
            a::atmospheric_gfx_frame_and_type,
            0x184,
            "frame sampled before OAM alias",
        ),
        _ => {}
    }
}

unsafe fn invoke(id: u32, g: &mut Game) {
    unsafe {
        if id >= 4049 {
            atmosphere_invoke(id, g);
            return;
        }
        crate::game::session_runtime::advance_special_movement(0x90a337, g);
    }
}
pub unsafe fn capture(id: u32, original: &[u8], g: &mut Game) -> Result<Vec<u8>, String> {
    unsafe {
        if !(FIRST..=LAST).contains(&id)
            || original.len() != 0x300000
            || g.g_cpu.is_null()
            || g.g_snes.is_null()
            || g.g_spc_player.is_null()
        {
            return Err("locomotion requires admitted ID and initialized host".into());
        }
        if !cfg!(target_endian = "little")
            || g.actor_identity_is_with_session()
            || observer::active()
        {
            return Err("locomotion ownership/endian/observer prerequisite".into());
        }
        let loaded_before = g.loaded_rom.clone();
        let native_before = g.capture_native_rom_binding();
        let fail_before = g.g_fail;
        let mut before = Box::new(Snapshot::INITIAL);
        {
            let snapshot_snes = g.g_snes;
            let snapshot_cpu = g.g_cpu;
            let snapshot_player = g.g_spc_player;
            let (backend, ram, _, _, _) = g.ram_and_snapshots();
            MakeSnapshot(
                &mut before,
                &*snapshot_snes,
                &*snapshot_cpu,
                snapshot_player,
                ram,
                backend,
            )
        };
        let cpu_before = *g.g_cpu;
        let cycles_before = (*g.g_snes).cpuCyclesLeft;
        let memory_ops_before = (*g.g_snes).cpuMemOps;
        let calling_before = g.g_calling_asm_from_c;
        let return_before = g.g_calling_asm_from_c_ret;
        let queue_before = crate::audio_snapshot::ApuQueueSnapshot::capture(g);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let (loaded, native, lm, nm) = seed(id, g);
            let initial = (0..g.ram().len())
                .map(|at| g.ram().read_byte(at))
                .collect::<Vec<_>>();
            let mut out = b"PTRS1".to_vec();
            let mut identity = id.to_le_bytes().to_vec();
            identity.push(1);
            record(&mut out, 0, &identity);
            record(&mut out, 8, &nm);
            record(&mut out, 10, &lm);
            let mut allocation = (loaded.len() as u32).to_le_bytes().to_vec();
            allocation.extend_from_slice(&(native.len() as u32).to_le_bytes());
            record(&mut out, 12, &allocation);
            observer::begin(out, g, Vec::new());
            observer::observe(1, g);
            invoke(id, g);
            check(id, g, &initial);
            assert!(
                !g.g_fail && !g.actor_identity_is_with_session(),
                "normal dispatch returns clean authority"
            );
            observer::scalar(4000, 1);
            observer::observe(11, g);
            assert!(
                g.capture_native_rom().same_allocation(&native)
                    && g.loaded_rom
                        .as_ref()
                        .expect("Loaded binding")
                        .same_allocation(&loaded)
            );
            let bytes = observer::finish().ok_or("missing locomotion trace")?;
            if bytes.len() > 2 * 1024 * 1024 {
                return Err("locomotion trace exceeds existing capacity".into());
            }
            Ok(bytes)
        }));
        let _ = observer::finish();
        g.loaded_rom = loaded_before;
        g.restore_native_rom_binding(native_before);
        {
            let snapshot_snes = g.g_snes;
            let snapshot_cpu = g.g_cpu;
            let (backend, ram, _, _, _) = g.ram_and_snapshots();
            RestoreSnapshot(
                &before,
                &mut *snapshot_snes,
                &mut *snapshot_cpu,
                ram,
                backend,
            )
        };
        before.audio.restore(g.g_snes);
        before.player.restore(g.g_spc_player);
        *g.g_cpu = cpu_before;
        (*g.g_snes).cpuCyclesLeft = cycles_before;
        (*g.g_snes).cpuMemOps = memory_ops_before;
        queue_before.restore(g);
        g.g_calling_asm_from_c = calling_before;
        g.g_calling_asm_from_c_ret = return_before;
        g.g_fail = fail_before;
        result.unwrap_or_else(|_| Err("locomotion invocation unexpectedly panicked".into()))
    }
}
