//! Compare ordered game-loop visits within a shared frame interval.
use crate::{atomic_toml, digest};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct CaptureMetadata {
    pub completed_frames: u64,
    pub checkpoint_sha256: String,
    pub movie_sha256: Option<String>,
    pub rom_sha256: Option<String>,
    pub core: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub struct Request {
    pub baseline: std::path::PathBuf,
    pub candidate: std::path::PathBuf,
    pub start_frame: i64,
    pub end_frame: i64,
    #[serde(default)]
    pub frame_offset: i64,
    pub output: Option<std::path::PathBuf>,
}
#[derive(Serialize, Deserialize, Default)]
pub struct Report {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_events: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_event: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_event: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_frame: Option<i64>,
    pub differences: BTreeMap<String, [i64; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_events: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_events: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_frame: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_frame: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_frame_offset: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<std::path::PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate: Option<std::path::PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_pc: Option<String>,
}
use std::{collections::BTreeMap, fs, path::Path};
pub(crate) const FIELDS: [&str; 10] =
    ["game_state", "room", "x", "y", "health", "items", "time_frames", "time_seconds", "time_minutes", "time_hours"];
pub fn read(path: &Path) -> Result<Vec<BTreeMap<String, i64>>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    let expected = [vec!["frame", "event"], FIELDS.to_vec()].concat();
    ensure!(
        lines.next().context("empty checkpoints")?.trim_end_matches('\r').split(',').collect::<Vec<_>>() == expected,
        "unexpected checkpoint columns"
    );
    let mut rows: Vec<BTreeMap<String, i64>> = Vec::new();
    for line in lines {
        let columns = line.trim_end_matches('\r').split(',').collect::<Vec<_>>();
        ensure!(columns.len() == expected.len(), "unexpected checkpoint row extent");
        let row = expected
            .iter()
            .zip(columns)
            .enumerate()
            .map(|(index, (key, value))| {
                Ok(((*key).to_owned(), i64::from_str_radix(value, if index < 2 { 10 } else { 16 })?))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        ensure!(
            row["event"] == rows.len() as i64 + 1
                && rows.last().is_none_or(|previous| row["frame"] >= previous["frame"]),
            "missing/reordered checkpoint events"
        );
        rows.push(row);
    }
    ensure!(!rows.is_empty(), "empty checkpoints");
    Ok(rows)
}
fn coverage(path: &Path, rows: &[BTreeMap<String, i64>]) -> Result<i64> {
    let sidecar = path.with_extension("capture.toml");
    if !sidecar.exists() {
        return Ok(rows.last().context("empty checkpoints")?["frame"]);
    }
    let v: CaptureMetadata = toml::from_str(&fs::read_to_string(sidecar)?)?;
    ensure!(v.checkpoint_sha256 == digest(path)?, "capture checksum mismatch");
    Ok(i64::try_from(v.completed_frames)?)
}
pub fn compare(
    baseline: &[BTreeMap<String, i64>],
    candidate: &[BTreeMap<String, i64>],
    start: i64,
    end: i64,
    offset: i64,
) -> Result<Report> {
    ensure!(start >= 1 && end >= start, "require 1 <= start-frame <= end-frame");
    let expected = baseline.iter().filter(|row| (start..=end).contains(&row["frame"])).collect::<Vec<_>>();
    let actual = candidate.iter().filter(|row| (start..=end).contains(&(row["frame"] - offset))).collect::<Vec<_>>();
    if expected.is_empty() || actual.is_empty() {
        return Ok(Report {
            status: "FAIL".into(),
            reason: Some("selected interval has no checkpoints".into()),
            ..Default::default()
        });
    }
    for (index, (left, right)) in expected.iter().zip(&actual).enumerate() {
        let mut differences = BTreeMap::new();
        if left["frame"] != right["frame"] - offset {
            differences.insert("frame".into(), [left["frame"], right["frame"] - offset]);
        }
        for field in FIELDS {
            if left[field] != right[field] {
                differences.insert(field.into(), [left[field], right[field]]);
            }
        }
        if !differences.is_empty() {
            return Ok(Report {
                status: "FAIL".into(),
                matched_events: Some(index),
                baseline_event: Some(left["event"]),
                candidate_event: Some(right["event"]),
                baseline_frame: Some(left["frame"]),
                differences,
                ..Default::default()
            });
        }
    }
    if expected.len() != actual.len() {
        return Ok(Report {
            status: "FAIL".into(),
            reason: Some("checkpoint counts differ".into()),
            baseline_events: Some(expected.len()),
            candidate_events: Some(actual.len()),
            ..Default::default()
        });
    }
    Ok(Report {
        status: "PASS".into(),
        events: Some(expected.len()),
        start_frame: Some(start),
        end_frame: Some(end),
        candidate_frame_offset: Some(offset),
        ..Default::default()
    })
}
pub fn operate(request: Request) -> Result<Report> {
    let baseline = request.baseline.as_path();
    let candidate = request.candidate.as_path();
    let start = request.start_frame;
    let end = request.end_frame;
    let offset = request.frame_offset;
    let expected = read(baseline)?;
    let actual = read(candidate)?;
    ensure!(
        coverage(baseline, &expected)? >= end && coverage(candidate, &actual)? - offset >= end,
        "checkpoint captures do not cover end-frame"
    );
    let mut report = compare(&expected, &actual, start, end, offset)?;
    report.baseline = Some(baseline.to_path_buf());
    report.candidate = Some(candidate.to_path_buf());
    report.checkpoint_pc = Some("828948".into());
    if let Some(out) = request.output.as_deref() {
        fs::create_dir_all(out.parent().context("output parent missing")?)?;
        atomic_toml(out, &report)?;
    }
    Ok(report)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn row(event: i64, frame: i64) -> BTreeMap<String, i64> {
        let mut row = FIELDS.into_iter().map(|name| (name.to_owned(), 0)).collect::<BTreeMap<_, _>>();
        row.insert("event".into(), event);
        row.insert("frame".into(), frame);
        row
    }
    #[test]
    fn repeated_visits_keep_order_and_frame_offset_is_constant() {
        let baseline = vec![row(1, 1), row(2, 2), row(3, 2)];
        let candidate = vec![row(1, 8), row(2, 9), row(3, 9)];
        assert_eq!(compare(&baseline, &candidate, 1, 2, 7).unwrap().status, "PASS");
        assert_eq!(compare(&baseline, &candidate[..2], 1, 2, 7).unwrap().status, "FAIL");
        let mut changed = candidate;
        changed[2].insert("health".into(), 1);
        assert_eq!(compare(&baseline, &changed, 1, 2, 7).unwrap().matched_events, Some(2));
        assert_eq!(compare(&baseline, &changed, 1, 2, 8).unwrap().status, "FAIL");
    }
    #[test]
    fn completed_sidecar_authenticates_capture_beyond_final_visit() {
        let root = std::env::temp_dir().join(format!(
            "sm-checkpoint-test-{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let path = root.join("capture.csv");
        let header = [vec!["frame", "event"], FIELDS.to_vec()].concat().join(",");
        fs::write(&path, format!("{header}\n1,1,0,0,0,0,0,0,0,0,0,0\n2,2,0,0,0,0,0,0,0,0,0,0\n")).unwrap();
        let rows = read(&path).unwrap();
        assert_eq!(coverage(&path, &rows).unwrap(), 2);
        atomic_toml(
            &path.with_extension("capture.toml"),
            &CaptureMetadata {
                completed_frames: 10,
                checkpoint_sha256: digest(&path).unwrap(),
                movie_sha256: None,
                rom_sha256: None,
                core: None,
            },
        )
        .unwrap();
        assert_eq!(coverage(&path, &rows).unwrap(), 10);
        fs::write(&path, format!("{header}\n1,2,0,0,0,0,0,0,0,0,0,0\n")).unwrap();
        assert!(read(&path).is_err());
        assert!(coverage(&path, &rows).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
