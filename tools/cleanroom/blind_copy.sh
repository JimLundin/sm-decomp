#!/usr/bin/env bash
# blind_copy.sh EXPORT DEST: turn a clean-room export into the code-only copy a blind reviewer
# reads (PROCESS, the blind review step), as a fresh repository at DEST holding one commit.
#
# From EXPORT's committed tree (an export from export.sh, with or without an implementer's
# commits) it keeps the clean crates, the vector file and the export's workspace files, and
#   - removes docs/, CLEANROOM.md and .agents/: the spec, the design note and the implementer's
#     skill are what the blind reviewer must not see;
#   - adds this repository's blind-reviewer skill, .agents/skills/blind-reviewer/SKILL.md;
#   - leaves out each test file under a crate's tests/ that includes a file from docs/ (a test
#     that reads the spec as text), since it can't compile without it. A docs/ include anywhere
#     else, or a manifest's [[test]] naming a file it left out, stops the copy: that crate can't
#     be made to build here.
# DEST must not exist or be empty. CARGO, if set, runs `cargo check --all-targets` in the copy.
# Prints each file left out, then `BLIND COPY OK DEST`.
set -euo pipefail
. "$(dirname "$0")/lib.sh"

[ $# -eq 2 ] || die "usage: blind_copy.sh EXPORT DEST"
export_dir=$1 dest=$2
skill="$cleanroom_dir/../../.agents/skills/blind-reviewer/SKILL.md"
[ -f "$skill" ] || die "no blind-reviewer skill at $skill"
git_isolated "$export_dir" rev-parse -q --verify HEAD >/dev/null || die "$export_dir is not a repository with a commit"
mkdir -p "$dest"
[ -z "$(ls -A "$dest")" ] || die "$dest is not empty; remove it or choose another DEST"

git_isolated "$export_dir" archive --format=tar HEAD | tar -x -m -C "$dest"
rm -rf -- "$dest/docs" "$dest/CLEANROOM.md" "$dest/.agents"
mkdir -p "$dest/.agents/skills/blind-reviewer"
cp -- "$skill" "$dest/.agents/skills/blind-reviewer/SKILL.md"

# A docs/ path inside an include macro, with any number of ../ before it.
reads_docs='include(_str|_bytes)?!\s*\(\s*"([^"]*/)?docs/'
left=()
while IFS= read -r f; do
  rel=${f#"$dest"/}
  case $rel in
    crates/*/tests/*.rs | crates/clean_model/*/tests/*.rs) left+=("$rel") ;;
    *) die "$rel includes a file from docs/ outside a tests/ directory; the copy can't build without it" ;;
  esac
done < <(grep -rlE "$reads_docs" --include='*.rs' "$dest/crates" 2>/dev/null || true)
for rel in "${left[@]}"; do
  name=$(basename "$rel")
  manifest="$dest/${rel%%/tests/*}/Cargo.toml"
  grep -qE "^path *= *\"tests/$name\"" "$manifest" 2>/dev/null &&
    die "${manifest#"$dest"/} names tests/$name in a [[test]], which reads docs/"
  rm -- "$dest/$rel"
  echo "blind copy: left out $rel (it reads docs/)"
done

git_isolated "$dest" init -q --template= -b main
git_isolated "$dest" -c core.autocrlf=false add -A
git_isolated "$dest" -c user.name=cleanroom -c user.email=cleanroom@invalid -c commit.gpgsign=false \
  commit -q -m "Blind review copy"
if [ -n "${CARGO:-}" ]; then
  (cd "$dest" && "$CARGO" check -q --all-targets) || die "cargo check failed in $dest"
  echo "  ok: cargo check"
fi
echo "BLIND COPY OK $dest"
