#!/usr/bin/env bash
# Sourced by check_crate_rules.sh; cargo acquisition must succeed before graph filtering.
engine="data game runtime app"
# The workspace packages each new crate may depend on directly (crates/README.md).
# `canonical` holds the codec trait so `data` can derive it; `canonical_derive` is its macro.
allowed() {
  case $1 in
    data) echo "canonical types" ;;
    game) echo "data canonical types samus_movement world_blocks world_camera interface_hud run_progress samus_appearance boss_ceres_ridley" ;;
    runtime) echo "data" ;;
    app) echo "data import game runtime" ;;
    oracle) echo "data oracle_record" ;;
    oracle_record) echo "" ;;
    verify) echo "data import game runtime oracle_record canonical types samus_movement world_blocks world_camera interface_hud run_progress samus_appearance boss_ceres_ridley world_doors weapons_projectiles" ;;
    tools) echo "verify oracle runtime import" ;;
    # The importers (docs/architecture/assets.md §5): game and runtime never list it.
    import) echo "data canonical" ;;
    canonical) echo "canonical_derive" ;;
    canonical_derive) echo "" ;;
    # The clean-room crates depend only on each other (MODEL §5).
    types) echo "" ;;
    samus_movement) echo "types" ;;
    world_blocks) echo "types" ;;
    world_camera) echo "types" ;;
    interface_hud) echo "types" ;;
    run_progress) echo "types" ;;
    samus_appearance) echo "types" ;;
    boss_ceres_ridley) echo "types" ;;
    world_doors) echo "types" ;;
    weapons_projectiles) echo "types" ;;
  esac
}
new_crates="data import game runtime app oracle oracle_record verify tools canonical canonical_derive types samus_movement world_blocks world_camera interface_hud run_progress samus_appearance boss_ceres_ridley world_doors weapons_projectiles"

# Name the rule a forbidden edge breaks, for the message.
rule() {
  case $2 in sm|sm_*) echo "D1: no new crate depends on sm or sm_*"; return ;; esac
  case " $engine " in *" $1 "*)
    case $2 in oracle|verify|tools) echo "rule 1: engine crates don't depend on oracle, verify or tools"; return ;; esac ;;
  esac
  case $1/$2 in game/runtime|runtime/game) echo "rule 2: game and runtime don't depend on each other"; return ;; esac
  case $1/$2 in game/import|runtime/import|data/import) echo "rule 2: only the importer knows the ROM's layout; game and runtime don't depend on import"; return ;; esac
  [ "$1" = oracle ] && { echo "rule 3: oracle depends only on data and its record types"; return; }
  echo "not in the crate plan"
}

check_direct_dependencies() {
for crate in $new_crates; do
  # Direct dependencies that are workspace members: path packages, "name vX (path)", outside
  # vendor/, whose baked copies stand in for registry crates.
  if ! deps=$("$cargo" tree -q -p "$crate" --all-features -e normal,build,dev --depth 1 \
      --prefix none --format '{p}' 2>&1); then
    fail "$crate: cargo tree failed: $deps"; continue
  fi
  for dep in $(echo "$deps" | tail -n +2 | grep ' (' | grep -v ' (\*)$' | grep -vE '[\\/]vendor[\\/][^\\/]*\)$' | cut -d' ' -f1 | sort -u); do
    case " $(allowed "$crate") " in *" $dep "*) continue ;; esac
    fail "$crate depends on $dep ($(rule "$crate" "$dep"))"
  done
done

}

check_dependency_reachability() {
# Engine crates reach none of oracle, verify, tools, sm or sm_* even through another package.
for crate in $engine; do
  if ! deps=$("$cargo" tree -q -p "$crate" --all-features -e normal,build,dev --prefix none \
      --format '{p}' 2>&1); then
    fail "$crate: cargo tree failed: $deps"; continue
  fi
  reach=$(printf '%s\n' "$deps" | tail -n +2 | cut -d' ' -f1 | sort -u \
    | grep -Ex 'oracle|verify|tools|sm|sm_.*' || true)
  for dep in $reach; do fail "$crate reaches $dep through its dependencies (rule 1)"; done
done

# Content is already in our formats; data, game and runtime never reach the importer.
for crate in data game runtime; do
  if ! deps=$("$cargo" tree -q -p "$crate" --all-features -e normal,build,dev --prefix none \
      --format '{p}' 2>&1); then
    fail "$crate: cargo tree failed: $deps"; continue
  fi
  if printf '%s\n' "$deps" | tail -n +2 | cut -d' ' -f1 | grep -qx import; then
    fail "$crate reaches import through its dependencies (rule 2)"
  fi
done

}
