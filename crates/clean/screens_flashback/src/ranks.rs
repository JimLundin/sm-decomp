//! Spec: `FightPictures`. Draw ranks: each live picture, and each one waiting to appear, holds
//! a rank of its own, and a new one takes the foremost rank left vacant.

use types::flashback::{DrawRank, EncounterState};

/// Spec: `FightPictures`. The pending ring pair reserves the foremost rank.
const PAIR_RANK: DrawRank = DrawRank::FOREMOST;

/// The ranks held now: each ring's, each burst's (waiting and blank ones included) and the
/// pending pair's.
pub fn taken(state: &EncounterState) -> Vec<DrawRank> {
    let rings = state.rings.values().flatten().map(|ring| ring.rank);
    let bursts = state.bursts.values().flatten().map(|burst| burst.rank);
    let pair = state.next_pair.map(|_| PAIR_RANK);
    rings.chain(bursts).chain(pair).collect()
}

/// The foremost rank not in `taken`, which it then joins.
pub fn claim(taken: &mut Vec<DrawRank>) -> DrawRank {
    let mut rank = DrawRank::FOREMOST;
    while taken.contains(&rank) {
        rank = rank.behind();
    }
    taken.push(rank);
    rank
}
