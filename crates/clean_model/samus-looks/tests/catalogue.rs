//! The catalogue matches the spec one for one, and each table keyed by an enum names every
//! member once.

use clean_model_samus_looks::{BehaviourId, CATALOGUE, Rules};
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

/// The stages numbered 0 to 4 name every stage once, in the order the stages go up; the
/// stage-keyed tables (`Rules::laps_per_stage`, `Paces::boosted`) are `EnumMap`s, one entry per
/// stage by their type.
#[test]
fn the_stages_name_every_stage_once_in_order() {
    let stages = members::<BoostStage>();
    assert_eq!(BoostStage::ALL.to_vec(), stages);
    assert_eq!(Rules::VANILLA.laps_per_stage.len(), stages.len());
}
