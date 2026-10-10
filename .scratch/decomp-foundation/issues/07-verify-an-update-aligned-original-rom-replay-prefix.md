# 07: Verify an update-aligned original-ROM replay prefix

**What to build:** Capture and replay a bounded any% prefix using actual native gameplay/input boundaries, preserving a source-frame map and reporting the first semantic divergence.

**Blocked by:** 01: Retain the executable C baseline

Status: resolved
Triage: ready-for-agent

- [x] Instrument the unmodified original ROM at completed gameplay boundaries and actual controller reads, including applicable boot, NMI continuation, message-box and upload-wait boundaries.
- [x] Derive a replay input timeline with source-frame mapping and explicit retained/skipped/retimed classifications; preserve held inputs, press/release edges and deliberate waits.
- [x] Run the unchanged C candidate from one initial state on derived inputs, comparing declared semantic state and relevant effects without corrective state injection.
- [x] Preserve raw mismatches and narrowly justified comparison exclusions; include a changed-gameplay control showing lag compensation cannot conceal behavioral errors.
- [x] Validate an agreed bounded prefix, or retain the first concrete unsupported boundary/divergence as a nonaccepting result with reproducible diagnostics; failure does not claim adapted completion.

## Completion evidence

Implemented the bounded original-ROM capture/map/independent C replay command
and read-only C observer. See `docs/DECOMP_PREFIX.md` for semantics, exclusions
and reproducible invocation. Retained durable evidence:
`out/decomp/ticket07/20261010T182149.168780Z/verdict.json`.

320 native frames completed; 160 actual register-paired input polls were
retained and replayed through one fresh unchanged C subject. Both originals
and C reported 10 completed dispatches. First unsupported mapping boundary:
source frame 19 prelude input return (event 23). First diagnostic semantic
mismatch: original frame 272/event 1025 versus C step 151/event 303 has
held/pressed/previous 0 versus $0080. Full native WRAM, source event map, C
WRAM/SRAM and raw effects remain retained. The fixed-map separate changed-RNG
process differs from unchanged C at step 151: RNG 758 versus 759, independently
of the original mapping rejection.

Verification: `python3 -m unittest tools/test_decomp_prefix.py` passes both
public command tests; `git diff --check` passes. The result is explicitly
nonaccepting (CLI exit 2); it resolves the bounded investigation, not original
parity or adapted movie completion.
