use std::fmt::Write as _;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::process::{Lane, output, repo};
use crate::{rules, scan, vocabulary};

pub const IMAGE: &str = "sm-cleanroom-root:1";
const TOOLS: &str = "/opt/cleanroom";
const RUSTUP: &str = "/opt/rustup";
const PORT: &str = "8765";
const SESSION_SECONDS: &str = "21600";
const PROXIES: &[&str] = &[
    "cargo",
    "cargo-clippy",
    "cargo-fmt",
    "clippy-driver",
    "rustc",
    "rustdoc",
    "rustfmt",
    "rustup",
];
const TRUSTED: &[&str] = &[
    "Cargo.toml",
    "tools/cleanroom/allowed_deps.txt",
    "tools/cleanroom/scan_allow.txt",
    "tools/cleanroom/scan_lexicon",
];

pub struct Sandbox {
    args: Vec<String>,
}

fn which(name: &str) -> Result<PathBuf, String> {
    let path = output(Command::new("sh").args(["-c", &format!("command -v {name}")]))?;
    let path = PathBuf::from(path.trim());
    path.canonicalize()
        .map_err(|e| format!("{name} at {}: {e}", path.display()))
}

impl Sandbox {
    pub fn new(export: &Path, repo: &Path, lane: &Path) -> Result<Self, String> {
        let uid = output(Command::new("id").arg("-u"))?;
        let gid = output(Command::new("id").arg("-g"))?;
        let (uid, gid) = (uid.trim(), gid.trim());
        fs::write(
            lane.join("passwd"),
            format!("implementer:x:{uid}:{gid}::/tmp:/bin/sh\n"),
        )
        .map_err(|e| e.to_string())?;
        fs::write(lane.join("group"), format!("implementer:x:{gid}:\n"))
            .map_err(|e| e.to_string())?;
        let mut args: Vec<String> = [
            "run",
            "--rm",
            "-i",
            "--pull=never",
            "--network",
            "none",
            "--read-only",
            "--tmpfs",
            "/tmp:rw,nosuid,nodev,mode=1777,size=2g",
            "--memory",
            "12g",
            "--pids-limit",
            "4096",
            "--cap-drop",
            "ALL",
            "--security-opt",
            "no-new-privileges",
        ]
        .map(str::to_owned)
        .to_vec();
        args.extend([
            "--user".to_owned(),
            format!("{uid}:{gid}"),
            "--workdir".to_owned(),
            export.display().to_string(),
        ]);
        let mut cell = Self { args };
        cell.mount(export, export, true)?;
        for system in ["/usr", "/etc/alternatives", "/etc/ld.so.cache"] {
            cell.mount(Path::new(system), Path::new(system), false)?;
        }
        cell.mount(&lane.join("passwd"), Path::new("/etc/passwd"), false)?;
        cell.mount(&lane.join("group"), Path::new("/etc/group"), false)?;
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        cell.mount(&exe, &Path::new(TOOLS).join("cleanroom"), false)?;
        for path in TRUSTED {
            cell.mount(&repo.join(path), &Path::new(TOOLS).join(path), false)?;
        }
        let rustup = std::env::var_os("RUSTUP_HOME").map_or_else(
            || {
                std::env::var_os("HOME")
                    .map(|h| PathBuf::from(h).join(".rustup"))
                    .unwrap_or_default()
            },
            PathBuf::from,
        );
        cell.mount(&rustup, Path::new(RUSTUP), false)?;
        let cargo = which("cargo")?;
        if !cargo.starts_with("/usr") {
            for proxy in PROXIES {
                cell.mount(&cargo, &Path::new(TOOLS).join("bin").join(proxy), false)?;
            }
        }
        for (key, value) in [
            (
                "PATH",
                format!("{TOOLS}/bin:{TOOLS}:/usr/local/bin:/usr/bin:/bin"),
            ),
            ("HOME", "/tmp".to_owned()),
            ("TMPDIR", "/tmp".to_owned()),
            ("CARGO_HOME", "/tmp/.cargo".to_owned()),
            ("RUSTUP_HOME", RUSTUP.to_owned()),
            ("SHELL", "/bin/bash".to_owned()),
            ("LANG", "C.UTF-8".to_owned()),
            ("GIT_CONFIG_NOSYSTEM", "1".to_owned()),
            ("GIT_AUTHOR_NAME", "implementer".to_owned()),
            (
                "GIT_AUTHOR_EMAIL",
                "implementer@cleanroom.invalid".to_owned(),
            ),
            ("GIT_COMMITTER_NAME", "implementer".to_owned()),
            (
                "GIT_COMMITTER_EMAIL",
                "implementer@cleanroom.invalid".to_owned(),
            ),
        ] {
            cell.env(key, &value);
        }
        Ok(cell)
    }

    pub fn mount(&mut self, from: &Path, to: &Path, writable: bool) -> Result<(), String> {
        let from = from
            .canonicalize()
            .map_err(|e| format!("nothing to mount at {}: {e}", from.display()))?;
        let (from, to) = (from.display().to_string(), to.display().to_string());
        if from.contains([',', '\n']) || to.contains([',', '\n']) {
            return Err(format!("can't mount {from}: it holds a comma or a newline"));
        }
        let mode = if writable { "" } else { ",readonly" };
        self.args.push("--mount".to_owned());
        self.args
            .push(format!("type=bind,src={from},dst={to}{mode}"));
        Ok(())
    }

    pub fn env(&mut self, key: &str, value: &str) {
        self.args.push("--env".to_owned());
        self.args.push(format!("{key}={value}"));
    }

    pub fn command(&self, name: &str, argv: &[&str]) -> Command {
        let mut cmd = Command::new("docker");
        cmd.args(&self.args)
            .args(["--name", name, IMAGE])
            .args(argv);
        cmd
    }
}

pub fn image(repo: &Path) -> Result<(), String> {
    output(
        Command::new("python3")
            .arg("-I")
            .arg(repo.join("tools/cleanroom/box_relay.py"))
            .args(["image", IMAGE]),
    )
    .map(drop)
}

struct Relay(Child);

impl Drop for Relay {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn relay(repo: &Path, lane: &Path) -> Result<(Relay, Vec<String>), String> {
    let script = repo.join("tools/cleanroom/box_relay.py");
    let auth = std::env::var_os("CLEANROOM_AUTH").map_or_else(
        || {
            std::env::var_os("HOME")
                .map(|h| PathBuf::from(h).join(".claude/settings.json"))
                .unwrap_or_default()
        },
        PathBuf::from,
    );
    let env = output(
        Command::new("python3")
            .arg("-I")
            .arg(&script)
            .arg("env")
            .arg(&auth)
            .arg(PORT),
    )?;
    let socket = lane.join("api.sock");
    let _ = fs::remove_file(&socket);
    let log = File::create(lane.join("relay.log")).map_err(|e| e.to_string())?;
    let child = Command::new("python3")
        .arg("-I")
        .arg(&script)
        .arg("serve")
        .arg(&socket)
        .arg(&auth)
        .arg(std::process::id().to_string())
        .stdin(Stdio::null())
        .stderr(log)
        .spawn()
        .map_err(|e| e.to_string())?;
    let relay = Relay(child);
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(10))
        .unwrap_or_else(Instant::now);
    while !socket.exists() {
        if Instant::now() > deadline {
            return Err(format!(
                "the relay didn't start; see {}",
                lane.join("relay.log").display()
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Ok((relay, env.lines().map(str::to_owned).collect()))
}

pub fn run(name: &str, brief: Option<&Path>) -> Result<(String, bool), String> {
    let lane = Lane::of(name)?;
    let repo = repo()?;
    if !lane.export.join(".git").is_dir() {
        return Err(format!("no export at {}", lane.export.display()));
    }
    if let Some(brief) = brief {
        let text = fs::read_to_string(brief).map_err(|e| format!("{}: {e}", brief.display()))?;
        let hits = crate::scan_text(&repo, &brief.display().to_string(), &text)?;
        if !hits.is_empty() {
            return Err(format!(
                "the brief names what the clean room hides:\n{hits}"
            ));
        }
        fs::write(lane.brief(), text).map_err(|e| e.to_string())?;
    }
    image(&repo)?;
    let (relay, env) = relay(&repo, &lane.dir)?;
    let mut cell = Sandbox::new(&lane.export, &repo, &lane.dir)?;
    for line in env {
        if let Some((key, value)) = line.split_once('=') {
            cell.env(key, value);
        }
    }
    cell.env("DISABLE_AUTOUPDATER", "1");
    let cli =
        std::env::var_os("CLAUDE_BIN").map_or_else(|| which("claude"), |p| Ok(PathBuf::from(p)))?;
    cell.mount(&cli, &Path::new(TOOLS).join("claude"), false)?;
    cell.mount(
        &repo.join("tools/cleanroom/box_relay.py"),
        &Path::new(TOOLS).join("box_relay.py"),
        false,
    )?;
    let socket = format!("{TOOLS}/api.sock");
    cell.mount(&lane.dir.join("api.sock"), Path::new(&socket), true)?;
    let model = std::env::var("CLEANROOM_MODEL").unwrap_or_else(|_| "opus".to_owned());
    let transcript = lane.dir.join("transcript.jsonl");
    let container = format!("cleanroom-{name}-{}", std::process::id());
    let status = cell
        .command(
            &container,
            &[
                "/usr/bin/python3",
                "-I",
                &format!("{TOOLS}/box_relay.py"),
                "box",
                PORT,
                &socket,
                SESSION_SECONDS,
                "--",
                &format!("{TOOLS}/claude"),
                "-p",
                "--verbose",
                "--output-format",
                "stream-json",
                "--model",
                &model,
                "--tools",
                "Read,Edit,Write,Bash,Glob,Grep",
                "--permission-mode",
                "bypassPermissions",
            ],
        )
        .stdin(File::open(lane.brief()).map_err(|e| e.to_string())?)
        .stdout(File::create(&transcript).map_err(|e| e.to_string())?)
        .stderr(File::create(lane.dir.join("stderr.txt")).map_err(|e| e.to_string())?)
        .status()
        .map_err(|e| e.to_string())?;
    drop(relay);
    let mut out = String::new();
    let _ = writeln!(out, "transcript {}", transcript.display());
    if status.success() {
        let _ = writeln!(out, "RUN OK {name}");
        Ok((out, false))
    } else {
        let _ = writeln!(
            out,
            "RUN FAILED {name}: the box exited {status}; see {}",
            lane.dir.join("stderr.txt").display()
        );
        Ok((out, true))
    }
}

pub fn test() -> Result<(String, bool), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let trusted = exe.parent().ok_or("the tool has no folder")?;
    let here = PathBuf::from(".");
    let mut out = String::new();
    let mut failed = Vec::new();
    let found = rules::check(&here, &rules::Policy::load(trusted)?)?;
    for finding in &found {
        let _ = writeln!(out, "{finding}");
    }
    if !found.is_empty() {
        failed.push(format!("code rules ({})", found.len()));
    }
    let added = new_names(trusted, &mut out)?;
    if added > 0 {
        failed.push(format!("names ({added})"));
    }
    print_now(&std::mem::take(&mut out));
    for (label, args) in [
        ("fmt", &["fmt", "--check"][..]),
        (
            "clippy",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ][..],
        ),
        ("tests", &["test", "--workspace"][..]),
    ] {
        let ok = Command::new("cargo")
            .args(args)
            .status()
            .is_ok_and(|s| s.success());
        if !ok {
            failed.push(label.to_owned());
        }
    }
    if failed.is_empty() {
        out.push_str("TEST OK\n");
    } else {
        let _ = writeln!(out, "TEST FAILED: {}", failed.join(", "));
    }
    Ok((out, !failed.is_empty()))
}

fn print_now(text: &str) {
    use std::io::Write as _;
    let _ = std::io::stdout().write_all(text.as_bytes());
}

fn new_names(trusted: &Path, out: &mut String) -> Result<usize, String> {
    let (lex, allow) = vocabulary(trusted)?;
    let git = |args: &[&str]| output(Command::new("git").args(args));
    let root = git(&["rev-list", "--max-parents=0", "HEAD"])?;
    let root = root.lines().next().unwrap_or_default().to_owned();
    let mut changed: Vec<String> = git(&[
        "diff",
        "--name-only",
        "--no-renames",
        "--diff-filter=d",
        &root,
        "--",
        "crates/clean",
    ])?
    .lines()
    .map(str::to_owned)
    .collect();
    changed.extend(
        git(&[
            "ls-files",
            "--others",
            "--exclude-standard",
            "--",
            "crates/clean",
        ])?
        .lines()
        .map(str::to_owned),
    );
    let mut added = 0usize;
    for path in &changed {
        let head = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let start = git(&["show", &format!("{root}:{path}")]).ok();
        for f in scan::added(&lex, &allow, path, start.as_deref(), &head) {
            let _ = writeln!(out, "{}:{}: {}: {}", f.path, f.line, f.class, f.detail);
            added = added.saturating_add(1);
        }
    }
    Ok(added)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::{Sandbox, image};

    #[test]
    fn the_box_reads_its_export_and_nothing_outside_it() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        image(repo).unwrap();
        let base = std::env::temp_dir().join(format!("cleanroom-box-{}", std::process::id()));
        let (export, lane) = (base.join("export"), base.join("lane"));
        fs::create_dir_all(&export).unwrap();
        fs::create_dir_all(&lane).unwrap();
        fs::write(export.join("inside.txt"), "inside").unwrap();
        fs::write(base.join("outside.txt"), "outside").unwrap();
        let sandbox = Sandbox::new(&export, repo, &lane).unwrap();
        let read = |path: &Path| {
            let name = format!("cleanroom-probe-{}", std::process::id());
            let out = sandbox
                .command(&name, &["/usr/bin/cat", &path.display().to_string()])
                .output()
                .unwrap();
            out.status
                .success()
                .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
        };
        let inside = read(&export.join("inside.txt"));
        let outside: Vec<_> = [
            base.join("outside.txt"),
            repo.join("README.md"),
            repo.join("docs/behaviours/samus-movement.decode.md"),
            Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
        ]
        .iter()
        .filter_map(|p| read(p).map(|_| p.display().to_string()))
        .collect();
        fs::remove_dir_all(&base).unwrap();
        assert_eq!(inside.as_deref(), Some("inside"));
        assert_eq!(outside, Vec::<String>::new());
    }
}
