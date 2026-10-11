//! Local TOML helpers for heterogeneous diagnostic report tables.
use anyhow::{Context, Result};
use serde::Serialize;
use serde::de::DeserializeOwned;
use toml::Value;

pub fn toml_from_slice<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    Ok(toml::from_str(std::str::from_utf8(bytes)?)?)
}
pub fn toml_value_decode<T: DeserializeOwned>(value: Value) -> Result<T> {
    Ok(value.try_into()?)
}

pub fn value<T: Serialize + ?Sized>(value: &T) -> Result<Value> {
    Value::try_from(value).context("diagnostic field is not representable in TOML")
}

pub fn insert<T: Serialize + ?Sized>(table: &mut toml::Table, name: &str, value: &T) -> Result<()> {
    // Optional fields are represented by absence. Serialization distinguishes
    // absence from a malformed or otherwise unsupported value.
    match Value::try_from(value) {
        Ok(value) => {
            table.insert(name.into(), value);
        }
        Err(error) if error.to_string() == "unsupported None value" => {}
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

#[macro_export]
macro_rules! report {
    (vec![]) => { toml::Value::Array(Vec::new()) };
    (vec![$($value:expr),+ $(,)?]) => { toml::Value::Array(vec![$($crate::readiness_metadata::value(&$value).expect("native diagnostic array member must be representable in TOML")),+]) };
    ({$($name:literal : $value:expr),* $(,)?}) => {{
        #[allow(unused_mut)]
        let mut table = toml::Table::new();
        $($crate::readiness_metadata::insert(&mut table, $name, &$value).expect("native diagnostic field must be representable in TOML");)*
        toml::Value::Table(table)
    }};
    ($value:expr) => { $crate::readiness_metadata::value(&$value).expect("native diagnostic value must be representable in TOML") };
}
