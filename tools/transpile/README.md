# How `sm/src` was produced

This is a record, not a step to repeat. The C source is gone from this project
and `sm/src` is maintained by hand from here on.

1. snesrev/sm at `578f90b3cc49557bb70060ad033bb90b8cf8ac50` (2023-04-17), built
   on Ubuntu 24.04 with clang 18 and SDL2 2.30 under `bear` to record
   `compile_commands.json`:
   `CFLAGS="-O2 -fno-strict-aliasing" bear -- make CC=clang`
   (upstream's default `-Werror` trips on clang 18 pointer-sign warnings).
2. `c2rust transpile compile_commands.json --binary main -o out-rs` with
   c2rust 0.22.1 (`cargo install --locked c2rust`, LLVM 18). All 52 C files,
   about 279k lines of Rust.
3. The one-time `fixup.py` script fixed the 19 compile errors, all mechanical:
   - `VaListImpl`/`as_va_list()` (removed from nightly) → `VaList` (1 site)
   - `(int expression) as bool` → `!= 0` (3 sites)
   - `let ref mut` borrows of fields in `#[repr(packed)]` structs → raw
     pointers with `read_unaligned`/`write_unaligned` (15 sites)
4. Files moved into `sm/src/{game,snes,platform}`, `sm_cpu_infra.rs` →
   `cpu_infra.rs`, `sm_rtl.rs` → `rtl.rs`, and `cargo fmt` applied.
5. `sm/src/ram_map.rs` was generated from `variables.h` and
   `variables_extra.h`, because c2rust expands those macros into raw offsets.

The retired fixup script is preserved in Git history at
`1311d3b4:tools/transpile/fixup.py`. It depended on the original output's paths
and line numbers and has no role in building or maintaining the current Rust.

Check performed before deleting the C: on the same synthetic ROM, the C binary
and the Rust binary produce byte-identical output. That only exercises boot
and ROM patching. The real check is the per-frame comparison against the
original ROM ([movies/README.md](../../movies/README.md)), which needs a real
ROM and was not run when this was committed.
