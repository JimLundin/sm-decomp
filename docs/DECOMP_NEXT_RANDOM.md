Run `cargo run -p pipeline --locked --offline -- decomp-jobs --accepted PATH --output DIR` to execute one
additional dependency-ready job. `--count` defaults to 1; zero, negative values,
counts above the available frontier and unknown `--job` names fail with an error
progress record and no accepted jobs. The bounded job is `next-random`.

The manifest declares the original $80:8111 `NextRandom` contract. It reads and
writes only the unaligned little-endian word at WRAM $05E5, returns the new word,
has no callees or ROM assets and performs no hardware access in the C/Rust seam.
The original assembly uses $4202/$4203 multiplication: the low product is pushed,
the high product is byte-swapped and added with carry, then `ADC #$0011` retains
the final carry before `STA $05E5`. The original 53-byte extent is $80:8111–$80:8145; SHA-256
`7ff43231d5ad3af98d7133d8830c28afb00ccec34a326b443cfb565a934d2258`
is recorded as contextual assembly identity (not a native execution verdict). Its multiplication
uses byte truncation and retains the carry from the high-byte sum. Independently
worked examples include the high-byte overflow at seeds $FFFF and $3300. CPU
registers, flags, instruction timing and scratch state are outside this C/Rust
pure-function contract. The source is verified byte for byte against the pinned
MIT upstream archive before building; this is a pinned-source freshness check,
not a claim about newer unreviewed upstream work.

The job builds the actual unchanged whole C bank with dead section elimination,
and a disjoint standalone Rust candidate. This keeps the accepted decompressor's
implementation seal intact. Exhaustive C/Rust calls cover every 16-bit seed and
check the returned word, the stored word and all surrounding WRAM bytes. The
independent literal examples must also match. ABI controls check null pointers,
invalid extent and valid allocated state. Allocation validity and exclusive
synchronous access are the caller's contract; the function retains no pointers.

Live integration performs one reviewed source transformation: only the pinned
C function definition is renamed to `C_NextRandom`, with an exact replacement
count of one and retained original/transformed bank digests. Every original
same-bank and cross-unit call consequently resolves to the external host
`NextRandom` entry. C subjects call `C_NextRandom`; mixed subjects call Rust.
No inputs or gameplay state are injected to exercise this routine.

Four independent processes replay the frozen 9000-frame prefix twice after real
reset. Every checkpoint, decompression fixture and ordered hardware effect must
match. First 16 actual RNG calls retain complete before/after WRAM and are replayed
through separate C and Rust processes offline. Every actual RNG entry retains its
seed, result and semantic low-byte/high-byte write effects; every observed result
is checked against the exhaustive C oracle table. The byte order expresses the
one little-endian word store's semantic contract, not the native compiler's store
instruction schedule. No machine-level transient-store claim is made.

Progress retains the exercised accepted routine and measured C-to-CPU fallback;
remaining untranslated C routines are explicitly uncensused. The predecessor
acceptance pointer is verified before and after execution and is never rewritten.
An error run retains a failing verdict and cannot acquire predecessor acceptance.
This serial job owns its Rust source, host adapter and evidence directory; no
concurrent translation or shared-build mutation is performed. Original-ROM
movie parity and full Rust ownership remain unproven.

The retained ticket08 accepted run checks all 65,536 seeds, 16 complete captured
calls and 8,783 actual calls per replay epoch. Each mixed epoch has 8,783 Rust RNG
hits and zero C RNG hits; the decompressor retains 23 Rust hits and zero C hits.
Measured gameplay CPU fallback is zero in this prefix. The error evidence for an
incorrect initial hand-worked expectation remains retained; diagnosis corrected
the final carry at $FFFF (269) and $3300 (18) before acceptance.
