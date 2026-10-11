# Projectile death beam call
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate CallFunc opcode and its single native death-beam target, complete
C684 initializer and sine/cosine helpers. Preserve two movement steps, RNG,
Mother Brain position state, boundary dust spawn and audio/quake writes.
Require actual-C full-WRAM/return evidence and finished-run cleanup.

## Work evidence

Implemented 8171 function-call dispatch, native SpawnMotherBrainDeathBeam,
complete C684 initializer and signed sine-product helper. All 48 actual-C
full-WRAM spawn cases pass:
out/port/projectile-death-beam/20261011T101318.100020Z/verdict.json.
Covers source slots, full/low/high allocation, interior/boundary coordinates,
two RNG seeds and native dust/audio/quake branch. Full port suite and strict
Clippy pass. Direct CallFunc operand/return oracle remains pending; ticket stays
claimed. No live ownership claim. Successful run cleanup results below.

Ordinary cleanup zero; dedup reclaimed 6291456 bytes; all 48 cases authenticate
afterward. Free disk 268701556736 bytes. Retained binary/source/pinned ROM
dependencies; no implementer worktree created. Branch decomp-port-integration.

## Completion evidence

Direct CallFunc oracle uses all three real ROM operands C79C/C7BD/C7DE and
passes 144 actual-C full-WRAM/pointer-return cases:
out/port/projectile-death-beam-call/20261011T101508.038060Z/verdict.json.
Together with 48 native spawn cases proves complete declared function-call path,
including exact three-byte operand advance, two movement steps and boundary
dust/audio/quake effects. Full port suite and strict Clippy pass. Direct call
cleanup ordinary zero; dedup/authentication results recorded below. Offline
coverage only; live ownership remains separate. No worktree created.

Direct call dedup reclaimed 44040192 bytes; all 144 cases authenticate after
cleanup. Free disk 268505047040 bytes. Retained binary/source/pinned ROM evidence.
