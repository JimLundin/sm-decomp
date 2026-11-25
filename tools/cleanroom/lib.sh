# lib.sh: shared by export.sh and import.sh. Source it; it defines functions and one array.

# The allowlist next to this file, unless CLEANROOM_ALLOWLIST names another.
cleanroom_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
allowlist=${CLEANROOM_ALLOWLIST:-$cleanroom_dir/allowlist.txt}

die() { echo "cleanroom: $*" >&2; exit 1; }

# Clippy's arguments for the capability rule alone: every lint allowed but those the workspace
# forbids (the clean room's disallowed paths in clippy.toml, unsafe code), which no flag lifts.
# responses_controller.py's `capability` command passes the same.
capability_lints=(-A warnings -A clippy::all -A clippy::pedantic -A clippy::nursery
  -A clippy::restriction -A missing_docs)

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

# A git command that sees only the repository at $1: no inherited GIT_* overrides, and no
# configuration the caller's environment carries (GIT_CONFIG_COUNT and its keys, or `git -c` of
# a parent git), which belongs to the caller's session and not to the repository. No system or
# user configuration either, and the programs a repository's own configuration can name for these
# commands (hooks, fsmonitor, signature checks, a pager) are switched off on the command line,
# which wins over it. A repository a session could have written is still read through
# export_mirror, which leaves its configuration behind.
git_isolated() (
  dir=$1; shift
  unset GIT_DIR GIT_WORK_TREE GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES GIT_INDEX_FILE \
    GIT_COMMON_DIR GIT_NAMESPACE GIT_CEILING_DIRECTORIES GIT_CONFIG_COUNT GIT_CONFIG_PARAMETERS \
    GIT_EXTERNAL_DIFF
  export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_ATTR_NOSYSTEM=1 GIT_PAGER=cat \
    GIT_NO_REPLACE_OBJECTS=1
  git -c core.hooksPath=/dev/null -c core.fsmonitor=false -c log.showSignature=false \
    -c core.attributesFile=/dev/null -c protocol.allow=never -C "$dir" "$@"
)

# The entries a session's git leaves at the top of an export's .git. Anything else (commondir,
# config.worktree, gitdir, worktrees/, shallow, modules/) points git elsewhere or changes what it
# reads, and is refused.
export_git_entries=" HEAD config description index index.lock info hooks objects refs logs packed-refs ORIG_HEAD COMMIT_EDITMSG branches "

# export_git_odd DIR [ALSO...]: each thing in the export DIR that a session's ordinary git work
# doesn't leave, one a line: a .git that is not a plain directory, an entry at its top outside
# export_git_entries, alternates, anything in branches/, a ref but refs/heads/main (a replace ref
# among them), and any other repository (a .git, a folder holding HEAD and objects/, or a HEAD
# that is a symlink) in DIR or in the ALSO folders.
# Prints nothing for a clean export. A host tool that runs git where the box wrote (an editor's
# git status, a shell prompt) would read these, so launch.sh reports them as a breach.
export_git_odd() {
  local dir=$1 src=$1/.git e name ref
  shift
  if [ -L "$src" ] || [ ! -d "$src" ]; then echo "$src is not a directory"; return 0; fi
  for e in "$src"/* "$src"/.[!.]* "$src"/..?*; do
    [ -e "$e" ] || [ -L "$e" ] || continue
    name=${e##*/}
    case $export_git_entries in *" $name "*) ;; *) echo "$e" ;; esac
  done
  for e in "$src/objects/info/alternates" "$src/objects/info/http-alternates"; do
    [ ! -e "$e" ] || echo "$e"
  done
  # branches/ is an old way to name remotes; git leaves it empty or absent.
  if [ -d "$src/branches" ] && [ -n "$(ls -A "$src/branches")" ]; then echo "$src/branches"; fi
  if [ -d "$src/refs" ]; then
    find "$src/refs" ! -type d ! -path "$src/refs/heads/main" -print
  fi
  if [ -f "$src/packed-refs" ]; then
    while read -r _ ref; do
      case $ref in '' | refs/heads/main) ;; *) echo "$src/packed-refs: $ref" ;; esac
    done < <(grep -v '^[#^]' "$src/packed-refs")
  fi
  for d in "$dir" "$@"; do
    [ -d "$d" ] || continue
    find "$d" -path "$src" -prune -o -name .git -print -o \( -type f -o -type l \) -name HEAD -print |
      while IFS= read -r e; do
        case ${e##*/} in
          .git) echo "$e" ;;
          HEAD) [ ! -d "${e%/HEAD}/objects" ] && [ ! -L "$e" ] || echo "${e%/HEAD}" ;;
        esac
      done
  done
}

# export_mirror DIR MIRROR: a bare repository at MIRROR (new) holding DIR/.git's objects, refs
# (but replace refs) and HEAD and nothing else, under a configuration of its own. What a session
# could write in .git to make a later git on the host run a program or read elsewhere (config,
# info/attributes, hooks, grafts, replace refs) stays behind. A .git that isn't a directory, an
# entry that points git elsewhere (commondir, alternates and the rest export_git_odd names),
# anything in it but files and directories (a symlink, a fifo), or objects over
# CLEANROOM_MIRROR_MAX_MB (default 1024) stop it.
export_mirror() {
  local src=$1/.git odd size max=${CLEANROOM_MIRROR_MAX_MB:-1024}
  [ -d "$src" ] && [ ! -L "$src" ] || die "$1/.git is not a directory"
  [ -d "$src/objects" ] && [ -f "$src/HEAD" ] || die "$src holds no objects or HEAD"
  odd=$(find "$src" ! -type f ! -type d -print -quit) || die "can't list $src"
  [ -z "$odd" ] || die "$src holds something other than files and folders: $odd"
  for odd in "$src"/* "$src"/.[!.]* "$src"/..?* "$src/objects/info/alternates" "$src/objects/info/http-alternates"; do
    [ -e "$odd" ] || continue
    case $odd in
      */objects/info/*) die "$src holds $odd, which points git elsewhere" ;;
      *) case $export_git_entries in *" ${odd##*/} "*) ;; *) die "$src holds ${odd##*/}, which no session's git leaves" ;; esac ;;
    esac
  done
  size=$(du -sm "$src/objects" | cut -f1) || die "can't measure $src/objects"
  [ "$size" -le "$max" ] || die "$src/objects holds $size MB, over CLEANROOM_MIRROR_MAX_MB=$max"
  env GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null git init -q --bare --template= "$2" >/dev/null ||
    die "can't make a mirror at $2"
  cp -R "$src/objects/." "$2/objects/" || die "can't copy $src/objects"
  if [ -d "$src/refs" ]; then cp -R "$src/refs/." "$2/refs/" || die "can't copy $src/refs"; fi
  rm -rf "$2/refs/replace"
  if [ -f "$src/packed-refs" ]; then grep -v ' refs/replace/' "$src/packed-refs" >"$2/packed-refs" || true; fi
  cp "$src/HEAD" "$2/HEAD"
}

# outside_home_temp: the temporary folder ($TEMP, else $TMPDIR, else /tmp), or /tmp when that
# is in the home. The self-tests keep the repositories they make there: host tools open and fetch
# the repositories they find under the home (VS Code's git extension runs `git fetch` in each one
# it opens in its workspace), which adds to their .git what export_mirror and export_git_odd check.
outside_home_temp() {
  local base=${TEMP:-${TMPDIR:-/tmp}}
  case "$(cd "$base" && pwd -P)/" in "$(cd "$HOME" && pwd -P)"/*) base=/tmp ;; esac
  printf '%s\n' "$base"
}

# export_remotes DIR: the remote and url lines of DIR/.git/config, read as a file (no includes,
# nothing run).
export_remotes() {
  GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null git config --file "$1/.git/config" \
    --get-regexp '^(remote|url)\.' || true
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
