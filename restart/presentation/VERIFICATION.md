# Presentation implementation checks

Date: 2026-10-16. Worker: `/root/presentation`.
Worktree: `/home/jim/sm-decomp-restart-presentation`.
Scope: presentation only; no gameplay or original-ROM result is claimed.

The worker owns only `restart/presentation/**`. To leave the coordinator's
workspace manifest untouched, the checked crate was copied into the ignored
`out/source` with `[workspace]` appended to its manifest. All four Rust files
were checked byte-identical with `cmp`; the integrator registers the original
crate in the parent workspace. No C headers or system package changes occurred.

Focused commands, all exit 0 after the documented environment fix:

```sh
cargo test --manifest-path restart/presentation/out/source/Cargo.toml --offline -j1
cargo clippy --manifest-path restart/presentation/out/source/Cargo.toml --locked --offline -j1 --all-targets -- -D warnings
rustfmt --check --edition 2024 restart/presentation/src/lib.rs restart/presentation/src/sdl.rs restart/presentation/src/tests.rs restart/presentation/examples/smoke.rs
```

The ordinary suite passed two tests. Its real SDL test is opt-in and was run
separately with the README's `SM_RESTART_SDL2` and `LD_LIBRARY_PATH`:

```sh
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy cargo test --manifest-path restart/presentation/out/source/Cargo.toml --locked --offline -j1 -- --ignored --test-threads=1
```

One real-SDL test passed: native bits 0–11 from queued keyboard events,
held/released keys, both Shift keys, focus-loss clearing, exact RGB renderer
readback from zero-alpha input, queue limits, malformed-buffer rejection,
window close, Escape, complete SDL shutdown, and reopening.

The three-second desktop example passed on `DISPLAY=:0` both with dummy audio
and with the default PulseAudio device (`RDPSink`, through WSLg). Each presented
180 frames with zero audio queue resets. Final queued bytes were 5600 with the
dummy device and 3552 with the real device. The example plays silence; this
checks device opening and consumption, not audible game waveform correctness.

```sh
DISPLAY=:0 SDL_AUDIODRIVER=dummy cargo run --manifest-path restart/presentation/out/source/Cargo.toml --locked --offline -j1 --example smoke
DISPLAY=:0 cargo run --manifest-path restart/presentation/out/source/Cargo.toml --locked --offline -j1 --example smoke
```

The first dummy attempt failed before SDL initialization because the retained
SDL runtime could not resolve `libwayland-egl.so.1`. `ldd` also identified
`libwayland-cursor.so.0`; both already exist beside SDL in the retained runtime.
Adding that directory to `LD_LIBRARY_PATH` resolved the failure, including on
the real desktop. This requirement is recorded in README; nothing was installed.

## Cleanup and retained dependencies

All launched smoke processes completed. `pgrep` found no remaining matching
presentation process. `cargo clean --manifest-path
restart/presentation/out/source/Cargo.toml` removed 260 files, reported as
32.7 MiB. The target tree's measured apparent size beforehand was 28,928,246
bytes. A preceding direct removal command was rejected by the command guard;
Cargo's scoped cleanup completed instead.

The remaining ignored `out/` measured 13,861,145 bytes: two retained executables
and the small exact source/manifest/lock snapshot. Available filesystem space
after cleanup was 107,818,090,496 bytes. The integrator should preserve these
retained artifacts before removing this implementation worktree, then remove
the worktree after integration.

Retained hashes were rechecked after cleanup:

| Artifact | SHA-256 |
| --- | --- |
| `out/retained/presentation-smoke` | `ae12c114e1484627cbd157196f20c795933b5b92cea72eaf54a810bc16c150ea` |
| `out/retained/presentation-tests` | `2e2e7d053b2e283dbd88a1dff54a59f19fcc0622adb017f74146eb4ff0dd4b66` |
| `src/lib.rs` | `db710df0aeadbacc07b17012660efc35fa6a09d4d149f108d2e4855d3a020e77` |
| `src/sdl.rs` | `a3a4cdbfd406128e6831a97d24a0ed337e6dbedd1c07088ea3667d7f00b8ecc6` |
| `src/tests.rs` | `5fbe58f5d5705cc3291afd3d2e528986e4a30019a309421931d469d9c18ba9b4` |
| `examples/smoke.rs` | `f46367cae91f3a6ab1f94f0a081594648d51b633b0b8ed7b859c54362b717388` |
| Retained `libSDL2-2.0.so.0.10.0` | `4f5bd612f19b4bb298ae31ffcd43a3725aa4cbb272423117deeeecc0b1b68938` |

The retained test executable's real-SDL test passed again after cleanup. SDL2,
its adjacent Wayland libraries, and the host desktop/audio remain external
runtime dependencies; their source/archive provenance is not established by
this presentation check. The existing runtime's notices remain in place.
