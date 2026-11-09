#!/usr/bin/env bash
# vendor_nobuild_check.sh: each crate in vendor/ bakes in exactly what its removed build script
# emits on this toolchain and this target. Run where build scripts run (CI's Linux runner, with
# network): it builds the published crate, unpatched, in a scratch project outside the workspace,
# once with no features, once with its defaults and once with each feature set the workspace
# turns on, then compares the script's output with the copy's manifest:
#   - each `cargo:rustc-cfg` the script emits, against [package.metadata.nobuild.rustc-cfg],
#     whose predicates (`all()`, `any()`, `feature = "x"`) are evaluated on the build's features;
#   - each `cargo:rustc-check-cfg`, against [lints.rust.unexpected_cfgs] check-cfg;
#   - each file the script writes to OUT_DIR, byte for byte against out/ in the copy;
#   - any other instruction (rustc-env, a link library, a warning) is a failure: nothing bakes it.
# It also reruns tools/vendor_nobuild.sh on each copy and fails if the result differs from the
# committed one, so the copies are the script's output from the published crate and nothing else.
# CARGO overrides the cargo binary. Exit 1 lists each difference.
set -uo pipefail
cargo=${CARGO:-cargo}
root=$(git rev-parse --show-toplevel)
cd "$root" || exit 2
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT
bad=0
fail() { echo "vendor_nobuild_check: $*"; bad=1; }

# A TOML array of strings spread one per line, as vendor_nobuild.sh writes it: its items.
array_items() { # FILE KEY
  awk -v key="$2" '
    $0 ~ "^" key " = \\[" { on = 1; next }
    on && /^\]/ { exit }
    on { sub(/^[ \t]*'"'"'/, ""); sub(/'"'"',[ \t]*$/, ""); print }' "$1"
}
# The cfg table: `NAME<TAB>PREDICATE` per line.
cfg_table() { # FILE
  awk '/^\[/ { on = ($0 == "[package.metadata.nobuild.rustc-cfg]"); next }
    on && / = / { n = $1; sub(/^[^=]*= '"'"'/, ""); sub(/'"'"'[ \t]*$/, ""); print n "\t" $0 }' "$1"
}
holds() { # PREDICATE FEATURES(,a,b,)
  case $1 in
    'all()') return 0 ;;
    'any()') return 1 ;;
    'feature = "'*'"') local f=${1#feature = \"}; f=${f%\"}; case $2 in *",$f,"*) return 0 ;; esac; return 1 ;;
    *) fail "can't evaluate the predicate \`$1\`"; return 1 ;;
  esac
}

# The feature sets the workspace builds each package with, `name|f1,f2` per line.
"$cargo" tree -q --workspace --target all -e normal,build,dev --prefix none --format '{p}|{f}' \
  >"$work/tree" 2>"$work/tree.err" || { cat "$work/tree.err"; fail "cargo tree failed"; }

for dir in vendor/*/; do
  dir=${dir%/}; manifest=$dir/Cargo.toml
  grep -q '^\[package\.metadata\.nobuild\]' "$manifest" || continue
  name=$(sed -n 's/^name = "\(.*\)"$/\1/p' "$manifest" | head -n 1)
  version=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$manifest" | head -n 1)

  # The copy is what the script makes of the published crate.
  if ! out=$(bash tools/vendor_nobuild.sh "$name" "$version" 2>&1); then
    fail "$name $version: vendor_nobuild.sh failed: $out"
  elif ! git diff --quiet -- "$dir" || [ -n "$(git ls-files --others -- "$dir")" ]; then
    fail "$name $version: the committed copy differs from vendor_nobuild.sh's output:"
    { git diff --stat -- "$dir"; git ls-files --others -- "$dir"; } | head -n 20
    git checkout -q -- "$dir"; git clean -fdq -- "$dir"
  fi

  mapfile -t sets < <(
    echo "none"; echo "default"
    grep -F "$name v$version" "$work/tree" | grep -v ' (\*)' | sed 's/^[^|]*|//' | sort -u | sed 's/^/ws:/'
  )
  for set in "${sets[@]}"; do
    case $set in
      none) spec="default-features = false" ;;
      default) spec="" ;;
      ws:*) feats=${set#ws:}
        spec="default-features = false, features = [$(sed 's/[^,][^,]*/"&"/g; s/,/, /g' <<<"$feats")]" ;;
    esac
    probe=$work/probe-$name-$(echo "$set" | tr -c 'a-z0-9-' '_')
    mkdir -p "$probe/src"
    printf '[package]\nname = "nobuild-probe"\nversion = "0.0.0"\nedition = "2021"\npublish = false\n\n[workspace]\n\n[dependencies]\n%s = { version = "=%s"%s }\n' \
      "$name" "$version" "${spec:+, $spec}" >"$probe/Cargo.toml"
    : >"$probe/src/lib.rs"
    if ! (cd "$probe" && CARGO_TARGET_DIR="$probe/target" "$cargo" check -q >"$probe/log" 2>&1); then
      fail "$name $version ($set): the published crate didn't build:"; tail -n 20 "$probe/log"; continue
    fi
    used=",$(cd "$probe" && "$cargo" tree -q -e normal --prefix none --format '{p}|{f}' -i "$name@$version" 2>/dev/null \
      | head -n 1 | sed 's/^[^|]*|//'),"
    mapfile -t outputs < <(ls -d "$probe/target/debug/build/$name-"*/output 2>/dev/null)
    [ ${#outputs[@]} -eq 1 ] || { fail "$name $version ($set): ${#outputs[@]} build script outputs"; continue; }
    output=${outputs[0]}; outdir=$(dirname "$output")/out

    : >"$work/cfg.got"; : >"$work/check.got"
    while IFS= read -r line; do
      line=${line#cargo::}; line=${line#cargo:}
      case $line in
        rerun-if-changed=* | rerun-if-env-changed=*) ;;
        rustc-cfg=*) echo "${line#rustc-cfg=}" >>"$work/cfg.got" ;;
        rustc-check-cfg=*) echo "${line#rustc-check-cfg=}" >>"$work/check.got" ;;
        *) fail "$name $version ($set): the build script emits \`$line\`, which nothing bakes in" ;;
      esac
    done <"$output"
    : >"$work/cfg.want"
    while IFS=$'\t' read -r cfg pred; do
      holds "$pred" "$used" && echo "$cfg" >>"$work/cfg.want"
    done < <(cfg_table "$manifest")
    array_items "$manifest" check-cfg >"$work/check.want"
    for kind in cfg check; do
      if ! diff <(sort -u "$work/$kind.want") <(sort -u "$work/$kind.got") >"$work/diff"; then
        fail "$name $version ($set, features $used): rustc-$kind differs (< the copy, > the script):"
        cat "$work/diff"
      fi
    done
    mapfile -t baked < <(sed -n 's/^out = \[\(.*\)\]$/\1/p' "$manifest" | tr -d '" ' | tr ',' '\n' | sed '/^$/d')
    made=$(cd "$outdir" 2>/dev/null && find . -type f | sed 's#^\./##' | sort)
    [ "$made" = "$(printf '%s\n' "${baked[@]}" | sed '/^$/d' | sort)" ] \
      || fail "$name $version ($set): OUT_DIR holds [$(echo $made)], the copy bakes [${baked[*]}]"
    for f in "${baked[@]}"; do
      cmp -s "$outdir/$f" "$dir/out/$f" || fail "$name $version ($set): OUT_DIR/$f differs from $dir/out/$f"
    done
  done
  echo "checked $name $version: ${#sets[@]} feature sets"
done
[ "$bad" -eq 0 ] && echo "VENDOR NOBUILD OK"
exit $bad
