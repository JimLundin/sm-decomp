# Projectile native state leaves
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate missing D1C7 graphics-clear and DC5A property-assignment instructions.
Expose native A3BE saved-position and GotoWithProbability25 wrappers and verify
existing semantics directly against actual C. Require full-WRAM/return evidence,
tests and scoped cleanup. Offline coverage remains separate from live ownership.

## Completion evidence

Added missing graphics-clear/property-assignment opcode paths and direct native
wrappers for those plus existing saved-position/probability branch semantics.
All 288 actual-C full-WRAM/return cases across all projectile slots pass:
out/port/projectile-native-state-leaves/20261011T100319.338369Z/verdict.json.
Independent test checks exact full-WRAM delta and bank-end pointer preservation
for new leaves. Full port suite, strict Clippy and diff check pass. Ordinary
cleanup zero; dedup reclaimed 66060288 bytes; all cases authenticate afterward.
Free disk 268320493568 bytes. Retained binary/source/pinned ROM dependencies;
no implementer worktree created. Offline scope only; live ownership separate.
Integration branch decomp-port-integration.
