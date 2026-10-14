# Freeze the persistence fixture and fresh-process adapter ABI

Type: research
Status: proposed; coordinator review/publication required
Origin: exact research19 claim; see [Answer](../Answer.md) and [children.toml](../children.toml).
No writable ownership, supported native scope or gameplay acceptance is granted.

Use the already-recorded [state contract](../state-contract.toml) to prepare one
reviewed Rust infrastructure task. Generic family currently has no SRAM fixture
input/output and never initializes g_sram. Exact proposed new files:
persistence_adapter.rs, persistence_model.rs, persistence_host.c.in,
persistence_abi.rs.in, tests/persistence_adapter.rs and tests/persistence_live.rs
under crates/pipeline. Shared memory/lib/game_host/family/family_model/live/
live_model/main/batch hooks stay integrator-owned. Preserve legacy87/94/96/97,
native137 and held141; workers must preserve others' changes.

Import/export131072byte WRAM and8192byte cartridge SRAM, raw load return,
ordered buffer/payload/checksum writes and synchronous disk service requests/
results. Preflight full slot/map/pointer/count bounds before effects; refusal
leaves state/services unchanged. Save requests persistence exactly once after
mutations; invalid load requests none. Accepted service failure is owned/fatal
without C retry. Distinguish gameplay memory outcome from recovered C's void
rename/open/write/close durability; declare stronger Rust policy explicitly.

Fresh-process support must import only a saved8192byte file and reject/declare
short or oversized reads under a reviewed policy. Current headless reset0
clears SRAM and never calls RtlReadSram. Publish real isolated-file subject-chain
and original-ROM argv/captures; no invented supported live flags or snapshot
injection. Original direct-page scratch/carry mapping and controller/APU/NMI
domain must be reviewed.

Proposed unrun gates: `cargo test -p pipeline --locked --offline --test persistence_adapter`
(60s); `cargo test -p pipeline --locked --offline --test persistence_adapter --test persistence_live --test family --test live`
(3600s). All new targets are unimplemented. Register supported adapter/manifests
and nonzero controls before native scope admission. This is the minimum ABI
review, with no additional subsystem inventory or runtime experiment requested.
