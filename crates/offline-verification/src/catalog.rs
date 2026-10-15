//! Read-only catalog declarations and bounded Fixture discovery.
use crate::inputs::bounded;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CatalogEntry {
    Kind(String),
    Details(FamilyEntry),
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FamilyEntry {
    pub kind: String,
    pub manifest: PathBuf,
    pub live: Option<String>,
    pub prepare: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Catalog {
    pub families: BTreeMap<String, CatalogEntry>,
    pub profiles: BTreeMap<String, Vec<String>>,
}
pub fn catalog(checkout: &Path) -> Result<Catalog> {
    bounded::limited(|| catalog_using(checkout, |path| bounded::read(path)))
}

pub fn catalog_until(checkout: &Path, deadline: u64, max_bytes: u64) -> Result<Catalog> {
    bounded::until(deadline, max_bytes, || catalog_using(checkout, |path| bounded::read(path)))
}

/// Keep catalog discovery identical for the supervisor's bounded source reader.
pub fn catalog_using(checkout: &Path, mut read: impl FnMut(&Path) -> Result<Vec<u8>>) -> Result<Catalog> {
    let mut config: Catalog = toml::from_str(std::str::from_utf8(&read(&checkout.join("tools/port/batch.toml"))?)?)?;
    let paths = bounded::entries(&checkout.join("tools/port"))?;
    ensure!(
        paths.len() <= 4096 && config.families.len() <= 4096 && config.profiles.len() <= 1024,
        "native catalog discovery inventory exceeded"
    );
    for path in paths {
        bounded::check()?;
        if path.extension().is_none_or(|v| v != crate::fixture_format::EXTENSION) {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy();
        if config.families.contains_key(name.as_ref()) {
            continue;
        }
        let manifest = crate::fixture_format::decode(read(&path)?)
            .with_context(|| format!("invalid discovery manifest {}", path.display()))?;
        let specialized = manifest.projectile_host
            || manifest.projectile_rendering
            || manifest.routines.iter().any(|r| {
                r.pointer_bank.is_some()
                    || ["EprojInstruction", "EprojPreinstruction", "PlmInstruction", "PlmFrame", "PlmControlFrame"]
                        .contains(&r.symbol.as_str())
            });
        config.families.insert(
            name.into_owned(),
            CatalogEntry::Details(FamilyEntry {
                kind: if specialized { "dispatcher" } else { "family" }.into(),
                manifest: path.strip_prefix(checkout)?.into(),
                live: None,
                prepare: None,
            }),
        );
        ensure!(config.families.len() <= 4096, "native catalog family inventory exceeded");
    }
    config.profiles.insert("offline-all".into(), config.families.keys().cloned().collect());
    Ok(config)
}
pub fn entry(config: &Catalog, name: &str) -> Result<FamilyEntry> {
    Ok(match config.families.get(name).with_context(|| format!("unknown family: {name}"))? {
        CatalogEntry::Kind(kind) => FamilyEntry {
            kind: kind.clone(),
            manifest: format!("tools/port/{name}.smf").into(),
            live: ["family", "callback", "projectile"].contains(&kind.as_str()).then(|| kind.clone()),
            prepare: None,
        },
        CatalogEntry::Details(item) => item.clone(),
    })
}
