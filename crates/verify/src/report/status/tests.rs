use std::collections::BTreeSet;
use std::path::Path;

use super::super::{FREE_FLOORS, Tree, read};
use super::{Least, Plan, RungFigures};
use crate::harness::free::{Accounting, Floors};
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
fn a_steady_stream_is_a_compared_one_and_its_pairs_are_shown_apart() {
    let steady = "[[steady]]\nname = \"sram\"\n";
    let Ok(parsed) = plan("[]", steady) else {
        panic!("a steady compared stream doesn't parse");
    };
    assert!(parsed.steady(Stream::Sram));
    assert!(!parsed.steady(Stream::Vram));
    assert!(plan("[]", &steady.replace("sram", "wram")).is_err());
    assert!(plan("[]", &steady.replace("sram", "saves")).is_err());
    assert!(plan("[]", &format!("{steady}why = \"saves\"\n")).is_err());

    let mut rung = figures([]);
    (rung.green, rung.pairs) = (1, 4);
    assert_eq!(rung.green_cell(), "1 of 4");
    (rung.steady_green, rung.steady_pairs, rung.steady) = (2, 2, vec!["sram", "timing"]);
    assert_eq!(rung.green_cell(), "1 of 4 (and 2 of 2 on sram, timing)");
}

#[test]
fn the_first_rung_is_measured_over_its_rows_and_claimed_streams() {
    let tree = Tree::measure(root());
    let free = read(&root().join(FREE_FLOORS)).and_then(|text| Floors::parse(&text));
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
    // Its one row, any-boundary:3's, is locked on the six streams it counts, two of them
    // steady and counted apart.
    let row = "chain-any-0003-DFD7";
    assert_eq!((m0.rows, m0.pairs, m0.steady_pairs), (1, 4, 2));
    assert_eq!(m0.steady, ["sram", "timing"]);
    assert!(m0.green <= m0.pairs && m0.steady_green <= m0.steady_pairs);
    assert!(m0.started <= m0.rows);
    // Each domain accounts for every frame of it after its seed, as its line holds.
    for (domain, held) in &m0.held {
        let line = free.domains.get(&(domain.clone(), row.to_owned()));
        assert_eq!(Some(held), line);
        assert_eq!(held.frames(), m0.after_seeds());
    }
    assert!(!m0.held.is_empty());
    assert_eq!(free.uncompared.get(row), Some(&m0.uncompared));
}

#[test]
fn the_least_held_domain_holds_the_smallest_share_of_what_it_compares() {
    let held = |held, compared, out_of_scope| Accounting {
        held,
        compared,
        out_of_scope,
        ..Accounting::default()
    };
    let rung = figures([
        // Holds every frame it compares, though most of the row is out of its scope.
        (String::from("folds"), held(40, 40, 60)),
        // Holds more frames, but not all it compares.
        (String::from("misses"), held(90, 100, 0)),
        // Compares nothing, so holds no share.
        (String::from("absent"), held(0, 0, 100)),
    ]);
    assert_eq!(
        rung.least_held(),
        Some(Least {
            domains: vec!["misses"],
            share: (90, 100),
            every: false,
        })
    );
    assert_eq!(rung.after_seeds(), 100);
    // Two domains that each hold all they compare tie, and are every domain that compares.
    let mut whole = rung;
    whole.held.insert(String::from("misses"), held(100, 100, 0));
    assert_eq!(
        whole.least_held().map(|least| least.cell()),
        Some(String::from("100.0 % (every domain)"))
    );
}

#[test]
fn the_least_gain_is_the_held_share_beyond_what_the_null_model_holds() {
    let held = |held, null_held, compared| Accounting {
        held,
        null_held,
        compared,
        ..Accounting::default()
    };
    let rung = figures([
        // Holds every frame, but so would a step that does nothing on nearly all of them.
        (String::from("still"), held(100, 98, 100)),
        // Holds fewer frames, most of which only the model holds.
        (String::from("moving"), held(90, 10, 100)),
        // Holds fewer than the null model, so gains nothing.
        (String::from("behind"), held(5, 50, 100)),
    ]);
    assert_eq!(
        rung.least_gain(),
        Some(Least {
            domains: vec!["behind"],
            share: (0, 100),
            every: false,
        })
    );
    let mut ahead = rung;
    ahead.held.remove("behind");
    assert_eq!(
        ahead.least_gain().map(|least| least.cell()),
        Some(String::from("2.0 % (still)"))
    );
}

#[test]
fn the_least_compared_domain_compares_the_smallest_share_of_its_scope() {
    let accounted = |compared, unreached, out_of_scope| Accounting {
        compared,
        out_of_scope,
        unreached,
        ..Accounting::default()
    };
    let rung = figures([
        // A passage-only domain: most of the row isn't its, and it compares all that is.
        (String::from("passage"), accounted(30, 0, 70)),
        // Answers for every frame and doesn't reach a tenth of them.
        (String::from("short"), accounted(90, 10, 0)),
        // Answers for none of the row's frames, so has no share.
        (String::from("elsewhere"), accounted(0, 0, 100)),
    ]);
    assert_eq!(
        rung.least_compared(),
        Some(Least {
            domains: vec!["short"],
            share: (90, 100),
            every: false,
        })
    );
}

/// A one-row rung of 101 frames whose domains account for its frames as `held` says.
fn figures<const N: usize>(held: [(String, Accounting); N]) -> RungFigures {
    RungFigures {
        id: String::from("M0"),
        name: String::from("r"),
        span: (0, 101),
        alone: 0,
        still: String::new(),
        order: String::new(),
        steady_green: 0,
        steady_pairs: 0,
        steady: Vec::new(),
        claimed: BTreeSet::new(),
        rows: 1,
        started: 1,
        green: 0,
        pairs: 0,
        frames: 101,
        held: held.into_iter().collect(),
        uncompared: 0,
    }
}
