use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// `config/functions.toml`
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Functions {
    #[serde(default, rename = "function")]
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    /// 24-bit address as a hex string, e.g. "808338" or "$80:8338".
    pub addr: String,
    /// Name, ideally from snesrev/sm or the bank logs.
    #[serde(default)]
    pub name: Option<String>,
    /// Width variants to generate: "m0x0", "m0x1", "m1x0", "m1x1".
    pub variants: Vec<String>,
    /// Hand-written replacement lives in crates/game/src/overrides; don't generate.
    #[serde(default, rename = "override")]
    pub override_: bool,
}

pub fn parse_addr(s: &str) -> Result<u32> {
    let clean: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    let v = u32::from_str_radix(&clean, 16).with_context(|| format!("bad address {s:?}"))?;
    if v > 0xff_ffff {
        bail!("address out of range: {s}");
    }
    Ok(v)
}

pub fn parse_variant(s: &str) -> Result<(bool, bool)> {
    Ok(match s {
        "m0x0" => (false, false),
        "m0x1" => (false, true),
        "m1x0" => (true, false),
        "m1x1" => (true, true),
        _ => bail!("bad variant {s:?}, expected m0x0/m0x1/m1x0/m1x1"),
    })
}

/// `config/jump_tables.toml`: known targets of indirect jumps, keyed by the
/// address of the JMP/JSR instruction. Not needed for correctness (indirect
/// jumps dispatch at runtime) but listing targets lets `recomp` pre-generate them.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct JumpTables {
    #[serde(default)]
    pub tables: BTreeMap<String, Vec<String>>,
}

pub fn load<T: for<'de> Deserialize<'de> + Default>(path: &std::path::Path) -> Result<T> {
    if !path.exists() {
        return Ok(T::default());
    }
    let s = std::fs::read_to_string(path)?;
    toml::from_str(&s).with_context(|| format!("parsing {}", path.display()))
}
