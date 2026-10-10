# Super Metroid source-recovery project comparison

Researched against upstream GitHub READMEs, source trees, licenses, commit histories, and issue discussions on **2026-10-10**. The intended use is **private**, and the goal is understandable equivalent source rather than rebuilding a byte-identical ROM. No candidate was built or played through as part of this research.

## Recommendation

**Follow-up correction, 2026-10-10:** the broader [community audit](DECOMP_COMMUNITY_AUDIT.md) found the actively developed [MrJackSpade/SuperMetroidDecomp C# port](https://github.com/MrJackSpade/SuperMetroidDecomp), plus substantive unmerged C fixes and testing work. The candidate table below records the earlier C/assembly inspection and is not a complete current-community ranking. Consult the updated [source stack](DECOMP_SOURCES.md) before selecting translation inputs. No project license was located for the C# pin, and its documentation has inconsistent completeness claims, so its active development does not by itself establish unrestricted reuse or full vanilla correctness.

The strongest starting point is the **snesrev/sm C decompilation family**. Within that family, **testyourmine/sm_rewrite is the best inspected candidate for readable source and continued cleanup**, because it inherits the broad game implementation and adds documentation, function rewrites, data extraction, and a unified CMake build. Its MIT license is compatible with this workspace's declared MIT license. Keep the original **snesrev/sm** as the baseline for comparison and debugging. This is a recommendation based on inspected source and author statements, not a verified ranking of full-game correctness.

**InsaneFirebat/sm_disassembly has the most permissive declared project license (0BSD)** and is a valuable named assembly reference. It does not provide high-level C/Rust game source, so it complements rather than replaces the C decompilation.

Private use makes distribution obligations much less relevant; there is no reason to exclude a useful copyleft tool solely because it is copyleft. For the leading C candidates, licensing does not force a choice: both carry the same MIT notice.

## Main candidates

| Project | What exists | Progress and limitations supported by sources | Declared licensing | Role |
| --- | --- | --- | --- | --- |
| [snesrev/sm](https://github.com/snesrev/sm) | C implementation organized into ROM-bank files, named functions, gameplay/boss/menu/cinematic logic, C sound player, emulated hardware, and frame comparison with original ROM execution | README explicitly says early, buggy, messy. Broad code is present; this is much more than a few translated instruction handlers. No published completeness percentage or automated full-game success evidence found | MIT for project software; root license additionally includes an Opus BSD-style notice | Original C baseline and validation reference |
| [testyourmine/sm_rewrite](https://github.com/testyourmine/sm_rewrite) | Continuation of the C port, with rewrites, types/enums, per-bank headers, documentation, extraction scratchpad, CMake | Latest main commit reports bank 81 rewritten/documented and mostly extracted; earlier commit reports same for bank 90. README says very early, ROM pointers remain, variable naming remains, MSU-1 disables mismatch checks, widescreen not started | Same MIT and Opus notice as upstream | Best inspected readable-source continuation |
| [InsaneFirebat/sm_disassembly](https://github.com/InsaneFirebat/sm_disassembly) | Relocatable named assembly based on P.JBoy's bank logs; ROM assets extracted locally; Asar rebuild workflow | Extensive bank coverage and recent maintenance, but an assembly disassembly rather than portable high-level source | Project 0BSD; bundled Asar LGPL-3.0-or-later, with separate exceptions described in its license | Strong supplementary reference |
| [strager/supermetroid](https://github.com/strager/supermetroid) | Assembly source intended to produce the original NTSC ROM hash | Useful older disassembly, outside the requested high-level-source direction | GitHub reports no detected license; no affirmative project permission established in this inspection | Prefer explicitly licensed references |
| [thijsrijkers/super_metroid_disassembly](https://github.com/thijsrijkers/super_metroid_disassembly) | DiztinGUIsh-derived disassembly project | Older assembly project; README gives little completeness information | GitHub reports no detected license | No advantage over the stronger named reference found |

Evidence: [snesrev README](https://github.com/snesrev/sm/blob/578f90b3cc49557bb70060ad033bb90b8cf8ac50/README.md), [source tree](https://github.com/snesrev/sm/tree/578f90b3cc49557bb70060ad033bb90b8cf8ac50/src), [snesrev license](https://github.com/snesrev/sm/blob/578f90b3cc49557bb70060ad033bb90b8cf8ac50/LICENSE.txt); [rewrite README](https://github.com/testyourmine/sm_rewrite/blob/523787384a7c912153883b21714f5edabf4e2406/README.md), [bank 81 commit](https://github.com/testyourmine/sm_rewrite/commit/523787384a7c912153883b21714f5edabf4e2406), [bank 90 commit](https://github.com/testyourmine/sm_rewrite/commit/45fcc94c37710d00deabe3d7cc24076caf1a9ca3), [rewrite license](https://github.com/testyourmine/sm_rewrite/blob/523787384a7c912153883b21714f5edabf4e2406/LICENSE.txt); [InsaneFirebat README](https://github.com/InsaneFirebat/sm_disassembly/blob/main/README.md), [0BSD license](https://github.com/InsaneFirebat/sm_disassembly/blob/main/LICENSE.txt), [Asar notice](https://github.com/InsaneFirebat/sm_disassembly/blob/main/tools/asar-LICENSE.txt); [strager README](https://github.com/strager/supermetroid/blob/master/README.md), [metadata](https://api.github.com/repos/strager/supermetroid); [thijsrijkers README](https://github.com/thijsrijkers/super_metroid_disassembly/blob/main/README.md), [metadata](https://api.github.com/repos/thijsrijkers/super_metroid_disassembly).

## Why the C family is already substantially useful

The upstream source includes actual reconstructed algorithms. For example, [`DecompressToMem`, corresponding to ROM address $80B119](https://github.com/snesrev/sm/blob/578f90b3cc49557bb70060ad033bb90b8cf8ac50/src/sm_80.c#L2488), is already a C routine. Bank A9 contains Mother Brain/baby Metroid logic, while bank 8B handles cinematics and credits. The [rewrite bank guide](https://github.com/testyourmine/sm_rewrite/blob/523787384a7c912153883b21714f5edabf4e2406/docs/banks.md) explains these responsibilities; the [rewrite example](https://github.com/testyourmine/sm_rewrite/blob/523787384a7c912153883b21714f5edabf4e2406/docs/rewrite_example.md) shows the intended transformation from decompiler temporaries and memory aliases to meaningful controller-binding logic. This aligns directly with readable source recovery.

The original [CPU infrastructure](https://github.com/snesrev/sm/blob/578f90b3cc49557bb70060ad033bb90b8cf8ac50/src/sm_cpu_infra.c) runs original emulated frames and C frames and compares memory/state. It also patches original execution for comparison and includes `RunAsmCode`, so this is not proof of an entirely emulator-independent game or strict original-hardware equivalence. ROM reads can also represent game data or game-specific instruction lists; the presence of `RomPtr` alone does not prove untranslated CPU instructions.

Playability evidence reaches late-game encounters: upstream has a [closed Mother Brain phase-two crash report](https://github.com/snesrev/sm/issues/30), and the rewrite has [closed Draygon](https://github.com/testyourmine/sm_rewrite/issues/5) and [Mother Brain room](https://github.com/testyourmine/sm_rewrite/issues/6) reports with author comments confirming fixes. These support broad playable coverage and an actual debugging history. They **do not establish a successful credits run, all-items run, absence of regressions, or 100% translated code**. Neither primary C repository publishes GitHub releases at the time inspected ([upstream releases API](https://api.github.com/repos/snesrev/sm/releases), [rewrite releases API](https://api.github.com/repos/testyourmine/sm_rewrite/releases)).

## Recent forks do not automatically indicate further decompilation

The recent fork ecosystem was inspected so that upstream age was not mistaken for the state of the whole family:

| Fork | What the primary source says | Implication |
| --- | --- | --- |
| [Raekwon1603/super_metroid-android](https://github.com/Raekwon1603/super_metroid-android) | README explicitly says Android gameplay executes the original ROM through the emulator, after an arm64 C mistranslation softlock. Development has moved to a RetroArch/bsnes-hd fork | Recent platform features are not evidence of a more complete C decompilation |
| [D-Rex3/super-metroid-x](https://github.com/D-Rex3/super-metroid-x) | Main change is an Xbox D3D8 framebuffer/platform layer | Platform port, not a demonstrated advance in recovered source |
| [CharlesAverill/sm-3ds-lib](https://github.com/CharlesAverill/sm-3ds-lib) | Build options for native/emulated execution and 3DS-oriented build changes | Useful platform work; no stronger documented readable-source recovery found |
| [testyourmine/sm-redux](https://github.com/testyourmine/sm-redux) | Ports the Redux ROM hack, requires patched ROM, disables mismatches | Relevant if Redux is desired; unsuitable as the primary vanilla correctness baseline |
| [Dakangel9/sm_rw_doble](https://github.com/Dakangel9/sm_rw_doble) | Adds dual-screen display and a makefile to rewrite | Extra UI, not a demonstrated additional bank rewrite |
| [aric-davison/sm_rewrite](https://github.com/aric-davison/sm_rewrite), [ganola-net/sm_rewrite](https://github.com/ganola-net/sm_rewrite) | Despite recent repository push dates, inspected default branches resolve to the same rewrite main commit | Push timestamps are not evidence of changed game logic |

Sources: each linked repository's README and default-branch commit history, accessed 2026-10-10. The Android [README at inspected head](https://github.com/Raekwon1603/super_metroid-android/blob/bca0803b2c621d153dba72b296815e8cca7a0a86/README.md) makes the execution-mode distinction explicit. Xbox [platform commit](https://github.com/D-Rex3/super-metroid-x/commit/f3f45b916e04383e55b3b1a96d3d82d8f0492dfd), 3DS [build-mode commit](https://github.com/CharlesAverill/sm-3ds-lib/commit/d4e4f4250e22581dd6f8d07e8a56009c0790946b), Redux [README](https://github.com/testyourmine/sm-redux/blob/main/README.md), dual-display [commit](https://github.com/Dakangel9/sm_rw_doble/commit/6f9b46117da13b55e132b8518977fc029161b1c0).

## Maintenance snapshots

Use actual default-branch commits when assessing maintenance. Repository `updated_at` can change from stars or metadata, and `pushed_at` can reflect other branches.

| Repository | Inspected main HEAD | HEAD committer date UTC | Repository pushed_at UTC |
| --- | --- | --- | --- |
| snesrev/sm | `578f90b3cc49557bb70060ad033bb90b8cf8ac50` | 2023-04-17 20:27:21 | 2023-08-21 22:53:49 |
| testyourmine/sm_rewrite | `523787384a7c912153883b21714f5edabf4e2406` | 2024-10-08 08:54:39 | 2024-10-08 03:54:45 |
| InsaneFirebat/sm_disassembly | main inspected through API/source | Not recorded in this comparison | 2026-10-05 03:09:16 |

Sources: [upstream commits API](https://api.github.com/repos/snesrev/sm/commits?per_page=1), [rewrite commits API](https://api.github.com/repos/testyourmine/sm_rewrite/commits?per_page=1), and each repository's [GitHub metadata API](https://api.github.com/repos/InsaneFirebat/sm_disassembly). The differing push/commit timestamps are reported verbatim, not normalized into an inferred activity date.

## Licensing detail appropriate to private use

- **MIT C candidates:** permit use, modification, copying, and incorporation into an MIT project, subject to preservation of their notice in copies/substantial portions. GitHub labels them `Other`/`NOASSERTION` because the root file contains multiple notices; the actual text clearly begins with MIT. Do not infer no license from the badge.
- **0BSD assembly reference:** fewer attribution conditions than MIT for the project's own contributions. The bundled **Asar tool** has its own LGPL terms. Its notice expressly says personal use is unrestricted and claims no copyright over input or generated binary/text output. The tool license does not automatically determine the license of recovered game logic.
- **Components:** the C candidates bundle `stb_image.h`, which explicitly offers a choice of MIT or public-domain/Unlicense at the end of the file ([source](https://github.com/snesrev/sm/blob/578f90b3cc49557bb70060ad033bb90b8cf8ac50/third_party/stb/stb_image.h#L7859)). SDL2 is an external dependency ([rewrite manifest](https://github.com/testyourmine/sm_rewrite/blob/523787384a7c912153883b21714f5edabf4e2406/vcpkg.json)); preserve dependency-specific licensing when incorporating those components. This comparison is not a complete audit of every generated loader/platform asset.
- **Underlying game:** contributor MIT/0BSD permissions do not by themselves grant Nintendo's rights in the original game, extracted art/music, or ROM. Both primary C builds require the user's own ROM ([upstream build instructions](https://github.com/snesrev/sm/blob/578f90b3cc49557bb70060ad033bb90b8cf8ac50/BUILDING.md), [rewrite README](https://github.com/testyourmine/sm_rewrite/blob/523787384a7c912153883b21714f5edabf4e2406/README.md)). For this private technical project, retain that local-ROM workflow.

## Practical next step

Pin the original C source already available locally and the rewrite HEAD above; build the rewrite against the same locally supplied ROM; run representative gameplay and a credits-reaching replay with comparison enabled. Evaluate readable routines and remaining ROM dependencies while preserving the original source as a reference. This settles the important uncertainty—whether the readability improvements retain enough correctness for this project's intended private workflow—before committing to a new decompilation architecture.
