# Rust bootstrap and hardware checks

Status: resolved
Blocked by: None

Bootstrap must work before the root workspace's patched dependency exists.
Implement pinned download/patch verification in an independent Rust tool;
retain locked dependency versions and isolated hardware test behavior.

## Completion evidence

Final bootstrap/doctor checks and disposable target cleanup are in
[completion.md](../evidence/completion.md). This supersedes pending notes below.

Independent Rust bootstrap replaces source setup, patch preparation, doctor,
symbol generation and hardware checks. Fresh archive extraction and all seven
patches passed; the already-patched checkout authenticated too. Hardware check
passed 36 APU tests, bootstrap tests and strict Clippy passed, and doctor now
passes all tools, ROM, hardware profile, corpus, emulators and three movies.
Rust movie corpus download/import verifies pinned source digests and frame counts.
See `../evidence/sm-rust-bootstrap-doctor.log`. Final build-output cleanup and
commit pending.

## Native TOML migration follow-up

2026-10-11 utilities subagent converted dependency/patch/lsnes/movie pins to
versioned TOML and replaced bootstrap dynamic JSON parsing with typed records.
Missing original patch hashes are omitted optional fields. Bootstrap remains an
independent locked Rust workspace. Tests and strict Clippy passed after migration;
`doctor` authenticated patched hardware, emulators, corpus and all three movies.
Logs: `/tmp/sm-native-bootstrap-doctor.log`,
`/tmp/sm-native-bootstrap-clippy-final.log`. Temporary build-output cleanup is
owned by the orchestrator before resolution.
