//! Fune and namihe run in `sm_game::actors::fune` through the actor lease.
#![forbid(unsafe_code)]

// The named bank $A8 addresses are the ones `session_runtime` dispatches on.
const _: () = {
    use sm_game::actors::fune::instruction as i;
    assert!(i::QUEUE_SPIT_SOUND == 11048485 && i::SPAWN_LEFTWARDS_NAMIHE_FIREBALL == 11048497);
    assert!(i::SPAWN_RIGHTWARDS_NAMIHE_FIREBALL == 11048522);
    assert!(i::SPAWN_LEFTWARDS_FUNE_FIREBALL == 11048547);
    assert!(i::SPAWN_RIGHTWARDS_FUNE_FIREBALL == 11048572);
    assert!(i::FINISH_ACTIVITY == 11048597 && i::FINISH_ACTIVITY_2 == 11048628);
};
