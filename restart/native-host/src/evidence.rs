//! Explicit complete counter field mapping for binary reports.
#[cfg(any(test, feature = "private-checks"))]
mod field_oracle {
    use super::*;

    // This oracle constructs expected values without calling ToEvidence.
    // Every leaf receives a distinct nonzero value, including nested arrays.
    trait Sentinel: Sized {
        fn sentinel(next: &mut u64) -> (Self, Value);
    }

    impl Sentinel for u64 {
        fn sentinel(next: &mut u64) -> (Self, Value) {
            let value = *next;
            *next += 1;
            (value, Value::Integer(i64::try_from(value).unwrap()))
        }
    }

    impl<T: Sentinel, const N: usize> Sentinel for [T; N] {
        fn sentinel(next: &mut u64) -> (Self, Value) {
            let mut expected = Vec::with_capacity(N);
            let values = std::array::from_fn(|_| {
                let (value, encoded) = T::sentinel(next);
                expected.push(encoded);
                value
            });
            (values, Value::Array(expected))
        }
    }

    impl Sentinel for crate::ItemInstructionEntryCounters {
        fn sentinel(next: &mut u64) -> (Self, Value) {
            let (entries, e0) = Sentinel::sentinel(next);
            let (c_entries, e1) = Sentinel::sentinel(next);
            let (rust_entries, e2) = Sentinel::sentinel(next);
            let (rust_completed, e3) = Sentinel::sentinel(next);
            let (rust_refused, e4) = Sentinel::sentinel(next);
            (
                Self { entries, c_entries, rust_entries, rust_completed, rust_refused },
                Value::Table(Map::from([
                    ("entries".into(), e0),
                    ("c_entries".into(), e1),
                    ("rust_entries".into(), e2),
                    ("rust_completed".into(), e3),
                    ("rust_refused".into(), e4),
                ])),
            )
        }
    }

    macro_rules! check_fields {
        ($next:expr, $counter:ident, [$($field:ident => $name:literal),+ $(,)?]) => {{
            let mut expected = Map::new();
            $(
                let ($field, encoded) = Sentinel::sentinel($next);
                assert!(expected.insert($name.into(), encoded).is_none());
            )+
            // A complete literal initializer fails compilation if a field is added.
            let counters = crate::$counter { $($field),+ };
            let expected = Value::Table(expected);
            let actual = counters.to_evidence().unwrap();
            assert_eq!(actual, expected, stringify!($counter));
            let wire = sm_binary_evidence::encode(&actual).unwrap();
            assert_eq!(sm_binary_evidence::decode(&wire).unwrap(), expected, stringify!($counter));
        }};
    }

    #[cfg_attr(test, test)]
    pub(crate) fn all_native_counter_fields_preserve_distinct_sentinels() {
        let mut next = 0x0102u64;
        check_fields!(&mut next, CameraCounters, [
            c_entries => "c_entries",
            rust_entries => "rust_entries",
            rust_completed => "rust_completed",
            rust_refused => "rust_refused",
            routine_c_entries => "routine_c_entries",
            routine_c_completed => "routine_c_completed",
            routine_rust_entries => "routine_rust_entries",
            routine_rust_completed => "routine_rust_completed",
        ]);
        check_fields!(&mut next, CeresDoorCounters, [
            c_entries => "c_entries",
            rust_entries => "rust_entries",
            rust_completed => "rust_completed",
            rust_refused => "rust_refused",
            foreign_requests => "foreign_requests",
            foreign_completions => "foreign_completions",
            proximity_successes => "proximity_successes",
            solid_setups => "solid_setups",
        ]);
        check_fields!(&mut next, ProjectileCounters, [
            c_entries => "c_entries",
            rust_entries => "rust_entries",
            rust_completed => "rust_completed",
            rust_refused => "rust_refused",
            foreign_calls => "foreign_calls",
            drawn_oam_entries => "drawn_oam_entries",
            ceres_high_bit_entries => "ceres_high_bit_entries",
            ceres_high_bit_completed => "ceres_high_bit_completed",
            ceres_high_bit_refused => "ceres_high_bit_refused",
            other_refused => "other_refused",
        ]);
        check_fields!(&mut next, DoorScrollingCounters, [
            c_entries => "c_entries",
            rust_entries => "rust_entries",
            rust_completed => "rust_completed",
            rust_refused => "rust_refused",
            routine_c_entries => "routine_c_entries",
            routine_c_completed => "routine_c_completed",
            routine_rust_entries => "routine_rust_entries",
            routine_rust_completed => "routine_rust_completed",
            service_requests => "service_requests",
            service_completions => "service_completions",
            foreign_requests => "foreign_requests",
            foreign_completions => "foreign_completions",
            child_entries => "child_entries",
            child_completions => "child_completions",
            coordinator_entries => "coordinator_entries",
            standalone_entries => "standalone_entries",
        ]);
        check_fields!(&mut next, ItemInstructionEntryCounters, [
            entries => "entries",
            c_entries => "c_entries",
            rust_entries => "rust_entries",
            rust_completed => "rust_completed",
            rust_refused => "rust_refused",
        ]);
        check_fields!(&mut next, ItemInstructionCounters, [
            total => "total",
            scope_standalone => "scope_standalone",
            scope_service52 => "scope_service52",
            other_standalone => "other_standalone",
            other_service52 => "other_service52",
            morph_standalone => "morph_standalone",
            morph_service52 => "morph_service52",
            tank_standalone => "tank_standalone",
            tank_service52 => "tank_service52",
            service_requests => "service_requests",
            service_completions => "service_completions",
        ]);
        check_fields!(&mut next, PersistenceCounters, [
            entries => "entries",
            c_entries => "c_entries",
            rust_entries => "rust_entries",
            rust_refused => "rust_refused",
            rust_completed => "rust_completed",
            write_requests => "write_requests",
            write_completions => "write_completions",
        ]);
        check_fields!(&mut next, PlayerMotionCounters, [
            entries => "entries",
            c_entries => "c_entries",
            rust_entries => "rust_entries",
            rust_refused => "rust_refused",
            rust_completed => "rust_completed",
            terrain_unavailable => "terrain_unavailable",
            terrain_requests => "terrain_requests",
            terrain_completions => "terrain_completions",
            terrain_invalid_requests => "terrain_invalid_requests",
            enemy_hit_returns => "enemy_hit_returns",
            terrain_returns => "terrain_returns",
            feedback_returns => "feedback_returns",
            no_collision_move_returns => "no_collision_move_returns",
        ]);
        check_fields!(&mut next, RoomTransitionCounters, [
            c_entries => "c_entries",
            rust_entries => "rust_entries",
            rust_completed => "rust_completed",
            rust_refused => "rust_refused",
            root_pending => "root_pending",
            root_resumed => "root_resumed",
            routine_c_entries => "routine_c_entries",
            routine_c_completed => "routine_c_completed",
            routine_rust_entries => "routine_rust_entries",
            routine_rust_completed => "routine_rust_completed",
            service_requests => "service_requests",
            service_completions => "service_completions",
            foreign_requests => "foreign_requests",
            foreign_completions => "foreign_completions",
            service_pending => "service_pending",
            child_entries => "child_entries",
            child_completions => "child_completions",
        ]);
        check_fields!(&mut next, RipperCounters, [
            entries => "entries",
            c_entries => "c_entries",
            rust_entries => "rust_entries",
            rust_refused => "rust_refused",
            rust_completed => "rust_completed",
            terrain_unavailable => "terrain_unavailable",
            service_requests => "service_requests",
            service_completions => "service_completions",
            invalid_requests => "invalid_requests",
        ]);
        check_fields!(&mut next, ScrollCounters, [
            c_entries => "c_entries",
            rust_entries => "rust_entries",
            rust_completed => "rust_completed",
            rust_refused => "rust_refused",
            foreign_requests => "foreign_requests",
            foreign_completions => "foreign_completions",
            flush_requests => "flush_requests",
            flush_completions => "flush_completions",
            columns_flushed => "columns_flushed",
            rows_flushed => "rows_flushed",
            entry_vram_dma_bytes => "entry_vram_dma_bytes",
            total_flush_requests => "total_flush_requests",
            total_flush_completions => "total_flush_completions",
            consumed_columns => "consumed_columns",
            consumed_rows => "consumed_rows",
            vram_dma_bytes => "vram_dma_bytes",
        ]);
        check_fields!(&mut next, WaverCounters, [
            entries => "entries",
            c_entries => "c_entries",
            rust_entries => "rust_entries",
            rust_refused => "rust_refused",
            rust_completed => "rust_completed",
            terrain_unavailable => "terrain_unavailable",
            service_requests => "service_requests",
            service_completions => "service_completions",
            invalid_requests => "invalid_requests",
        ]);
    }
}

#[cfg(feature = "private-checks")]
pub(crate) use field_oracle::all_native_counter_fields_preserve_distinct_sentinels;
use sm_binary_evidence::{Map, ToEvidence, Value};

#[cfg(any(test, feature = "private-checks"))]
#[cfg_attr(test, test)]
pub(crate) fn counter_transport_retains_last_service_and_child() {
    let mut counts = crate::RoomTransitionCounters::default();
    counts.foreign_completions[52] = 17;
    counts.child_completions[52][2] = 29;
    let bytes = sm_binary_evidence::encode(&counts.to_evidence().unwrap()).unwrap();
    let value = sm_binary_evidence::decode(&bytes).unwrap();
    assert_eq!(value["foreign_completions"].as_array().unwrap().len(), 53);
    assert_eq!(value["foreign_completions"][52].as_integer(), Some(17));
    assert_eq!(value["child_completions"].as_array().unwrap().len(), 53);
    assert_eq!(value["child_completions"][52].as_array().unwrap().len(), 3);
    assert_eq!(value["child_completions"][52][2].as_integer(), Some(29));
    counts.service_requests[52] = u64::MAX;
    assert!(counts.to_evidence().is_err());
}
impl ToEvidence for crate::CameraCounters {
    fn to_evidence(&self) -> Result<Value, String> {
        Ok(Value::Table(Map::from([
            ("c_entries".into(), self.c_entries.to_evidence()?),
            ("rust_entries".into(), self.rust_entries.to_evidence()?),
            ("rust_completed".into(), self.rust_completed.to_evidence()?),
            ("rust_refused".into(), self.rust_refused.to_evidence()?),
            ("routine_c_entries".into(), self.routine_c_entries.to_evidence()?),
            ("routine_c_completed".into(), self.routine_c_completed.to_evidence()?),
            ("routine_rust_entries".into(), self.routine_rust_entries.to_evidence()?),
            ("routine_rust_completed".into(), self.routine_rust_completed.to_evidence()?),
        ])))
    }
}
impl ToEvidence for crate::CeresDoorCounters {
    fn to_evidence(&self) -> Result<Value, String> {
        Ok(Value::Table(Map::from([
            ("c_entries".into(), self.c_entries.to_evidence()?),
            ("rust_entries".into(), self.rust_entries.to_evidence()?),
            ("rust_completed".into(), self.rust_completed.to_evidence()?),
            ("rust_refused".into(), self.rust_refused.to_evidence()?),
            ("foreign_requests".into(), self.foreign_requests.to_evidence()?),
            ("foreign_completions".into(), self.foreign_completions.to_evidence()?),
            ("proximity_successes".into(), self.proximity_successes.to_evidence()?),
            ("solid_setups".into(), self.solid_setups.to_evidence()?),
        ])))
    }
}
impl ToEvidence for crate::ProjectileCounters {
    fn to_evidence(&self) -> Result<Value, String> {
        Ok(Value::Table(Map::from([
            ("c_entries".into(), self.c_entries.to_evidence()?),
            ("rust_entries".into(), self.rust_entries.to_evidence()?),
            ("rust_completed".into(), self.rust_completed.to_evidence()?),
            ("rust_refused".into(), self.rust_refused.to_evidence()?),
            ("foreign_calls".into(), self.foreign_calls.to_evidence()?),
            ("drawn_oam_entries".into(), self.drawn_oam_entries.to_evidence()?),
            ("ceres_high_bit_entries".into(), self.ceres_high_bit_entries.to_evidence()?),
            ("ceres_high_bit_completed".into(), self.ceres_high_bit_completed.to_evidence()?),
            ("ceres_high_bit_refused".into(), self.ceres_high_bit_refused.to_evidence()?),
            ("other_refused".into(), self.other_refused.to_evidence()?),
        ])))
    }
}
impl ToEvidence for crate::DoorScrollingCounters {
    fn to_evidence(&self) -> Result<Value, String> {
        Ok(Value::Table(Map::from([
            ("c_entries".into(), self.c_entries.to_evidence()?),
            ("rust_entries".into(), self.rust_entries.to_evidence()?),
            ("rust_completed".into(), self.rust_completed.to_evidence()?),
            ("rust_refused".into(), self.rust_refused.to_evidence()?),
            ("routine_c_entries".into(), self.routine_c_entries.to_evidence()?),
            ("routine_c_completed".into(), self.routine_c_completed.to_evidence()?),
            ("routine_rust_entries".into(), self.routine_rust_entries.to_evidence()?),
            ("routine_rust_completed".into(), self.routine_rust_completed.to_evidence()?),
            ("service_requests".into(), self.service_requests.to_evidence()?),
            ("service_completions".into(), self.service_completions.to_evidence()?),
            ("foreign_requests".into(), self.foreign_requests.to_evidence()?),
            ("foreign_completions".into(), self.foreign_completions.to_evidence()?),
            ("child_entries".into(), self.child_entries.to_evidence()?),
            ("child_completions".into(), self.child_completions.to_evidence()?),
            ("coordinator_entries".into(), self.coordinator_entries.to_evidence()?),
            ("standalone_entries".into(), self.standalone_entries.to_evidence()?),
        ])))
    }
}
impl ToEvidence for crate::ItemInstructionEntryCounters {
    fn to_evidence(&self) -> Result<Value, String> {
        Ok(Value::Table(Map::from([
            ("entries".into(), self.entries.to_evidence()?),
            ("c_entries".into(), self.c_entries.to_evidence()?),
            ("rust_entries".into(), self.rust_entries.to_evidence()?),
            ("rust_completed".into(), self.rust_completed.to_evidence()?),
            ("rust_refused".into(), self.rust_refused.to_evidence()?),
        ])))
    }
}
impl ToEvidence for crate::ItemInstructionCounters {
    fn to_evidence(&self) -> Result<Value, String> {
        Ok(Value::Table(Map::from([
            ("total".into(), self.total.to_evidence()?),
            ("scope_standalone".into(), self.scope_standalone.to_evidence()?),
            ("scope_service52".into(), self.scope_service52.to_evidence()?),
            ("other_standalone".into(), self.other_standalone.to_evidence()?),
            ("other_service52".into(), self.other_service52.to_evidence()?),
            ("morph_standalone".into(), self.morph_standalone.to_evidence()?),
            ("morph_service52".into(), self.morph_service52.to_evidence()?),
            ("tank_standalone".into(), self.tank_standalone.to_evidence()?),
            ("tank_service52".into(), self.tank_service52.to_evidence()?),
            ("service_requests".into(), self.service_requests.to_evidence()?),
            ("service_completions".into(), self.service_completions.to_evidence()?),
        ])))
    }
}
impl ToEvidence for crate::PersistenceCounters {
    fn to_evidence(&self) -> Result<Value, String> {
        Ok(Value::Table(Map::from([
            ("entries".into(), self.entries.to_evidence()?),
            ("c_entries".into(), self.c_entries.to_evidence()?),
            ("rust_entries".into(), self.rust_entries.to_evidence()?),
            ("rust_refused".into(), self.rust_refused.to_evidence()?),
            ("rust_completed".into(), self.rust_completed.to_evidence()?),
            ("write_requests".into(), self.write_requests.to_evidence()?),
            ("write_completions".into(), self.write_completions.to_evidence()?),
        ])))
    }
}
impl ToEvidence for crate::PlayerMotionCounters {
    fn to_evidence(&self) -> Result<Value, String> {
        Ok(Value::Table(Map::from([
            ("entries".into(), self.entries.to_evidence()?),
            ("c_entries".into(), self.c_entries.to_evidence()?),
            ("rust_entries".into(), self.rust_entries.to_evidence()?),
            ("rust_refused".into(), self.rust_refused.to_evidence()?),
            ("rust_completed".into(), self.rust_completed.to_evidence()?),
            ("terrain_unavailable".into(), self.terrain_unavailable.to_evidence()?),
            ("terrain_requests".into(), self.terrain_requests.to_evidence()?),
            ("terrain_completions".into(), self.terrain_completions.to_evidence()?),
            ("terrain_invalid_requests".into(), self.terrain_invalid_requests.to_evidence()?),
            ("enemy_hit_returns".into(), self.enemy_hit_returns.to_evidence()?),
            ("terrain_returns".into(), self.terrain_returns.to_evidence()?),
            ("feedback_returns".into(), self.feedback_returns.to_evidence()?),
            ("no_collision_move_returns".into(), self.no_collision_move_returns.to_evidence()?),
        ])))
    }
}
impl ToEvidence for crate::RoomTransitionCounters {
    fn to_evidence(&self) -> Result<Value, String> {
        Ok(Value::Table(Map::from([
            ("c_entries".into(), self.c_entries.to_evidence()?),
            ("rust_entries".into(), self.rust_entries.to_evidence()?),
            ("rust_completed".into(), self.rust_completed.to_evidence()?),
            ("rust_refused".into(), self.rust_refused.to_evidence()?),
            ("root_pending".into(), self.root_pending.to_evidence()?),
            ("root_resumed".into(), self.root_resumed.to_evidence()?),
            ("routine_c_entries".into(), self.routine_c_entries.to_evidence()?),
            ("routine_c_completed".into(), self.routine_c_completed.to_evidence()?),
            ("routine_rust_entries".into(), self.routine_rust_entries.to_evidence()?),
            ("routine_rust_completed".into(), self.routine_rust_completed.to_evidence()?),
            ("service_requests".into(), self.service_requests.to_evidence()?),
            ("service_completions".into(), self.service_completions.to_evidence()?),
            ("foreign_requests".into(), self.foreign_requests.to_evidence()?),
            ("foreign_completions".into(), self.foreign_completions.to_evidence()?),
            ("service_pending".into(), self.service_pending.to_evidence()?),
            ("child_entries".into(), self.child_entries.to_evidence()?),
            ("child_completions".into(), self.child_completions.to_evidence()?),
        ])))
    }
}
impl ToEvidence for crate::RipperCounters {
    fn to_evidence(&self) -> Result<Value, String> {
        Ok(Value::Table(Map::from([
            ("entries".into(), self.entries.to_evidence()?),
            ("c_entries".into(), self.c_entries.to_evidence()?),
            ("rust_entries".into(), self.rust_entries.to_evidence()?),
            ("rust_refused".into(), self.rust_refused.to_evidence()?),
            ("rust_completed".into(), self.rust_completed.to_evidence()?),
            ("terrain_unavailable".into(), self.terrain_unavailable.to_evidence()?),
            ("service_requests".into(), self.service_requests.to_evidence()?),
            ("service_completions".into(), self.service_completions.to_evidence()?),
            ("invalid_requests".into(), self.invalid_requests.to_evidence()?),
        ])))
    }
}
impl ToEvidence for crate::ScrollCounters {
    fn to_evidence(&self) -> Result<Value, String> {
        Ok(Value::Table(Map::from([
            ("c_entries".into(), self.c_entries.to_evidence()?),
            ("rust_entries".into(), self.rust_entries.to_evidence()?),
            ("rust_completed".into(), self.rust_completed.to_evidence()?),
            ("rust_refused".into(), self.rust_refused.to_evidence()?),
            ("foreign_requests".into(), self.foreign_requests.to_evidence()?),
            ("foreign_completions".into(), self.foreign_completions.to_evidence()?),
            ("flush_requests".into(), self.flush_requests.to_evidence()?),
            ("flush_completions".into(), self.flush_completions.to_evidence()?),
            ("columns_flushed".into(), self.columns_flushed.to_evidence()?),
            ("rows_flushed".into(), self.rows_flushed.to_evidence()?),
            ("entry_vram_dma_bytes".into(), self.entry_vram_dma_bytes.to_evidence()?),
            ("total_flush_requests".into(), self.total_flush_requests.to_evidence()?),
            ("total_flush_completions".into(), self.total_flush_completions.to_evidence()?),
            ("consumed_columns".into(), self.consumed_columns.to_evidence()?),
            ("consumed_rows".into(), self.consumed_rows.to_evidence()?),
            ("vram_dma_bytes".into(), self.vram_dma_bytes.to_evidence()?),
        ])))
    }
}
impl ToEvidence for crate::WaverCounters {
    fn to_evidence(&self) -> Result<Value, String> {
        Ok(Value::Table(Map::from([
            ("entries".into(), self.entries.to_evidence()?),
            ("c_entries".into(), self.c_entries.to_evidence()?),
            ("rust_entries".into(), self.rust_entries.to_evidence()?),
            ("rust_refused".into(), self.rust_refused.to_evidence()?),
            ("rust_completed".into(), self.rust_completed.to_evidence()?),
            ("terrain_unavailable".into(), self.terrain_unavailable.to_evidence()?),
            ("service_requests".into(), self.service_requests.to_evidence()?),
            ("service_completions".into(), self.service_completions.to_evidence()?),
            ("invalid_requests".into(), self.invalid_requests.to_evidence()?),
        ])))
    }
}
