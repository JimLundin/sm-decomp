//! External Haze owner receipt and exact band encoding development evidence.
#[path = "../src/replay/frame_prelude_receipts.rs"]
mod frame_prelude_receipts;
#[path = "../src/replay/frame_prelude_seed.rs"]
mod frame_prelude_seed;
use frame_prelude_receipts::PreludeReceiptFault;
use game::production::frame::borrowed::ColourComponent;
use game::production::frame::borrowed_prelude_receipts::{
    BackdropBand, BandEncodingFault, BandIntensity, HazeBandsReceipt, HazeReadyOwnerReceipt,
    HazeTerminalReceipt, LayerBlendDisposition, PreparedFixedColour, WindowTwoDisposition,
    encode_bands, encode_fixed_colour,
};

#[test]
fn completed_band_encoding_preserves_component_and_order() {
    assert!(encode_fixed_colour(PreparedFixedColour::Black).eq(&[0x20, 0x40, 0x80]));
    for (component, encoding) in [
        (ColourComponent::Red, 0x23),
        (ColourComponent::Green, 0x43),
        (ColourComponent::Blue, 0x83),
    ] {
        let bands = [BackdropBand {
            component,
            intensity: BandIntensity(3),
        }; 16];
        assert!(encode_bands(&bands).unwrap().eq(&[encoding; 16]));
    }
    let bands = [BackdropBand {
        component: ColourComponent::Red,
        intensity: BandIntensity(16),
    }; 16];
    assert!(encode_bands(&bands).is_err_and(|fault| fault == BandEncodingFault::Intensity));
    let window = WindowTwoDisposition::Unchanged;
    let restored: WindowTwoDisposition =
        canonical::from_bytes(&canonical::to_bytes(&window)).unwrap();
    assert_eq!(restored, window);
}

#[test]
#[ignore = "requires qualified current origin and separate external Haze receipts"]
fn external_haze_receipts_retain_current_dependency_and_separate_terminal() {
    let bytes = std::fs::read(std::env::var_os("SM_FRAME_ORIGIN_WRAM").unwrap()).unwrap();
    let current = frame_prelude_seed::decode(&bytes).unwrap().haze;
    let ready = std::fs::read_to_string(std::env::var_os("SM_HAZE_READY").unwrap()).unwrap();
    let terminal = std::fs::read_to_string(std::env::var_os("SM_HAZE_TERMINAL").unwrap()).unwrap();
    let owner_stage =
        std::fs::read_to_string(std::env::var_os("SM_HAZE_READY_OWNER").unwrap()).unwrap();
    let bands = frame_prelude_receipts::haze_bands(&current, &ready).unwrap();
    let complete = frame_prelude_receipts::haze_terminal(&current, &terminal).unwrap();
    let owner = frame_prelude_receipts::haze_ready_owner(&bands, &owner_stage).unwrap();
    assert!(owner.dependency.eq(&current));
    assert_eq!(owner.fixed_colour, PreparedFixedColour::Black);
    let restored: HazeReadyOwnerReceipt =
        canonical::from_bytes(&canonical::to_bytes(&owner)).unwrap();
    assert!(restored.eq(&owner));
    let premature = owner_stage.replace(
        "first_countdown_writer_complete_before_ready = false",
        "first_countdown_writer_complete_before_ready = true",
    );
    assert!(
        frame_prelude_receipts::haze_ready_owner(&bands, &premature)
            .is_err_and(|fault| fault == PreludeReceiptFault::Unsupported)
    );
    assert!(bands.dependency.eq(&current));
    assert!(complete.dependency.eq(&current));
    assert_eq!(complete.blend, LayerBlendDisposition::NoBackgroundThree);
    let restored: HazeBandsReceipt = canonical::from_bytes(&canonical::to_bytes(&bands)).unwrap();
    assert!(restored.eq(&bands));
    let restored: HazeTerminalReceipt =
        canonical::from_bytes(&canonical::to_bytes(&complete)).unwrap();
    assert!(restored.eq(&complete));
    let mut changed = current.clone();
    changed.default_blend.0 ^= 1;
    assert!(
        frame_prelude_receipts::haze_bands(&changed, &ready)
            .is_err_and(|fault| fault == PreludeReceiptFault::Dependency)
    );
    assert!(
        frame_prelude_receipts::haze_terminal(&changed, &terminal)
            .is_err_and(|fault| fault == PreludeReceiptFault::Dependency)
    );
    assert!(
        frame_prelude_receipts::haze_bands(&current, "")
            .is_err_and(|fault| fault == PreludeReceiptFault::Format)
    );
    assert!(
        frame_prelude_receipts::haze_terminal(&current, "")
            .is_err_and(|fault| fault == PreludeReceiptFault::Format)
    );
    for (from, to) in [
        (
            "full_haze_terminal_publication_at_ready = false",
            "full_haze_terminal_publication_at_ready = true",
        ),
        (
            "remaining_source_tail_fully_billed = true",
            "remaining_source_tail_fully_billed = false",
        ),
        (
            "expected_callbacks_used_as_inputs = false",
            "expected_callbacks_used_as_inputs = true",
        ),
        ("intensity = 0", "intensity = 16"),
    ] {
        let invalid = ready.replace(from, to);
        assert!(invalid.ne(&ready));
        assert!(
            frame_prelude_receipts::haze_bands(&current, &invalid)
                .is_err_and(|fault| fault == PreludeReceiptFault::Unsupported)
        );
    }
    let early = terminal.replace(
        "full_haze_terminal_only_after_physical_portion = true",
        "full_haze_terminal_only_after_physical_portion = false",
    );
    assert!(
        frame_prelude_receipts::haze_terminal(&current, &early)
            .is_err_and(|fault| fault == PreludeReceiptFault::Unsupported)
    );
}
