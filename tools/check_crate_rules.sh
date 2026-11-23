#!/usr/bin/env bash
# check_crate_rules.sh: the crate plan's rules (docs/architecture/ARCHITECTURE.md §4.2; PRINCIPLES
# K1, K8, K9) that no standard tool holds, over the new crates' sources. Exit 1 lists each
# violation. The rest are standard tools: the dependency rules and build scripts are
# `cargo deny check bans` (deny.toml); `#![feature]`, `allow`, `cfg(verify)` and the clean room's
# capability rule are the workspace's forbidden lints (Cargo.toml, tools/cleanroom/clippy.toml).
# Run from the repository root.
set -u
engine="data game runtime app"
bad=0
fail() { echo "$1"; bad=1; }

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
  # Rule 4: no verification hooks in the engine (P6). `cfg(verify)` is the forbidden
  # unexpected_cfgs; a probe or an observer is a name, which no lint sees.
  scan "rule 4 (no probe or observer)" '[Pp]robes?\b|[Oo]bservers?\b' $engine
  # Rule 5 (K1): SNES addresses. Long WRAM ($7E/$7F), a bank:address pair, a LoROM long
  # address (banks $80-$DF, offsets $8000-$FFFF), and the PPU, CPU and DMA registers.
  # A bank-log reference in a comment is allowed (K9).
  scan "rule 5 (no SNES address literal)" \
    '0x7[EeFf]_?[0-9A-Fa-f]{4}\b|\$[0-9A-Fa-f]{2}:?[0-9A-Fa-f]{4}\b|0x[89A-Da-d][0-9A-Fa-f]_?[89A-Fa-f][0-9A-Fa-f]{3}\b|0x(21|42)[0-9A-Fa-f]{2}\b|0x43[0-7][0-9A-Fa-f]\b' \
    game runtime app
  # K8: no `expect` in the engine. `allow` and `#![feature]` are forbidden lints, but an
  # `expect` with its reason is a lint level the clean crates may use, so it is a name here.
  scan "K8 (no expect)" '#!?\[[[:space:]]*expect[[:space:]]*\(' $engine
  # K9: no routine labels for names.
  scan "K9 (no routine-label names)" '\b(func|sub|loc)_[0-9A-Fa-f]{2}_?[0-9A-Fa-f]{4}\b' $engine
)
[ -n "$hits" ] && fail "$hits"

# The clean-room crates: what clippy can't see before cargo runs them (tools/cleanroom/code_check.sh).
if [ -d crates/clean ]; then
  out=$(bash tools/cleanroom/code_check.sh crates/clean/* 2>&1) || fail "$out"
fi

# What an export of each specced domain would carry, and the blind reviewer's skill, mention no
# score or budget (MODEL §1 decision 10); export.sh checks the export itself too.
facing=$(
  . tools/cleanroom/lib.sh
  for spec in docs/behaviours/*.md; do
    case $spec in *.decode.md|*.gaps.md) continue ;; esac
    allowed_entries "$(basename "$spec" .md)"
  done
  echo .agents/skills/blind-reviewer/SKILL.md
  echo tools/cleanroom/clippy.toml
)
mapfile -t facing < <(sort -u <<<"$facing" | while IFS= read -r p; do [ -e "${p%/}" ] && echo "${p%/}"; done)
out=$(bash tools/cleanroom/score_scan.sh "${facing[@]}" 2>&1) || fail "$out"

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
