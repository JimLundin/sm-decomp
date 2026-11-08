#!/usr/bin/env bash
# Local source gate: the formatting, lint, crate-rule, doc-link and report checks of Quality. Only
# the report check links (verify's lib test, tools/check_report.sh); CI runs the other tests.
# Overrides: SM_RUST_BIN (toolchain bin), SM_GATE_DIR.
CHANNEL=$(sed -n 's/^channel = "\(.*\)"$/\1/p' rust-toolchain.toml)
R="${SM_RUST_BIN:-$USERPROFILE/.rustup/toolchains/$CHANNEL-x86_64-pc-windows-gnu/bin}"
O="${SM_GATE_DIR:-$TEMP/sm-gate}/$(git rev-parse --short HEAD)-$$"
mkdir -p "$O"; ok=1
trap 'rm -f -- "$O/check.log"; rmdir -- "$O" 2>/dev/null || true' EXIT
fail() { echo "$1"; ok=0; }


"$R/cargo" fmt --all --check >/dev/null 2>&1 || fail "FMT FAIL"

# Check mode doesn't link, so oracle needs no native archive.
if ! "$R/cargo" clippy --locked -p data -p game -p runtime -p app -p verify -p tools -p canonical -p canonical_derive -p oracle_record -p oracle -p clean_types -p clean_model_samus_motion -p clean_model_camera --all-targets --no-deps -- -D warnings >"$O/check.log" 2>&1; then
  fail "CLIPPY FAIL"
  grep -E '^(error|warning)|-->' "$O/check.log" | head -10
fi
# The clean crates under the clean room's ceilings (G5): tools/cleanroom/clippy.toml, the export's.
if ! CLIPPY_CONF_DIR="$(git rev-parse --show-toplevel)/tools/cleanroom" "$R/cargo" clippy --locked -p clean_types -p clean_model_samus_motion -p clean_model_camera --all-targets --no-deps -- -D warnings >"$O/check.log" 2>&1; then
  fail "CLEAN CLIPPY FAIL"
  grep -E '^(error|warning)|-->' "$O/check.log" | head -10
fi
CARGO="$R/cargo" bash tools/check_crate_rules.sh >/dev/null 2>&1 || fail "CRATE RULES FAIL"
sh tools/check_doc_links.sh >/dev/null 2>&1 || fail "DOC LINKS FAIL"
CARGO="$R/cargo" SM_RUST_BIN="$R" bash tools/check_report.sh >/dev/null 2>&1 || fail "REPORT STALE: run cargo run -p tools -- report --write"
if [ "$ok" -eq 1 ]; then
  echo "GATE OK $(git rev-parse --short HEAD) $(git branch --show-current)"
else
  echo "GATE FAILED $(git branch --show-current)"
  exit 1
fi
