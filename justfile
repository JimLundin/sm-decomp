default:
    @just --list

# Download pinned dependencies and build the Rust tools.
setup:
    python3 tools/setup.py

setup-lsnes:
    python3 tools/setup_lsnes.py

doctor:
    python3 tools/doctor.py

# Immutable setup evidence; a pass verifies setup only and cannot accept readiness.
readiness-preflight:
    python3 tools/readiness.py preflight

# Complete enabled native corpus and independent generated-step timing checks.
pattern-check:
    python3 tools/readiness_patterns.py --timing

# Cold instruction/bus replay; immutable evidence names the first disagreement.
diagnose start="0" instructions="100":
    python3 tools/readiness_diagnostics.py --start-input {{start}} --instructions {{instructions}}

# Includes corpus cases if `just fuzz` has generated them.
check:
    cargo fmt --all --check
    cargo test --workspace --locked --offline
    just hardware-check
    python3 -m unittest discover -s tools -p 'test_*.py'

hardware-check:
    python3 tools/test_hardware.py

# Independent synthetic timing diagnostic; NOP cases expose an open HDMA issue.
probe-hdma kind="indirect" loop="nop" phases="8":
    python3 tools/probe_hdma.py --kind {{quote(kind)}} --loop {{quote(loop)}} --phases {{phases}}

fmt:
    cargo fmt --all

lint:
    cargo clippy --workspace --all-targets --locked --offline -- -D warnings

demo:
    cargo run -p game --locked --offline

# Writes ROM-derived output and switches game away from the built-in demo.
recompile:
    cargo run -p recomp --locked --offline
    cargo build -p game --locked --offline

recompile-steps:
    cargo run -p recomp --locked --offline -- --out out/generated/game.rs --steps-out out/generated/steps.rs
    cargo build --release -p diff --locked --offline

diff-generated input frames="9000" out="out/diff-generated":
    cargo run --release -p diff --locked --offline -- {{quote(input)}} --generated --frames {{frames}} --out {{quote(out)}}

iterate input="movies/super-metroid-any-3653.lsmv" frames="9000":
    just recompile-steps
    just diff-generated {{quote(input)}} {{frames}}

sync-compare baseline candidate start end offset="0":
    python3 tools/compare_sync.py {{quote(baseline)}} {{quote(candidate)}} --start-frame {{start}} --end-frame {{end}} --candidate-frame-offset {{offset}}

lsnes-replay input frames="12000" out="out/sync/lsnes.csv":
    python3 tools/lsnes-replay.py {{quote(input)}} --frames {{frames}} --out {{quote(out)}}

sync-smoke:
    just sync-check movies/super-metroid-any-3653.lsmv 390 272

sync-check input="movies/super-metroid-any-3653.lsmv" frames="12000" start="272":
    just lsnes-replay {{quote(input)}} {{frames}}
    just sync-replay out/sync/lsnes.csv {{quote(input)}} {{frames}} {{start}}

# Reuse a completed original-core capture while iterating on hardware fixes.
sync-replay baseline="out/sync/lsnes.csv" input="movies/super-metroid-any-3653.lsmv" frames="12000" start="272":
    cargo run --release -p snes --bin reference-replay --locked --offline -- {{quote(input)}} --frames {{frames}} --out out/sync/reference-fingerprints.csv --checkpoints out/sync/reference.csv --lsnes-frame-labels
    just sync-compare {{quote(baseline)}} out/sync/reference.csv {{start}} {{frames}}

# Initial corpus any% movie: verify every recorded frame from a clean start.
sync-full:
    just sync-check movies/super-metroid-any-3653.lsmv 129712 272

reference-smoke frames="120":
    cargo run -p snes --bin reference-smoke --locked --offline -- rom/sm.sfc {{frames}}

mesen-smoke:
    tools/mesen --testRunner tools/mesen-smoke.lua rom/sm.sfc --timeout=30

# Optional instruction semantics check; existing decimal ADC failures are documented.
fuzz cases="20":
    cargo run -p fuzz --locked --offline -- gen deps/65816/v1 {{cases}}
    cargo test -p fuzz --locked --offline

symbols:
    python3 tools/symbols.py

fetch-movies:
    python3 tools/fetch_movies.py

movie input output:
    cargo run -p movie --locked --offline -- {{quote(input)}} {{quote(output)}}

import-movies:
    python3 tools/import_movies.py

reference-replay input frames="3600" out="out/reference-replay.csv":
    cargo run --release -p snes --bin reference-replay --locked --offline -- {{quote(input)}} --frames {{frames}} --out {{quote(out)}}

diff-fallback input frames="3600" out="out/diff":
    cargo run --release -p diff --locked --offline -- {{quote(input)}} --fallback-only --frames {{frames}} --out {{quote(out)}}

mesen-replay input frames="3600":
    python3 tools/mesen-replay.py {{quote(input)}} --frames {{frames}}
    tools/mesen --doNotSaveSettings --snes.port1.type=SnesController --snes.port2.type=None --testRunner out/mesen-replay.lua rom/sm.sfc --timeout=180

trace-merge misses:
    cargo run -p trace --locked --offline -- merge config/functions.toml {{quote(misses)}}

trace-names:
    cargo run -p trace --locked --offline -- names config/functions.toml out/symbols.toml

# Fresh strict original-core synchronization with immutable evidence.
readiness-sync *args:
    python3 tools/readiness_sync.py {{args}}

# One isolated cold parity run; readiness defaults retain the complete domain.
readiness-parity mode="generated" frames="18000":
    python3 tools/readiness_parity.py --mode {{quote(mode)}} --frames {{frames}}
