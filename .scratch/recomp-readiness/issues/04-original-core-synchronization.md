# 04: Verify original-core provenance and strict synchronization

**What to build:** A command verifies or recaptures the original oracle and compares a fresh current reference replay against it without tolerance or resynchronization.

**Blocked by:** 01: Capture immutable runs and verified preflight.

**Status:** done

- [x] Verify original CSV digest, completed 129712-frame scope, ROM/movie/core identity and actual-fetch observer semantics; bind missing legacy provenance with an independently verified immutable attestation or recapture, never guessed metadata.
- [x] Produce a fresh current candidate replay bound to hardware source/settings and all applicable input/reset/profile identities.
- [x] Compare actual opcode-fetch checkpoints at $82:8948 using the ten initialized semantic words and original labels 272–18000 inclusively, the documented label convention and zero additional offset.
- [x] Preserve ordered multiple visits per frame and reject empty intervals, missing/extra/reordered events, incompatible identities, truncated captures or completion metadata with a mismatched digest.
- [x] Write uniform synchronization outcomes, expected/completed scope, timing and the first independent disagreement; historical passes do not certify the current candidate.
- [x] Keep identity-linked reproductions of event 15292 at original frame 18195 versus 18196 and the later frame-35203 X=44 versus 43 mismatch as debt outside the initial domain.



Evidence: `just readiness-sync` completed a fresh clean original build, 18,000-frame
original replay and 18,000-frame current reference replay with byte-identical
checkpoints: 15,184 ordered events, labels 272–18,000, zero offset. CSV SHA-256:
`445d190a5f9c7b0ea62f5dd37291c0497ca7b4c29bcf37e4294e5e1782f270b7`.
Build/replay timings: original 523.83s/136.17s, reference 24.53s/266.6s.

Verified original-only attestation retained in the main checkout at run
`20261010T082358.413496Z-53c5f3809a64`. It independently rechecks official origin,
declared build edits, source inventory, build/input dependencies, observer and
completion command/report identities. It certifies the original oracle only.
The final integration readiness run still requires a fresh current candidate.
Legacy full capture remains ineligible for oracle reuse without its missing
provenance; available historical captures/reports are retained with hashes as debt.

Validation: twelve synchronization behavior tests, eight evidence lifecycle
tests, Rust formatting and diff checks pass. Tests cover strict visit/value
comparison and missing/truncated/incompatible/forged source/build/observer proof.
