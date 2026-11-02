//! Spore Spawn runs in `sm_game::actors::spore_spawn`; its stem and boss
//! bits come from here.
#![forbid(unsafe_code)]
pub(crate) use sm_game::actors::spore_spawn::*;

// The named bank $A5 functions are the ones Draygon's function dispatch names.
const _: () = {
    use sm_game::actors::spore_spawn::function as f;
    assert!(
        0xa5_0000 | f::NOTHING as u32 == 10873626 && 0xa5_0000 | f::BEGIN_RETURN as u32 == 10873755
    );
    assert!(0xa5_0000 | f::DESCEND as u32 == 10873627 && 0xa5_0000 | f::ORBIT as u32 == 10873682);
    assert!(0xa5_0000 | f::FINISH_RETURN as u32 == 10873838);
};
