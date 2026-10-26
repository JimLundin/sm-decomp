# Shader dependency

GLSL preset parsing, shader compilation, multipass rendering and resource
destruction use selected [Zelda3 shader routines](https://github.com/snesrev/zelda3/blob/45a149d8b4bb8098f53d1009f76e76e7d8b79bbf/src/glsl_shader.c).
Revision `45a149d8b4bb8098f53d1009f76e76e7d8b79bbf` is pinned by source hashes.
The installer applies explicit compatibility changes against current Rust.
Distribute the [Zelda3 MIT notice](../THIRD_PARTY_NOTICES.md#snesrevzelda3) with binaries.

The existing configuration installer builds the shader as a separate member of
`libsm_config.a`. Rebuild the desktop archive too: its source checks include the
shared installer and OpenGL caller.

```text
python tools/install_native_config.py --target x86_64-windows-gnu --zig <zig.exe> --sdl-include <SDL-include/SDL2> --library-directory <SDL-lib>
python tools/install_native_desktop.py --target x86_64-windows-gnu --zig <zig.exe> --sdl-include <SDL-include/SDL2> --library-directory <SDL-lib>
```

Keep that directory in `LIBRARY_PATH`. The configuration installer also accepts
`--target x86_64-linux-gnu`, but the desktop installer currently supports only
Windows GNU x64. A Linux shader build does not establish a working Linux frontend.
The headless
verifier does not link the shader implementation.

The existing GL loader supplies the shader's 42 function pointers without extra
SDL queries. Fallback order and reload behavior match the generated GL bindings.
Rust retains source vectors, PNG loading and temporary pixel buffers, CString
allocation and deallocation, saturating dimensions and bounds failures. The
presenter's allocation, drawing and cleanup algorithms remain unchanged.

Compatibility preserves comments before version directives, configuration reads,
unsigned parameter locations, the one-based 20-slot parameter boundary, zeroed
status/log outputs, assertion diagnostics and GL call order. Recursive file and
path cleanup uses C scope cleanup with `-fexceptions`. Failed creation retains
the original raw-root ownership; a Rust guard separates GL destruction from
subsequent owned-memory release. Floating-point contraction is disabled.
Allocation sizes and counts change with the representation; allocator event
identity is not promised. The original non-unwinding GL ABI provides no guarantee
of recoverable missing-function panics, and these checks claim none.

Seven exact source inputs are embedded in the archive, including the Rust/C
adapters, PNG loader, OpenGL caller, shared types and installer. Stale inputs fail
at shader use or GL loading. Configuration and utilities retain their source
checks. No original Super Metroid C was consulted.

The complete replacement removes **282 maintained lines**: the shader loses 572
Rust lines; registrations add two, C/header adapters add 161 and the installer
adds 127. All retained callers and compatibility code count. It adds no macros.
The published input is **659 C and 99 header lines**, disclosed separately;
the existing published types header is shared.

Validation passes **38 actual old/new shader scenarios**, comparing logical state,
ordered GL calls, diagnostics, uploaded pixels and returned textures. They cover
presets, duplicate entries, recursive includes, missing/empty inputs, PNG success
and failure, compile/link failures, partial or absent GL outputs, scaling edges,
configuration mutation, rendering history and parameter bounds. **46 loader
cases** compare query order and resolved slots, including fallbacks and reloads.
The normal archive and a separate instrumented archive pass these checks.

Four old/new allocation-failure cases check returning and unwinding host errors
during recursive source growth. Temporary files, paths and vectors are released;
after explicitly reclaiming the raw root, no tracked allocations remain. A
candidate-only injected failure at the declared C-unwind destructor boundary
also checks the heap guard; it establishes no native GL-unwind guarantee.

The final normal release library separately passes actual presenter initialization,
22 valid/altered source checks and five shader comparisons; all 96 embedded
library inputs match current source. Release library/frontend/verifier builds,
formatting and progress checks pass. GL/SDL calls are controlled boundaries;
there is no physical GPU, device-concurrency or gameplay coverage claim.
No movie ran.
