//! Session state as typed copies, read and stored on the compat host's lease.
//!
//! A function on the compatibility host that owns some Session fields for its
//! call either updates them on one lease ([`update`]) or, when it calls back
//! into the host while it works on them, takes a typed copy ([`with`]): the
//! copy is read once from the Session on a lease, the function works on its
//! fields, and the fields it changed are stored back on another lease. A store
//! that changed anything refreshes the observations, as the copies' stores did.
//!
//! The function must not reach the copied fields by any other path while it
//! holds the copy; code it calls takes the copy instead.
use crate::runtime_access::RuntimeAccess;
use crate::session::Session;
use crate::session_access::SessionAccess as _;

/// Session fields held as one typed copy.
pub trait Part: Clone + PartialEq {
    /// The fields as `session` holds them.
    fn of(session: &Session) -> Self;
    /// Store to `session` the fields `self` changed from `before`.
    fn store(&self, before: &Self, session: &mut Session);
}

/// `P` as the Session holds it, read on a fresh lease.
pub fn read<P: Part, H: RuntimeAccess + ?Sized>(game: &mut H) -> P {
    P::of(game.runtime().session())
}

/// The Session field `field` selects, read on a fresh lease.
pub fn get<H: RuntimeAccess + ?Sized, F: Clone>(game: &mut H, field: fn(&Session) -> &F) -> F {
    field(game.runtime().session()).clone()
}

/// Store the fields `after` changed from `before` on a fresh lease, then
/// refresh the observations; nothing when it changed none.
pub fn store<P: Part, H: RuntimeAccess + ?Sized>(game: &mut H, before: &P, after: &P) {
    if after == before {
        return;
    }
    let mut runtime = game.runtime();
    after.store(before, runtime.session_mut());
    runtime.refresh_inputs();
}

/// Run `action` on a copy of `P` read from the Session, then store the fields
/// it changed.
pub fn with<P, H, T>(game: &mut H, action: impl FnOnce(&mut H, &mut P) -> T) -> T
where
    P: Part,
    H: RuntimeAccess + ?Sized,
{
    let before = read::<P, H>(game);
    let mut part = before.clone();
    let result = action(game, &mut part);
    store(game, &before, &part);
    result
}

/// Run `action` on a copy of `P` on one fresh lease, store the fields it
/// changed, and refresh the observations when it changed any. For an `action`
/// that does not call back into the host.
pub fn edit<P, H, T>(game: &mut H, action: impl FnOnce(&mut P) -> T) -> T
where
    P: Part,
    H: RuntimeAccess + ?Sized,
{
    let mut runtime = game.runtime();
    let session = runtime.session_mut();
    let before = P::of(session);
    let mut part = before.clone();
    let result = action(&mut part);
    if part != before {
        part.store(&before, session);
        runtime.refresh_inputs();
    }
    result
}

/// Run `update` on the Session field `field` selects, on a fresh lease, and
/// refresh the observations when it changed the field.
pub fn update<H, F, T>(
    game: &mut H,
    field: fn(&mut Session) -> &mut F,
    update: impl FnOnce(&mut F) -> T,
) -> T
where
    H: RuntimeAccess + ?Sized,
    F: Clone + PartialEq,
{
    let mut runtime = game.runtime();
    let field = field(runtime.session_mut());
    let before = field.clone();
    let result = update(field);
    if *field != before {
        runtime.refresh_inputs();
    }
    result
}

/// Store `after` to `field` when it differs from `before`.
pub fn store_changed<T: Clone + PartialEq>(field: &mut T, before: &T, after: &T) {
    if after != before {
        field.clone_from(after);
    }
}

/// Store each element of `after` that differs from `before` to its element of
/// `fields`.
pub fn store_changed_each<'a, T: Copy + PartialEq + 'a>(
    fields: impl IntoIterator<Item = &'a mut T>,
    before: &[T],
    after: &[T],
) {
    for ((field, before), after) in fields.into_iter().zip(before).zip(after) {
        if after != before {
            *field = *after;
        }
    }
}
