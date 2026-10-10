# 02: Run one literal decompression fixture through C and Rust

**What to build:** One public offline job command compiles the actual pinned C routine and a readable Rust replacement, executes a synthetic literal stream from identical initial state and compares declared outputs.

**Blocked by:** 01: Retain the executable C baseline

Status: open
Triage: ready-for-agent

- [ ] Declare the routine address, source revision and notices, dependencies, memory reads/writes, ROM mapping, numeric behavior, fixture scope and exclusions in a small versioned job manifest.
- [ ] Use bounded byte-addressed WRAM views, explicit little-endian access and validated LoROM reads, preserving aliasing and source rollover semantics without broad state restructuring.
- [ ] Compile the selected upstream C routine with the minimal necessary shims instead of retyping a second reference algorithm.
- [ ] Execute literal and terminator cases with independently specified expected output and unchanged surrounding memory; retain C/Rust results and machine-readable failures.
- [ ] Expose a deterministic prepare/build/compare/verdict command; incomplete algorithm coverage is labeled partial and cannot accept the whole routine.
