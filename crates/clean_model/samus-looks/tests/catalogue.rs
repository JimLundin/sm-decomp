//! The catalogue matches the spec one for one, and the stages are numbered as the spec numbers
//! them.

use clean_model_samus_looks::{BehaviourId, CATALOGUE};
use clean_types::BoostStage;
use enum_map::{Enum, EnumMap};

/// Every member of the enum `K`, in its order.
fn members<K: Enum>() -> Vec<K> {
    EnumMap::<K, ()>::default()
        .into_iter()
        .map(|(member, ())| member)
        .collect()
}

#[test]
fn the_catalogue_names_every_behaviour_once() {
    for id in members::<BehaviourId>() {
        let rows = CATALOGUE
            .iter()
            .filter(|behaviour| behaviour.id == id)
            .count();
        assert_eq!(rows, 1, "{id:?} has {rows} rows in the catalogue");
    }
}

/// The stages numbered 0 to 4 (`BoostStage::ALL`, which `BoostingLap` numbers by the sounds
/// waiting) name every stage once, in the order the stages go up.
#[test]
fn the_stages_name_every_stage_once_in_order() {
    assert_eq!(BoostStage::ALL.to_vec(), members::<BoostStage>());
}
