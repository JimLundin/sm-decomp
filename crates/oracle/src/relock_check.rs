//! Checking format-2 recordings against each other (docs/oracle/RELOCK.md §6,
//! step d). lsnes's `video` stream rarely differs between two runs of a row,
//! so a lock is recorded at least twice and an entry is kept only where two
//! recordings agree byte for byte.
//!
//! A recording is one `--record-v2` file or several (the shards of one run),
//! read as text: the header, then each `[[row]]` entry up to the next.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

/// One `[[row]]` entry as recorded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    /// The entry's lines, LF-terminated, without trailing blank lines.
    pub text: String,
    /// `streams.NAME = [...]`: each stream's span marks, in entry order.
    pub streams: Vec<(String, Vec<String>)>,
    /// Every other line of the entry (the pins, schedules and facts).
    pub pins: Vec<String>,
}

/// One run's recording: the header its files share and their rows.
#[derive(Clone, Debug, Default)]
pub struct Recording {
    pub header: String,
    pub rows: Vec<Row>,
}

impl Recording {
    pub fn row(&self, id: &str) -> Option<&Row> {
        self.rows.iter().find(|row| row.id == id)
    }

    /// The recording in `files`, its shards' files separated by commas.
    pub fn read(files: &str) -> Result<Self, String> {
        let mut recording = Self::default();
        for (number, file) in files.split(',').enumerate() {
            let text = std::fs::read_to_string(file).map_err(|error| format!("{file}: {error}"))?;
            let part = Self::parse(&text).map_err(|error| format!("{file}: {error}"))?;
            if number > 0 && part.header != recording.header {
                return Err(format!("{file}: its header differs from the first file's"));
            }
            recording.header = part.header;
            recording.rows.extend(part.rows);
        }
        Ok(recording)
    }

    /// A recording from one file's text.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut header = String::new();
        let mut entries: Vec<Vec<&str>> = Vec::new();
        for line in text.lines() {
            if line.trim() == "[[row]]" {
                entries.push(vec![line]);
            } else if let Some(entry) = entries.last_mut() {
                entry.push(line);
            } else if !line.trim().is_empty() {
                header.push_str(line);
                header.push('\n');
            }
        }
        let rows = entries
            .into_iter()
            .map(|mut lines| {
                while lines.last().is_some_and(|line| line.trim().is_empty()) {
                    lines.pop();
                }
                Row::parse(&lines)
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { header, rows })
    }
}

impl Row {
    fn parse(lines: &[&str]) -> Result<Self, String> {
        let mut id = None;
        let mut streams = Vec::new();
        let mut pins = Vec::new();
        for &line in lines.iter().skip(1) {
            if let Some(value) = line.strip_prefix("id = ") {
                id = Some(value.trim_matches('"').to_owned());
            }
            match line
                .strip_prefix("streams.")
                .and_then(|rest| rest.split_once(" = "))
            {
                Some((name, marks)) => {
                    let marks = marks
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .split(',')
                        .map(|mark| mark.trim().trim_matches('"').to_owned())
                        .filter(|mark| !mark.is_empty())
                        .collect();
                    streams.push((name.to_owned(), marks));
                }
                None => pins.push(line.to_owned()),
            }
        }
        let id = id.ok_or_else(|| "a [[row]] without an id".to_owned())?;
        let text = lines.iter().map(|line| format!("{line}\n")).collect();
        Ok(Self {
            id,
            text,
            streams,
            pins,
        })
    }

    fn marks(&self, stream: &str) -> Option<&[String]> {
        self.streams
            .iter()
            .find(|(name, _)| name == stream)
            .map(|(_, marks)| &marks[..])
    }
}

/// Where two runs' sidecars first differ in one stream's column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FirstFrame {
    /// The line of the first differing frame in each sidecar.
    At { left: String, right: String },
    /// One sidecar ends first, after this many frames.
    Shorter(usize),
    /// The column is equal on every frame both have.
    Equal,
    /// A sidecar is missing or has no such column.
    Unknown(String),
}

/// The first frame whose `stream` column differs between two sidecars.
pub fn first_frame(left: &str, right: &str, stream: &str) -> FirstFrame {
    let column = |text: &str| {
        let head = text.lines().next()?.strip_prefix("# ")?;
        head.split(' ').position(|name| name == stream)
    };
    let (Some(column), Some(other)) = (column(left), column(right)) else {
        return FirstFrame::Unknown(format!("no {stream} column"));
    };
    if column != other {
        return FirstFrame::Unknown("the sidecars' columns differ".to_owned());
    }
    let mut lines = (left.lines().skip(1), right.lines().skip(1));
    let mut frames = 0;
    loop {
        match (lines.0.next(), lines.1.next()) {
            (Some(left), Some(right)) => {
                if left.split(' ').nth(column) != right.split(' ').nth(column) {
                    return FirstFrame::At {
                        left: left.to_owned(),
                        right: right.to_owned(),
                    };
                }
                frames += 1;
            }
            (None, None) => return FirstFrame::Equal,
            (Some(_), None) | (None, Some(_)) => return FirstFrame::Shorter(frames),
        }
    }
}

/// A row's sidecar file in `directory`.
pub fn sidecar_path(directory: &Path, id: &str) -> std::path::PathBuf {
    directory.join(format!("{id}.frames"))
}

/// The report of comparing two recordings, and how many rows disagree.
pub fn compare(
    left: &Recording,
    right: &Recording,
    sidecars: Option<(&Path, &Path)>,
) -> (String, usize) {
    let mut report = String::new();
    let mut disagreeing = 0;
    if left.header != right.header {
        disagreeing += 1;
        report.push_str("header: differs\n");
    }
    let mut ids: Vec<&str> = left.rows.iter().map(|row| row.id.as_str()).collect();
    ids.extend(
        right
            .rows
            .iter()
            .map(|row| row.id.as_str())
            .filter(|id| left.row(id).is_none()),
    );
    for id in ids {
        let (a, b) = match (left.row(id), right.row(id)) {
            (Some(a), Some(b)) => (a, b),
            (Some(_), None) => {
                disagreeing += 1;
                let _ = writeln!(report, "{id}: only in A");
                continue;
            }
            (None, _) => {
                disagreeing += 1;
                let _ = writeln!(report, "{id}: only in B");
                continue;
            }
        };
        if a.text == b.text {
            let _ = writeln!(report, "{id}: agree");
            continue;
        }
        disagreeing += 1;
        let _ = writeln!(report, "{id}: DISAGREE");
        for (pin, other) in a.pins.iter().zip(&b.pins).filter(|(a, b)| a != b) {
            let _ = writeln!(report, "  A: {pin}\n  B: {other}");
        }
        if a.pins.len() != b.pins.len() {
            let _ = writeln!(report, "  the entries have different keys");
        }
        let texts = sidecars.map(|(da, db)| {
            let read = |directory: &Path| std::fs::read_to_string(sidecar_path(directory, id));
            (read(da), read(db))
        });
        let names: Vec<&str> = a
            .streams
            .iter()
            .chain(b.streams.iter().filter(|(name, _)| a.marks(name).is_none()))
            .map(|(name, _)| name.as_str())
            .collect();
        for name in names {
            let (marks_a, marks_b) = (a.marks(name), b.marks(name));
            if marks_a == marks_b {
                continue;
            }
            let span = marks_a
                .unwrap_or_default()
                .iter()
                .zip(marks_b.unwrap_or_default())
                .position(|(a, b)| a != b);
            let _ = match span {
                Some(span) => write!(report, "  {name}: span {span} differs"),
                None => write!(report, "  {name}: the span counts differ"),
            };
            match &texts {
                Some((Ok(da), Ok(db))) => match first_frame(da, db, name) {
                    FirstFrame::At { left, right } => {
                        let _ = write!(report, "; first frame\n    A: {left}\n    B: {right}");
                    }
                    FirstFrame::Shorter(frames) => {
                        let _ = write!(report, "; a sidecar ends after {frames} frames");
                    }
                    FirstFrame::Equal => report.push_str("; the sidecars agree on every frame"),
                    FirstFrame::Unknown(why) => {
                        let _ = write!(report, "; sidecars: {why}");
                    }
                },
                Some(_) => report.push_str("; no sidecar for the row"),
                None => {}
            }
            report.push('\n');
        }
    }
    let _ = writeln!(report, "{disagreeing} disagreeing");
    (report, disagreeing)
}

/// The merged lock text from two or three recordings: the header and each
/// row's entry where at least two recordings agree byte for byte, in the
/// order rows first appear; and the rows with no agreeing pair.
pub fn merge(recordings: &[Recording]) -> Result<(String, Vec<String>), String> {
    let agreed = |texts: &[&str]| -> Option<String> {
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for text in texts {
            *counts.entry(text).or_default() += 1;
        }
        let best = counts.into_iter().max_by_key(|&(_, count)| count)?;
        (best.1 >= 2).then(|| best.0.to_owned())
    };
    let headers: Vec<&str> = recordings
        .iter()
        .map(|recording| recording.header.as_str())
        .collect();
    let mut text = agreed(&headers).ok_or("no two recordings share a header")?;
    let mut ids: Vec<&str> = Vec::new();
    for row in recordings.iter().flat_map(|recording| &recording.rows) {
        if !ids.contains(&row.id.as_str()) {
            ids.push(&row.id);
        }
    }
    let mut unmatched = Vec::new();
    for id in ids {
        let texts: Vec<&str> = recordings
            .iter()
            .filter_map(|recording| recording.row(id))
            .map(|row| row.text.as_str())
            .collect();
        match agreed(&texts) {
            Some(entry) => {
                text.push('\n');
                text.push_str(&entry);
            }
            None => unmatched.push(id.to_owned()),
        }
    }
    Ok((text, unmatched))
}

/// `relock-compare A B [--sidecars DA DB]`: prints [`compare`]'s report;
/// exits 1 on any disagreement, 2 on bad arguments or files.
pub fn compare_command(arguments: &[String]) -> i32 {
    let (a, b, sidecars) = match arguments {
        [a, b] => (a, b, None),
        [a, b, option, da, db] if option == "--sidecars" => {
            (a, b, Some((Path::new(da), Path::new(db))))
        }
        _ => {
            eprintln!("relock-compare A[,A2...] B[,B2...] [--sidecars DA DB]");
            return 2;
        }
    };
    match (Recording::read(a), Recording::read(b)) {
        (Ok(a), Ok(b)) => {
            let (report, disagreeing) = compare(&a, &b, sidecars);
            print!("{report}");
            i32::from(disagreeing > 0)
        }
        (Err(error), _) | (_, Err(error)) => {
            eprintln!("{error}");
            2
        }
    }
}

/// `relock-merge OUT A B [C]`: writes [`merge`]'s lock to OUT and prints the
/// rows with no agreeing pair; exits 1 when there are any.
pub fn merge_command(arguments: &[String]) -> i32 {
    let [out, inputs @ ..] = arguments else {
        eprintln!("relock-merge OUT A[,A2...] B[,B2...] [C[,C2...]]");
        return 2;
    };
    if !(2..=3).contains(&inputs.len()) {
        eprintln!("relock-merge takes two or three recordings");
        return 2;
    }
    let merged = inputs
        .iter()
        .map(|files| Recording::read(files))
        .collect::<Result<Vec<_>, _>>()
        .and_then(|recordings| merge(&recordings))
        .and_then(|(text, unmatched)| {
            std::fs::write(out, text).map_err(|error| format!("{out}: {error}"))?;
            Ok(unmatched)
        });
    match merged {
        Ok(unmatched) => {
            for id in &unmatched {
                println!("{id}: no two recordings agree");
            }
            println!("{} rows without an agreeing pair", unmatched.len());
            i32::from(!unmatched.is_empty())
        }
        Err(error) => {
            eprintln!("{error}");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "format = 2\nmark_frames = 1000\n";

    fn entry(id: &str, video: &str) -> String {
        format!("[[row]]\nid = \"{id}\"\nframes = 2\nstreams.video = [\"{video}\"]\n")
    }

    fn recording(rows: &[(&str, &str)]) -> Recording {
        let text: String = rows
            .iter()
            .map(|(id, video)| format!("\n{}", entry(id, video)))
            .collect();
        Recording::parse(&format!("{HEADER}{text}")).unwrap_or_default()
    }

    #[test]
    fn compare_names_the_disagreeing_stream() {
        let a = recording(&[("one", "aa"), ("two", "bb")]);
        let b = recording(&[("one", "aa"), ("two", "cc")]);
        let (report, disagreeing) = compare(&a, &b, None);
        assert_eq!(disagreeing, 1);
        assert!(report.contains("two: DISAGREE\n  video: span 0 differs"));
    }

    #[test]
    fn merge_keeps_majority_entries() {
        let a = recording(&[("one", "aa"), ("two", "bb")]);
        let b = recording(&[("one", "aa"), ("two", "cc")]);
        let c = recording(&[("one", "dd"), ("two", "ee")]);
        let merged = merge(&[a, b, c]).unwrap_or_default();
        assert_eq!(merged.0, format!("{HEADER}\n{}", entry("one", "aa")));
        assert_eq!(merged.1, ["two"]);
    }

    #[test]
    fn first_frame_finds_the_column() {
        let head = "# frame lsnes_frame cut video timing\n";
        let left = format!("{head}0 10 R aa 11\n1 11 R bb 11\n");
        let right = format!("{head}0 10 R aa 11\n1 11 R cc 11\n");
        assert_eq!(first_frame(&left, &right, "timing"), FirstFrame::Equal);
        assert_eq!(
            first_frame(&left, &right, "video"),
            FirstFrame::At {
                left: "1 11 R bb 11".to_owned(),
                right: "1 11 R cc 11".to_owned()
            }
        );
    }
}
