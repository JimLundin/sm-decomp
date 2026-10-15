//! Native Ridley fixture adapter with explicit C services and hardware effects.
//!
//! The coordinator registers this adapter with `family`. All case scheduling,
//! worker leases and fresh-process isolation stay in the maintained executor.
use crate::family_model::{Asset, CallbackMetrics, Case, CaseRecord, Implementation, Manifest, Routine, VerifyRequest};
use crate::{atomic_toml, builds, digest, family, inputs};
use anyhow::{Context, Result, bail, ensure};
use std::{collections::BTreeMap, fs, path::Path};

const WRAM: usize = 0x20000;
const CAPTURE_SUFFIXES: [&str; 7] =
    ["before.host", "before.ppu", "after.wram", "after.host", "after.ppu", "effects.log", "services.bin"];
pub(crate) const CONTRACT: &str = "rust-ridley-subsystem-v1";

fn capture_path(fixture: &Case, suffix: &str) -> Result<std::path::PathBuf> {
    let path = fixture.before_file.as_ref().context("Ridley capture input missing")?;
    let stem = path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix("-before.wram"))
        .context("Ridley capture input must end in -before.wram")?;
    Ok(path.with_file_name(format!("{stem}-{suffix}")))
}

fn validate_capture_assets(manifest: &Manifest, checkout: &Path) -> Result<()> {
    let declared = manifest
        .rust_dependencies
        .iter()
        .map(|path| fs::canonicalize(if path.is_absolute() { path.clone() } else { checkout.join(path) }))
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    for fixture in manifest.routines.iter().flat_map(|routine| &routine.cases).filter(|case| case.before_file.is_some())
    {
        ensure!(fixture.continue_from.is_none(), "Ridley capture replaces a chained input");
        let after = capture_path(fixture, "after.wram")?;
        ensure!(
            fixture.expected_after_file.as_ref().is_some_and(|path| path == &after)
                && fixture.expected_after_sha256.is_some(),
            "Ridley captured output declaration missing"
        );
        for suffix in CAPTURE_SUFFIXES {
            let path = capture_path(fixture, suffix)?;
            ensure!(
                declared.contains(&fs::canonicalize(&path)?),
                "Ridley capture dependency undeclared: {}",
                path.display()
            );
        }
    }
    Ok(())
}

fn instrument_units(source: &Path) -> Result<BTreeMap<String, String>> {
    let mut units = BTreeMap::new();
    for bank in ["sm_a0.c", "sm_90.c", "sm_80.c", "sm_86.c"] {
        let mut unit = fs::read_to_string(source.join("src").join(bank))?;
        let mut functions = Vec::new();
        match bank {
            "sm_a0.c" => {
                for (signature, symbol) in [
                    ("uint16 SpawnEnemy(uint8 db, uint16 k)", "SpawnEnemy"),
                    ("void EnemyDeathAnimation(uint16 k, uint16 a)", "EnemyDeathAnimation"),
                    ("void Enemy_ItemDrop_Ridley(uint16 k)", "Enemy_ItemDrop_Ridley"),
                    ("void NormalEnemyTouchAiSkipDeathAnim_CurEnemy(void)", "NormalEnemyTouchAiSkipDeathAnim_CurEnemy"),
                    ("void NormalEnemyShotAiSkipDeathAnim_CurEnemy(void)", "NormalEnemyShotAiSkipDeathAnim_CurEnemy"),
                    (
                        "void NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(void)",
                        "NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy",
                    ),
                ] {
                    functions.push((signature.into(), symbol.into()));
                }
            }
            "sm_90.c" => functions.push(("uint16 CallSomeSamusCode(uint16 a)".into(), "CallSomeSamusCode".into())),
            "sm_80.c" => {
                for channel in 1..=3 {
                    for maximum in [1, 3, 6, 9, 15] {
                        let symbol = format!("QueueSfx{channel}_Max{maximum}");
                        functions.push((format!("void {symbol}(uint16 a)"), symbol));
                    }
                }
                functions.push(("void QueueMusic_Delayed8(uint16 a)".into(), "QueueMusic_Delayed8".into()));
            }
            "sm_86.c" => functions
                .push(("void SpawnEprojWithRoomGfx(uint16 j, uint16 a)".into(), "SpawnEprojWithRoomGfx".into())),
            _ => unreachable!(),
        }
        for (signature, symbol) in functions {
            let definition = format!("{signature} {{");
            ensure!(unit.matches(&definition).count() == 1, "Ridley service definition must be unique: {symbol}");
            unit = unit.replacen(
                &definition,
                &format!("{} {{", signature.replacen(&symbol, &format!("RidleyActual_{symbol}"), 1)),
                1,
            );
        }
        units.insert(bank.into(), unit);
    }
    Ok(units)
}

/// Reconstructs the exact actual-C units while preserving every public call site.
pub(crate) fn check_instrumented_units(source: &Path, run: &Path) -> Result<()> {
    for (bank, expected) in instrument_units(source)? {
        ensure!(
            fs::read_to_string(run.join(format!("ridley-services-{bank}")))? == expected,
            "Ridley service source changed: {bank}"
        );
    }
    Ok(())
}

fn branches(manifest: &Manifest) -> Result<String> {
    let mut branches = Vec::new();
    for routine in &manifest.routines {
        let symbol = &routine.symbol;
        let args = routine.argument_types()?;
        ensure!(
            family::identifier(symbol) && args.len() == routine.args && args.len() <= 2,
            "invalid Ridley signature"
        );
        let arguments = args
            .iter()
            .zip(["a", "b"])
            .map(|(kind, name)| {
                if kind == "rom_ptr" {
                    Ok(format!("(const uint16 *)RomPtr((uint32){name})"))
                } else {
                    Ok(format!("({}){name}", family::c_type(kind)?))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        let call = format!("{symbol}({})", arguments.join(","));
        let statement = match routine.returns.as_str() {
            "void" => format!("{call};result=0;"),
            "rom_ptr" => format!("result=RidleyPointer({call},0xa60000);"),
            kind => {
                family::c_type(kind)?;
                format!("result=(uint32){call};")
            }
        };
        branches.push(format!("if(!strcmp(symbol,\"{symbol}\")){{{statement}}}"));
    }
    ensure!(!branches.is_empty(), "empty Ridley adapter");
    Ok(branches.join("else "))
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
    ensure!(cfg!(all(target_os = "linux", target_arch = "x86_64")), "Ridley fixture worker requires Linux x86_64");
    ensure!(manifest.projectile_host && manifest.rom.is_some(), "Ridley fixtures require canonical ROM and host words");
    ensure!(manifest.callback_adapter.as_deref() == Some("ridley-subsystem"), "Ridley adapter declaration missing");
    validate_capture_assets(manifest, checkout)?;
    let module = fs::canonicalize(family::source_path(checkout, &manifest.rust_module))?;
    let child_modules = [
        "ridley_movement",
        "ridley_rendering",
        "ridley_cutscene",
        "ridley_subsystem_instructions",
        "ridley_subsystem_admission",
    ];
    let memory = if let Some(execution) = execution {
        let roots = execution
            .rust_inputs
            .iter()
            .filter(|path| path.file_stem().is_none_or(|name| !child_modules.iter().any(|child| name == *child)))
            .cloned()
            .collect::<Vec<_>>();
        inputs::canonical_wrapper(checkout, run, &roots)?
    } else {
        inputs::wrapper(crate::authoring::WrapperRequest {
            checkout: checkout.into(),
            run: run.into(),
            modules: vec![module.clone()],
            imports: manifest.rust_imports.clone(),
        })?
    };
    // These files are children of the subsystem module. Lexical discovery sees
    // their path attributes as import names, but their `super` is the Ridley
    // context, so they must never be emitted as independent crate modules.
    let discovered = fs::read_to_string(&memory.path)?;
    let child_modules = [
        "ridley_movement",
        "ridley_rendering",
        "ridley_cutscene",
        "ridley_subsystem_instructions",
        "ridley_subsystem_admission",
    ];
    let mut scoped = String::new();
    for line in discovered.lines() {
        if !child_modules.iter().any(|name| line.ends_with(&format!("pub mod {name};"))) {
            scoped.push_str(line);
            scoped.push('\n');
        }
    }
    family::write_build_input(&memory.path, scoped.as_bytes(), execution.is_some())?;
    let imports = manifest
        .rust_imports
        .iter()
        .map(|name| {
            ensure!(family::identifier(name), "invalid Ridley import");
            Ok(format!("use memory::{name};\n"))
        })
        .collect::<Result<String>>()?;
    let abi = include_str!("ridley_subsystem_abi.rs.in").replace("__MODULE__", &format!("{module:?}"));
    let wrapper = if execution.is_some() {
        format!(
            "include!(\"/inputs/generated/memory.rs\");\n#[path=\"/inputs/generated/fixture-worker.rs\"] mod fixture_worker;\n{}",
            abi.replace(
                &format!("#[path={module:?}]\nmod family;"),
                &format!("use {} as family;", module.file_stem().unwrap().to_string_lossy())
            )
        )
    } else {
        format!(
            "#[path={:?}] mod memory;use memory::*;\n{imports}#[path={:?}] mod fixture_worker;\n{abi}",
            memory.path,
            checkout.join("crates/offline-verification/src/fixture_worker.rs")
        )
    };
    family::write_build_input(&run.join("wrapper.rs"), wrapper.as_bytes(), execution.is_some())?;
    if execution.is_some() {
        tool_execution::bounded::immutable_bytes(&run.join("fixture-worker.rs"), include_bytes!("fixture_worker.rs"))?;
    }
    let headless = if execution.is_some() {
        "/inputs/rust/tools/decomp/headless.c".into()
    } else {
        checkout.join("tools/decomp/headless.c")
    };
    family::write_build_input(
        &run.join("oracle.c"),
        include_str!("ridley_subsystem_host.c.in")
            .replace("__HEADLESS__", &format!("{headless:?}"))
            .replace("__BRANCHES__", &branches(manifest)?)
            .as_bytes(),
        execution.is_some(),
    )?;
    let instrumented = instrument_units(source)?;
    for (bank, code) in &instrumented {
        family::write_build_input(&run.join(format!("ridley-services-{bank}")), code.as_bytes(), execution.is_some())?;
    }
    let description = crate::recipe::rust_compiler_command(recipe, run);
    if let Some(execution) = execution {
        execution.compile_description(&description, run, "rust-build", checkout, source)?;
    } else {
        tool_execution::builds::execute(&description.command, run, "rust-build", &request.root, request.workers)?;
    }
    let description =
        crate::recipe::c_compiler_command(recipe, manifest, source, run, &decomp_source::c_units(source)?, false)?;
    if let Some(execution) = execution {
        execution.compile_description(&description, run, "c-build", checkout, source)?;
    } else {
        crate::builds::ordinary_c_build(&description, run, checkout, &request.root, request.workers)?;
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
        tool_execution::builds::execute(&fixture.command, run, "abi-controls", &request.root, request.workers)?;
    }
    // The executor preloads these separately. Each fixture still forks a fresh
    // child from its respective native image; the helper never invokes gameplay.
    for subject in ["c", "rust"] {
        family::write_build_input(
            &run.join(subject),
            &tool_execution::bounded::read(run.join("ridley"))?,
            execution.is_some(),
        )?;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(run.join(subject), fs::Permissions::from_mode(0o500))?;
    }
    fs::copy(&module, run.join("candidate.rs"))?;
    fs::copy(checkout.join("crates/offline-verification/src/fixture_worker.rs"), run.join("fixture-worker.rs"))?;
    let dependency_request = crate::authoring::DependenciesRequest {
        checkout: checkout.into(),
        run: run.into(),
        extra: manifest.rust_dependencies.clone(),
    };
    let dependencies = if inputs::captured_ridley_manifest(manifest) {
        inputs::captured_ridley_dependencies(dependency_request, manifest, execution.is_some())?
    } else if execution.is_some() {
        inputs::confined_dependencies(dependency_request)?
    } else {
        inputs::dependencies(dependency_request)?
    };
    let mut harness = inputs::implementation_sources(checkout, run, execution.is_some())?;
    let headless = "tools/decomp/headless.c";
    harness.insert(headless.into(), digest(&checkout.join(headless))?);
    fs::copy(checkout.join(headless), run.join("pipeline-headless.c"))?;
    Ok(Implementation {
        fixture_execution: Some(
            if request.execution == "fresh" { "dependency-v1-fresh" } else { "dependency-v1-fork" }.into(),
        ),
        dependency_contract: CONTRACT.into(),
        callback_adapter: Some("ridley-subsystem".into()),
        source_root: checkout.into(),
        rust_dependencies: dependencies,
        harness_sources: harness,
        module_sha256: digest(&module)?,
        memory_sha256: digest(&checkout.join("crates/port/src/memory.rs"))?,
        module,
        binaries: ["ridley", "c", "rust", "libridley.a"]
            .into_iter()
            .map(|name| Ok((name.into(), digest(&run.join(name))?)))
            .collect::<Result<_>>()?,
        ..Default::default()
    })
}

fn host_before(fixture: &Case, run: &Path) -> Result<Vec<u8>> {
    if let Some(previous) = &fixture.continue_from {
        ensure!(fixture.host_words.is_none(), "Ridley continuation must inherit host words");
        let c = fs::read(run.join(previous).join("c-host-after.bin"))?;
        let rust = fs::read(run.join(previous).join("rust-host-after.bin"))?;
        ensure!(c.len() == 16 && c == rust, "Ridley predecessor host state differs");
        Ok(c)
    } else {
        let words = fixture.host_words.as_ref().context("Ridley initial host state missing")?;
        ensure!(words.len() == 8, "Ridley host requires exactly eight words");
        Ok(words.iter().flat_map(|word| word.to_le_bytes()).collect())
    }
}

fn ppu_before(fixture: &Case, run: &Path) -> Result<Vec<u8>> {
    if let Some(previous) = &fixture.continue_from {
        let c = fs::read(run.join(previous).join("c-ppu.bin"))?;
        let rust = fs::read(run.join(previous).join("rust-ppu.bin"))?;
        ensure!(c.len() == 20 && c == rust, "Ridley predecessor hardware state differs");
        Ok(c)
    } else if fixture.before_file.is_some() {
        let captured = fs::read(capture_path(fixture, "before.ppu")?)?;
        ensure!(captured.len() == 20, "Ridley captured hardware input extent differs");
        Ok(captured)
    } else {
        Ok(vec![0; 20])
    }
}

fn retain_capture(directory: &Path, fixture: &Case) -> Result<()> {
    if fixture.before_file.is_none() {
        return Ok(());
    }
    let mut assets = BTreeMap::new();
    for suffix in CAPTURE_SUFFIXES {
        let source = capture_path(fixture, suffix)?;
        let hash = digest(&source)?;
        let retained = directory.join(format!("captured-{suffix}"));
        fs::copy(&source, &retained)?;
        ensure!(digest(&retained)? == hash, "Ridley capture changed during retention");
        assets.insert(suffix, Asset { file: fs::canonicalize(source)?, sha256: hash });
    }
    atomic_toml(&directory.join("capture-inputs.toml"), &assets)
}

fn authenticate_capture(directory: &Path, fixture: &Case) -> Result<()> {
    if fixture.before_file.is_none() {
        ensure!(!directory.join("capture-inputs.toml").exists(), "synthetic Ridley fixture gained a capture");
        return Ok(());
    }
    let assets: BTreeMap<String, Asset> = toml::from_str(&fs::read_to_string(directory.join("capture-inputs.toml"))?)?;
    ensure!(assets.len() == CAPTURE_SUFFIXES.len(), "Ridley capture companion inventory differs");
    for suffix in CAPTURE_SUFFIXES {
        let asset = assets.get(suffix).context("Ridley captured companion missing")?;
        ensure!(
            asset.file == fs::canonicalize(capture_path(fixture, suffix)?)?,
            "Ridley capture companion path differs"
        );
        ensure!(
            digest(&asset.file)? == asset.sha256
                && digest(&directory.join(format!("captured-{suffix}")))? == asset.sha256,
            "Ridley capture companion bytes differ"
        );
    }
    ensure!(
        fs::read(directory.join("captured-before.host"))? == fs::read(directory.join("host-before.bin"))?,
        "Ridley captured host input differs"
    );
    for (suffix, output) in [
        ("after.wram", "after.wram"),
        ("after.host", "host-after.bin"),
        ("after.ppu", "ppu.bin"),
        ("effects.log", "effects.log"),
        ("services.bin", "services.bin"),
    ] {
        let expected = fs::read(directory.join(format!("captured-{suffix}")))?;
        ensure!(expected == fs::read(directory.join(format!("c-{output}")))?, "Ridley natural {suffix} differs");
    }
    Ok(())
}

fn authenticate_before(directory: &Path, fixture: &Case, before: &[u8]) -> Result<()> {
    let run = directory.parent().context("Ridley fixture parent missing")?;
    if let Some(previous) = &fixture.continue_from {
        ensure!(fixture.before_file.is_none() && fixture.fill.is_none(), "Ridley continuation replaced WRAM");
        let mut expected = fs::read(run.join(previous).join("c-after.wram"))?;
        ensure!(expected == fs::read(run.join(previous).join("rust-after.wram"))?, "Ridley predecessor WRAM differs");
        apply_writes(&mut expected, fixture)?;
        ensure!(before == expected, "Ridley continuation did not inherit full WRAM");
    } else if let Some(file) = &fixture.before_file {
        ensure!(fixture.writes.is_empty() && fixture.fill.is_none(), "captured Ridley fixture was patched");
        ensure!(
            digest(file)? == fixture.before_sha256.as_deref().context("Ridley captured input hash missing")?
                && before == fs::read(file)?,
            "Ridley captured input differs"
        );
    } else {
        let mut expected = vec![fixture.fill.unwrap_or(0); WRAM];
        apply_writes(&mut expected, fixture)?;
        ensure!(before == expected, "Ridley fixture input differs");
    }
    ensure!(
        fs::read(directory.join("host-before.bin"))? == host_before(fixture, run)?,
        "Ridley fixture host input differs"
    );
    ensure!(
        fs::read(directory.join("ppu-before.bin"))? == ppu_before(fixture, run)?,
        "Ridley fixture hardware input differs"
    );
    Ok(())
}

fn apply_writes(bytes: &mut [u8], fixture: &Case) -> Result<()> {
    ensure!(bytes.len() == WRAM, "Ridley predecessor WRAM extent differs");
    for write in &fixture.writes {
        let end = write
            .offset
            .checked_add(write.data.len())
            .filter(|end| *end <= WRAM)
            .context("Ridley fixture write outside WRAM")?;
        bytes[write.offset..end].copy_from_slice(&write.data);
    }
    Ok(())
}

fn metrics(directory: &Path, subject: &str) -> Result<CallbackMetrics> {
    let text = fs::read_to_string(directory.join(format!("{subject}.stdout")))?;
    let values = text
        .lines()
        .last()
        .context("Ridley subject metrics missing")?
        .split_whitespace()
        .map(str::parse::<u64>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(values.len() == 4, "Ridley metrics extent differs");
    Ok(CallbackMetrics {
        returns: values[0],
        c_gameplay_callbacks: values[1],
        cpu_fallback_calls: values[2],
        cpu_fallback_instructions: values[3],
    })
}

fn authenticate_services(bytes: &[u8], effects: &str, expected_calls: u64) -> Result<()> {
    const RECORD: usize = 20 + WRAM + 16;
    let (records, remainder) = bytes.as_chunks::<RECORD>();
    ensure!(remainder.is_empty(), "Ridley partial service snapshot");
    let mut stack = Vec::new();
    let mut entries = Vec::new();
    for record in records {
        let fields = record[..20].as_chunks::<4>().0.iter().map(|value| u32::from_le_bytes(*value)).collect::<Vec<_>>();
        let [phase, kind, a, b, returned] = fields.as_slice() else { unreachable!() };
        ensure!(*kind <= 6 && *a <= u16::MAX.into() && *b <= u16::MAX.into(), "Ridley service signature differs");
        let key = (*kind, *a, *b);
        match phase {
            0 => {
                ensure!(*returned == 0, "Ridley entry service already returned");
                stack.push(key);
                entries.push(key);
            }
            1 => {
                ensure!(stack.pop() == Some(key), "Ridley service return order or signature differs");
                ensure!(
                    if matches!(*kind, 0 | 6) { *returned <= u16::MAX.into() } else { *returned == 0 },
                    "Ridley service return extent differs"
                );
            }
            _ => bail!("Ridley service phase differs"),
        }
    }
    ensure!(stack.is_empty() && entries.len() as u64 == expected_calls, "Ridley service transactions incomplete");
    let ordered = effects
        .lines()
        .filter(|line| line.starts_with("service "))
        .map(|line| {
            let values =
                line.split_whitespace().skip(1).map(str::parse::<u32>).collect::<std::result::Result<Vec<_>, _>>()?;
            let [kind, a, b] = values.as_slice() else { bail!("Ridley ordered service record extent differs") };
            Ok((*kind, *a, *b))
        })
        .collect::<Result<Vec<_>>>()?;
    ensure!(entries == ordered, "Ridley service snapshots and ordered requests differ");
    Ok(())
}

/// Rechecks retained case outputs, independent expectations and predecessor state.
pub fn check(directory: &Path, fixture: &Case) -> Result<CaseRecord> {
    let before = fs::read(directory.join("before.wram"))?;
    ensure!(before.len() == WRAM, "Ridley input WRAM extent differs");
    authenticate_before(directory, fixture, &before)?;
    let c = metrics(directory, "c")?;
    let rust = metrics(directory, "rust")?;
    ensure!(c == rust, "Ridley return/callback/fallback metrics differ");
    ensure!(rust.cpu_fallback_calls == 0 && rust.cpu_fallback_instructions == 0, "Ridley fixture invoked CPU fallback");
    ensure!(rust.c_gameplay_callbacks >= fixture.minimum_callbacks, "Ridley external service coverage missing");
    if let Some(expected) = fixture.expected_return {
        ensure!(rust.returns == expected, "Ridley independent return differs");
    }
    let c_state = fs::read(directory.join("c-after.wram"))?;
    let rust_state = fs::read(directory.join("rust-after.wram"))?;
    ensure!(c_state.len() == WRAM && rust_state.len() == WRAM, "Ridley full output WRAM missing");
    let mut record = CaseRecord {
        name: fixture.name.clone(),
        before_sha256: digest(&directory.join("before.wram"))?,
        after_sha256: ["c", "rust"]
            .into_iter()
            .map(|subject| Ok((subject.into(), digest(&directory.join(format!("{subject}-after.wram")))?)))
            .collect::<Result<_>>()?,
        returns: BTreeMap::from([("c".into(), c.returns), ("rust".into(), rust.returns)]),
        callbacks: rust.c_gameplay_callbacks,
        results: BTreeMap::from([("c".into(), c), ("rust".into(), rust)]),
        ..Default::default()
    };
    if c_state != rust_state {
        let offset = c_state.iter().zip(&rust_state).position(|(c, rust)| c != rust).expect("different WRAM");
        record.first_mismatch =
            Some(crate::family_model::Mismatch { offset, c: c_state[offset], rust: rust_state[offset] });
        atomic_toml(&directory.join("rejection.toml"), &record)?;
        bail!("Ridley WRAM mismatch at {offset:#x}");
    }
    for (suffix, extent) in
        [("host-after.bin", Some(16)), ("ppu.bin", Some(20)), ("effects.log", None), ("services.bin", None)]
    {
        let c = fs::read(directory.join(format!("c-{suffix}")))?;
        let rust = fs::read(directory.join(format!("rust-{suffix}")))?;
        if let Some(extent) = extent {
            ensure!(c.len() == extent && rust.len() == extent, "Ridley {suffix} extent differs");
        }
        ensure!(c == rust, "Ridley ordered hardware/service or external state differs: {suffix}");
        if suffix == "services.bin" {
            authenticate_services(&c, &fs::read_to_string(directory.join("c-effects.log"))?, record.callbacks)?;
        }
    }
    if fixture.expected_after_file.is_some() {
        let path = directory.join("expected-after.wram");
        let hash = digest(&path)?;
        ensure!(Some(hash.as_str()) == fixture.expected_after_sha256.as_deref(), "Ridley captured output hash differs");
        ensure!(c_state == fs::read(&path)?, "Ridley captured full WRAM output differs");
        record.capture_after_sha256 = Some(hash);
    }
    authenticate_capture(directory, fixture)?;
    Ok(record)
}

pub(crate) fn case(
    routine: &Routine,
    fixture: &Case,
    run: &Path,
    resume: bool,
    mut execute: impl FnMut(&Path, &str) -> Result<()>,
) -> Result<CaseRecord> {
    let directory = run.join(&fixture.name);
    family::prepare_case_with(fixture, &directory, &routine.argument_types()?, resume)?;
    fs::write(directory.join("host-before.bin"), host_before(fixture, run)?)?;
    fs::write(directory.join("ppu-before.bin"), ppu_before(fixture, run)?)?;
    retain_capture(&directory, fixture)?;
    for subject in ["rust", "c"] {
        execute(&directory, subject)?;
    }
    let mut record = check(&directory, fixture)?;
    record.routine = routine.symbol.clone();
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Retained(std::path::PathBuf);
    impl Drop for Retained {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn retained() -> (Retained, Case) {
        let root = std::env::temp_dir().join(format!(
            "sm-ridley-adapter-test-{}-{}",
            std::process::id(),
            tool_execution::unique_id()
        ));
        let dir = root.join("frame");
        fs::create_dir_all(&dir).unwrap();
        let fixture = Case { name: "frame".into(), host_words: Some(vec![0; 8]), ..Default::default() };
        fs::write(dir.join("before.wram"), vec![0; WRAM]).unwrap();
        fs::write(dir.join("host-before.bin"), vec![0; 16]).unwrap();
        fs::write(dir.join("ppu-before.bin"), vec![0; 20]).unwrap();
        for subject in ["c", "rust"] {
            for (suffix, bytes) in
                [("after.wram", WRAM), ("host-after.bin", 16), ("ppu.bin", 20), ("effects.log", 0), ("services.bin", 0)]
            {
                fs::write(dir.join(format!("{subject}-{suffix}")), vec![0; bytes]).unwrap();
            }
            fs::write(dir.join(format!("{subject}.stdout")), "native diagnostic retained\n0 0 0 0\n").unwrap();
        }
        (Retained(root), fixture)
    }
    #[test]
    fn host_hardware_and_service_differences_reject_equal_wram() {
        let (retained, fixture) = retained();
        let dir = retained.0.join("frame");
        assert!(check(&dir, &fixture).is_ok());
        for (suffix, original) in [
            ("host-after.bin", vec![0; 16]),
            ("ppu.bin", vec![0; 20]),
            ("effects.log", vec![]),
            ("services.bin", vec![]),
        ] {
            fs::write(dir.join(format!("rust-{suffix}")), [1]).unwrap();
            assert!(check(&dir, &fixture).is_err(), "accepted different {suffix}");
            fs::write(dir.join(format!("rust-{suffix}")), original).unwrap();
        }
        fs::write(dir.join("c.stdout"), "0 0 1 0\n").unwrap();
        fs::write(dir.join("rust.stdout"), "0 0 1 0\n").unwrap();
        assert!(check(&dir, &fixture).is_err(), "accepted matching CPU fallback");
    }

    #[test]
    fn agreeing_service_bytes_require_complete_native_ordered_transactions() {
        let (retained, fixture) = retained();
        let dir = retained.0.join("frame");
        let mut services = vec![0; 2 * (20 + WRAM + 16)];
        services[20 + WRAM + 16..24 + WRAM + 16].copy_from_slice(&1u32.to_le_bytes());
        for subject in ["c", "rust"] {
            fs::write(dir.join(format!("{subject}.stdout")), "0 1 0 0\n").unwrap();
            fs::write(dir.join(format!("{subject}-services.bin")), &services).unwrap();
            fs::write(dir.join(format!("{subject}-effects.log")), "service 0 0 0\n").unwrap();
        }
        assert!(check(&dir, &fixture).is_ok());
        for offset in [0, 4, 20 + WRAM + 16 + 8] {
            let mut malformed = services.clone();
            malformed[offset..offset + 4].copy_from_slice(&7u32.to_le_bytes());
            for subject in ["c", "rust"] {
                fs::write(dir.join(format!("{subject}-services.bin")), &malformed).unwrap();
            }
            assert!(check(&dir, &fixture).is_err(), "accepted agreeing malformed service headers");
        }
        for subject in ["c", "rust"] {
            fs::write(dir.join(format!("{subject}-services.bin")), &services).unwrap();
            fs::write(dir.join(format!("{subject}-effects.log")), "service 1 0 0\n").unwrap();
        }
        assert!(check(&dir, &fixture).is_err(), "accepted different ordered service identities");
    }
    #[test]
    fn equal_outputs_do_not_admit_a_patched_capture_or_broken_chain() {
        let (retained, mut fixture) = retained();
        let dir = retained.0.join("frame");
        let captured = retained.0.join("natural-before.wram");
        fs::write(&captured, vec![0; WRAM]).unwrap();
        fixture.before_sha256 = Some(digest(&captured).unwrap());
        fixture.before_file = Some(captured);
        for suffix in CAPTURE_SUFFIXES {
            let source = capture_path(&fixture, suffix).unwrap();
            let bytes = match suffix {
                "before.host" | "after.host" => vec![0; 16],
                "before.ppu" | "after.ppu" => vec![0; 20],
                "after.wram" => vec![0; WRAM],
                _ => vec![],
            };
            fs::write(source, bytes).unwrap();
        }
        fixture.expected_after_file = Some(capture_path(&fixture, "after.wram").unwrap());
        fixture.expected_after_sha256 = Some(digest(fixture.expected_after_file.as_ref().unwrap()).unwrap());
        fs::copy(fixture.expected_after_file.as_ref().unwrap(), dir.join("expected-after.wram")).unwrap();
        retain_capture(&dir, &fixture).unwrap();
        assert!(check(&dir, &fixture).is_ok());
        fixture.writes.push(crate::family_model::Write { offset: 0, data: vec![1] });
        assert!(check(&dir, &fixture).is_err());
        fixture.before_file = None;
        fixture.before_sha256 = None;
        fixture.expected_after_file = None;
        fixture.expected_after_sha256 = None;
        fixture.writes.clear();
        fixture.continue_from = Some("missing-predecessor".into());
        fixture.host_words = None;
        assert!(check(&dir, &fixture).is_err());
    }

    #[test]
    fn continuation_inherits_hardware_and_rejects_different_predecessors() {
        let (retained, mut fixture) = retained();
        let dir = retained.0.join("frame");
        let previous = retained.0.join("previous");
        fs::create_dir_all(&previous).unwrap();
        let hardware = (0..20).collect::<Vec<u8>>();
        for subject in ["c", "rust"] {
            fs::write(previous.join(format!("{subject}-after.wram")), vec![0; WRAM]).unwrap();
            fs::write(previous.join(format!("{subject}-host-after.bin")), vec![0; 16]).unwrap();
            fs::write(previous.join(format!("{subject}-ppu.bin")), &hardware).unwrap();
        }
        fixture.continue_from = Some("previous".into());
        fixture.host_words = None;
        assert!(check(&dir, &fixture).is_err(), "accepted reset hardware in a continuation");
        fs::write(dir.join("ppu-before.bin"), &hardware).unwrap();
        assert!(check(&dir, &fixture).is_ok());
        fs::write(previous.join("rust-ppu.bin"), vec![0; 20]).unwrap();
        assert!(check(&dir, &fixture).is_err(), "accepted different predecessor hardware");
    }
}
