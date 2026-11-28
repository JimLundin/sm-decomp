//! A clean model's vector file, read by its vector test when the test runs, so a probe of the
//! file needs no rebuild (MODEL §1 decision 16).

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

const OVERRIDE: &str = "SM_VECTOR_FILE";

/// The text of the file `SM_VECTOR_FILE` names, or else of `scenarios/vectors/<domain>.toml`.
///
/// # Errors
///
/// The path and why it can't be read, or that the domain holds a `/` or `..` and so could name
/// a file outside the vectors folder.
pub fn read(domain: &str) -> Result<String, String> {
    if domain.contains('/') || domain.contains("..") {
        return Err(format!("{domain}: a domain holds no / or .."));
    }
    let path = path(env::var_os(OVERRIDE), domain);
    fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))
}

fn path(chosen: Option<OsString>, domain: &str) -> PathBuf {
    chosen.map_or_else(
        || {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../scenarios/vectors")
                .join(format!("{domain}.toml"))
        },
        PathBuf::from,
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn a_domain_s_file_is_in_the_vectors_folder() {
        let path = super::path(None, "demo");
        assert!(path.ends_with("scenarios/vectors/demo.toml"));
        assert!(path.parent().unwrap().is_dir(), "{}", path.display());
    }

    #[test]
    fn the_variable_names_the_file_in_its_place() {
        let chosen = super::path(Some("/probe/vectors.toml".into()), "samus-health");
        assert_eq!(chosen, Path::new("/probe/vectors.toml"));
    }

    #[test]
    fn a_domain_cannot_leave_the_vectors_folder() {
        for domain in ["../vectors/demo", "a/b", ".."] {
            let error = super::read(domain).unwrap_err();
            assert!(error.contains("holds no"), "{error}");
        }
    }

    #[test]
    fn an_unread_file_names_its_path() {
        let error = super::read("no-such-domain").unwrap_err();
        assert!(
            error.contains("scenarios/vectors/no-such-domain.toml"),
            "{error}"
        );
    }
}
