//! External preparation codec evidence; actual runtime hardware binding is proved by caller replay.
#[path = "../src/replay/frame_prelude_seed.rs"]
mod frame_prelude_seed;
#[path = "../src/replay/frame_preparation_receipts.rs"]
mod frame_preparation_receipts;
use frame_preparation_receipts::PreparationFault;
use game::production::frame::borrowed_preparation::{
    ActorEligibilityReceipt, ActorSlotIdentity, BuilderCompletionReceipt, CompletedSeedProducts,
    DebugAction, DebugCompletionReceipt, DeferredRandomReceipt, FirstSeedProduct,
    PaletteCompletionReceipt, PaletteDisposition, RandomCompletionReceipt, RandomDependencyFault,
    SecondSeedProductLow,
};
fn member(name: &str) -> Option<std::path::PathBuf> {
    std::env::var_os("SM_PREPARATION_PACKET").map(|path| std::path::PathBuf::from(path).join(name))
}
fn projection(name: &str) -> Option<String> {
    member(name).and_then(|path| std::fs::read_to_string(path).ok())
}
fn origin() -> Option<Vec<u8>> {
    std::env::var_os("SM_FRAME_ORIGIN_WRAM").and_then(|path| std::fs::read(path).ok())
}

#[test]
fn complete_origin_required_before_import() {
    for length in [0, 131_071, 131_073] {
        let bytes = vec![0; length];
        assert!(
            frame_preparation_receipts::random_origin(&bytes)
                .is_err_and(|fault| fault == PreparationFault::Length)
        );
        assert!(
            frame_preparation_receipts::debug_origin(&bytes)
                .is_err_and(|fault| fault == PreparationFault::Length)
        );
        assert!(
            frame_preparation_receipts::palette_origin(&bytes)
                .is_err_and(|fault| fault == PreparationFault::Length)
        );
    }
}

#[test]
#[ignore = "requires qualified origin and independent builder and actor observations"]
fn live_builder_and_borrowed_actor_use_the_current_origins() {
    let current = frame_prelude_seed::decode(&origin().unwrap()).unwrap();
    let content = std::fs::read(member("BUILDER-TERMINAL-CONTENT.private.bin").unwrap()).unwrap();
    let builder = BuilderCompletionReceipt::reset(current.builder.clone());
    assert!(builder.dependency.eq(&current.builder));
    assert!(builder.terminal.backing.as_slice().eq(&content));
    let restored: BuilderCompletionReceipt =
        canonical::from_bytes(&canonical::to_bytes(&builder)).unwrap();
    assert!(restored.eq(&builder));
    let mut changed = current.builder.clone();
    changed.backing[0] ^= 1;
    changed.backing[512] ^= 1;
    let changed = BuilderCompletionReceipt::reset(changed);
    assert!(
        !changed
            .terminal
            .backing
            .iter()
            .take(512)
            .eq(content.iter().take(512))
    );
    assert!(
        changed
            .terminal
            .backing
            .iter()
            .skip(512)
            .eq(content.iter().skip(512))
    );
    let actor_projection = projection("ACTOR-ELIGIBILITY-COMPLETION.private.toml").unwrap();
    let actors = frame_preparation_receipts::actors(&current.actors, &actor_projection).unwrap();
    assert!(actors.dependency.eq(&current.actors));
    assert!(
        actors
            .active
            .as_slice()
            .eq(&[ActorSlotIdentity(0), ActorSlotIdentity(1)])
    );
    assert!(actors.interactive.as_slice().eq(&[ActorSlotIdentity(1)]));
    let restored: ActorEligibilityReceipt =
        canonical::from_bytes(&canonical::to_bytes(&actors)).unwrap();
    assert!(restored.eq(&actors));
    let mut changed = current.actors.clone();
    changed.permit_marked_offscreen = false;
    assert!(
        frame_preparation_receipts::actors(&changed, &actor_projection)
            .is_err_and(|fault| fault == PreparationFault::Dependency)
    );
    let invalid = actor_projection.replace(
        "ai_or_geometry_authorized = false",
        "ai_or_geometry_authorized = true",
    );
    assert!(invalid.ne(&actor_projection));
    assert!(
        frame_preparation_receipts::actors(&current.actors, &invalid)
            .is_err_and(|fault| fault == PreparationFault::Unsupported)
    );
}

#[test]
#[ignore = "requires qualified origin and external random dependency projection; codec shape only"]
fn random_projection_checks_seed_and_completed_product_quantities() {
    let current = frame_preparation_receipts::random_origin(&origin().unwrap()).unwrap();
    let source = projection("RANDOM-COMPLETION.private.toml").unwrap();
    let parsed: toml::Value = toml::from_str(&source).unwrap();
    // These source quantities are codec fixture data, not actual runtime ALU inputs or proof of hardware completion.
    let products = CompletedSeedProducts {
        first: FirstSeedProduct(
            u16::try_from(
                parsed["actual_product_dependency"][0]["required_actual_result_encoding"]
                    .as_integer()
                    .unwrap(),
            )
            .unwrap(),
        ),
        second_low: SecondSeedProductLow(
            u8::try_from(
                parsed["actual_product_dependency"][1]["required_actual_result_encoding"]
                    .as_integer()
                    .unwrap(),
            )
            .unwrap(),
        ),
    };
    let offer = frame_preparation_receipts::random_offer(current, &source).unwrap();
    let restored: DeferredRandomReceipt =
        canonical::from_bytes(&canonical::to_bytes(&offer)).unwrap();
    assert!(restored.eq(&offer));
    let receipt = offer.clone().complete(current, products).unwrap();
    assert_eq!(restored.complete(current, products).unwrap(), receipt);
    assert!(receipt.dependency.eq(&current));
    let restored: RandomCompletionReceipt =
        canonical::from_bytes(&canonical::to_bytes(&receipt)).unwrap();
    assert!(restored.eq(&receipt));
    let mut altered = products;
    altered.first.0 ^= 1;
    assert!(
        offer
            .clone()
            .complete(current, altered)
            .is_err_and(|fault| fault == RandomDependencyFault::Products)
    );
    let mut altered = current;
    altered.0 ^= 1;
    assert!(
        offer
            .complete(altered, products)
            .is_err_and(|fault| fault == RandomDependencyFault::Seed)
    );
    assert!(
        frame_preparation_receipts::random_offer(altered, &source)
            .is_err_and(|fault| fault == PreparationFault::Dependency)
    );
    let invalid = source.replace(
        "source_hardware_result_values_used_as_device_inputs = false",
        "source_hardware_result_values_used_as_device_inputs = true",
    );
    assert!(invalid.ne(&source));
    assert!(
        frame_preparation_receipts::random_offer(current, &invalid)
            .is_err_and(|fault| fault == PreparationFault::Unsupported)
    );
}

#[test]
#[ignore = "requires qualified current physical debug and palette origins plus external completions"]
fn debug_and_palette_receipts_preserve_actual_dependencies() {
    let bytes = origin().unwrap();
    let current_debug = frame_preparation_receipts::debug_origin(&bytes).unwrap();
    let source = projection("DEBUG-COMPLETION.private.toml").unwrap();
    let receipt = frame_preparation_receipts::debug(current_debug, &source).unwrap();
    assert_eq!(receipt.action, DebugAction::None);
    assert!(!receipt.skip_processing);
    let restored: DebugCompletionReceipt =
        canonical::from_bytes(&canonical::to_bytes(&receipt)).unwrap();
    assert!(restored.eq(&receipt));
    let mut changed = current_debug;
    changed.controller_one.0 |= 0x80;
    assert!(
        frame_preparation_receipts::debug(changed, &source)
            .is_err_and(|fault| fault == PreparationFault::Dependency)
    );
    let current_palette = frame_preparation_receipts::palette_origin(&bytes).unwrap();
    let source = projection("PALETTE-OBJECTS-COMPLETION.private.toml").unwrap();
    let receipt = frame_preparation_receipts::palette(current_palette, &source).unwrap();
    assert_eq!(receipt.terminal, PaletteDisposition::RetainCurrent);
    let restored: PaletteCompletionReceipt =
        canonical::from_bytes(&canonical::to_bytes(&receipt)).unwrap();
    assert!(restored.eq(&receipt));
    let mut changed = current_palette;
    changed.enabled = false;
    assert!(
        frame_preparation_receipts::palette(changed, &source)
            .is_err_and(|fault| fault == PreparationFault::Dependency)
    );
    let invalid = source.replace(
        "palette_replaced_with_future_buffer = false",
        "palette_replaced_with_future_buffer = true",
    );
    assert!(invalid.ne(&source));
    assert!(
        frame_preparation_receipts::palette(current_palette, &invalid)
            .is_err_and(|fault| fault == PreparationFault::Unsupported)
    );
    assert!(
        frame_preparation_receipts::palette(current_palette, "")
            .is_err_and(|fault| fault == PreparationFault::Format)
    );
}
