//! Caller population at world construction, joined to each actual eligible-box search.
//! A prefix describes caller membership; it is not elapsed time or native writer admission.

use crate::collision::solid_enemy::{Obstacle, SolidEnemies};
use crate::state::room::Enemy;
use crate::state::slots::SlotIndex;

/// An occurrence in the ordered caller population, independent of slot reuse or repetition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Occurrence(pub usize);

/// Where the caller obtained a member.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberIdentity {
    /// Actual interactive slot; repeated occurrences remain distinct.
    Interactive(SlotIndex<Enemy>),
    /// Explicitly supplied obstacle without an interactive-slot claim.
    Supplied(Occurrence),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::room::{EnemyBody, EnemyCommon, EnemyProperty};
    use crate::state::slots::{Frames, Slot};
    use core::num::Wrapping;

    fn attempt(candidates: Vec<types::EnemyCandidate>) -> types::MotionAttempt {
        types::MotionAttempt {
            purpose: types::CollisionPurpose::OrdinaryMove,
            direction: types::Direction::Right,
            world: types::RequestedWorld::EnemiesThenBlocks,
            start: types::Pair {
                x: Wrapping(0),
                y: Wrapping(0),
            },
            radius: types::Pair {
                x: Wrapping(5),
                y: Wrapping(21),
            },
            intended: types::TravelDistance(Wrapping(65536)),
            enemy_search: Some(types::EnemySearch {
                supplied_count: types::CandidateCount(candidates.len()),
                candidates,
            }),
            effect: types::RequestEffect::Travel(types::TravelDistance(Wrapping(0))),
            result: types::MotionResult::EnemyStopped,
        }
    }

    fn candidate(index: usize, result: types::CandidateResult) -> types::EnemyCandidate {
        types::EnemyCandidate {
            candidate: types::CandidateIndex(index),
            centre: types::Pair {
                x: Wrapping(10),
                y: Wrapping(10),
            },
            radius: types::Pair {
                x: Wrapping(5),
                y: Wrapping(5),
            },
            result,
        }
    }

    #[test]
    fn repeated_requests_keep_raw_occurrences_properties_and_stop_prefix() {
        let mut slots: [Slot<EnemyCommon, EnemyBody>; 4] =
            core::array::from_fn(|_| Slot::default());
        slots[1].common.frozen = Frames(1);
        slots[2]
            .common
            .properties
            .insert(EnemyProperty::SolidToSamus);
        let interactive = [0, 1, 0, 2, 3].map(|index| SlotIndex(index, core::marker::PhantomData));
        let roster = Roster::interactive(SolidEnemies {
            interactive: &interactive,
            slots: &slots,
        });
        assert_eq!(roster.members.len(), 5);
        assert_eq!(roster.members[0].identity, roster.members[2].identity);
        assert_eq!(roster.eligible, [Occurrence(1), Occurrence(3)]);
        assert!(roster.members[1].obstacle.unwrap().frozen);
        assert!(!roster.members[1].obstacle.unwrap().solid);
        assert!(
            roster.members[2]
                .obstacle
                .is_some_and(|enemy| !enemy.frozen && !enemy.solid)
        );
        let request = attempt(vec![
            candidate(0, types::CandidateResult::Miss),
            candidate(1, types::CandidateResult::Touching),
        ]);
        let fit = types::BodyFitConsideration {
            old_radius: request.radius,
            new_radius: request.radius,
            centre: request.start,
            checks: vec![request.clone(), request.clone()],
            result: types::BodyFitResult::GrowthClear,
            placement: None,
        };
        let searches = roster.searches(&[
            types::MovementWork::Motion(request),
            types::MovementWork::BodyFit(fit),
        ]);
        assert_eq!(searches.len(), 3);
        for search in searches {
            assert_eq!(
                search.prefix,
                [Occurrence(0), Occurrence(1), Occurrence(2), Occurrence(3)]
            );
            assert_eq!(search.end, SearchEnd::Accepted);
            assert_eq!(search.attempt.enemy_search.unwrap().candidates.len(), 2);
        }
    }

    #[test]
    fn empty_population_and_excluded_exhaustion_have_distinct_ends() {
        let request = attempt(Vec::new());
        let work = [types::MovementWork::Motion(request)];
        let empty = Roster::default().searches(&work);
        assert_eq!(empty[0].end, SearchEnd::Empty);
        assert_eq!(empty[0].prefix, []);
        let roster = Roster::supplied([Obstacle {
            position: crate::state::samus::Point::default(),
            radius_x: 1,
            radius_y: 1,
            frozen: false,
            solid: false,
        }]);
        let excluded = roster.searches(&work);
        assert_eq!(excluded[0].prefix, [Occurrence(0)]);
        assert_eq!(excluded[0].end, SearchEnd::Exhausted);
    }
}

/// Current properties retained even for members excluded from the eligible boxes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    /// Identity in the caller's population.
    pub identity: MemberIdentity,
    /// Missing slot data is retained, rather than silently collapsed out of the roster.
    pub obstacle: Option<Obstacle>,
}

/// Owned construction snapshot and eligible-to-raw occurrence mapping.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Roster {
    /// Every supplied occurrence in order, including excluded members.
    pub members: Vec<Member>,
    /// Eligible box ordinal to caller occurrence.
    pub eligible: Vec<Occurrence>,
}

/// A joined actual search; absence of a suffix is distinct from population exhaustion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Search {
    /// Actual request geometry, purpose and result, with its reached eligible candidates.
    pub attempt: types::MotionAttempt,
    /// Caller occurrences through the accepted box, or the entire roster on exhaustion.
    pub prefix: Vec<Occurrence>,
    /// Empty admission bypasses a scan; a nonempty exhausted population reaches its end.
    pub end: SearchEnd,
}

/// Population boundary for a joined request, without a source timing claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchEnd {
    /// No raw members were supplied; the source caller returns before scanning.
    Empty,
    /// A reached eligible member accepted; remaining raw occurrences are unvisited.
    Accepted,
    /// Every raw occurrence lies in the prefix; only this case admits a terminator.
    Exhausted,
    /// A reported eligible identity has no caller mapping; no raw prefix is claimed.
    UnmappedCandidate(types::CandidateIndex),
}

impl Roster {
    pub(super) fn interactive(enemies: SolidEnemies<'_>) -> Self {
        Self::members(enemies.interactive.iter().map(|&slot| {
            Member {
                identity: MemberIdentity::Interactive(slot),
                obstacle: enemies
                    .slots
                    .get(usize::from(slot.0))
                    .map(|enemy| Obstacle::from(&enemy.common)),
            }
        }))
    }

    pub(super) fn supplied(enemies: impl IntoIterator<Item = Obstacle>) -> Self {
        Self::members(
            enemies
                .into_iter()
                .enumerate()
                .map(|(index, obstacle)| Member {
                    identity: MemberIdentity::Supplied(Occurrence(index)),
                    obstacle: Some(obstacle),
                }),
        )
    }

    fn members(members: impl IntoIterator<Item = Member>) -> Self {
        let members: Vec<_> = members.into_iter().collect();
        let eligible = members
            .iter()
            .enumerate()
            .filter(|(_, member)| {
                member
                    .obstacle
                    .is_some_and(|enemy| enemy.solid || enemy.frozen)
            })
            .map(|(index, _)| Occurrence(index))
            .collect();
        Self { members, eligible }
    }

    /// Preserve a separate join for every actual request, including repeated fit scans.
    #[must_use]
    pub fn searches(&self, work: &[types::MovementWork]) -> Vec<Search> {
        let mut searches = Vec::new();
        for event in work {
            match event {
                types::MovementWork::Motion(attempt) => self.join(attempt, &mut searches),
                types::MovementWork::Support(_) => {}
                types::MovementWork::BodyFit(fit) => {
                    for attempt in &fit.checks {
                        self.join(attempt, &mut searches);
                    }
                }
            }
        }
        searches
    }

    fn join(&self, attempt: &types::MotionAttempt, searches: &mut Vec<Search>) {
        let Some(search) = &attempt.enemy_search else {
            return;
        };
        let accepted = search.candidates.last().filter(|candidate| {
            matches!(
                candidate.result,
                types::CandidateResult::Touching | types::CandidateResult::StopsAtGap(_)
            )
        });
        let (end, boundary) = match accepted {
            Some(candidate) => self.eligible.get(candidate.candidate.0).map_or(
                (0, SearchEnd::UnmappedCandidate(candidate.candidate)),
                |occurrence| (occurrence.0.saturating_add(1), SearchEnd::Accepted),
            ),
            None if self.members.is_empty() => (0, SearchEnd::Empty),
            None => (self.members.len(), SearchEnd::Exhausted),
        };
        searches.push(Search {
            attempt: attempt.clone(),
            prefix: (0..end).map(Occurrence).collect(),
            end: boundary,
        });
    }
}
