//! The native origin, qualified by the blackout or departure adapter when opening its source.

use crate::replay::{
    data_failure,
    index::{Case, InputProvider, Responsibility},
    result::Outcome,
};
use serde::Deserialize;

#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::replay) enum Origin {
    Reach,
    Locked,
    Suspended,
    Standing,
    SuspendedStanding,
}

impl Origin {
    /// Index admission can check the reader and scope before opening the pinned native record.
    pub(super) fn candidates(case: &Case) -> &'static [Self] {
        use Origin::{Locked, Reach, Standing, Suspended, SuspendedStanding};
        if case.inputs != InputProvider::Blackout {
            &[]
        } else if case.responsibilities == [Responsibility::Appearance] {
            &[Standing, SuspendedStanding]
        } else {
            &[Reach, Locked, Suspended]
        }
    }

    pub(in crate::replay) fn check(self, case: &Case, cut: u64) -> Result<(), Outcome> {
        if super::valid_for(case, Some(self)) {
            Ok(())
        } else {
            Err(data_failure(
                Some(cut),
                "borrowed operations for the admitted escape origin",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Origin;
    use crate::replay::{borrowed, index::Index};

    #[test]
    fn checking_the_admitted_origin_rejects_the_other_valid_standing_contract() {
        let index =
            Index::parse(include_str!("../../../../../scenarios/replay/index.toml")).unwrap();
        let find = |id| index.cases.iter().find(|case| case.id.0 == id).unwrap();
        let mut suspended = find("appearance/elevator-standing-suspended").clone();
        let locked = find("appearance/elevator-standing-picture");
        assert!(Origin::SuspendedStanding.check(&suspended, 0).is_ok());
        suspended.borrowed.clone_from(&locked.borrowed);
        assert!(borrowed::valid(&suspended));
        assert!(Origin::SuspendedStanding.check(&suspended, 0).is_err());
    }
}
