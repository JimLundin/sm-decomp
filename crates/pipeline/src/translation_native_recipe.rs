//! Reviewed recipes extend the original family adapter without granting a
//! candidate access to a writable verifier or acceptance directory.
use super::*;
use crate::family_model::{Case, Implementation, Routine, Write};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum NativeRecipe {
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

const GENERATED_PLM: &str = ".scratch/translation-native/plm-instructions/manifest.toml";

pub(crate) fn prepare_generated_plm(checkout: &Path, output: &Path) -> Result<crate::family_model::PlmGeneration> {
    bounded::limited(|| crate::plm_fixtures::generate(checkout, output, Some(&checkout.join(GENERATED_PLM))))
}

pub(super) fn for_family(checkout: &Path, declared: &Manifest) -> Result<Option<NativeRecipe>> {
    let entry = registered_entry(checkout, declared)?;
    ensure!(entry.prepare.is_none(), "unreviewed native preparation command");
    Ok(match entry.kind.as_str() {
        "plm-instructions" => {
            let path = checkout.join(GENERATED_PLM);
            Some(NativeRecipe::GeneratedPlm {
                manifest: Asset { file: GENERATED_PLM.into(), sha256: digest(&file(&path)?)? },
            })
        }
        "plm" => Some(NativeRecipe::Plm),
        "ridley-subsystem" => Some(NativeRecipe::Ridley),
        "power-bomb" => Some(NativeRecipe::PowerBomb),
        "callback" => Some(if declared.callback_adapter.as_deref() == Some("game-host") {
            NativeRecipe::GameHost
        } else {
            NativeRecipe::Callback
        }),
        "family" | "offline-family" | "dispatcher" | "projectile" | "offline-projectile" => {
            ensure!(declared.callback_adapter.is_none(), "unregistered native callback adapter");
            if family::dispatcher(declared) {
                Some(NativeRecipe::Dispatcher)
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
                    Some(NativeRecipe::SupplementedC { support: additional.iter().map(|s| (*s).into()).collect() })
                }
            }
        }
        _ => anyhow::bail!("unsupported registered native recipe: {}", entry.kind),
    })
}

pub(super) fn registered_entry(checkout: &Path, declared: &Manifest) -> Result<batch::FamilyEntry> {
    let catalog: batch::Catalog = family::object(&checkout.join("tools/port/batch.toml"))?;
    if catalog.families.contains_key(&declared.family) {
        return batch::entry(&catalog, &declared.family);
    }
    let path = PathBuf::from(format!("tools/port/{}.toml", declared.family));
    ensure!(relative(&path), "native family path alias");
    let registered: Manifest = family::object(&file(&checkout.join(&path))?)?;
    ensure!(registered == *declared, "native discovery declaration differs");
    Ok(batch::FamilyEntry {
        kind: if family::dispatcher(declared) { "dispatcher" } else { "family" }.into(),
        manifest: path,
        live: None,
        prepare: None,
    })
}

pub(super) fn manifest(checkout: &Path, path: &Path, recipe: Option<&NativeRecipe>) -> Result<Manifest> {
    let declared: Manifest = family::object(path)?;
    Ok(match recipe {
        Some(NativeRecipe::Plm) => crate::plm_spawn::normalize(&declared)?,
        Some(NativeRecipe::GeneratedPlm { manifest }) => {
            let generated: Manifest = family::object(&check_asset(checkout, manifest)?)?;
            let rom = generated.rom.as_ref().context("generated PLM ROM missing")?;
            let (expected, bytes) = crate::plm_fixtures::generated_contents(checkout, &rom.file)?;
            ensure!(
                generated == expected && digest(&check_asset(checkout, rom)?)? == sha256(&bytes),
                "generated PLM coverage or ROM differs from complete generator"
            );
            generated
        }
        Some(NativeRecipe::DirectAnimation { plan }) => direct_manifest(checkout, &declared, plan)?,
        Some(NativeRecipe::SupplementedC { support }) => {
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

pub(super) fn registration(checkout: &Path, contract: &NativeContract, declared: &Manifest) -> Result<()> {
    let original: Manifest = family::object(&check_asset(checkout, &contract.manifest)?)?;
    let entry = registered_entry(checkout, &original)?;
    ensure!(entry.prepare.is_none() && entry.manifest == contract.manifest.file, "native family registration differs");
    if let Some(NativeRecipe::DirectAnimation { .. }) = contract.recipe {
        ensure!(
            original.family == "pause-equipment-render" && declared.family == "pause-direct-animation",
            "auxiliary family identity differs"
        );
    } else {
        ensure!(
            for_family(checkout, &original)? == contract.recipe && declared.family == original.family,
            "native registered recipe differs"
        );
    }
    Ok(())
}

pub(super) fn harness(recipe: Option<&NativeRecipe>) -> Vec<&'static str> {
    let Some(recipe) = recipe else {
        return vec![];
    };
    let mut paths = vec!["crates/pipeline/src/translation_native_recipe.rs"];
    let names: &[&str] = match recipe {
        NativeRecipe::Callback | NativeRecipe::PowerBomb | NativeRecipe::GameHost => &[
            "callback.rs",
            "callback_abi.rs.in",
            "callback_host.c.in",
            "power_bomb_abi.rs.in",
            "power_bomb_host.c.in",
            "game_host_abi.rs.in",
            "game_host_dependency.c.in",
        ],
        NativeRecipe::Plm => &["plm_spawn.rs", "plm_spawn_abi.rs.in", "plm_spawn_host.c.in"],
        NativeRecipe::Ridley => &[
            "ridley_subsystem_adapter.rs",
            "ridley_subsystem_fixtures.rs",
            "ridley_subsystem_abi.rs.in",
            "ridley_subsystem_host.c.in",
        ],
        _ => &[],
    };
    // All paths are static reviewed source names. Preserve path text in the seal.
    for name in names {
        paths.push(match *name {
            "callback.rs" => "crates/pipeline/src/callback.rs",
            "callback_abi.rs.in" => "crates/pipeline/src/callback_abi.rs.in",
            "callback_host.c.in" => "crates/pipeline/src/callback_host.c.in",
            "power_bomb_abi.rs.in" => "crates/pipeline/src/power_bomb_abi.rs.in",
            "power_bomb_host.c.in" => "crates/pipeline/src/power_bomb_host.c.in",
            "game_host_abi.rs.in" => "crates/pipeline/src/game_host_abi.rs.in",
            "game_host_dependency.c.in" => "crates/pipeline/src/game_host_dependency.c.in",
            "plm_spawn.rs" => "crates/pipeline/src/plm_spawn.rs",
            "plm_spawn_abi.rs.in" => "crates/pipeline/src/plm_spawn_abi.rs.in",
            "plm_spawn_host.c.in" => "crates/pipeline/src/plm_spawn_host.c.in",
            "ridley_subsystem_adapter.rs" => "crates/pipeline/src/ridley_subsystem_adapter.rs",
            "ridley_subsystem_fixtures.rs" => "crates/pipeline/src/ridley_subsystem_fixtures.rs",
            "ridley_subsystem_abi.rs.in" => "crates/pipeline/src/ridley_subsystem_abi.rs.in",
            "ridley_subsystem_host.c.in" => "crates/pipeline/src/ridley_subsystem_host.c.in",
            _ => unreachable!(),
        });
    }
    if full_c(Some(recipe)) {
        paths.extend(["tools/decomp/headless.c", "tools/decomp/headless_frame.h"]);
    }
    paths
}

pub(super) fn freeze_assets(
    checkout: &Path,
    recipe: Option<&NativeRecipe>,
    inputs: &mut BTreeMap<String, String>,
) -> Result<()> {
    let asset = match recipe {
        Some(NativeRecipe::GeneratedPlm { manifest }) => Some(manifest),
        Some(NativeRecipe::DirectAnimation { plan }) => Some(plan),
        _ => None,
    };
    if let Some(asset) = asset {
        check_asset(checkout, asset)?;
        inputs.insert(asset.file.to_str().context("recipe asset path UTF-8")?.into(), asset.sha256.clone());
    }
    if matches!(recipe, Some(NativeRecipe::GeneratedPlm { .. })) {
        inputs.insert("rom/sm.sfc".into(), digest(&file(&checkout.join("rom/sm.sfc"))?)?);
    }
    Ok(())
}

pub(super) fn full_c(recipe: Option<&NativeRecipe>) -> bool {
    recipe.is_some_and(|r| !matches!(r, NativeRecipe::DirectAnimation { .. } | NativeRecipe::SupplementedC { .. }))
}
pub(super) fn generated(recipe: Option<&NativeRecipe>) -> bool {
    matches!(
        recipe,
        Some(
            NativeRecipe::GeneratedPlm { .. }
                | NativeRecipe::DirectAnimation { .. }
                | NativeRecipe::SupplementedC { .. }
        )
    )
}
pub(super) fn adapter(recipe: Option<&NativeRecipe>) -> &'static str {
    match recipe {
        Some(NativeRecipe::Callback | NativeRecipe::GameHost) => "callback",
        Some(NativeRecipe::PowerBomb) => "power-bomb",
        Some(NativeRecipe::Plm) => "plm",
        Some(NativeRecipe::Ridley) => "ridley-subsystem",
        _ => "family",
    }
}
pub(super) fn manifest_bytes(prepared: &PreparedNative, checkout: &Path) -> Result<Vec<u8>> {
    if generated(prepared.contract.recipe.as_ref()) {
        Ok(toml::to_string(&prepared.manifest)?.into_bytes())
    } else {
        bounded::read(checkout.join(&prepared.contract.manifest.file))
    }
}
pub(super) fn source_symbols<'a>(manifest: &'a Manifest, recipe: Option<&NativeRecipe>) -> Vec<&'a str> {
    manifest
        .routines
        .iter()
        .map(|r| match r.symbol.as_str() {
            "terrain" if matches!(recipe, Some(NativeRecipe::Plm)) => "SpawnPLM",
            "room" if matches!(recipe, Some(NativeRecipe::Plm)) => "SpawnRoomPLM",
            "hardcoded" if matches!(recipe, Some(NativeRecipe::Plm)) => "SpawnHardcodedPlm",
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

pub(super) fn build(
    request: &VerifyRequest,
    p: &PreparedNative,
    checkout: &Path,
    run: &Path,
    execution: &builds::StrictExecution,
) -> Result<Implementation> {
    match p.contract.recipe.as_ref() {
        Some(NativeRecipe::Callback | NativeRecipe::PowerBomb | NativeRecipe::GameHost) => {
            crate::callback::build_with(request, &p.manifest, checkout, &p.pinned_source, run, Some(execution))
        }
        Some(NativeRecipe::Plm) => {
            crate::plm_spawn::build_with(request, &p.manifest, checkout, &p.pinned_source, run, Some(execution))
        }
        Some(NativeRecipe::Ridley) => crate::ridley_subsystem_adapter::build_with(
            request,
            &p.manifest,
            checkout,
            &p.pinned_source,
            run,
            Some(execution),
        ),
        _ => family::build_confined(request, &p.manifest, checkout, &p.pinned_source, run, execution),
    }
}
pub(super) fn fixture(
    p: &PreparedNative,
    routine: &Routine,
    fixture: &Case,
    index: usize,
    run: &Path,
    execution: &builds::StrictExecution,
) -> Result<crate::family_model::CaseRecord> {
    match p.contract.recipe.as_ref() {
        Some(NativeRecipe::Callback | NativeRecipe::PowerBomb | NativeRecipe::GameHost) => {
            crate::callback::confined_case(&p.manifest, routine, fixture, index, run, execution)
        }
        Some(NativeRecipe::Plm) => crate::plm_spawn::confined_case(routine, fixture, run, execution),
        Some(NativeRecipe::Ridley) => crate::ridley_subsystem_adapter::confined_case(routine, fixture, run, execution),
        _ => family::confined_case(&p.manifest, routine, fixture, run, execution),
    }
}
pub(super) fn verdict_fields(p: &PreparedNative, verdict: &mut Verdict) -> Result<()> {
    match p.contract.recipe.as_ref() {
        Some(NativeRecipe::Callback | NativeRecipe::PowerBomb | NativeRecipe::GameHost | NativeRecipe::Ridley) => {
            verdict.gameplay_translation = Some(!p.manifest.infrastructure_probe);
            if matches!(p.contract.recipe, Some(NativeRecipe::GameHost | NativeRecipe::Ridley)) {
                verdict.remaining_c_gameplay_callbacks =
                    usize::try_from(verdict.cases.iter().map(|c| c.callbacks).sum::<u64>())?;
            }
            if matches!(p.contract.recipe, Some(NativeRecipe::Ridley)) {
                verdict.rejected_abi_inputs = 7;
            }
        }
        Some(NativeRecipe::Plm) => {
            verdict.rejected_abi_inputs = 5;
            verdict.native_rust_spawn_calls = verdict.cases.len();
            verdict.allocator_cases = p.manifest.routines.iter().map(|r| (r.symbol.clone(), r.cases.len())).collect();
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn compiler_files(
    recipe: Option<&NativeRecipe>,
    rust: bool,
) -> (Vec<&'static str>, Vec<&'static str>, bool) {
    if rust {
        let mut generated = vec!["wrapper.rs", "memory.rs"];
        let output = match recipe {
            Some(NativeRecipe::Callback | NativeRecipe::PowerBomb | NativeRecipe::GameHost) => "libcollision.a",
            Some(NativeRecipe::Plm) => "libplm.a",
            Some(NativeRecipe::Ridley) => {
                generated.push("fixture-worker.rs");
                "libridley.a"
            }
            _ => "rust",
        };
        return (generated, vec![output, "compiler.d"], false);
    }
    let (generated, binary) = match recipe {
        Some(NativeRecipe::Dispatcher | NativeRecipe::GeneratedPlm { .. }) => {
            (vec!["oracle.c", "plm-shim.c", "host-support.c"], "c")
        }
        Some(NativeRecipe::Callback) => (vec!["oracle.c", "libcollision.a"], "collision"),
        Some(NativeRecipe::PowerBomb) => (vec!["oracle.c", "libcollision.a", "sm_88-allocation-seam.c"], "collision"),
        Some(NativeRecipe::GameHost) => {
            (vec!["oracle.c", "libcollision.a", "sm_88-allocation-seam.c", "sm_80-vram-seam.c"], "collision")
        }
        Some(NativeRecipe::Plm) => (vec!["oracle.c", "libplm.a"], "spawn"),
        Some(NativeRecipe::Ridley) => (
            vec![
                "oracle.c",
                "libridley.a",
                "ridley-services-sm_80.c",
                "ridley-services-sm_86.c",
                "ridley-services-sm_90.c",
                "ridley-services-sm_a0.c",
            ],
            "ridley",
        ),
        _ => (vec!["oracle.c"], "c"),
    };
    (generated, vec![binary], full_c(recipe))
}

pub(super) fn abi(recipe: Option<&NativeRecipe>) -> bool {
    matches!(recipe, Some(NativeRecipe::Plm | NativeRecipe::Ridley))
}

pub(super) fn fixture_program(
    prepared: &PreparedNative,
    run: &Path,
    directory: &Path,
    label: &str,
    helper: &Path,
) -> Result<translation_execution::confinement::Program> {
    use builds::StrictFixture as K;
    let recipe = prepared.contract.recipe.as_ref();
    if label == "abi-controls" {
        let (command, kind) = match recipe {
            Some(NativeRecipe::Plm) => (
                vec![
                    run.join("spawn").display().to_string(),
                    "abi".into(),
                    "0".into(),
                    "0".into(),
                    "reserved".into(),
                    "reserved".into(),
                    run.join("rom.bin").display().to_string(),
                    "reserved".into(),
                ],
                K::PlmAbi,
            ),
            Some(NativeRecipe::Ridley) => {
                (vec![run.join("ridley").display().to_string(), "--abi-controls".into()], K::RidleyAbi)
            }
            _ => anyhow::bail!("unexpected native ABI operation"),
        };
        return builds::strict_fixture_program(helper, &command, kind);
    }
    let (routine, fixture, index) = family::ordered_tasks(&prepared.manifest)?
        .into_iter()
        .find(|(_, fixture, _)| run.join(&fixture.name) == directory)
        .context("undeclared native fixture")?;
    let text = |path: PathBuf| path.display().to_string();
    let before = text(directory.join("before.wram"));
    let after = text(directory.join(format!("{label}-after.wram")));
    let rom = text(run.join("rom.bin"));
    let a = fixture.a.to_string();
    let b = fixture.b.to_string();
    let (command, kind) = match recipe {
        Some(NativeRecipe::Callback | NativeRecipe::PowerBomb | NativeRecipe::GameHost) => (
            vec![
                text(run.join("collision")),
                label.into(),
                routine.symbol.clone(),
                a,
                b,
                before,
                after,
                index.to_string(),
                rom,
                text(directory.join(label)),
            ],
            K::Callback,
        ),
        Some(NativeRecipe::Plm) => (
            vec![
                text(run.join("spawn")),
                label.into(),
                a,
                b,
                before,
                after,
                rom,
                match routine.symbol.as_str() {
                    "terrain" => "0",
                    "room" => "1",
                    "hardcoded" => "2",
                    _ => anyhow::bail!("unknown native allocator"),
                }
                .into(),
            ],
            K::Plm,
        ),
        Some(NativeRecipe::Ridley) => (
            vec![
                text(run.join(label)),
                label.into(),
                routine.symbol.clone(),
                a,
                b,
                before,
                after,
                "0".into(),
                rom,
                text(directory.join("host-before.bin")),
                text(directory.join(format!("{label}-host-after.bin"))),
                text(directory.join(format!("{label}-effects.log"))),
                text(directory.join(format!("{label}-ppu.bin"))),
                text(directory.join(format!("{label}-services.bin"))),
                text(directory.join("ppu-before.bin")),
            ],
            K::Ridley,
        ),
        _ => {
            let mut command = vec![text(run.join(label)), routine.symbol.clone(), a, b, before, after];
            if prepared.manifest.rom.is_some() {
                command.push(rom);
            }
            if prepared.manifest.projectile_host {
                command.extend([
                    text(directory.join("host-before.bin")),
                    text(directory.join(format!("{label}-host-after.bin"))),
                ]);
            }
            (command, K::Family)
        }
    };
    builds::strict_fixture_program(helper, &command, kind)
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
pub(crate) fn contract_for_direct_animation(
    checkout: &Path,
    plan: &Path,
    source_inventory: Asset,
    provenance: NativeProvenance,
    compiler_limits: NativeLimits,
    fixture_limits: NativeLimits,
) -> Result<NativeContract> {
    bounded::limited(|| {
        ensure!(relative(plan), "direct animation plan must be a frozen relative input");
        let recipe = NativeRecipe::DirectAnimation {
            plan: Asset { file: plan.into(), sha256: digest(&file(&checkout.join(plan))?)? },
        };
        contract_for_declared(
            checkout,
            Path::new("tools/port/pause-equipment-render.toml"),
            source_inventory,
            provenance,
            compiler_limits,
            fixture_limits,
            Some(recipe),
        )
    })
}

fn direct_manifest(checkout: &Path, original: &Manifest, plan: &Asset) -> Result<Manifest> {
    let plan: AnimationPlan = family::object(&check_asset(checkout, plan)?)?;
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
                            writes,
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
