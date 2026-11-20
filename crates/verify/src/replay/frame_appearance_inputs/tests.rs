use super::*;
use std::fmt::Write as _;

fn proof(bytes: &[u8]) -> String {
    let mut proof = String::from(
        "format = \"bounded-current-appearance-neighbors-v1\"\n\
         actual_appearance_playhead_boost_and_figure_are_required = true\n\
         source_pose_playhead_substitution_allowed = false\n\
         sound_queue_authority = \"ActualCompletedSenderOfferAndCurrentSenderState\"\n\
         source_queue_indices_are_model_input = false\n\
         liquid_history_air_authority = \"AcceptedIndependentS5AnimationRetainedPhysicsDecodeAt3615\"\n\
         current_dry_surfaces_are_history_authority = false\n",
    );
    for (name, offset) in NEIGHBORS {
        for opportunity in OPPORTUNITIES {
            write!(
                proof,
                "\n[[current_appearance_neighbor]]\nname = {name:?}\n\
                 private_wram_offset = {offset}\nquantity_width = 2\n\
                 opportunity_source_index = {opportunity}\n\
                 private_current_encoding = {}\norigin_equal = true\n\
                 preceding_writer_indices = []\n\
                 authority = \"ActualCurrentNeighborOwnerRevalidatedAtOpportunity\"\n\
                 physical_read_at_projection_claimed = false\n",
                word(bytes, offset).unwrap(),
            )
            .unwrap();
        }
    }
    proof
}

#[test]
fn qualified_contact_reads_only_current_suppressors() {
    let mut bytes = vec![0; 131_072];
    assert_eq!(
        qualified_contact(&bytes, &proof(&bytes)),
        Ok(Set::default())
    );
    for (address, owner) in [
        (0x1f51, ContactSuppressor::Cinematic),
        (0x179c, ContactSuppressor::Boss),
        (0x0a68, ContactSuppressor::SpecialColours),
    ] {
        bytes[address] = 1;
        let mut expected = Set::default();
        expected[owner] = true;
        assert_eq!(qualified_contact(&bytes, &proof(&bytes)), Ok(expected));
        bytes[address] = 0;
    }
    assert_eq!(
        qualified_contact(&[], ""),
        Err(AppearanceInputFault::Length)
    );
}

#[test]
fn incomplete_stale_or_promoted_source_descriptors_reject() {
    let bytes = vec![0; 131_072];
    let proof = proof(&bytes);
    let value: toml::Value = toml::from_str(&proof).unwrap();
    for (field, replacement) in [
        ("private_wram_offset", toml::Value::Integer(0x09a2)),
        ("quantity_width", toml::Value::Integer(1)),
        ("private_current_encoding", toml::Value::Integer(1)),
        ("origin_equal", toml::Value::Boolean(false)),
        (
            "physical_read_at_projection_claimed",
            toml::Value::Boolean(true),
        ),
        (
            "preceding_writer_indices",
            toml::Value::Array(vec![toml::Value::Integer(1)]),
        ),
    ] {
        let mut changed = value.clone();
        changed
            .get_mut("current_appearance_neighbor")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .first_mut()
            .unwrap()
            .as_table_mut()
            .unwrap()
            .insert(field.to_owned(), replacement);
        assert_eq!(
            qualified_contact(&bytes, &toml::to_string(&changed).unwrap()),
            Err(AppearanceInputFault::Provenance)
        );
    }
    for field in [
        "source_queue_indices_are_model_input",
        "source_pose_playhead_substitution_allowed",
        "current_dry_surfaces_are_history_authority",
    ] {
        let mut changed = value.clone();
        changed
            .as_table_mut()
            .unwrap()
            .insert(field.to_owned(), toml::Value::Boolean(true));
        assert_eq!(
            qualified_contact(&bytes, &toml::to_string(&changed).unwrap()),
            Err(AppearanceInputFault::Provenance)
        );
    }
    let mut changed = value;
    changed
        .get_mut("current_appearance_neighbor")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .pop();
    assert_eq!(
        qualified_contact(&bytes, &toml::to_string(&changed).unwrap()),
        Err(AppearanceInputFault::Provenance)
    );
}

#[test]
#[ignore = "requires independently accepted appearance neighbor proof and qualified origin"]
fn actual_source_packet_imports_without_native_queue_or_liquid_history_substitution() {
    let path = std::env::var_os("SM_FRAME_APPEARANCE_CURRENT_PROOF").unwrap();
    let proof = std::fs::read(path).unwrap();
    assert_eq!(
        crate::lock::Sha256::of(&proof).to_string(),
        "ee0f4c5816570843d001dde8103d97aada9b2a225f59ce889d811dd72e09be7b"
    );
    let path = std::env::var_os("SM_FRAME_CURRENT_ORIGIN").unwrap();
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(
        crate::lock::Sha256::of(&bytes).to_string(),
        "bfee6ac13da413a36ad42194a36aa3224af6a84e818092b1e7040ed4782f03db"
    );
    assert!(qualified_contact(&bytes, std::str::from_utf8(&proof).unwrap()).is_ok());
}
