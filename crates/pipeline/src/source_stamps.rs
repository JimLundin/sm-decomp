//! Compiled application inputs for the conservative Offline evidence contract.
//!
//! Keep this data usable by production composition and synthetic source fixtures.
//! Implementation owners supply their own paths; no lower crate imports pipeline.
pub(crate) const HOST_SOURCES: &[(&str, &str)] = &[
    ("crates/pipeline/Cargo.toml", include_str!("../Cargo.toml")),
    ("crates/pipeline/src/main.rs", include_str!("main.rs")),
    ("crates/pipeline/src/offline.rs", include_str!("offline.rs")),
    ("crates/pipeline/src/source_stamps.rs", include_str!("source_stamps.rs")),
    ("crates/pipeline/src/batch.rs", include_str!("batch.rs")),
    ("crates/pipeline/src/builds.rs", include_str!("builds.rs")),
    ("crates/pipeline/src/cleanup.rs", include_str!("cleanup.rs")),
    ("crates/pipeline/src/fixture_cli.rs", include_str!("fixture_cli.rs")),
    ("crates/pipeline/src/source_inventory.rs", include_str!("source_inventory.rs")),
    ("crates/pipeline/src/translation.rs", include_str!("translation.rs")),
    ("crates/pipeline/src/translation_native.rs", include_str!("translation_native.rs")),
    ("crates/pipeline/src/ridley_subsystem_fixtures.rs", include_str!("ridley_subsystem_fixtures.rs")),
];

pub(crate) fn all() -> Vec<(&'static str, &'static str)> {
    HOST_SOURCES
        .iter()
        .chain(offline_verification::IMPLEMENTATION_SOURCES)
        .chain(decomp_source::IMPLEMENTATION_SOURCES)
        .chain(tool_execution::IMPLEMENTATION_SOURCES)
        .chain(translation_execution::compiler_runtime::IMPLEMENTATION_SOURCES)
        .copied()
        .collect()
}
