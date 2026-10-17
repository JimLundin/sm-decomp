//! Compile the host's exact Rio wrapper and native registration together.
// This package has no Game loader, so registration symbols are intentionally
// only type-checked here; production Game references them in native-host.
#![allow(dead_code)]
type Bytes = unsafe extern "C" fn() -> *const u8;
type Size = unsafe extern "C" fn() -> usize;
type Counts = unsafe extern "C" fn(*mut u64);
#[path = "../../src/rio_hooks.rs"]
pub mod rio_hooks;
#[path = "../../src/rio_native.rs"]
mod rio_native;

// Compile the maintained patcher and three actual preparation stages without
// unrelated archive/process dependencies. Synthetic input is only an anchor
// integration fixture, never recovered-C or gameplay parity evidence.
#[derive(Debug)]
struct PatchRecord {
    path: String,
    before: String,
    after: String,
}
#[path = "../../src/patches.rs"]
mod patches;
use patches::replace_once;
#[path = "../../src/compiler_process.rs"]
mod compiler_process;
#[path = "../../src/source_prepare.rs"]
mod source_prepare;
#[cfg(test)]
use source_prepare::{rio_prepare, ripper_prepare, waver_prepare};

/// Diagnostic source-only seam. Callers own a private authenticated copy of the
/// recovered tree and seal every input/output. This executes no compiler/game.
pub fn prepare_sources(root: &std::path::Path) -> Result<Vec<(String, String, String)>, String> {
    source_prepare::prepare_sources(root)
        .map(|patches| patches.into_iter().map(|p| (p.path, p.before, p.after)).collect())
}

#[cfg(test)]
mod preparation_tests {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Tree(PathBuf);
    impl Tree {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "sm-rio-patches-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            fs::create_dir(root.join("src")).unwrap();
            fs::write(
                root.join("src/sm_a0.c"),
                concat!(
                    "case fnWaver_Init: Waver_Init(); return;\n",
                    "case fnWaver_Main: Waver_Main(); return;\n",
                    "case fnWaver_Instr_1: return Waver_Instr_1(k, j);\n",
                    "case fnJetPowerRipper_Init: JetPowerRipper_Init(); return;\n",
                    "case fnJetPowerRipper_Main: JetPowerRipper_Main(); return;\n",
                    "case fnRipper_Init: Ripper_Init(); return;\n",
                    "case fnRipper_Main: Ripper_Main(); return;\n",
                    "case fnRio_Init: Rio_Init(); return;\n",
                    "case fnRio_Main: Rio_Main(); return;\n",
                    "case fnEnemyInstr_Rio_Instr_1: return EnemyInstr_Rio_Instr_1(k, j);\n",
                    "uint8 Enemy_MoveRight_IgnoreSlopes(uint16 k, int32 amt) { return 0; }\n",
                    "uint16 SineMult8bit(uint16 a, uint16 varE32) { return 0; }\n",
                    "uint8 Enemy_MoveDown(uint16 k, int32 amount32) { return 0; }\n",
                    "uint16 CalculateAngleOfSamusFromEnemy(uint16 k) { return 0; }\n"
                ),
            )
            .unwrap();
            fs::write(root.join("restart_observe.h"), "#ifndef TEST_OBSERVE\n#define TEST_OBSERVE\n#endif\n").unwrap();
            for name in [
                "pilot",
                "scroll",
                "ceres_door",
                "room_transition",
                "door_scrolling",
                "camera",
                "item_instruction",
                "persistence",
            ] {
                let source = if name == "pilot" {
                    "sm_native_player_motion_active();\nstatic uint64_t effect_kind_counts[60];\nif (kind < 60) effect_kind_counts[kind]++;\nfor (unsigned kind = 1; kind < 60; kind++) {}\n"
                } else {
                    "sm_native_player_motion_active();\n"
                };
                fs::write(root.join(format!("restart_{name}.c")), source).unwrap();
            }
            fs::write(root.join("restart_player_motion.c"), "static bool SiblingActive(void) {\n  return false; }\n")
                .unwrap();
            Self(root)
        }
    }
    impl Drop for Tree {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn ordered_actor_preparation_preserves_previous_wrappers_and_fences() {
        let tree = Tree::new();
        let mut receipts = vec![];
        waver_prepare::instrument(&tree.0, &mut receipts).unwrap();
        ripper_prepare::instrument(&tree.0, &mut receipts).unwrap();
        rio_prepare::instrument(&tree.0, &mut receipts).unwrap();
        let source = fs::read_to_string(tree.0.join("src/sm_a0.c")).unwrap();
        for symbol in [
            "NativeRioPrevious_Enemy_MoveRight_IgnoreSlopes",
            "NativeRipperPrevious_Horizontal",
            "NativeWaverOriginal_Enemy_MoveRight_IgnoreSlopes",
            "NativeRioPrevious_Enemy_MoveDown",
            "NativeWaverOriginal_Enemy_MoveDown",
            "NativeRioPrevious_CalculateAngleOfSamusFromEnemy",
        ] {
            assert!(source.contains(symbol), "lost wrapper {symbol}");
        }
        assert!(source.contains("case fnRio_Main: sm_native_rio_ai(1); return;"));
        let header = fs::read_to_string(tree.0.join("restart_observe.h")).unwrap();
        assert_eq!(header.matches("#endif").count(), 1);
        assert!(header.contains("sm_native_rio_service_done"));
        for name in [
            "pilot",
            "scroll",
            "ceres_door",
            "room_transition",
            "door_scrolling",
            "camera",
            "item_instruction",
            "persistence",
            "player_motion",
            "waver",
            "ripper",
        ] {
            assert!(
                fs::read_to_string(tree.0.join(format!("restart_{name}.c")))
                    .unwrap()
                    .contains("sm_native_rio_active()"),
                "missing fence {name}"
            );
        }
        assert!(fs::read_to_string(tree.0.join("restart_pilot.c")).unwrap().contains("effect_kind_counts[72]"));
        assert_eq!(fs::read_to_string(tree.0.join("restart_rio.c")).unwrap(), include_str!("../../src/rio.c"));
        assert!(!receipts.is_empty());
    }
    #[test]
    fn missing_or_duplicated_scheduler_anchor_rejects_without_rio_publication() {
        for duplicate in [false, true] {
            let tree = Tree::new();
            let path = tree.0.join("src/sm_a0.c");
            let source = fs::read_to_string(&path).unwrap();
            let anchor = "case fnRio_Init: Rio_Init(); return;";
            fs::write(
                path,
                source.replace(
                    anchor,
                    if duplicate {
                        "case fnRio_Init: Rio_Init(); return;\ncase fnRio_Init: Rio_Init(); return;"
                    } else {
                        "case fnRio_Init: Changed(); return;"
                    },
                ),
            )
            .unwrap();
            let error = rio_prepare::instrument(&tree.0, &mut vec![]).unwrap_err();
            assert!(error.contains("exactly one instrumentation anchor"));
            assert!(!tree.0.join("restart_rio.c").exists());
        }
    }
    #[test]
    fn patch_anchors_and_receipts_accept_windows_line_endings() {
        let tree = Tree::new();
        let path = tree.0.join("restart_observe.h");
        fs::write(&path, "first\r\nanchor\r\nlast\r\n").unwrap();
        let mut receipts = vec![];
        replace_once(&tree.0, "restart_observe.h", "first\nanchor", "first\r\nreplacement", &mut receipts).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "first\nreplacement\nlast\n");
        assert_eq!(receipts[0].before, "first\nanchor");
        assert_eq!(receipts[0].after, "first\nreplacement");
        fs::write(&path, "anchor\r\nanchor\r\n").unwrap();
        assert!(replace_once(&tree.0, "restart_observe.h", "anchor\n", "replacement\n", &mut receipts).is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "anchor\r\nanchor\r\n");
    }
}
