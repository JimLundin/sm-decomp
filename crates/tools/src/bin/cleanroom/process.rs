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

/// Git in a repository on the host, reading no attributes from outside it: not the system's
/// file, `core.attributesFile`, nor its default under `$XDG_CONFIG_HOME`.
pub fn host_git(dir: &Path) -> Command {
    let mut cmd = Command::new("git");
    no_outside_attributes(&mut cmd);
    cmd.arg("-C").arg(dir);
    cmd
}

fn no_outside_attributes(cmd: &mut Command) {
    cmd.env("GIT_ATTR_NOSYSTEM", "1")
        .args(["-c", "core.attributesFile=/dev/null"]);
}

/// Git on an export or its mirror: none of the caller's git environment, configuration or
/// attributes, and no hooks.
pub fn git(dir: &Path) -> Command {
    let mut cmd = Command::new("git");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            cmd.env_remove(key);
        }
    }
    no_outside_attributes(&mut cmd);
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
    let top = output(host_git(Path::new(".")).args(["rev-parse", "--show-toplevel"]))?;
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

    /// The export's one commit, as `cleanroom export` made it.
    pub fn export_root(&self) -> PathBuf {
        self.dir.join("export-root")
    }

    /// The export's `.git` folder as `cleanroom export` made it, by device and inode.
    pub fn git_inode(&self) -> PathBuf {
        self.dir.join("git-inode")
    }

    /// The bytes of the export's `.git/config` as `cleanroom export` wrote them.
    pub fn git_config(&self) -> PathBuf {
        self.dir.join("git-config")
    }

    /// Notes what a session may not change in the export's `.git`: the folder itself and its
    /// configuration. The notes live in the lane folder, outside the box.
    pub fn record_git(&self) -> Result<(), String> {
        let git = self.export.join(".git");
        let meta =
            std::fs::symlink_metadata(&git).map_err(|e| format!("{}: {e}", git.display()))?;
        std::fs::write(self.git_inode(), inode(&meta)).map_err(|e| e.to_string())?;
        std::fs::copy(git.join("config"), self.git_config()).map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub fn inode(meta: &std::fs::Metadata) -> String {
    use std::os::unix::fs::MetadataExt as _;
    format!("{} {}\n", meta.dev(), meta.ino())
}
