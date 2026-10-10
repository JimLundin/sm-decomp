# 01: Capture immutable runs and verified preflight

**What to build:** A repository-root command allocates an immutable evidence run before checking setup, captures the exact maintained candidate, and reports verified inputs or the precise missing setup step.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] Allocate a unique UTC run directory before preflight/build, expose run and brief locations, use a versioned manifest, and never overwrite completed evidence.
- [ ] Capture a restorable maintained source/config snapshot and ordered path/digest inventory including relevant dirty/untracked inputs; exclude ROMs, downloaded trees, generated/build/run output, credentials and unrelated files. Keep ROM-derived material local and ignored.
- [ ] Verify the unheadered Japan/USA Super Metroid ROM, original clean-start any% movie and parsed P1 input digests/counts against the movie container, manifest and compiled table where available; bind seed 0, reset settings and explicit recording profile.
- [ ] Record pinned dependency revision, actual patched hardware source/patches, build features/settings, lockfile/compiler and artifacts as they become available; failed builds never acquire invented binary identities.
- [ ] Run the existing doctor check and log stage commands, stdout/stderr, underlying exit codes/signals and elapsed times; name exact missing user steps after checking workspace facts.
- [ ] Use scoped terminal pass/divergence/uncovered/error/incomplete reports and briefs with exit codes 0/1/2. Caught failure finalizes evidence; absent verdict or uncatchable termination is visibly incomplete. Preflight-only evidence is ineligible for readiness acceptance.
- [ ] Detect source/input mutation during checks and serialize shared generated/build mutations or isolate them, invalidating contaminated runs while preserving existing edits.

