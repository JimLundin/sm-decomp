use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use serde::Deserialize;

use crate::process::{Lane, git, host_git, output, repo};
use crate::rules;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Implementer,
    Reviewer,
}

#[derive(Deserialize)]
struct Root {
    workspace: Workspace,
}

#[derive(Deserialize)]
struct Workspace {
    resolver: String,
    package: toml::Table,
    dependencies: toml::Table,
    lints: toml::Table,
}

pub struct Export<'a> {
    pub domain: &'a str,
    pub commit: &'a str,
    pub role: Role,
}

fn crate_of(domain: &str) -> String {
    domain.replace('-', "_")
}

pub fn lane_name(domain: &str, role: Role) -> String {
    match role {
        Role::Implementer => domain.to_owned(),
        Role::Reviewer => format!("{domain}-review"),
    }
}

impl Export<'_> {
    fn entries(&self) -> Vec<String> {
        let model = format!("crates/clean/{}", crate_of(self.domain));
        let mut entries = vec![
            "crates/clean/types".to_owned(),
            model,
            format!("scenarios/vectors/{}.toml", self.domain),
            "rust-toolchain.toml".to_owned(),
        ];
        entries.extend(match self.role {
            Role::Implementer => vec![
                format!("docs/behaviours/{}.md", self.domain),
                "docs/cleanroom/DESIGN.md".to_owned(),
                ".agents/skills/implementer/SKILL.md".to_owned(),
            ],
            Role::Reviewer => vec![".agents/skills/blind-reviewer/SKILL.md".to_owned()],
        });
        entries
    }

    pub fn write(&self, brief: &Path) -> Result<String, String> {
        let repo = repo()?;
        let lane = Lane::of(&lane_name(self.domain, self.role))?;
        let brief_text =
            fs::read_to_string(brief).map_err(|e| format!("{}: {e}", brief.display()))?;
        let hits = crate::scan_text(&repo, &brief.display().to_string(), &brief_text)?;
        if !hits.is_empty() {
            return Err(format!(
                "the brief names what the clean room hides:\n{hits}"
            ));
        }
        let sha = output(host_git(&repo).args([
            "rev-parse",
            "--verify",
            &format!("{}^{{commit}}", self.commit),
        ]))?
        .trim()
        .to_owned();
        if lane.export.exists() || lane.dir.exists() {
            return Err(format!(
                "{} or {} exists; remove them or choose another name",
                lane.export.display(),
                lane.dir.display()
            ));
        }
        fs::create_dir_all(&lane.export).map_err(|e| e.to_string())?;
        fs::create_dir_all(&lane.dir).map_err(|e| e.to_string())?;
        fs::write(lane.brief(), brief_text).map_err(|e| e.to_string())?;
        self.copy(&repo, &sha, &lane.export)?;
        self.workspace(&repo, &sha, &lane.export)?;
        if self.role == Role::Reviewer {
            leave_out_spec_readers(&lane.export)?;
        } else {
            fs::write(lane.export.join("CLEANROOM.md"), cleanroom_md(self.domain))
                .map_err(|e| e.to_string())?;
        }
        init(&lane.export)?;
        lane.record_git()?;
        output(git(&lane.export).args(["add", "-A"]))?;
        output(git(&lane.export).args([
            "commit",
            "-q",
            "-m",
            &format!("Clean-room export: {}", self.domain),
        ]))?;
        let root = output(git(&lane.export).args(["rev-parse", "HEAD"]))?;
        fs::write(lane.export_root(), root).map_err(|e| e.to_string())?;
        Ok(format!(
            "EXPORT OK {} from {} into {}\n",
            self.domain,
            sha.get(..9).unwrap_or(&sha),
            lane.export.display()
        ))
    }

    fn copy(&self, repo: &Path, sha: &str, dest: &Path) -> Result<(), String> {
        let mut present = Vec::new();
        for entry in self.entries() {
            let listed =
                output(host_git(repo).args(["ls-tree", "--name-only", sha, "--", &entry]))?;
            if !listed.trim().is_empty() {
                present.push(entry);
            } else if !entry.starts_with("crates/clean/") {
                return Err(format!("{entry} is not in {sha}"));
            }
        }
        let archive = host_git(repo)
            .args(["archive", "--format=tar", sha, "--"])
            .args(&present)
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        let stdout = archive.stdout.ok_or("git archive gave no output")?;
        let status = Command::new("tar")
            .args(["-x", "-m", "-C"])
            .arg(dest)
            .stdin(stdout)
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("git archive of {sha} didn't unpack"))
        }
    }

    fn workspace(&self, repo: &Path, sha: &str, dest: &Path) -> Result<(), String> {
        let show = |path: &str| output(host_git(repo).args(["show", &format!("{sha}:{path}")]));
        let root: Root = toml::from_str(&show("Cargo.toml")?).map_err(|e| e.to_string())?;
        let ws = root.workspace;
        let model = crate_of(self.domain);
        let approved = show("tools/cleanroom/allowed_deps.txt")?;
        let mut dependencies = toml::Table::new();
        for name in approved
            .lines()
            .filter_map(|l| l.split('#').next()?.split_whitespace().next())
        {
            let version = ws
                .dependencies
                .get(name)
                .ok_or_else(|| format!("{name} is approved but not a workspace dependency"))?;
            dependencies.insert(name.to_owned(), version.clone());
        }
        let mut workspace = toml::Table::new();
        workspace.insert("resolver".to_owned(), ws.resolver.into());
        workspace.insert(
            "members".to_owned(),
            vec![
                "crates/clean/types".to_owned(),
                format!("crates/clean/{model}"),
            ]
            .into(),
        );
        workspace.insert("package".to_owned(), ws.package.into());
        workspace.insert("dependencies".to_owned(), dependencies.into());
        workspace.insert("lints".to_owned(), ws.lints.into());
        let mut root = toml::Table::new();
        root.insert("workspace".to_owned(), workspace.into());
        let manifest = toml::to_string(&root).map_err(|e| e.to_string())?;
        write(dest, "Cargo.toml", &manifest)?;
        scaffold(
            dest,
            "types",
            "The domain types the clean models share",
            "[dependencies]\n",
        )?;
        scaffold(
            dest,
            &model,
            &format!("The {} model, built from its behaviour spec", self.domain),
            "[dependencies]\ntypes = { path = \"../types\" }\nenum-map.workspace = true\n\n[dev-dependencies]\nserde = { workspace = true, features = [\"derive\"] }\ntoml.workspace = true\n",
        )?;
        write(dest, ".gitignore", "/target\n/vendor\n")?;
        write(dest, "Cargo.lock", &show("Cargo.lock")?)?;
        write(dest, "clippy.toml", &show("crates/clean/.clippy.toml")?)?;
        output(Command::new("cargo").current_dir(dest).args([
            "metadata",
            "-q",
            "--format-version",
            "1",
        ]))?;
        let mut config = "[net]\noffline = true\n".to_owned();
        let lock = fs::read_to_string(dest.join("Cargo.lock")).map_err(|e| e.to_string())?;
        if lock.contains("source = \"registry+") {
            output(Command::new("cargo").current_dir(dest).args([
                "vendor",
                "-q",
                "--locked",
                "vendor/crates-io",
            ]))?;
            config.push_str(
                "\n[source.crates-io]\nreplace-with = \"vendored\"\n\n[source.vendored]\ndirectory = \"vendor/crates-io\"\n",
            );
        }
        write(dest, ".cargo/config.toml", &config)
    }
}

/// The export's git, with the `info` and `hooks` folders the box mounts read-only, and a
/// configuration that never packs on its own: an automatic gc writes `packed-refs` and `gc.log`,
/// which the import refuses.
pub fn init(export: &Path) -> Result<(), String> {
    output(git(export).args(["init", "-q", "--template=", "-b", "main"]))?;
    for folder in ["info", "hooks"] {
        fs::create_dir_all(export.join(".git").join(folder)).map_err(|e| e.to_string())?;
    }
    output(git(export).args(["config", "gc.auto", "0"]))?;
    output(git(export).args(["config", "maintenance.auto", "false"]))?;
    Ok(())
}

fn write(dest: &Path, path: &str, text: &str) -> Result<(), String> {
    let target = dest.join(path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(&target, text).map_err(|e| format!("{}: {e}", target.display()))
}

fn scaffold(dest: &Path, name: &str, description: &str, deps: &str) -> Result<(), String> {
    let folder = format!("crates/clean/{name}");
    if dest.join(&folder).join("Cargo.toml").exists() {
        return Ok(());
    }
    write(
        dest,
        &format!("{folder}/Cargo.toml"),
        &format!(
            "[package]\nname = \"{name}\"\nversion.workspace = true\nedition.workspace = true\nlicense.workspace = true\npublish.workspace = true\ndescription = \"{description}\"\n\n{deps}\n[lints]\nworkspace = true\n"
        ),
    )?;
    write(
        dest,
        &format!("{folder}/src/lib.rs"),
        &format!("//! {description}.\n"),
    )
}

fn leave_out_spec_readers(dest: &Path) -> Result<(), String> {
    let clean = dest.join("crates/clean");
    for krate in fs::read_dir(&clean).map_err(|e| e.to_string())? {
        let krate = krate.map_err(|e| e.to_string())?.path();
        let manifest = fs::read_to_string(krate.join("Cargo.toml")).unwrap_or_default();
        for dir in ["src", "tests"] {
            let mut files = Vec::new();
            rules::rust_files(&krate.join(dir), &mut files)?;
            for file in files {
                let text = fs::read_to_string(&file).map_err(|e| e.to_string())?;
                if !rules::reads_docs(&text) {
                    continue;
                }
                let name = file
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if dir == "src" || manifest.contains(&format!("tests/{name}")) {
                    return Err(format!(
                        "{} reads docs/ and can't be left out of a code-only copy",
                        file.display()
                    ));
                }
                fs::remove_file(&file).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

fn cleanroom_md(domain: &str) -> String {
    let model = crate_of(domain);
    format!(
        "# Clean room: {domain}

This repository is everything you may read. Build the {domain} model from
`docs/behaviours/{domain}.md` and `scenarios/vectors/{domain}.toml` alone; the rules are in
`docs/cleanroom/DESIGN.md` and `.agents/skills/implementer/SKILL.md`. Use nothing you remember
about how the original game is programmed (its code, memory layout, disassemblies or annotated
listings); if you recognise something from such a source, report it.

- Write the model in `crates/clean/{model}` and the types it shares in `crates/clean/types`.
  They depend on each other, std and the registry crates their manifests name. The manifests
  are not yours to edit: if a widely used crate would model the domain better, ask for it in
  your report. Cargo works offline here.
- Check with `cleanroom test`. It runs the code rules of DESIGN.md, the name scan on the files
  you changed, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace`, and passes only when all of them do. A name hit is a word the
  original's code uses: reword it in the player's terms.
- Commit only the two crates' Rust files under `src/` and `tests/`, each message ending with
  the trailer `Role: implementer`. Anything else is refused on import.
- Report a gap you can't explain, with the vector, the field, the expected and actual values and
  the spec entry you suspect.
"
    )
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::process::{git, output};

    #[test]
    fn the_export_s_git_never_packs_on_its_own() {
        let dir = std::env::temp_dir().join(format!("cleanroom-export-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        super::init(&dir).unwrap();
        let get = |key: &str| output(git(&dir).args(["config", "--get", key]));
        let (gc, maintenance) = (get("gc.auto"), get("maintenance.auto"));
        let folders = ["info", "hooks"].map(|f| dir.join(".git").join(f).is_dir());
        fs::remove_dir_all(&dir).unwrap();
        assert_eq!(gc.unwrap(), "0\n");
        assert_eq!(maintenance.unwrap(), "false\n");
        assert_eq!(folders, [true, true]);
    }
}
