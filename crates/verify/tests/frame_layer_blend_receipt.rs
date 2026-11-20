//! Actual immutable default inputs and separate external prepared-setting receipt evidence.
#[path = "../src/replay/frame_layer_blend_receipt.rs"]
mod frame_layer_blend_receipt;
#[path = "../src/replay/frame_prelude_receipts.rs"]
mod frame_prelude_receipts;
#[path = "../src/replay/frame_prelude_seed.rs"]
mod frame_prelude_seed;
use frame_layer_blend_receipt::LayerBlendFault;
use game::production::frame::borrowed_layer_blend::{
    LayerBlendReceipt, LayerEncodingFault, PreparedLayerSettings, encode,
};
use game::state::video::{ColourMathLayers, ColourMathSelect, Layer, Layers, WindowSettings};

#[test]
fn exact_prepared_setting_encoding_validates_complete_layers() {
    let mut main = Layers::default();
    main.insert(Layer::Bg1);
    let mut sub = Layers::default();
    sub.insert(Layer::Bg2);
    let settings = PreparedLayerSettings {
        window_select: [WindowSettings(7), WindowSettings(8), WindowSettings(9)],
        main_screen: main,
        sub_screen: sub,
        main_window_mask: Layers::default(),
        sub_window_mask: Layers::default(),
        colour_math_select: ColourMathSelect(3),
        colour_math_layers: ColourMathLayers(4),
    };
    assert!(encode(settings).unwrap().eq(&[7, 8, 9, 1, 2, 0, 0, 3, 4]));
    let mut invalid = settings;
    invalid.main_window_mask.bits = 32;
    assert!(encode(invalid).is_err_and(|fault| fault == LayerEncodingFault::Layers));
}

#[test]
#[ignore = "requires qualified player ROM, current origin and external owner receipts"]
fn source_completion_requires_actual_default_operands_and_completed_haze() {
    let rom_bytes = std::fs::read(std::env::var_os("SM_FRAME_ROM").unwrap()).unwrap();
    let mut changed_rom = rom_bytes.clone();
    let rom = data::rom::Rom::new(rom_bytes).unwrap();
    let bytes = std::fs::read(std::env::var_os("SM_FRAME_ORIGIN_WRAM").unwrap()).unwrap();
    let current = frame_prelude_seed::decode(&bytes).unwrap().haze;
    let bands_projection =
        std::fs::read_to_string(std::env::var_os("SM_HAZE_READY").unwrap()).unwrap();
    let owner_projection =
        std::fs::read_to_string(std::env::var_os("SM_HAZE_READY_OWNER").unwrap()).unwrap();
    let bands = frame_prelude_receipts::haze_bands(&current, &bands_projection).unwrap();
    let ready = frame_prelude_receipts::haze_ready_owner(&bands, &owner_projection).unwrap();
    assert!(ready.dependency.eq(&current));
    let haze_projection =
        std::fs::read_to_string(std::env::var_os("SM_HAZE_TERMINAL").unwrap()).unwrap();
    let haze = frame_prelude_receipts::haze_terminal(&current, &haze_projection).unwrap();
    let projection = std::fs::read_to_string(std::env::var_os("SM_LAYER_BLEND").unwrap()).unwrap();
    let defaults = frame_layer_blend_receipt::defaults(&rom, &projection).unwrap();
    let completion = frame_layer_blend_receipt::completion(defaults, &haze, &projection).unwrap();
    assert!(completion.dependency.eq(&defaults));
    let restored: LayerBlendReceipt =
        canonical::from_bytes(&canonical::to_bytes(&completion)).unwrap();
    assert!(restored.eq(&completion));
    let external: toml::Value = toml::from_str(&projection).unwrap();
    let expected: Vec<u8> = [
        "REG_W12SEL",
        "REG_W34SEL",
        "REG_WOBJSEL",
        "REG_TM",
        "REG_TS",
        "REG_TMW",
        "REG_TSW",
        "NEXT_GAMEPLAY_CGWSEL",
        "NEXT_GAMEPLAY_CGADSUB",
    ]
    .iter()
    .map(|name| u8::try_from(external["terminal"][name].as_integer().unwrap()).unwrap())
    .collect();
    assert!(
        encode(completion.terminal)
            .unwrap()
            .as_slice()
            .eq(&expected)
    );
    let offset = usize::try_from(
        external["immutable_default_input"][0]["private_rom_file_offset"]
            .as_integer()
            .unwrap(),
    )
    .unwrap();
    changed_rom[offset] ^= 1;
    assert!(data::rom::Rom::new(changed_rom).is_err());
    let mut changed_default = defaults;
    changed_default.colour_math_select.0 ^= 1;
    assert!(
        frame_layer_blend_receipt::completion(changed_default, &haze, &projection)
            .is_err_and(|fault| fault == LayerBlendFault::Content)
    );
    assert!(
        frame_layer_blend_receipt::defaults(&rom, "")
            .is_err_and(|fault| fault == LayerBlendFault::Format)
    );
    for (from, to) in [
        (
            "expected_callbacks_used_as_inputs = false",
            "expected_callbacks_used_as_inputs = true",
        ),
        (
            "previous_output_mirrors_are_default_content_authority = false",
            "previous_output_mirrors_are_default_content_authority = true",
        ),
        (
            "read_only_at_actual_operation_completion = true",
            "read_only_at_actual_operation_completion = false",
        ),
    ] {
        let invalid = projection.replace(from, to);
        assert!(invalid.ne(&projection));
        assert!(
            frame_layer_blend_receipt::completion(defaults, &haze, &invalid)
                .is_err_and(|fault| fault == LayerBlendFault::Unsupported)
        );
    }
}
