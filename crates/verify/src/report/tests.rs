use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::lines::code_lines;
use super::status::{PLAN, Plan, undeclared};
use super::{
    Coverage, DOMAINS, EntryCoverage, Frames, Proof, baseline, check, coverage, current, fit,
    floors, generate, grouped, read, status_current, tables, with_status,
};

/// The repository root, from this crate's manifest.
fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

#[test]
fn committed_report_is_current() {
    assert_eq!(check(root()), Ok(()));
}

#[test]
fn a_changed_report_is_stale() {
    let fresh = "# Rebuild report\n\n| a | 1 |\n";
    assert_eq!(current(fresh, fresh), Ok(()));
    assert_eq!(current(&fresh.replace('\n', "\r\n"), fresh), Ok(()));
    assert!(current("# Rebuild report\n\n| a | 2 |\n", fresh).is_err());
    assert!(current("", fresh).is_err());
}

#[test]
fn a_changed_status_is_stale() {
    let fresh = "\n| rounds | 1 |\n";
    let readme = "# Title\n\n<!-- status:begin -->\n| rounds | 1 |\n<!-- status:end -->\n\nMore.\n";
    assert_eq!(status_current(readme, fresh), Ok(()));
    assert_eq!(status_current(&readme.replace('\n', "\r\n"), fresh), Ok(()));
    // The text around the markers is the README's own.
    assert_eq!(
        status_current(&readme.replace("More.", "Other."), fresh),
        Ok(())
    );
    assert!(status_current(&readme.replace("| 1 |", "| 2 |"), fresh).is_err());
    assert!(status_current("# Title\n", fresh).is_err());
}

#[test]
fn the_status_goes_between_the_markers() {
    let readme = "# Title\n<!-- status:begin -->\nold\n<!-- status:end -->\nMore.\n";
    assert_eq!(
        with_status(readme, "\nnew\n").as_deref(),
        Ok("# Title\n<!-- status:begin -->\nnew\n<!-- status:end -->\nMore.\n")
    );
    assert!(with_status("# Title\n<!-- status:end -->\n", "\nnew\n").is_err());
}

#[test]
fn the_report_is_deterministic() {
    let first = generate(root());
    assert!(first.is_ok());
    assert_eq!(first, generate(root()));
}

#[test]
fn blank_and_comment_lines_dont_count() {
    let src = "//! Crate doc.\n\n/// Item doc.\nfn a() {\n    // a comment\n    let x = 1; // trailing\n}\n";
    assert_eq!(code_lines(src), 3);
}

#[test]
fn block_comments_nest() {
    let src = "/* one\n /* two */\n still */\nfn a() {}\n/** doc */\n";
    assert_eq!(code_lines(src), 1);
}

#[test]
fn literals_are_code_and_hide_comment_marks() {
    let src = "let a = \"// not a comment\";\nlet b = \"line\n\n/* inside */\";\nlet c = r#\"x \" // y\"#;\n";
    assert_eq!(code_lines(src), 4);
}

#[test]
fn test_items_dont_count() {
    let src = "fn a() {}\n#[cfg(test)]\nmod tests {\n    fn b() { let c = '{'; }\n}\n#[cfg(test)]\nuse std::fmt;\nfn d<'a>(x: &'a str) -> char { '}' }\n#[cfg(test)]\nmod tests;\n";
    assert_eq!(code_lines(src), 2);
}

#[test]
fn figures_read_as_an_observer_would() {
    assert_eq!(grouped(0), "0");
    assert_eq!(grouped(1_234_567), "1,234,567");
    assert_eq!(fit(8 * 104, Some(100)), "1.04");
    assert_eq!(fit(8, None), "-");
}

#[test]
fn the_baseline_commit_is_the_last_the_budget_paragraph_names() {
    let section =
        "**Budget.** at spine `bc6f74f9c`, over `x + 1` at `c7f927ab6`.\n\nLater `0123456789`.\n";
    assert_eq!(baseline(section).as_deref(), Some("c7f927ab6"));
    // A baseline paragraph names it first, wherever it stands.
    let section = "**Budget.** k = 36, at `c7f927ab6`.\n\n**Baseline.** at spine `387640758`.\n";
    assert_eq!(baseline(section).as_deref(), Some("387640758"));
}

#[test]
fn floors_and_tables_parse() {
    let parsed = floors("# floors\ncamera r1 10\ncamera r2 5\n");
    assert_eq!(parsed.map(|floors| floors.values().sum::<usize>()), Ok(15));
    assert!(floors("camera r1\n").is_err());
    assert_eq!(
        tables("[[vector]]\nid = 1\n[[pose_step]]\n[[vector]]\n", "vector"),
        2
    );
}

#[test]
fn coverage_parses() {
    let parsed = coverage(
        "# coverage\ncamera frames 12 10 8 1 5\ncamera field 9 1 position x\n\
         camera entry Follow 3 1 4 2\n",
    );
    let camera = Coverage {
        frames: Some(Frames {
            ran: 12,
            compared: 10,
            matched: 8,
            unverified: 1,
            null_matched: 5,
        }),
        fields: vec![("position x".to_owned(), 9, 1)],
        entries: vec![EntryCoverage {
            name: "Follow".to_owned(),
            matched: 3,
            null_matched: 1,
            exercised: 4,
            unchecked: 2,
        }],
    };
    assert_eq!(
        parsed.map(|domains| domains.get("camera") == Some(&camera)),
        Ok(true)
    );
    assert!(coverage("camera frames 10 8 1 5\n").is_err());
    assert!(coverage("camera entry Follow x 0 4 0\n").is_err());
    assert!(coverage("camera entry Follow 3 4 2\n").is_err());
    assert!(coverage("camera field 9 1\n").is_err());
}

#[test]
fn a_models_headline_is_its_gain_over_the_null_model() {
    let frames = |matched, null_matched| Frames {
        ran: 120,
        compared: 100,
        matched,
        unverified: 0,
        null_matched,
    };
    assert_eq!(frames(80, 30).gain(), 50);
    // Matching fewer frames than doing nothing gains nothing.
    assert_eq!(frames(30, 80).gain(), 0);
    // Under a tenth of the compared frames is flagged; a tenth is not.
    assert!(frames(99, 90).weak());
    assert!(!frames(100, 90).weak());
    let entry = |matched, null_matched, exercised| EntryCoverage {
        name: String::from("Follow"),
        matched,
        null_matched,
        exercised,
        unchecked: 0,
    };
    assert_eq!(entry(5, 2, 6).proof(), Proof::Gain(3));
    // The null model passing as often as the model leaves the entry unproven, not matched.
    assert_eq!(entry(5, 5, 6).proof(), Proof::Unproven);
    assert_eq!(entry(0, 0, 6).proof(), Proof::Unproven);
    assert_eq!(entry(0, 0, 0).proof(), Proof::VectorsOnly);
}

#[test]
fn the_plan_agrees_with_the_rebuilt_domains() {
    let rebuilt = BTreeSet::from(["world-camera"]);
    let plan = |domain: &str| {
        format!(
            "platform = []\n[[model]]\nname = \"world-camera\"\nrounds = 5\n[[domain]]\n{domain}\
             [milestone]\nname = \"m\"\nmovie = \"any\"\nends = \"any:12\"\nneeds = [\"CAM\"]\n"
        )
    };
    let parse = |domain: &str| Plan::parse(&plan(domain), &rebuilt).map(|_| ());
    let named = "id = \"CAM\"\nname = \"The camera\"\narea = \"world\"\nwave = 0\nrounds = 1\n";
    assert_eq!(parse(named), Ok(()));
    // The milestone needs plan items the plan holds, and ends at a seed.
    let other = |from: &str, to: &str| Plan::parse(&plan(named).replace(from, to), &rebuilt);
    assert!(other("[\"CAM\"]", "[\"CAM\", \"R1\"]").is_err());
    assert!(other("any:12", "any").is_err());
    assert!(other("any:12", "any:twelve").is_err());
    // A rung of the ladder needs plan items the plan holds, and runs between two seeds.
    let rung = |needs: &str, starts: &str| {
        Plan::parse(
            &format!(
                "{}[[rung]]\nid = \"M0\"\nname = \"r\"\nstarts = \"{starts}\"\n\
                 ends = \"any:2\"\nneeds = {needs}\n",
                plan(named)
            ),
            &rebuilt,
        )
        .map(|_| ())
    };
    assert_eq!(rung("[\"CAM\"]", "any:1"), Ok(()));
    assert!(rung("[\"R1\"]", "any:1").is_err());
    assert!(rung("[\"CAM\"]", "any").is_err());
    // Platform work comes before other work of the plan, and a stretch that needs that work
    // needs the platform work too.
    let before = |ids: &str, needs: &str| {
        let platform = format!(
            "[[platform]]\nid = \"AF1\"\nname = \"a\"\nwave = 0\nlanes = 1\nbefore = {ids}\n"
        );
        Plan::parse(
            &plan(named)
                .replace("platform = []\n", &platform)
                .replace("needs = [\"CAM\"]", &format!("needs = {needs}")),
            &rebuilt,
        )
        .map(|_| ())
    };
    assert_eq!(before("[\"CAM\"]", "[\"CAM\", \"AF1\"]"), Ok(()));
    assert!(before("[\"CAM\"]", "[\"CAM\"]").is_err());
    assert!(before("[\"R1\"]", "[\"CAM\", \"AF1\"]").is_err());
    assert!(before("[\"AF1\"]", "[\"CAM\", \"AF1\"]").is_err());
    assert_eq!(
        parse(&format!(
            "{named}model = \"world-camera\"\nstatus = \"in-progress\"\n"
        )),
        Ok(())
    );
    // A domain's model is named in its area, and the area is one of the plan's.
    let started = format!("{named}model = \"world-camera\"\n");
    assert!(parse(&started.replace("\"world\"", "\"samus\"")).is_err());
    assert!(parse(&named.replace("\"world\"", "\"rooms\"")).is_err());
    // A started domain names its model, and a named model is one the report rebuilds.
    assert!(parse(&format!("{named}status = \"in-progress\"\n")).is_err());
    assert!(parse(&format!("{named}model = \"interface-hud\"\n")).is_err());
    assert!(parse(&format!("{named}status = \"started\"\n")).is_err());
    // Every rebuilt domain is a model of the plan, once.
    let others = BTreeSet::from(["world-camera", "interface-hud"]);
    assert!(Plan::parse(&plan(named), &others).is_err());
}

#[test]
fn a_domain_reads_only_other_domains_of_the_plan() {
    let rebuilt = BTreeSet::from(["world-camera"]);
    let plan = |reads: &str| {
        format!(
            "platform = []
[[model]]
name = \"world-camera\"
rounds = 5
             [[domain]]
id = \"CAM\"
name = \"c\"
area = \"world\"
wave = 0
rounds = 1
{reads}             [[domain]]
id = \"S1\"
name = \"s\"
area = \"samus\"
wave = 0
rounds = 1
             [milestone]
name = \"m\"
movie = \"any\"
ends = \"any:12\"
needs = []
"
        )
    };
    let parse = |reads: &str| Plan::parse(&plan(reads), &rebuilt).map(|_| ());
    assert_eq!(parse(""), Ok(()));
    assert_eq!(
        parse(
            "reads = [\"S1\"]
"
        ),
        Ok(())
    );
    assert!(
        parse(
            "reads = [\"S9\"]
"
        )
        .is_err()
    );
    assert!(
        parse(
            "reads = [\"CAM\"]
"
        )
        .is_err()
    );
}

#[test]
fn the_clean_crates_use_only_the_domains_they_read() {
    let names: BTreeSet<&str> = DOMAINS.iter().map(|domain| domain.name).collect();
    let plan = read(&root().join(PLAN)).and_then(|text| Plan::parse(&text, &names));
    assert_eq!(plan.and_then(|plan| undeclared(root(), &plan)), Ok(()));
}

/// A tree of two clean crates, `world_camera` and `samus_movement`, with `types` and a bridge
/// each, under the system's temporary folder; its plan has the camera read `reads`.
struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("verify-report-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for krate in ["types", "world_camera", "samus_movement"] {
            let dir = root.join("crates/clean").join(krate);
            assert!(fs::create_dir_all(&dir).is_ok());
            let types = if krate == "types" {
                ""
            } else {
                "types = { path = \"../types\" }
"
            };
            assert!(
                fs::write(
                    dir.join("Cargo.toml"),
                    format!(
                        "[dependencies]
{types}"
                    )
                )
                .is_ok()
            );
        }
        let bridges = root.join("crates/game/src/world_camera_bridge");
        assert!(fs::create_dir_all(&bridges).is_ok());
        assert!(
            fs::write(
                bridges.with_extension("rs"),
                "use types::Parity;
"
            )
            .is_ok()
        );
        assert!(
            fs::write(
                bridges.join("tests.rs"),
                "use world_camera::View;
"
            )
            .is_ok()
        );
        Self { root }
    }

    fn write(&self, path: &str, text: &str) {
        assert!(fs::write(self.root.join(path), text).is_ok());
    }

    fn check(&self, reads: &str) -> Result<(), String> {
        let rebuilt = BTreeSet::from(["world-camera", "samus-movement"]);
        let text = format!(
            "platform = []
[[model]]
name = \"world-camera\"
rounds = 1
             [[model]]
name = \"samus-movement\"
rounds = 1
             [[domain]]
id = \"CAM\"
name = \"c\"
area = \"world\"
wave = 0
rounds = 1
             model = \"world-camera\"
{reads}             [[domain]]
id = \"S1\"
name = \"s\"
area = \"samus\"
wave = 0
rounds = 1
             model = \"samus-movement\"
             [milestone]
name = \"m\"
movie = \"any\"
ends = \"any:12\"
needs = []
"
        );
        undeclared(&self.root, &Plan::parse(&text, &rebuilt)?)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn a_use_of_a_domain_not_read_is_found() {
    let fixture = Fixture::new("uses");
    let reads = "reads = [\"S1\"]
";
    // Its own crate and the shared types, in the bridge and its tests, need no read.
    assert_eq!(fixture.check(""), Ok(()));
    // A bridge's use of another domain's crate needs the read.
    fixture.write(
        "crates/game/src/world_camera_bridge/tests.rs",
        "use samus_movement::Outcome;
",
    );
    assert!(fixture.check("").is_err());
    assert_eq!(fixture.check(reads), Ok(()));
    // So does a crate's dependency, in any table of its manifest.
    fixture.write(
        "crates/clean/world_camera/Cargo.toml",
        "[dev-dependencies]
samus_movement = { path = \"../samus_movement\" }
",
    );
    assert!(fixture.check("").is_err());
    assert_eq!(fixture.check(reads), Ok(()));
    // The read runs one way: the reader may use the read domain's crate, not the reverse.
    fixture.write(
        "crates/game/src/samus_movement_bridge.rs",
        "use world_camera::View;
",
    );
    assert!(fixture.check(reads).is_err());
    // A clean crate whose model holds no domain of the plan is found too.
    fixture.write(
        "crates/game/src/samus_movement_bridge.rs",
        "
",
    );
    assert!(fs::create_dir_all(fixture.root.join("crates/clean/enemy_families")).is_ok());
    fixture.write(
        "crates/clean/enemy_families/Cargo.toml",
        "[dependencies]
",
    );
    assert!(fixture.check(reads).is_err());
}
