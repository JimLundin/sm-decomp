# Presentation consumer

`Window::new(width, height, scale, optional_sdl_path)` opens one SDL2 window
and a 32000 Hz signed-16-bit stereo output device. `poll()` returns held native
button bits, and `present(pixels, audio)` consumes completed output. Keep all
calls on the creating main thread. The integrated Game's caller paces at
`32000/534` native host steps per second, matching its completed PCM block.
There are no gameplay, emulation, save-state or replay dependencies.

Pixels are packed `u32` ARGB8888; alpha is ignored. The recovered renderer writes
B,G,R,0 bytes on the supported little-endian host (`src/snes/ppu.c:854–858`),
and its own SDL renderer uses `SDL_PIXELFORMAT_ARGB8888` (`src/main.c:271`).
The upload therefore preserves the native buffer without a color conversion.

Fresh recovered-C defaults use native A for jump, X for shoot and B for run:

| Keyboard | Native button | Default action |
| --- | --- | --- |
| Arrows | D-pad | Move |
| Z | A | Jump |
| X | X | Shoot |
| C | B | Run |
| D | Y | Item cancel |
| A / S | L / R | Aim down / up |
| Either Shift | Select | Item select |
| Enter | Start | Start / pause |
| Escape | — | Close |

Focus loss releases held keys. The window close event and Escape return `None`.
Game input normalization stays in Game. Audio is copied to SDL's queue, never
generated in a callback. If old queued audio plus the incoming block exceeds
250 ms, the old queue is cleared; `audio_resets()` exposes this. Blocks larger
than 250 ms and incomplete stereo frames fail before presentation.

SDL2 is loaded through `libloading` without headers or link-time dependencies.
Use the explicit path argument, `SM_RESTART_SDL2`, or an installed system SDL2.
On the development machine the existing dependency is:

```sh
export SM_RESTART_SDL2=/home/jim/sm-decomp/deps/mesen-runtime/usr/lib/x86_64-linux-gnu/libSDL2-2.0.so.0.10.0
export LD_LIBRARY_PATH=/home/jim/sm-decomp/deps/mesen-runtime/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
```

No runtime is copied into this crate. Keep the dependency's license notices
with any redistributed runtime. A missing display or audio device is an error;
for deliberate headless/silent checks select SDL's dummy drivers explicitly.
The retained SDL2 also needs its adjacent Wayland libraries even when selecting
the X11/dummy driver, so set `LD_LIBRARY_PATH` before launching the executable.

This crate is registered in the independent restart workspace. From the
repository root, its focused checks and standalone presentation example are:

```sh
cargo test --manifest-path restart/Cargo.toml -p sm-presentation --locked --offline -j1
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy cargo test --manifest-path restart/Cargo.toml -p sm-presentation --locked --offline -j1 -- --ignored --test-threads=1
DISPLAY=:0 SDL_AUDIODRIVER=dummy cargo run --manifest-path restart/Cargo.toml -p sm-presentation --example presentation-smoke --locked --offline -j1
```

The opt-in smoke checks actual SDL event delivery, held/released buttons, focus
loss, channel-correct renderer readback, bounded PCM queueing, window close,
Escape, and shutdown/reopening. The desktop example displays a color gradient
for three seconds. It intentionally queues silence; audible game output needs
the integrated Game. Neither check makes a gameplay or original-ROM claim.

For actual gameplay, use `sm-restart play` or `sm-restart run --present replay`
as shown in the [restart README](../README.md). Those commands share the same
Game, create fresh run directories and retain native inputs and observations.
The Ceres replay uses `out/runs/ceres-06/inputs.txt` from the successful native
run. Gameplay remains recovered C; original-ROM comparison is not tested.
