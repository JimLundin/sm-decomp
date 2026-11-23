//! The touched-cells receipt codec: the cells her moves reached, in order, and the cells she
//! started inside. It performs no movement decisions.
use super::{compare::Observation, data_failure, native_search::Kind, result::Outcome};
use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};
use std::path::Path;
use types::Direction;
pub(super) const PIN: &str = "00fdffb102bbf05fa184010538d66207c4cbf5f43375ecde123886d0e2945443";
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Xy<T> {
    x: T,
    y: T,
}
#[derive(Serialize, Deserialize)]
enum Way {
    Down,
    Left,
    Right,
    Up,
    Look,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Reached {
    cell: Xy<u16>,
    way: Way,
}
/// The reviewed source projection. Its receipts of the original's work inside the frame
/// (motion, support and fit work, collision tests and the skip reason) have no counterpart in
/// the whole-frame model, so they are read past unchecked.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    format: u16,
    id: String,
    source_start: u64,
    source_completion: u64,
    video_frame: u64,
    movement_skip_present: bool,
    #[serde(default)]
    movement_skip: Option<IgnoredAny>,
    #[serde(rename = "work")]
    _work: IgnoredAny,
    #[serde(default, rename = "collision_tests")]
    _collision_tests: Option<IgnoredAny>,
    #[serde(default)]
    reached: Vec<Reached>,
    inside: Vec<Xy<u16>>,
}
struct Body {
    reached: Vec<Reached>,
    inside: Vec<Xy<u16>>,
}
const fn cell(value: types::Cell) -> Xy<u16> {
    Xy {
        x: value.x,
        y: value.y,
    }
}
const fn way(value: types::Way) -> Way {
    match value {
        types::Way::Look => Way::Look,
        types::Way::Move(direction) => match direction {
            Direction::Down => Way::Down,
            Direction::Left => Way::Left,
            Direction::Right => Way::Right,
            Direction::Up => Way::Up,
        },
    }
}
fn observation<T: Serialize>(field: &str, value: &T) -> Result<Observation, Outcome> {
    #[derive(Serialize)]
    struct Value<'a, T> {
        value: &'a T,
    }
    let bytes = toml::to_string(&Value { value })
        .map_err(|_| data_failure(None, "touched cells receipt encoding"))?
        .into_bytes();
    Ok(Observation {
        field: field.to_owned(),
        bytes,
    })
}
impl Body {
    fn observations(&self) -> Result<Vec<Observation>, Outcome> {
        Ok(vec![
            observation("movement.reached.complete", &self.reached)?,
            observation("movement.inside", &self.inside)?,
        ])
    }
}
pub(super) fn actual(value: &samus_movement::Outcome) -> Result<Vec<Observation>, Outcome> {
    Body {
        reached: value
            .reached
            .iter()
            .map(|touch| Reached {
                cell: cell(touch.cell),
                way: way(touch.way),
            })
            .collect(),
        inside: value.inside.iter().copied().map(cell).collect(),
    }
    .observations()
}
pub(super) fn initial() -> Result<Vec<Observation>, Outcome> {
    Body {
        reached: Vec::new(),
        inside: Vec::new(),
    }
    .observations()
}
pub(super) fn expected(root: &Path, kind: Kind) -> Result<Vec<Observation>, Outcome> {
    let pins = std::fs::read(root.join("packet-pins.txt"))
        .map_err(|_| data_failure(None, "touched cells expectation manifest"))?;
    if crate::lock::Sha256::try_from(PIN).ok() != Some(crate::lock::Sha256::of(&pins)) {
        return Err(data_failure(
            None,
            "touched cells expectation admission pin",
        ));
    }
    let pins = String::from_utf8(pins)
        .map_err(|_| data_failure(None, "touched cells manifest encoding"))?;
    let (name, completion) = match kind {
        Kind::Miss => ("baseline-expected.toml", 4283),
        Kind::Touch => ("intervention-expected.toml", 4052),
    };
    let bytes = super::native_search::pinned(root, &pins, name)?;
    let text = String::from_utf8(bytes)
        .map_err(|_| data_failure(None, "touched cells expectation encoding"))?;
    let receipt: Receipt = toml::from_str(&text)
        .map_err(|_| data_failure(None, "touched cells expectation schema"))?;
    if receipt.format != 1
        || receipt.id.is_empty()
        || receipt.source_start != 424
        || receipt.source_completion != completion
        || receipt.video_frame != 9233
        || receipt.movement_skip_present != receipt.movement_skip.is_some()
    {
        return Err(data_failure(None, "touched cells expectation coordinates"));
    }
    Body {
        reached: receipt.reached,
        inside: receipt.inside,
    }
    .observations()
}

/// Locate a typed receipt difference without exposing any native quantities.
pub(super) fn difference(expected: &[u8], actual: &[u8]) -> Option<String> {
    fn first(expected: &toml::Value, actual: &toml::Value, path: String) -> Option<String> {
        match (expected, actual) {
            (toml::Value::Array(expected), toml::Value::Array(actual)) => {
                if expected.len() != actual.len() {
                    return Some(format!("{path}.count"));
                }
                expected
                    .iter()
                    .zip(actual)
                    .enumerate()
                    .find_map(|(index, (expected, actual))| {
                        first(expected, actual, format!("{path}[{index}]"))
                    })
            }
            (toml::Value::Table(expected), toml::Value::Table(actual)) => {
                for (key, expected) in expected {
                    let Some(actual) = actual.get(key) else {
                        return Some(format!("{path}.{key}.missing"));
                    };
                    if let Some(path) = first(expected, actual, format!("{path}.{key}")) {
                        return Some(path);
                    }
                }
                actual
                    .keys()
                    .find(|key| !expected.contains_key(*key))
                    .map(|key| format!("{path}.{key}.extra"))
            }
            (expected, actual) => (expected != actual).then_some(path),
        }
    }
    let expected: toml::Value = toml::from_str(std::str::from_utf8(expected).ok()?).ok()?;
    let actual: toml::Value = toml::from_str(std::str::from_utf8(actual).ok()?).ok()?;
    first(&expected, &actual, String::new())
        .map(|path| path.strip_prefix(".value").unwrap_or(&path).to_owned())
}
