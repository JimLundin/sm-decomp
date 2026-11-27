use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::process::{Lane, git, host_git, inode, output, repo};
use crate::scan::Class;
use crate::{mirror, rules, scan, vocabulary};

const CAPABILITY: &[&str] = &[
    "-A",
    "warnings",
    "-A",
    "clippy::all",
    "-A",
    "clippy::pedantic",
    "-A",
    "clippy::nursery",
    "-A",
    "clippy::restriction",
    "-A",
    "missing_docs",
];

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Mirror(PathBuf);

/// What a session's git leaves in `.git`, with `config`, `info` and `hooks`, which the box
/// mounts read-only.
const GIT_ENTRIES: &[&str] = &[
    "objects",
    "refs",
    "HEAD",
    "index",
    "logs",
    "COMMIT_EDITMSG",
    "ORIG_HEAD",
    "config",
    "info",
    "hooks",
];
pub const GIT_READ_ONLY: &[&str] = &["config", "info", "hooks"];
const MIRROR_MAX_MB: u64 = 1024;

/// Everything in an export's `.git` that a session's git work doesn't leave, one a line: a
/// `.git` other than the folder `cleanroom export` made, a configuration other than the one it
/// wrote, anything in `info` or `hooks`, an entry beyond `GIT_ENTRIES`, anything but files and
/// folders, and alternates, which point git elsewhere.
pub fn git_breaches(lane: &Lane) -> Result<Vec<String>, String> {
    let src = lane.export.join(".git");
    let meta = fs::symlink_metadata(&src).map_err(|e| format!("{}: {e}", src.display()))?;
    if !meta.is_dir() {
        return Ok(vec![format!("{} is not a directory", src.display())]);
    }
    let read = |path: &Path| fs::read(path).map_err(|e| format!("{}: {e}", path.display()));
    if read(&lane.git_inode())? != inode(&meta).into_bytes() {
        return Ok(vec![format!(
            "{} is not the folder `cleanroom export` made",
            src.display()
        )]);
    }
    let mut found = Vec::new();
    if fs::read(src.join("config")).ok() != Some(read(&lane.git_config())?) {
        found.push(format!(
            "{} is not the exported configuration",
            src.join("config").display()
        ));
    }
    for folder in ["info", "hooks"] {
        let dir = src.join(folder);
        let empty = fs::symlink_metadata(&dir).is_ok_and(|m| m.is_dir())
            && fs::read_dir(&dir).is_ok_and(|mut d| d.next().is_none());
        if !empty {
            found.push(format!("{} is not the empty exported folder", dir.display()));
        }
    }
    for entry in fs::read_dir(&src).map_err(|e| format!("{}: {e}", src.display()))? {
        let name = entry.map_err(|e| e.to_string())?.file_name();
        if !GIT_ENTRIES.iter().any(|e| name == *e) {
            found.push(format!("{} is not git's", src.join(name).display()));
        }
    }
    let mut stack = vec![src.clone()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let entry = entry.map_err(|e| e.to_string())?;
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            let path = entry.path();
            if kind.is_dir() {
                stack.push(path);
            } else if !kind.is_file() {
                found.push(format!("{} is not a file or folder", path.display()));
            }
        }
    }
    for pointer in ["objects/info/alternates", "objects/info/http-alternates"] {
        if fs::symlink_metadata(src.join(pointer)).is_ok() {
            found.push(format!(
                "{} points git elsewhere",
                src.join(pointer).display()
            ));
        }
    }
    found.sort();
    Ok(found)
}

fn size(dir: &Path) -> Result<u64, String> {
    let mut total = 0u64;
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let meta = entry.metadata().map_err(|e| e.to_string())?;
        total = total.saturating_add(if meta.is_dir() {
            size(&entry.path())?
        } else {
            meta.len()
        });
    }
    Ok(total)
}

impl Mirror {
    fn of(lane: &Lane, at: &Path, max_mb: u64) -> Result<Self, String> {
        let src = lane.export.join(".git");
        let breaches = git_breaches(lane)?;
        if !breaches.is_empty() {
            return Err(format!(
                "the export's .git holds what no session's git leaves:\n{}",
                breaches.join("\n")
            ));
        }
        let bytes = size(&src.join("objects"))?;
        if bytes > max_mb.saturating_mul(1 << 20) {
            return Err(format!(
                "{} holds {bytes} bytes, over CLEANROOM_MIRROR_MAX_MB={max_mb}",
                src.join("objects").display()
            ));
        }
        output(
            git(Path::new("."))
                .args(["init", "-q", "--bare", "--template="])
                .arg(at),
        )?;
        copy_files(&src.join("objects"), &at.join("objects"))?;
        let head = main_ref(&src)?;
        let mirror = Self(at.to_owned());
        output(mirror.git().args(["update-ref", "refs/heads/main", &head]))?;
        output(
            mirror
                .git()
                .args(["symbolic-ref", "HEAD", "refs/heads/main"]),
        )?;
        Ok(mirror)
    }

    fn git(&self) -> Command {
        git(&self.0)
    }

    fn text(&self, args: &[&str]) -> Result<String, String> {
        output(self.git().args(args))
    }

    /// A blob as stored: `cat-file` applies no attributes, so no filter or `export-ignore` the
    /// export wrote changes what the checks read.
    fn blob(&self, object: &str) -> Result<Vec<u8>, String> {
        let out = self
            .git()
            .args(["cat-file", "blob", object])
            .stdin(Stdio::null())
            .output()
            .map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(out.stdout)
        } else {
            Err(format!("can't read {object}"))
        }
    }

    fn show(&self, rev: &str, path: &str) -> Option<String> {
        String::from_utf8(self.blob(&format!("{rev}:{path}")).ok()?).ok()
    }

    fn lines(&self, args: &[&str]) -> Result<Vec<String>, String> {
        Ok(self.text(args)?.lines().map(str::to_owned).collect())
    }
}

fn copy_files(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        let target = to.join(entry.file_name());
        if kind.is_dir() {
            copy_files(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
        } else {
            return Err(format!(
                "{} is not a file or folder",
                entry.path().display()
            ));
        }
    }
    Ok(())
}

fn main_ref(git_dir: &Path) -> Result<String, String> {
    fs::read_to_string(git_dir.join("refs/heads/main"))
        .map(|text| text.trim().to_owned())
        .map_err(|_| format!("{} has no main branch", git_dir.display()))
}

/// Writes the plain files under `paths` at `rev` into `dest`, each blob read as stored.
fn unpack(mirror: &Mirror, rev: &str, paths: &[&str], dest: &Path) -> Result<(), String> {
    let listing = mirror.text(&[&["ls-tree", "-r", "-z", rev, "--"][..], paths].concat())?;
    for entry in listing.split('\0').filter(|e| !e.is_empty()) {
        let (meta, path) = entry.split_once('\t').unwrap_or((entry, ""));
        let [mode, _, object] = meta.split(' ').collect::<Vec<_>>()[..] else {
            return Err(format!("can't read the tree entry {entry}"));
        };
        let plain = Path::new(path)
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_)));
        if !matches!(mode, "100644" | "100755") || !plain {
            return Err(format!("{path} in {rev} is not a plain file"));
        }
        let target = dest.join(path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&target, mirror.blob(object)?).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Why a commit may not touch `path`, or None. A commit changes the shared types or the
/// model, and in them only the manifest and the Rust sources under `src/` and `tests/`.
fn refused(path: &str, model: &str) -> Option<&'static str> {
    let Some(rest) = path
        .strip_prefix("crates/clean/types/")
        .or_else(|| path.strip_prefix(model)?.strip_prefix('/'))
    else {
        return Some("is outside the shared types and the model");
    };
    let parts: Vec<&str> = rest.split('/').collect();
    if parts.contains(&".gitattributes") {
        return Some("is an attributes file");
    }
    if parts.contains(&".cargo") {
        return Some("is cargo configuration");
    }
    if parts.iter().any(|p| p.starts_with('.') || p.is_empty()) {
        return Some("is hidden");
    }
    if parts.last() == Some(&"build.rs") {
        return Some("is a build script");
    }
    match parts.as_slice() {
        ["Cargo.toml"] => None,
        ["src" | "tests", .., file] if Path::new(file).extension().is_some_and(|e| e == "rs") => {
            None
        }
        _ => Some("is not the manifest or a Rust source under src/ or tests/"),
    }
}

struct Import {
    repo: PathBuf,
    domain: String,
    model: String,
    mirror: Mirror,
    root: String,
}

pub fn run(name: &str) -> Result<(String, bool), String> {
    run_at(&repo()?, &Lane::of(name)?, name)
}

fn run_at(repo: &Path, lane: &Lane, name: &str) -> Result<(String, bool), String> {
    let branch = output(host_git(repo).args(["branch", "--show-current"]))?;
    let dirty = output(host_git(repo).args(["status", "--porcelain", "--untracked-files=no"]))?;
    if branch.trim().is_empty() || !dirty.trim().is_empty() {
        return Err("check out the lane branch, with nothing uncommitted, first".to_owned());
    }
    let scratch = Scratch(lane.dir.join("import"));
    let _ = fs::remove_dir_all(&scratch.0);
    fs::create_dir_all(&scratch.0).map_err(|e| e.to_string())?;
    let max_mb = match std::env::var("CLEANROOM_MIRROR_MAX_MB") {
        Ok(mb) => mb
            .parse()
            .map_err(|e| format!("CLEANROOM_MIRROR_MAX_MB={mb}: {e}"))?,
        Err(_) => MIRROR_MAX_MB,
    };
    let mirror = Mirror::of(lane, &scratch.0.join("export.git"), max_mb)?;
    let roots = mirror.lines(&["rev-list", "--max-parents=0", "main"])?;
    let exported = fs::read_to_string(lane.export_root())
        .map_err(|e| format!("{}: {e}", lane.export_root().display()))?;
    if roots.as_slice() != [exported.trim()] {
        return Err(format!(
            "the export's history doesn't start at its export commit {}",
            exported.trim()
        ));
    }
    let import = Import {
        domain: name.to_owned(),
        model: format!("crates/clean/{}", name.replace('-', "_")),
        root: exported.trim().to_owned(),
        repo: repo.to_owned(),
        mirror,
    };
    let mut out = String::new();
    let commits = import.commits(&mut out)?;
    import.rules(&scratch.0.join("head"), &mut out)?;
    import.names(&mut out)?;
    import.mirror_scan(&mut out)?;
    import.apply(commits, &mut out)
}

fn refuse(out: &str, why: &str) -> Result<(), String> {
    Err(format!("{out}{why}; nothing applied"))
}

impl Import {
    fn commits(&self, out: &mut String) -> Result<usize, String> {
        if !self
            .mirror
            .text(&["rev-list", "--merges", "main"])?
            .trim()
            .is_empty()
        {
            return Err("the export holds a merge commit".to_owned());
        }
        let commits =
            self.mirror
                .lines(&["rev-list", "--reverse", &format!("{}..main", self.root)])?;
        if commits.is_empty() {
            return Err("no commits after the export's".to_owned());
        }
        let mut refused_any = false;
        for c in &commits {
            let subject = self.mirror.text(&["log", "-1", "--format=%s", c])?;
            let short = c.get(..9).unwrap_or(c);
            for line in self.mirror.lines(&[
                "diff-tree",
                "-r",
                "--no-commit-id",
                "--no-renames",
                "--raw",
                c,
            ])? {
                let (meta, path) = line.split_once('\t').unwrap_or((&line, ""));
                let mode = meta.split_whitespace().nth(1).unwrap_or_default();
                if let Some(why) = refused(path, &self.model) {
                    let _ = writeln!(out, "refused {short} {}: {path} {why}", subject.trim());
                    refused_any = true;
                }
                if matches!(mode, "120000" | "160000") {
                    let _ = writeln!(
                        out,
                        "refused {short} {}: {path} is a symlink or submodule",
                        subject.trim()
                    );
                    refused_any = true;
                }
            }
            let role =
                self.mirror
                    .text(&["log", "-1", "--format=%(trailers:key=Role,valueonly)", c])?;
            if role.trim() != "implementer" {
                let _ = writeln!(
                    out,
                    "refused {short} {}: no 'Role: implementer' trailer",
                    subject.trim()
                );
                refused_any = true;
            }
        }
        if refused_any {
            refuse(out, "a commit reaches outside the clean crates")?;
        }
        Ok(commits.len())
    }

    fn rules(&self, head: &Path, out: &mut String) -> Result<(), String> {
        unpack(&self.mirror, "main", &["crates"], head)?;
        fs::copy(self.repo.join("Cargo.toml"), head.join("Cargo.toml"))
            .map_err(|e| e.to_string())?;
        let found = rules::check(head, &rules::Policy::load(&self.repo)?)?;
        for finding in &found {
            let _ = writeln!(out, "{finding}");
        }
        if !found.is_empty() {
            refuse(
                out,
                &format!("the code rules refuse {} things", found.len()),
            )?;
        }
        out.push_str("import: the code rules pass\n");
        Ok(())
    }

    fn names(&self, out: &mut String) -> Result<(), String> {
        let (lex, allow) = vocabulary(&self.repo)?;
        let changed = self.mirror.lines(&[
            "diff",
            "--name-only",
            "--no-renames",
            "--diff-filter=d",
            &self.root,
            "main",
            "--",
            "crates/clean",
        ])?;
        let mut added = 0usize;
        for path in &changed {
            let head = self.mirror.show("main", path).unwrap_or_default();
            let start = self.mirror.show(&self.root, path);
            for f in scan::added(&lex, &allow, path, start.as_deref(), &head) {
                let _ = writeln!(
                    out,
                    "scan: {}:{}: {}: {}",
                    f.path, f.line, f.class, f.detail
                );
                added = added.saturating_add(1);
            }
        }
        if added > 0 {
            refuse(
                out,
                &format!("the name scan finds {added} hits the export's start doesn't hold"),
            )?;
        }
        let _ = writeln!(
            out,
            "import: the name scan finds nothing new in {} changed files",
            changed.len()
        );
        Ok(())
    }

    fn model_sources(&self, rev: &str) -> Result<Vec<(String, String)>, String> {
        let files = self
            .mirror
            .lines(&["ls-tree", "-r", "--name-only", rev, "--", &self.model])?;
        Ok(files
            .into_iter()
            .filter(|f| Path::new(f).extension().is_some_and(|e| e == "rs"))
            .filter_map(|f| self.mirror.show(rev, &f).map(|text| (f, text)))
            .collect())
    }

    fn mirror_scan(&self, out: &mut String) -> Result<(), String> {
        let show =
            |path: &str| output(host_git(&self.repo).args(["show", &format!("HEAD:{path}")]));
        let spec_path = format!("docs/behaviours/{}.md", self.domain);
        let spec = show(&spec_path)
            .map_err(|_| format!("no spec {spec_path} on the branch; nothing applied"))?;
        let decode =
            show(&format!("docs/behaviours/{}.decode.md", self.domain)).unwrap_or_default();
        let (lex, _) = vocabulary(&self.repo)?;
        let entry_points = |rev: &str| -> Result<Vec<(String, String)>, String> {
            let mut failed = false;
            let (found, _) = mirror::check(
                &lex,
                &spec_path,
                &spec,
                &decode,
                &self.model_sources(rev)?,
                &mut failed,
            );
            Ok(found
                .into_iter()
                .filter(|f| f.class == Class::EntryPoint)
                .map(|f| (f.path, f.detail))
                .collect())
        };
        let start = entry_points(&self.root)?;
        let new: Vec<_> = entry_points("main")?
            .into_iter()
            .filter(|f| !start.contains(f))
            .collect();
        for (path, detail) in &new {
            let _ = writeln!(out, "mirror: {path}: {detail}");
        }
        if !new.is_empty() {
            refuse(
                out,
                "a new function opens with an entry whose source is one routine",
            )?;
        }
        out.push_str("import: the mirror scan finds no new single-routine function\n");
        Ok(())
    }

    fn repo_git(&self) -> Command {
        host_git(&self.repo)
    }

    fn cargo(&self) -> Command {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&self.repo);
        cmd
    }

    fn head(&self) -> Result<String, String> {
        Ok(output(self.repo_git().args(["rev-parse", "HEAD"]))?
            .trim()
            .to_owned())
    }

    fn apply(&self, commits: usize, out: &mut String) -> Result<(String, bool), String> {
        let before = self.head()?;
        let missing: Vec<&str> = ["crates/clean/types", self.model.as_str()]
            .into_iter()
            .filter(|dir| !self.repo.join(dir).join("Cargo.toml").exists())
            .collect();
        if !missing.is_empty() {
            unpack(&self.mirror, &self.root, &missing, &self.repo)?;
            output(
                self.cargo()
                    .args(["metadata", "-q", "--offline", "--format-version", "1"]),
            )?;
            output(
                self.repo_git()
                    .arg("add")
                    .arg("--")
                    .args(&missing)
                    .arg("Cargo.lock"),
            )?;
            output(self.repo_git().args([
                "commit",
                "-q",
                "-m",
                &format!(
                    "Clean room: the empty clean crates for {}, as exported [exact]",
                    self.domain
                ),
            ]))?;
            let _ = writeln!(out, "import: scaffold committed for {}", missing.join(" "));
        }
        let scaffolded = self.head()?;
        let patches = self
            .mirror
            .git()
            .args(["format-patch", "--no-textconv", "--no-ext-diff", "--stdout"])
            .arg(format!("{}..main", self.root))
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        let stdout = patches.stdout.ok_or("git format-patch gave no output")?;
        let applied = self
            .repo_git()
            .args(["am", "-q", "--3way", "--include=crates/clean/types/*"])
            .arg(format!("--include={}/*", self.model))
            .stdin(stdout)
            .status()
            .map_err(|e| e.to_string())?
            .success();
        if !applied {
            let _ = self.repo_git().args(["am", "--abort"]).status();
            return Err(format!(
                "{out}a patch didn't apply; the branch is at {scaffolded}"
            ));
        }
        self.applied(&before, &scaffolded, out)?;
        self.clippy(&before, commits, out)
    }

    /// Checks what `git am` left: no path outside the clean crates, and the code rules over the
    /// branch's crates as applied, since a three-way merge can leave other code than the export
    /// holds. On a refusal the branch goes back to BEFORE.
    fn applied(&self, before: &str, scaffolded: &str, out: &mut String) -> Result<(), String> {
        let outside: Vec<String> =
            output(
                self.repo_git()
                    .args(["diff", "--name-only", scaffolded, "HEAD"]),
            )?
            .lines()
            .filter(|p| refused(p, &self.model).is_some())
            .map(str::to_owned)
            .collect();
        let found = rules::check(&self.repo, &rules::Policy::load(&self.repo)?)?;
        if outside.is_empty() && found.is_empty() {
            return Ok(());
        }
        output(self.repo_git().args(["reset", "-q", "--hard", before]))?;
        for finding in &found {
            let _ = writeln!(out, "{finding}");
        }
        if !outside.is_empty() {
            let _ = writeln!(
                out,
                "applied changes outside the clean crates: {}",
                outside.join(" ")
            );
        }
        Err(format!(
            "{out}the applied branch fails the checks; nothing applied"
        ))
    }

    fn clippy(
        &self,
        before: &str,
        commits: usize,
        out: &mut String,
    ) -> Result<(String, bool), String> {
        let package = self.model.trim_start_matches("crates/clean/");
        let capability = self
            .cargo()
            .env("CLIPPY_CONF_DIR", self.repo.join("crates/clean"))
            .args([
                "clippy",
                "--locked",
                "-p",
                "types",
                "-p",
                package,
                "--all-targets",
                "--no-deps",
                "--",
            ])
            .args(CAPABILITY)
            .output()
            .map_err(|e| e.to_string())?;
        if !capability.status.success() {
            out.push_str(&errors(&capability.stderr));
            output(self.repo_git().args(["reset", "-q", "--hard", before]))?;
            return Err(format!(
                "{out}the capability rule refused the export's crates; nothing applied"
            ));
        }
        let mut packages = Vec::new();
        for entry in fs::read_dir(self.repo.join("crates/clean")).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.join("Cargo.toml").exists()
                && let Some(name) = path.file_name()
            {
                packages.push(name.to_string_lossy().into_owned());
            }
        }
        packages.sort();
        let mut cmd = self.cargo();
        cmd.env_remove("CLIPPY_CONF_DIR").args([
            "clippy",
            "--locked",
            "--keep-going",
            "--all-targets",
            "--no-deps",
        ]);
        for p in &packages {
            cmd.args(["-p", p]);
        }
        let lint = cmd
            .args(["--", "-D", "warnings"])
            .output()
            .map_err(|e| e.to_string())?;
        let applied = format!(
            "{}..{}",
            before.get(..9).unwrap_or(before),
            self.head()?.get(..9).unwrap_or_default()
        );
        if !lint.status.success() {
            out.push_str("IMPORT CLIPPY FAIL\n");
            out.push_str(&errors(&lint.stderr));
            let _ = writeln!(
                out,
                "the patches are applied ({applied}): commit a mechanical fix as the root, or git reset --hard {before} and send the change back"
            );
            return Ok((std::mem::take(out), true));
        }
        let _ = writeln!(out, "IMPORT OK {commits} commits, {applied}");
        Ok((std::mem::take(out), false))
    }
}

fn errors(stderr: &[u8]) -> String {
    let mut out = String::new();
    for line in String::from_utf8_lossy(stderr)
        .lines()
        .filter(|l| l.starts_with("error") || l.starts_with("warning") || l.contains("-->"))
        .take(20)
    {
        let _ = writeln!(out, "{line}");
    }
    out
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::{Import, Mirror, git_breaches, refused, run_at};
    use crate::process::{Lane, git, output};

    const MODEL: &str = "crates/clean/world_haze";

    #[test]
    fn a_patch_may_touch_only_the_sources_and_manifests_of_the_shared_types_and_its_model() {
        for path in [
            "crates/clean/types/src/lib.rs",
            "crates/clean/types/Cargo.toml",
            "crates/clean/world_haze/Cargo.toml",
            "crates/clean/world_haze/src/lib.rs",
            "crates/clean/world_haze/src/motion/step.rs",
            "crates/clean/world_haze/tests/vectors.rs",
        ] {
            assert_eq!(refused(path, MODEL), None, "{path}");
        }
        for path in [
            "crates/clean/world_hazel/src/lib.rs",
            "crates/clean/samus_movement/src/lib.rs",
            "docs/behaviours/world-haze.md",
            "Cargo.toml",
            ".cargo/config.toml",
            "crates/clean/world_haze/.cargo/config.toml",
            "crates/clean/world_haze/.gitattributes",
            "crates/clean/world_haze/src/.gitattributes",
            "crates/clean/world_haze/build.rs",
            "crates/clean/world_haze/src/build.rs",
            "crates/clean/world_haze/src/.hidden.rs",
            "crates/clean/world_haze/src/lib.txt",
            "crates/clean/world_haze/benches/b.rs",
            "crates/clean/world_haze/Cargo.lock",
        ] {
            assert!(refused(path, MODEL).is_some(), "{path}");
        }
    }

    /// An export of `world-haze` with its lane folder and a lane branch to import onto, all in
    /// one temporary folder that is removed on drop.
    struct Round {
        base: PathBuf,
        lane: Lane,
        repo: PathBuf,
    }

    impl Drop for Round {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.base);
        }
    }

    fn run(dir: &Path, args: &[&str]) -> String {
        output(git(dir).args(args)).unwrap()
    }

    fn write(dir: &Path, path: &str, text: &str) {
        let target = dir.join(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, text).unwrap();
    }

    impl Round {
        fn new(tag: &str) -> Self {
            let base =
                std::env::temp_dir().join(format!("cleanroom-import-{tag}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&base);
            let lane = Lane {
                export: base.join("world-haze"),
                dir: base.join("world-haze.lane"),
            };
            let repo = base.join("repo");
            for dir in [&lane.export, &lane.dir, &repo] {
                fs::create_dir_all(dir).unwrap();
            }
            for dir in [&lane.dir, &repo] {
                run(dir, &["init", "-q", "--template=", "-b", "main"]);
            }
            crate::export::init(&lane.export).unwrap();
            lane.record_git().unwrap();
            for krate in ["types", "world_haze"] {
                let manifest = format!("[package]\nname = \"{krate}\"\n");
                write(
                    &lane.export,
                    &format!("crates/clean/{krate}/Cargo.toml"),
                    &manifest,
                );
                write(
                    &lane.export,
                    &format!("crates/clean/{krate}/src/lib.rs"),
                    "//! A crate.\n",
                );
            }
            run(&lane.export, &["add", "-A"]);
            run(
                &lane.export,
                &["commit", "-q", "-m", "Clean-room export: world-haze"],
            );
            fs::write(
                lane.export_root(),
                run(&lane.export, &["rev-parse", "HEAD"]),
            )
            .unwrap();
            write(&repo, "README.md", "lane\n");
            run(&repo, &["add", "-A"]);
            run(&repo, &["commit", "-q", "-m", "lane"]);
            Self { base, lane, repo }
        }

        fn commit(&self, message: &str) {
            run(&self.lane.export, &["add", "-A"]);
            run(&self.lane.export, &["commit", "-q", "-m", message]);
        }

        /// The import's refusal, after checking it applied nothing.
        fn refusal(&self) -> String {
            let before = run(&self.repo, &["rev-parse", "HEAD"]);
            let result = run_at(&self.repo, &self.lane, "world-haze");
            assert_eq!(run(&self.repo, &["rev-parse", "HEAD"]), before);
            assert_eq!(run(&self.repo, &["status", "--porcelain"]), "");
            result.expect_err("the import was refused")
        }
    }

    fn copy(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                copy(&entry.path(), &to.join(entry.file_name()));
            } else {
                fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
            }
        }
    }

    const TRAILER: &str = "model: a change\n\nRole: implementer";

    #[test]
    fn the_import_refuses_a_commit_outside_the_model() {
        let round = Round::new("outside");
        write(
            &round.lane.export,
            "docs/behaviours/world-haze.md",
            "changed\n",
        );
        round.commit(TRAILER);
        assert!(
            round
                .refusal()
                .contains("docs/behaviours/world-haze.md is outside")
        );
    }

    #[test]
    fn the_import_refuses_a_symlink() {
        let round = Round::new("symlink");
        std::os::unix::fs::symlink(
            "/etc/passwd",
            round.lane.export.join(MODEL).join("src/link.rs"),
        )
        .unwrap();
        round.commit(TRAILER);
        assert!(
            round
                .refusal()
                .contains("src/link.rs is a symlink or submodule")
        );
    }

    #[test]
    fn the_import_refuses_a_submodule() {
        let round = Round::new("submodule");
        let head = run(&round.lane.export, &["rev-parse", "HEAD"]);
        let entry = format!("160000,{},{MODEL}/src/sub", head.trim());
        run(
            &round.lane.export,
            &["update-index", "--add", "--cacheinfo", &entry],
        );
        run(&round.lane.export, &["commit", "-q", "-m", TRAILER]);
        assert!(
            round
                .refusal()
                .contains("src/sub is a symlink or submodule")
        );
    }

    #[test]
    fn the_import_refuses_a_commit_without_the_role_trailer() {
        let round = Round::new("trailer");
        write(
            &round.lane.export,
            &format!("{MODEL}/src/lib.rs"),
            "//! Changed.\n",
        );
        round.commit("model: a change");
        assert!(round.refusal().contains("no 'Role: implementer' trailer"));
    }

    #[test]
    fn the_import_refuses_an_attributes_file() {
        let round = Round::new("attributes");
        write(
            &round.lane.export,
            &format!("{MODEL}/.gitattributes"),
            "*.rs -diff\n",
        );
        round.commit(TRAILER);
        assert!(
            round
                .refusal()
                .contains("/.gitattributes is an attributes file")
        );
    }

    #[test]
    fn the_import_refuses_a_build_script_an_attributes_file_hides() {
        let round = Round::new("build");
        write(
            &round.lane.export,
            &format!("{MODEL}/.gitattributes"),
            "build.rs export-ignore\n",
        );
        write(
            &round.lane.export,
            &format!("{MODEL}/build.rs"),
            "fn main() {}\n",
        );
        round.commit(TRAILER);
        assert!(round.refusal().contains("/build.rs is a build script"));
    }

    #[test]
    fn the_import_refuses_a_hidden_file() {
        let round = Round::new("hidden");
        write(
            &round.lane.export,
            &format!("{MODEL}/src/.hidden.rs"),
            "fn f() {}\n",
        );
        round.commit(TRAILER);
        assert!(round.refusal().contains("src/.hidden.rs is hidden"));
    }

    #[test]
    fn the_import_refuses_a_history_that_does_not_start_at_the_export() {
        let round = Round::new("root");
        run(
            &round.lane.export,
            &["checkout", "-q", "--orphan", "forged"],
        );
        round.commit(TRAILER);
        run(&round.lane.export, &["branch", "-q", "-M", "main"]);
        assert!(
            round
                .refusal()
                .contains("doesn't start at its export commit")
        );
    }

    #[test]
    fn the_import_refuses_an_export_whose_git_holds_more_than_a_session_leaves() {
        let round = Round::new("breach");
        write(
            &round.lane.export,
            &format!("{MODEL}/src/lib.rs"),
            "//! Changed.\n",
        );
        round.commit(TRAILER);
        let git = round.lane.export.join(".git");
        assert_eq!(
            git_breaches(&round.lane).unwrap(),
            Vec::<String>::new()
        );
        for (made, found) in [
            ("hooks/post-checkout", "is not the empty exported folder"),
            ("info/attributes", "is not the empty exported folder"),
            ("hooks/sub/post-checkout", "is not the empty exported folder"),
            ("config.worktree", "is not git's"),
            ("commondir", "is not git's"),
            ("packed-refs", "is not git's"),
            ("objects/info/alternates", "points git elsewhere"),
        ] {
            write(&git, made, "x\n");
            let refusal = round.refusal();
            assert!(refusal.contains(found), "{made}: {refusal}");
            let _ = fs::remove_file(git.join(made));
        }
        let _ = fs::remove_dir(git.join("hooks/sub"));
        let config = fs::read(git.join("config")).unwrap();
        write(&git, "config", "[core]\n\tfsmonitor = /bin/true\n");
        assert!(round.refusal().contains("is not the exported configuration"));
        fs::write(git.join("config"), &config).unwrap();
        let moved = round.base.join("moved-git");
        fs::rename(&git, &moved).unwrap();
        copy(&moved, &git);
        assert!(
            round
                .refusal()
                .contains("is not the folder `cleanroom export` made")
        );
        fs::remove_dir_all(&git).unwrap();
        fs::rename(&moved, &git).unwrap();
        std::os::unix::fs::symlink("/etc", git.join("refs/link")).unwrap();
        assert!(round.refusal().contains("is not a file or folder"));
    }

    #[test]
    fn the_mirror_refuses_objects_over_its_cap() {
        let round = Round::new("cap");
        let mut state = 0x2545_f491_u32;
        let noise: String = (0..3 << 20)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                char::from(b'a'.saturating_add(u8::try_from(state % 26).unwrap_or(0)))
            })
            .collect();
        write(&round.lane.export, &format!("{MODEL}/src/big.rs"), &noise);
        round.commit(TRAILER);
        let over = Mirror::of(&round.lane, &round.base.join("small.git"), 1);
        let under = Mirror::of(&round.lane, &round.base.join("large.git"), 1024);
        assert!(
            over.err()
                .is_some_and(|e| e.contains("over CLEANROOM_MIRROR_MAX_MB=1"))
        );
        assert!(under.is_ok());
    }

    #[test]
    fn the_code_rules_read_what_an_attributes_file_would_leave_out_of_an_archive() {
        let round = Round::new("archive");
        write(
            &round.lane.export,
            &format!("{MODEL}/.gitattributes"),
            "build.rs export-ignore\n",
        );
        write(
            &round.lane.export,
            &format!("{MODEL}/build.rs"),
            "fn main() {}\n",
        );
        round.commit(TRAILER);
        let mirror = Mirror::of(&round.lane, &round.base.join("mirror.git"), 1024).unwrap();
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let import = Import {
            repo: repo.to_owned(),
            domain: "world-haze".to_owned(),
            model: MODEL.to_owned(),
            mirror,
            root: String::new(),
        };
        let mut out = String::new();
        let refusal = import
            .rules(&round.base.join("head"), &mut out)
            .unwrap_err();
        assert!(refusal.contains("build.rs"), "{refusal}");
    }

    #[test]
    fn the_code_rules_run_again_on_the_applied_branch() {
        let round = Round::new("applied");
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        for path in ["Cargo.toml", "tools/cleanroom/allowed_deps.txt"] {
            write(
                &round.repo,
                path,
                &fs::read_to_string(repo.join(path)).unwrap(),
            );
        }
        run(&round.repo, &["add", "-A"]);
        run(&round.repo, &["commit", "-q", "-m", "trusted"]);
        let before = run(&round.repo, &["rev-parse", "HEAD"]);
        write(&round.repo, &format!("{MODEL}/build.rs"), "fn main() {}\n");
        run(&round.repo, &["add", "-A"]);
        run(&round.repo, &["commit", "-q", "-m", "applied"]);
        let applied = run(&round.repo, &["rev-parse", "HEAD"]);
        let import = Import {
            repo: round.repo.clone(),
            domain: "world-haze".to_owned(),
            model: MODEL.to_owned(),
            mirror: Mirror::of(&round.lane, &round.base.join("mirror.git"), 1024).unwrap(),
            root: String::new(),
        };
        let mut out = String::new();
        let refusal = import
            .applied(before.trim(), applied.trim(), &mut out)
            .unwrap_err();
        assert!(refusal.contains("build.rs"), "{refusal}");
        assert_eq!(run(&round.repo, &["rev-parse", "HEAD"]), before);
    }

    #[test]
    fn host_git_reads_no_attributes_from_outside_the_repository() {
        let round = Round::new("xdg");
        write(&round.base, "xdg/git/attributes", "* export-ignore\n");
        let seen = output(
            crate::process::host_git(&round.repo)
                .env("XDG_CONFIG_HOME", round.base.join("xdg"))
                .args(["check-attr", "export-ignore", "--", "README.md"]),
        )
        .unwrap();
        assert_eq!(seen, "README.md: export-ignore: unspecified\n");
        let isolated = git(&round.repo);
        assert!(
            isolated
                .get_envs()
                .any(|(k, v)| k == "GIT_ATTR_NOSYSTEM" && v == Some("1".as_ref()))
        );
    }
}
