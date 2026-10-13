# Corrected progress reporting implementation

Status: reporting code committed; publication and audit cleanup complete
Blocked by: None
Owner: recovery coordinator
Commit: `66cde48`

The Rust registry now reports evidence context independently for every stage.
Legacy observations retain their original serialization and unspecified
context. New `supersedes` observations correct prior claims without mutating
their evidence or history. Targets must exist at the same stage and C inventory;
duplicates, self-reference and cycles reject. Superseded observations stay
excluded if replacement evidence later becomes stale, avoiding false revival.
The viewer excludes both stale and superseded observations and displays the
replacement reason alongside the original evidence.

The final observed coverage is 1,460 translated, 924 verified, 38 integrated and
121 live-exercised definitions from the 6,053-definition C inventory. Context
counts remain distinct and may overlap; checked source correspondence includes
the reviewed eight inlined Ridley helpers. The old whole-reachability claim is
superseded by a reviewed caller-domain observation. Eleven internal native
opcodes and successful initializers have measured live counts inside the
accepted composed interpreter/allocator; they are not new direct wrapper roots.

Validation is retained in `progress-final-tests.log` (19 tests),
`progress-final-build.stderr`, and `progress-accuracy-review.md` (format,
Clippy with four unchanged baseline exemptions, source/evidence review).
`browser-correction-result/checks.json` proves permanent exclusion and visible
history after a stale replacement. Desktop/phone screenshots and inspection
records prove rendering with no phone horizontal overflow.

Both scoped project cleanup passes on `browser-correction-result` succeeded,
reclaiming zero utility bytes. After the browser exited, the identified
4,131,090-byte Chromium profile was removed; all six screenshots/checks/log
artifacts authenticate against `browser-correction-evidence.sha256` after
cleanup. Test scratch directories removed themselves. Shared Cargo outputs
remain for the progress server and coordinator; no gameplay run or worktree
was created. Final free space at this cleanup was 231,313,166,336 bytes.

The prior browser probes were separately cleaned by the read-only recovery
owner: 21 evidence files retained and authenticated, 34,086,872 apparent bytes
reclaimed. See `recovery-browser-cleanup.md`. Source/ticket audit cleanup and
whole Ridley diagnostic/family cleanup remain independently recorded; sealed
gameplay evidence, shared store, source contexts and readers were preserved.

The older viewer task in thread `01a13394-29b8-76a3-aab7-5ab8eff60694` remains
stalled on its preview approval. The maintained layout and three-layout
comparison were completed through `../decomp-progress/coverage-finish-handoff.md`;
progress ticket 02 is resolved. The comparison is retained at `e574fef4` on
`throwaway/progress-layout-comparison-20261013`. Final registry has 256
observations, 16 stale exclusions and one superseded observation.
