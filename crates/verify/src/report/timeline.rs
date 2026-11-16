//! Independent movie seed coordinates used to describe the plan's target.

use std::path::Path;

pub(super) const SVG: &str = "docs/timeline.svg";

pub(super) fn seed_frame(root: &Path, capture: &str, wanted: u32) -> Result<usize, String> {
    let path = format!("movies/room-seeds-{capture}.csv");
    let text =
        std::fs::read_to_string(root.join(&path)).map_err(|_| format!("{path}: unavailable"))?;
    let mut lines = text.lines();
    let header: Vec<_> = lines.next().unwrap_or_default().split(',').collect();
    let column = |name: &str| {
        header
            .iter()
            .position(|field| field.trim() == name)
            .ok_or_else(|| format!("{path}: missing column {name}"))
    };
    let (id_column, frame_column) = (column("room_index")?, column("movie_frame")?);
    let mut found = None;
    for (line_number, line) in lines
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
    {
        let fields: Vec<_> = line.split(',').map(str::trim).collect();
        let invalid = || {
            format!(
                "{path}: invalid seed metadata at row {}",
                line_number.saturating_add(2)
            )
        };
        let id: u32 = fields
            .get(id_column)
            .ok_or_else(invalid)?
            .parse()
            .map_err(|_| invalid())?;
        let frame: usize = fields
            .get(frame_column)
            .ok_or_else(invalid)?
            .parse()
            .map_err(|_| invalid())?;
        if id == wanted && found.replace(frame).is_some() {
            return Err(format!("{path}: duplicate seed {wanted}"));
        }
    }
    found.ok_or_else(|| format!("{path}: missing seed {wanted}"))
}
