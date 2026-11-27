use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn output(cmd: &mut Command) -> Result<String, String> {
    let out = cmd
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("{cmd:?}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{cmd:?} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    String::from_utf8(out.stdout).map_err(|e| format!("{cmd:?}: {e}"))
}

pub fn git(dir: &Path) -> Command {
    let mut cmd = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(key);
        }
    }
    cmd.env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .args([
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "protocol.allow=never",
            "-c",
            "user.name=cleanroom",
            "-c",
            "user.email=cleanroom@invalid",
            "-c",
            "commit.gpgsign=false",
            "-C",
        ])
        .arg(dir);
    cmd
}

pub fn repo() -> Result<PathBuf, String> {
    let top = output(Command::new("git").args(["rev-parse", "--show-toplevel"]))?;
    Ok(PathBuf::from(top.trim()))
}

pub fn exports_root() -> Result<PathBuf, String> {
    let root = std::env::var_os("CLEANROOM_ROOT")
        .map_or_else(|| PathBuf::from("/cleanroom"), PathBuf::from);
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .and_then(|h| h.canonicalize().ok());
    let resolved = root
        .canonicalize()
        .map_err(|e| format!("the exports' root {}: {e}", root.display()))?;
    if home.is_some_and(|h| resolved.starts_with(h)) {
        return Err(format!(
            "{} is in the home, where host tools run git in the repositories they find; set CLEANROOM_ROOT outside it",
            resolved.display()
        ));
    }
    Ok(resolved)
}

pub fn check_name(name: &str) -> Result<(), String> {
    let plain = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-');
    if !plain {
        return Err(format!(
            "{name}: a name is lowercase letters, digits and inner dashes"
        ));
    }
    if name == "types" {
        return Err("types is the shared crate's folder, not a domain".to_owned());
    }
    Ok(())
}

pub struct Lane {
    pub export: PathBuf,
    pub dir: PathBuf,
}

impl Lane {
    pub fn of(name: &str) -> Result<Self, String> {
        check_name(name)?;
        let root = exports_root()?;
        Ok(Self {
            export: root.join(name),
            dir: root.join(format!("{name}.lane")),
        })
    }

    pub fn brief(&self) -> PathBuf {
        self.dir.join("brief.md")
    }
}
