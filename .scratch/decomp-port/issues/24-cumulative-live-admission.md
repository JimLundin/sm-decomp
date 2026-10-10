# Admit cumulative gameplay families in live replay
Status: resolved
Blocked by: 16, 17, 18, 19, 20, 21
Triage: ready-for-agent

Run authenticated cumulative gameplay offline verification and independent
C/mixed live replay using tools/port_batch.py. Diagnose actual failures and keep
rejected evidence. Resolve ticket12 only with exercised collision/movement and
matching state, returns and effects. Record per-family hits and remaining C/CPU
ownership; zero-call families remain offline. Investigate PLM execution before
requiring its admission. No state repair or hidden C retry is acceptable.

Extend replay prefix where practical to expose substantive gameplay dependencies.


## Storage cleanup evidence

Reusable `tools/port_cleanup.py` previews by default and supports explicit
application. New live runs delete unsealed Cargo/archive intermediates; completed
batches deduplicate sealed immutable snapshots and replay effect logs by default.
Historical source seals remain unchanged. CLI tests cover preview/apply behavior,
sealed builds, unfinished runs, symlink/path escape, byte/hash preservation and
restart after interrupted temporary links.

Scoped accepted flight-fixed-live cleanup reclaimed1,317,372,945 bytes by
deduplicating identical authenticated observations (run20261010T204643.443282Z);
root additionally reclaimed355,637,476 bytes of unsealed historical build
intermediates. A broad snapshot-only cleanup was interrupted after preserving
already completed atomic deduplications; its unmeasured savings are not added
to these totals. No accepted verdict, pointer, hash or observation was rewritten.


The finished cumulative batch's sealed snapshot deduplication additionally
reclaimed11,747,590,144 bytes. Combined with root's measured historical cleanup
of9,417,670,502 bytes, measured total cleanup is21,165,260,646 bytes (19.71GiB).
Interrupted broad-scan savings are excluded.


## Completion evidence

Authenticated cumulative15-family verification and live replay passed in
`out/port/batches/20261010T213631.753519Z`, final verdict
`replays/20261010T215856.742725Z/evidence/20261010T215856.841221Z/verdict.json`.
Four fresh independent18,000-frame C/mixed processes match full periodic
hardware/game state, terminal WRAM/SRAM, per-call captured states/raw returns
and ordered PPU/APU effects without state repair or hidden retry. Each mixed
process executes235,952 Rust gameplay calls across76 symbols,37 accepted Rust
decompressions and17,594 Rust RNG calls. Measured CPU fallback is zero; C still
owns frame scheduling and untranslated gameplay. Final frame is game-state8,
room0xdf8d, Samus111/187; adapted movie completion/original-ROM parity unproven.

| Family | Rust calls | Exercised symbols |
| --- | ---: | ---: |
| system |46707|7|
| audio |538|9|
| Samus physics |61227|10|
| enemy mechanics |28267|2|
| input/control |28655|16|
| instructions |9241|1|
| projectile core |1104|2|
| room/scroll |47229|20|
| collision/movement |12021|5|
| projectile flight |963|4|

Enemy AI/Rio/jump, jump instructions and special projectiles have zero calls and
remain offline verified. New crawlers and power-bomb lifecycle remain offline;
they were not installed in this accepted15-family replay. Terrain SpawnPLM is
unexercised (zero callbacks); room/hardcoded allocation is a separate C entry
point, so zero terrain calls do not mean no PLM gameplay. Native PLM setup and
bounded PLM instruction engine keep their offline status.

The initial batch retained a rejected changed-source collision seal after module
registration advanced during verification; all1,752 comparisons matched but
authentication correctly rejected the changed lib.rs dependency. The final batch
rebuilt every stale source seal before replay.


Source-snapshot scope: the live/static libraries include the earlier uninstalled
power-bomb module as an exhaustive shared-crate dependency. Its subsequent
argument-guard review correction is verified separately by ticket23; power-bomb
was never installed or exercised in this replay. The retained source snapshot
keeps the exact earlier module bytes so its exhaustive dependency seal remains
authenticatable. All15 installed gameplay family bytes match final integration.
