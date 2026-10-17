//! Body wrappers at the three actual roots; fixed scoped service substitutions.
use super::{PatchRecord, replace_once, room_transition_prepare::structural_end};
use std::{fs, path::Path};

const ROUTINES: [&str; 13] = [
    "FixDoorsMovingUp",
    "DoorTransitionScrollingSetup",
    "Irq_FollowDoorTransition",
    "DoorTransitionScrollingSetup_Right",
    "DoorTransitionScrollingSetup_Left",
    "DoorTransitionScrollingSetup_Down",
    "DoorTransitionScrollingSetup_Up",
    "UpdatePreviousLayerBlocks",
    "UpdateBgScrollOffsets",
    "DoorTransition_Right",
    "DoorTransition_Left",
    "DoorTransition_Down",
    "DoorTransition_Up",
];
const SERVICES: [&str; 4] = [
    "CalculateLayer2Xpos",
    "CalculateLayer2Ypos",
    "CalculateBgScrollAndLayerPositionBlocks",
    "UpdateScrollVarsUpdateMap",
];
pub(super) fn instrument(root: &Path, patches: &mut Vec<PatchRecord>) -> Result<(), String> {
    let path = "src/sm_80.c";
    let mut seen = [0; 4];
    for (id, name) in ROUTINES.iter().enumerate() {
        let source = fs::read_to_string(root.join(path)).map_err(|e| e.to_string())?;
        let ty = if id < 9 { "void" } else { "uint8" };
        let declaration = format!("{ty} {name}(void) {{");
        if source.matches(&declaration).count() != 1 {
            return Err(format!("missing or ambiguous door-scrolling routine {name}"));
        }
        let start = source.find(&declaration).unwrap();
        let open = start + declaration.len() - 1;
        let end = structural_end(&source, open, b'{', b'}')?;
        let mut body = source[open + 1..end - 1].to_owned();
        for (service, name) in SERVICES.iter().enumerate() {
            let anchor = format!("{name}()");
            seen[service] += body.matches(&anchor).count();
            body = body.replace(&anchor, &format!("sm_native_door_scrolling_call({service})"));
        }
        let dispatch = if id < 3 {
            format!("if (sm_native_door_scrolling_dispatch({id})) {{ sm_native_door_scrolling_done({id}); return; }}\n")
        } else {
            String::new()
        };
        let call = if id < 9 {
            format!("NativeDirectionalOriginal_{name}();")
        } else {
            format!("uint8 result = NativeDirectionalOriginal_{name}();")
        };
        let result = if id < 9 { "0" } else { "result" };
        let done = if id < 3 {
            format!("sm_native_door_scrolling_done({id});")
        } else if id >= 9 {
            "return result;".into()
        } else {
            String::new()
        };
        let replacement = format!(
            "static {ty} NativeDirectionalOriginal_{name}(void);\n\
             {ty} {name}(void) {{\n{dispatch}\
             sm_native_door_scrolling_routine(0, {id}, 0);\n{call}\n\
             sm_native_door_scrolling_routine(1, {id}, {result});\n{done}\n}}\n\
             static {ty} NativeDirectionalOriginal_{name}(void) {{{body}}}"
        );
        replace_once(root, path, &source[start..end], &replacement, patches)?;
    }
    // Counts authenticate the complete fixed call-site catalog as well as roots.
    if seen != [4, 4, 7, 8] {
        return Err(format!("unexpected door-scrolling service sites: {seen:?}"));
    }
    Ok(())
}
