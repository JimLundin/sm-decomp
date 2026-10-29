//! Detached shared-actor access over the actual Session and prepared byte edges.
#![forbid(unsafe_code)]
use super::access::{self, AccessMode, ActorAccess, ActorKey, ActorRead};
use crate::Composed;
use sm_game::actors::state::Actor;

impl ActorAccess for Composed {
    fn resolve_actor<'a>(&'a self, key: ActorKey, mode: AccessMode) -> ActorRead<'a> {
        access::resolve_actor_with(self, key, mode, Composed::flag_byte_at)
    }

    fn edit_actor<R>(
        &mut self,
        key: ActorKey,
        mode: AccessMode,
        edit: impl for<'edit> FnOnce(&'edit mut Actor) -> R,
    ) -> R {
        access::edit_actor_with(
            self,
            key,
            mode,
            Composed::flag_byte_at,
            Composed::write_flag_byte,
            edit,
        )
    }
}
