#!/usr/bin/env bash
# code_check.sh DIR...: refuse clean-room crates whose code could reach outside its own
# computation when `cargo test` or `cargo build` runs it (MODEL §5; launch.sh cannot confine
# what cargo runs). Each DIR is a crate directory (it holds a Cargo.toml) or a root holding
# crates/clean_types and crates/clean_model/*: an export, a git archive of one, or this
# repository. A root with no clean crate passes. Exit 1 prints one line per finding:
#   code_check: PATH: TAG: DETAIL
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
# Manifest: build-key links-key proc-macro build-deps dep-path dep-git dep-workspace
#   dep-registry (not in allowed_deps.txt, or not at its pinned version) target-path
#   manifest-section ([patch], [replace], [workspace])
# Files: extra-file for anything in a crate besides Cargo.toml, src/ and tests/ (build.rs,
#   .cargo/), and for a symlink.
# Shapes, over a crate's library sources together (tools/cleanroom/shape_scan.awk):
#   argument-object  (G6) a struct taken by exactly one signature and built only in the argument
#     lists of calls to that function, with no impl, and not a field's, variant's, return,
#     constant's or alias's type or named in a trait: a bag of one call's arguments
#
# What it can't see: see the report of the lane that wrote it; in short, macro-generated paths
# beyond `std::$x`, registry crates the allowlist admits (and their transitive build scripts),
# and anything that runs outside cargo.
set -uo pipefail
here=$(cd "$(dirname "$0")" && pwd)
[ $# -ge 1 ] || { echo "usage: code_check.sh DIR..." >&2; exit 2; }
bad=0
count=0

hit() { # REL TAG DETAIL
  printf 'code_check: %s: %s: %s\n' "$1" "$2" "$3"
  bad=1; count=$((count + 1))
}

# Whitespace collapsed, and removed around punctuation, so `std ::\n fs` is `std::fs`.
squash() { tr '\n\r\t' '   ' | sed -E 's/ +/ /g; s/ ?(::|[]{}(),;!#*[]) ?/\1/g'; }

include_re='^include(_str)?![([{]"([^"]*)"[])}]'

scan_source() { # FILE REL ROOT DOMAIN
  local f=$1 rel=$2 root=$3 domain=$4 b k m g rest lit want
  b=$(awk -f "$here/strip_rust.awk" "$f" | squash)
  k=$(awk -v keep=1 -f "$here/strip_rust.awk" "$f" | squash)

  while IFS= read -r m; do hit "$rel" "std-${m#std::}" "$m"; done \
    < <(grep -oE 'std::(fs|net|process|env|os)\b' <<<"$b")
  while IFS= read -r m; do
    case ${m//,\}/\}} in
      std::io::Write|'std::io::{Write}') ;;
      *) hit "$rel" std-io "$m" ;;
    esac
  done < <(grep -oE 'std::io(::\{[^}]*\}|::[A-Za-z_*]+|[^A-Za-z0-9_]|$)' <<<"$b")
  while IFS= read -r g; do
    rest=$(sed -E 's/io::\{Write,?\}//g; s/io::Write\b//g' <<<"$g")
    while IFS= read -r m; do hit "$rel" "std-${m//[^a-z*]/}" "$g"; done \
      < <(grep -oE '[{,](fs|net|process|env|os|io|\*)([^A-Za-z0-9_]|$)' <<<"$rest")
    grep -qE '[{,]self as ' <<<"$rest" && hit "$rel" std-alias "$g"
  done < <(grep -oE 'std::\{[^;]*' <<<"$b")
  while IFS= read -r m; do hit "$rel" std-glob "$m"; done < <(grep -oE 'std::\*' <<<"$b")
  while IFS= read -r m; do hit "$rel" std-alias "$m"; done < <(grep -oE '\bstd as [A-Za-z_]+' <<<"$b")
  while IFS= read -r m; do hit "$rel" std-macro-ident "$m"; done < <(grep -oE 'std::\$[A-Za-z_]+' <<<"$b")

  while IFS= read -r m; do hit "$rel" "$m" "name used"; done \
    < <(grep -oE '\b(File|TcpStream|UdpSocket|Command)\b' <<<"$b")

  while IFS= read -r m; do hit "$rel" "$(tr _ - <<<"$m")" "$m"; done \
    < <(grep -oE '\b(include_bytes|global_asm|option_env)\b' <<<"$b")
  while IFS= read -r m; do hit "$rel" asm "$m"; done < <(grep -oE '[A-Za-z_]*asm!' <<<"$b")
  while IFS= read -r m; do hit "$rel" env-macro "$m"; done < <(grep -oE '\benv!' <<<"$b")
  while IFS= read -r m; do hit "$rel" macro-path "$m"; done \
    < <(grep -oE '(std|core|alloc)::(include_str|include|asm)\b|core::env\b' <<<"$b")
  want=""
  [ -n "$domain" ] && want=$(realpath -m "$root/scenarios/vectors/$domain.toml")
  while IFS= read -r m; do
    if [[ $m =~ $include_re ]]; then
      lit=${BASH_REMATCH[2]}
      target=$(realpath -m "$(dirname "$f")/$lit")
      [ -n "$want" ] && [ "$target" = "$want" ] && continue
      # A test may read the clean crates' own Rust sources as text (a size check): they hold
      # nothing from outside the clean room.
      if [[ $m == include_str!* && $target == *.rs ]]; then
        case $target in
          "$(realpath -m "$root/crates/clean_types/src")"/* | \
            "$(realpath -m "$root/crates/clean_model")"/*/src/*) continue ;;
        esac
      fi
      hit "$rel" include-path "\"$lit\" is not \"$domain\"'s vector file or a clean crate's source"
    else
      hit "$rel" include-path "${m:0:60} is not a literal path"
    fi
  done < <(grep -oE '\binclude(_str)?![(\[{].{0,200}' <<<"$k")

  while IFS= read -r m; do hit "$rel" extern "$m"; done < <(grep -oE '\bextern\b' <<<"$b")
  while IFS= read -r m; do hit "$rel" unsafe "$m"; done < <(grep -oE '\bunsafe\b' <<<"$b")
  while IFS= read -r m; do hit "$rel" link "$m"; done \
    < <(grep -oE '#!?\[[^]]*\blink\(|\blink_name\b|\blink_section\b|\bexport_name\b|\bno_mangle\b|#!?\[link\b' <<<"$b")
  while IFS= read -r m; do hit "$rel" path-attr "$m"; done < <(grep -oE '#!?\[[^]]*\bpath ?=' <<<"$b")
  while IFS= read -r m; do hit "$rel" proc-macro "$m"; done < <(grep -oE '\bproc[-_]macro\w*' <<<"$b")
}

check_crate() { # CRATE_DIR
  local crate root domain rel f p allowed
  crate=$(cd "$1" && pwd)
  case $crate in
    */crates/clean_model/*) root=${crate%/crates/clean_model/*}; domain=${crate##*/} ;;
    */crates/clean_types) root=${crate%/crates/clean_types}; domain="" ;;
    *) root=$crate; domain="" ;;
  esac
  rel=${crate#"$root"/}; [ "$rel" = "$crate" ] && rel=$(basename "$crate")
  allowed="$root/crates/clean_types"
  [ -n "$domain" ] && allowed="$allowed|$root/crates/clean_model/$domain"

  [ -f "$crate/Cargo.toml" ] || hit "$rel" extra-file "no Cargo.toml"
  if [ -f "$crate/Cargo.toml" ]; then
    while IFS=$'\t' read -r tag detail; do
      [ -n "$tag" ] && hit "$rel/Cargo.toml" "$tag" "$detail"
    done < <(awk -v crate="$crate" -v allowed="$allowed" -v allowfile="$here/allowed_deps.txt" \
      -f "$here/check_manifest.awk" "$crate/Cargo.toml")
  fi

  while IFS= read -r p; do
    case $p in
      Cargo.toml|src/*|tests/*) ;;
      *) hit "$rel" extra-file "$p" ;;
    esac
  done < <(cd "$crate" && find . \( -type f -o -type l \) | sed 's#^\./##' | sort)
  while IFS= read -r p; do hit "$rel" extra-file "$p is a symlink"; done \
    < <(cd "$crate" && find . -type l | sed 's#^\./##')

  while IFS= read -r f; do
    scan_source "$crate/$f" "$rel/$f" "$root" "$domain"
  done < <(cd "$crate" && find src tests -type f -name '*.rs' 2>/dev/null | sort)

  # Crate-wide shapes (G6) over the library's sources: test files are not its shape.
  while IFS=$'\t' read -r tag p detail; do
    [ -n "$tag" ] && hit "$rel/src/$p" "$tag" "$detail"
  done < <(cd "$crate" && find src -type f -name '*.rs' ! -name tests.rs ! -path '*/tests/*' 2>/dev/null | sort |
    while IFS= read -r f; do printf '\001FILE %s\n' "${f#src/}"; awk -f "$here/strip_rust.awk" "$f"; done |
    awk -f "$here/shape_scan.awk")
}

for dir in "$@"; do
  [ -d "$dir" ] || { echo "code_check: no directory $dir" >&2; exit 2; }
  if [ -f "$dir/Cargo.toml" ]; then
    check_crate "$dir"
  else
    [ -d "$dir/crates/clean_types" ] && check_crate "$dir/crates/clean_types"
    for m in "$dir"/crates/clean_model/*/; do
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
