//! Private conveniences for application request and report documents.
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use toml::Value;

pub(crate) fn field<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key].as_str().with_context(|| format!("missing {key}"))
}

pub(crate) fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    value[key].as_array().with_context(|| format!("missing {key}"))
}

pub(crate) fn resolve(checkout: &Path, text: &str) -> PathBuf {
    let path = Path::new(text);
    if path.is_absolute() { path.to_path_buf() } else { checkout.join(path) }
}
