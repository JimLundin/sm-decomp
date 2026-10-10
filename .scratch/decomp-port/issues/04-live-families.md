# Integrate and exercise family replacements
Status: resolved
Blocked by: 01, 02, 03
Triage: ready-for-agent

Build and verify both family contracts, wire measured live replacements into the
existing C host, replay independently and record actual hit coverage, failures
and remaining ownership. Preserve original predecessor sources and evidence.

## Completion evidence

tools/port_live.py verifies source identities and builds definition-only C
renames plus bounded Rust dispatch. Four independent processes (two C, two
mixed) replay 9,000 frames with equal periodic serialized game/hardware state,
terminal WRAM/SRAM, complete decompression capture states and ordered PPU/APU
effects. Evidence: out/port/live/20261010T194953.940696Z/verdict.json.
Mixed processes execute 20,794 Rust family calls across 14 distinct routines,
zero C calls for those replacements and zero measured C-to-CPU fallback.
Other 24 translated symbols are offline-verified, not live-exercised by this
prefix. Rust errors stop the process; no C retry. Missing offline evidence rejects
with retained verdict through the public command test. This run does not claim
original-ROM movie parity, same-process reset or full Rust ownership; original
decompressor/RNG seals remain untouched and reusable.
