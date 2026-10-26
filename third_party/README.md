# Upstream code and licenses

These folders keep the notices for code used by this project. Each reference
below explains the pinned source and how it enters the build.

| Source | Used for |
| --- | --- |
| [snesrev/sm](snesrev/README.md) | Original source of the one-time Rust translation |
| [LakeSnes](lakesnes/README.md) | Pinned CPU, PPU/DSP and SPC implementations |
| [stb_image](stb_image/README.md) | PNG decoding |
| [snesrev/zelda3](zelda3/README.md) | Selected native music, configuration, utility, shader and desktop routines |

Native installers fetch and verify external sources into local build directories;
project adapters and compatibility changes remain in this repository. Include
the applicable license notices when redistributing source or binaries.
