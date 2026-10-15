//! Native terrain, room and hardcoded allocation with synchronous amount checks.
use crate::family_model::*;
use crate::{atomic_toml, builds, digest, family, inputs};
use anyhow::{Context, Result, ensure};
use std::{collections::BTreeMap, fs, path::Path};

pub fn normalize(manifest: &Manifest) -> Result<Manifest> {
    let mut normalized = manifest.clone();
    let mut groups = BTreeMap::<String, Vec<Case>>::new();
    for case in &manifest.cases {
        let kind = case.allocator.as_deref().unwrap_or("terrain");
        ensure!(["terrain", "room", "hardcoded"].contains(&kind), "invalid allocator");
        groups.entry(kind.into()).or_default().push(case.clone());
    }
    normalized.routines = groups
        .into_iter()
        .map(|(symbol, cases)| Routine {
            symbol,
            arg_types: Some(vec!["uint16".into(), "int32".into()]),
            returns: "uint32".into(),
            cases,
            ..Default::default()
        })
        .collect();
    Ok(normalized)
}

fn rust_wrapper(memory: &Path, module: &Path, confined: bool) -> String {
    let mut wrapper = include_str!("plm_spawn_abi.rs.in")
        .replace("__MEMORY__", &format!("{memory:?}"))
        .replace("__MODULE__", &format!("{module:?}"));
    if confined {
        let module_name = module.file_stem().unwrap().to_string_lossy();
        let binding = if module_name == "plm" { String::new() } else { format!("use {module_name} as plm;") };
        wrapper = wrapper
            .replace(
                &format!("#[path={memory:?}]mod memory;use memory::*;"),
                "include!(\"/inputs/generated/memory.rs\");",
            )
            .replace(&format!("#[path={module:?}]mod plm;"), &binding);
    }
    wrapper
}

pub(crate) fn build(
    recipe: Option<&crate::recipe::Recipe>,
    request: &VerifyRequest,
    manifest: &Manifest,
    checkout: &Path,
    source: &Path,
    run: &Path,
    execution: Option<&builds::StrictExecution>,
) -> Result<Implementation> {
    let module = fs::canonicalize(family::source_path(checkout, &manifest.rust_module))?;
    let memory = if let Some(execution) = execution {
        inputs::canonical_wrapper(checkout, run, &execution.rust_inputs)?
    } else {
        inputs::wrapper(crate::authoring::WrapperRequest {
            checkout: checkout.into(),
            run: run.into(),
            modules: vec![module.clone()],
            imports: vec![],
        })?
    };
    let wrapper = rust_wrapper(&memory.path, &module, execution.is_some());
    family::write_build_input(&run.join("wrapper.rs"), wrapper.as_bytes(), execution.is_some())?;
    let headless = if execution.is_some() {
        "/inputs/rust/tools/decomp/headless.c".into()
    } else {
        checkout.join("tools/decomp/headless.c")
    };
    family::write_build_input(
        &run.join("oracle.c"),
        include_str!("plm_spawn_host.c.in").replace("__HEADLESS__", &format!("{headless:?}")).as_bytes(),
        execution.is_some(),
    )?;
    let root = &request.root;
    let workers = request.workers;
    let description = crate::recipe::rust_compiler_command(recipe, run);
    if let Some(execution) = execution {
        execution.compile_description(&description, run, "rust-build", checkout, source)?;
    } else {
        tool_execution::builds::execute(&description.command, run, "rust-build", root, workers)?;
    }
    let description =
        crate::recipe::c_compiler_command(recipe, manifest, source, run, &decomp_source::c_units(source)?, false)?;
    if let Some(execution) = execution {
        execution.compile_description(&description, run, "c-build", checkout, source)?;
    } else {
        crate::builds::ordinary_c_build(&description, run, checkout, root, workers)?;
    }
    let fixture = crate::recipe::fixture_command(
        recipe,
        manifest,
        crate::recipe::FixtureSelection::AbiControls,
        run,
        "abi-controls",
    )?;
    if let Some(execution) = execution {
        execution.fixture_recipe(&fixture.command, run, "abi-controls", fixture.kind)?;
    } else {
        tool_execution::builds::execute(&fixture.command, run, "abi-controls", root, workers)?;
    }
    fs::copy(&module, run.join("candidate.rs"))?;
    let dependency_request = crate::authoring::DependenciesRequest {
        checkout: checkout.into(),
        run: run.into(),
        extra: manifest.rust_dependencies.clone(),
    };
    let dependencies = if execution.is_some() {
        inputs::confined_dependencies(dependency_request)?
    } else {
        inputs::dependencies(dependency_request)?
    };
    let mut harness = inputs::implementation_sources(checkout, run, execution.is_some())?;
    let key = "tools/decomp/headless.c";
    harness.insert(key.to_owned(), digest(&checkout.join(key))?);
    fs::copy(checkout.join(key), run.join("pipeline-headless.c"))?;
    Ok(Implementation {
        fixture_execution: Some("dependency-v1-fresh".into()),
        dependency_contract: "rust-plm-v2".into(),
        source_root: checkout.into(),
        rust_dependencies: dependencies,
        harness_sources: harness,
        module: module.clone(),
        module_sha256: digest(&module)?,
        memory_sha256: digest(&checkout.join("crates/port/src/memory.rs"))?,
        binaries: BTreeMap::from([
            ("spawn".into(), digest(&run.join("spawn"))?),
            ("libplm.a".into(), digest(&run.join("libplm.a"))?),
        ]),
        ..Default::default()
    })
}

pub fn check(directory: &Path, fixture: &Case) -> Result<CaseRecord> {
    let mut results = BTreeMap::new();
    for subject in ["c", "rust"] {
        let text = fs::read_to_string(directory.join(format!("{subject}.stdout")))?;
        let values = text
            .lines()
            .last()
            .context("PLM metrics missing")?
            .split_whitespace()
            .map(str::parse::<i64>)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(values.len() == 4, "PLM metric extent");
        results.insert(
            subject.into(),
            SpawnMetrics {
                flags: values[0],
                amount: values[1],
                cpu_fallback_calls: u64::try_from(values[2])?,
                cpu_fallback_instructions: u64::try_from(values[3])?,
            },
        );
    }
    ensure!(results["c"] == results["rust"], "native SpawnPLM flags/amount mismatch");
    let rust = &results["rust"];
    ensure!(rust.cpu_fallback_calls == 0 && rust.cpu_fallback_instructions == 0, "SpawnPLM CPU fallback");
    if let Some(expected) = fixture.expected_amount {
        ensure!(expected == rust.amount, "independent amount mismatch");
    }
    if let Some(expected) = fixture.expected_flags {
        ensure!(expected == rust.flags, "independent flags mismatch");
    }
    let before = fs::read(directory.join("before.wram"))?;
    let c = fs::read(directory.join("c-after.wram"))?;
    let rust = fs::read(directory.join("rust-after.wram"))?;
    ensure!(before.len() == 0x20000 && c.len() == 0x20000 && rust.len() == 0x20000, "PLM full WRAM extent");
    if c != rust {
        let offset = c.iter().zip(&rust).position(|(a, b)| a != b).unwrap();
        atomic_toml(
            &directory.join("rejection.toml"),
            &CaseRecord {
                first_mismatch: Some(Mismatch { offset, c: c[offset], rust: rust[offset] }),
                ..Default::default()
            },
        )?;
        anyhow::bail!("WRAM mismatch at {offset:#x}");
    }
    if fixture.expected_after_file.as_ref().is_some() {
        ensure!(c == fs::read(directory.join("expected-after.wram"))?, "captured PLM WRAM mismatch");
    }
    Ok(CaseRecord { spawn_results: results, ..Default::default() })
}

pub(crate) fn case(
    routine: &Routine,
    fixture: &Case,
    run: &Path,
    resume: bool,
    mut execute: impl FnMut(&Path, &str) -> Result<()>,
) -> Result<CaseRecord> {
    let name = fixture.name.as_str();
    let kind = routine.symbol.as_str();
    let directory = run.join(name);
    family::prepare_case_with(fixture, &directory, &["uint16".into(), "int32".into()], resume)?;
    for subject in ["rust", "c"] {
        execute(&directory, subject)?;
    }
    let mut record = check(&directory, fixture)?;
    record.name = name.into();
    record.routine = kind.into();
    Ok(record)
}

#[cfg(test)]
#[path = "plm_spawn_tests.rs"]
mod tests;
