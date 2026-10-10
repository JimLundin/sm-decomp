# Flying enemy AI and shared enemy terrain movement
Status: resolved
Blocked by: 06, 11
Triage: ready-for-agent

Translate pinned A2 Rio five-state swoop/chase/return, Ripper and jet Ripper patrol/reversal, and A3 Waver oscillation/animation. Own A0 movement dependency with explicit PLM host, complete canonical WRAM and actual C ordered callback comparison. Preserve integer and slope quirks. Unknown state/table and cyclic terrain fail admission. Full bank-specific enemy instruction engine and projectile/death AI remain separate tasks.

## Completion evidence
Implemented `enemy_ai.rs`: Rio five-state swoop/chase/return, Waver movement/animation, jet and normal Ripper patrol/reversal, and actual A0 enemy horizontal/vertical movement. Canonical enemy records begin at 0xf78; signed/fractional arithmetic and instruction timers retained. Square/non-square slopes, all 16 terrain classes and bounded finite horizontal/vertical extension chains supported. Spike headers use an explicit Host; commercial ROM contains 15 zero header entries, so vanilla admitted cases spawn no PLM. BTS outside those 15 entries rejects rather than interpreting following machine code as a header. Linear speed admission is conservatively 64 rows. Unknown AI state, out-of-range records/tables, cyclic/out-of-allocation extension chains fail admission.

Actual C collision harness, unchanged entire pinned bank closure, final stable source:
`/tmp/enemy-ai-final/20261010T202927.658122Z/verdict.json`: 17 routines / 454 cases, full 128 KiB WRAM and return equal, all16 tile classes and square slope flips, active Rio movement/turn/hover/chase transitions, oscillation and patrol collisions; zero CPU fallback. Offline only, live integration pending.

Separate synthetic data experiment `/tmp/enemy-ai-host-verify/20261010T202929.147413Z/verdict.json`: 2 horizontal/vertical spike cases each execute actual C SpawnPLM/header/setup and match ordered callback state and argument records. ROM differs only in the first spike header at file offset 0x1042da, from zero to actual vanilla header 0xb797 (empty setup 0xab27); free PLM slots. This proves the host boundary, not commercial-ROM callback coverage. Manifest `/tmp/enemy-ai-host.json`, patched ROM `/tmp/enemy-ai-host.sfc` retained locally. No synthetic commercial ROM is committed.

Four independent Rust tests passed: fractional movement and wall snap; Rio velocity preservation entering hover; Waver's nonstandard SignExtend8 initialization; cyclic terrain and unknown state rejection. `rustfmt --check` passed. Isolated worktree lacks ignored RustySNES dependency, so tests were compiled with `rustc --crate-type rlib` for the real port library and `rustc --test` for the integration test; root Cargo integration required after registration.

Differential diagnosis retained first failure `/tmp/enemy-ai-verify/20261010T202709.656865Z/verdict.json`: reconstructed C sine/cosine table is an explicit C constant, while its 94A957 comment points to unrelated original ROM code. Translation now retains the actual recovered constant table to match the selected source. This remains distinct from original-ROM parity.

Commit: this ticket's implementation commit (see git history).
