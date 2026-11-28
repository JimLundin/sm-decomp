use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{Read as _, Write as _};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::import::{GIT_READ_ONLY, git_breaches};
use crate::lexicon::Lexicon;
use crate::process::{Lane, exports_root, output, repo};
use crate::scan::Allow;
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
const TRUSTED: &[&str] = &["Cargo.toml", "tools/cleanroom/allowed_deps.txt"];
/// The relay's socket in the box. The name scan is served there, from outside, so the
/// lexicon never enters the box.
const SOCKET: &str = "/opt/cleanroom/api.sock";

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
    /// The box on an export: the export writable at its own path, but for the parts of its
    /// `.git` a session may not change.
    pub fn new(export: &Path, repo: &Path, lane: &Path) -> Result<Self, String> {
        let mut cell = Self::bare(export, repo, lane)?;
        cell.mount(export, export, true)?;
        for name in GIT_READ_ONLY {
            let path = export.join(".git").join(name);
            let meta =
                fs::symlink_metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            if meta.file_type().is_symlink() {
                return Err(format!("{} is a symlink", path.display()));
            }
            cell.mount(&path, &path, false)?;
        }
        Ok(cell)
    }

    /// The box with no network, no capabilities and a read-only root, holding the system, the
    /// toolchain, this binary and the trusted files read-only, a fresh `/tmp` as `HOME` and
    /// `CARGO_HOME`, and nothing of the host's beyond what the caller mounts. It starts in
    /// WORKDIR; LANE holds the box's passwd and group files.
    pub fn bare(workdir: &Path, repo: &Path, lane: &Path) -> Result<Self, String> {
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
            workdir.display().to_string(),
        ]);
        let mut cell = Self { args };
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

fn relay(repo: &Path, lane: &Path, model: &str) -> Result<(Relay, Vec<String>), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
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
        .arg("--scan")
        .arg(&exe)
        .arg(repo)
        .arg(model)
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

fn cli() -> Result<PathBuf, String> {
    std::env::var_os("CLAUDE_BIN").map_or_else(|| which("claude"), |p| Ok(PathBuf::from(p)))
}

/// The CLI's version as it prints it.
fn version(cli: &Path) -> Result<String, String> {
    Ok(output(Command::new(cli).arg("--version"))?
        .trim()
        .to_owned())
}

/// Where the last passing probe records the CLI's version: beside the lane folders, since each
/// export makes a fresh one.
fn probed() -> Result<PathBuf, String> {
    Ok(exports_root()?.join("cli-version"))
}

/// `cleanroom run --probe NAME`: the box on NAME's export with a probe that writes there and
/// tries to read outside, then the mounted CLI's `--version`. It passes when the probe is
/// confined and the CLI is the one the last passing probe ran, and records that version.
pub fn probe(name: &str) -> Result<(String, bool), String> {
    probe_at(&Lane::of(name)?, &repo()?, &cli()?, &probed()?)
}

fn probe_at(lane: &Lane, repo: &Path, cli: &Path, record: &Path) -> Result<(String, bool), String> {
    let breaches = git_breaches(lane)?;
    if !breaches.is_empty() {
        return Err(format!(
            "the export's .git holds what no session's git leaves:\n{}",
            breaches.join("\n")
        ));
    }
    image(repo)?;
    let mut cell = Sandbox::new(&lane.export, repo, &lane.dir)?;
    cell.mount(cli, &Path::new(TOOLS).join("claude"), false)?;
    let home = std::env::var("HOME").unwrap_or_default();
    let outside = [
        repo.join("AGENTS.md").display().to_string(),
        lane.dir.display().to_string(),
        record.display().to_string(),
        home,
        format!("{TOOLS}/tools/cleanroom/scan_lexicon"),
    ];
    let script = "ok=1; touch .probe && rm .probe || { echo 'cannot write the export'; ok=0; }; \
                  for p in \"$@\"; do if ls \"$p\" >/dev/null 2>&1; then echo \"reads $p\"; ok=0; \
                  else echo \"cannot read $p\"; fi; done; [ $ok = 1 ] && echo CONFINED";
    let mut argv = vec!["/bin/sh", "-c", script, "probe"];
    argv.extend(outside.iter().map(String::as_str));
    let export_name = lane
        .export
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let container = format!("cleanroom-probe-{export_name}-{}", std::process::id());
    let seen = cell
        .command(&container, &argv)
        .output()
        .map_err(|e| e.to_string())?;
    let confined = String::from_utf8_lossy(&seen.stdout).into_owned();
    let seen = cell
        .command(&container, &[&format!("{TOOLS}/claude"), "--version"])
        .output()
        .map_err(|e| e.to_string())?;
    let version = String::from_utf8_lossy(&seen.stdout).trim().to_owned();
    let mut out = format!("{confined}CLI {version}\n");
    if !confined.lines().any(|l| l == "CONFINED") || version.is_empty() {
        let _ = writeln!(
            out,
            "PROBE FAILED: the box is not confined or its CLI didn't run"
        );
        return Ok((out, true));
    }
    match fs::read_to_string(record) {
        Ok(last) if last.trim() != version => {
            let _ = writeln!(
                out,
                "PROBE FAILED: the CLI is {version}, the last passing probe ran {}; review what the new CLI changes in its flags, then delete {} and probe again",
                last.trim(),
                record.display()
            );
            return Ok((out, true));
        }
        _ => fs::write(record, &version).map_err(|e| e.to_string())?,
    }
    let _ = writeln!(out, "PROBE OK {version}");
    Ok((out, false))
}

/// Refuses a session whose CLI is not the one the last passing probe ran.
fn check_probed(cli: &Path, record: &Path) -> Result<(), String> {
    let now = version(cli)?;
    let last = fs::read_to_string(record).unwrap_or_default();
    if last.trim() == now {
        Ok(())
    } else {
        Err(format!(
            "the CLI is {now} and no passing probe ran it; run `cleanroom run --probe NAME` first"
        ))
    }
}

pub fn run(name: &str, brief: Option<&Path>) -> Result<(String, bool), String> {
    let lane = Lane::of(name)?;
    let repo = repo()?;
    if !lane.export.join(".git").is_dir() {
        return Err(format!("no export at {}", lane.export.display()));
    }
    let cli = cli()?;
    check_probed(&cli, &probed()?)?;
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
    let breaches = git_breaches(&lane)?;
    if !breaches.is_empty() {
        return Err(format!(
            "the export's .git holds what no session's git leaves:\n{}",
            breaches.join("\n")
        ));
    }
    image(&repo)?;
    let model = format!(
        "crates/clean/{}",
        name.trim_end_matches("-review").replace('-', "_")
    );
    let (relay, env) = relay(&repo, &lane.dir, &model)?;
    let mut cell = Sandbox::new(&lane.export, &repo, &lane.dir)?;
    for line in env {
        if let Some((key, value)) = line.split_once('=') {
            cell.env(key, value);
        }
    }
    cell.env("DISABLE_AUTOUPDATER", "1");
    cell.mount(&cli, &Path::new(TOOLS).join("claude"), false)?;
    cell.mount(
        &repo.join("tools/cleanroom/box_relay.py"),
        &Path::new(TOOLS).join("box_relay.py"),
        false,
    )?;
    cell.mount(&lane.dir.join("api.sock"), Path::new(SOCKET), true)?;
    let model = std::env::var("CLEANROOM_MODEL").unwrap_or_else(|_| "opus".to_owned());
    let mut argv = vec![
        "/usr/bin/python3".to_owned(),
        "-I".to_owned(),
        format!("{TOOLS}/box_relay.py"),
        "box".to_owned(),
        PORT.to_owned(),
        SOCKET.to_owned(),
        SESSION_SECONDS.to_owned(),
        "--".to_owned(),
    ];
    argv.extend(session_argv(&model));
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    let transcript = lane.dir.join("transcript.jsonl");
    let container = format!("cleanroom-{name}-{}", std::process::id());
    let status = cell
        .command(&container, &argv)
        .stdin(File::open(lane.brief()).map_err(|e| e.to_string())?)
        .stdout(File::create(&transcript).map_err(|e| e.to_string())?)
        .stderr(File::create(lane.dir.join("stderr.txt")).map_err(|e| e.to_string())?)
        .status()
        .map_err(|e| e.to_string())?;
    drop(relay);
    let mut out = String::new();
    let _ = writeln!(out, "transcript {}", transcript.display());
    let breaches = git_breaches(&lane)?;
    if !breaches.is_empty() {
        let _ = writeln!(
            out,
            "RUN BREACH {name}: the export's .git holds what no session's git leaves, and the import will refuse it:\n{}",
            breaches.join("\n")
        );
        return Ok((out, true));
    }
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

/// The CLI's command line. Besides the box, the CLI loads no settings file, MCP server, skill,
/// command, hook, plugin sync or CLAUDE.md of its own accord.
fn session_argv(model: &str) -> Vec<String> {
    [
        &format!("{TOOLS}/claude"),
        "-p",
        "--verbose",
        "--output-format",
        "stream-json",
        "--model",
        model,
        "--bare",
        "--setting-sources",
        "",
        "--strict-mcp-config",
        "--mcp-config",
        r#"{"mcpServers":{}}"#,
        "--disable-slash-commands",
        "--tools",
        "Read,Edit,Write,Bash,Glob,Grep",
        "--permission-mode",
        "bypassPermissions",
    ]
    .map(str::to_owned)
    .to_vec()
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
    let added = new_names(Path::new(SOCKET), &mut out)?;
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

#[derive(Serialize, Deserialize)]
struct ScanFile {
    path: String,
    start: Option<String>,
    head: String,
}

#[derive(Serialize, Deserialize)]
struct ScanRequest {
    file: Vec<ScanFile>,
}

/// The host side of the box's name scan: per file of a request, its new hits' count and lines,
/// a tab between each, and never the words that hit.
pub fn names(request: &str, trusted: &Path) -> Result<(String, bool), String> {
    let (lex, allow) = vocabulary(trusted)?;
    names_in(&lex, &allow, request)
}

fn names_in(lex: &Lexicon, allow: &Allow, request: &str) -> Result<(String, bool), String> {
    let request: ScanRequest = toml::from_str(request).map_err(|e| e.to_string())?;
    let mut out = String::new();
    let mut total = 0usize;
    for file in &request.file {
        let found = scan::added(lex, allow, &file.path, file.start.as_deref(), &file.head);
        let mut lines: Vec<usize> = found.iter().map(|f| f.line).collect();
        lines.sort_unstable();
        lines.dedup();
        let lines: Vec<String> = lines.iter().map(usize::to_string).collect();
        let _ = writeln!(out, "{}\t{}\t{}", file.path, found.len(), lines.join(","));
        total = total.saturating_add(found.len());
    }
    Ok((out, total > 0))
}

/// Sends the files to the relay's scan and returns its answer.
fn scan_remote(socket: &Path, files: Vec<ScanFile>) -> Result<String, String> {
    let body = toml::to_string(&ScanRequest { file: files }).map_err(|e| e.to_string())?;
    let mut stream = UnixStream::connect(socket)
        .map_err(|e| format!("the name scan's socket {}: {e}", socket.display()))?;
    write!(
        stream,
        "POST /cleanroom/scan HTTP/1.1\r\nHost: relay\r\nContent-Type: application/toml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .map_err(|e| e.to_string())?;
    let mut reply = String::new();
    stream
        .read_to_string(&mut reply)
        .map_err(|e| e.to_string())?;
    let (head, text) = reply.split_once("\r\n\r\n").unwrap_or((&reply, ""));
    if !head.starts_with("HTTP/1.1 200") && !head.starts_with("HTTP/1.0 200") {
        return Err(format!(
            "the name scan refused: {}",
            head.lines().next().unwrap_or_default()
        ));
    }
    Ok(text.to_owned())
}

fn new_names(socket: &Path, out: &mut String) -> Result<usize, String> {
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
    if changed.is_empty() {
        return Ok(0);
    }
    let mut files = Vec::new();
    for path in changed {
        let head = fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?;
        let start = git(&["show", &format!("{root}:{path}")]).ok();
        files.push(ScanFile { path, start, head });
    }
    let mut added = 0usize;
    for line in scan_remote(socket, files)?.lines() {
        let mut fields = line.split('\t');
        let (Some(path), Some(hits), Some(lines)) = (fields.next(), fields.next(), fields.next())
        else {
            return Err(format!("the name scan answered {line}"));
        };
        let hits: usize = hits
            .parse()
            .map_err(|_| format!("the name scan answered {line}"))?;
        if hits > 0 {
            let at = if lines.is_empty() {
                String::new()
            } else {
                format!(", on lines {lines}")
            };
            let _ = writeln!(out, "{path}: {hits} words the original's code uses{at}");
            added = added.saturating_add(hits);
        }
    }
    Ok(added)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::{Read as _, Write as _};
    use std::os::unix::net::UnixListener;
    use std::path::Path;

    use super::{
        Sandbox, ScanFile, check_probed, image, names_in, probe_at, scan_remote, session_argv,
    };
    use crate::process::Lane;

    fn repo() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
    }

    /// An export with the `.git` folders the box mounts read-only, its lane folder, and a CLI
    /// that prints VERSION, all under BASE.
    fn probe_lane(base: &Path, version: &str) -> (Lane, std::path::PathBuf) {
        let lane = Lane {
            export: base.join("probe"),
            dir: base.join("probe.lane"),
        };
        for dir in [&lane.export, &lane.dir] {
            fs::create_dir_all(dir).unwrap();
        }
        crate::export::init(&lane.export).unwrap();
        lane.record_git().unwrap();
        let cli = base.join("claude");
        fs::write(&cli, format!("#!/bin/sh\necho '{version}'\n")).unwrap();
        let mut perms = fs::metadata(&cli).unwrap().permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o755);
        fs::set_permissions(&cli, perms).unwrap();
        (lane, cli)
    }

    #[test]
    fn the_probe_records_the_cli_and_refuses_a_changed_one() {
        let base = std::env::temp_dir().join(format!("cleanroom-probe-{}", std::process::id()));
        let (lane, cli) = probe_lane(&base, "9.9.9 (Claude Code)");
        let record = base.join("cli-version");
        let first = probe_at(&lane, repo(), &cli, &record).unwrap();
        let recorded = fs::read_to_string(&record).unwrap();
        let session = check_probed(&cli, &record);
        fs::write(&record, "1.0.0 (Claude Code)").unwrap();
        let changed = probe_at(&lane, repo(), &cli, &record).unwrap();
        let kept = fs::read_to_string(&record).unwrap();
        let refused = check_probed(&cli, &record);
        fs::remove_dir_all(&base).unwrap();
        assert!(!first.1, "{}", first.0);
        assert!(
            first
                .0
                .contains("CONFINED\nCLI 9.9.9 (Claude Code)\nPROBE OK"),
            "{}",
            first.0
        );
        assert_eq!(recorded, "9.9.9 (Claude Code)");
        assert!(session.is_ok());
        assert!(changed.1);
        assert!(
            changed.0.contains("the last passing probe ran 1.0.0"),
            "{}",
            changed.0
        );
        assert_eq!(kept, "1.0.0 (Claude Code)");
        assert!(refused.is_err());
    }
    use crate::lexicon::Lexicon;
    use crate::scan::Allow;

    #[test]
    fn the_cli_loads_no_settings_mcp_servers_or_commands_of_its_own() {
        let argv = session_argv("opus");
        let after = |flag: &str| {
            let at = argv.iter().position(|a| a == flag);
            at.and_then(|i| argv.get(i.saturating_add(1))).cloned()
        };
        for flag in ["--bare", "--strict-mcp-config", "--disable-slash-commands"] {
            assert!(argv.iter().any(|a| a == flag), "{flag}");
        }
        assert_eq!(after("--setting-sources").as_deref(), Some(""));
        assert_eq!(
            after("--mcp-config").as_deref(),
            Some(r#"{"mcpServers":{}}"#)
        );
    }

    #[test]
    fn the_name_scan_answers_counts_and_lines_and_never_the_words() {
        let mut lex = Lexicon::default();
        lex.names.insert(crate::text::hash(&lex.salt, "u:quux"));
        let request = "[[file]]\npath = \"a.rs\"\nhead = \"fn f() {}\\nlet quux = 1;\\n\"\n\n\
                       [[file]]\npath = \"b.rs\"\nstart = \"quux\"\nhead = \"quux\"\n";
        let (out, failed) = names_in(&lex, &Allow::parse(""), request).unwrap();
        assert_eq!(out, "a.rs\t1\t2\nb.rs\t0\t\n");
        assert!(failed);
    }

    #[test]
    fn the_box_asks_the_relay_for_the_name_scan() {
        let dir = std::env::temp_dir().join(format!("cleanroom-scan-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let socket = dir.join("api.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let relay = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = vec![0; 4096];
            let n = stream.read(&mut request).unwrap();
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\na.rs\t2\t3,7\n")
                .unwrap();
            String::from_utf8_lossy(&request[..n]).into_owned()
        });
        let answer = scan_remote(
            &socket,
            vec![ScanFile {
                path: "a.rs".to_owned(),
                start: None,
                head: "x".to_owned(),
            }],
        );
        let request = relay.join().unwrap();
        fs::remove_dir_all(&dir).unwrap();
        assert_eq!(answer.unwrap(), "a.rs\t2\t3,7\n");
        assert!(
            request.starts_with("POST /cleanroom/scan HTTP/1.1\r\n"),
            "{request}"
        );
        assert!(
            request.ends_with("[[file]]\npath = \"a.rs\"\nhead = \"x\"\n"),
            "{request}"
        );
    }

    #[test]
    fn the_box_reads_its_export_and_nothing_outside_it() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        image(repo).unwrap();
        let base = std::env::temp_dir().join(format!("cleanroom-box-{}", std::process::id()));
        let (export, lane) = (base.join("export"), base.join("lane"));
        for dir in [&export.join(".git/info"), &export.join(".git/hooks"), &lane] {
            fs::create_dir_all(dir).unwrap();
        }
        fs::write(export.join(".git/config"), "").unwrap();
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
        let write = |path: &Path| {
            let name = format!("cleanroom-probe-{}", std::process::id());
            let script = format!("echo x >> '{}'", path.display());
            sandbox
                .command(&name, &["/bin/sh", "-c", &script])
                .output()
                .unwrap()
                .status
                .success()
        };
        let written = write(&export.join("written.txt"));
        let git_written: Vec<_> = ["config", "info/attributes", "hooks/pre-commit"]
            .iter()
            .filter(|p| write(&export.join(".git").join(p)))
            .collect();
        let inside = read(&export.join("inside.txt"));
        let outside: Vec<_> = [
            base.join("outside.txt"),
            repo.join("README.md"),
            repo.join("docs/behaviours/samus-movement.decode.md"),
            Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
            Path::new("/opt/cleanroom/tools/cleanroom/scan_lexicon/salt").to_owned(),
            Path::new("/opt/cleanroom/tools/cleanroom/scan_lexicon/names").to_owned(),
            repo.join("tools/cleanroom/scan_lexicon/names"),
        ]
        .iter()
        .filter_map(|p| read(p).map(|_| p.display().to_string()))
        .collect();
        fs::remove_dir_all(&base).unwrap();
        assert_eq!(inside.as_deref(), Some("inside"));
        assert!(written);
        assert_eq!(git_written, Vec::<&&str>::new());
        assert_eq!(outside, Vec::<String>::new());
    }
}
