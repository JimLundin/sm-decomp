//! Actual root-body interception plus thirteen passive private routine wrappers.
use super::{PatchRecord, replace_once, room_transition_prepare::structural_end};
use std::{fs, path::Path};

const ROUTINES: [(&str, &str); 13] = [
    ("src/sm_90.c", "MainScrollingRoutine"),
    ("src/sm_90.c", "CallScrollingFinishedHook"),
    ("src/sm_90.c", "Samus_ScrollFinishedHook_SporeSpawnFight"),
    ("src/sm_90.c", "Samus_HandleScroll_X"),
    ("src/sm_90.c", "Samus_HandleScroll_Y"),
    ("src/sm_90.c", "Samus_CalcDistanceMoved_X"),
    ("src/sm_90.c", "Samus_CalcDistanceMoved_Y"),
    ("src/sm_80.c", "HandleAutoscrolling_X"),
    ("src/sm_80.c", "HandleScrollingWhenTriggeringScrollRight"),
    ("src/sm_80.c", "HandleScrollingWhenTriggeringScrollLeft"),
    ("src/sm_80.c", "HandleAutoscrolling_Y"),
    ("src/sm_80.c", "HandleScrollingWhenTriggeringScrollDown"),
    ("src/sm_80.c", "HandleScrollingWhenTriggeringScrollUp"),
];
pub(super) fn instrument(root: &Path, patches: &mut Vec<PatchRecord>) -> Result<(), String> {
    for (id, (path, name)) in ROUTINES.iter().enumerate() {
        let source = fs::read_to_string(root.join(path)).map_err(|e| e.to_string())?;
        let params = if id == 1 { "uint32 ea" } else { "void" };
        let declaration = format!("void {name}({params}) {{");
        if source.matches(&declaration).count() != 1 {
            return Err(format!("missing or ambiguous camera routine {name}"));
        }
        let start = source.find(&declaration).unwrap();
        let open = start + declaration.len() - 1;
        let end = structural_end(&source, open, b'{', b'}')?;
        let body = &source[open + 1..end - 1];
        let args = if id == 1 { "ea" } else { "" };
        let operand = if id == 1 { "(uint16)ea" } else { "0" };
        let dispatch =
            if id == 0 { "if (sm_native_camera_dispatch()) { sm_native_camera_done(); return; }\n" } else { "" };
        let done = if id == 0 { "sm_native_camera_done();" } else { "" };
        let replacement = format!(
            "static void NativeCameraOriginal_{name}({params});\n\
             void {name}({params}) {{\n{dispatch}\
             sm_native_camera_routine(0, {id}, {operand});\n\
             NativeCameraOriginal_{name}({args});\n\
             sm_native_camera_routine(1, {id}, 0);\n{done}\n}}\n\
             static void NativeCameraOriginal_{name}({params}) {{{body}}}"
        );
        replace_once(root, path, &source[start..end], &replacement, patches)?;
    }
    Ok(())
}
