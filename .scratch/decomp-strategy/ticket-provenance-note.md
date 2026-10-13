# All-ticket provenance audit

Date: 2026-10-13. Status: complete, read-only handoff to coverage reconciliation.
This audit read all 141 local decomp-port tickets and their spec, then joined
retained receipts, maintained Rust correspondence and native fixture dispatch.
It launched no gameplay verification or translation runs and published no stages.

The ticket snapshot contains 135 resolved tickets and six claimed tickets
(78,87,94,96,97,141). Partial implemented coverage remains included. The cumulative
ticket candidate union contains 1,391 native implementation correspondences,
846 native historical equivalence candidates backed by the representative
118-passing-run artifact audit or explicit post-cleanup family receipts, and
105 direct or explicitly bounded native live roots. These are provenance
candidates with per-routine limits, not the final shared registry totals.

| Artifact | Content | SHA256 |
| --- | --- | --- |
| ticket-provenance.json | All 141 records, exact symbols, commits, scope/cleanup/capture pointers, source identity and stage limits | c726cd86bb0211d83b4f6888291b379874ac7900ee3fceb44e452d69bd00c9b2 |
| ticket-authenticated-internal-mappings.json | 128 exact manifest/native mappings covering 93 native symbols; frozen for publication | 6737fa270cea550bf64ad30e129bebe0fb61b62c0c6595a5173406111189f12a |
| ticket-live-provenance.json | 14 selected retained live verdicts cover 105 direct/bounded roots, with positive native Rust counts | dba6def1edf442bb5e59a9dcbaff0509b95bfaef0a7e8ebd3fca17584bce68b2 |
| ridley-inlined-source-alias-review.md | Exact source correspondence for eight Ridley helpers inlined in callers; translated domain only | 4bb1eeca281f3bea3daec13c4dc15d271a15f640367788e6b6ef81cdb5e9e34b |

All artifacts are in `.scratch/decomp-strategy/`. The native source revision is
`578f90b3cc49557bb70060ad033bb90b8cf8ac50`, native inventory SHA256
`7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25`.
The pinned ROM SHA256 is
`12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`.

## Interpretation

`routines` records cumulative implemented lineage at the ticket's documented
domain. Early module tickets may include subsequent additions to that module;
commits and source correspondence remain separate from verification.
`verified_routines` equals `authenticated_verified_routines` and describes
retained historical fixture equivalence, not automatic current-checkout
dependency authentication. `historical_equivalence_claim_routines` preserves
passing fixture claims whose receipt was not among the authenticated selection.
Ticket-level `supported_stages` means at least one routine supports that stage.
Use the per-routine arrays and evidence to publish.

`rust_identities` separates sealed module identity, retained snapshot identity
and current module comparison. Matching one module does not authenticate all
shared dependencies or harnesses. There are 417 identity records and 78 missing
path references, mostly historical `/tmp` scopes. Missing early paths were
reconciled through later retained cumulative families where possible; they
remain visible and must not be represented as retained evidence themselves.

Synthetic `EprojInstruction` and `PlmInstruction` fixtures pass a script pointer.
Their retained fixture ROM word selects the native opcode; the pointer is not
the native routine address. `EprojPreinstruction` and `EprojInitializer` carry
direct callback addresses. Spawn fixtures read the retained definition's first
word; `SpawnEprojWithGfx` packs the definition into `b >> 16`. Four rendering
adapters map through the actual C shim to their native rendering definitions.
Function defines in `funcs.h` resolve table-adjacent address-comment ambiguities
only when the exact symbol is an actual lexical definition.

The audit excludes declarations, synthetic wrappers and proved empty source
bodies. Seven retained unmapped fixture references are classified explicitly:
four dispatcher-only no-op references and three saturated allocator fixtures
whose malformed definition is never read. None grants implementation credit.
The representative authentication audit excludes four older PLM receipts with
missing module pins; later source/fixture lineage remains separately recorded.

## Live and internal boundaries

The live selection keeps exact positive wrapper counts. Zero direct contact-hit
wrappers in ticket136, zero sprite allocation in ticket94, and zero paired
movement helpers in ticket87 do not receive direct live credit. Animated tile
handler execution with no active IDs establishes only the observed handler
domain. Palette object processing is composed inside the Rust handler; its
zero separate wrapper count does not negate the translated implementation.

The current composed receipt records EprojRunAll95,325, RoomGfx spawn16,
WithGfx spawn8, MainScrollingRoutine94,425 and bounded attack interpreter2,393
Rust calls per mixed subject. The interpreter leaves209,673 calls in C/refusal
ownership. Its eight measured opcodes are recorded as composed internal
observations, while native initializer allocations16/4/4 are separate composed
observations. The trail receipt records3,824 owning frames,167active; three
movement opcodes have no natural hits. These receipts are frozen source and
observation claims with post-cleanup authentication.

Ticket141's220-entry native list establishes reachability. Owned source and
reviewed inlined correspondence establish translated lineage; only33declared
controlled roots have direct6457-pair retained authentication. The eight
inlined helper mappings in the dedicated review have bounded caller domains
and no direct fixture or live counts. Formal whole Ridley live acceptance and
original-ROM parity remain open. Source closure alone never grants those stages.

## Completion and cleanup

The Rust audit helpers, Cargo manifest/lock and logs are retained locally for
review. Temporary audit processes finished or were stopped before cleanup.
`CARGO_TARGET_DIR=/tmp/sm-ticket-provenance-target cargo clean --manifest-path
/tmp/sm-ticket-provenance-tool/Cargo.toml` removed246files/101.4MiB. Measured
logical target bytes before cleanup were87,567,241; the disposable14,821-byte
Cargo wrapper and temporary source copies were then removed using scoped file
deletion. No Git worktree or gameplay evidence directory was created. No sealed
evidence, shared store, ROM, retained candidate or acceptance pointer was moved
or removed. Remaining disk space was 231,383,646,208 bytes. The final JSON parses,
all 141 ticket IDs are present, and retained artifact hashes above were recorded
after cleanup.
