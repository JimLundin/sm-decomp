#!/usr/bin/env bash
# export.sh [-c COMMIT] DOMAIN [DEST]: build the isolated repository a domain's implementer
# works in (MODEL §5, "The clean room is enforced by construction").
#
# The export is a fresh repository holding one commit: the entries of allowlist.txt taken by
# `git archive` from COMMIT (default HEAD), plus the generated files. It has no history, no other
# ref, no remote and no alternates, and DEST must lie outside the project's directories. The
# script ends by asserting all of that. DEST defaults to ROOT/DOMAIN, ROOT the exports' root:
# $CLEANROOM_ROOT, default /cleanroom. DEST must lie
# outside the home: host tools run git in the repositories they find there (an editor's source
# control runs `git status` under its workspace), and a session writes the export.
#
# The clean workspace shape, inside the export and in this repository alike:
#   Cargo.toml                         generated: clean members, shared package settings, dependencies and lints
#   crates/clean/types/                package `types`: the domain types an implementer
#                                      needs (speeds, poses as enums, tables as typed records)
#   crates/clean/CRATE/                package CRATE, the domain with dashes as underscores:
#                                      the domain's model, depending on types, std and
#                                      allowed_deps.txt
#   docs/behaviours/DOMAIN.md          the spec;  scenarios/vectors/DOMAIN.toml  its vectors
#   CLEANROOM.md                       how to work and commit in the export
#   clippy.toml                        generated: the selected revision's clean Clippy configuration, the G5 ceilings
#   Cargo.lock, .cargo/config.toml     generated: COMMIT's lock pruned to the export; offline
#   vendor/crates-io/                  generated: the locked registry sources for offline builds
# A clean crate absent from COMMIT is created empty. Nothing of today's model is copied.
#
# Run from the repository (any worktree). CARGO (default `cargo`) resolves and vendors the
# export's crates, reaching crates.io for any the download cache lacks; if CARGO is set, it also
# runs `cargo check` on the export.
set -euo pipefail
. "$(dirname "$0")/lib.sh"

commit=HEAD
[ "${1:-}" = -c ] && { commit=$2; shift 2; }
[ $# -ge 1 ] || die "usage: export.sh [-c COMMIT] DOMAIN [DEST]"
domain=$1
check_domain "$domain"
dest=${2:-${CLEANROOM_ROOT:-/cleanroom}/$domain}
src=$(git rev-parse --show-toplevel)
sha=$(git rev-parse --verify "$commit^{commit}")

# Older workspaces patched registry sources with local files. Only their exporter knows
# which copies to carry alongside their lock.
if git grep -qE '^[[:space:]]*\[patch\.crates-io\][[:space:]]*(#.*)?$' "$sha" -- Cargo.toml; then
  die "$sha patches crates.io; run tools/cleanroom/export.sh from a worktree checked out at $sha"
fi

# DEST is outside every worktree of this repository and outside the project's directories
# (the checkouts beside the main one, named sm-decomp*).
mkdir -p "$dest"
dest=$(cd "$dest" && pwd -P)
[ -z "$(ls -A "$dest")" ] || die "$dest is not empty; remove it or choose another DEST"
main=$(cd "$(git rev-parse --path-format=absolute --git-common-dir)/.." && pwd -P)
under() { case "$1/" in "$2"/*) return 0 ;; esac; return 1; }
if under "$dest" "$(cd "$HOME" && pwd -P)"; then
  rmdir "$dest" 2>/dev/null || true
  die "$dest is in the home; exports go under a root outside it (CLEANROOM_ROOT, default /cleanroom)"
fi
while IFS= read -r wt; do
  wt=$(cd "$wt" 2>/dev/null && pwd -P) || continue
  under "$dest" "$wt" && die "$dest is inside the worktree $wt"
done < <(git worktree list --porcelain | sed -n 's/^worktree //p')
for proj in "$(dirname "$main")"/sm-decomp*; do
  [ -d "$proj" ] && under "$dest" "$(cd "$proj" && pwd -P)" && die "$dest is inside $proj"
done
temp=${TEMP:-${TMPDIR:-/tmp}}
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
      crates/clean/*/) ;;
      *) [ "${CLEANROOM_ALLOW_MISSING:-0}" = 1 ] || die "$entry is not in $sha"
         echo "export: $entry is not in $sha; skipped" ;;
    esac
  fi
done
[ ${#present[@]} -gt 0 ] && git archive --format=tar "$sha" -- "${present[@]}" | tar -x -m -C "$dest"

# Supply the workspace tables the clean crates use and scaffold any missing package.
crate=$(model_package "$domain")
"${SM_PY:-python3}" -I "$cleanroom_dir/export_workspace.py" "$sha" "$dest" "$domain" "$crate"
printf '/target\n' >"$dest/.gitignore"

# The lock and the registry crates, so cargo resolves offline to the versions this repository
# locked: COMMIT's Cargo.lock, pruned to the export's crates, and every registry crate it then
# names copied into vendor/crates-io by `cargo vendor`, which cargo checks against the lock's
# checksums. Unlike cargo's download cache, the copies can't be cleaned away or differ between
# machines.
cargo=${CARGO:-cargo}
git show "$sha:Cargo.lock" >"$dest/Cargo.lock"
(cd "$dest" && "$cargo" metadata -q --format-version 1 >/dev/null) \
  || die "cargo can't resolve the export from $sha's Cargo.lock"
mkdir -p "$dest/.cargo"
printf '# Cargo never reaches the network here: the lock and vendor/ hold every crate.\n[net]\noffline = true\n' \
  >"$dest/.cargo/config.toml"
if grep -q '^source = "registry+' "$dest/Cargo.lock"; then
  (cd "$dest" && "$cargo" vendor -q --locked vendor/crates-io >/dev/null) || die "cargo vendor failed in $dest"
  printf '\n[source.crates-io]\nreplace-with = "vendored"\n\n[source.vendored]\ndirectory = "vendor/crates-io"\n' \
    >>"$dest/.cargo/config.toml"
fi
# The clean crates' clippy configuration: the root's test allowances and the G5 ceilings.
clippy=crates/clean/.clippy.toml
if ! git cat-file -e "$sha:$clippy" 2>/dev/null; then clippy=tools/cleanroom/clippy.toml; fi
git show "$sha:$clippy" >"$dest/clippy.toml"

tests_bullet="- Run the tests with the one command your brief gives, a fixed runner outside this directory,
  exactly as written and with nothing added. It checks the module and argument rules
  in DESIGN.md, then runs \`cargo test\` for each test target of the
  clean crates and prints libtest's output. A broken rule fails the run as a failed test does.
  No Python is allowed."

cat >"$dest/CLEANROOM.md" <<EOF
# Clean room: $domain

This repository is everything you may read. Build the $domain model from
\`docs/behaviours/$domain.md\` and \`scenarios/vectors/$domain.toml\` alone; the rules are in
\`docs/cleanroom/DESIGN.md\` and \`.agents/skills/implementer/SKILL.md\`. Read nothing outside this
directory: no other repository, worktree, temporary folder or web page about the original.
Use nothing you remember about how the original game is programmed (its code, memory layout,
disassemblies or annotated listings); if you recognise something from such a source, report it.

- Write the model in \`crates/clean/$crate\` and the types it shares in
  \`crates/clean/types\`. They depend on each other, std and the registry crates their
  manifests name. The manifests are not yours to edit: if a widely used crate would model the
  domain better, ask for it in your report. Cargo works offline here.
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
$tests_bullet
- Report a gap you can't explain, with the vector, the field, the expected and actual values and
  the spec entry you suspect.
EOF

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
