# sm_app

The desktop application. It builds the binary `sm` (`target/release/sm.exe`)
and keeps the original command line:
`sm [--config FILE] [--debug] [ROM] [--room-pack PATH]`.

| File | Contents |
| --- | --- |
| `main.rs` | Startup, the run loop, host callbacks and the native desktop bridge (`libsm_desktop.a`) |
| `config.rs` | `sm.ini` parsing through the native configuration library (`libsm_config.a`) |
| `platform/opengl.rs`, `platform/glsl_shader.rs` | OpenGL presentation and GLSL shader loading |
| `platform/texture_png.rs` | PNG loading through the pinned stb_image archive (`libsm_stb_image.a`) |

It still depends on `sm` for the game, emulated hardware and audio, and builds
only for Windows x86-64 GNU. The accepted architecture replaces SDL2 and
OpenGL with `winit`, `wgpu` and `cpal` behind `sm_runtime`. Build steps are in
the [build guide](../../docs/ITERATION.md).
