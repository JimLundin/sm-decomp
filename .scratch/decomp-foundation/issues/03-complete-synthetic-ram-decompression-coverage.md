# 03: Complete synthetic RAM decompression coverage

**What to build:** The offline job correctly executes every RAM decompression command and the important numeric, overlap and bank-boundary cases.

**Blocked by:** 02: Run one literal decompression fixture through C and Rust

Status: resolved
Triage: ready-for-agent

- [x] Implement literal, byte fill, word fill, incrementing fill, absolute/relative backreferences and XOR variants with readable Rust control flow.
- [x] Compare C, Rust and independently specified expected bytes for short and extended lengths, all command modes, termination, odd word fills, incrementing byte wrap and sequential overlapping copies.
- [x] Cover input rollover from a bank end to the next bank at $8000, including multi-byte header and operand boundaries.
- [x] Declare malformed-stream handling separately from the valid C-equivalence contract; bounded Rust errors cannot claim parity with unchecked C accesses.
- [x] Produce exercised branch/case coverage tied to the job manifest. VRAM decompression and MMIO remain outside this routine job.

## Completion evidence

Implemented all eight RAM modes at the safe decompressor API, including sequential
absolute/relative XOR overlap, initialized WRAM before the destination, odd word
fill, wrapping increment fill and source bank rollover. The job manifest contains
40 independently specified expected fixtures and 38 observed semantic/branch cases.
Every fixture matches whole 128 KiB WRAM from unchanged pinned C, Rust and expected
bytes. Relative XOR is extended-only and its maximum valid length is 768 because
FF terminates before extended decode; other extended modes reach 1024.

Malformed bounds/truncation handling remains separate: checked Rust errors and
pre-C fixture admission rejection establish no malformed C parity. The verdict
reports `synthetic_verified: true`, `status: partial`, `routine_accepted: false`.

Verification: `cargo test -p decomp --locked --offline` (5 API tests),
`python3 -m unittest tools.test_decomp_routine` (3 real-CLI tests), and both direct
relative-path CLI and `just decomp-routine` invocations passed. Retained evidence:
`/tmp/sm-decomp-ticket03-evidence/20261010T182355.172355Z/verdict.json` and
`/tmp/sm-decomp-ticket03-just-evidence/20261010T182356.451427Z/verdict.json`.
Implementation commit: ticket03 branch commit titled "Complete synthetic RAM decompression coverage".
