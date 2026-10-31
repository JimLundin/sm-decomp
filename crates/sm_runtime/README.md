# sm_runtime

Prepared media and shared rendering. It depends only on `sm_data` and forbids
`unsafe`. The accepted architecture makes this package the home of the
engine's renderer, sound engine, asset loading and device adapters; today it
holds the first pieces:

| Module | Contents |
| --- | --- |
| `rendering/` | The shared scanline core and its bounded input records; `prepared` drives it from prepared indexed media |
| `rendering/registers` | The PPU's line-rendering registers as a typed model; the verifier checks it against every emulated PPU write |
| `scene` | A read-only consumer of committed Scene history |
| `message` | Rendering of an extracted, prepared message display |

The emulated PPU in `sm` and the prepared-content paths both call this
renderer, which is why the oracle's image comparison is not independent
evidence for it. Audio still lives in `sm` (`spc_player.rs`, `snes/`).
