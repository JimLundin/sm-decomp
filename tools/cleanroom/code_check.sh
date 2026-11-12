#!/usr/bin/env bash
# code_check.sh [--import] DIR...: refuse clean-room crates whose code could reach outside its own
# computation when `cargo test` or `cargo build` runs it (MODEL §5; launch.sh cannot confine
# what cargo runs). Each DIR is a crate directory (it holds a Cargo.toml) or a root holding
# crates/clean/* (the shared types and the models): an export, a git archive of one, or this
# repository. A root with no clean crate passes. Exit 1 prints one line per finding:
#   code_check: PATH: TAG: DETAIL
# A warning, which doesn't fail, is `code_check: warning: PATH: TAG: DETAIL`.
#
# Sources (src/ and tests/ *.rs, comments and string contents removed, whitespace collapsed):
#   std-fs std-net std-process std-env std-os std-io std-glob std-alias std-macro-ident
#     std::fs and the rest, by full path, `use`, group or glob; std::io only as io::Write;
#     `use std as x`, `std::$name`
#   File TcpStream UdpSocket Command            those names, anywhere
#   include-bytes include-path env-macro option-env asm global-asm macro-path
#     include_bytes!, include!/include_str! of anything but a literal path that resolves to
#     <root>/scenarios/vectors/<domain>.toml, env!, option_env!, asm!, global_asm!
#   extern unsafe link path-attr proc-macro     extern, unsafe, #[link], #[path = ...], proc macros
#   lint-attr      (L1) an #[allow] (the workspace denies clippy::allow_attributes); an #[expect],
#     #[allow] or #[warn] without reason = "..."; or, outside test code, one that names a lint
#     this repository's [workspace.lints] denies by name, a denied group, or `warnings`. A lint
#     denied only through a group (pedantic, nursery) may be expected with its reason.
#   magic-literal  (L2) a number other than 0 or 1 in logic. Allowed in test code (tests/,
#     tests.rs, #[cfg(test)] items), attributes, struct/enum/union/type items, a constant's or
#     static's head and value while the value is data (L3, as `tools size` decides: literals,
#     paths, tuples, arrays, struct literals, calls whose every argument is data and ranges
#     whose two ends are literals or paths; a method
#     call, an operator, control flow, a closure or a macro makes it logic), the values of an
#     exhaustive lookup (a guard-free `match` of two or more arms, or an `enum_map!`, each arm a
#     variant, constant or literal key, `=>` and a data value naming no local, as `tools size`
#     counts it; a key's own number is still refused), array lengths and tuple fields.
#   variant-import (L4) a `use` outside test code that imports an enum's variants (`use Kind::A`,
#     `use Kind::{A, B}`, `use Kind::*`). In a crate variant_import_warn.txt lists (crates written
#     before the rule) it is a warning, unless --import, which import.sh passes: an implementer
#     round removes them from every crate its export holds. A listed crate with none left is a
#     `variant-import` finding too, so its line goes. The three rules are lint_scan.awk.
# Manifest: build-key links-key proc-macro build-deps dep-path dep-git dep-workspace
#   dep-registry (not in allowed_deps.txt; or not at its pinned version, of an allowed kind, with
#   allowed features only and no other key than version, features, default-features and package)
#   target-path
#   manifest-section ([patch], [replace], [workspace])
# Files: extra-file for anything in a crate besides Cargo.toml, src/ and tests/ (build.rs,
#   .cargo/), and for a symlink.
# Shapes, over a crate's library sources together (tools/cleanroom/shape_scan.awk):
#   argument-object  (G6) a struct taken by exactly one signature and built only in the argument
#     lists of calls to that function, with no impl, and not a field's, variant's, return,
#     constant's or alias's type or named in a trait: a bag of one call's arguments
#   layering         (G7) a cycle of module references within the crate: modules import in one
#     direction only
#
# What it can't see: see the report of the lane that wrote it; in short, macro-generated paths
# beyond `std::$x`, registry crates the allowlist admits (and their transitive build scripts),
# and anything that runs outside cargo.
#
# Speed: process start-up is the cost on Windows (MSYS and an on-access scanner make each spawn
# 50-500 ms), so a crate costs a handful of processes, not some per file: source_scan.awk runs
# every per-source rule over all of a crate's sources in one awk, shape_scan.awk the shapes in
# another, and the rest is shell built-ins.
set -uo pipefail
here=$(cd "$(dirname "$0")" && pwd)
start=$PWD
importing=0
[ "${1:-}" = --import ] && { importing=1; shift; }
[ $# -ge 1 ] || { echo "usage: code_check.sh [--import] DIR..." >&2; exit 2; }
bad=0
count=0
# The crates whose variant imports still warn; CODE_CHECK_VARIANT_WARN names another list (tests).
warn_list=${CODE_CHECK_VARIANT_WARN:-$here/variant_import_warn.txt}
# Whether the crate being checked only warns of its variant imports, and how many it has.
variant_warn=0
variants=0

hit() { # REL TAG DETAIL
  if [ "$2" = variant-import ]; then
    variants=$((variants + 1))
    [ "$variant_warn" -eq 1 ] && { printf 'code_check: warning: %s: %s: %s\n' "$1" "$2" "$3"; return; }
  fi
  printf 'code_check: %s: %s: %s\n' "$1" "$2" "$3"
  bad=1; count=$((count + 1))
}

include_re='^include(_str)?![([{]"([^"]*)"[])}]'

# The lints this repository denies or forbids by name (clippy's with their prefix), its denied
# groups, and `warnings`, which the clean room denies on the command line. Read from the
# trunk's Cargo.toml, not the checked tree's, so an export can't loosen them.
denied=$(awk '
  /^\[workspace\.lints\.rust\]/ { p = ""; on = 1; next }
  /^\[workspace\.lints\.clippy\]/ { p = "clippy::"; on = 1; next }
  /^\[/ { on = 0 }
  on && /=/ && /"(deny|forbid)"/ { name = $1; sub(/=.*/, "", name); printf "%s%s ", p, name }
  END { print "warnings" }' "$here/../../Cargo.toml")

# The warn list's crates, one per line, trailing blanks and # lines dropped.
warned=()
if [ "$importing" -eq 0 ] && [ -f "$warn_list" ]; then
  while IFS= read -r line || [ -n "$line" ]; do
    line=${line%"${line##*[![:space:]]}"}
    [[ $line == '#'* ]] || warned+=("$line")
  done <"$warn_list"
fi

# An include!/include_str! call in FILE (the call and what follows it): allowed only with a
# literal path to the domain's vector file, or, as text, its spec or a clean crate's source.
check_include() { # FILE REL ROOT DOMAIN CALL
  local f=$1 rel=$2 root=$3 domain=$4 m=$5 lit target want=""
  [ -n "$domain" ] && want=$(realpath -m "$root/scenarios/vectors/$domain.toml")
  if [[ $m =~ $include_re ]]; then
    lit=${BASH_REMATCH[2]}
    target=$(realpath -m "${f%/*}/$lit")
    [ -n "$want" ] && [ "$target" = "$want" ] && return
    # A test may read its domain's spec as text (the catalogue's checked lines): the spec is
    # the implementer's own input, exported to it.
    [ -n "$domain" ] && [[ $m == include_str!* ]] &&
      [ "$target" = "$(realpath -m "$root/docs/behaviours/$domain.md")" ] && return
    # A test may read the clean crates' own Rust sources as text (a size check): they hold
    # nothing from outside the clean room.
    if [[ $m == include_str!* && $target == *.rs ]]; then
      case $target in
        "$(realpath -m "$root/crates/clean")"/*/src/*) return ;;
      esac
    fi
    hit "$rel" include-path "\"$lit\" is not \"$domain\"'s vector file, its spec or a clean crate's source"
  else
    hit "$rel" include-path "${m:0:60} is not a literal path"
  fi
}

check_crate() { # CRATE_DIR
  local crate root domain rel p t allowed w tag detail sources=() library=() paths=()
  cd "$start" && cd "$1" && crate=$PWD
  case $crate in
    */crates/clean/types) root=${crate%/crates/clean/types}; domain="" ;;
    */crates/clean/*) root=${crate%/crates/clean/*}; domain=${crate##*/}; domain=${domain//_/-} ;;
    *) root=$crate; domain="" ;;
  esac
  rel=${crate#"$root"/}; [ "$rel" = "$crate" ] && rel=${crate##*/}
  variants=0; variant_warn=0
  for w in "${warned[@]}"; do [ "$w" = "$rel" ] && variant_warn=1; done
  allowed="$root/crates/clean/types"
  [ -n "$domain" ] && allowed="$allowed|$crate"

  [ -f Cargo.toml ] || hit "$rel" extra-file "no Cargo.toml"
  if [ -f Cargo.toml ]; then
    while IFS=$'\t' read -r tag detail; do
      [ -n "$tag" ] && hit "$rel/Cargo.toml" "$tag" "$detail"
    done < <(awk -v crate="$crate" -v allowed="$allowed" -v allowfile="$here/allowed_deps.txt" \
      -f "$here/check_manifest.awk" Cargo.toml)
  fi

  # One listing serves every rule below: what the crate holds, its sources and its library.
  while IFS= read -r p; do paths+=("${p#./}"); done < <(find . \( -type f -o -type l \) | sort)
  for p in "${paths[@]}"; do
    case $p in
      Cargo.toml|src/*|tests/*) ;;
      *) hit "$rel" extra-file "$p" ;;
    esac
  done
  for p in "${paths[@]}"; do [ -L "$p" ] && hit "$rel" extra-file "$p is a symlink"; done
  for p in "${paths[@]}"; do
    [[ $p == src/*.rs || $p == tests/*.rs ]] && [ -f "$p" ] && [ ! -L "$p" ] || continue
    sources+=("$p")
    # Crate-wide shapes (G6, G7) are over the library's sources: test files are not its shape.
    [[ $p == src/* && ${p##*/} != tests.rs && $p != */tests/* ]] && library+=("$p")
  done

  if [ "${#sources[@]}" -gt 0 ]; then
    while IFS=$'\t' read -r p tag detail; do
      if [ "$tag" = '?include' ]; then check_include "$crate/$p" "$rel/$p" "$root" "$domain" "$detail"
      else hit "$rel/$p" "$tag" "$detail"; fi
    done < <(awk -v denied="$denied" -f "$here/strip_rust.awk" -f "$here/lint_scan.awk" \
      -f "$here/source_scan.awk" "${sources[@]}")
  fi
  if [ "${#library[@]}" -gt 0 ]; then
    while IFS=$'\t' read -r tag p detail; do
      [ -n "$tag" ] && hit "$rel/src/$p" "$tag" "$detail"
    done < <(awk -f "$here/strip_rust.awk" -f "$here/shape_scan.awk" "${library[@]}")
  fi
  if [ "$variant_warn" -eq 1 ] && [ "$variants" -eq 0 ]; then
    variant_warn=0
    hit "$rel" variant-import "listed in ${warn_list##*/} with no variant import left: delete its line"
  fi
  cd "$start" || exit 2
}

for dir in "$@"; do
  [ -d "$dir" ] || { echo "code_check: no directory $dir" >&2; exit 2; }
  if [ -f "$dir/Cargo.toml" ]; then
    check_crate "$dir"
  else
    for m in "$dir"/crates/clean/*/; do
      [ -d "$m" ] && check_crate "$m"
    done
  fi
done
if [ "$bad" -eq 0 ]; then
  echo "CODE CHECK OK"
else
  echo "CODE CHECK FAILED: $count findings"
fi
exit $bad
