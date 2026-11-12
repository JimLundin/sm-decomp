# lib.sh: shared by export.sh and import.sh. Source it; it defines functions only.

# The allowlist next to this file, unless CLEANROOM_ALLOWLIST names another.
cleanroom_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
allowlist=${CLEANROOM_ALLOWLIST:-$cleanroom_dir/allowlist.txt}

die() { echo "cleanroom: $*" >&2; exit 1; }

# A domain is a lowercase kebab-case name: it becomes a path and a package name. `types` is the
# shared crate's folder beside the models, so it names no domain.
check_domain() {
  case $1 in
    ''|*[!a-z0-9-]*|-*|*-) die "domain '$1' must be lowercase letters, digits and inner dashes" ;;
    types) die "domain '$1' is the shared crate's folder, crates/clean/types" ;;
  esac
}

# The package name of a domain's model crate, and its folder under crates/clean: the domain,
# dashes as underscores (`world-camera` is `world_camera`). A domain's name starts with its area.
model_package() { echo "$1" | tr - _; }

# The domain a model crate's folder under crates/clean holds: its name, underscores as dashes.
domain_of() { echo "$1" | tr _ -; }

# The model crates a listing of paths (one per line, on stdin) holds: each folder of
# crates/clean/ with a Cargo.toml, but the shared types'.
model_dirs() { sed -n 's#^\(crates/clean/[^/]*\)/Cargo\.toml$#\1#p' | grep -vx crates/clean/types || true; }

# allowlist_section SECTION DOMAIN: the entries of [SECTION], comments and blanks dropped,
# `{domain}` replaced by DOMAIN and `{crate}` by its model_package. A section header is
# `[name]`; `[domain NAME]` holds the entries for one domain only.
allowlist_section() {
  [ -f "$allowlist" ] || die "no allowlist at $allowlist"
  awk -v want="$1" -v domain="$2" -v crate="$(model_package "$2")" '
    /^[ \t]*(#|$)/ { next }
    /^\[.*\]$/ { cur = substr($0, 2, length($0) - 2); next }
    cur == want {
      line = $0; gsub(/\{domain\}/, domain, line); gsub(/\{crate\}/, crate, line)
      sub(/[ \t]+$/, "", line); print line
    }
  ' "$allowlist"
}

# The tree entries an export holds: [all] plus [domain DOMAIN]. An entry ending in `/` is a
# directory and covers everything under it.
allowed_entries() { allowlist_section all "$1"; allowlist_section "domain $1" "$1"; }

# in_entries PATH ENTRY...: whether PATH is one of the entries or under a directory entry.
in_entries() {
  path=$1; shift
  for entry in "$@"; do
    case $entry in
      */) case $path in "$entry"*) return 0 ;; esac ;;
      *) [ "$path" = "$entry" ] && return 0 ;;
    esac
  done
  return 1
}

# A git command that sees only the repository at $1: no inherited GIT_* overrides.
git_isolated() {
  dir=$1; shift
  env -u GIT_DIR -u GIT_WORK_TREE -u GIT_OBJECT_DIRECTORY -u GIT_ALTERNATE_OBJECT_DIRECTORIES \
    -u GIT_INDEX_FILE -u GIT_COMMON_DIR -u GIT_NAMESPACE -u GIT_CEILING_DIRECTORIES \
    git -C "$dir" "$@"
}

# assert_isolated DEST DOMAIN [ALLOW_EXTRA_COMMITS]: the export's isolation, each check
# printed. Exits non-zero on the first failure. With ALLOW_EXTRA_COMMITS=1 (import), the
# implementer's commits on top of the one export commit are allowed, but nothing else.
assert_isolated() {
  dest=$1 domain=$2 extra=${3:-0}
  ok() { echo "  ok: $*"; }
  [ -d "$dest/.git" ] || die "$dest has no .git directory"
  top=$(git_isolated "$dest" rev-parse --show-toplevel) || die "$dest is not a repository"
  [ "$(cd "$top" && pwd -P)" = "$(cd "$dest" && pwd -P)" ] || die "$dest is inside $top"
  common=$(git_isolated "$dest" rev-parse --path-format=absolute --git-common-dir)
  [ "$(cd "$common" && pwd -P)" = "$(cd "$dest/.git" && pwd -P)" ] \
    || die "$dest shares the git directory $common"
  ok "its own repository, git directory $dest/.git"
  [ ! -e "$dest/.git/objects/info/alternates" ] || die "alternates present"
  [ ! -e "$dest/.git/objects/info/http-alternates" ] || die "http-alternates present"
  ok "no alternates"
  [ -z "$(git_isolated "$dest" remote)" ] || die "a remote is configured: $(git_isolated "$dest" remote)"
  ! git_isolated "$dest" config --get-regexp '^(remote|url)\.' >/dev/null || die "remote or url config present"
  ok "no remote"
  refs=$(git_isolated "$dest" for-each-ref --format='%(refname)')
  [ "$refs" = refs/heads/main ] || die "refs other than refs/heads/main: $(echo $refs)"
  ok "one ref, refs/heads/main"
  roots=$(git_isolated "$dest" rev-list --max-parents=0 --all | wc -l)
  [ "$roots" -eq 1 ] || die "$roots root commits"
  count=$(git_isolated "$dest" rev-list --all | wc -l)
  if [ "$extra" = 1 ]; then
    [ -z "$(git_isolated "$dest" rev-list --merges --all)" ] || die "merge commits present"
    ok "one root commit, $count commits, no merges"
  else
    [ "$count" -eq 1 ] || die "rev-list --all shows $count commits"
    ok "rev-list --all shows one commit"
  fi
  # Every object in the store is reachable from that history: nothing smuggled in loose or
  # packed.
  stored=$(git_isolated "$dest" cat-file --batch-all-objects --batch-check='%(objectname)' | sort -u | wc -l)
  reach=$(git_isolated "$dest" rev-list --objects --all | cut -d' ' -f1 | sort -u | wc -l)
  # An implementer's amends leave unreachable objects, and import takes only patches, so this
  # holds for a fresh export alone.
  [ "$extra" = 1 ] && return 0
  [ "$stored" -eq "$reach" ] || die "$stored objects stored, $reach reachable"
  ok "$stored objects stored, all reachable"
  # The tree holds only allowlisted or generated paths, and the work tree only those plus
  # the ignored build outputs.
  mapfile -t entries < <(allowed_entries "$domain"; allowlist_section generated "$domain")
  root=$(git_isolated "$dest" rev-list --max-parents=0 HEAD)
  stray=$(git_isolated "$dest" ls-tree -r --name-only "$root" | while IFS= read -r p; do
    in_entries "$p" "${entries[@]}" || echo "$p"
  done)
  [ -z "$stray" ] || die "files outside the allowlist: $(echo $stray)"
  untracked=$(git_isolated "$dest" ls-files --others --exclude-standard)
  [ -z "$untracked" ] || die "untracked files: $(echo $untracked)"
  ok "$(git_isolated "$dest" ls-tree -r --name-only "$root" | wc -l) files, all on the allowlist"
}
