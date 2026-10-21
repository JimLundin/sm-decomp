# PNG dependency

The frontend's PNG decoder links a static build of unmodified **stb_image v2.26**.
`sm/native/stb_image.c` is the implementation bridge;
`sm/src/platform/texture_png.rs` owns the Rust interface and copies decoded pixels
into a Vec before freeing the upstream allocation. File loading uses the C
runtime. The native archive is `libsm_stb_image.a`.

## Pinned source and required configuration

Use the header distributed in the published, non-yanked
[`stb-sys 0.6.0` package](https://crates.io/crates/stb-sys/0.6.0), released
2020-08-08. The package records repository commit
`01151c4e6387eaa8bae53c3132d1c3fb0e365896` in
[`mxpv/stb`](https://github.com/mxpv/stb). Only its unmodified C header is used;
the Rust wrapper and its Cargo build script are not dependencies.

| Input | Exact value |
| --- | --- |
| Package download | `https://crates.io/api/v1/crates/stb-sys/0.6.0/download` |
| Package SHA256 | `fedcddefe320931b97fea7c9bc9168567eadaf71c87a74a3491d5b46b5c573de` |
| Header within package | `stb-sys-0.6.0/vendor/stb/stb_image.h` |
| Header SHA256 | `8e5b0d717dfc8a834c97ef202d20e78d083d009586e1731c985817d0155d568c` |

The bridge defines `STB_IMAGE_IMPLEMENTATION`, `STBI_ONLY_PNG` and
`STBI_MAX_DIMENSIONS=4096`. Keep its stdio support enabled. Compile with
`-ftrivial-auto-var-init=zero`: the existing decoder returns zero for absent
palette entries, while this upstream source otherwise leaves those local bytes
uninitialized. An unsupported initialization flag is a build error; do not drop
the flag or patch the header to make a build succeed.

The older version is deliberate: newer stb releases reject reserved DEFLATE
codes that the existing decoder accepts. Changing the pin or these settings
requires renewed compatibility checks. The header adds 7,762 lines of external
dependency code; it has not been eliminated from the program.

## Windows GNU installation

Run these direct PowerShell commands from the repository root. They require
Windows `tar`, the `stable-x86_64-pc-windows-gnu` Rust toolchain and an existing
SDL2 installation. Install the [CPU dependency](CPU_DEPENDENCY.md) first.
Set `$codecNativeDirectory` to the native library directory
already used by that installation; the path below is the verified workspace
layout. The commands install the distinct codec archive and leave SDL files alone.
When `LIBRARY_PATH` is already configured, choose a directory already in that
search path; the commands preserve the existing setting.

The verified compiler is the official **Zig 0.15.2 Windows x86-64** distribution:

```powershell
$codecTools = Join-Path $env:LOCALAPPDATA 'sm-decomp-tools'
$codecWork = Join-Path $codecTools 'stb-image-2.26'
$codecZig = Join-Path $codecTools 'zig-x86_64-windows-0.15.2/zig.exe'
$codecNativeDirectory = Join-Path $codecTools 'SDL2-2.32.10/SDL2-2.32.10/x86_64-w64-mingw32/lib'
New-Item -ItemType Directory -Force -Path $codecTools, $codecWork -ErrorAction Stop | Out-Null
if (-not (Test-Path -LiteralPath $codecNativeDirectory -PathType Container)) { throw 'Configure the existing SDL native library directory first.' }
if (-not (Test-Path -LiteralPath $codecZig -PathType Leaf)) {
    $codecZigZip = Join-Path $codecTools 'zig-x86_64-windows-0.15.2.zip'
    Invoke-WebRequest 'https://ziglang.org/download/0.15.2/zig-x86_64-windows-0.15.2.zip' -OutFile $codecZigZip -ErrorAction Stop
    if ((Get-FileHash -Algorithm SHA256 -LiteralPath $codecZigZip).Hash -ne '3a0ed1e8799a2f8ce2a6e6290a9ff22e6906f8227865911fb7ddedc3cc14cb0c') { throw 'Zig archive checksum mismatch.' }
    Expand-Archive -LiteralPath $codecZigZip -DestinationPath $codecTools -ErrorAction Stop
}
$codecZigVersion = & $codecZig version
if ($LASTEXITCODE -ne 0 -or $codecZigVersion -ne '0.15.2') { throw 'This recipe requires Zig 0.15.2.' }

$codecPackage = Join-Path $codecWork 'stb-sys-0.6.0.crate'
Invoke-WebRequest 'https://crates.io/api/v1/crates/stb-sys/0.6.0/download' -OutFile $codecPackage -ErrorAction Stop
if ((Get-FileHash -Algorithm SHA256 -LiteralPath $codecPackage).Hash -ne 'fedcddefe320931b97fea7c9bc9168567eadaf71c87a74a3491d5b46b5c573de') { throw 'Codec package checksum mismatch.' }
tar -xf $codecPackage -C $codecWork 'stb-sys-0.6.0/vendor/stb/stb_image.h'
if ($LASTEXITCODE -ne 0) { throw 'Header extraction failed.' }
$codecHeader = Join-Path $codecWork 'stb-sys-0.6.0/vendor/stb/stb_image.h'
if ((Get-FileHash -Algorithm SHA256 -LiteralPath $codecHeader).Hash -ne '8e5b0d717dfc8a834c97ef202d20e78d083d009586e1731c985817d0155d568c') { throw 'Codec header checksum mismatch.' }

$codecObject = Join-Path $codecWork 'stb_image.o'
$codecArchive = Join-Path $codecWork 'libsm_stb_image.a'
& $codecZig cc -target x86_64-windows-gnu -O2 -std=c11 -ftrivial-auto-var-init=zero -I (Split-Path -Parent $codecHeader) -c sm/native/stb_image.c -o $codecObject
if ($LASTEXITCODE -ne 0) { throw 'Codec compilation failed.' }
& $codecZig ar rcs $codecArchive $codecObject
if ($LASTEXITCODE -ne 0) { throw 'Codec archive creation failed.' }
Copy-Item -LiteralPath $codecArchive -Destination (Join-Path $codecNativeDirectory 'libsm_stb_image.a') -ErrorAction Stop
Copy-Item -LiteralPath LICENSE-stb_image -Destination (Join-Path $codecNativeDirectory 'LICENSE-stb_image') -ErrorAction Stop

$env:PATH = (Join-Path $env:USERPROFILE '.rustup/toolchains/stable-x86_64-pc-windows-gnu/bin') + ';' + $env:PATH
if (-not $env:LIBRARY_PATH) { $env:LIBRARY_PATH = $codecNativeDirectory }
$env:CARGO_BUILD_JOBS = '1'
cargo build --release -p sm
if ($LASTEXITCODE -ne 0) { throw 'Game build failed.' }
```

The adapter uses
`#[link(name = "sm_stb_image", kind = "static", modifiers = "-bundle")]`.
This leaves the archive for the final linker to resolve from the configured
native directory. The Windows production build passed with this attribute and
the existing `LIBRARY_PATH`, without a new build script or global Rust flags.

## Host Unix installation

These commands compile for the current Unix host. **Linux and other Unix builds
were not exercised in this experiment.** They require SDL2 development libraries,
Rust, `curl`, `tar`, `shasum`, `ar`, and Clang with
`-ftrivial-auto-var-init=zero` support, such as Clang 16 or newer. Run them from
the repository root after installing the [CPU dependency](CPU_DEPENDENCY.md).
The default `$codec_lib` below uses the same library prefix as that guide.

```sh
set -eu
codec_work="${XDG_CACHE_HOME:-$HOME/.cache}/sm-decomp/stb-image-2.26"
codec_lib="$HOME/.local/lib"
mkdir -p "$codec_work" "$codec_lib"
curl --fail --location 'https://crates.io/api/v1/crates/stb-sys/0.6.0/download' --output "$codec_work/stb-sys-0.6.0.crate"
printf '%s  %s\n' 'fedcddefe320931b97fea7c9bc9168567eadaf71c87a74a3491d5b46b5c573de' "$codec_work/stb-sys-0.6.0.crate" | shasum -a 256 -c -
tar -xf "$codec_work/stb-sys-0.6.0.crate" -C "$codec_work" 'stb-sys-0.6.0/vendor/stb/stb_image.h'
codec_header="$codec_work/stb-sys-0.6.0/vendor/stb/stb_image.h"
printf '%s  %s\n' '8e5b0d717dfc8a834c97ef202d20e78d083d009586e1731c985817d0155d568c' "$codec_header" | shasum -a 256 -c -
clang -O2 -std=c11 -fPIC -ftrivial-auto-var-init=zero -I "$(dirname "$codec_header")" -c sm/native/stb_image.c -o "$codec_work/stb_image.o"
ar rcs "$codec_work/libsm_stb_image.a" "$codec_work/stb_image.o"
cp "$codec_work/libsm_stb_image.a" "$codec_lib/libsm_stb_image.a"
cp LICENSE-stb_image "$codec_lib/LICENSE-stb_image"
LIBRARY_PATH="$codec_lib${LIBRARY_PATH:+:$LIBRARY_PATH}" cargo build --release -p sm
```

The last command supplies both static dependencies through the normal linker
search path. The static `-bundle` linkage allows intermediate rlibs to build
before archive resolution. These instructions do not establish Unix runtime
compatibility.

## Validation and distribution

On Windows GNU, the pinned source and required settings matched the previous
Rust decoder on 6,704 distinct generated PNG inputs, including malformed input,
reserved codes, palettes, all supported depths, Adam7, filters, transparency,
CgBI and truncated chunks. Memory and actual-file APIs produced 13,412 matching
comparisons. A later linkage-only adjustment passed another 202 focused API
comparisons with fresh embedded-source checks; decoder bodies were unchanged.
The complete default-feature release library and frontend build passed.
Allocation failures and every possible malformed stream were not tested.

The project uses the upstream header's **MIT license alternative**. Ship
[`LICENSE-stb_image`](../LICENSE-stb_image) with redistributed source, native
archives and game binaries that contain this statically linked code. It preserves
the complete upstream license block, including Sean Barrett's copyright notice.
Keep the license comments in any distributed header. The Rust wrapper package's
code is not copied into the project.
