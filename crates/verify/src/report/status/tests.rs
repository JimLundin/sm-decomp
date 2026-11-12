use std::collections::BTreeSet;
use std::path::Path;

use super::super::{FREE_FLOORS, Tree, floors, read};
use super::Plan;
use crate::lock::Stream;

fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

/// A plan with one domain, CAM, a rung that needs `needs` and the stream claims `claims`.
fn plan(needs: &str, claims: &str) -> Result<Plan, String> {
    Plan::parse(
        &format!(
            "platform = []\n[[model]]\nname = \"world-camera\"\nrounds = 5\n[[domain]]\n\
             id = \"CAM\"\nname = \"The camera\"\narea = \"world\"\nwave = 0\nrounds = 1\n\
             [milestone]\nname = \"m\"\nmovie = \"any\"\nends = \"any:12\"\nneeds = [\"CAM\"]\n\
             [[rung]]\nid = \"M0\"\nname = \"r\"\nstarts = \"any:1\"\nends = \"any:2\"\n\
             needs = {needs}\n{claims}"
        ),
        &BTreeSet::from(["world-camera"]),
    )
}

#[test]
fn a_rung_counts_a_claimed_stream_only_when_it_needs_its_work() {
    let claim = "[[stream]]\nname = \"video\"\ngiven_by = \"CAM\"\n";
    let counts = |needs: &str, stream: Stream| {
        plan(needs, claim).is_ok_and(|plan| {
            plan.rung
                .first()
                .is_some_and(|rung| plan.claims(rung, stream))
        })
    };
    assert!(counts("[\"CAM\"]", Stream::Video));
    assert!(!counts("[]", Stream::Video));
    assert!(counts("[]", Stream::Vram));
    // wram is never compared, so no rung counts it.
    assert!(!counts("[\"CAM\"]", Stream::Wram));
    // A claim names a compared stream and work the plan holds.
    assert!(plan("[]", &claim.replace("video", "wram")).is_err());
    assert!(plan("[]", &claim.replace("video", "picture")).is_err());
    assert!(plan("[]", &claim.replace("\"CAM\"", "\"E3\"")).is_err());
}

#[test]
fn the_first_rung_is_measured_over_its_rows_and_claimed_streams() {
    let tree = Tree::measure(root());
    let free = read(&root().join(FREE_FLOORS)).and_then(|text| floors(&text));
    let (Ok(tree), Ok(free)) = (tree, free) else {
        panic!("the tree doesn't measure");
    };
    let Some(m0) = tree.rungs.iter().find(|rung| rung.id == "M0") else {
        panic!("no rung M0");
    };
    // M0 needs neither the line program nor the sound requests, so it counts neither video
    // nor apu, and never wram.
    for stream in [Stream::Video, Stream::Apu, Stream::Wram] {
        assert!(!m0.claimed.contains(&stream));
    }
    // Its one row, any-boundary:3's, is locked on the six streams it counts.
    let row = "chain-any-0003-DFD7";
    assert_eq!((m0.rows, m0.pairs), (1, 6));
    assert!(m0.green <= m0.pairs && m0.started <= m0.rows);
    for (domain, held) in &m0.held {
        let floor = free.get(&(domain.clone(), row.to_owned())).copied();
        assert_eq!(usize::try_from(*held).ok(), Some(floor.unwrap_or_default()));
    }
    assert!(!m0.held.is_empty());
}
