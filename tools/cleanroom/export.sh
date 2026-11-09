#!/usr/bin/env bash
# export.sh [-c COMMIT] DOMAIN [DEST]: build the isolated repository a domain's implementer
# works in (MODEL §5, "The clean room is enforced by construction").
#
# The export is a fresh repository holding one commit: the entries of allowlist.txt taken by
# `git archive` from COMMIT (default HEAD), plus the generated files. It has no history, no other
# ref, no remote and no alternates, and DEST (default C:/cleanroom/DOMAIN) must lie outside the
# project's directories. The script ends by asserting all of that.
#
# The clean workspace shape, inside the export and in this repository alike:
#   Cargo.toml                         generated: members below, the root's [workspace.lints]
#   crates/clean_types/                package `clean_types`: the domain types an implementer
#                                      needs (speeds, poses as enums, tables as typed records)
#   crates/clean_model/DOMAIN/         package `clean_model_<domain>`: the domain's model,
#                                      depending only on clean_types and std
#   docs/behaviours/DOMAIN.md          the spec;  scenarios/vectors/DOMAIN.toml  its vectors
#   CLEANROOM.md                       how to work and commit in the export
#   clippy.toml                        generated: tools/cleanroom/clippy.toml, the G5 ceilings
# A clean crate absent from COMMIT is created empty. Nothing of today's model is copied.
#
# Run from the repository (any worktree). CARGO, if set, runs `cargo check` on the export.
set -euo pipefail
. "$(dirname "$0")/lib.sh"

commit=HEAD
[ "${1:-}" = -c ] && { commit=$2; shift 2; }
[ $# -ge 1 ] || die "usage: export.sh [-c COMMIT] DOMAIN [DEST]"
domain=$1
check_domain "$domain"
dest=${2:-C:/cleanroom/$domain}
src=$(git rev-parse --show-toplevel)
sha=$(git rev-parse --verify "$commit^{commit}")

# DEST is outside every worktree of this repository and outside the project's directories
# (the checkouts beside the main one, named sm-decomp*).
mkdir -p "$dest"
dest=$(cd "$dest" && pwd -P)
[ -z "$(ls -A "$dest")" ] || die "$dest is not empty; remove it or choose another DEST"
main=$(cd "$(git rev-parse --path-format=absolute --git-common-dir)/.." && pwd -P)
under() { case "$1/" in "$2"/*) return 0 ;; esac; return 1; }
while IFS= read -r wt; do
  wt=$(cd "$wt" 2>/dev/null && pwd -P) || continue
  under "$dest" "$wt" && die "$dest is inside the worktree $wt"
done < <(git worktree list --porcelain | sed -n 's/^worktree //p')
for proj in "$(dirname "$main")"/sm-decomp*; do
  [ -d "$proj" ] && under "$dest" "$(cd "$proj" && pwd -P)" && die "$dest is inside $proj"
done
temp=$(cygpath -u "${TEMP:-/tmp}" 2>/dev/null || echo "${TEMP:-/tmp}")
[ -d "$temp/collab" ] && under "$dest" "$(cd "$temp/collab" && pwd -P)" && die "$dest is inside $temp/collab"

# Copy the allowlisted entries that exist in COMMIT. A missing spec or vector file stops the
# export unless CLEANROOM_ALLOW_MISSING=1 (a dummy domain).
mapfile -t entries < <(allowed_entries "$domain")
present=()
for entry in "${entries[@]}"; do
  if [ -n "$(git ls-tree -r --name-only "$sha" -- "${entry%/}")" ]; then
    present+=("${entry%/}")
  else
    case $entry in
      crates/clean_*/) ;;
      *) [ "${CLEANROOM_ALLOW_MISSING:-0}" = 1 ] || die "$entry is not in $sha"
         echo "export: $entry is not in $sha; skipped" ;;
    esac
  fi
done
[ ${#present[@]} -gt 0 ] && git archive --format=tar "$sha" -- "${present[@]}" | tar -x -m -C "$dest"

# The empty clean crates, where COMMIT has none. Their lints are the workspace's.
scaffold() { # DIR PACKAGE DESCRIPTION DOC [DEPENDENCY_LINE [DEV_DEPENDENCY_LINE]]
  [ -e "$dest/$1/Cargo.toml" ] && return 0
  mkdir -p "$dest/$1/src"
  {
    printf '[package]\nname = "%s"\nversion = "0.1.0"\nedition = "2024"\nlicense = "MIT"\n' "$2"
    printf 'publish = false\ndescription = "%s"\n\n[dependencies]\n' "$3"
    [ -n "${5:-}" ] && printf '%s\n' "$5"
    [ -n "${6:-}" ] && printf '\n[dev-dependencies]\n%s\n' "$6"
    printf '\n[lints]\nworkspace = true\n'
  } >"$dest/$1/Cargo.toml"
  printf '//! %s\n' "$4" >"$dest/$1/src/lib.rs"
}
# A dev-dependency for each line of allowed_deps.txt (none today: vector tests parse by hand).
vector_parser=$(awk '$1 == "toml" { printf "toml = \"%s\"", $2 }' "$cleanroom_dir/allowed_deps.txt")
scaffold crates/clean_types clean_types "The domain types the clean models share" \
  "The domain types the clean models share: quantities, poses and tables in player terms."
scaffold "crates/clean_model/$domain" "$(model_package "$domain")" \
  "The $domain model, built from its behaviour spec" \
  "The $domain model, built from its behaviour spec alone." \
  'clean_types = { path = "../../clean_types" }' "$vector_parser"

# The workspace manifest: the clean crates and the root manifest's lint set, unchanged.
{
  printf '[workspace]\nresolver = "3"\nmembers = ["crates/clean_types", "crates/clean_model/%s"]\n\n' "$domain"
  git show "$sha:Cargo.toml" | sed -n '/^\[workspace\.lints\.rust\]/,/^\[profile/p' | sed '$d'
} >"$dest/Cargo.toml"
printf '/target\n/Cargo.lock\n' >"$dest/.gitignore"
# The clean crates' clippy configuration: the root's test allowances and the G5 ceilings.
git show "$sha:tools/cleanroom/clippy.toml" >"$dest/clippy.toml"


cat >"$dest/CLEANROOM.md" <<EOF
# Clean room: $domain

This repository is everything you may read. Build the $domain model from
\`docs/behaviours/$domain.md\` and \`scenarios/vectors/$domain.toml\` alone; the rules are in
\`docs/cleanroom/DESIGN.md\` and \`.agents/skills/implementer/SKILL.md\`. Read nothing outside this
directory: no other repository, worktree, temporary folder or web page about the original.
Use nothing you remember about how the original game is programmed (its code, memory layout,
disassemblies or annotated listings); if you recognise something from such a source, report it.

- Write the model in \`crates/clean_model/$domain\` and the types it shares in
  \`crates/clean_types\`. They depend on nothing but each other and std.
- Every commit touches only those two directories, and its message ends with the trailer
  \`Role: implementer\`. Commits that touch anything else are refused on import.
- Commit with one \`-m\` per paragraph, the trailer the last:
  \`git commit -m "Subject" -m "Body paragraph." -m "Role: implementer"\`. The shell rules refuse
  a command holding any of \`; & | < > \$ ~\`, \`..\`, a space or colon before \`/\`, or \`-F\`, so
  write the message without them (no heredoc, no message file).
- Delete or rename a file with \`git rm PATH\` or \`git mv OLD NEW\`, each path relative to this
  directory and inside the two crates.
- Check with \`cargo check\`, \`cargo clippy --all-targets -- -D warnings\` and \`cargo fmt --check\`,
  each as its own command.
- \`cargo test\` builds the tests but cannot run them: this machine blocks the test executables.
  Run the tests with the one command your brief gives, a Python interpreter on a fixed runner
  outside this directory, exactly as written and with nothing added. It links each test target
  of the clean crates as a library, runs it and prints libtest's output. No other Python is
  allowed.
- Report a gap you can't explain, with the vector, the field, the expected and actual values and
  the spec entry you suspect.
EOF

# Nothing in the export mentions the score, the budget, k, the fit or token weights (MODEL §1
# decision 10): the implementer is told the code rules only. score_words.txt holds the patterns.
bash "$cleanroom_dir/score_scan.sh" "$dest" \
  || die "the export mentions the score or the budget (above); fix the source files, not the export"
echo "  ok: no mention of the score or the budget"

# One commit, no remote, no template hooks.
git_isolated "$dest" init -q --template= -b main
git_isolated "$dest" add -A
git_isolated "$dest" -c user.name=cleanroom -c user.email=cleanroom@invalid -c commit.gpgsign=false \
  commit -q -m "Clean-room export: $domain"

# Nothing names the source commit (an `export-subst` attribute would).
! grep -rqF "$sha" --exclude-dir=.git "$dest" || die "the export names the source commit $sha"

echo "export: $domain from ${sha:0:9} into $dest"
assert_isolated "$dest" "$domain"
if [ -n "${CARGO:-}" ]; then
  (cd "$dest" && "$CARGO" check -q --all-targets) || die "cargo check failed in $dest"
  echo "  ok: cargo check"
fi
# The integrator's record of where the export came from; it stays outside the export.
record=${CLEANROOM_PRIVATE:-$HOME/.cleanroom}
mkdir -p "$record"
printf '%s\t%s\t%s\t%s\n' "$(date -u +%FT%TZ)" "$domain" "$sha" "$dest" >>"$record/exports.tsv"
echo "EXPORT OK $domain $dest"
