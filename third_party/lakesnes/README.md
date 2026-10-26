# LakeSnes

Different hardware components use separate pinned upstream revisions:

| Component | Upstream revision | Installer | Notice |
| --- | --- | --- | --- |
| CPU | [dinkc64/LakeSnes, 048a0d7](https://github.com/dinkc64/LakeSnes/tree/048a0d72568668f74e0cdf371dcd9e8efd614965) | [CPU installer](../../tools/install_lakesnes_cpu.ps1) | [MIT](LICENSE.txt) |
| PPU/DSP | [angelo-wf/LakeSnes, 28b8fbc](https://github.com/angelo-wf/LakeSnes/tree/28b8fbc15ad15afff003165ff4f2a8adb1c93d18) | [Backend installer](../../tools/install_lakesnes_backend.ps1) | [MIT](LICENSE-2022.txt) |
| SPC700 | [elzo-d/LakeSnes, 1da16a2](https://github.com/elzo-d/LakeSnes/tree/1da16a22db4d0f2e7f6960a9846ef4e1f3acbd29) | [SPC installer](../../tools/install_lakesnes_spc.ps1) | [MIT](LICENSE-SPC-2021.txt) |

The installers verify the downloaded C/header files before building static
archives. Rust retains hardware state, buses and compatibility adapters; this
is not a replacement of the entire SNES backend. Setup and boundary details:
[CPU/PPU/DSP](../../docs/CPU_DEPENDENCY.md), [SPC](../../docs/SPC_DEPENDENCY.md).
