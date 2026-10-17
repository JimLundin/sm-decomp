# Ordinary Rio actual-C comparison

Standalone Rust verifier for ticket 29. It links `sm-rio` with fixture support
from the integrated main-checkout candidate. The integrator owns all producers.
No Python, shared workspace edits or native runtime adapter are required.

From the main checkout, prepare the standalone lockfile, then run serially:

```powershell
cargo generate-lockfile --offline --manifest-path restart/rio-verify/Cargo.toml
cargo run --locked --offline -j 1 --manifest-path restart/rio-verify/Cargo.toml -- C:/Users/Jim.Lundin/Documents/Projects/sm-decomp C:/Users/Jim.Lundin/Documents/Projects/sm-decomp/rom/sm.sfc C:/Users/Jim.Lundin/Documents/Projects/sm-decomp/restart/private/toolchain/w64devkit/bin/gcc.exe C:/Users/Jim.Lundin/Documents/Projects/sm-decomp/restart/out/rio-actual-c-20261016
```

The output directory must be new and its parent must exist. Retries use another
new output directory and preserve failed diagnostics. The pinned pristine ROM
is read privately and not copied into the run. Source SHA-256 seals authenticate
the relevant files restored from revision
`578f90b3cc49557bb70060ad033bb90b8cf8ac50` and its coordinator-authenticated archive
`69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193`.

The driver extracts the ten owned Rio definitions and exact NextRandom,
QueueSfx2_Max6/Internal, distance/screen helpers and 320-word signed sine table.
Original headers provide the structs, offsets, macros and address constants.
GCC receives `COMPILER_TCC=1` to select portable assert.h in upstream types.h;
this does not replace any gameplay macro or body. Original signed shift macros
are retained with GCC `-O0 -fwrapv -fno-strict-aliasing`; this is a comparison
against this specific compiled recovered C implementation.

Every shared fixture compares the entire 128-KiB canonical WRAM, typed result,
ordered horizontal/vertical/angle requests including index and signed amount,
and exact script consumption. Services apply declared byte patches synchronously
before returning; they implement no actual terrain or angle algorithm. All ten
entries must occur. A failure produces no passing verdict.

Inputs, original headers, extracted reference source, compiled Rust source
snapshots, both protocol outputs, stdout/stderr, hashes and the final TOML
verdict are retained. Runtime rejects subject edits after verifier compilation.
Each producer has a 60-second deadline and a 512-KiB retained output cap per
stream. Log readers close their pipes on excess output. The child status is
monitored every 20 ms. GCC helper-process termination remains host-dependent;
the integrator must inspect descendants before run cleanup after a timeout.
The subject manifest and module paths are retained as a buildable source layout.
The driver embeds its lockfile and rejects changes after compilation.
The compiled verifier's own hash is recorded; preserve its executable separately
if future replay requires it. The lockfile should be retained after generation.

Passing this verifier proves only the declared actor-body fixture domain.
Natural live encounters, terrain parity, production admission/fatal controls,
composition and original-ROM parity remain separate gates. Run cleanup belongs
to the integrator after producers finish; retain the source/protocol evidence
and verdict, and delete only identified build intermediates.
