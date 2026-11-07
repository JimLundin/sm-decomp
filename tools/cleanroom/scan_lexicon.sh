#!/usr/bin/env bash
# scan_lexicon.sh LOGS_DIR... -- WORDS: rebuilds tools/cleanroom/scan_lexicon on the observer
# side, with a fresh salt, from
# - the bank logs' text pages (every *.txt under each LOGS_DIR): routine headings, the RAM they
#   name, and the codes their comments name;
# - verify's sources and decoders (crates/verify/src/**/*.rs) and data's pose list (crates/data/src/pose.rs);
# - the decode notes (docs/behaviours/*.decode.md);
# - WORDS, a file of distinctive single words (one per line), kept outside the repository.
# Only salted hashes are written; none of these inputs enter the repository.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
logs=()
while [ $# -gt 0 ] && [ "$1" != "--" ]; do
  while IFS= read -r f; do logs+=("$f"); done < <(find "$1" -name '*.txt' | sort)
  shift
done
[ "${1:-}" = "--" ] && [ $# -eq 2 ] || { echo "usage: scan_lexicon.sh LOGS_DIR... -- WORDS" >&2; exit 2; }
words="$2"
# source.rs first: the decoders' fields resolve against its WRAM constants.
rust=(crates/verify/src/source.rs)
while IFS= read -r f; do rust+=("$f"); done < <(find crates/verify/src -name '*.rs' ! -name source.rs | sort)
rust+=(crates/data/src/pose.rs)
notes=(docs/behaviours/*.decode.md)
salt=$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')
exec bash tools/cleanroom/scan_tool.sh lexicon tools/cleanroom/scan_lexicon "$salt" \
  --logs "${logs[@]}" --rust "${rust[@]}" --notes "${notes[@]}" --words "$words"
