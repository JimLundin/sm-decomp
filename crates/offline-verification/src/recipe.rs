//! Reviewed Offline family recipes shared by ordinary and confined execution.
//! Recipe selection owns the complete declared domain; execution adapters do
//! not choose a different compiler, Fixture command or evidence prescription.
use crate::family_model::{Asset, Case, CaseRecord, Implementation, Manifest, Routine, Verdict, VerifyRequest, Write};
use crate::{builds, callback, catalog, family, fixture_execution, fixture_format, inputs, plm_fixtures, plm_spawn};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};
use tool_execution::bounded::{self, digest};
use translation_execution::model::sha256;

#[path = "recipe_commands.rs"]
mod commands;
pub(crate) use commands::{
    CompilerCommand, FixtureKind, FixtureSelection, c_compiler_command, compiler_files, confined_fixture_program,
    fixture_command, fixture_output_path, fixture_program, rust_compiler_command,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Recipe {
    Dispatcher,
    Callback,
    PowerBomb,
    GameHost,
    Plm,
    Ridley,
    GeneratedPlm { manifest: Asset },
    DirectAnimation { plan: Asset },
    SupplementedC { support: Vec<String> },
}

const GENERATED_PLM: &str = ".scratch/translation-native/plm-instructions/manifest.smf";

pub(crate) fn input_inventory_limit(recipe: Option<&Recipe>) -> usize {
    // Captured Ridley declares 5,371 files. Preserve its complete closure while
    // keeping the original, smaller ceiling for other recipes.
    if matches!(recipe, Some(Recipe::Ridley)) { 8192 } else { 4096 }
}

pub(crate) fn dependency_digest(recipe: Option<&Recipe>, path: &Path) -> Result<String> {
    if matches!(recipe, Some(Recipe::Ridley)) && path == Path::new(inputs::CAPTURED_RIDLEY_EVENT) {
        inputs::captured_ridley_event_digest(path)
    } else {
        digest(path)
    }
}

pub(crate) fn prepare_generated_plm(checkout: &Path, output: &Path) -> Result<crate::family_model::PlmGeneration> {
    decomp_source::context::limited(|| plm_fixtures::generate(checkout, output, Some(&checkout.join(GENERATED_PLM))))
}

fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|part| matches!(part, Component::Normal(_)))
}

fn file(path: &Path) -> Result<PathBuf> {
    bounded::safe(path)?;
    ensure!(path.symlink_metadata()?.is_file(), "native input is not a regular file");
    Ok(path.into())
}

fn asset_path(checkout: &Path, asset: &Asset) -> Result<PathBuf> {
    let path = if asset.file.is_absolute() {
        asset.file.clone()
    } else {
        ensure!(relative(&asset.file), "native asset relative path alias");
        checkout.join(&asset.file)
    };
    file(&path)
}

fn check_asset(checkout: &Path, asset: &Asset) -> Result<PathBuf> {
    let path = asset_path(checkout, asset)?;
    ensure!(digest(&path)? == asset.sha256, "native asset changed: {}", path.display());
    Ok(path)
}

fn read_fixture(checkout: &Path, asset: &Asset) -> Result<Manifest> {
    fixture_format::read_authenticated(&asset_path(checkout, asset)?, &asset.sha256)
}

pub(crate) fn plain(manifest: &Manifest) -> Result<()> {
    ensure!(
        !manifest.uses_dispatcher()
            && !manifest.projectile_host
            && !manifest.projectile_rendering
            && manifest.callback_adapter.is_none()
            && !manifest.infrastructure_probe,
        "native recipe does not support this registered harness"
    );
    ensure!(manifest.cases.is_empty(), "plain family has undeclared top-level cases");
    Ok(())
}

pub(crate) fn for_family(checkout: &Path, path: &Path, declared: &Manifest) -> Result<Option<Recipe>> {
    let registration = registered_entry(checkout, path, declared)?;
    recipe_for_entry(checkout, declared, &registration.entry)
}

/// Preserve ordinary manifest/adapter requests without requiring a Native
/// catalog registration or silently selecting a different callback adapter.
pub(crate) fn for_adapter(adapter: &str, manifest: &Manifest) -> Result<Option<Recipe>> {
    if adapter == "ridley-subsystem" {
        ensure!(manifest.callback_adapter.as_deref() == Some("ridley-subsystem"), "Ridley adapter declaration differs");
        return Ok(Some(Recipe::Ridley));
    }
    ensure!(
        manifest.callback_adapter.as_deref() != Some("ridley-subsystem"),
        "Ridley manifest requires its native adapter"
    );
    Ok(match adapter {
        "plm" => Some(Recipe::Plm),
        "callback" | "power-bomb" if manifest.callback_adapter.as_deref() == Some("game-host") => {
            Some(Recipe::GameHost)
        }
        "power-bomb" => Some(Recipe::PowerBomb),
        "callback" => Some(Recipe::Callback),
        _ if manifest.uses_dispatcher() => Some(Recipe::Dispatcher),
        _ => None,
    })
}

fn recipe_for_entry(checkout: &Path, declared: &Manifest, entry: &catalog::FamilyEntry) -> Result<Option<Recipe>> {
    ensure!(entry.prepare.is_none(), "unreviewed native preparation command");
    Ok(match entry.kind.as_str() {
        "plm-instructions" => {
            let path = checkout.join(GENERATED_PLM);
            Some(Recipe::GeneratedPlm {
                manifest: Asset { file: GENERATED_PLM.into(), sha256: digest(&file(&path)?)? },
            })
        }
        "plm" => Some(Recipe::Plm),
        "ridley-subsystem" => Some(Recipe::Ridley),
        "power-bomb" => Some(Recipe::PowerBomb),
        "callback" => Some(if declared.callback_adapter.as_deref() == Some("game-host") {
            Recipe::GameHost
        } else {
            Recipe::Callback
        }),
        "family" | "offline-family" | "dispatcher" | "projectile" | "offline-projectile"
            if declared.callback_adapter.as_deref() == Some("ridley-subsystem") =>
        {
            // Its original callback metadata selects the reviewed Ridley
            // adapter even when catalog discovery sees projectile-host fields.
            Some(Recipe::Ridley)
        }
        "family" | "offline-family" | "dispatcher" | "projectile" | "offline-projectile" => {
            ensure!(declared.callback_adapter.is_none(), "unregistered native callback adapter");
            if declared.uses_dispatcher() {
                Some(Recipe::Dispatcher)
            } else {
                plain(declared)?;
                let additional: &[&str] = match declared.family.as_str() {
                    "animated-tile-frames" => &["sm_84.c", "sm_94.c", "sm_a0.c"],
                    "animated-tile-spawns" => &["sm_80.c", "sm_84.c", "sm_94.c", "sm_a0.c"],
                    "ceres-pad" => &["sm_90.c"],
                    _ => &[],
                };
                if additional.is_empty() {
                    None
                } else {
                    Some(Recipe::SupplementedC { support: additional.iter().map(|s| (*s).into()).collect() })
                }
            }
        }
        _ => anyhow::bail!("unsupported registered native recipe: {}", entry.kind),
    })
}

pub(crate) struct Registration {
    pub catalog_family: String,
    pub entry: catalog::FamilyEntry,
}

fn registered_entry(checkout: &Path, path: &Path, declared: &Manifest) -> Result<Registration> {
    // Read the explicitly reviewed catalog first. Discovered declarations keep
    // the existing exact family/path rule rather than being promoted to aliases.
    let catalog: catalog::Catalog = bounded::object(&checkout.join("tools/port/batch.toml"))?;
    ensure!(catalog.families.len() <= 4096, "native registered family inventory exceeded");
    let mut registered = None;
    for name in catalog.families.keys() {
        let entry = catalog::entry(&catalog, name)?;
        if entry.manifest == path {
            ensure!(registered.is_none(), "ambiguous native registered manifest path");
            registered = Some(Registration { catalog_family: name.clone(), entry });
        }
    }
    let registered = if let Some(registered) = registered {
        registered
    } else {
        ensure!(!catalog.families.contains_key(&declared.family), "native family registration path differs");
        Registration {
            catalog_family: declared.family.clone(),
            entry: catalog::FamilyEntry {
                kind: if declared.uses_dispatcher() { "dispatcher" } else { "family" }.into(),
                manifest: PathBuf::from(format!("tools/port/{}.smf", declared.family)),
                live: None,
                prepare: None,
            },
        }
    };
    ensure!(relative(path) && registered.entry.manifest == path, "native family registration path differs");
    Ok(registered)
}

pub(crate) fn registration(
    checkout: &Path,
    manifest: &Asset,
    family: &str,
    recipe: Option<&Recipe>,
    original: &Manifest,
) -> Result<Registration> {
    let registration = registered_entry(checkout, &manifest.file, original)?;
    let entry = &registration.entry;
    ensure!(entry.prepare.is_none(), "native family registration differs");
    if matches!(recipe, Some(Recipe::DirectAnimation { .. })) {
        ensure!(
            original.family == "pause-equipment-render" && family == "pause-direct-animation",
            "auxiliary family identity differs"
        );
    } else {
        ensure!(
            recipe_for_entry(checkout, original, entry)?.as_ref() == recipe && family == original.family,
            "native registered recipe differs"
        );
    }
    Ok(registration)
}

#[cfg(test)]
pub(crate) fn manifest(checkout: &Path, path: &Path, recipe: Option<&Recipe>) -> Result<Manifest> {
    normalize_manifest(checkout, fixture_format::read(path)?, recipe)
}

pub(crate) fn normalize_manifest(checkout: &Path, declared: Manifest, recipe: Option<&Recipe>) -> Result<Manifest> {
    Ok(match recipe {
        Some(Recipe::Plm) => plm_spawn::normalize(&declared)?,
        Some(Recipe::GeneratedPlm { manifest }) => {
            let generated = read_fixture(checkout, manifest)?;
            let rom = generated.rom.as_ref().context("generated PLM ROM missing")?;
            let (expected, bytes) = plm_fixtures::generated_contents(checkout, &rom.file)?;
            ensure!(
                generated == fixture_format::decode(fixture_format::encode(&expected)?)?
                    && digest(&check_asset(checkout, rom)?)? == sha256(&bytes),
                "generated PLM coverage or ROM differs from complete generator"
            );
            generated
        }
        Some(Recipe::DirectAnimation { plan }) => {
            fixture_format::decode(fixture_format::encode(&direct_manifest(checkout, &declared, plan)?)?)?
        }
        Some(Recipe::SupplementedC { support }) => {
            let mut supplemented = declared;
            for provider in support {
                if !supplemented.c_support.contains(provider) {
                    supplemented.c_support.push(provider.clone());
                }
            }
            supplemented
        }
        _ => declared,
    })
}

/// Generated declarations are already materialized in retained runs. Only PLM
/// allocation fixtures still need their ordinary normalized view on readback.
pub(crate) fn retained_manifest(manifest: Manifest, recipe: Option<&Recipe>) -> Result<Manifest> {
    match recipe {
        Some(Recipe::Plm) => plm_spawn::normalize(&manifest),
        _ => Ok(manifest),
    }
}

pub(crate) fn harness(recipe: Option<&Recipe>) -> Vec<&'static str> {
    let mut paths =
        vec!["crates/offline-verification/src/recipe.rs", "crates/offline-verification/src/recipe_commands.rs"];
    paths.extend_from_slice(match recipe {
        Some(Recipe::Callback | Recipe::PowerBomb | Recipe::GameHost) => &[
            "crates/offline-verification/src/callback.rs",
            "crates/offline-verification/src/callback_abi.rs.in",
            "crates/offline-verification/src/callback_host.c.in",
            "crates/offline-verification/src/power_bomb_abi.rs.in",
            "crates/offline-verification/src/power_bomb_host.c.in",
            "crates/offline-verification/src/game_host_abi.rs.in",
            "crates/offline-verification/src/game_host_dependency.c.in",
        ],
        Some(Recipe::Plm) => &[
            "crates/offline-verification/src/plm_spawn.rs",
            "crates/offline-verification/src/plm_spawn_abi.rs.in",
            "crates/offline-verification/src/plm_spawn_host.c.in",
        ],
        Some(Recipe::Ridley) => &[
            "crates/offline-verification/src/ridley_subsystem_adapter.rs",
            "crates/pipeline/src/ridley_subsystem_fixtures.rs",
            "crates/offline-verification/src/ridley_subsystem_abi.rs.in",
            "crates/offline-verification/src/ridley_subsystem_host.c.in",
        ],
        _ => &[],
    });
    if full_c(recipe) {
        paths.extend(["tools/decomp/headless.c", "tools/decomp/headless_frame.h"]);
    }
    paths
}

pub(crate) fn freeze_assets(
    checkout: &Path,
    recipe: Option<&Recipe>,
    inputs: &mut BTreeMap<String, String>,
) -> Result<()> {
    let asset = match recipe {
        Some(Recipe::GeneratedPlm { manifest }) => Some(manifest),
        Some(Recipe::DirectAnimation { plan }) => Some(plan),
        _ => None,
    };
    if let Some(asset) = asset {
        check_asset(checkout, asset)?;
        inputs.insert(asset.file.to_str().context("recipe asset path UTF-8")?.into(), asset.sha256.clone());
    }
    if matches!(recipe, Some(Recipe::GeneratedPlm { .. })) {
        inputs.insert("rom/sm.sfc".into(), digest(&file(&checkout.join("rom/sm.sfc"))?)?);
    }
    Ok(())
}

pub(crate) fn full_c(recipe: Option<&Recipe>) -> bool {
    recipe.is_some_and(|r| !matches!(r, Recipe::DirectAnimation { .. } | Recipe::SupplementedC { .. }))
}

pub(crate) fn generated(recipe: Option<&Recipe>) -> bool {
    matches!(recipe, Some(Recipe::GeneratedPlm { .. } | Recipe::DirectAnimation { .. } | Recipe::SupplementedC { .. }))
}

pub(crate) fn adapter(recipe: Option<&Recipe>) -> &'static str {
    match recipe {
        Some(Recipe::Callback | Recipe::GameHost) => "callback",
        Some(Recipe::PowerBomb) => "power-bomb",
        Some(Recipe::Plm) => "plm",
        Some(Recipe::Ridley) => "ridley-subsystem",
        _ => "family",
    }
}

pub(crate) fn manifest_bytes(
    recipe: Option<&Recipe>,
    manifest: &Manifest,
    asset: &Asset,
    checkout: &Path,
) -> Result<Vec<u8>> {
    if generated(recipe) { fixture_format::encode(manifest) } else { bounded::read(checkout.join(&asset.file)) }
}

pub(crate) fn source_symbols<'a>(manifest: &'a Manifest, recipe: Option<&Recipe>) -> Vec<&'a str> {
    manifest
        .routines
        .iter()
        .map(|r| match r.symbol.as_str() {
            "terrain" if matches!(recipe, Some(Recipe::Plm)) => "SpawnPLM",
            "room" if matches!(recipe, Some(Recipe::Plm)) => "SpawnRoomPLM",
            "hardcoded" if matches!(recipe, Some(Recipe::Plm)) => "SpawnHardcodedPlm",
            "PlmInstruction" => "CallPlmInstr",
            "PlmControlFrame" | "PlmFrame" => "PlmHandler_Async",
            "EprojInstruction" => "CallEprojInstr",
            "EprojPreinstruction" => "CallEprojPreInstr",
            "EprojInitializer" => "CallEprojInit",
            "PortScreenShake" => "GetValuesForScreenShaking",
            "PortDrawEprojs" => "DrawEprojs",
            "PortDrawEprojSpritemap" => "DrawEprojSpritemapWithBaseTile",
            "PortDrawEprojSpritemapOffscreen" => "DrawEprojSpritemapWithBaseTileOffscreen",
            symbol => symbol,
        })
        .collect()
}

pub(crate) fn build(
    recipe: Option<&Recipe>,
    manifest: &Manifest,
    request: &VerifyRequest,
    checkout: &Path,
    pinned_source: &Path,
    run: &Path,
    execution: Option<&builds::StrictExecution>,
) -> Result<Implementation> {
    match recipe {
        Some(Recipe::Callback | Recipe::PowerBomb | Recipe::GameHost) => {
            callback::build(recipe, request, manifest, checkout, pinned_source, run, execution)
        }
        Some(Recipe::Plm) => plm_spawn::build(recipe, request, manifest, checkout, pinned_source, run, execution),
        Some(Recipe::Ridley) => {
            crate::ridley_subsystem_adapter::build(recipe, request, manifest, checkout, pinned_source, run, execution)
        }
        _ => family::build(recipe, request, manifest, checkout, pinned_source, run, execution),
    }
}

pub(crate) enum FixtureExecution<'a> {
    Ordinary { worker: &'a mut fixture_execution::Worker },
    Confined(&'a builds::StrictExecution),
}

pub(crate) fn fixture(
    recipe: Option<&Recipe>,
    manifest: &Manifest,
    routine: &Routine,
    fixture: &Case,
    index: usize,
    run: &Path,
    mut execution: FixtureExecution<'_>,
) -> Result<CaseRecord> {
    let resume = matches!(execution, FixtureExecution::Confined(_));
    let execute = |directory: &Path, label: &str| {
        let command =
            fixture_command(recipe, manifest, FixtureSelection::Case { routine, fixture, index }, run, label)?;
        match &mut execution {
            FixtureExecution::Ordinary { worker } => worker.execute(&command.command, directory, label),
            FixtureExecution::Confined(execution) => {
                execution.fixture_recipe(&command.command, directory, label, command.kind).map(|_| ())
            }
        }
    };
    match recipe {
        Some(Recipe::Callback | Recipe::PowerBomb | Recipe::GameHost) => {
            callback::case(manifest, routine, fixture, run, resume, execute)
        }
        Some(Recipe::Plm) => plm_spawn::case(routine, fixture, run, resume, execute),
        Some(Recipe::Ridley) => crate::ridley_subsystem_adapter::case(routine, fixture, run, resume, execute),
        _ => family::case(manifest, routine, fixture, run, resume, execute),
    }
}

pub(crate) fn start_verdict(recipe: Option<&Recipe>, manifest: &Manifest, verdict: &mut Verdict) {
    if matches!(recipe, Some(Recipe::Callback | Recipe::PowerBomb | Recipe::GameHost | Recipe::Ridley)) {
        verdict.gameplay_translation = Some(!manifest.infrastructure_probe);
    }
}

pub(crate) fn verdict_fields(recipe: Option<&Recipe>, manifest: &Manifest, verdict: &mut Verdict) -> Result<()> {
    start_verdict(recipe, manifest, verdict);
    match recipe {
        Some(Recipe::Callback | Recipe::PowerBomb | Recipe::GameHost | Recipe::Ridley) => {
            if matches!(recipe, Some(Recipe::GameHost | Recipe::Ridley)) {
                verdict.remaining_c_gameplay_callbacks =
                    usize::try_from(verdict.cases.iter().map(|c| c.callbacks).sum::<u64>())?;
            }
            if matches!(recipe, Some(Recipe::Ridley)) {
                verdict.rejected_abi_inputs = 7;
            }
        }
        Some(Recipe::Plm) => {
            verdict.rejected_abi_inputs = 5;
            verdict.native_rust_spawn_calls = verdict.cases.len();
            verdict.allocator_cases = manifest.routines.iter().map(|r| (r.symbol.clone(), r.cases.len())).collect();
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn abi(recipe: Option<&Recipe>) -> bool {
    matches!(recipe, Some(Recipe::Plm | Recipe::Ridley))
}

/// Authenticate adapter artifacts after a completed fixture's confined output
/// has been authenticated against its command and retained receipt.
pub(crate) fn authenticate_fixture_artifacts(recipe: Option<&Recipe>, directory: &Path, label: &str) -> Result<()> {
    if matches!(recipe, Some(Recipe::Callback | Recipe::PowerBomb | Recipe::GameHost)) {
        callback::authenticate_packed_snapshots(directory, label)?;
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnimationPlan {
    schema: String,
    routine: String,
    animation_ids: Vec<u16>,
    timers: Vec<u16>,
    frames: Vec<u16>,
    coordinates: Vec<[u16; 2]>,
    oam_cursors: Vec<u16>,
    limitations: Vec<String>,
}

pub(crate) fn direct_manifest(checkout: &Path, original: &Manifest, plan: &Asset) -> Result<Manifest> {
    let plan: AnimationPlan = bounded::object(&check_asset(checkout, plan)?)?;
    ensure!(
        plan.schema == "decomp-direct-animation-cases-v1"
            && plan.routine == "DrawPauseScreenSpriteAnim"
            && plan.animation_ids == [2, 3, 6, 7, 8, 9]
            && plan.timers == [0, 1, 2, 32768, 65535]
            && plan.frames == [0, 255]
            && plan.coordinates == [[0, 0], [255, 224], [65535, 65535]]
            && plan.oam_cursors == [0, 508]
            && !plan.limitations.is_empty(),
        "original direct animation domain differs"
    );
    let rom = bounded::read(check_asset(checkout, original.rom.as_ref().context("animation ROM missing")?)?)?;
    let offset = |address: usize| (((address >> 16) << 15) | (address & 0x7fff)) & 0x3fffff;
    let word = |address: usize| -> Result<u16> {
        let at = offset(address);
        Ok(u16::from_le_bytes(rom.get(at..at + 2).context("animation ROM table extent")?.try_into()?))
    };
    let mut cases = Vec::new();
    for animation in plan.animation_ids {
        let index = usize::from(animation - 1) * 2;
        let script = word(0x82c0e8 + index)?;
        let start = offset(0x820000 | usize::from(script));
        let count =
            (0..64).find(|frame| rom.get(start + frame * 3) == Some(&255)).context("animation sentinel missing")?;
        ensure!(count > 0, "empty animation sequence");
        for timer in &plan.timers {
            for requested in &plan.frames {
                let frame = if *requested == 255 { (count - 1) as u16 } else { *requested };
                ensure!(usize::from(frame) < count, "animation frame out of range");
                for [x, y] in &plan.coordinates {
                    for oam in &plan.oam_cursors {
                        let mut writes = Vec::new();
                        for (table, value) in [(0x82c0b2, *timer), (0x82c0c4, frame), (0x82c0d6, 0)] {
                            let offset = usize::from(word(table + index)?);
                            ensure!(offset < 0x1fff, "animation pointer outside direct-page RAM");
                            writes.push(Write { offset, data: value.to_le_bytes().to_vec() });
                        }
                        writes.push(Write { offset: 0x590, data: oam.to_le_bytes().to_vec() });
                        cases.push(Case {
                            name: format!("pair-{}", cases.len()),
                            a: i64::from(u32::from(animation) | (u32::from(*x) << 16)),
                            b: i64::from(*y),
                            fill: Some(0xa5),
                            writes: writes.into(),
                            expected_return: Some(0),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }
    ensure!(cases.len() == 360, "direct animation coverage differs");
    Ok(Manifest {
        family: "pause-direct-animation".into(),
        rust_module: original.rust_module.clone(),
        bank: "sm_82.c".into(),
        c_support: vec!["sm_81.c".into(), "sm_rtl.c".into()],
        rom: original.rom.clone(),
        domain: Some(plan.limitations.join("; ")),
        routines: vec![Routine {
            symbol: plan.routine,
            arg_types: Some(vec!["uint32".into(), "uint16".into()]),
            packed_u16_args: Some(3),
            returns: "void".into(),
            cases,
            ..Default::default()
        }],
        ..Default::default()
    })
}
