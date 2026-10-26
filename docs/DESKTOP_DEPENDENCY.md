# Desktop host dependency

Window management, SDL presentation, keyboard/gamepad commands, frame scheduling,
performance digits and audio buffering use selected [Zelda3 desktop routines](https://github.com/snesrev/zelda3/blob/45a149d8b4bb8098f53d1009f76e76e7d8b79bbf/src/main.c).
The revision is `45a149d8b4bb8098f53d1009f76e76e7d8b79bbf`; current Rust defines
the behavior and the installer applies explicit compatibility changes.
Distribute the [Zelda3 MIT notice](../THIRD_PARTY_NOTICES.md#snesrevzelda3) with binaries.

Install alongside SDL and the other native libraries:

```text
python tools/install_native_desktop.py --target x86_64-windows-gnu --zig <zig.exe> --sdl-include <SDL-include/SDL2> --library-directory <SDL-lib>
python tools/install_native_config.py --target x86_64-windows-gnu --zig <zig.exe> --sdl-include <SDL-include/SDL2> --library-directory <SDL-lib>
```

Keep that directory in `LIBRARY_PATH`. Reinstall configuration/utilities when
updating from the preceding version: their embedded source checks include the
changed renderer callback types. The desktop installer verifies pinned source
and license hashes and builds `libsm_desktop.a`. The desktop installer accepts
only `--target x86_64-windows-gnu`: its current native bridge requires a 32-bit
`long`, so Linux LP64 is not supported. Match the Windows SDL2 headers and libraries.
The headless verifier does not link the desktop implementation.

Rust retains `GameOwner`, all game and SNES layouts, both pixel buffers, the
verifier entry, host diagnostics and the OpenGL presenter. Each input handler,
frame, presentation operation and audio callback retains its owner gate.
Audio acquires that gate before the SDL audio mutex. Internal callbacks permit
unwinding; the SDL audio entry retains its non-unwinding ABI. Renderer callbacks
no longer receive their unused Game argument; their caller still holds the gate.

Compatibility preserves SM filenames, command IDs, settings, startup/shutdown
order and early-failure behavior. It also preserves the forced 44,100 Hz stereo
audio request, obtained-frequency block sizing, native pixel-copy dimensions,
turbo cadence, bug-title updates, ordinary PauseDimmed toggle, live configuration
reads and Windows pointer-difference narrowing. Rust supplies the original
saturating FPS conversion and bounds/null-callback failures. Floating-point
contraction is disabled. The OpenGL allocation, viewport and cleanup algorithms
are unchanged.

The frontend checks seven exact source inputs embedded in the native archive,
including its installer, callback types and OpenGL source. Missing or stale
dependencies fail before desktop use. Rebuild the archive after changing one
of these inputs. Configuration/input lookups retain their own source checks.

The complete replacement removes **697 maintained lines**: Rust loses 1,010;
C/header adapters add 72, the installer adds 240 and the verifier launcher adds
one. This includes retained code, registrations and callers. The published
desktop input adds **882 external C lines**, sharing the existing config/types
headers; the installer emits a 720-line selection including compatibility edits.
No original Super Metroid C was consulted.

Source-checked comparisons pass **125 complete old/new host scenarios**. They
compare ordered calls, desktop state, configuration, relevant Game fields,
pixel buffers and audio buffers. Checks execute input, window, drawing, audio,
directory discovery, startup, event processing, scheduling and shutdown code,
including selected failures. They use the real GameOwner gate with observation
markers. SDL devices, filesystem effects and game-runtime operations are
controlled boundaries; this is single-threaded gate-scope coverage.

The actual installed archive separately passes **33** source, state, numeric
and failure checks through the normal release library. Final library, frontend
and headless verifier builds and source checks pass. The main implementation
matches the tested candidate; the final native object differs only in debug
sections. No movie ran. These checks establish no gameplay, physical controller,
GPU, audio-device or multithreaded stress-test coverage.
