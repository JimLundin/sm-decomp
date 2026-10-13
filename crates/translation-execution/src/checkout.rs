//! Trusted detached checkouts. Git and source writes remain supervisor-owned.
use crate::{model::*, rewrite};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn exact_commit(commit: &str) -> bool {
    commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_hexdigit())
}
fn protected(path: &Path) -> Result<()> {
    ensure!(path.is_absolute(), "controller checkout paths must be absolute");
    for ancestor in path.ancestors() {
        ensure!(!ancestor.is_symlink(), "controller path crosses a symlink");
    }
    Ok(())
}
fn git_output(checkout: &Path, argv: &[&str]) -> Result<Output> {
    let result = Command::new("/usr/bin/git")
        .args(["-c", "core.hooksPath=/dev/null", "-c", "commit.gpgsign=false", "-c", "protocol.file.allow=always"])
        .args(argv)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_AUTHOR_NAME", "Translation supervisor")
        .env("GIT_AUTHOR_EMAIL", "translation@localhost")
        .env("GIT_COMMITTER_NAME", "Translation supervisor")
        .env("GIT_COMMITTER_EMAIL", "translation@localhost")
        .env("GIT_AUTHOR_DATE", "2000-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2000-01-01T00:00:00Z")
        .stdin(std::process::Stdio::null())
        .current_dir(checkout)
        .output()?;
    ensure!(result.status.success(), "supervisor Git operation failed: {}", String::from_utf8_lossy(&result.stderr));
    Ok(result)
}
fn git(checkout: &Path, argv: &[&str]) -> Result<String> {
    Ok(String::from_utf8(git_output(checkout, argv)?.stdout)?.trim().into())
}
fn git_dir(checkout: &Path) -> Result<PathBuf> {
    protected(checkout)?;
    ensure!(
        checkout.join(".git").is_dir() && !checkout.join(".git").is_symlink(),
        "supervisor checkout must own isolated Git metadata"
    );
    ensure!(git(checkout, &["symbolic-ref", "-q", "HEAD"]).is_err(), "supervisor checkout must use detached source");
    Ok(checkout.join(".git"))
}
fn lock(checkout: &Path) -> Result<File> {
    let path = git_dir(checkout)?.join("translation.lock");
    ensure!(!path.is_symlink(), "supervisor checkout lock cannot be aliased");
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    file.lock()?;
    Ok(file)
}
fn names(checkout: &Path, argv: &[&str]) -> Result<Vec<String>> {
    let output = git_output(checkout, argv)?;
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .map(|name| Ok(std::str::from_utf8(name)?.to_owned()))
        .collect()
}
fn dirty(checkout: &Path) -> Result<BTreeSet<String>> {
    let mut paths = names(checkout, &["diff", "--name-only", "--no-renames", "-z", "--"])?;
    paths.extend(names(checkout, &["diff", "--cached", "--name-only", "--no-renames", "-z", "--"])?);
    paths.extend(names(checkout, &["ls-files", "--others", "--exclude-standard", "-z"])?);
    Ok(paths.into_iter().collect())
}
fn checked_file(checkout: &Path, path: &str) -> Result<Vec<u8>> {
    ensure!(relative_file(path), "source path is not root-relative");
    let target = checkout.join(path);
    protected(&target)?;
    let metadata = fs::metadata(&target)?;
    ensure!(metadata.is_file(), "source snapshot requires regular files");
    use std::os::unix::fs::MetadataExt;
    ensure!(metadata.nlink() == 1, "source snapshot cannot contain hardlink aliases");
    Ok(fs::read(target)?)
}

/// Create one detached controller checkout at its exact baseline. This does not
/// register a worktree or change the shared checkout's source, index or HEAD.
pub fn create(source_repository: &Path, destination: &Path, baseline: &str) -> Result<()> {
    protected(source_repository)?;
    protected(destination)?;
    ensure!(exact_commit(baseline), "controller baseline must be exact");
    let parent = destination.parent().context("checkout destination parent missing")?;
    fs::create_dir_all(parent)?;
    let intent_path =
        parent.join(format!(".translation-create-{}.json", sha256(destination.as_os_str().as_encoded_bytes())));
    let identity = serde_json::to_vec(&(source_repository, destination, baseline))?;
    let owned = intent_path.exists();
    if owned {
        ensure!(fs::read(&intent_path)? == identity, "checkout creation intent differs");
    } else {
        ensure!(!destination.exists(), "controller destination already exists without creation ownership");
        atomic(&intent_path, &identity)?;
    }
    if destination.join(".git").is_dir() {
        if git_dir(destination).is_ok() {
            ensure!(
                git(destination, &["rev-parse", "HEAD"])? == baseline && dirty(destination)?.is_empty(),
                "existing prepared checkout differs from baseline"
            );
            return Ok(());
        }
        // A clone completed before detached checkout. Its working tree has no
        // tracked changes and every existing tracked file is baseline content.
        fn original_only(root: &Path, path: &Path, baseline: &str) -> Result<()> {
            protected(path)?;
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                if entry.path() == root.join(".git") {
                    continue;
                }
                if entry.path().is_dir() {
                    original_only(root, &entry.path(), baseline)?;
                } else {
                    let relative =
                        entry.path().strip_prefix(root)?.to_str().context("clone source path UTF8")?.to_owned();
                    let original = git_output(root, &["show", &format!("{baseline}:{relative}")])?.stdout;
                    ensure!(checked_file(root, &relative)? == original, "incomplete clone contains unexpected source");
                }
            }
            Ok(())
        }
        original_only(destination, destination, baseline)?;
        git(destination, &["checkout", "--detach", baseline])?;
        ensure!(dirty(destination)?.is_empty(), "recovered prepared source is dirty");
        return Ok(());
    }
    if destination.exists() {
        ensure!(
            owned && destination.is_dir() && fs::read_dir(destination)?.next().is_none(),
            "interrupted clone contains unreconciled files"
        );
        fs::remove_dir(destination)?;
    }
    git(
        parent,
        &[
            "clone",
            "--shared",
            "--no-checkout",
            source_repository.to_str().context("source path UTF8")?,
            destination.to_str().context("destination path UTF8")?,
        ],
    )?;
    git(destination, &["checkout", "--detach", baseline])?;
    ensure!(dirty(destination)?.is_empty(), "new prepared source is dirty");
    Ok(())
}

pub fn snapshot(checkout: &Path, approved_paths: &[String]) -> Result<SourceSnapshot> {
    git_dir(checkout)?;
    ensure!(dirty(checkout)?.is_empty(), "controller source must be clean");
    let commit = git(checkout, &["rev-parse", "HEAD"])?;
    ensure!(exact_commit(&commit), "controller source commit is inexact");
    let tree = git(checkout, &["rev-parse", "HEAD^{tree}"])?;
    let tracked = names(checkout, &["ls-files", "-z"])?;
    let mut full = BTreeMap::new();
    for path in tracked {
        full.insert(path.clone(), sha256(checked_file(checkout, &path)?));
    }
    let mut files_sha256 = BTreeMap::new();
    for path in approved_paths {
        ensure!(!files_sha256.contains_key(path), "duplicate approved file");
        files_sha256.insert(path.clone(), full.get(path).context("approved source is not tracked")?.clone());
    }
    ensure!(!files_sha256.is_empty(), "approved source snapshot is empty");
    Ok(SourceSnapshot { sha256: sealed(&(&commit, &tree, &full))?, commit, files_sha256, clean: true })
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SourceIntent {
    operation_key: String,
    base_commit: String,
    base_source_sha256: String,
    expected_files: BTreeMap<String, String>,
    original_files: BTreeMap<String, String>,
}

fn atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    protected(path)?;
    let pending = path.with_extension("pending");
    protected(&pending)?;
    if pending.exists() {
        fs::remove_file(&pending)?;
    }
    let mut file = OpenOptions::new().create_new(true).write(true).open(&pending)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(pending, path)?;
    File::open(path.parent().context("atomic parent missing")?)?.sync_all()?;
    Ok(())
}
fn source_intent(checkout: &Path, intent: &SourceIntent) -> Result<()> {
    let path = git_dir(checkout)?.join(format!("translation-{}.json", intent.operation_key));
    if path.exists() {
        let saved: SourceIntent = serde_json::from_slice(&fs::read(path)?)?;
        ensure!(saved == *intent, "source operation retry differs from durable intent");
    } else {
        atomic(&path, &serde_json::to_vec(intent)?)?;
    }
    Ok(())
}
fn recorded_intent(checkout: &Path, operation_key: &str) -> Result<Option<SourceIntent>> {
    let path = git_dir(checkout)?.join(format!("translation-{operation_key}.json"));
    protected(&path)?;
    if path.exists() { Ok(Some(serde_json::from_slice(&fs::read(path)?)?)) } else { Ok(None) }
}
fn keyed_commit(checkout: &Path, key: &str) -> Result<bool> {
    Ok(git(checkout, &["show", "-s", "--format=%B", "HEAD"])?
        .lines()
        .any(|line| line == format!("Translation-Operation: {key}")))
}
fn commit(checkout: &Path, intent: &SourceIntent) -> Result<()> {
    let paths: Vec<_> = intent.expected_files.keys().map(String::as_str).collect();
    let mut argv = vec!["add", "--"];
    argv.extend(paths);
    git(checkout, &argv)?;
    let message = format!("Apply Translation scope source\n\nTranslation-Operation: {}", intent.operation_key);
    git(checkout, &["commit", "--no-verify", "-m", &message])?;
    Ok(())
}
fn intended_dirty(checkout: &Path, intent: &SourceIntent) -> Result<()> {
    ensure!(git(checkout, &["rev-parse", "HEAD"])? == intent.base_commit, "source operation HEAD changed");
    for path in dirty(checkout)? {
        ensure!(intent.expected_files.contains_key(&path), "source operation includes foreign dirty path");
    }
    for (path, expected) in &intent.expected_files {
        let current = sha256(checked_file(checkout, path)?);
        ensure!(
            current == *expected || intent.original_files.get(path) == Some(&current),
            "source is neither original nor intended replacement"
        );
    }
    Ok(())
}
fn completed_source(checkout: &Path, intent: &SourceIntent, paths: &[String]) -> Result<SourceSnapshot> {
    ensure!(keyed_commit(checkout, &intent.operation_key)?, "source commit operation trailer differs");
    ensure!(git(checkout, &["rev-parse", "HEAD^"])? == intent.base_commit, "source commit baseline differs");
    let source = snapshot(checkout, paths)?;
    ensure!(source.files_sha256 == intent.expected_files, "committed replacements differ");
    let changed = names(checkout, &["diff", "--name-only", "--no-renames", "-z", &intent.base_commit, "HEAD", "--"])?;
    ensure!(changed.iter().all(|path| intent.expected_files.contains_key(path)), "source commit changed foreign files");
    Ok(source)
}

/// Apply/recover source replacements and make one deterministic scoped commit.
pub fn apply_commit(
    checkout: &Path,
    context: &OperationContext,
    capsule: &RewriteCapsule,
    replacements: &SourceReplacements,
    limits: RewriteLimits,
) -> Result<SourceSnapshot> {
    let _lock = lock(checkout)?;
    ensure!(digest_valid(&context.operation_key), "source operation key must be sealed");
    ensure!(
        capsule.run_id == context.run_id
            && capsule.attempt == context.attempt
            && capsule.policy_sha256 == context.policy_sha256,
        "source operation differs from rewrite attempt identity"
    );
    rewrite::decode(capsule, &serde_json::to_vec(replacements)?, limits)?;
    let paths: Vec<_> = capsule.files.iter().map(|file| file.path.clone()).collect();
    let mut expected: BTreeMap<_, _> =
        capsule.files.iter().map(|file| (file.path.clone(), file.sha256.clone())).collect();
    for file in &replacements.files {
        expected.insert(file.path.clone(), sha256(file.content.as_bytes()));
    }
    let intent = if let Some(saved) = recorded_intent(checkout, &context.operation_key)? {
        ensure!(
            saved.expected_files == expected && saved.base_source_sha256 == capsule.source_sha256,
            "replacement retry differs"
        );
        saved
    } else {
        let before = snapshot(checkout, &paths)?;
        ensure!(
            before.sha256 == capsule.source_sha256
                && before.files_sha256
                    == capsule.files.iter().map(|file| (file.path.clone(), file.sha256.clone())).collect(),
            "capsule source changed before application"
        );
        let intent = SourceIntent {
            operation_key: context.operation_key.clone(),
            base_commit: before.commit,
            base_source_sha256: before.sha256,
            expected_files: expected,
            original_files: before.files_sha256,
        };
        source_intent(checkout, &intent)?;
        intent
    };
    if keyed_commit(checkout, &intent.operation_key)? {
        return completed_source(checkout, &intent, &paths);
    }
    intended_dirty(checkout, &intent)?;
    for replacement in &replacements.files {
        let current = sha256(checked_file(checkout, &replacement.path)?);
        if current == sha256(replacement.content.as_bytes()) {
            continue;
        }
        ensure!(current == replacement.base_sha256, "source changed before replacement recovery");
        atomic(&checkout.join(&replacement.path), replacement.content.as_bytes())?;
    }
    commit(checkout, &intent)?;
    completed_source(checkout, &intent, &paths)
}

/// Copy only the verified owned files into a distinct controller checkout. Its
/// expected HEAD is checked before writes; an exact completed retry is reused.
pub fn integrate(
    candidate_checkout: &Path,
    integration_checkout: &Path,
    context: &OperationContext,
    candidate: &SourceSnapshot,
    expected_head: &str,
    approved_paths: &[String],
) -> Result<SourceSnapshot> {
    ensure!(
        candidate_checkout != integration_checkout
            && exact_commit(expected_head)
            && digest_valid(&context.operation_key),
        "integration needs distinct checkouts and exact identities"
    );
    let candidate_now = snapshot(candidate_checkout, approved_paths)?;
    ensure!(candidate_now == *candidate, "candidate source changed before integration");
    let _lock = lock(integration_checkout)?;
    let original = if let Some(saved) = recorded_intent(integration_checkout, &context.operation_key)? {
        ensure!(
            saved.base_commit == expected_head && saved.expected_files == candidate.files_sha256,
            "integration retry source differs"
        );
        saved
    } else {
        let before = snapshot(integration_checkout, approved_paths)?;
        ensure!(before.commit == expected_head, "integration source HEAD differs from expected clean freeze");
        let intent = SourceIntent {
            operation_key: context.operation_key.clone(),
            base_commit: before.commit,
            base_source_sha256: before.sha256,
            expected_files: candidate.files_sha256.clone(),
            original_files: before.files_sha256,
        };
        source_intent(integration_checkout, &intent)?;
        intent
    };
    if keyed_commit(integration_checkout, &context.operation_key)? {
        return completed_source(integration_checkout, &original, approved_paths);
    }
    intended_dirty(integration_checkout, &original)?;
    for (path, expected) in &original.expected_files {
        let bytes = checked_file(candidate_checkout, path)?;
        ensure!(sha256(&bytes) == *expected, "candidate file changed during integration");
        if sha256(checked_file(integration_checkout, path)?) != *expected {
            atomic(&integration_checkout.join(path), &bytes)?;
        }
    }
    ensure!(!dirty(integration_checkout)?.is_empty(), "integration contains no changed owned source");
    commit(integration_checkout, &original)?;
    completed_source(integration_checkout, &original, approved_paths)
}

/// Stable protected publication. An identical retry authenticates original bytes.
pub fn publish_completion(path: &Path, context: &OperationContext, completion: &Completion) -> Result<Publication> {
    protected(path)?;
    ensure!(
        context.run_id == completion.run_id
            && context.attempt == completion.attempt
            && context.policy_sha256 == completion.policy_sha256,
        "completion publication lineage differs"
    );
    let bytes = serde_json::to_vec(completion)?;
    if path.exists() {
        ensure!(fs::read(path)? == bytes, "completion publication retry changed accepted result")
    } else {
        fs::create_dir_all(path.parent().context("publication parent missing")?)?;
        atomic(path, &bytes)?;
    }
    Ok(Publication {
        seal: OutcomeSeal {
            operation_key: context.operation_key.clone(),
            policy_sha256: context.policy_sha256.clone(),
            source_sha256: completion.integrated.sha256.clone(),
            evidence_sha256: sha256(&bytes),
        },
        observation_ids: vec![format!("{}-integrated", context.operation_key)],
    })
}

/// Preserve exact source paths before an implementer checkout is retired. Caller
/// authenticates evidence against these retained snapshots before deletion.
pub fn retain_source(checkout: &Path, retained: &Path, paths: &[String]) -> Result<BTreeMap<String, String>> {
    protected(retained)?;
    let source = snapshot(checkout, paths)?;
    for path in paths {
        let target = retained.join(path);
        protected(&target)?;
        fs::create_dir_all(target.parent().context("source parent missing")?)?;
        let bytes = checked_file(checkout, path)?;
        if target.exists() {
            ensure!(fs::read(&target)? == bytes, "retained source differs")
        } else {
            atomic(&target, &bytes)?;
        }
    }
    Ok(source.files_sha256)
}
