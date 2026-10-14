//! Confined Cargo acceptance for the preserved original production jobs.
use crate::{
    acquire_lease_until, builds, family,
    family_model::Asset,
    inputs::{self, bounded},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    os::unix::fs::PermissionsExt,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};
use translation_execution::{
    OperationContext,
    budget::ByteBudget,
    confinement::{Program, ReadOnlyBind, RunLimits},
    model::{digest_valid, sealed, sha256},
    operation,
    retention::{self, Reference},
};

const MAX_SOURCE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_DEPENDENCY_BYTES: u64 = 512 * 1024 * 1024;
const MAX_FILES: usize = 32768;
const MAX_RECORD_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CargoKind {
    SharedGameHost,
    CompactController,
    PauseEquipment,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CargoClosure {
    pub root: PathBuf,
    pub files: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CargoLimits {
    pub timeout_seconds: u64,
    pub writable_bytes: u64,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CargoContract {
    pub schema: String,
    pub kind: CargoKind,
    pub source_files: BTreeMap<String, String>,
    pub approved_files: Vec<String>,
    pub vendor: CargoClosure,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependency_cache: Option<CargoClosure>,
    pub limits: CargoLimits,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PreparedCargo {
    pub contract: CargoContract,
    pub dependencies_sha256: String,
    pub verification_sha256: String,
    pub test_sources: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct StrictRuntime {
    pub helper: PathBuf,
    pub rustc: PathBuf,
    pub cargo: PathBuf,
    pub producer: PathBuf,
    pub operation_root: PathBuf,
    pub pipeline_root: PathBuf,
    pub workers: usize,
    pub gate_timeout_seconds: u64,
    pub max_output_bytes: u64,
    pub deadline_unix_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CargoReceipt {
    pub prepared_sha256: String,
    pub candidate_sha256: String,
    pub context: OperationContext,
    pub runtime_sha256: String,
    pub max_output_bytes: u64,
    pub reserved_bytes: u64,
    pub verdict: Asset,
    pub passed: bool,
    pub diagnostic: String,
    pub evidence_bytes: u64,
    pub runtime_binary: Option<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CargoVerdict {
    pub schema: String,
    pub evidence_kind: String,
    pub kind: CargoKind,
    pub passed: bool,
    pub test_count: usize,
    pub commands: Vec<CargoCommand>,
    pub prepared_sha256: String,
    pub candidate_sha256: String,
    pub context: OperationContext,
    pub runtime: StrictRuntime,
    pub reserved_bytes: u64,
    pub diagnostic: String,
    pub source_snapshots: BTreeMap<String, Reference>,
    pub supporting_records: Vec<Reference>,
    pub operations: Vec<CargoInvocation>,
    pub runtime_binary: Option<Reference>,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CargoInvocation {
    pub command_id: String,
    pub context: OperationContext,
    pub request: Reference,
    pub program: Program,
    pub limits: RunLimits,
    pub deadline_unix_ms: u64,
    pub operation_root: PathBuf,
    pub result_sha256: String,
    pub references: Vec<Reference>,
}

/// A bounded prerequisite import, separate from original Cargo acceptance.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CargoEnvironmentReceipt {
    pub schema: String,
    pub prepared_sha256: String,
    pub candidate_sha256: String,
    pub context: OperationContext,
    pub runtime_sha256: String,
    pub passed: bool,
    pub diagnostic: String,
    pub cache: Option<CargoClosure>,
    pub record: Asset,
    pub evidence_bytes: u64,
    pub reserved_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct EnvironmentRequest {
    schema: String,
    build_dependencies: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct EnvironmentRecord {
    schema: String,
    prepared_sha256: String,
    candidate_sha256: String,
    context: OperationContext,
    runtime: StrictRuntime,
    request: Reference,
    program: Program,
    limits: RunLimits,
    deadline_unix_ms: u64,
    reserved_bytes: u64,
    passed: bool,
    diagnostic: String,
    cache: Option<CargoClosure>,
    snapshots: BTreeMap<String, Reference>,
    operation_result_sha256: String,
    references: Vec<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ProducerRequest {
    schema: String,
    kind: CargoKind,
    command_id: String,
    dependency_cache: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CargoIntent {
    context: OperationContext,
    prepared_sha256: String,
    candidate_sha256: String,
    runtime_sha256: String,
    deadline_unix_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct InvocationIntent {
    context: OperationContext,
    program: Program,
    limits: RunLimits,
    deadline_unix_ms: u64,
    reserved_bytes: u64,
    request: Reference,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CargoSuite {
    pub source: String,
    pub names: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CargoCommand {
    pub id: String,
    pub argv: Vec<String>,
    pub suites: Vec<CargoSuite>,
    pub filtered: bool,
    pub export_binary: bool,
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).into()).collect()
}

impl CargoKind {
    /// Reviewed original commands and controls are adapter code, never rewrite
    /// input or a count inferred from the candidate's own stdout.
    pub(crate) fn commands(self) -> Vec<CargoCommand> {
        let suite = |source: &str, names: &[&str]| {
            let mut names = strings(names);
            names.sort();
            CargoSuite { source: source.into(), names }
        };
        match self {
            Self::SharedGameHost => vec![CargoCommand {
                id: "shared-host-rust-tests".into(),
                argv: strings(&[
                    "cargo",
                    "test",
                    "-p",
                    "port",
                    "--locked",
                    "--offline",
                    "--test",
                    "game_host",
                    "--test",
                    "samus_projectile_frame",
                    "--test",
                    "room_scroll",
                ]),
                suites: vec![
                    suite(
                        "crates/port/tests/game_host.rs",
                        &[
                            "whole_projectile_frame_transfers_current_wram_between_ordered_hdma_requests",
                            "unsupported_later_slot_refuses_without_publishing_earlier_scheduler_work",
                            "first_failing_request_retains_rust_ownership",
                            "later_failing_request_does_not_repeat_the_first_effect_or_publish_partial_wram",
                            "unsupported_lower_slot_after_host_effects_is_an_owned_failure",
                            "response_kind_mismatch_is_an_owned_failure",
                            "whole_projectile_frame_routes_repeated_bomb_terrain_contacts_through_shared_plm_request",
                            "second_existing_module_flushes_all_seventeen_columns_synchronously",
                            "malformed_later_room_column_refuses_before_the_first_flush",
                            "fixture_compatibility_rejects_unknown_or_parameterized_calls_before_requests",
                        ],
                    ),
                    suite(
                        "crates/port/tests/samus_projectile_frame.rs",
                        &[
                            "ten_slot_scheduler_executes_delete_goto_and_data_in_reverse_order",
                            "unsupported_later_preinstruction_refuses_before_publishing_earlier_work",
                            "native_plm_and_hdma_dependencies_do_not_enter_immutable_selection",
                            "plain_and_plasma_beams_follow_the_two_native_flicker_patterns",
                            "unaligned_spritemap_stores_extension_before_unsigned_vertical_and_tile",
                            "oam_alias_can_activate_a_lower_projectile_in_the_same_pass",
                            "player_pass_draws_echoes_before_the_frozen_trail_overlay",
                            "elevated_player_pass_subtracts_eight_and_skips_echo_flicker",
                            "ceres_player_uninitialized_draw_y_refuses_before_publishing",
                            "ceres_uninitialized_explosion_locals_refuse_without_wram_changes",
                            "ceres_ignored_mode7_return_refuses_even_after_an_initialized_slot",
                            "mode7_rotation_preserves_signed_products_and_wrapped_camera_subtraction",
                        ],
                    ),
                    suite(
                        "crates/port/tests/room_scroll.rs",
                        &[
                            "column_and_row_preserve_all_four_native_flip_orders",
                            "background_buffers_keep_native_264_byte_offset_and_foreground_unchanged",
                            "row_contains_seventeen_blocks_and_keeps_wrapped_and_unwrapped_lengths",
                            "column_uses_low_byte_product_and_wrapping_word_stride",
                            "mode7_and_frozen_frame_do_not_write_queue_or_tile_buffers",
                            "rejects_full_frame_tile_extent_before_publishing_registers_or_previous_blocks",
                            "full_frame_schedules_foreground_and_background_x_then_y",
                            "full_display_flushes_seventeen_columns_before_incrementing_each_block",
                            "full_display_rejects_all_columns_before_first_flush_or_write",
                        ],
                    ),
                ],
                filtered: false,
                export_binary: false,
            }],
            Self::CompactController => vec![
                CargoCommand {
                    id: "controller-unit-tests".into(),
                    argv: strings(&[
                        "cargo",
                        "test",
                        "-p",
                        "pipeline",
                        "--locked",
                        "--offline",
                        "--bin",
                        "sm-pipeline",
                        "decomp_prefix::tests",
                    ]),
                    suites: vec![suite(
                        "crates/pipeline/src/decomp_prefix.rs",
                        &[
                            "decomp_prefix::tests::clearing_severs_poll_ownership_and_completion_never_borrows_future_input",
                            "decomp_prefix::tests::observer_keeps_read_addresses_passive_and_rejects_lua_delimiter",
                            "decomp_prefix::tests::compact_profile_has_two_causal_subjects_and_an_independent_observer_identity",
                            "decomp_prefix::tests::compact_budget_is_shared_and_never_exceeds_sixty_seconds",
                            "decomp_prefix::tests::compact_worker_wait_consumes_the_same_deadline_without_starting_a_child",
                            "decomp_prefix::tests::repeated_semantic_matches_cannot_extend_a_prefix_past_a_missing_poll",
                            "decomp_prefix::tests::failed_subject_keeps_complete_boundaries_before_an_unterminated_tail",
                            "decomp_prefix::tests::native_only_validation_refuses_truncated_tampered_and_cross_profile_captures",
                        ],
                    )],
                    filtered: true,
                    export_binary: false,
                },
                CargoCommand {
                    id: "controller-public-tests".into(),
                    argv: strings(&["cargo", "test", "-p", "pipeline", "--locked", "--offline", "--test", "prefix"]),
                    suites: vec![suite(
                        "crates/pipeline/tests/prefix.rs",
                        &[
                            "missing_movie_cannot_accept_or_start_prefix_build",
                            "compact_error_retains_profile_budget_and_unproved_checks",
                        ],
                    )],
                    filtered: false,
                    export_binary: true,
                },
            ],
            Self::PauseEquipment => vec![CargoCommand {
                id: "pause-equipment-rust-tests".into(),
                argv: strings(&[
                    "cargo",
                    "test",
                    "-p",
                    "port",
                    "--locked",
                    "--offline",
                    "--test",
                    "pause_equipment_render",
                ]),
                suites: vec![suite(
                    "crates/port/tests/pause_equipment_render.rs",
                    &[
                        "menu_wrap_and_extension_bits_preserve_unsigned_coordinates_and_tile_mask",
                        "menu_unaligned_cursor_preserves_the_extension_alias_write_order",
                        "selector_uses_low_category_byte_even_when_item_row_is_nonzero",
                        "sprite_timer_signed_boundary_and_terminator_reset_match_sequential_frames",
                        "empty_selector_and_empty_palette_need_no_rom_and_preserve_state",
                        "reserve_rounding_flickers_only_below_the_four_step_cutoff",
                        "reserve_full_tanks_end_cap_and_palette_wrap_use_exact_outputs",
                        "wireframe_writes_only_seventeen_eight_word_rows_and_exact_seven_byte_queue",
                        "failed_preflight_leaves_full_wram_unchanged_even_after_late_selected_assets",
                        "dispatch_requires_a_known_routine_and_keeps_packed_sprite_words_intact",
                    ],
                )],
                filtered: false,
                export_binary: false,
            }],
        }
    }
}

fn relative(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 1024
        && !name.contains(['\\', '\0'])
        && name.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
        && Path::new(name).components().all(|part| matches!(part, Component::Normal(_)))
        && !name.split('/').any(|part| matches!(part, ".git" | ".scratch" | ".codex"))
}

fn tree_files(root: &Path) -> Result<BTreeSet<String>> {
    bounded::safe(root)?;
    let mut files = BTreeSet::new();
    fn walk(root: &Path, path: &Path, files: &mut BTreeSet<String>, depth: usize) -> Result<()> {
        ensure!(depth <= 32, "Cargo input tree depth exceeded");
        for path in bounded::entries(path)? {
            bounded::check()?;
            let name = path.strip_prefix(root)?.to_str().context("non-UTF8 Cargo dependency path")?.to_owned();
            ensure!(relative(&name), "Cargo dependency path is protected or aliased");
            let metadata = path.symlink_metadata()?;
            if metadata.is_dir() {
                walk(root, &path, files, depth + 1)?;
            } else {
                ensure!(metadata.is_file(), "Cargo dependency is not a regular file");
                ensure!(files.len() < MAX_FILES, "Cargo dependency file inventory exceeded");
                files.insert(name);
            }
        }
        Ok(())
    }
    walk(root, root, &mut files, 0)?;
    Ok(files)
}

fn check_closure(closure: &CargoClosure, cache: bool) -> Result<()> {
    ensure!(closure.files.len() <= MAX_FILES, "Cargo dependency file inventory exceeded");
    ensure!(
        tree_files(&closure.root)? == closure.files.keys().cloned().collect(),
        "Cargo dependency closure has missing or undeclared files"
    );
    let mut bytes = 0u64;
    for (name, hash) in &closure.files {
        bounded::check()?;
        ensure!(relative(name) && digest_valid(hash), "invalid Cargo dependency seal");
        if cache {
            ensure!(
                !name.split('/').any(|part| part.starts_with("pipeline-")
                    || part.starts_with("port-")
                    || part.starts_with("libport")
                    || part.starts_with("sm_pipeline")
                    || part == "sm-pipeline"
                    || part.starts_with("game_host-")
                    || part.starts_with("room_scroll-")
                    || part.starts_with("samus_projectile_frame-")
                    || part.starts_with("pause_equipment_render-")
                    || part.starts_with("prefix-")),
                "Cargo dependency cache contains a candidate package or test artifact"
            );
        }
        let path = closure.root.join(name);
        bytes = bytes.checked_add(path.symlink_metadata()?.len()).context("Cargo dependency extent overflow")?;
        ensure!(bytes <= MAX_DEPENDENCY_BYTES, "Cargo dependency closure byte budget exceeded");
        ensure!(bounded::digest(&path)? == *hash, "Cargo dependency changed: {name}");
    }
    Ok(())
}

fn test_sources(checkout: &Path, contract: &CargoContract) -> Result<BTreeMap<String, String>> {
    let mut test_sources = BTreeMap::new();
    for command in contract.kind.commands() {
        for suite in command.suites {
            ensure!(contract.source_files.contains_key(&suite.source), "original Cargo test source is absent");
            let source = bounded::text(checkout.join(&suite.source))?;
            let tests = if suite.source == "crates/pipeline/src/decomp_prefix.rs" {
                let (_, tests) =
                    source.split_once("#[cfg(test)]").context("original compact unit-test module is absent")?;
                tests
            } else {
                &source
            };
            for name in &suite.names {
                let leaf = name.rsplit("::").next().context("empty Cargo control identity")?;
                ensure!(tests.contains(&format!("fn {leaf}(")), "original Cargo control is missing: {name}");
            }
            test_sources.insert(suite.source, sha256(tests));
        }
    }
    Ok(test_sources)
}

fn check_source(checkout: &Path, contract: &CargoContract, baseline: bool) -> Result<BTreeMap<String, String>> {
    bounded::safe(checkout)?;
    let mut hashes = BTreeMap::new();
    let mut bytes = 0u64;
    for (name, expected) in &contract.source_files {
        bounded::check()?;
        ensure!(
            relative(name) && !name.split('/').any(|part| part == ".cargo") && digest_valid(expected),
            "invalid Cargo source identity or configuration override"
        );
        let path = checkout.join(name);
        bounded::safe(&path)?;
        let metadata = path.symlink_metadata()?;
        ensure!(metadata.is_file(), "Cargo source is not a regular file");
        bytes = bytes.checked_add(metadata.len()).context("Cargo source byte extent overflow")?;
        ensure!(bytes <= MAX_SOURCE_BYTES, "Cargo source closure exceeds hard byte bound");
        let actual = bounded::digest(&path)?;
        if baseline || !contract.approved_files.contains(name) || name == "Cargo.lock" || name.ends_with("Cargo.toml") {
            ensure!(actual == *expected, "frozen Cargo test, manifest or dependency changed: {name}");
        }
        hashes.insert(name.clone(), actual);
    }
    Ok(hashes)
}

pub(crate) fn prepare(checkout: &Path, contract: &CargoContract) -> Result<PreparedCargo> {
    bounded::limited(|| {
        ensure!(contract.schema == "translation-cargo-contract-v1", "unsupported Cargo contract");
        ensure!(
            !contract.source_files.is_empty() && contract.source_files.len() <= MAX_FILES,
            "Cargo source closure missing or excessive"
        );
        ensure!(
            contract.source_files.contains_key("Cargo.toml") && contract.source_files.contains_key("Cargo.lock"),
            "Cargo workspace manifest or lockfile is absent"
        );
        ensure!(
            (1..=60).contains(&contract.limits.timeout_seconds)
                && (4096..=1024 * 1024 * 1024).contains(&contract.limits.writable_bytes)
                && contract.limits.writable_bytes.is_multiple_of(4096)
                && (1..=1024 * 1024).contains(&contract.limits.stdout_bytes)
                && (1..=1024 * 1024).contains(&contract.limits.stderr_bytes)
                && (contract.limits.stdout_bytes + contract.limits.stderr_bytes) as u64
                    <= contract.limits.writable_bytes,
            "Cargo original whole-command time/output budget is missing or exceeds 60 seconds"
        );
        let mut approved = BTreeSet::new();
        for name in &contract.approved_files {
            ensure!(
                contract.source_files.contains_key(name) && relative(name) && approved.insert(name),
                "Cargo original ownership path is missing, duplicated or aliased"
            );
        }
        check_source(checkout, contract, true)?;
        check_closure(&contract.vendor, false)?;
        if let Some(cache) = &contract.dependency_cache {
            check_closure(cache, true)?;
            let projects = project_packages(checkout, &contract.source_files)?;
            ensure!(
                cache.files.keys().all(|name| external_cache_path(name, &projects)),
                "Cargo dependency cache contains project artifacts or an unsupported target layout"
            );
        }
        let test_sources = test_sources(checkout, contract)?;
        let dependencies_sha256 = sealed(&(
            "original-cargo-dependencies-v1",
            &contract.source_files,
            &contract.vendor,
            &contract.dependency_cache,
        ))?;
        let verification_sha256 =
            sealed(&("original-cargo-controls-v1", contract.kind.commands(), &test_sources, &contract.limits))?;
        Ok(PreparedCargo { contract: contract.clone(), dependencies_sha256, verification_sha256, test_sources })
    })
}

pub(crate) fn run(
    prepared: &PreparedCargo,
    checkout: &Path,
    output: &Path,
    runtime: &StrictRuntime,
    context: &OperationContext,
) -> Result<CargoReceipt> {
    let entered = bounded::now_unix_ms()?;
    bounded::safe(output)?;
    fs::create_dir_all(output)?;
    let _lock = gate_lock(output)?;
    let receipt_path = output.join("cargo-receipt.toml");
    let verdict_path = output.join("cargo-verdict.toml");
    if receipt_path.is_file() || verdict_path.is_file() {
        return bounded::reconcile(4 * bounded::DEFAULT_BYTES, || {
            if receipt_path.is_file() {
                let receipt: CargoReceipt = family::object(&receipt_path)?;
                ensure!(
                    receipt.context == *context && receipt.runtime_sha256 == sealed(runtime)?,
                    "Cargo retry changed operation or installed runtime"
                );
                authenticate(prepared, checkout, &receipt)?;
                return Ok(receipt);
            }
            let verdict: CargoVerdict = family::object(&verdict_path)?;
            ensure!(
                verdict.context == *context && verdict.runtime == *runtime,
                "Cargo finalization recovery changed identity"
            );
            finish(prepared, checkout, &verdict_path, &verdict)
        });
    }
    ensure!(
        (1..=64).contains(&runtime.workers)
            && (1..=3600).contains(&runtime.gate_timeout_seconds)
            && (128 * 1024 * 1024..=8 * 1024 * 1024 * 1024).contains(&runtime.max_output_bytes),
        "Cargo installed aggregate budget is missing or excessive"
    );
    let intent_path = output.join("cargo-intent.toml");
    let saved: Option<CargoIntent> = if intent_path.is_file() { Some(family::object(&intent_path)?) } else { None };
    let resumed = saved.is_some();
    let local = entered
        .checked_add(runtime.gate_timeout_seconds.checked_mul(1000).context("Cargo deadline overflow")?)
        .context("Cargo deadline overflow")?;
    let deadline = saved
        .as_ref()
        .map_or(runtime.deadline_unix_ms.map_or(local, |outer| outer.min(local)), |intent| intent.deadline_unix_ms);
    let setup = || -> Result<_> {
        let candidate = validate_prepared(prepared, checkout)?;
        bounded::safe(&runtime.operation_root)?;
        bounded::safe(&runtime.pipeline_root)?;
        let prepared_sha256 = sealed(prepared)?;
        let candidate_sha256 = sealed(&candidate)?;
        let runtime_sha256 = sealed(runtime)?;
        let intent = saved.unwrap_or(CargoIntent {
            context: context.clone(),
            prepared_sha256: prepared_sha256.clone(),
            candidate_sha256: candidate_sha256.clone(),
            runtime_sha256: runtime_sha256.clone(),
            deadline_unix_ms: deadline,
        });
        ensure!(
            intent.context == *context
                && intent.prepared_sha256 == prepared_sha256
                && intent.candidate_sha256 == candidate_sha256
                && intent.runtime_sha256 == runtime_sha256,
            "Cargo attempt changed source, contract or runtime"
        );
        let mut budget = ByteBudget::new(runtime.max_output_bytes);
        budget.reserve(MAX_RECORD_BYTES * (prepared.contract.kind.commands().len() as u64 * 3 + 3))?;
        let source_bytes = candidate.keys().try_fold(0u64, |total, name| {
            total
                .checked_add(checkout.join(name).symlink_metadata()?.len())
                .context("Cargo source reservation overflow")
        })?;
        budget.reserve(source_bytes.checked_mul(2).context("Cargo snapshot reservation overflow")?)?;
        let mut records = vec![publish_record(&intent_path, &intent)?];
        let mut snapshots = BTreeMap::new();
        for name in candidate.keys() {
            bounded::check()?;
            let bytes = bounded::read_limit(&checkout.join(name), MAX_SOURCE_BYTES)?;
            let destination = output.join("source").join(name);
            publish_bytes(&destination, &bytes, false)?;
            snapshots.insert(name.clone(), retention::seal_file(&destination, MAX_SOURCE_BYTES)?);
        }
        let verdict = CargoVerdict {
            schema: "translation-cargo-verdict-v1".into(),
            evidence_kind: "original-rust-controls".into(),
            kind: prepared.contract.kind,
            passed: false,
            test_count: 0,
            commands: prepared.contract.kind.commands(),
            prepared_sha256,
            candidate_sha256,
            context: context.clone(),
            runtime: runtime.clone(),
            reserved_bytes: 0,
            diagnostic: String::new(),
            source_snapshots: snapshots,
            supporting_records: std::mem::take(&mut records),
            operations: vec![],
            runtime_binary: None,
            limitations: limitations(),
        };
        Ok((candidate, budget, verdict))
    };
    let (candidate, mut budget, mut verdict) = if resumed && entered >= deadline {
        bounded::reconcile(4 * bounded::DEFAULT_BYTES, setup)?
    } else {
        bounded::until(deadline, 4 * bounded::DEFAULT_BYTES, setup)?
    };
    let produce = || -> Result<()> {
        if !resumed {
            check_closure(&prepared.contract.vendor, false)?;
            if let Some(cache) = &prepared.contract.dependency_cache {
                check_closure(cache, true)?;
            }
        }
        for (index, command) in verdict.commands.clone().into_iter().enumerate() {
            let request = ProducerRequest {
                schema: "translation-cargo-producer-v1".into(),
                kind: prepared.contract.kind,
                command_id: command.id.clone(),
                dependency_cache: prepared.contract.dependency_cache.is_some(),
            };
            let request_path = output.join(format!("{}-request.json", command.id));
            let request_bytes = serde_json::to_vec(&request)?;
            publish_bytes(&request_path, &request_bytes, false)?;
            let request_ref = retention::seal_file(&request_path, MAX_RECORD_BYTES)?;
            verdict.supporting_records.push(request_ref.clone());
            let invocation_path = output.join(format!("{}-invocation.toml", command.id));
            let existing = invocation_path.is_file();
            let mut operation_context = context.clone();
            operation_context.operation_key = sealed(&(&context.operation_key, output, &command.id))?;
            let invocation = if existing {
                let intent: InvocationIntent = family::object(&invocation_path)?;
                ensure!(
                    intent.context == operation_context && intent.request == request_ref,
                    "Cargo invocation retry changed identity"
                );
                budget.reserve(intent.reserved_bytes)?;
                intent
            } else {
                let started = if index == 0 { entered } else { bounded::now_unix_ms()? };
                let command_deadline = deadline.min(
                    started
                        .checked_add(prepared.contract.limits.timeout_seconds * 1000)
                        .context("Cargo command deadline overflow")?,
                );
                ensure!(
                    bounded::now_unix_ms()? < command_deadline,
                    "Cargo original whole-command 60-second budget exhausted before launch"
                );
                let program = producer_program(prepared, output, runtime, &request_path, &command)?;
                let mut limits = run_limits(&prepared.contract.limits)?;
                let reservation = operation::reserve_bytes_until(&program, &limits, 0, command_deadline)?
                    .checked_add(MAX_RECORD_BYTES)
                    .context("Cargo operation reservation overflow")?;
                budget.reserve(reservation)?;
                let remaining = command_deadline.saturating_sub(bounded::now_unix_ms()?);
                ensure!(remaining > 0, "Cargo whole-command budget exhausted during closure preparation");
                limits.timeout = Duration::from_millis(remaining).min(limits.timeout);
                let intent = InvocationIntent {
                    context: operation_context.clone(),
                    program,
                    limits,
                    deadline_unix_ms: command_deadline,
                    reserved_bytes: reservation,
                    request: request_ref.clone(),
                };
                publish_record(&invocation_path, &intent)?;
                intent
            };
            verdict.supporting_records.push(retention::seal_file(&invocation_path, MAX_RECORD_BYTES)?);
            validate_invocation(prepared, output, runtime, &command, &invocation)?;
            let directory = operation::directory(&runtime.operation_root, &operation_context)?;
            if existing {
                ensure!(
                    directory.join("intent.json").is_file(),
                    "Cargo invocation interrupted before durable launch; explicit reviewed retry required"
                );
            }
            let actual = if directory.join("result.json").is_file() {
                operation::authenticate_request(
                    &runtime.operation_root,
                    &operation_context,
                    &invocation.program,
                    &invocation.limits,
                    &[],
                )?
            } else if bounded::now_unix_ms()? >= invocation.deadline_unix_ms {
                ensure!(
                    directory.join("intent.json").is_file(),
                    "Cargo whole-command budget exhausted before durable process start"
                );
                operation::stop(&runtime.operation_root, &operation_context)?;
                operation::authenticate_request(
                    &runtime.operation_root,
                    &operation_context,
                    &invocation.program,
                    &invocation.limits,
                    &[],
                )?
            } else {
                let _lease =
                    acquire_lease_until(&runtime.pipeline_root, runtime.workers, invocation.deadline_unix_ms)?.0;
                operation::run_until(
                    &runtime.operation_root,
                    &operation_context,
                    &invocation.program,
                    &invocation.limits,
                    &[],
                    invocation.deadline_unix_ms,
                )?
            };
            verdict.operations.push(CargoInvocation {
                command_id: command.id.clone(),
                context: operation_context.clone(),
                request: request_ref,
                program: invocation.program,
                limits: invocation.limits,
                deadline_unix_ms: invocation.deadline_unix_ms,
                operation_root: runtime.operation_root.clone(),
                result_sha256: actual.result_sha256.clone(),
                references: operation::references(&runtime.operation_root, &operation_context)?,
            });
            ensure!(
                actual.passed && actual.status_raw == Some(0),
                "confined original Cargo {} failed: {}",
                command.id,
                actual.diagnostic
            );
            passed_controls(&actual.stdout, &command)?;
            if command.export_binary {
                let binary = actual
                    .outputs
                    .get(Path::new("sm-pipeline"))
                    .context("Cargo omitted the candidate-built compact runtime")?;
                ensure!(
                    binary.starts_with(b"\x7fELF") && binary.len() as u64 <= MAX_SOURCE_BYTES,
                    "Cargo returned an invalid compact runtime"
                );
                let path = output.join("sm-pipeline");
                publish_bytes(&path, binary, true)?;
                verdict.runtime_binary = Some(retention::seal_file(&path, MAX_SOURCE_BYTES)?);
            }
        }
        ensure!(
            check_source(checkout, &prepared.contract, false)? == candidate,
            "Cargo candidate changed during verification"
        );
        verdict.passed = true;
        verdict.test_count =
            verdict.commands.iter().flat_map(|command| &command.suites).map(|suite| suite.names.len()).sum();
        Ok(())
    };
    let result = if resumed && bounded::now_unix_ms()? >= deadline {
        bounded::reconcile(4 * bounded::DEFAULT_BYTES, produce)
    } else {
        bounded::until(deadline, 4 * bounded::DEFAULT_BYTES, produce)
    };
    if let Err(error) = result {
        if error.downcast_ref::<translation_execution::ExecutionPending>().is_some() {
            return Err(error);
        }
        verdict.passed = false;
        verdict.test_count = 0;
        verdict.diagnostic = diagnostic(&format!("{error:#}"));
    }
    verdict.reserved_bytes = budget.used();
    bounded::reconcile(4 * bounded::DEFAULT_BYTES, || {
        publish_record(&verdict_path, &verdict)?;
        finish(prepared, checkout, &verdict_path, &verdict)
    })
}

fn diagnostic(value: &str) -> String {
    let mut end = value.len().min(4096);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}

fn limitations() -> Vec<String> {
    strings(&[
        "These frozen original Cargo controls establish Rust infrastructure acceptance only.",
        "Actual-C Fixture pairs, the compact causal diagnostic, Live replay and original-ROM parity require their distinct contracts.",
        "Only the complete prescribed control identities with zero failures or ignored controls can pass.",
    ])
}

fn gate_lock(output: &Path) -> Result<fs::File> {
    use std::os::unix::{fs::OpenOptionsExt, io::AsRawFd};
    let path = output.join("cargo.lock");
    bounded::safe(&path)?;
    let file =
        fs::OpenOptions::new().append(true).create(true).mode(0o600).custom_flags(libc::O_NOFOLLOW).open(path)?;
    ensure!(file.metadata()?.is_file(), "Cargo gate lock is not regular");
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err(translation_execution::ExecutionPending {
            reason: "Cargo gate has a surviving controller; retain ownership".into(),
        }
        .into());
    }
    Ok(file)
}

fn validate_prepared(prepared: &PreparedCargo, checkout: &Path) -> Result<BTreeMap<String, String>> {
    ensure!(
        prepared.dependencies_sha256
            == sealed(&(
                "original-cargo-dependencies-v1",
                &prepared.contract.source_files,
                &prepared.contract.vendor,
                &prepared.contract.dependency_cache
            ))?
            && prepared.verification_sha256
                == sealed(&(
                    "original-cargo-controls-v1",
                    prepared.contract.kind.commands(),
                    &prepared.test_sources,
                    &prepared.contract.limits
                ))?,
        "Cargo prepared dependency or verification identity changed"
    );
    let hashes = check_source(checkout, &prepared.contract, false)?;
    ensure!(
        test_sources(checkout, &prepared.contract)? == prepared.test_sources,
        "Cargo frozen original test source changed"
    );
    Ok(hashes)
}

fn publish_bytes(path: &Path, bytes: &[u8], executable: bool) -> Result<()> {
    bounded::safe(path)?;
    fs::create_dir_all(path.parent().context("Cargo retained path has no parent")?)?;
    inputs::immutable_bytes(path, bytes)?;
    let mode = if executable { 0o500 } else { 0o400 };
    if path.symlink_metadata()?.permissions().mode() & 0o777 != mode {
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }
    Ok(())
}

fn publish_record(path: &Path, value: &impl Serialize) -> Result<Reference> {
    let bytes = toml::to_string_pretty(value)?.into_bytes();
    ensure!(bytes.len() as u64 <= MAX_RECORD_BYTES, "Cargo evidence metadata extent exceeded");
    publish_bytes(path, &bytes, false)?;
    retention::seal_file(path, MAX_RECORD_BYTES)
}

fn run_limits(limits: &CargoLimits) -> Result<RunLimits> {
    ensure!(
        (1..=60).contains(&limits.timeout_seconds)
            && limits.writable_bytes <= 1024 * 1024 * 1024
            && limits.writable_bytes >= 4096
            && limits.stdout_bytes > 0
            && limits.stdout_bytes <= 1024 * 1024
            && limits.stderr_bytes > 0
            && limits.stderr_bytes <= 1024 * 1024,
        "Cargo limits exceed original whole-command bound"
    );
    Ok(RunLimits {
        timeout: Duration::from_secs(limits.timeout_seconds),
        writable_bytes: limits.writable_bytes,
        stdout_bytes: limits.stdout_bytes,
        stderr_bytes: limits.stderr_bytes,
        stdin_bytes: 0,
    })
}

fn producer_program(
    prepared: &PreparedCargo,
    output: &Path,
    runtime: &StrictRuntime,
    request: &Path,
    command: &CargoCommand,
) -> Result<Program> {
    let rustc = fs::canonicalize(&runtime.rustc)?;
    let cargo = fs::canonicalize(&runtime.cargo)?;
    ensure!(
        rustc == runtime.rustc
            && cargo == runtime.cargo
            && rustc.file_name().is_some_and(|name| name == "rustc")
            && cargo.file_name().is_some_and(|name| name == "cargo")
            && rustc.parent() == cargo.parent(),
        "Cargo requires explicit matching installed compiler/toolchain files"
    );
    let mut binds = builds::strict_compiler_runtime(&rustc)?;
    binds.push(ReadOnlyBind { source: rustc, destination: "/toolchain/bin/rustc".into() });
    binds.push(ReadOnlyBind { source: cargo, destination: "/toolchain/bin/cargo".into() });
    for (source, destination) in
        [("/usr/bin/ar", "/usr/bin/ar"), ("/usr/bin/strip", "/usr/bin/strip"), ("/bin/sh", "/bin/sh")]
    {
        binds.push(ReadOnlyBind { source: fs::canonicalize(source)?, destination: destination.into() });
    }
    let mut inputs = vec![
        ReadOnlyBind { source: output.join("source"), destination: "/inputs/source".into() },
        ReadOnlyBind { source: prepared.contract.vendor.root.clone(), destination: "/inputs/vendor".into() },
        ReadOnlyBind { source: request.into(), destination: "/inputs/cargo-request.json".into() },
    ];
    if let Some(cache) = &prepared.contract.dependency_cache {
        inputs.push(ReadOnlyBind { source: cache.root.clone(), destination: "/inputs/cache".into() });
    }
    Ok(Program {
        executable: runtime.producer.clone(),
        arguments: strings(&["translation-cargo-producer", "--request", "/inputs/cargo-request.json"]),
        helper: runtime.helper.clone(),
        runtime: binds,
        inputs,
        outputs: if command.export_binary { vec!["sm-pipeline".into()] } else { vec![] },
    })
}

fn validate_invocation(
    prepared: &PreparedCargo,
    output: &Path,
    runtime: &StrictRuntime,
    command: &CargoCommand,
    invocation: &InvocationIntent,
) -> Result<()> {
    let expected = producer_program(prepared, output, runtime, &invocation.request.path, command)?;
    let declared = run_limits(&prepared.contract.limits)?;
    ensure!(
        invocation.program == expected
            && !invocation.limits.timeout.is_zero()
            && invocation.limits.timeout <= declared.timeout
            && invocation.limits.writable_bytes == declared.writable_bytes
            && invocation.limits.stdout_bytes == declared.stdout_bytes
            && invocation.limits.stderr_bytes == declared.stderr_bytes
            && invocation.limits.stdin_bytes == 0,
        "Cargo invocation differs from the prescribed confined command"
    );
    Ok(())
}

fn passed_controls(bytes: &[u8], command: &CargoCommand) -> Result<usize> {
    let text = std::str::from_utf8(bytes)?;
    let mut lines = text.lines().filter(|line| !line.is_empty()).peekable();
    let mut remaining = command.suites.iter().map(|suite| suite.names.clone()).collect::<Vec<_>>();
    let mut count = 0usize;
    while let Some(header) = lines.next() {
        let number = header
            .strip_prefix("running ")
            .and_then(|line| line.strip_suffix(" tests"))
            .context("original Cargo control header is absent or substituted")?
            .parse::<usize>()?;
        ensure!(number > 0, "original Cargo command ran zero tests");
        let mut names = Vec::new();
        while lines.peek().is_some_and(|line| line.starts_with("test ") && !line.starts_with("test result:")) {
            let line = lines.next().context("Cargo test line disappeared")?;
            names.push(
                line.strip_prefix("test ")
                    .and_then(|line| line.strip_suffix(" ... ok"))
                    .context("original Cargo control failed, was ignored or substituted")?
                    .to_owned(),
            );
        }
        names.sort();
        ensure!(number == names.len(), "Cargo executed count differs from named controls");
        let matching = remaining
            .iter()
            .position(|expected| expected == &names)
            .context("Cargo omitted or substituted an original control identity")?;
        remaining.remove(matching);
        let summary = lines.next().context("original Cargo test summary is absent")?;
        let prefix = format!("test result: ok. {number} passed; 0 failed; 0 ignored; 0 measured; ");
        let (filtered, elapsed) = summary
            .strip_prefix(&prefix)
            .and_then(|summary| summary.split_once(" filtered out; finished in "))
            .context("original Cargo summary is incomplete or accepting ignored controls")?;
        let filtered = filtered.parse::<usize>()?;
        ensure!(command.filtered || filtered == 0, "original Cargo controls were filtered away");
        let seconds = elapsed.strip_suffix('s').context("Cargo duration is absent")?.parse::<f64>()?;
        ensure!(seconds.is_finite() && seconds >= 0.0, "invalid Cargo test duration");
        count = count.checked_add(number).context("Cargo test count overflow")?;
    }
    ensure!(remaining.is_empty() && count > 0, "original Cargo control suites are missing");
    Ok(count)
}

fn verdict_references(verdict: &CargoVerdict, path: &Path) -> Result<Vec<Reference>> {
    let mut files = BTreeMap::new();
    for reference in verdict
        .source_snapshots
        .values()
        .chain(&verdict.supporting_records)
        .chain(verdict.operations.iter().flat_map(|operation| &operation.references))
        .chain(verdict.runtime_binary.iter())
    {
        if let Some(prior) = files.insert(reference.path.clone(), reference.clone()) {
            ensure!(prior == *reference, "Cargo evidence reference has conflicting seals");
        }
    }
    files.insert(path.into(), retention::seal_file(path, MAX_RECORD_BYTES)?);
    Ok(files.into_values().collect())
}

fn finish(prepared: &PreparedCargo, checkout: &Path, path: &Path, verdict: &CargoVerdict) -> Result<CargoReceipt> {
    let evidence_bytes = verdict_references(verdict, path)?.iter().try_fold(0u64, |total, reference| {
        total.checked_add(reference.bytes).context("Cargo evidence extent overflow")
    })?;
    let receipt = CargoReceipt {
        prepared_sha256: verdict.prepared_sha256.clone(),
        candidate_sha256: verdict.candidate_sha256.clone(),
        context: verdict.context.clone(),
        runtime_sha256: sealed(&verdict.runtime)?,
        max_output_bytes: verdict.runtime.max_output_bytes,
        reserved_bytes: verdict.reserved_bytes,
        verdict: Asset { file: path.into(), sha256: bounded::digest(path)? },
        passed: verdict.passed,
        diagnostic: verdict.diagnostic.clone(),
        evidence_bytes,
        runtime_binary: verdict.runtime_binary.clone(),
    };
    authenticate(prepared, checkout, &receipt)?;
    publish_record(&path.parent().context("Cargo verdict parent missing")?.join("cargo-receipt.toml"), &receipt)?;
    Ok(receipt)
}

pub(crate) fn authenticate(prepared: &PreparedCargo, checkout: &Path, receipt: &CargoReceipt) -> Result<CargoVerdict> {
    bounded::limited(|| {
        let candidate = validate_prepared(prepared, checkout)?;
        ensure!(
            receipt.prepared_sha256 == sealed(prepared)? && receipt.candidate_sha256 == sealed(&candidate)?,
            "Cargo receipt belongs to another preparation or source"
        );
        ensure!(bounded::digest(&receipt.verdict.file)? == receipt.verdict.sha256, "retained Cargo verdict changed");
        let verdict: CargoVerdict = family::object(&receipt.verdict.file)?;
        ensure!(
            verdict.schema == "translation-cargo-verdict-v1"
                && verdict.evidence_kind == "original-rust-controls"
                && verdict.kind == prepared.contract.kind
                && verdict.passed == receipt.passed
                && verdict.prepared_sha256 == receipt.prepared_sha256
                && verdict.candidate_sha256 == receipt.candidate_sha256
                && verdict.context == receipt.context
                && sealed(&verdict.runtime)? == receipt.runtime_sha256
                && verdict.runtime.max_output_bytes == receipt.max_output_bytes
                && verdict.reserved_bytes == receipt.reserved_bytes
                && receipt.evidence_bytes <= receipt.reserved_bytes
                && receipt.reserved_bytes <= receipt.max_output_bytes
                && verdict.diagnostic == receipt.diagnostic
                && verdict.diagnostic.len() <= 4096
                && verdict.runtime_binary == receipt.runtime_binary
                && verdict.commands == prepared.contract.kind.commands()
                && verdict.limitations == limitations(),
            "Cargo receipt acceptance, limits or original requirements changed"
        );
        let output = receipt.verdict.file.parent().context("Cargo verdict parent missing")?;
        ensure!(verdict.source_snapshots.len() == candidate.len(), "Cargo retained source closure differs");
        for (name, hash) in &candidate {
            let snapshot = verdict.source_snapshots.get(name).context("Cargo source snapshot missing")?;
            ensure!(
                snapshot.path == output.join("source").join(name) && snapshot.sha256 == *hash,
                "Cargo source snapshot identity differs"
            );
            retention::authenticate_reference(snapshot, MAX_SOURCE_BYTES)?;
        }
        for reference in &verdict.supporting_records {
            ensure!(reference.path.parent() == Some(output), "Cargo supporting record escaped its gate");
            retention::authenticate_reference(reference, MAX_RECORD_BYTES)?;
        }
        let intent_path = output.join("cargo-intent.toml");
        let intent: CargoIntent = family::object(&intent_path)?;
        ensure!(
            intent.context == verdict.context
                && intent.prepared_sha256 == verdict.prepared_sha256
                && intent.candidate_sha256 == verdict.candidate_sha256
                && intent.runtime_sha256 == receipt.runtime_sha256
                && verdict.supporting_records.contains(&retention::seal_file(&intent_path, MAX_RECORD_BYTES)?),
            "Cargo durable gate intent changed"
        );
        ensure!(verdict.operations.len() <= verdict.commands.len(), "Cargo duplicated an original operation");
        let mut counted = 0usize;
        let mut exported = false;
        for (invocation, command) in verdict.operations.iter().zip(&verdict.commands) {
            let mut expected_context = verdict.context.clone();
            expected_context.operation_key = sealed(&(&verdict.context.operation_key, output, &command.id))?;
            ensure!(
                invocation.command_id == command.id
                    && invocation.context == expected_context
                    && invocation.operation_root == verdict.runtime.operation_root,
                "Cargo operation escaped original order or lineage"
            );
            let request = ProducerRequest {
                schema: "translation-cargo-producer-v1".into(),
                kind: verdict.kind,
                command_id: command.id.clone(),
                dependency_cache: prepared.contract.dependency_cache.is_some(),
            };
            let request_bytes = serde_json::to_vec(&request)?;
            ensure!(
                invocation.request.path == output.join(format!("{}-request.json", command.id))
                    && invocation.request.sha256 == sha256(&request_bytes)
                    && invocation.request.bytes == request_bytes.len() as u64,
                "Cargo producer request changed"
            );
            retention::authenticate_reference(&invocation.request, MAX_RECORD_BYTES)?;
            let invocation_path = output.join(format!("{}-invocation.toml", command.id));
            let saved: InvocationIntent = family::object(&invocation_path)?;
            ensure!(
                saved.context == invocation.context
                    && saved.program == invocation.program
                    && saved.limits == invocation.limits
                    && saved.deadline_unix_ms == invocation.deadline_unix_ms
                    && saved.deadline_unix_ms <= intent.deadline_unix_ms
                    && saved.request == invocation.request
                    && verdict.supporting_records.contains(&retention::seal_file(&invocation_path, MAX_RECORD_BYTES)?),
                "Cargo invocation receipt differs from durable intent"
            );
            validate_invocation(prepared, output, &verdict.runtime, command, &saved)?;
            let actual = operation::authenticate_request(
                &invocation.operation_root,
                &invocation.context,
                &invocation.program,
                &invocation.limits,
                &[],
            )?;
            ensure!(
                actual.result_sha256 == invocation.result_sha256
                    && operation::references(&invocation.operation_root, &invocation.context)? == invocation.references,
                "Cargo operation evidence changed"
            );
            if verdict.passed {
                ensure!(actual.passed && actual.status_raw == Some(0), "Cargo accepted a failed original command");
                counted += passed_controls(&actual.stdout, command)?;
            }
            if command.export_binary && verdict.runtime_binary.is_some() {
                let reference = verdict.runtime_binary.as_ref().context("Cargo runtime disappeared")?;
                let bytes = actual
                    .outputs
                    .get(Path::new("sm-pipeline"))
                    .context("Cargo runtime was not produced by the original public controls")?;
                ensure!(
                    actual.passed
                        && reference.path == output.join("sm-pipeline")
                        && reference.sha256 == sha256(bytes)
                        && reference.bytes == bytes.len() as u64
                        && bytes.starts_with(b"\x7fELF"),
                    "Cargo runtime binary provenance changed"
                );
                retention::authenticate_reference(reference, MAX_SOURCE_BYTES)?;
                exported = true;
            }
        }
        ensure!(verdict.runtime_binary.is_none() || exported, "Cargo runtime binary has no original producing command");
        if verdict.passed {
            ensure!(
                verdict.operations.len() == verdict.commands.len()
                    && verdict.test_count == counted
                    && (verdict.kind != CargoKind::CompactController || verdict.runtime_binary.is_some())
                    && verdict.diagnostic.is_empty(),
                "Cargo acceptance omitted an original command or control"
            );
        } else {
            ensure!(
                verdict.test_count == 0 && !verdict.diagnostic.is_empty(),
                "Cargo stopped gate omitted its bounded diagnosis"
            );
        }
        let references = verdict_references(&verdict, &receipt.verdict.file)?;
        let actual_bytes = references.iter().try_fold(0u64, |total, reference| {
            total.checked_add(reference.bytes).context("Cargo evidence byte overflow")
        })?;
        ensure!(actual_bytes == receipt.evidence_bytes, "Cargo evidence extent differs from the sealed receipt");
        Ok(verdict)
    })
}

pub(crate) fn references(receipt: &CargoReceipt) -> Result<Vec<Reference>> {
    ensure!(bounded::digest(&receipt.verdict.file)? == receipt.verdict.sha256, "Cargo retained verdict changed");
    let verdict: CargoVerdict = family::object(&receipt.verdict.file)?;
    let mut references = verdict_references(&verdict, &receipt.verdict.file)?;
    references.push(retention::seal_file(
        &receipt.verdict.file.parent().context("Cargo verdict parent missing")?.join("cargo-receipt.toml"),
        MAX_RECORD_BYTES,
    )?);
    Ok(references)
}

/// Import the exact installed runtime under its own preparation deadline and,
/// optionally, build only frozen dependency libraries. No original test gate
/// is credited here. Reuse the same operation_root for the later 60s commands.
pub(crate) fn prepare_environment(
    prepared: &PreparedCargo,
    checkout: &Path,
    output: &Path,
    runtime: &StrictRuntime,
    context: &OperationContext,
    timeout_seconds: u64,
    build_dependencies: bool,
) -> Result<CargoEnvironmentReceipt> {
    ensure!(
        (1..=600).contains(&timeout_seconds),
        "Cargo environment preparation requires a reviewed bound of at most 600 seconds"
    );
    let entered = bounded::now_unix_ms()?;
    bounded::safe(output)?;
    fs::create_dir_all(output)?;
    let _lock = gate_lock(output)?;
    let record_path = output.join("cargo-environment.toml");
    let receipt_path = output.join("cargo-environment-receipt.toml");
    if receipt_path.is_file() {
        let receipt: CargoEnvironmentReceipt = family::object(&receipt_path)?;
        ensure!(
            receipt.context == *context && receipt.runtime_sha256 == sealed(runtime)?,
            "Cargo environment retry changed context or runtime"
        );
        authenticate_environment(prepared, checkout, &receipt)?;
        return Ok(receipt);
    }
    if record_path.is_file() {
        let record: EnvironmentRecord = family::object(&record_path)?;
        ensure!(
            record.context == *context && record.runtime == *runtime,
            "Cargo environment finalization changed identity"
        );
        return finish_environment(prepared, checkout, &record_path, &record);
    }
    let intent_path = output.join("cargo-environment-intent.toml");
    let saved: Option<EnvironmentRecord> =
        if intent_path.is_file() { Some(family::object(&intent_path)?) } else { None };
    let deadline = saved.as_ref().map_or(
        entered.checked_add(timeout_seconds * 1000).context("Cargo environment deadline overflow")?,
        |record| record.deadline_unix_ms,
    );
    let setup = || -> Result<EnvironmentRecord> {
        let candidate = validate_prepared(prepared, checkout)?;
        let expected_request =
            EnvironmentRequest { schema: "translation-cargo-environment-producer-v1".into(), build_dependencies };
        let request_path = output.join("cargo-request.json");
        let bytes = serde_json::to_vec(&expected_request)?;
        publish_bytes(&request_path, &bytes, false)?;
        let request = retention::seal_file(&request_path, MAX_RECORD_BYTES)?;
        if let Some(record) = saved {
            ensure!(
                record.prepared_sha256 == sealed(prepared)?
                    && record.candidate_sha256 == sealed(&candidate)?
                    && record.context == *context
                    && record.runtime == *runtime
                    && record.request == request,
                "Cargo environment retry changed frozen inputs"
            );
            return Ok(record);
        }
        check_closure(&prepared.contract.vendor, false)?;
        let mut snapshots = BTreeMap::new();
        let source_bytes = candidate.keys().try_fold(0u64, |total, name| {
            total
                .checked_add(checkout.join(name).symlink_metadata()?.len())
                .context("Cargo environment source extent overflow")
        })?;
        ensure!(
            source_bytes * 2 + MAX_RECORD_BYTES * 3 <= runtime.max_output_bytes,
            "Cargo environment source reservation exceeds aggregate budget"
        );
        for name in candidate.keys() {
            let bytes = bounded::read_limit(&checkout.join(name), MAX_SOURCE_BYTES)?;
            let path = output.join("source").join(name);
            publish_bytes(&path, &bytes, false)?;
            snapshots.insert(name.clone(), retention::seal_file(&path, MAX_SOURCE_BYTES)?);
        }
        let mut program =
            producer_program(prepared, output, runtime, &request_path, &prepared.contract.kind.commands()[0])?;
        program.arguments =
            strings(&["translation-cargo-environment-producer", "--request", "/inputs/cargo-request.json"]);
        program.outputs = if build_dependencies { vec!["dependencies.zip".into()] } else { vec![] };
        let limits = RunLimits {
            timeout: Duration::from_secs(timeout_seconds),
            writable_bytes: if build_dependencies { 512 * 1024 * 1024 } else { 64 * 1024 * 1024 },
            stdout_bytes: 65536,
            stderr_bytes: 65536,
            stdin_bytes: 0,
        };
        let reserved_bytes = operation::reserve_bytes_until(&program, &limits, 0, deadline)?
            .checked_add(
                source_bytes * 2 + MAX_RECORD_BYTES * 3 + if build_dependencies { MAX_DEPENDENCY_BYTES * 2 } else { 0 },
            )
            .context("Cargo environment reservation overflow")?;
        ensure!(
            reserved_bytes <= runtime.max_output_bytes,
            "Cargo environment preparation exceeds aggregate reserved bytes"
        );
        let record = EnvironmentRecord {
            schema: "translation-cargo-environment-v1".into(),
            prepared_sha256: sealed(prepared)?,
            candidate_sha256: sealed(&candidate)?,
            context: context.clone(),
            runtime: runtime.clone(),
            request,
            program,
            limits,
            deadline_unix_ms: deadline,
            reserved_bytes,
            passed: false,
            diagnostic: String::new(),
            cache: None,
            snapshots,
            operation_result_sha256: String::new(),
            references: vec![],
        };
        publish_record(&intent_path, &record)?;
        Ok(record)
    };
    let mut record = if entered >= deadline {
        bounded::reconcile(4 * bounded::DEFAULT_BYTES, setup)?
    } else {
        bounded::until(deadline, 4 * bounded::DEFAULT_BYTES, setup)?
    };
    let directory = operation::directory(&runtime.operation_root, context)?;
    let result = if directory.join("result.json").is_file() {
        operation::authenticate_request(&runtime.operation_root, context, &record.program, &record.limits, &[])?
    } else {
        ensure!(
            bounded::now_unix_ms()? < deadline,
            "Cargo environment prerequisite interrupted before durable launch; explicit reviewed retry required"
        );
        let _lease = acquire_lease_until(&runtime.pipeline_root, runtime.workers, deadline)?.0;
        operation::run_until(&runtime.operation_root, context, &record.program, &record.limits, &[], deadline)?
    };
    record.operation_result_sha256 = result.result_sha256;
    record.references = operation::references(&runtime.operation_root, context)?;
    record.references.push(retention::seal_file(&intent_path, MAX_RECORD_BYTES)?);
    record.references.push(record.request.clone());
    record.references.extend(record.snapshots.values().cloned());
    record.passed = result.passed && result.status_raw == Some(0);
    if !record.passed {
        record.diagnostic = diagnostic(&result.diagnostic);
    }
    if record.passed && build_dependencies {
        let project_names = project_packages(checkout, &prepared.contract.source_files)?;
        let unpacked = result
            .outputs
            .get(Path::new("dependencies.zip"))
            .context("Cargo dependency preparation omitted its sealed archive")
            .and_then(|bytes| unpack_cache(bytes, &output.join("dependencies"), &project_names));
        match unpacked {
            Ok(cache) => record.cache = Some(cache),
            Err(error) => {
                record.passed = false;
                record.diagnostic = diagnostic(&format!("{error:#}"));
            }
        }
    }
    bounded::reconcile(4 * bounded::DEFAULT_BYTES, || {
        publish_record(&record_path, &record)?;
        finish_environment(prepared, checkout, &record_path, &record)
    })
}

fn environment_record_references(record: &EnvironmentRecord, path: &Path) -> Result<Vec<Reference>> {
    let mut references = record
        .references
        .iter()
        .map(|reference| (reference.path.clone(), reference.clone()))
        .collect::<BTreeMap<_, _>>();
    if let Some(cache) = &record.cache {
        for (name, hash) in &cache.files {
            let file = retention::seal_file(&cache.root.join(name), MAX_SOURCE_BYTES)?;
            ensure!(file.sha256 == *hash, "Cargo dependency cache changed");
            references.insert(file.path.clone(), file);
        }
    }
    references.insert(path.into(), retention::seal_file(path, MAX_RECORD_BYTES)?);
    Ok(references.into_values().collect())
}

fn finish_environment(
    prepared: &PreparedCargo,
    checkout: &Path,
    path: &Path,
    record: &EnvironmentRecord,
) -> Result<CargoEnvironmentReceipt> {
    let evidence_bytes = environment_record_references(record, path)?.iter().try_fold(0u64, |total, reference| {
        total.checked_add(reference.bytes).context("Cargo environment evidence overflow")
    })?;
    let receipt = CargoEnvironmentReceipt {
        schema: "translation-cargo-environment-receipt-v1".into(),
        prepared_sha256: record.prepared_sha256.clone(),
        candidate_sha256: record.candidate_sha256.clone(),
        context: record.context.clone(),
        runtime_sha256: sealed(&record.runtime)?,
        passed: record.passed,
        diagnostic: record.diagnostic.clone(),
        cache: record.cache.clone(),
        record: Asset { file: path.into(), sha256: bounded::digest(path)? },
        evidence_bytes,
        reserved_bytes: record.reserved_bytes,
    };
    authenticate_environment(prepared, checkout, &receipt)?;
    publish_record(
        &path.parent().context("Cargo environment record parent missing")?.join("cargo-environment-receipt.toml"),
        &receipt,
    )?;
    Ok(receipt)
}

pub(crate) fn authenticate_environment(
    prepared: &PreparedCargo,
    checkout: &Path,
    receipt: &CargoEnvironmentReceipt,
) -> Result<()> {
    bounded::limited(|| {
        let candidate = validate_prepared(prepared, checkout)?;
        ensure!(
            receipt.schema == "translation-cargo-environment-receipt-v1"
                && receipt.prepared_sha256 == sealed(prepared)?
                && receipt.candidate_sha256 == sealed(&candidate)?
                && bounded::digest(&receipt.record.file)? == receipt.record.sha256,
            "Cargo environment receipt differs from source or preparation"
        );
        let record: EnvironmentRecord = family::object(&receipt.record.file)?;
        ensure!(
            record.schema == "translation-cargo-environment-v1"
                && record.prepared_sha256 == receipt.prepared_sha256
                && record.candidate_sha256 == receipt.candidate_sha256
                && record.context == receipt.context
                && sealed(&record.runtime)? == receipt.runtime_sha256
                && record.cache == receipt.cache
                && record.passed == receipt.passed
                && record.diagnostic == receipt.diagnostic
                && record.diagnostic.len() <= 4096
                && record.reserved_bytes == receipt.reserved_bytes
                && receipt.evidence_bytes <= receipt.reserved_bytes
                && receipt.reserved_bytes <= record.runtime.max_output_bytes,
            "Cargo environment outcome or limits changed"
        );
        let root = receipt.record.file.parent().context("Cargo environment record parent missing")?;
        let request: EnvironmentRequest =
            serde_json::from_slice(&bounded::read_limit(&record.request.path, MAX_RECORD_BYTES)?)?;
        ensure!(
            request.schema == "translation-cargo-environment-producer-v1"
                && record.request.path == root.join("cargo-request.json")
                && record.request.sha256 == sealed_json(&request)?,
            "Cargo environment producer request changed"
        );
        let mut program = producer_program(
            prepared,
            root,
            &record.runtime,
            &record.request.path,
            &prepared.contract.kind.commands()[0],
        )?;
        program.arguments =
            strings(&["translation-cargo-environment-producer", "--request", "/inputs/cargo-request.json"]);
        program.outputs = if request.build_dependencies { vec!["dependencies.zip".into()] } else { vec![] };
        ensure!(
            record.program == program
                && !record.limits.timeout.is_zero()
                && record.limits.timeout <= Duration::from_secs(600)
                && record.limits.writable_bytes
                    == if request.build_dependencies { 512 * 1024 * 1024 } else { 64 * 1024 * 1024 }
                && record.limits.stdout_bytes == 65536
                && record.limits.stderr_bytes == 65536
                && record.limits.stdin_bytes == 0,
            "Cargo prerequisite recipe or budget changed"
        );
        ensure!(record.snapshots.len() == candidate.len(), "Cargo environment snapshot closure differs");
        for (name, hash) in &candidate {
            let snapshot = record.snapshots.get(name).context("Cargo environment source missing")?;
            ensure!(
                snapshot.path == root.join("source").join(name) && snapshot.sha256 == *hash,
                "Cargo environment snapshot differs from source"
            );
            retention::authenticate_reference(snapshot, MAX_SOURCE_BYTES)?;
        }
        let actual = operation::authenticate_request(
            &record.runtime.operation_root,
            &record.context,
            &record.program,
            &record.limits,
            &[],
        )?;
        ensure!(actual.result_sha256 == record.operation_result_sha256, "Cargo environment operation changed");
        for reference in &record.references {
            retention::authenticate_reference(reference, MAX_RECORD_BYTES.max(MAX_SOURCE_BYTES))?;
        }
        if record.passed {
            ensure!(
                actual.passed && actual.status_raw == Some(0) && record.diagnostic.is_empty(),
                "Cargo environment accepted a failed prerequisite"
            );
            if request.build_dependencies {
                let cache = record.cache.as_ref().context("Cargo environment omitted its dependency cache")?;
                let project_names = project_packages(checkout, &prepared.contract.source_files)?;
                let files = archive_cache_files(
                    actual.outputs.get(Path::new("dependencies.zip")).context("Cargo environment archive missing")?,
                    &project_names,
                )?;
                ensure!(
                    cache.root == root.join("dependencies") && cache.files == files,
                    "Cargo dependency cache differs from the confined producer archive"
                );
                check_closure(cache, true)?;
            } else {
                ensure!(record.cache.is_none(), "runtime-only preparation cannot supply a dependency cache");
            }
        } else {
            ensure!(
                !record.diagnostic.is_empty() && record.cache.is_none(),
                "Cargo prerequisite failure omitted diagnosis or retained an accepted cache"
            );
        }
        let actual_bytes = environment_record_references(&record, &receipt.record.file)?
            .iter()
            .try_fold(0u64, |total, reference| {
                total.checked_add(reference.bytes).context("Cargo environment extent overflow")
            })?;
        ensure!(actual_bytes == receipt.evidence_bytes, "Cargo environment retained extent changed");
        Ok(())
    })
}

fn sealed_json(value: &impl Serialize) -> Result<String> {
    Ok(sha256(serde_json::to_vec(value)?))
}

pub(crate) fn environment_references(receipt: &CargoEnvironmentReceipt) -> Result<Vec<Reference>> {
    ensure!(bounded::digest(&receipt.record.file)? == receipt.record.sha256, "Cargo environment record changed");
    let record: EnvironmentRecord = family::object(&receipt.record.file)?;
    let mut files = environment_record_references(&record, &receipt.record.file)?;
    files.push(retention::seal_file(
        &receipt
            .record
            .file
            .parent()
            .context("Cargo environment record parent missing")?
            .join("cargo-environment-receipt.toml"),
        MAX_RECORD_BYTES,
    )?);
    Ok(files)
}

fn project_packages(checkout: &Path, files: &BTreeMap<String, String>) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    for name in files.keys().filter(|name| name.ends_with("Cargo.toml")) {
        let document: toml::Value = toml::from_str(&bounded::text(checkout.join(name))?)?;
        if let Some(name) =
            document.get("package").and_then(|package| package.get("name")).and_then(toml::Value::as_str)
        {
            names.insert(name.into());
        }
    }
    Ok(names)
}

fn external_cache_path(name: &str, projects: &BTreeSet<String>) -> bool {
    let parts = name.split('/').collect::<Vec<_>>();
    relative(name)
        && parts.len() >= 5
        && parts[0] == "debug"
        && parts[1] == "build"
        && !projects.contains(parts[2])
        && parts[3].bytes().all(|byte| byte.is_ascii_hexdigit())
        && !parts[3].is_empty()
}

fn archive_cache_files(bytes: &[u8], projects: &BTreeSet<String>) -> Result<BTreeMap<String, String>> {
    use std::io::Read;
    ensure!(bytes.len() as u64 <= MAX_SOURCE_BYTES, "Cargo dependency archive extent exceeded");
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
    ensure!(!archive.is_empty() && archive.len() <= MAX_FILES, "Cargo dependency archive is empty or excessive");
    let mut files = BTreeMap::new();
    let mut total = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        ensure!(
            external_cache_path(&name, projects)
                && !entry.is_dir()
                && entry
                    .unix_mode()
                    .is_none_or(|mode| mode & libc::S_IFMT == 0 || mode & libc::S_IFMT == libc::S_IFREG),
            "Cargo cache archive includes a project artifact, alias or special file"
        );
        total = total.checked_add(entry.size()).context("Cargo cache aggregate extent overflow")?;
        ensure!(
            total <= MAX_DEPENDENCY_BYTES && entry.size() <= MAX_SOURCE_BYTES,
            "Cargo cache decompressed extent exceeded"
        );
        let mut body = Vec::new();
        entry.by_ref().take(MAX_SOURCE_BYTES + 1).read_to_end(&mut body)?;
        ensure!(
            body.len() as u64 == entry.size() && files.insert(name, sha256(&body)).is_none(),
            "Cargo cache archive duplicates or changes an entry"
        );
    }
    Ok(files)
}

fn unpack_cache(bytes: &[u8], output: &Path, projects: &BTreeSet<String>) -> Result<CargoClosure> {
    use std::io::Read;
    let files = archive_cache_files(bytes, projects)?;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        let executable = entry.unix_mode().is_some_and(|mode| mode & 0o111 != 0);
        let mut body = Vec::new();
        entry.by_ref().take(MAX_SOURCE_BYTES + 1).read_to_end(&mut body)?;
        ensure!(files.get(&name) == Some(&sha256(&body)), "Cargo dependency archive changed during extraction");
        publish_bytes(&output.join(name), &body, executable)?;
    }
    Ok(CargoClosure { root: output.into(), files })
}

/// This installed producer is entered only inside the quota namespace. Every
/// child receives a fresh environment and the declared offline closure.
pub(crate) fn producer_cli(args: &[String]) -> Result<()> {
    ensure!(args == ["--request", "/inputs/cargo-request.json"], "Cargo producer requires its installed request path");
    ensure!(
        std::env::current_dir()? == Path::new("/work")
            && std::env::var("CARGO_HOME").ok().as_deref() == Some("/work/tmp/cargo-home")
            && !Path::new("/trusted-helper").try_exists()?,
        "Cargo producer requires the confined execution namespace"
    );
    let request: ProducerRequest =
        serde_json::from_slice(&bounded::read_limit(Path::new("/inputs/cargo-request.json"), 65536)?)?;
    ensure!(request.schema == "translation-cargo-producer-v1", "unknown Cargo producer request");
    let command = request
        .kind
        .commands()
        .into_iter()
        .find(|command| command.id == request.command_id)
        .context("unknown original Cargo command")?;
    fs::create_dir_all("/work/tmp/target")?;
    if request.dependency_cache {
        copy_cache(Path::new("/inputs/cache"), Path::new("/work/tmp/target"))?;
    }
    let status = configured_cargo()?.args(&command.argv[1..]).status()?;
    ensure!(status.success(), "original Cargo command exited with {status}");
    if command.export_binary {
        let source = Path::new("/work/tmp/target/debug/sm-pipeline");
        let bytes = bounded::read_limit(source, MAX_SOURCE_BYTES)?;
        ensure!(bytes.starts_with(b"\x7fELF"), "Cargo did not build the normal compact controller ELF");
        fs::write("/work/sm-pipeline", bytes)?;
    }
    Ok(())
}

fn configured_cargo() -> Result<Command> {
    fs::create_dir_all("/work/tmp/target")?;
    fs::write(
        "/work/tmp/cargo-home/config.toml",
        "[source.crates-io]\nreplace-with = \"sealed-vendor\"\n[source.sealed-vendor]\ndirectory = \"/inputs/vendor\"\n[net]\noffline = true\n",
    )?;
    let mut command = Command::new("/toolchain/bin/cargo");
    command
        .current_dir("/inputs/source")
        .env_clear()
        .env("PATH", "/toolchain/bin:/usr/bin:/bin")
        .env("HOME", "/work/tmp/home")
        .env("TMPDIR", "/work/tmp")
        .env("CARGO_HOME", "/work/tmp/cargo-home")
        .env("CARGO_TARGET_DIR", "/work/tmp/target")
        .env("CARGO_BUILD_JOBS", "1")
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_PROFILE_DEV_DEBUG", "0")
        .env("CARGO_PROFILE_TEST_DEBUG", "0")
        .env("CARGO_PROFILE_DEV_STRIP", "symbols")
        .env("CARGO_PROFILE_TEST_STRIP", "symbols")
        .env("CARGO_TERM_COLOR", "never")
        .env("CARGO_TERM_PROGRESS_WHEN", "never")
        .env("RUST_TEST_THREADS", "1")
        .env("RUSTC", "/toolchain/bin/rustc")
        .env("RUSTDOC", "/toolchain/bin/rustdoc")
        .env("RUSTFLAGS", "-C linker=/usr/bin/cc -C linker-features=-lld -C link-arg=-B/usr/lib/gcc/x86_64-linux-gnu/9/ -C link-arg=-B/usr/bin/")
        .env("CC", "/usr/bin/cc")
        .env("AR", "/usr/bin/ar")
        .env("LD_LIBRARY_PATH", "/toolchain/lib")
        .env("SM_PIPELINE_ROOT", "/work/tmp/pipeline")
        .env("SM_PIPELINE_WORKERS", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    Ok(command)
}

pub(crate) fn environment_producer_cli(args: &[String]) -> Result<()> {
    ensure!(
        args == ["--request", "/inputs/cargo-request.json"]
            && std::env::current_dir()? == Path::new("/work")
            && std::env::var("CARGO_HOME").ok().as_deref() == Some("/work/tmp/cargo-home")
            && !Path::new("/trusted-helper").try_exists()?,
        "Cargo environment producer requires the installed confined request"
    );
    let request: EnvironmentRequest =
        serde_json::from_slice(&bounded::read_limit(Path::new("/inputs/cargo-request.json"), 65536)?)?;
    ensure!(request.schema == "translation-cargo-environment-producer-v1", "unsupported Cargo prerequisite producer");
    if !request.build_dependencies {
        let status = Command::new("/toolchain/bin/rustc")
            .arg("--version")
            .env_clear()
            .env("LD_LIBRARY_PATH", "/toolchain/lib")
            .stdin(Stdio::null())
            .status()?;
        ensure!(status.success(), "installed compiler prerequisite failed");
        return Ok(());
    }
    let status = configured_cargo()?
        .args([
            "build",
            "--locked",
            "--offline",
            "--lib",
            "-p",
            "translation-execution",
            "-p",
            "evidence",
            "-p",
            "movie",
            "-p",
            "pipeline-model",
        ])
        .status()?;
    ensure!(status.success(), "confined frozen dependency build failed with {status}");
    let source_names = tree_files(Path::new("/inputs/source"))?.into_iter().map(|name| (name, String::new())).collect();
    let projects = project_packages(Path::new("/inputs/source"), &source_names)?;
    let mut vendor_names = BTreeSet::new();
    for directory in bounded::entries(Path::new("/inputs/vendor"))? {
        if !directory.is_dir() {
            continue;
        }
        let manifest: toml::Value = toml::from_str(&bounded::text(directory.join("Cargo.toml"))?)?;
        if let Some(name) =
            manifest.get("package").and_then(|package| package.get("name")).and_then(toml::Value::as_str)
        {
            vendor_names.insert(name.to_owned());
        }
    }
    let mut archive = zip::ZipWriter::new(fs::File::create("/work/dependencies.zip")?);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(1));
    let root = Path::new("/work/tmp/target");
    let mut count = 0usize;
    let mut total = 0u64;
    for name in tree_files(root)? {
        if !external_cache_path(&name, &projects)
            || !vendor_names.contains(name.split('/').nth(2).context("Cargo target package missing")?)
        {
            continue;
        }
        let path = root.join(&name);
        let metadata = path.symlink_metadata()?;
        total = total.checked_add(metadata.len()).context("Cargo dependency target extent overflow")?;
        ensure!(
            total <= MAX_DEPENDENCY_BYTES && metadata.len() <= MAX_SOURCE_BYTES,
            "Cargo dependency target exceeds reviewed extent"
        );
        archive.start_file(&name, options.unix_permissions(metadata.permissions().mode() & 0o777))?;
        std::io::copy(&mut fs::File::open(path)?, &mut archive)?;
        count += 1;
    }
    ensure!(count > 0, "installed Cargo target layout produced no dependency cache entries");
    let archive = archive.finish()?;
    archive.sync_all()?;
    ensure!(archive.metadata()?.len() <= MAX_SOURCE_BYTES, "Cargo dependency archive exceeds retained extent");
    Ok(())
}

fn copy_cache(source: &Path, destination: &Path) -> Result<()> {
    let names = tree_files(source)?;
    let mut total = 0u64;
    for name in names {
        let input = source.join(&name);
        let output = destination.join(&name);
        let metadata = input.symlink_metadata()?;
        total = total.checked_add(metadata.len()).context("Cargo cache extent overflow")?;
        ensure!(total <= MAX_DEPENDENCY_BYTES, "Cargo cache extent exceeded");
        fs::create_dir_all(output.parent().context("Cargo cache parent missing")?)?;
        fs::copy(&input, &output)?;
        fs::set_permissions(&output, fs::Permissions::from_mode(metadata.permissions().mode() & 0o777))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "translation_cargo_tests.rs"]
mod tests;
