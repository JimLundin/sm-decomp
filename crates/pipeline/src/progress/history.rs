//! Calendar-time coverage history from retained committed source, never report
//! import timestamps. The fixed native denominator makes re-audits comparable.
use super::*;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

const TIMING_INPUT: &str = "tools/progress/pipeline-timings.tsv";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PipelineSample {
    label: String,
    elapsed_ns: u64,
    detailed_stages: bool,
    evidence: Evidence,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PipelineEvidence {
    samples: Vec<PipelineSample>,
    audit: Option<Evidence>,
    status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Checkpoint {
    pub(super) commit: String,
    pub(super) at: u64,
    pub(super) subject: String,
    pub(super) added: Vec<String>,
    pub(super) removed: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct History {
    pub(super) schema_version: u32,
    pub(super) generated_at: u64,
    pub(super) head: String,
    pub(super) source_revision: String,
    pub(super) inventory_sha256: String,
    pub(super) baseline_commit: String,
    pub(super) baseline_at: u64,
    pub(super) checkpoints: Vec<Checkpoint>,
    pub(super) current: Count,
    pub(super) current_symbols: Vec<String>,
    pub(super) uncommitted: Vec<String>,
    pub(super) files: Vec<Evidence>,
    pub(super) inputs: Vec<Evidence>,
    pub(super) status: String,
    pub(super) limitations: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) pipeline: Option<PipelineEvidence>,
}

fn pipeline_evidence(root: &Path) -> Result<Option<PipelineEvidence>> {
    let path = root.join(TIMING_INPUT);
    if !path.exists() {
        return Ok(None);
    }
    let mut samples = Vec::new();
    for line in bounded_text(&path, MAX_UPDATE)?.lines().filter(|line| !line.is_empty() && !line.starts_with('#')) {
        let (path, label) = line.split_once('\t').context("invalid retained pipeline timing input")?;
        ensure!(relative(path), "unsafe retained timing path");
        let file = root.join(path);
        if !file.is_file() {
            continue;
        }
        // Only top-level metrics are needed. Full files can include thousands
        // of per-process receipts, which remain immutable and hash-addressed.
        let mut prefix = String::new();
        for line in BufReader::new(File::open(&file)?).lines() {
            let line = line?;
            if line.starts_with('[') {
                break;
            }
            ensure!(prefix.len() + line.len() < MAX_UPDATE as usize, "pipeline metrics header exceeds bound");
            prefix.push_str(&line);
            prefix.push('\n');
        }
        let metrics: toml::Value = toml::from_str(&prefix)?;
        ensure!(metrics["format"].as_str() == Some("pipeline-metrics-v1"), "unsupported retained timing format");
        ensure!(metrics["outcome"].as_str() == Some("success"), "retained timing did not finish successfully");
        let elapsed_ns = metrics["elapsed_ns"].as_integer().context("pipeline elapsed time missing")?;
        ensure!(elapsed_ns >= 0, "negative pipeline elapsed time");
        samples.push(PipelineSample {
            label: label.into(),
            elapsed_ns: elapsed_ns as u64,
            detailed_stages: metrics.get("enabled").and_then(toml::Value::as_bool).unwrap_or(false),
            evidence: Evidence { path: path.into(), sha256: crate::digest(&file)? },
        });
    }
    if samples.is_empty() {
        return Ok(None);
    }
    let audit_path = ".scratch/decomp-strategy/architecture-audit.md";
    let audit = root
        .join(audit_path)
        .is_file()
        .then(|| -> Result<Evidence> {
            Ok(Evidence { path: audit_path.into(), sha256: crate::digest(&root.join(audit_path))? })
        })
        .transpose()?;
    Ok(Some(PipelineEvidence { samples, audit, status: "current".into() }))
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let result = Command::new("git").arg("-C").arg(root).args(args).output()?;
    ensure!(
        result.status.success(),
        "source history git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(result.stdout)
}

fn tree(root: &Path, commit: &str) -> Result<BTreeMap<String, String>> {
    let raw = git(root, &["ls-tree", "-rz", commit, "--", "crates/port/src"])?;
    let mut files = BTreeMap::new();
    for record in raw.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let record = std::str::from_utf8(record)?;
        let (header, path) = record.split_once('\t').context("invalid Git tree entry")?;
        if !path.ends_with(".rs") {
            continue;
        }
        ensure!(relative(path), "unsafe source history path");
        let blob = header.split_whitespace().nth(2).context("Git tree object missing")?;
        files.insert(path.to_owned(), blob.to_owned());
    }
    Ok(files)
}

fn blobs(root: &Path, objects: &BTreeSet<String>) -> Result<BTreeMap<String, String>> {
    let mut command = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut input = command.stdin.take().context("Git batch input unavailable")?;
    let request = objects.iter().map(|v| format!("{v}\n")).collect::<String>();
    // Drain stdout while writing IDs: a long history can fill both pipes.
    let writer = std::thread::spawn(move || input.write_all(request.as_bytes()));
    let output = command.wait_with_output()?;
    writer.join().map_err(|_| anyhow::anyhow!("Git batch input writer panicked"))??;
    ensure!(output.status.success(), "Git source blob read failed");
    let mut cursor = 0;
    let mut result = BTreeMap::new();
    while cursor < output.stdout.len() {
        let end = output.stdout[cursor..].iter().position(|b| *b == b'\n').context("Git blob header missing")? + cursor;
        let header = std::str::from_utf8(&output.stdout[cursor..end])?.split_whitespace().collect::<Vec<_>>();
        ensure!(header.len() == 3 && header[1] == "blob", "invalid source blob header");
        let length: usize = header[2].parse()?;
        ensure!(length <= MAX_METADATA as usize, "source blob exceeds bound");
        cursor = end + 1;
        ensure!(cursor + length < output.stdout.len(), "truncated source blob");
        result.insert(header[0].into(), std::str::from_utf8(&output.stdout[cursor..cursor + length])?.to_owned());
        cursor += length + 1;
    }
    ensure!(result.len() == objects.len(), "source blob count differs");
    Ok(result)
}

fn current_files(root: &Path) -> Result<Vec<Evidence>> {
    fn collect(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                collect(&entry.path(), files)?;
            } else if kind.is_file() && entry.path().extension().is_some_and(|v| v == "rs") {
                files.push(entry.path());
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    collect(&root.join("crates/port/src"), &mut files)?;
    files.sort();
    files
        .into_iter()
        .map(|path| {
            Ok(Evidence {
                path: path.strip_prefix(root)?.to_string_lossy().into_owned(),
                sha256: crate::digest(&path)?,
            })
        })
        .collect()
}

pub(super) fn reconstruct(root: &Path, registry: &Registry) -> Result<History> {
    let scanner = source::Scanner::new(root, &registry.routines)?;
    let head = String::from_utf8(git(root, &["rev-parse", "HEAD"])?)?.trim().to_owned();
    let log = String::from_utf8(git(
        root,
        &["log", "--first-parent", "--reverse", "--format=%H%x09%ct%x09%s", &head, "--", "crates/port/src"],
    )?)?;
    let mut commits = Vec::new();
    for line in log.lines() {
        let fields = line.splitn(3, '\t').collect::<Vec<_>>();
        ensure!(fields.len() == 3 && hex(fields[0], 40), "invalid source history commit");
        commits.push((fields[0].to_owned(), fields[1].parse::<u64>()?, fields[2].to_owned()));
    }
    let first = commits.first().context("no committed Rust port source history")?;
    let baseline_commit = String::from_utf8(git(root, &["rev-parse", &format!("{}^", first.0)])?)?.trim().to_owned();
    let baseline_at = first.1.saturating_sub(1);
    let baseline_tree = tree(root, &baseline_commit)?;
    ensure!(
        baseline_tree.is_empty(),
        "history baseline already contains Rust source; an explicit earlier baseline is required"
    );
    let mut trees = Vec::new();
    let mut object_ids = BTreeSet::new();
    for (commit, _, _) in &commits {
        let files = tree(root, commit)?;
        object_ids.extend(files.values().cloned());
        trees.push(files);
    }
    let contents = blobs(root, &object_ids)?;
    let mut cache = BTreeMap::<(String, String), BTreeSet<String>>::new();
    let mut previous = BTreeSet::new();
    let mut checkpoints = Vec::new();
    let mut previous_at = baseline_at;
    let mut clamped = 0;
    for ((commit, timestamp, subject), files) in commits.into_iter().zip(trees) {
        let mut symbols = BTreeSet::new();
        for (path, blob) in files {
            let supported =
                cache.entry((path.clone(), blob.clone())).or_insert_with(|| scanner.scan(&path, &contents[&blob]));
            symbols.extend(supported.iter().cloned());
        }
        let at = timestamp.max(previous_at);
        clamped += usize::from(at != timestamp);
        checkpoints.push(Checkpoint {
            commit,
            at,
            subject,
            added: symbols.difference(&previous).cloned().collect(),
            removed: previous.difference(&symbols).cloned().collect(),
        });
        previous = symbols;
        previous_at = at;
    }
    let files = current_files(root)?;
    let mut current = BTreeSet::new();
    for file in &files {
        current.extend(scanner.scan(&file.path, &bounded_text(&root.join(&file.path), MAX_METADATA)?));
    }
    let reported = registry
        .updates
        .iter()
        .filter(|u| {
            u.stage == Stage::Translated
                && u.evidence_context == EvidenceContext::CurrentCheckout
                && !registry.stale_update_ids.contains(&u.id)
                && !registry.superseded_update_ids.contains(&u.id)
        })
        .flat_map(|u| u.routines.iter().cloned())
        .collect::<BTreeSet<_>>();
    ensure!(
        current == reported,
        "current source scan differs from reviewed checkout translation reports: missing {:?}; extra {:?}",
        reported.difference(&current).collect::<Vec<_>>(),
        current.difference(&reported).collect::<Vec<_>>()
    );
    let pipeline = pipeline_evidence(root)?;
    let mut input_paths = source::INPUTS.to_vec();
    input_paths.push("crates/pipeline/src/progress/history.rs");
    if root.join(TIMING_INPUT).is_file() {
        input_paths.push(TIMING_INPUT);
    }
    let inputs = input_paths
        .iter()
        .map(|path| Ok(Evidence { path: path.to_string(), sha256: crate::digest(&root.join(path))? }))
        .collect::<Result<Vec<_>>>()?;
    let uncommitted = current.symmetric_difference(&previous).cloned().collect();
    let body_lines = registry.routines.iter().filter(|r| current.contains(&r.symbol)).map(|r| r.body_lines).sum();
    let mut limitations = vec![
        "Retrospective source correspondence over the retained first-parent history, using today's pinned denominator and reviewed selector/helper mappings. This is an estimate of committed body coverage, not branch completeness or semantic equivalence.".into(),
        "A commit timestamps source arriving on the retained branch. Uncommitted or unmerged work has no velocity credit. Historical helper-family dates can be approximate when a dispatcher grew between commits.".into(),
        "Calendar-time rates include pauses, verification, tooling work and periods with no commits. Active effort and edit-to-result timings are not recorded consistently.".into(),
        "Verification, integration and live-exercised observations remain separate. Report imports and re-audits are never source-velocity events.".into(),
        "Forecasts are straight-line scenarios over a short history. Remaining definitions differ in complexity; source coverage alone cannot forecast a playable game or full Rust ownership.".into(),
    ];
    if clamped > 0 {
        limitations.push(format!("{clamped} historical commit timestamps moved backwards and were clamped to the preceding source checkpoint."));
    }
    Ok(History {
        schema_version: 1,
        generated_at: now()?,
        head,
        source_revision: registry.source.revision.clone(),
        inventory_sha256: registry.source.inventory_sha256.clone(),
        baseline_commit,
        baseline_at,
        checkpoints,
        current: Count { definitions: current.len(), body_lines },
        current_symbols: current.into_iter().collect(),
        uncommitted,
        files,
        inputs,
        status: "current".into(),
        limitations,
        pipeline,
    })
}

pub(super) fn observe(root: &Path, registry: &mut Registry) {
    let Some(history) = &mut registry.coverage_history else {
        return;
    };
    let current = current_files(root);
    history.status = if history.inventory_sha256 != registry.source.inventory_sha256
        || history.source_revision != registry.source.revision
    {
        "inventory-changed"
    } else if history.inputs.iter().any(|v| crate::digest(&root.join(&v.path)).ok().as_deref() != Some(&v.sha256)) {
        "mapping-changed"
    } else if current.as_ref().is_ok_and(|files| files != &history.files) {
        "source-changed"
    } else if current.is_err() {
        "source-unavailable"
    } else {
        "current"
    }
    .into();
    if let Some(pipeline) = &mut history.pipeline {
        pipeline.status = if pipeline
            .samples
            .iter()
            .map(|v| &v.evidence)
            .chain(pipeline.audit.iter())
            .any(|v| crate::digest(&root.join(&v.path)).ok().as_deref() != Some(&v.sha256))
        {
            "evidence-changed"
        } else {
            "current"
        }
        .into();
    }
}
