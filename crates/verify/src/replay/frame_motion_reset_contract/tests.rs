use super::*;
use std::fmt::Write as _;

fn selected_text() -> String {
    let mut text = String::new();
    for (name, at, writer) in SELECTED {
        write!(text, "\n[[selected_output_binding]]\nowner = \"MotionReset\"\nsection = \"MotionResetTerminal\"\n\
            name = {name:?}\nprivate_wram_offset = {at}\nquantity_width = 2\nsource_writer_index = {writer}\n\
            source_comment = \"\"\nownership = \"ActualSelectedModelOrParentOwnedResult\"\nsource_terminal_not_runtime_input = true\n").unwrap();
    }
    text
}

#[derive(Deserialize)]
struct SelectionFixture {
    selected_output_binding: Vec<Selected>,
}

#[test]
fn established_current_facing_and_action_codecs_are_both_required() {
    assert_eq!(
        current_facing([u8::from(PoseFacing::Right), u8::from(MovementType::Running)]),
        Ok(CurrentFacing::Right)
    );
    assert_eq!(
        current_facing([u8::from(PoseFacing::Left), u8::from(MovementType::Running)]),
        Ok(CurrentFacing::Left)
    );
    assert_eq!(
        current_facing([
            u8::from(PoseFacing::Right),
            u8::from(MovementType::Standing)
        ]),
        Err(Fault::Facing)
    );
    assert_eq!(
        current_facing([
            u8::from(PoseFacing::Forward),
            u8::from(MovementType::Running)
        ]),
        Err(Fault::Facing)
    );
}

#[test]
fn every_named_selected_binding_remains_required_without_native_values() {
    let text = selected_text();
    let fixture: SelectionFixture = toml::from_str(&text).unwrap();
    assert_eq!(selected(&fixture.selected_output_binding), Ok(()));
    let mut value: toml::Value = toml::from_str(&text).unwrap();
    value
        .get_mut("selected_output_binding")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .pop();
    let fixture: SelectionFixture = toml::from_str(&toml::to_string(&value).unwrap()).unwrap();
    assert_eq!(
        selected(&fixture.selected_output_binding),
        Err(Fault::Selected)
    );
}

#[test]
fn selected_native_terminal_encoding_or_wrong_ownership_rejects() {
    let value: toml::Value = toml::from_str(&selected_text()).unwrap();
    let mut changed = value.clone();
    changed
        .get_mut("selected_output_binding")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()
        .as_table_mut()
        .unwrap()
        .insert("declared_encoding".to_owned(), toml::Value::Integer(0));
    assert!(toml::from_str::<SelectionFixture>(&toml::to_string(&changed).unwrap()).is_err());
    let mut changed = value;
    changed
        .get_mut("selected_output_binding")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap()
        .as_table_mut()
        .unwrap()
        .insert(
            "source_terminal_not_runtime_input".to_owned(),
            toml::Value::Boolean(false),
        );
    let fixture: SelectionFixture = toml::from_str(&toml::to_string(&changed).unwrap()).unwrap();
    assert_eq!(
        selected(&fixture.selected_output_binding),
        Err(Fault::Selected)
    );
}

#[test]
#[ignore = "requires accepted MotionReset source, current origin and whole profile directory"]
fn accepted_current_motion_reset_has_all_bindings_and_whole_no_effect_profile() {
    let text = std::fs::read(std::env::var_os("SM_MOTION_RESET_CONTRACT").unwrap()).unwrap();
    assert_eq!(
        crate::lock::Sha256::of(&text).to_string(),
        "cb8c715ada04990e6bae72b4e63e7ea88e2ca02ac46d160b07451b75953e7254"
    );
    let bytes = std::fs::read(std::env::var_os("SM_FRAME_CURRENT_ORIGIN").unwrap()).unwrap();
    let labels =
        std::fs::read_to_string(std::env::var_os("SM_PLAYER_DECODER_LABELS").unwrap()).unwrap();
    assert!(contract(&bytes, std::str::from_utf8(&text).unwrap(), &labels).is_ok());
    let directory = std::path::PathBuf::from(std::env::var_os("SM_FRAME_PROFILES").unwrap());
    let texts = [
        "profiles.toml",
        "portions.toml",
        "publication-bindings.toml",
        "sample-bindings.toml",
        "materialization-bindings.toml",
        "arithmetic-receipts.toml",
    ]
    .map(|name| std::fs::read_to_string(directory.join(name)).unwrap());
    let templates =
        super::super::frame_origin::profiles::decode(texts.each_ref().map(String::as_str)).unwrap();
    assert!(whole_template(&templates).is_ok());
}
