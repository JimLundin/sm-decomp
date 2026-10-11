# Native enemy-drop probability selection
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate RandomDropRoutine (86:F106), preserving unsigned byte probability
arithmetic, inventory eligibility, health-bias hysteresis and RNG retry ordering.
Use pinned bank A2 enemy definition and B4 drop probability tables; do not invent
WRAM aliases for host spawn coordinates. Verify full WRAM and raw return against
actual C, including all RNG output bytes, thresholds and empty definitions.

## Verification required

Actual-C oracle, meaningful independent Rust examples, required crate checks,
finished-run cleanup and authenticated retained evidence. Offline scope only.

## Implementation evidence

Native RandomDropRoutine preserves six returned pickup codes, uint8 weight sums
and rare-ammo reservation, cumulative inclusive RNG boundaries, health-bias
hysteresis (30/50 thresholds), and retry when RNG's low byte is zero.
The misleading get_EnemyDef_A2 helper addresses bank A0; definition offset 0x3a
uses file 0x100000+(header&0x7fff)+0x3a. Drop chances use bank B4.
Independent Rust examples verify forced health, hysteresis, inventory exclusion,
rare ammo selection and no-header RNG preservation. Root full tests/fmt/strict
Clippy passed settled source. Offline verification does not establish live use.

## Completion evidence

Final actual-C comparison: 1,794 cases pass full WRAM and raw return comparison,
`out/port/enemy-drop-selection/20261011T060314.778547Z/verdict.json`.
Includes six actual-ROM headers, unsigned health overflow, inventory masks,
hysteresis/byte-alias cases and all 257 RNG seeds producing a zero first byte.
Initial 1,537-case run retained at `20261011T060223.470847Z`.
Both verdicts authenticated after cleanup with verify_plm_instruction_evidence.

Cleanup applied and deduplicated both finished run directories using
`python3 tools/port_cleanup.py --root <run> --apply` and `--deduplicate --apply`.
Unsealed reclamation zero bytes; snapshot reclamation 235,012,096 and
268,828,672 bytes, total 503,840,768 bytes. Remaining disk space 28 GiB.
No implementer worktree or disposable custom build remains. Retained snapshots
and sealed binaries stay authenticated. Root full port tests/fmt/strict Clippy
passed; two new independent Rust tests also pass. Live ownership remains open.
