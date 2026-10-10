# 03: Complete synthetic RAM decompression coverage

**What to build:** The offline job correctly executes every RAM decompression command and the important numeric, overlap and bank-boundary cases.

**Blocked by:** 02: Run one literal decompression fixture through C and Rust

Status: open
Triage: ready-for-agent

- [ ] Implement literal, byte fill, word fill, incrementing fill, absolute/relative backreferences and XOR variants with readable Rust control flow.
- [ ] Compare C, Rust and independently specified expected bytes for short and extended lengths, all command modes, termination, odd word fills, incrementing byte wrap and sequential overlapping copies.
- [ ] Cover input rollover from a bank end to the next bank at $8000, including multi-byte header and operand boundaries.
- [ ] Declare malformed-stream handling separately from the valid C-equivalence contract; bounded Rust errors cannot claim parity with unchecked C accesses.
- [ ] Produce exercised branch/case coverage tied to the job manifest. VRAM decompression and MMIO remain outside this routine job.
