//! Child of cpu_infra: real initialized snapshot/reset transitions, no frame execution.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::{game::actors::mutable_content_checks as fixture, game_state::Game};

/// Called only by initialized room_world case11, after SnesInit/audio/PPU setup.
pub(crate) unsafe fn run(game: &mut Game) -> Vec<u8> {
    unsafe {
        assert!(
            !game.g_snes.is_null() && !game.g_cpu.is_null() && !game.g_spc_player.is_null(),
            "actor.mutable.initialized_hardware_required"
        );
        let snes = game.g_snes;
        assert!(
            !(*snes).ppu.is_null()
                && !(*snes).my_ppu.is_null()
                && !(*snes).snes_ppu.is_null()
                && !(*snes).apu.is_null()
                && !(*snes).cart.is_null()
        );
        // Cart retains its original owned allocation and valid reset vector. Only game
        // native/loaded readers use this synthetic image; no CPU frame is executed.
        let image = fixture::image();
        fixture::seed(game, &image);
        let before_ram = (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>();
        fixture::consume_checked(game, false, "hardware_before_writer_positive_control");
        game.ram().copy_in(0, &before_ram);
        {
            let snapshot_snes = snes;
            let snapshot_cpu = game.g_cpu;
            let snapshot_player = game.g_spc_player;
            let (backend, ram, _, _, before, owner) = game.ram_and_owned_snapshots();
            super::make_snapshot_with_player_owner(
                before,
                &*snapshot_snes,
                &*snapshot_cpu,
                snapshot_player,
                owner,
                ram,
                backend,
            )
        };
        game.actor_snapshots.before = Some(game.capture_actor_snapshot());
        let queue = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
        let expected_image = fixture::written_image(&image);
        fixture::write_checked(game, &image);
        fixture::consume_checked(game, true, "hardware_after_writer");

        // Match actual before-sidecar restoration order, including queue/player.
        queue.restore(game);
        game.restore_native_player_snapshot(crate::game_state::PlayerSnapshotSource::Before);
        let restore = game
            .actor_snapshots
            .before
            .as_ref()
            .unwrap()
            .preflight(game.actor_identity(), game.world())
            .unwrap();
        {
            let snapshot_snes = snes;
            let snapshot_cpu = game.g_cpu;
            let (backend, ram, _, _, before, _) = game.ram_and_owned_snapshots();
            super::restore_snapshot(
                before,
                &mut *snapshot_snes,
                &mut *snapshot_cpu,
                ram,
                backend,
            )
        };
        restore.commit(game.actor_identity_mut());
        assert_eq!(
            (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .collect::<Vec<_>>()
                .as_slice(),
            before_ram.as_slice(),
            "actor.mutable.actual_cpu_restores_all_ram"
        );
        assert_eq!(
            game.capture_actor_snapshot(),
            game.actor_snapshots.before.as_ref().unwrap().clone(),
            "actor.mutable.actual_cpu_restores_actor_payload"
        );
        fixture::assert_image(game, &image, &expected_image, "actual_cpu_restore");
        fixture::consume_checked(game, true, "after_actual_cpu_restore");

        crate::rtl::rtl_reset(2, game);
        assert_eq!(
            game.g_snes, snes,
            "actor.mutable.reset_preserves_snes_owner"
        );
        assert_eq!(
            game.ram().read_byte(0x611),
            1,
            "actor.mutable.actual_reset_entry_pending"
        );
        assert_eq!(
            game.snes_frame_counter, 0,
            "actor.mutable.actual_reset_frame_counter"
        );
        fixture::assert_image(game, &image, &expected_image, "actual_rtl_reset");
        fixture::consume_checked(game, true, "after_actual_rtl_reset");
        fixture::assert_image(game, &image, &expected_image, "after_actual_reset_consumer");
        let mut output = b"AKMC1".to_vec();
        output.extend((0..game.ram().len()).map(|at| game.ram().read_byte(at)));
        output.extend_from_slice(&[0x4c, 0x80]);
        output
    }
}
