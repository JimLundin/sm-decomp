//! Cross-owner visual publication, projected aliases, and callback unwind.
#![forbid(unsafe_code)]
use crate::{
    game::{session_runtime::Runtime, visual_host::Aliases},
    game_state::Game,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
fn read(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes([g.ram().read_byte(at), g.ram().read_byte(at + 1)])
}
fn put(g: &mut Game, at: usize, value: u16) {
    let _ = &[(); 0x20000][at..at + 2];
    g.ram().copy_in(at, &value.to_le_bytes());
}
pub(crate) fn run() -> bool {
    catch_unwind(AssertUnwindSafe(check)).is_ok()
}
fn check() {
    let mut game = Game::new();
    game.ram().fill(0..game.ram().len(), 0);
    {
        let mut h = Runtime::new(&mut game);
        h.session_mut().presentation.visual_mut().colors[5] = 0x1234;
        h.session_mut().presentation.visual_mut().targets[5] = 0x5678;
        h.session_mut()
            .presentation
            .palette_mut()
            .crystal_flash_delay = 3;
        h.session_mut().inventory.health.current = 99;
        h.session_mut().enemy_projectiles.slots[0].definition = 0xbeef;
        h.session_mut().projectiles.shots[0].x = 0x4321;
        // These real visual aliases now target other typed aggregate fields.
        h.set_palette_color_slice(0x6c8, 0xa55a);
        assert_eq!(
            h.session().mechanisms.tilemap[0],
            0xa55a,
            "visual.guard.palette-mechanism-alias"
        );
        h.set_palette_color_slice(0x20a, 0xabcd);
        assert_eq!(
            h.session().presentation.visual().targets[5],
            0xabcd,
            "visual.guard.palette-target-alias"
        );
        h.foreign(|g| {
            for (at, value) in [
                (0xc00a, 0x1234),
                (0xc20a, 0xabcd),
                (0xdf2, 3),
                (0x9c2, 99),
                (0x1997, 0xbeef),
                (0xb64, 0x4321),
                (0xc6c8, 0xa55a),
            ] {
                assert_eq!(read(g, at), value, "visual.guard.publish-all-{at:x}");
                put(g, at, value ^ 0x1111);
            }
        });
        assert_eq!(
            h.session().presentation.visual().colors[5],
            0x0325,
            "visual.guard.import-current"
        );
        assert_eq!(
            h.session().presentation.visual().targets[5],
            0xbadc,
            "visual.guard.import-target"
        );
        assert_eq!(
            h.session().presentation.palette().crystal_flash_delay,
            0x1112,
            "visual.guard.import-timer"
        );
        assert_eq!(
            h.session().inventory.health.current,
            0x1172,
            "visual.guard.import-inventory"
        );
        assert_eq!(
            h.session().enemy_projectiles.slots[0].definition,
            0xaffe,
            "visual.guard.import-enemy"
        );
        assert_eq!(
            h.session().projectiles.shots[0].x,
            0x5230,
            "visual.guard.import-player-shot"
        );
        assert_eq!(
            h.session().mechanisms.tilemap[0],
            0xb44b,
            "visual.guard.import-mechanism"
        );
        assert!(
            catch_unwind(AssertUnwindSafe(|| h.foreign(|g| {
                for at in [0xc00a, 0xc20a, 0xdf2, 0x9c2, 0x1997, 0xb64, 0xc6c8] {
                    put(g, at, 0xdead);
                }
                panic!("intentional visual aggregate foreign unwind");
            })))
            .is_err()
        );
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                let _ = h.session();
            }))
            .is_err(),
            "visual.guard.owner-relinquished"
        );
    }
    for at in [0xc00a, 0xc20a, 0xdf2, 0x9c2, 0x1997, 0xb64, 0xc6c8] {
        assert_eq!(read(&game, at), 0xdead, "visual.guard.no-stale-drop-{at:x}");
    }
}
