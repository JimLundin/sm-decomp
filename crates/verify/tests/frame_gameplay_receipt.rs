//! Current scene import and deferred gameplay dependency validation; caller proves physical completion.
#[path = "../src/replay/frame_gameplay_receipt.rs"]
mod frame_gameplay_receipt;
#[path = "../src/replay/frame_prelude_seed.rs"]
mod frame_prelude_seed;
use frame_gameplay_receipt::GameplayReceiptFault;
use game::production::frame::borrowed::SpriteBuilderOrigin;
use game::production::frame::borrowed_gameplay::{
    GameplayDependencyFault, GameplayDispatchOffer, GameplayDispatchReceipt, GameplayRoute,
    SceneController,
};
use game::production::frame::borrowed_preparation::BuilderCompletionReceipt;

#[test]
fn dispatch_requires_actual_reset_and_current_scene() {
    let builder = SpriteBuilderOrigin {
        backing: [0; 544],
        cursor: 0,
        prepared_flags: 0,
        prepared_top: 0,
        prepared_bottom: 0,
    };
    let reset = BuilderCompletionReceipt {
        dependency: builder.clone(),
        terminal: builder,
    };
    let offer = GameplayDispatchOffer {
        dependency: SceneController::MainGameplay,
        route: GameplayRoute::MainGameplay,
    };
    assert!(
        offer
            .clone()
            .complete(SceneController::MainGameplay, &reset)
            .is_ok()
    );
    assert!(
        offer
            .clone()
            .complete(SceneController::Other, &reset)
            .is_err_and(|fault| fault == GameplayDependencyFault::Scene)
    );
    let mut invalid = reset;
    invalid.terminal.cursor = 2;
    assert!(
        offer
            .clone()
            .complete(SceneController::MainGameplay, &invalid)
            .is_err_and(|fault| fault == GameplayDependencyFault::Builder)
    );
    invalid.terminal.cursor = 0;
    invalid.terminal.backing[512] = 1;
    assert!(
        offer
            .complete(SceneController::MainGameplay, &invalid)
            .is_err_and(|fault| fault == GameplayDependencyFault::Builder)
    );
    assert!(
        frame_gameplay_receipt::offer(&[], "")
            .is_err_and(|fault| fault == GameplayReceiptFault::Format)
    );
}

#[test]
#[ignore = "requires independently qualified current scene origin and builder completion projection"]
fn actual_scene_owner_is_independent_of_haze_and_deferred_until_reset() {
    let bytes = std::fs::read(std::env::var_os("SM_FRAME_ORIGIN_WRAM").unwrap()).unwrap();
    let projection =
        std::fs::read_to_string(std::env::var_os("SM_GAMEPLAY_DISPATCH").unwrap()).unwrap();
    let current = frame_gameplay_receipt::current(&bytes, &projection).unwrap();
    let offer = frame_gameplay_receipt::offer(&bytes, &projection).unwrap();
    assert!(current.eq(&SceneController::MainGameplay));
    let restored: GameplayDispatchOffer =
        canonical::from_bytes(&canonical::to_bytes(&offer)).unwrap();
    assert!(restored.eq(&offer));
    let origin = frame_prelude_seed::decode(&bytes).unwrap();
    let content = std::fs::read(std::env::var_os("SM_BUILDER_TERMINAL_CONTENT").unwrap()).unwrap();
    // Separately qualified builder output is a codec fixture; actual physical reset completion belongs caller replay.
    let reset = BuilderCompletionReceipt {
        dependency: origin.builder,
        terminal: SpriteBuilderOrigin {
            backing: content.as_slice().try_into().unwrap(),
            cursor: 0,
            prepared_flags: 0,
            prepared_top: 0,
            prepared_bottom: 0,
        },
    };
    let complete = offer.complete(current, &reset).unwrap();
    let restored: GameplayDispatchReceipt =
        canonical::from_bytes(&canonical::to_bytes(&complete)).unwrap();
    assert!(restored.eq(&complete));
    let parsed: toml::Value = toml::from_str(&projection).ok().unwrap();
    let scene_at = usize::try_from(parsed["private_wram_offset"].as_integer().unwrap()).unwrap();
    let mut changed = bytes.clone();
    changed[scene_at] ^= 1;
    assert!(
        frame_gameplay_receipt::current(&changed, &projection)
            .is_err_and(|fault| fault == GameplayReceiptFault::Unsupported)
    );
    let mut changed_haze = bytes;
    changed_haze[2680] ^= 1;
    assert!(
        frame_gameplay_receipt::current(&changed_haze, &projection)
            .unwrap()
            .eq(&current)
    );
    assert!(
        frame_gameplay_receipt::current(&[], &projection)
            .is_err_and(|fault| fault == GameplayReceiptFault::Length)
    );
    for (from, to) in [
        (
            "simulation_haze_result_is_authority = false",
            "simulation_haze_result_is_authority = true",
        ),
        (
            "read_after_actual_builder_reset_completion = true",
            "read_after_actual_builder_reset_completion = false",
        ),
        (
            "origin_equal_at_actual_use = true",
            "origin_equal_at_actual_use = false",
        ),
    ] {
        let invalid = projection.replace(from, to);
        assert!(invalid.ne(&projection));
        assert!(
            frame_gameplay_receipt::offer(&changed_haze, &invalid)
                .is_err_and(|fault| fault == GameplayReceiptFault::Unsupported)
        );
    }
}
