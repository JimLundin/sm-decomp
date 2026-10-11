# Enemy death explosion allocation
Status: resolved
Blocked by: None
Triage: ready-for-agent

Port EF89 allocation used by normal enemy death: preserve coordinates, enemy
header and respawn bit, force room graphics and select the five supported
animation scripts. Verify against actual C and retain authenticated evidence.
Drop conversion, respawning and pickup lifecycle remain a separate frontier.

## Completion evidence

EF89 allocation now captures the dying enemy's coordinates, definition header
and slot with the properties respawn bit; forces graphics zero and selects
one of the five EFD5 animation scripts. Table reads validate before allocation.
Independent Rust example uses graphics-aware spawning and checks metadata.

961 actual-C full-WRAM/raw-return cases pass:
`out/port/enemy-death-explosion/20261011T055311.339013Z/verdict.json`.
Rebuild fixtures with `tools/port_enemy_death_fixtures.py`. Cases span all five
scripts, all 32 enemy slots, respawn-property variants and exhausted pools.
The first run rejected the timer/properties alias error; corrected properties
at enemy+0x0e and reran. Diagnostic evidence retained in the preceding run.
Full port tests, formatting and strict Clippy pass after the correction.

Cleanup applied/deduplicated all completed runs in `out/port/enemy-death-explosion`:
zero ordinary bytes and 231,604,224 snapshot bytes reclaimed. Final accepted
evidence authenticated after cleanup. Immutable source/ROM snapshots and
rejection diagnosis retained; no implementer worktree created. 31 GiB available.
Offline coverage; death/drop scripts and live gameplay remain open.
