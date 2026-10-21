# Configuration and utility dependency

Configuration parsing, keyboard/gamepad mapping, byte-string/file helpers and
ByteArray storage use selected [Zelda3 sources](https://github.com/snesrev/zelda3/tree/45a149d8b4bb8098f53d1009f76e76e7d8b79bbf/src)
at revision `45a149d8b4bb8098f53d1009f76e76e7d8b79bbf`. Current Rust defines the
behavior; the installer applies explicit compatibility changes to that pin.
Include the [MIT notice](../third_party/zelda3/LICENSE.txt) with distributed binaries.

Install alongside the existing SDL/native libraries:

```text
python tools/install_native_config.py --zig <zig.exe> --sdl-include <SDL-include/SDL2> --library-directory <SDL-lib>
```

Keep that directory in `LIBRARY_PATH`. The installer verifies source/license
hashes and builds `libsm_config.a` with separate configuration and utility
objects. Headless code pulls only the utility object; it needs no SDL input
implementation. The supplied installer and layout checks target Windows GNU
x64. Other targets require a matching native build and layout validation.

The frontend checks both objects' embedded source bytes before parsing or
looking up input. The verifier checks the utility object before execution.
These checks include shared Rust types and C declarations; rerun the installer
after changing an input named by it. Missing or stale dependencies fail before
use. Ordinary direct verifier builds now forward `LIBRARY_PATH` to the linker,
as Cargo verifier builds already did.

The compatibility changes preserve SM command numbers, defaults, filenames and
accepted options. `g_config` has the same layout and remains shared in place.
Unknown Zelda options remain unknown; unsupported OpenGL ES still selects SDL.
Duplicate keyboard entries consume slots, equal-specificity gamepad entries
keep newest-first precedence, and the original allocation thresholds remain.
File buffers and pointers into them keep their original lifetime. Rust still
checks invalid gamepad indices. `Die` may return or unwind; foreign declarations
do not mark it as nonreturning. Builds keep exception support and signed wrapping.

The utilities preserve partial allocation state, failure order, aliased string
replacement and Destroy's unchanged size/capacity fields. Two explicit casts
retain the native `c_ulong` file-read result and `c_long` comment-span length.
No malformed-input behavior was intentionally repaired.

The complete replacement removes **541 maintained lines**: Rust loses 808;
C/header adapters add 99, the installer adds 159, and the verifier launcher
adds nine. Registration and verifier Rust changes are included in the Rust
count. The fetched dependency adds 529 configuration C, 110 configuration-header
and 280 utility C lines, plus the same 103-line types header already used by
native music. Dependency code is disclosed separately from maintained savings.

Source-checked old/new DLL comparisons pass 31 configuration scenarios, 5,463
Boolean cases, 263,438 key/gamepad lookups, 133 utility cases, 32 layout checks,
35 source-stamp checks, five bounds failures and three configuration allocation/limit
failures. Comparisons cover logical state, mutated buffers, diagnostics and
ordered allocation/file calls. File tests use real small files; a controlled
read result exercises Windows narrowing without claiming large-file coverage.
The large comment-span cast has source/width checks only. Existing unchecked
out-of-bounds allocation behavior is preserved, not exercised as valid input.

Normal release library/frontend and headless verifier builds pass. A separate
check calls the actual installed archive through the release library and
exercises configuration, keyboard/gamepad lookup and byte-buffer consumers.
No movie ran: these checks directly exercise the changed functions; gameplay,
GPU presentation and physical controller events are not claimed.
