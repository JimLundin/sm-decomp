# Collision callback comparison

`cargo run -p pipeline --locked --offline -- family-verify --adapter callback --manifest tools/port/collision.toml --output out/port/collision`
compares the declared collision routine against the pinned actual C bank tree.
This command requires the collision manifest supplied with the translation; the
harness test uses an explicitly marked infrastructure probe instead.

The contract is the established canonical 128 KiB WRAM boundary, raw 32-bit
movement displacement, return value and synchronous PLM setup behavior. The
manifest uses the family schema (`family`, `rust_module`, `rom`, `routines` and
`cases`). Signatures currently allow up to two `int32`, `uint32`, `uint8` or `uint16`
arguments and `int32`, `uint32`, `uint8`, `uint16` or `void` returns. A case can require
`minimum_callbacks`; captured before/after images use digest-checked family
asset fields. Valid room and enemy domains remain the translation's admission
contract, and Rust runs before the unchecked reference C routine.

The generated Rust static library calls `family::dispatch_host` with an
implementation of `family::Host::spawn_plm`. Its external callback is:

```c
int32_t sm_rust_collision_spawn_plm(
    uint8_t *ram, size_t len, uint16_t header, int32_t *amount);
```

Rust copies canonical C WRAM into owned state before constructing `Wram`. For
each callback it makes a second owned transaction buffer. C copies that buffer
into canonical `g_ram`, installs the displacement pointer, invokes actual
`SpawnPLM`, restores the previous pointer and copies all WRAM back. The Rust
adapter synchronously commits every returned byte. C retains no pointer to Rust
memory. No live mutable slice of canonical C WRAM exists across a callback.

Linker wrapping observes each actual `SpawnPLM` call on both sides. Every callback
retains its header, input/output displacement, return flags and complete WRAM
before and after execution. Final routine WRAM, raw return and ordered callback
records must agree. C gameplay callbacks and CPU fallback calls/instructions
are reported separately; offline verification rejects CPU fallback. The C
callback work remains C-owned even when surrounding collision work is Rust.

`verify_collision_evidence` checks retained artifact identities, current Rust
module dependencies, full WRAM outputs, returns and callback coverage before
another tool consumes the result. Evidence remains local and has no external
signer. Neither an offline pass nor the callback infrastructure probe establishes
live exercise or original-ROM parity.

The seam test links real ROM PLM header `$84:B72B`: actual quicksand setup clamps
`$00010000` movement to `$00003000`, updates canonical WRAM, and agrees across
C and Rust adapters. An incorrect Rust store is rejected with full discrepancy
evidence. Modifying the retained callback sequence is also rejected by the
consumer. The infrastructure probe is explicitly barred from a gameplay
translation claim.
