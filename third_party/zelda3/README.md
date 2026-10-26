# snesrev/zelda3

Upstream: [snesrev/zelda3 at 45a149d8b4bb8098f53d1009f76e76e7d8b79bbf](https://github.com/snesrev/zelda3/tree/45a149d8b4bb8098f53d1009f76e76e7d8b79bbf).

The installers select upstream routines and apply maintained Super Metroid
compatibility changes. They verify source hashes and embed source identity
checks in the native archives; rebuild when the named adapter inputs change.

| Component | Installer | Guide |
| --- | --- | --- |
| Music sequencer | [install_native_music.py](../../tools/install_native_music.py) | [SPC/music](../../docs/SPC_DEPENDENCY.md#native-music-sequencer) |
| Configuration, utilities and shaders | [install_native_config.py](../../tools/install_native_config.py) | [Configuration](../../docs/CONFIG_DEPENDENCY.md), [shaders](../../docs/SHADER_DEPENDENCY.md) |
| Desktop host | [install_native_desktop.py](../../tools/install_native_desktop.py) | [Desktop](../../docs/DESKTOP_DEPENDENCY.md) |

All three installers require an explicit `--target`. Configuration and music
accept `x86_64-windows-gnu` or `x86_64-linux-gnu`; the desktop adapter currently
supports only `x86_64-windows-gnu`. Use matching native headers and libraries.

Retain the [MIT notice](LICENSE.txt) with distributed binaries.
