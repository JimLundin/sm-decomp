//! Complete native binary case execution with pinned external JSON corpus input.
use crate::atomic_toml;
use crate::digest;
use crate::pattern_contract as contract;
use crate::readiness_run::Run;
use crate::readiness_run::identity;
use crate::readiness_run::safe_artifact;
use crate::readiness_run::source_inventory;
use anyhow::{Context, Result, ensure};
use pipeline_model::{
    readiness::{ProcessRecord, SourceFile},
    timed_corpus::{Case, Cycle, State, decode_cases, decode_ledger, encode_cases, encode_ledger},
    timed_pattern::Declaration,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use toml::Value;

#[derive(Deserialize)]
struct ExternalCase {
    name: String,
    initial: State,
    #[serde(rename = "final")]
    fin: State,
    cycles: Vec<(u32, Option<u8>, String)>,
}
/// JSON is consumed solely at this external SingleStepTests boundary.
fn corpus(path: &Path) -> Result<Vec<Case>> {
    let rows: Vec<ExternalCase> = serde_json::from_slice(&fs::read(path)?)?;
    Ok(rows
        .into_iter()
        .map(|c| Case {
            name: c.name,
            initial: c.initial,
            fin: c.fin,
            cycles: c.cycles.into_iter().map(|(address, value, pins)| Cycle { address, value, pins }).collect(),
            case_sha256: String::new(),
            opcode: String::new(),
            conditions: vec![],
            independent_fixture: false,
            body: 0,
            bytes: vec![],
        })
        .collect())
}
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct Counts {
    expected: u64,
    run: u64,
    passed: u64,
    failed: u64,
    skipped: u64,
    independent_fixtures: Vec<String>,
}
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct Coverage {
    expected: u64,
    passed: u64,
}
#[derive(Clone, Serialize, Deserialize)]
struct CorpusIdentity {
    opcode: String,
    sha256: String,
    cases: usize,
}
#[derive(Clone, Serialize, Deserialize)]
struct Generator {
    path: String,
    sha256: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Dependency {
    name: String,
    archive: String,
    sha256: String,
    #[serde(default)]
    revision: String,
}
#[derive(Deserialize)]
struct Dependencies {
    schema_version: u32,
    dependencies: Vec<Dependency>,
}
#[derive(Serialize, Deserialize)]
struct PatternReport {
    schema_version: u32,
    status: String,
    complete: bool,
    expected: u64,
    run: u64,
    passed: u64,
    failed: u64,
    skipped: u64,
    opcodes: BTreeMap<String, Counts>,
    corpus: Vec<CorpusIdentity>,
    scope: String,
    declaration: Declaration,
    declaration_sha256: String,
    declaration_file_sha256: String,
    declaration_path: String,
    fixtures_path: String,
    fixtures_sha256: String,
    source_sha256: String,
    source_inventory: Vec<SourceFile>,
    coverage: BTreeMap<String, BTreeMap<String, Coverage>>,
    corpus_verified: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    corpus_archive_sha256: Option<String>,
    pinned_dependency: Dependency,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    prospective_opcode: Option<String>,
    run_owner: PathBuf,
    compiler: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    generator: Option<Generator>,
    #[serde(default)]
    boundary_fixtures: usize,
    #[serde(default)]
    expected_inventory_sha256: String,
    #[serde(default)]
    elapsed_seconds: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct BatchIdentity {
    bodies_sha256: String,
    prepared_sha256: String,
    binary: String,
    binary_sha256: String,
    input_sha256: String,
    ledger_sha256: String,
}
#[derive(Serialize, Deserialize)]
struct TimingReport {
    schema_version: u32,
    status: String,
    complete: bool,
    returncode: Option<i32>,
    process_evidence: String,
    process_sha256: String,
    scope: String,
    source_sha256: String,
    declaration: Declaration,
    declaration_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    prospective_opcode: Option<String>,
    oracles: BTreeMap<String, String>,
    sources: Vec<String>,
    tests: Vec<String>,
    conditions: TimingConditions,
}
#[derive(Serialize, Deserialize)]
struct TimingConditions {
    applicable: Vec<String>,
    inapplicable: Vec<String>,
}
fn prepare_case(mut case: Case, opcode: &str, pattern: &Value) -> Result<Case> {
    case.case_sha256.clear();
    case.opcode.clear();
    case.body = 0;
    case.bytes.clear();
    let coverage = case.conditions.clone();
    case.conditions = vec![];
    // Canonical source identity uses only externally supplied state/cycles and literal fixture coverage.
    #[derive(Serialize)]
    struct Source<'a> {
        name: &'a str,
        initial: &'a State,
        #[serde(rename = "final")]
        fin: &'a State,
        cycles: &'a [Cycle],
        coverage: &'a [String],
    }
    case.case_sha256 = identity(&Source {
        name: &case.name,
        initial: &case.initial,
        fin: &case.fin,
        cycles: &case.cycles,
        coverage: &coverage,
    })?;
    case.conditions = coverage;
    case.conditions = contract::conditions(&case, pattern)?;
    case.opcode = opcode.into();
    Ok(case)
}
fn pinned(run: &mut Run, corpus: &Path, pin: &Dependency, opcodes: &[String]) -> Result<Option<String>> {
    if fs::canonicalize(run.root.join("deps/65816/v1"))? != corpus {
        return Ok(None);
    }
    let archive = run.root.join("deps/downloads").join(&pin.archive);
    let hash = run.guard(&archive)?;
    ensure!(hash == pin.sha256, "pinned instruction corpus archive checksum differs");
    let extracted = run.path.join("generated/pinned-corpus");
    fs::create_dir(&extracted)?;
    let result = (|| -> Result<()> {
        let mut command = vec![
            "tar".into(),
            "-xzf".into(),
            archive.to_string_lossy().into_owned(),
            "-C".into(),
            extracted.to_string_lossy().into_owned(),
        ];
        let members = opcodes.iter().map(|o| format!("65816-{}/v1/{o}.n.json", pin.revision)).collect::<Vec<_>>();
        command.extend(members.clone());
        let process = run.stage(
            "corpus-archive",
            &command,
            &run.root.clone(),
            &BTreeMap::new(),
            Duration::from_secs(120),
            &BTreeMap::new(),
        )?;
        ensure!(crate::documents::field(&process, "status")? == "pass", "pinned corpus extraction failed");
        for (opcode, member) in opcodes.iter().zip(members) {
            ensure!(
                digest(&safe_artifact(&extracted, &member)?)? == run.guard(&corpus.join(format!("{opcode}.n.json")))?,
                "consumed corpus differs from pinned archive"
            );
        }
        Ok(())
    })();
    fs::remove_dir_all(&extracted)?;
    result?;
    Ok(Some(hash))
}
fn write_report(output: &Path, report: &PatternReport) -> Result<()> {
    atomic_toml(&output.join("instruction.toml"), report)
}
pub(crate) fn produce(run: &mut Run, output: &Path, request: &Value, report: &mut Value) -> Result<()> {
    let started = Instant::now();
    let root = run.root.clone();
    let prospective = request.get("prospective").and_then(Value::as_str);
    let declaration_value =
        contract::load(&root, request.get("declaration").and_then(Value::as_str).map(Path::new), prospective)?;
    let declaration: Declaration = declaration_value.clone().try_into()?;
    let declared = declaration.patterns.iter().map(|p| p.opcode.clone()).collect::<Vec<_>>();
    let opcodes = request
        .get("opcodes")
        .and_then(Value::as_str)
        .map(|s| s.split(',').map(str::to_owned).collect::<Vec<_>>())
        .unwrap_or(declared.clone());
    let selected = opcodes.iter().cloned().collect::<BTreeSet<_>>();
    ensure!(
        !selected.is_empty()
            && selected.len() == opcodes.len()
            && selected.is_subset(&declared.iter().cloned().collect()),
        "selected opcodes outside production declaration"
    );
    let batch_size = request.get("batch_size").and_then(Value::as_integer).unwrap_or(500);
    ensure!(batch_size > 0, "batch size must be positive");
    let corpus_path =
        fs::canonicalize(root.join(request.get("corpus").and_then(Value::as_str).unwrap_or("deps/65816/v1")))?;
    let deps: Dependencies = pipeline_model::read(&root.join("tools/dependencies.toml"))?;
    ensure!(deps.schema_version == 1, "dependency schema differs");
    let pin = deps.dependencies.into_iter().find(|p| p.name == "65816").context("corpus dependency pin missing")?;
    fs::copy(root.join(contract::DECLARATION), output.join("declaration.toml"))?;
    fs::copy(root.join(contract::FIXTURES), output.join("independent-fixtures.toml"))?;
    let inventory = source_inventory(&root)?;
    let mut native = PatternReport {
        schema_version: 3,
        status: "incomplete".into(),
        complete: false,
        expected: 0,
        run: 0,
        passed: 0,
        failed: 0,
        skipped: 0,
        opcodes: BTreeMap::new(),
        corpus: vec![],
        scope: if selected == declared.iter().cloned().collect() {
            if prospective.is_some() { "complete prospective declaration" } else { "complete production declaration" }
        } else {
            "development selected opcodes"
        }
        .into(),
        declaration_sha256: identity(&declaration)?,
        declaration,
        declaration_file_sha256: digest(&root.join(contract::DECLARATION))?,
        declaration_path: contract::DECLARATION.into(),
        fixtures_path: contract::FIXTURES.into(),
        fixtures_sha256: digest(&root.join(contract::FIXTURES))?,
        source_sha256: identity(&inventory)?,
        source_inventory: inventory,
        coverage: BTreeMap::new(),
        corpus_verified: false,
        corpus_archive_sha256: None,
        pinned_dependency: pin,
        prospective_opcode: prospective.map(str::to_owned),
        run_owner: run.path.clone(),
        compiler: String::new(),
        generator: None,
        boundary_fixtures: 0,
        expected_inventory_sha256: String::new(),
        elapsed_seconds: 0.0,
        reason: None,
    };
    write_report(output, &native)?;
    let process = run.stage(
        "pattern-compiler",
        &["rustc", "--version", "--verbose"].map(str::to_owned),
        &root,
        &BTreeMap::new(),
        Duration::from_secs(30),
        &BTreeMap::new(),
    )?;
    ensure!(crate::documents::field(&process, "status")? == "pass", "compiler identity failed");
    native.compiler = fs::read_to_string(run.path.join(crate::documents::field(&process, "stdout")?))?;
    run.manifest.identities.insert("compiler".into(), Value::Table(toml::toml! {version=(native.compiler.trim())}));
    run.save()?;
    native.corpus_archive_sha256 = pinned(run, &corpus_path, &native.pinned_dependency, &opcodes)?;
    native.corpus_verified = native.corpus_archive_sha256.is_some();
    let patterns = native
        .declaration
        .patterns
        .iter()
        .map(|p| Ok((p.opcode.clone(), Value::try_from(p)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    fs::create_dir(output.join("corpus"))?;
    let mut cases = vec![];
    for opcode in &opcodes {
        let path = corpus_path.join(format!("{opcode}.n.json"));
        let hash = run.guard(&path)?;
        let raw = corpus(&path)?;
        ensure!(!raw.is_empty(), "empty declared corpus");
        fs::copy(&path, output.join("corpus").join(path.file_name().unwrap()))?;
        ensure!(digest(&path)? == hash, "corpus changed during copy");
        native.corpus.push(CorpusIdentity { opcode: opcode.clone(), sha256: hash, cases: raw.len() });
        native.opcodes.insert(
            opcode.clone(),
            Counts {
                expected: raw.len() as u64,
                independent_fixtures: contract::strings(&patterns[opcode]["independent_fixtures"])?,
                ..Counts::default()
            },
        );
        for case in raw {
            let mx = (case.initial.p >> 4) & 3;
            ensure!(
                case.initial.e == 0
                    && case.fin.e == 0
                    && crate::documents::array(&patterns[opcode], "mx")?.contains(&Value::Integer(i64::from(mx))),
                "unsupported corpus width/mode"
            );
            cases.push(prepare_case(case, opcode, &patterns[opcode])?);
        }
    }
    let fixtures = contract::fixtures(&root.join(contract::FIXTURES))?;
    for opcode in &opcodes {
        for name in native.opcodes[opcode].independent_fixtures.clone() {
            let matches = fixtures.iter().filter(|c| c.name == name).collect::<Vec<_>>();
            ensure!(matches.len() == 1, "independent fixture missing/duplicate");
            let case = matches[0].clone();
            let pc = (u32::from(case.initial.pbr) << 16) | u32::from(case.initial.pc);
            ensure!(
                case.initial.ram.iter().any(|(a, v)| *a == pc && Some(*v) == u8::from_str_radix(opcode, 16).ok()),
                "fixture opcode differs"
            );
            cases.push(prepare_case(case, opcode, &patterns[opcode])?);
            native.opcodes.get_mut(opcode).unwrap().expected += 1;
            native.boundary_fixtures += 1;
        }
        let mut counts = BTreeMap::new();
        for condition in contract::strings(&patterns[opcode]["required_conditions"])?
            .into_iter()
            .chain(contract::strings(&patterns[opcode]["timing_branches"])?)
        {
            counts.insert(condition, Coverage::default());
        }
        native.coverage.insert(opcode.clone(), counts);
    }
    for case in &cases {
        for condition in &case.conditions {
            native
                .coverage
                .get_mut(&case.opcode)
                .unwrap()
                .get_mut(condition)
                .context("undeclared condition")?
                .expected += 1;
        }
    }
    native.expected = cases.len() as u64;
    fs::write(output.join("expected-cases.bin"), encode_cases(&cases)?)?;
    native.expected_inventory_sha256 = digest(&output.join("expected-cases.bin"))?;
    write_report(output, &native)?;
    let target = output.join("build");
    let mut env = BTreeMap::from([("CARGO_TARGET_DIR".into(), target.to_string_lossy().into_owned())]);
    let command = |bin: &str| {
        ["cargo", "build", "-p", "fuzz", "--bin", bin, "--locked", "--offline", "--jobs", "1"].map(str::to_owned)
    };
    let process = run.stage(
        "pattern-generator-build",
        &command("timed-gen"),
        &root,
        &env,
        Duration::from_secs(240),
        &BTreeMap::new(),
    )?;
    ensure!(crate::documents::field(&process, "status")? == "pass", "pattern generator build failed");
    let generator = output.join("timed-gen");
    fs::copy(target.join("debug/timed-gen"), &generator)?;
    native.generator = Some(Generator { path: "timed-gen".into(), sha256: digest(&generator)? });
    let mut combined = vec![];
    for (number, batch_cases) in cases.chunks(batch_size as usize).enumerate() {
        run.check_sources()?;
        let batch = output.join(format!("batch-{number:04}"));
        fs::create_dir(&batch)?;
        let input = batch.join("input.bin");
        let prepared = batch.join("prepared.bin");
        let bodies = batch.join("bodies.rs");
        let ledger = batch.join("cases.bin");
        fs::write(&input, encode_cases(batch_cases)?)?;
        let mut generation = vec![
            generator.to_string_lossy().into_owned(),
            input.to_string_lossy().into_owned(),
            bodies.to_string_lossy().into_owned(),
            prepared.to_string_lossy().into_owned(),
        ];
        if let Some(opcode) = prospective {
            generation.push(opcode.into());
        }
        for (kind, argv) in [("generate", generation), ("build", command("timed-run").to_vec())] {
            if kind == "build" {
                env.insert("SM_TIMED_STEPS".into(), bodies.to_string_lossy().into_owned());
            }
            let process = run.stage(
                &format!("pattern-{number:04}-{kind}"),
                &argv,
                &root,
                &env,
                Duration::from_secs(240),
                &BTreeMap::new(),
            )?;
            atomic_toml(&batch.join(format!("{kind}.toml")), &process)?;
            ensure!(crate::documents::field(&process, "status")? == "pass", "pattern {kind} failed");
        }
        let binary = batch.join("timed-run");
        fs::copy(target.join("debug/timed-run"), &binary)?;
        let argv = vec![
            binary.to_string_lossy().into_owned(),
            prepared.to_string_lossy().into_owned(),
            ledger.to_string_lossy().into_owned(),
        ];
        let process = run.stage(
            &format!("pattern-{number:04}-execute"),
            &argv,
            &root,
            &env,
            Duration::from_secs(240),
            &BTreeMap::from([(1, "divergence".into())]),
        )?;
        atomic_toml(&batch.join("execute.toml"), &process)?;
        let process: ProcessRecord = process.try_into()?;
        ensure!(process.complete && [Some(0), Some(1)].contains(&process.returncode), "pattern execution incomplete");
        let rows = decode_ledger(&fs::read(&ledger)?)?;
        ensure!(rows.len() == batch_cases.len(), "case ledger truncated");
        for (row, case) in rows.iter().zip(batch_cases) {
            ensure!(
                row.name == case.name
                    && row.case_sha256 == case.case_sha256
                    && row.opcode == case.opcode
                    && ["pass", "divergence"].contains(&row.status.as_str()),
                "case order/identity/status differs"
            );
            let pass = row.status == "pass";
            native.run += 1;
            let count = native.opcodes.get_mut(&row.opcode).unwrap();
            count.run += 1;
            if pass {
                native.passed += 1;
                count.passed += 1;
                for condition in &case.conditions {
                    native.coverage.get_mut(&row.opcode).unwrap().get_mut(condition).unwrap().passed += 1;
                }
            } else {
                native.failed += 1;
                count.failed += 1;
            }
        }
        ensure!((process.returncode == Some(0)) == rows.iter().all(|r| r.status == "pass"), "exit differs from ledger");
        combined.extend(rows);
        fs::write(output.join("cases.bin"), encode_ledger(&combined)?)?;
        atomic_toml(
            &batch.join("identities.toml"),
            &BatchIdentity {
                bodies_sha256: digest(&bodies)?,
                prepared_sha256: digest(&prepared)?,
                binary: "timed-run".into(),
                binary_sha256: digest(&binary)?,
                input_sha256: digest(&input)?,
                ledger_sha256: digest(&ledger)?,
            },
        )?;
        write_report(output, &native)?;
    }
    native.status = if native.failed == 0 { "pass" } else { "divergence" }.into();
    native.complete = true;
    native.elapsed_seconds = started.elapsed().as_secs_f64();
    write_report(output, &native)?;
    *report = Value::try_from(&native)?;
    if request.get("timing").and_then(Value::as_bool) == Some(true) {
        let mut argv = vec!["cargo".into(), "test".into(), "-p".into(), "snes".into()];
        for test in &native.declaration.timing_tests {
            argv.extend(["--test".into(), test.clone()]);
        }
        argv.extend(["--locked", "--offline", "--jobs", "1"].map(str::to_owned));
        let process =
            run.stage("pattern-timing-execution", &argv, &root, &env, Duration::from_secs(600), &BTreeMap::new())?;
        atomic_toml(&output.join("timing-execution.toml"), &process)?;
        let process: ProcessRecord = process.try_into()?;
        let mut oracles = BTreeMap::new();
        for name in &native.declaration.timing_oracles {
            oracles.insert(name.clone(), run.guard(&root.join(name))?);
        }
        let timing = TimingReport {
            schema_version: 3,
            status: process.status.clone(),
            complete: process.complete,
            returncode: process.returncode,
            process_evidence: "timing-execution.toml".into(),
            process_sha256: digest(&output.join("timing-execution.toml"))?,
            scope: native.scope.clone(),
            source_sha256: native.source_sha256.clone(),
            declaration: native.declaration.clone(),
            declaration_sha256: native.declaration_sha256.clone(),
            prospective_opcode: native.prospective_opcode.clone(),
            oracles,
            sources: native.declaration.timing_sources.clone(),
            tests: native.declaration.timing_tests.clone(),
            conditions: TimingConditions {
                applicable: native.declaration.hardware_conditions.clone(),
                inapplicable: native.declaration.exclusions.clone(),
            },
        };
        atomic_toml(&output.join("timing.toml"), &timing)?;
        ensure!(process.status == "pass", "timing execution failed");
    }
    run.check_sources()
}

pub fn operate(request: Value) -> Result<Value> {
    let root = request.get("checkout").and_then(Value::as_str).map(PathBuf::from).unwrap_or(std::env::current_dir()?);
    let mut run = Run::create(&root, "patterns", toml::toml! {native=true}, None)?;
    run.capture_source()?;
    eprintln!("Run: {}", run.path.display());
    let output = request
        .get("output")
        .and_then(Value::as_str)
        .map(|p| root.join(p))
        .unwrap_or_else(|| run.path.join("checks/patterns"));
    ensure!(!output.exists(), "output already exists");
    fs::create_dir_all(&output)?;
    let output = fs::canonicalize(output)?;
    let mut report = Value::Table(toml::Table::new());
    if let Err(error) = produce(&mut run, &output, &request, &mut report) {
        report = if output.join("instruction.toml").exists() {
            pipeline_model::read(&output.join("instruction.toml"))?
        } else {
            Value::Table(toml::Table::new())
        };
        let table = report.as_table_mut().context("pattern report table")?;
        table.insert("status".into(), Value::String(if run.was_interrupted() { "incomplete" } else { "error" }.into()));
        table.insert("complete".into(), Value::Boolean(false));
        table.insert("reason".into(), Value::String(format!("{error:#}")));
        atomic_toml(&output.join("instruction.toml"), &report)?;
    }
    run.record_check("instruction", report.clone())?;
    let mut mandatory = vec!["instruction".into()];
    if request.get("timing").and_then(Value::as_bool) == Some(true) {
        mandatory.push("timing".into());
        if output.join("timing.toml").exists() {
            run.record_check("timing", pipeline_model::read(&output.join("timing.toml"))?)?;
        }
    }
    run.manifest.identities.insert(
        "pattern-output".into(),
        crate::report!({"path":output,"instruction_sha256":digest(&output.join("instruction.toml"))?}),
    );
    let status = crate::documents::field(&report, "status")?.to_owned();
    run.finalize(
        &status,
        Some("patterns"),
        report
            .get("reason")
            .and_then(Value::as_str)
            .or(Some("Instruction/timing scope only; readiness gates remain pending")),
        &mandatory,
    )
}
fn authenticate_process(
    owner: &Path,
    path: &Path,
    expected: Option<&[String]>,
    allowed: &[i32],
) -> Result<ProcessRecord> {
    let process: ProcessRecord = pipeline_model::read(path)?;
    ensure!(
        process.complete && process.signal.is_none() && process.returncode.is_some_and(|n| allowed.contains(&n)),
        "pattern process failed/interrupted"
    );
    if let Some(command) = expected {
        ensure!(process.command == command, "pattern invocation differs");
    }
    let name = Path::new(&process.stdout)
        .file_name()
        .context("process name")?
        .to_str()
        .context("process UTF-8")?
        .strip_suffix(".stdout.log")
        .context("process name format")?;
    let (manifest, _) = crate::readiness_origins::snapshot_typed(owner)?;
    let seal = manifest.process_evidence.get(name).context("process seal")?;
    ensure!(
        manifest.commands.contains(&process)
            && digest(&owner.join(format!("logs/{name}.toml")))? == seal.report
            && pipeline_model::read::<ProcessRecord>(&owner.join(format!("logs/{name}.toml")))? == process
            && pipeline_model::read::<ProcessRecord>(&owner.join(format!("checks/{name}.toml")))? == process,
        "pattern process differs from owner evidence"
    );
    for (relative, hash) in [(&process.stdout, &seal.stdout), (&process.stderr, &seal.stderr)] {
        ensure!(Some(digest(&safe_artifact(owner, relative)?)?) == *hash, "pattern process log checksum differs");
    }
    Ok(process)
}
pub fn authenticate(request: Value) -> Result<Value> {
    let output = fs::canonicalize(crate::documents::field(&request, "output")?)?;
    let report: PatternReport = pipeline_model::read(&output.join("instruction.toml"))?;
    ensure!(report.schema_version == 3, "unsupported native pattern version");
    let owner = fs::canonicalize(
        request.get("run").and_then(Value::as_str).map(PathBuf::from).unwrap_or(report.run_owner.clone()),
    )?;
    let (manifest, snapshot) = crate::readiness_origins::snapshot_typed(&owner)?;
    ensure!(
        snapshot.files == report.source_inventory
            && snapshot.sha256 == report.source_sha256
            && identity(&report.source_inventory)? == report.source_sha256
            && Some(report.compiler.trim())
                == manifest
                    .identities
                    .get("compiler")
                    .and_then(|v| v.get("version"))
                    .and_then(Value::as_str)
                    .map(str::trim),
        "pattern source/compiler identity differs"
    );
    let declaration_value =
        contract::declaration(&output.join("declaration.toml"), report.prospective_opcode.as_deref())?;
    let declaration: Declaration = declaration_value.clone().try_into()?;
    ensure!(
        report.declaration == declaration && identity(&declaration)? == report.declaration_sha256,
        "pattern declaration differs"
    );
    for (source, artifact, hash) in [
        (contract::DECLARATION, "declaration.toml", &report.declaration_file_sha256),
        (contract::FIXTURES, "independent-fixtures.toml", &report.fixtures_sha256),
    ] {
        ensure!(
            snapshot.files.iter().any(|f| f.path == source && f.sha256 == *hash)
                && digest(&output.join(artifact))? == *hash,
            "pattern source artifact binding differs"
        );
    }
    ensure!(
        report.status == "pass"
            && report.complete
            && report.failed == 0
            && report.skipped == 0
            && report.passed == report.expected,
        "pattern success terminal counts missing"
    );
    if request.get("development_scope").and_then(Value::as_bool) != Some(true) {
        contract::validate_report(&Value::try_from(&report)?, false)?;
    }
    let generator = report.generator.as_ref().context("generator missing")?;
    let generator_path = safe_artifact(&output, &generator.path)?;
    ensure!(digest(&generator_path)? == generator.sha256, "generator differs");
    let expected = decode_cases(&fs::read(output.join("expected-cases.bin"))?)?;
    let rows = decode_ledger(&fs::read(output.join("cases.bin"))?)?;
    ensure!(
        digest(&output.join("expected-cases.bin"))? == report.expected_inventory_sha256
            && expected.len() as u64 == report.expected
            && rows.len() == expected.len()
            && report.run == rows.len() as u64,
        "expected/completed ledger differs"
    );
    let patterns = declaration
        .patterns
        .iter()
        .map(|p| Ok((p.opcode.clone(), Value::try_from(p)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let fixtures = contract::fixtures(&output.join("independent-fixtures.toml"))?;
    let mut reconstructed = vec![];
    let mut opcodes = BTreeSet::new();
    for item in &report.corpus {
        ensure!(
            opcodes.insert(item.opcode.clone()) && patterns.contains_key(&item.opcode),
            "duplicate/undeclared corpus opcode"
        );
        let path = output.join("corpus").join(format!("{}.n.json", item.opcode));
        ensure!(digest(&path)? == item.sha256, "preserved corpus differs");
        let cases = corpus(&path)?;
        ensure!(cases.len() == item.cases, "corpus count differs");
        for case in cases {
            reconstructed.push(prepare_case(case, &item.opcode, &patterns[&item.opcode])?);
        }
    }
    for item in &report.corpus {
        for name in contract::strings(&patterns[&item.opcode]["independent_fixtures"])? {
            let found = fixtures.iter().filter(|c| c.name == name).collect::<Vec<_>>();
            ensure!(found.len() == 1, "fixture missing/duplicate");
            reconstructed.push(prepare_case(found[0].clone(), &item.opcode, &patterns[&item.opcode])?);
        }
    }
    ensure!(reconstructed == expected, "ordered complete corpus/fixture scope differs");
    let mut batches = fs::read_dir(&output)?
        .filter_map(|v| v.ok().map(|e| e.path()))
        .filter(|p| p.is_dir() && p.file_name().unwrap().to_string_lossy().starts_with("batch-"))
        .collect::<Vec<_>>();
    batches.sort();
    let mut combined = vec![];
    let mut inputs = vec![];
    for (number, batch) in batches.iter().enumerate() {
        ensure!(
            !batch.is_symlink() && batch.file_name().unwrap() == format!("batch-{number:04}").as_str(),
            "batch order/path differs"
        );
        let ids: BatchIdentity = pipeline_model::read(&batch.join("identities.toml"))?;
        for (relative, hash) in [
            ("bodies.rs", &ids.bodies_sha256),
            ("prepared.bin", &ids.prepared_sha256),
            (ids.binary.as_str(), &ids.binary_sha256),
            ("input.bin", &ids.input_sha256),
            ("cases.bin", &ids.ledger_sha256),
        ] {
            ensure!(digest(&safe_artifact(batch, relative)?)? == *hash, "batch artifact changed: {relative}");
        }
        let argv = vec![
            batch.join(&ids.binary).to_string_lossy().into_owned(),
            batch.join("prepared.bin").to_string_lossy().into_owned(),
            batch.join("cases.bin").to_string_lossy().into_owned(),
        ];
        let process = authenticate_process(&owner, &batch.join("execute.toml"), Some(&argv), &[0, 1])?;
        let mut argv = vec![
            generator_path.to_string_lossy().into_owned(),
            batch.join("input.bin").to_string_lossy().into_owned(),
            batch.join("bodies.rs").to_string_lossy().into_owned(),
            batch.join("prepared.bin").to_string_lossy().into_owned(),
        ];
        if let Some(opcode) = &report.prospective_opcode {
            argv.push(opcode.clone());
        }
        authenticate_process(&owner, &batch.join("generate.toml"), Some(&argv), &[0])?;
        authenticate_process(
            &owner,
            &batch.join("build.toml"),
            Some(
                &["cargo", "build", "-p", "fuzz", "--bin", "timed-run", "--locked", "--offline", "--jobs", "1"]
                    .map(str::to_owned),
            ),
            &[0],
        )?;
        let cases = decode_cases(&fs::read(batch.join("input.bin"))?)?;
        let batch_rows = decode_ledger(&fs::read(batch.join("cases.bin"))?)?;
        ensure!(
            batch_rows.len() == cases.len()
                && (process.returncode == Some(0)) == batch_rows.iter().all(|r| r.status == "pass"),
            "batch scope/exit differs"
        );
        inputs.extend(cases);
        combined.extend(batch_rows);
    }
    ensure!(combined == rows && inputs == expected, "combined batch ledger/inputs differ");
    let mut counts: BTreeMap<String, Counts> = report
        .opcodes
        .iter()
        .map(|(o, c)| (o.clone(), Counts { independent_fixtures: c.independent_fixtures.clone(), ..Counts::default() }))
        .collect();
    let mut coverage: BTreeMap<String, BTreeMap<String, Coverage>> = report
        .coverage
        .iter()
        .map(|(o, c)| (o.clone(), c.keys().map(|k| (k.clone(), Coverage::default())).collect()))
        .collect();
    for (row, case) in rows.iter().zip(&expected) {
        ensure!(
            row.name == case.name
                && row.case_sha256 == case.case_sha256
                && row.opcode == case.opcode
                && row.status == "pass",
            "ordered identity/outcome differs"
        );
        let count = counts.get_mut(&row.opcode).context("undeclared validated opcode")?;
        count.expected += 1;
        count.run += 1;
        count.passed += 1;
        for condition in &case.conditions {
            let count = coverage.get_mut(&row.opcode).unwrap().get_mut(condition).context("undeclared condition")?;
            count.expected += 1;
            count.passed += 1;
        }
    }
    ensure!(counts == report.opcodes && coverage == report.coverage, "opcode/condition ledger counts differ");
    if request.get("timing").and_then(Value::as_bool) == Some(true) {
        let timing: TimingReport = pipeline_model::read(&output.join("timing.toml"))?;
        ensure!(
            timing.schema_version == 3
                && digest(&output.join("timing-execution.toml"))? == timing.process_sha256
                && timing.declaration == declaration
                && timing.source_sha256 == report.source_sha256,
            "timing identity differs"
        );
        contract::validate_report(&Value::try_from(&timing)?, true)?;
        let mut argv = vec!["cargo".into(), "test".into(), "-p".into(), "snes".into()];
        for test in &declaration.timing_tests {
            argv.extend(["--test".into(), test.clone()]);
        }
        argv.extend(["--locked", "--offline", "--jobs", "1"].map(str::to_owned));
        authenticate_process(&owner, &output.join("timing-execution.toml"), Some(&argv), &[0])?;
        for (name, hash) in timing.oracles {
            ensure!(digest(&safe_artifact(&owner.join("source/files"), &name)?)? == hash, "timing oracle differs");
        }
    }
    Ok(Value::try_from(&report)?)
}
