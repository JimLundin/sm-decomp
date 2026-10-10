# 07: Verify an update-aligned original-ROM replay prefix

**What to build:** Capture and replay a bounded any% prefix using actual native gameplay/input boundaries, preserving a source-frame map and reporting the first semantic divergence.

**Blocked by:** 01: Retain the executable C baseline

Status: open
Triage: ready-for-agent

- [ ] Instrument the unmodified original ROM at completed gameplay boundaries and actual controller reads, including applicable boot, NMI continuation, message-box and upload-wait boundaries.
- [ ] Derive a replay input timeline with source-frame mapping and explicit retained/skipped/retimed classifications; preserve held inputs, press/release edges and deliberate waits.
- [ ] Run the unchanged C candidate from one initial state on derived inputs, comparing declared semantic state and relevant effects without corrective state injection.
- [ ] Preserve raw mismatches and narrowly justified comparison exclusions; include a changed-gameplay control showing lag compensation cannot conceal behavioral errors.
- [ ] Validate an agreed bounded prefix, or retain the first concrete unsupported boundary/divergence as a nonaccepting result with reproducible diagnostics; failure does not claim adapted completion.
