use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::process::{Lane, git, output, repo};
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

impl Mirror {
    fn of(export: &Path, at: &Path) -> Result<Self, String> {
        let src = export.join(".git");
        let meta = fs::symlink_metadata(&src).map_err(|e| format!("{}: {e}", src.display()))?;
        if !meta.is_dir() {
            return Err(format!("{} is not a directory", src.display()));
        }
        for pointer in [
            "objects/info/alternates",
            "objects/info/http-alternates",
            "commondir",
        ] {
            if fs::symlink_metadata(src.join(pointer)).is_ok() {
                return Err(format!(
                    "{} points git elsewhere",
                    src.join(pointer).display()
                ));
            }
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

    fn show(&self, rev: &str, path: &str) -> Option<String> {
        self.text(&["show", &format!("{rev}:{path}")]).ok()
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
    let loose = git_dir.join("refs/heads/main");
    if let Ok(text) = fs::read_to_string(&loose) {
        return Ok(text.trim().to_owned());
    }
    let packed = fs::read_to_string(git_dir.join("packed-refs")).unwrap_or_default();
    packed
        .lines()
        .find_map(|l| l.strip_suffix(" refs/heads/main"))
        .map(str::to_owned)
        .ok_or_else(|| format!("{} has no main branch", git_dir.display()))
}

fn unpack(mirror: &Mirror, rev: &str, paths: &[&str], dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    let archive = mirror
        .git()
        .args(["archive", "--format=tar", rev, "--"])
        .args(paths)
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let stdout = archive.stdout.ok_or("git archive gave no output")?;
    let ok = Command::new("tar")
        .args(["-x", "-m", "-C"])
        .arg(dest)
        .stdin(stdout)
        .status()
        .map_err(|e| e.to_string())?
        .success();
    if ok {
        Ok(())
    } else {
        Err(format!("can't unpack {rev}"))
    }
}

fn writable(path: &str, model: &str) -> bool {
    path.starts_with("crates/clean/types/") || path.starts_with(&format!("{model}/"))
}

struct Import {
    repo: PathBuf,
    domain: String,
    model: String,
    mirror: Mirror,
    root: String,
}

pub fn run(name: &str) -> Result<(String, bool), String> {
    let lane = Lane::of(name)?;
    let repo = repo()?;
    let branch = output(
        Command::new("git")
            .current_dir(&repo)
            .args(["branch", "--show-current"]),
    )?;
    let dirty = output(Command::new("git").current_dir(&repo).args([
        "status",
        "--porcelain",
        "--untracked-files=no",
    ]))?;
    if branch.trim().is_empty() || !dirty.trim().is_empty() {
        return Err("check out the lane branch, with nothing uncommitted, first".to_owned());
    }
    let scratch = Scratch(lane.dir.join("import"));
    let _ = fs::remove_dir_all(&scratch.0);
    fs::create_dir_all(&scratch.0).map_err(|e| e.to_string())?;
    let mirror = Mirror::of(&lane.export, &scratch.0.join("export.git"))?;
    let roots = mirror.lines(&["rev-list", "--max-parents=0", "main"])?;
    let [root] = roots.as_slice() else {
        return Err(format!(
            "the export has {} root commits, not one",
            roots.len()
        ));
    };
    let import = Import {
        domain: name.to_owned(),
        model: format!("crates/clean/{}", name.replace('-', "_")),
        root: root.clone(),
        repo,
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
        let mut refused = false;
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
                if !writable(path, &self.model) {
                    let _ = writeln!(out, "refused {short} {}: touches {path}", subject.trim());
                    refused = true;
                }
                if matches!(mode, "120000" | "160000") {
                    let _ = writeln!(
                        out,
                        "refused {short} {}: {path} is a symlink or submodule",
                        subject.trim()
                    );
                    refused = true;
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
                refused = true;
            }
        }
        if refused {
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
        let show = |path: &str| {
            output(
                Command::new("git")
                    .current_dir(&self.repo)
                    .args(["show", &format!("HEAD:{path}")]),
            )
        };
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
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.repo);
        cmd
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
        let outside: Vec<String> =
            output(
                self.repo_git()
                    .args(["diff", "--name-only", &scaffolded, "HEAD"]),
            )?
            .lines()
            .filter(|p| !writable(p, &self.model))
            .map(str::to_owned)
            .collect();
        if !outside.is_empty() {
            return Err(format!(
                "applied changes outside the clean crates: {}",
                outside.join(" ")
            ));
        }
        self.clippy(&before, commits, out)
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
    use std::path::Path;

    use super::{Mirror, writable};

    #[test]
    fn a_patch_may_touch_only_the_shared_types_and_its_model() {
        let model = "crates/clean/world_haze";
        assert!(writable("crates/clean/types/src/lib.rs", model));
        assert!(writable("crates/clean/world_haze/src/lib.rs", model));
        for path in [
            "crates/clean/world_hazel/src/lib.rs",
            "crates/clean/samus_movement/src/lib.rs",
            "docs/behaviours/world-haze.md",
            "Cargo.toml",
            ".cargo/config.toml",
        ] {
            assert!(!writable(path, model), "{path}");
        }
    }

    #[test]
    fn an_export_whose_git_points_elsewhere_is_refused() {
        let dir = std::env::temp_dir().join(format!("cleanroom-mirror-{}", std::process::id()));
        let info = dir.join("export/.git/objects/info");
        fs::create_dir_all(&info).unwrap();
        fs::write(info.join("alternates"), "/elsewhere/objects\n").unwrap();
        let refused = Mirror::of(&dir.join("export"), &dir.join("mirror.git")).is_err();
        let made = Path::new(&dir.join("mirror.git")).exists();
        fs::remove_dir_all(&dir).unwrap();
        assert!(refused && !made);
    }
}
