# sm-decomp

A Super Metroid (SNES) decompilation project targeting a playable source port
with gameplay logic in readable Rust and emulated SNES hardware. Vanilla
Super Metroid comes first; Map Rando integration remains future work.

This public repository distributes reviewed source selections. It contains no
ROM, private captures, fixture payload archives or prior private Git history.
Supply your own authorized ROM for tests that require it. Historical private
evidence remains separate; this distribution makes no claim that such material
was never committed privately.

The existing decomp foundation includes accepted RAM decompression and
NextRandom comparisons documented in [docs/DECOMP_PIVOT.md](docs/DECOMP_PIVOT.md).
Full Rust gameplay ownership, original-ROM any% completion and Map Rando are
unfinished. Nine additional complete actor source candidates (191 definitions)
are in [translations](translations/README.md). See [translation progress](docs/TRANSLATION_PROGRESS.md). They are translated and unverified.
All nine are registered as restart library targets.
Library wiring does not provide production runtime dispatch or native adapters.
Their inclusion is not compilation,
integration, actual-C, live replay or original-ROM acceptance.

The project pins nightly-2026-09-29, Rust edition 2024. See
[docs/SETUP.md](docs/SETUP.md) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
The root patched RustySNES dependency requires the pinned prepare-rustysnes
bootstrap before clean-checkout Cargo resolution. Downloaded dependency trees
are excluded. The restart workspace and standalone tools have separate checks;
root checks alone do not cover them.

Binary SMF fixtures are excluded pending provenance review. Full fixture
tests also require separately authorized ROM, item-seed, energy-prefix and
Construction captures. Missing inputs remain failing acceptance gates.
The translation-batch workflow is acceptance pending and does not replace
root, standalone, actual-C, refusal/terminal, live or original-ROM gates.
Private inputs and complete run payloads must stay outside public artifacts.

Original contributions use the [MIT license](LICENSE). Applicable upstream
notices remain with their modules and must be preserved. Recovered function
boundaries and source are attributed to [snesrev/sm](https://github.com/snesrev/sm)
and [sm_rewrite](https://github.com/testyourmine/sm_rewrite); labels and tables
also use [bank logs](https://patrickjohnston.org/bank/). Hardware/reference work
builds on [RustySNES](https://github.com/doublegate/RustySNES),
[MesenCE](https://github.com/nesdev-org/MesenCE), lsnes and SingleStepTests.
See the retained module documentation for evidence and scope limits.
