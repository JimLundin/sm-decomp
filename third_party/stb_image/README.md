# stb_image

The PNG decoder uses **stb_image 2.26**, taken from the published
[stb-sys 0.6.0 package](https://crates.io/crates/stb-sys/0.6.0).
Its package repository is [mxpv/stb](https://github.com/mxpv/stb/tree/01151c4e6387eaa8bae53c3132d1c3fb0e365896).

Only `vendor/stb/stb_image.h` is used, through
[`sm/native/stb_image.c`](../../sm/native/stb_image.c). The Rust wrapper and its
Cargo build script are not dependencies. Exact package/header hashes, required
compiler flags and installation commands are in the
[PNG guide](../../docs/PNG_DEPENDENCY.md).

Retain the complete [license notice](LICENSE.txt).
