//! The full-route rooms and their jobs, as `.github/scripts/full_route.py`
//! `plan` and `jobs` give them: every room of a published route replays from
//! its own original-emulator seed, packed into jobs of similar steps.

use crate::scenario::{Lock, Row};

/// The routes, in the order the planner visits them.
pub const ROUTES: [&str; 2] = ["100", "any"];
/// Sixteen balanced jobs across the published routes; the workflow bounds how
/// many run at once.
const TARGET_JOBS: f64 = 16.0;

/// One route room.
struct Room<'a> {
    id: &'a str,
    index: i64,
    steps: i64,
    bundle: Option<&'a str>,
}

/// One route job: its rooms and the bundle they restore.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteJob {
    pub index: usize,
    pub name: String,
    pub steps: i64,
    pub ids: Vec<String>,
    pub route: &'static str,
    pub asset: String,
}

/// The route's rooms in order, from their scenarios and pins.
fn chain<'a>(rows: &'a [Row], route: &str) -> Result<Vec<Room<'a>>, String> {
    rows.iter()
        .filter(|row| row.chain.as_deref() == Some(route))
        .map(|row| {
            let index = row
                .seed
                .as_deref()
                .and_then(|seed| seed.split(':').nth(1))
                .and_then(|index| index.parse().ok())
                .ok_or_else(|| format!("{}: a route room needs its seed capture:index", row.id))?;
            Ok(Room {
                id: &row.id,
                index,
                steps: row.frames,
                bundle: row.bundle.as_deref(),
            })
        })
        .collect()
}

/// The route's ci-data asset and its pinned SHA-256; None until published.
pub fn published(
    rows: &[Row],
    lock: &Lock,
    route: &str,
) -> Result<Option<(String, String)>, String> {
    let rooms = chain(rows, route)?;
    let Some(first) = rooms.first() else {
        return Ok(None);
    };
    if rooms.iter().any(|room| room.bundle != first.bundle) {
        return Ok(None);
    }
    let Some(name) = first.bundle else {
        return Ok(None);
    };
    let digest = lock
        .bundles
        .get(name)
        .ok_or_else(|| format!("route {route}: bundle {name} has no pinned SHA-256"))?;
    Ok(Some((name.to_owned(), digest.clone())))
}

/// Python's `round` of a float: to the nearest integer, ties to even.
pub fn round(value: f64) -> i64 {
    // The value is a job count or a second count, far inside i64.
    value.round_ties_even() as i64
}

/// One route's rooms packed into groups of similar step totals, heaviest group first.
///
/// Every published route's groups target one of TARGET_JOBS equal shares of all
/// their steps. Each room replays from its own seed, so a group need not be
/// contiguous: rooms go heaviest first into the lightest group.
fn groups<'a>(rows: &'a [Row], lock: &Lock, route: &str) -> Result<Vec<Vec<Room<'a>>>, String> {
    let mut entries = chain(rows, route)?;
    let mut total = 0;
    for other in ROUTES {
        if published(rows, lock, other)?.is_some() {
            total += chain(rows, other)?
                .iter()
                .map(|room| room.steps)
                .sum::<i64>();
        }
    }
    let target = total as f64 / TARGET_JOBS;
    let own: i64 = entries.iter().map(|room| room.steps).sum();
    let count = usize::try_from(round(own as f64 / target).max(1)).unwrap_or(1);
    let mut groups: Vec<Vec<Room<'a>>> = (0..count).map(|_| Vec::new()).collect();
    entries.sort_by_key(|room| -room.steps);
    for room in entries {
        let lightest = lightest(&groups, |group| {
            group.iter().map(|room| room.steps).sum::<i64>()
        });
        groups[lightest].push(room);
    }
    for group in &mut groups {
        group.sort_by_key(|room| room.index);
    }
    groups.retain(|group| !group.is_empty());
    groups.sort_by_key(|group| -group.iter().map(|room| room.steps).sum::<i64>());
    Ok(groups)
}

/// The first group with the least load, as Python's `min` picks it.
pub fn lightest<T, K: PartialOrd>(groups: &[T], load: impl Fn(&T) -> K) -> usize {
    let mut best = 0;
    for (index, group) in groups.iter().enumerate().skip(1) {
        if load(group) < load(&groups[best]) {
            best = index;
        }
    }
    best
}

/// One matrix over every published route, heaviest job first (`full_route.py plan`).
pub fn plan(rows: &[Row], lock: &Lock) -> Result<Vec<RouteJob>, String> {
    let mut include = Vec::new();
    for route in ROUTES {
        let Some((asset, _)) = published(rows, lock, route)? else {
            continue;
        };
        for (index, group) in groups(rows, lock, route)?.into_iter().enumerate() {
            include.push(RouteJob {
                index,
                name: format!("{route}-{index:02}-{}-rooms", group.len()),
                steps: group.iter().map(|room| room.steps).sum(),
                ids: group.iter().map(|room| room.id.to_owned()).collect(),
                route,
                asset: asset.clone(),
            });
        }
    }
    if include.is_empty() {
        return Err(
            "no chain bundle is published; see the route pins in scenarios/lock.toml".into(),
        );
    }
    include.sort_by_key(|job| -job.steps);
    Ok(include)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_as_python_does() {
        assert_eq!([0.5, 1.5, 2.5, 2.6, -0.5].map(round), [0, 2, 2, 3, 0]);
    }

    #[test]
    fn lightest_takes_the_first_of_equal_loads() {
        assert_eq!(lightest(&[3, 1, 1], |&load| load), 1);
        assert_eq!(lightest(&[0, 0], |&load| load), 0);
    }
}
