// Generated once from tools/scenario.py lock --check over sm_pipeline/fixture; never edit.
// Each case: its edits to a copy of the fixture (file, old text, new text), then
// whether the check passed and what it printed (stdout) or exited with (stderr);
// None where scenario.py crashed (a KeyError) instead of naming the problem.

const CASES: &[LockCase] = &[
    LockCase {
        label: "current",
        edits: &[],
        passes: true,
        message: Some(r#"Scenarios current: 56 scenarios, 56 pins, 20 bundles; 41 locked, 6 unlocked
"#),
    },
    LockCase {
        label: "stale-entry",
        edits: &[("scenarios/corpus.toml", r#"id = "area-ceres"
start = "seed:c555cbf4669a2d3ed5e444aeb8e3e97d03bacafba50d5c93ec4d5c9dd42ae889"
ticks = 300
"#, r#"id = "area-ceres"
start = "seed:c555cbf4669a2d3ed5e444aeb8e3e97d03bacafba50d5c93ec4d5c9dd42ae889"
ticks = 301
"#)],
        passes: false,
        message: Some(r#"Scenarios or their lock are not current:
area-ceres: stale lock entry (start, inputs or budget changed); re-record it
"#),
    },
    LockCase {
        label: "missing-pin",
        edits: &[("scenarios/lock.toml", r#"    { id = "samus-scripted-intro", inputs = "corpus-speed-booster.inputs", input_sha256 = "bc907fc429252418216b7f575e26be148f7cf863a71c3947b4ffa3c6c6027de6" },
"#, r#""#)],
        passes: false,
        message: Some(r#"Scenarios or their lock are not current:
samus-scripted-intro: no pinned inputs
samus-scripted-intro: its private data is in no bundle
"#),
    },
    LockCase {
        label: "missing-pin-of-a-locked-row",
        edits: &[("scenarios/lock.toml", r#"    { id = "area-crateria", bundle = "seeds-v1-six-3a87864fbffb215a.tar.gz.enc", seed = "100-input-phase:12", inputs = "seed-area-crateria.inputs", input_sha256 = "3d0413b939faefc465e96ea037b4c7a9bdc019cdd975562348be445e6631043d" },
"#, r#""#)],
        passes: false,
        message: None,
    },
    LockCase {
        label: "unlocked-lockable-row",
        edits: &[("scenarios/lock.toml", r#"
[[scenario]]
id = "area-wrecked-ship"
oracle = "faithful"
start = "seed:8532ace90d98b9b90ac9a99dd831562ef24080bbfabca0d000d8a73799b99546"
inputs = "3d55cc2d756391519425b7cfc66bec5462c99edd8eab04030ef42133fc2d41cf"
frames = 300
format = 1
ticks = 300
combined = "2e62cf2a0c4b9a78"
digests = { "wram" = "cd8dd6a78ec151ec", "sram" = "3cd3fb01dc8b29b5", "vram" = "245c36b4151aa3a8", "cgram" = "9041e6d68cb2b46d", "oam" = "d88560343676ce86", "ppu" = "a386e636465a7985", "video" = "fa7b5e1cf6413c29", "apu" = "e98d7b4f3e3ed39f", "input" = "8f5ef71586bf2bb5", "samus" = "1e0e33d122061062", "camera" = "0788a0cbf4086b8c", "room" = "5c55dba0b61c420b", "progress" = "08a5505237f4d1ce", "actors" = "6ae4e8d0f41b7e67", "effects" = "69b51e8c579b4221" }
marks = {  }

"#, r#"
"#)],
        passes: false,
        message: Some(r#"Scenarios or their lock are not current:
area-wrecked-ship: neither locked nor unlocked
"#),
    },
    LockCase {
        label: "locked-and-unlocked",
        edits: &[("scenarios/corpus.toml", r#"id = "boss-phantoon"
start = "seed:14710aef93ab5d7de5be17122739391b7922bb5a5a9480e9e37755f28800bf3e"
ticks = 2062
"#, r#"id = "boss-phantoon"
start = "seed:14710aef93ab5d7de5be17122739391b7922bb5a5a9480e9e37755f28800bf3e"
ticks = 2062
unlocked = "a known divergence"
"#)],
        passes: false,
        message: Some(r#"Scenarios or their lock are not current:
boss-phantoon: both locked and unlocked
"#),
    },
    LockCase {
        label: "no-such-scenario",
        edits: &[("scenarios/corpus.toml", r#"
[[scenario]]
id = "area-ceres"
start = "seed:c555cbf4669a2d3ed5e444aeb8e3e97d03bacafba50d5c93ec4d5c9dd42ae889"
ticks = 300
facts = ["area:ceres", "room:DF45", "probe:player_projectiles.prepare.weapons", "probe:player_projectiles.history.finished"]
after_expectations = 120
tags = ["required"]

# Both scripted intro scenes, followed through the return to ordinary gameplay."#, r#""#)],
        passes: false,
        message: Some(r#"Scenarios or their lock are not current:
area-ceres: pinned, but no such scenario
area-ceres: locked, but no such lockable scenario
"#),
    },
];
