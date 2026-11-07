#!/usr/bin/env bash
# check_crate_rules.sh: the crate plan's rules (docs/architecture/ARCHITECTURE.md §4.1, §4.2,
# D1; PRINCIPLES K1, K8, K9), checked over the new crates. Exit 1 lists each violation.
#   Dependencies (rules 1-3, D1): each new crate depends on exactly the workspace packages the
#     plan allows, over normal, build and dev dependencies with every feature on. Registry
#     packages aren't constrained here.
#   Sources (rules 4, 5, K8, K9): pattern checks over the engine crates' Rust files.
# Run from the repository root. CARGO overrides the cargo binary.
set -u
cargo=${CARGO:-cargo}
bad=0
fail() { echo "$1"; bad=1; }

engine="data game runtime app"
# The workspace packages each new crate may depend on directly (crates/README.md).
# `canonical` holds the codec trait so `data` can derive it; `canonical_derive` is its macro.
allowed() {
  case $1 in
    data) echo "canonical" ;;
    game) echo "data canonical" ;;
    runtime) echo "data" ;;
    app) echo "data game runtime" ;;
    oracle) echo "data oracle_record" ;;
    oracle_record) echo "" ;;
    verify) echo "data game runtime oracle_record canonical" ;;
    tools) echo "verify oracle runtime" ;;
    canonical) echo "canonical_derive" ;;
    canonical_derive) echo "" ;;
  esac
}
new_crates="data game runtime app oracle oracle_record verify tools canonical canonical_derive"

# Name the rule a forbidden edge breaks, for the message.
rule() {
  case $2 in sm|sm_*) echo "D1: no new crate depends on sm or sm_*"; return ;; esac
  case " $engine " in *" $1 "*)
    case $2 in oracle|verify|tools) echo "rule 1: engine crates don't depend on oracle, verify or tools"; return ;; esac ;;
  esac
  case $1/$2 in game/runtime|runtime/game) echo "rule 2: game and runtime don't depend on each other"; return ;; esac
  [ "$1" = oracle ] && { echo "rule 3: oracle depends only on data and its record types"; return; }
  echo "not in the crate plan"
}

for crate in $new_crates; do
  # Direct dependencies that are path packages, that is workspace members: "name vX (path)".
  if ! deps=$("$cargo" tree -q -p "$crate" --all-features -e normal,build,dev --depth 1 \
      --prefix none --format '{p}' 2>&1); then
    fail "$crate: cargo tree failed: $deps"; continue
  fi
  for dep in $(echo "$deps" | tail -n +2 | grep ' (' | grep -v ' (\*)$' | cut -d' ' -f1 | sort -u); do
    case " $(allowed "$crate") " in *" $dep "*) continue ;; esac
    fail "$crate depends on $dep ($(rule "$crate" "$dep"))"
  done
done

# Engine crates reach none of oracle, verify, tools, sm or sm_* even through another package.
for crate in $engine; do
  reach=$("$cargo" tree -q -p "$crate" --all-features -e normal,build,dev --prefix none \
    --format '{p}' 2>/dev/null | tail -n +2 | cut -d' ' -f1 | sort -u \
    | grep -Ex 'oracle|verify|tools|sm|sm_.*' || true)
  for dep in $reach; do fail "$crate reaches $dep through its dependencies (rule 1)"; done
done

# Rust source lines matching an extended regex, outside line comments. $1 names the check.
scan() {
  what=$1 pattern=$2; shift 2
  for crate in "$@"; do
    [ -d "crates/$crate" ] || continue
    grep -rnE --include='*.rs' -- "$pattern" "crates/$crate" \
      | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//' \
      | while IFS= read -r hit; do echo "$what: $hit"; done
  done
}

# Each scan's hits are violations.
hits=$(
  # Rule 4: no verification hooks in the engine (P6).
  scan "rule 4 (no cfg(verify), probe or observer)" \
    'cfg[[:space:]]*\(.*verify|[Pp]robes?\b|[Oo]bservers?\b' $engine
  # Rule 5 (K1): SNES addresses. Long WRAM ($7E/$7F), a bank:address pair, a LoROM long
  # address (banks $80-$DF, offsets $8000-$FFFF), and the PPU, CPU and DMA registers.
  # A bank-log reference in a comment is allowed (K9).
  scan "rule 5 (no SNES address literal)" \
    '0x7[EeFf]_?[0-9A-Fa-f]{4}\b|\$[0-9A-Fa-f]{2}:?[0-9A-Fa-f]{4}\b|0x[89A-Da-d][0-9A-Fa-f]_?[89A-Fa-f][0-9A-Fa-f]{3}\b|0x(21|42)[0-9A-Fa-f]{2}\b|0x43[0-7][0-9A-Fa-f]\b' \
    game runtime app
  # K8: stable Rust and no silenced lints.
  scan "K8 (no #![feature], allow or expect)" '#!?\[[[:space:]]*(feature|allow|expect)[[:space:]]*\(' $engine
  # K9: no routine labels for names.
  scan "K9 (no routine-label names)" '\b(func|sub|loc)_[0-9A-Fa-f]{2}_?[0-9A-Fa-f]{4}\b' $engine
)
[ -n "$hits" ] && fail "$hits"

# The clean-room crates, where the spine has them: nothing in them reaches outside the process
# when cargo runs it (tools/cleanroom/code_check.sh), and the check itself still catches its
# fixtures.
if [ -d crates/clean_types ] || [ -d crates/clean_model ]; then
  out=$(bash tools/cleanroom/code_check.sh . 2>&1) || fail "$out"
fi
out=$(bash tools/cleanroom/code_check_test.sh 2>&1) || fail "$out"

# Rule 6: the public surface of `data` describes data in domain terms. A doc-comment line
# (`///` or `//!`) holding a bank:address pair, a bare four-digit `$XXXX` address or a
# "bank $XX" reference fails when it is part of a `//!` module doc, or is in a run of doc
# lines whose next line is a `pub` item or an enum variant (a capitalised name). The
# addresses belong in a `//` comment in the decoder or on a private item. This approximates
# visibility by the next line: it misses a field of a struct-like enum variant and a `pub`
# item hidden behind attributes other than `#[...]`, and it flags `pub(crate)` items too.
doc_addresses() {
  find crates/data/src -name '*.rs' | sort | while IFS= read -r file; do
    awk -v file="$file" '
      function flush() {
        if (n > 0 && bad && (inner || next_line ~ /^[ \t]*pub[ (]/ \
            || next_line ~ /^[ \t]*[A-Z][A-Za-z0-9_]*[ \t]*([({,=]|$)/))
          print "rule 6 (no address in a public doc): " file ":" first ": " first_line
        n = 0; bad = 0
      }
      /^[ \t]*\/\/[\/!]/ && !/^[ \t]*\/\/\/\// {
        if (n == 0) { inner = ($0 ~ /^[ \t]*\/\/!/) }
        n++
        if ($0 ~ /\$[0-9A-Fa-f][0-9A-Fa-f]:[0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f]|\$[0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f]([^0-9A-Fa-f]|$)|[Bb]ank `?\$[0-9A-Fa-f][0-9A-Fa-f]([^0-9A-Fa-f]|$)/ && !bad) {
          bad = 1; first = FNR; first_line = $0
        }
        next
      }
      n > 0 {
        if ($0 ~ /^[ \t]*#\[/) next
        next_line = $0; flush()
      }
      END { if (n > 0) { next_line = ""; flush() } }
    ' "$file"
  done
}
addr_hits=$(doc_addresses)
[ -n "$addr_hits" ] && fail "$addr_hits"

[ "$bad" -eq 0 ] && echo "CRATE RULES OK"
exit $bad
