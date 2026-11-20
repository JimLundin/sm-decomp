use super::*;
use std::fmt::Write as _;

fn seed() -> Vec<u8> {
    let mut bytes = vec![0; 131_072];
    for field in [source::ROOM_WIDTH_IN_BLOCKS, source::ROOM_HEIGHT_IN_BLOCKS] {
        let at = usize::from(field.offset);
        bytes
            .get_mut(at..at.saturating_add(2))
            .unwrap()
            .copy_from_slice(&2_u16.to_le_bytes());
    }
    bytes
}

fn field(proof: &mut String, bytes: &[u8], name: &str, at: usize, width: usize) {
    let pin = Sha256::of(bytes.get(at..at.saturating_add(width)).unwrap());
    write!(
        proof,
        "\n[[current_player_input]]\nname = {name:?}\nprivate_wram_offset = {at}\n\
         quantity_width = {width}\nprivate_current_sha256 = \"{pin}\"\n\
         origin_equal_at_preparation = true\npreceding_writer_indices = []\n\
         movement_window_writer_indices = []\nauthority = \"ActualCurrentNeighborOwner\"\n\
         physical_read_at_preparation_claimed = false\n"
    )
    .unwrap();
}

fn proof(bytes: &[u8]) -> String {
    let mut proof = String::from(
        "format = \"bounded-current-player-preparation-v1\"\n\
         opportunity_source = 2189\n\
         required_previous_owner = \"ActualCompletedS2ActorAndSpriteObjectWork\"\n\
         appearance_figure_playhead_source_substitution_allowed = false\n\
         actual_playing_sprite_required = true\n\
         actual_completed_spin_and_clip_end_results_required = true\n\
         current_actor_geometry_authority = \"ActualCompletedS2CurrentActorOwner\"\n\
         origin_roster_is_current_geometry_authority = false\n\
         revalidate_actual_world_and_control_dependencies = true\n",
    );
    for (name, scalar) in SCALARS {
        field(&mut proof, bytes, name, usize::from(scalar.offset), 2);
    }
    field(&mut proof, bytes, "CurrentRoomLevelWords", 0x10002, 8);
    field(&mut proof, bytes, "CurrentRoomBtsBytes", 0x16402, 4);
    proof.push_str(
        "\n[world]\n\
         room_content = \"CheckedPlayerRomRoomDecodeForActualCurrentRoomPointer\"\n\
         cell_and_permission_codec = \"ExistingMovementLevelWorldDecodedBlockKindAndCurrentBts\"\n\
         reaction_permission = \"CurrentDecodedSpecialCommon45Only\"\n\
         immutable_slope_authority = \"ExistingCheckedSlopes\"\n\
         current_world_is_future_output = false\n\
         \n[remaining_actor_join]\n\
         source_current_geometry_not_frontloaded = true\n\
         require_actual_completed_actor_owner_geometry = true\n\
         required_properties = \"InteractiveCurrentRosterOccupiedSolidOrFrozenCurrentCentreAndRadius\"\n\
         source_only_scalar_roster_is_insufficient = true\n\
         actual_actor_geometry_model_or_typed_external_completion_receipt_required = true\n\
         new_actor_geometry_proof_in_this_packet = false\n",
    );
    proof
}

#[test]
fn qualified_source_requires_full_current_room_and_scalar_fingerprints() {
    let mut bytes = seed();
    let proof = proof(&bytes);
    assert!(super::super::qualified(&bytes, &proof).is_ok());
    for offset in [0x008b, 0x10002, 0x16402] {
        *bytes.get_mut(offset).unwrap() = 1;
        assert_eq!(validate(&bytes, &proof), Err(PlayerInputFault::Provenance));
        *bytes.get_mut(offset).unwrap() = 0;
    }
    assert_eq!(validate(&[], &proof), Err(PlayerInputFault::Length));
}

#[test]
fn stale_descriptors_and_false_physical_read_claims_reject() {
    let bytes = seed();
    let value: toml::Value = toml::from_str(&proof(&bytes)).unwrap();
    for (key, replacement) in [
        ("origin_equal_at_preparation", toml::Value::Boolean(false)),
        (
            "physical_read_at_preparation_claimed",
            toml::Value::Boolean(true),
        ),
        ("quantity_width", toml::Value::Integer(1)),
        (
            "movement_window_writer_indices",
            toml::Value::Array(vec![toml::Value::Integer(1)]),
        ),
        (
            "preceding_writer_indices",
            toml::Value::Array(vec![toml::Value::Integer(1)]),
        ),
    ] {
        let mut changed = value.clone();
        changed
            .get_mut("current_player_input")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .first_mut()
            .unwrap()
            .as_table_mut()
            .unwrap()
            .insert(key.to_owned(), replacement);
        assert_eq!(
            validate(&bytes, &toml::to_string(&changed).unwrap()),
            Err(PlayerInputFault::Provenance)
        );
    }
}

#[test]
fn actor_roster_and_future_appearance_substitution_reject() {
    let bytes = seed();
    let value: toml::Value = toml::from_str(&proof(&bytes)).unwrap();
    for key in [
        "origin_roster_is_current_geometry_authority",
        "appearance_figure_playhead_source_substitution_allowed",
    ] {
        let mut changed = value.clone();
        changed
            .as_table_mut()
            .unwrap()
            .insert(key.to_owned(), toml::Value::Boolean(true));
        assert_eq!(
            validate(&bytes, &toml::to_string(&changed).unwrap()),
            Err(PlayerInputFault::Provenance)
        );
    }
    let mut changed = value;
    changed
        .get_mut("remaining_actor_join")
        .unwrap()
        .as_table_mut()
        .unwrap()
        .insert(
            "require_actual_completed_actor_owner_geometry".to_owned(),
            toml::Value::Boolean(false),
        );
    assert_eq!(
        validate(&bytes, &toml::to_string(&changed).unwrap()),
        Err(PlayerInputFault::Provenance)
    );
}

#[test]
#[ignore = "requires independently accepted player current proof and qualified origin"]
fn actual_source_packet_imports_without_actor_or_appearance_substitution() {
    let path = std::env::var_os("SM_FRAME_PLAYER_CURRENT_PROOF").unwrap();
    let proof = std::fs::read(path).unwrap();
    assert_eq!(
        Sha256::of(&proof).to_string(),
        "2dc66fd3daac37afccb9fa7e8ae7a1d2b39a13c5a832bf298743c7458827ee8c"
    );
    let path = std::env::var_os("SM_FRAME_CURRENT_ORIGIN").unwrap();
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(
        Sha256::of(&bytes).to_string(),
        "bfee6ac13da413a36ad42194a36aa3224af6a84e818092b1e7040ed4782f03db"
    );
    assert!(super::super::qualified(&bytes, std::str::from_utf8(&proof).unwrap()).is_ok());
}
