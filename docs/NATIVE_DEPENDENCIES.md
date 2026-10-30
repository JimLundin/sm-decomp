# Native dependencies

Read this when installing or changing the native libraries the current game
and verifier link. They support the compat runtime, the oracle and the
SDL2/OpenGL desktop; the [accepted architecture](target-architecture/README.md)
removes them from the finished engine. Maintained bridges, installers and
callers count as project code; fetched dependency code is counted separately.

## What is linked

| Archive | Contents | Installer | Linked from |
| --- | --- | --- | --- |
| `liblakesnes_cpu.a` | LakeSnes 65816 CPU, used by the oracle | `tools/install_lakesnes.py cpu` | `sm/src/snes/cpu.rs` |
| `liblakesnes_backend.a` | LakeSnes PPU, built through `sm/src/snes/backend.c` | `tools/install_lakesnes.py backend` | `sm/src/snes/ppu.rs` |
| `libsm_config.a` | Selected zelda3 configuration, utility and shader routines, with `sm/src/native_config.c`, `native_util.c` and `native_shader.c` | `tools/install_native_config.py` | `sm/src/util.rs`, `crates/sm_app/src/config.rs`, `crates/sm_app/src/platform/glsl_shader.rs` |
| `libsm_desktop.a` | Selected zelda3 desktop loop, with `sm/src/native_desktop.c`; Windows GNU x64 only | `tools/install_native_desktop.py` | `crates/sm_app/src/main.rs` |
| `libsm_stb_image.a` | stb_image 2.26 PNG decoder; [separate recipe](PNG_DEPENDENCY.md) | none | `crates/sm_app/src/platform/texture_png.rs` |

The SPC700, DSP and music sequencer now run as Rust in `sm/src/snes/` and
`sm/src/spc_player.rs`; no archive provides them. `install_lakesnes.py` still
accepts an `spc` component, but no build links its archive. The headless
verifier links the CPU, PPU and utility code, not the configuration-input,
shader or desktop implementations.

## Installation

Use the shared installer with Zig 0.15.2 and matching SDL2 headers and
libraries. Keep a separate cache and native-library directory per target:

```text
python tools/install_native_dependencies.py --target x86_64-windows-gnu --zig <zig.exe> --prefix <source-cache> --sdl-include <SDL-include/SDL2> --library-directory <native-lib>
```

It builds the CPU, PPU, configuration and (on Windows) desktop archives. Build
the PNG decoder separately with [its recipe](PNG_DEPENDENCY.md). Keep
`<native-lib>` on `LIBRARY_PATH` for Cargo and direct verifier builds. The
`build` job in [ci.yml](../.github/workflows/ci.yml) runs exactly these steps
on a hosted Windows runner.

The installers verify pinned source and licence hashes. Static archives stay
unbundled in Rust rlibs (`modifiers = "-bundle"`) and resolve at final link; a
missing archive fails linking. Distribute
[THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) with native archives and
binaries, and keep upstream licences with cached source.

`--target x86_64-linux-gnu` builds PIC archives and omits the desktop bridge,
which needs Windows' 32-bit `long`. A Linux configuration archive does not make
a working Linux frontend, and Linux development is currently stopped.

For a scoped rebuild, run the corresponding installer with the same target,
Zig, library-directory and optional prefix arguments; the configuration and
desktop installers also need `--sdl-include`. Rebuild an archive when its
bridge, shared layout or installer inputs change. Embedded source stamps reject
stale archives before use; never edit a stamp to make an old archive pass.
Shader and desktop stamps include shared OpenGL and callback inputs, so a
change there can require rebuilding both archives.

## Source pins

The installers own the per-file SHA-256 tables. The different LakeSnes
revisions are deliberate compatibility choices; do not unify or update them
without renewed behaviour comparisons.

| Implementation | Upstream revision | Licence notice |
| --- | --- | --- |
| CPU | [dinkc64/LakeSnes `048a0d7`](https://github.com/dinkc64/LakeSnes/tree/048a0d72568668f74e0cdf371dcd9e8efd614965) | [MIT](../THIRD_PARTY_NOTICES.md#lakesnes-cpu) |
| PPU | [angelo-wf/LakeSnes `28b8fbc`](https://github.com/angelo-wf/LakeSnes/tree/28b8fbc15ad15afff003165ff4f2a8adb1c93d18) | [MIT](../THIRD_PARTY_NOTICES.md#lakesnes-ppu-dsp-and-spc700) |
| Configuration, utilities, shaders, desktop | [snesrev/zelda3 `45a149d8`](https://github.com/snesrev/zelda3/tree/45a149d8b4bb8098f53d1009f76e76e7d8b79bbf) | [MIT](../THIRD_PARTY_NOTICES.md#snesrevzelda3) |
| PNG decoder | stb_image 2.26 from [`stb-sys 0.6.0`](https://crates.io/crates/stb-sys/0.6.0) | [MIT option](../THIRD_PARTY_NOTICES.md#stb_image-mit-option) |

LakeSnes inputs are unmodified. Zelda3 routines are selected and explicitly
patched by the installers; current Rust behaviour defines those compatibility
changes. Original Super Metroid C is not a regeneration source.

## Compatibility constraints

- **CPU.** Rust keeps the CPU layout, legacy saves, hardware bus, oracle hooks
  and outer interrupt sequencing. Foreign registers are copied per
  instruction; no foreign state survives a snapshot.
- **PPU.** Rust keeps rendering through `sm_runtime::rendering`, sprite
  evaluation, colour math, scanlines, DMA, scheduling, snapshots and CPU
  hooks. Rebuild the backend archive after editing `sm/src/snes/backend.c`;
  its compiled bridge hash is checked.
- **Configuration and utilities.** Keep Super Metroid command numbers,
  defaults, filenames, accepted options and the shared `g_config` layout.
  Builds keep C exceptions and signed wrapping; `Die` may return or unwind.
- **Desktop and shaders.** Preserve startup and shutdown order, settings,
  the forced 44,100 Hz stereo request, the owner gate around input, frame,
  presentation and audio, and GL-call order. Disable floating-point
  contraction.

The [archived edition](archive/NATIVE_DEPENDENCIES-2026-10-26.md) keeps the
full adapter contracts, the retired SPC700, DSP and music archives, and the
adoption comparisons and line accounting.
