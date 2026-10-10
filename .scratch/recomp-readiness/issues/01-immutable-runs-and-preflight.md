# 01: Capture immutable runs and verified preflight

**What to build:** A repository-root command allocates an immutable evidence run before checking setup, captures the exact maintained candidate, and reports verified inputs or the precise missing setup step.

**Blocked by:** None (can start immediately).

**Status:** done

- [x] Allocate a unique UTC run directory before preflight/build, expose run and brief locations, use a versioned manifest, and never overwrite completed evidence.
- [x] Capture a restorable maintained source/config snapshot and ordered path/digest inventory including relevant dirty/untracked inputs; exclude ROMs, downloaded trees, generated/build/run output, credentials and unrelated files. Keep ROM-derived material local and ignored.
- [x] Verify the unheadered Japan/USA Super Metroid ROM, original clean-start any% movie and parsed P1 input digests/counts against the movie container, manifest and compiled table where available; bind seed 0, reset settings and explicit recording profile.
- [x] Record pinned dependency revision, actual patched hardware source/patches, build features/settings, lockfile/compiler and artifacts as they become available; failed builds never acquire invented binary identities.
- [x] Run the existing doctor check and log stage commands, stdout/stderr, underlying exit codes/signals and elapsed times; name exact missing user steps after checking workspace facts.
- [x] Use scoped terminal pass/divergence/uncovered/error/incomplete reports and briefs with exit codes 0/1/2. Caught failure finalizes evidence; absent verdict or uncatchable termination is visibly incomplete. Preflight-only evidence is ineligible for readiness acceptance.
- [x] Detect source/input mutation during checks and serialize shared generated/build mutations or isolate them, invalidating contaminated runs while preserving existing edits.


## Completion evidence

- Public command and evidence seam: eight behavior tests cover immutable allocation, dirty/untracked restoration, setup failure, subprocess failures/timeouts/signals, mutation invalidation, movie/input/table identity, stale verdict rejection, and artifact preservation. All 12 repository Python tests pass.
- Real `just readiness-preflight` passed in the isolated ticket worktree: run `20261010T074743.268509Z-23d4a0078434`, with verified 129,712-frame movie/P1 input identities and archive-verified patched hardware source. Verdict is explicitly preflight-only and ineligible for readiness acceptance. An earlier run retained the precise missing symbol step as an error.
- Shared lifecycle and restoration contract documented in Readiness evidence documentation; no accepted pointer or generated/shared dependency source was changed. Later replay builds must bind their own generated/binary identities and isolate or lock shared mutations.
