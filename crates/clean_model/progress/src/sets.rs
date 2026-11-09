//! Typed flag sets of a small enum: the items, the beams, the events, the boss kinds. A set is
//! `Copy` and built in a constant, so a room's conditions can be constant data.

/// The bit of `member`: each member listed takes the next bit up from `bit`, in one complete
/// `match`, so a member left out of the list is refused by the compiler.
macro_rules! bit_of {
    ($member:expr, $bit:expr, [$($arms:tt)*]) => {
        match $member { $($arms)* }
    };
    ($member:expr, $bit:expr, [$($arms:tt)*] $first:path $(, $rest:path)*) => {
        $crate::sets::bit_of!($member, $bit << 1, [$($arms)* $first => $bit,] $($rest),*)
    };
}

/// Declares `$set`, a set of `$member`s, given every member of the enum.
macro_rules! flag_set {
    ($(#[$doc:meta])* $set:ident of $member:ident { $($each:ident),+ $(,)? }) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Default, PartialEq, Eq)]
        pub struct $set(u32);

        impl $set {
            /// Every member, in the order listed.
            const MEMBERS: &[$member] = &[$($member::$each),+];

            /// The set of `members`.
            #[must_use]
            pub const fn of(members: &[$member]) -> Self {
                let mut set = Self(0);
                let mut rest = members;
                while let [first, others @ ..] = rest {
                    set = set.with(*first);
                    rest = others;
                }
                set
            }

            /// Whether `member` is in the set.
            #[must_use]
            pub const fn contains(self, member: $member) -> bool {
                self.0 & Self::bit(member) != 0
            }

            /// Whether any member is in both sets.
            #[must_use]
            pub const fn intersects(self, other: Self) -> bool {
                self.0 & other.0 != 0
            }

            /// The set with `member` in it.
            #[must_use]
            pub const fn with(self, member: $member) -> Self {
                Self(self.0 | Self::bit(member))
            }

            /// The set without `member`.
            #[must_use]
            pub const fn without(self, member: $member) -> Self {
                Self(self.0 & !Self::bit(member))
            }

            /// How many members the set holds.
            #[must_use]
            pub const fn count(self) -> $crate::count::Count {
                $crate::count::count(self.0.count_ones())
            }

            const fn bit(member: $member) -> u32 {
                $crate::sets::bit_of!(member, 1, [] $($member::$each),+)
            }
        }

        impl ::core::fmt::Debug for $set {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let members = Self::MEMBERS.iter().filter(|&&member| self.contains(member));
                f.debug_set().entries(members).finish()
            }
        }
    };
}

pub(crate) use {bit_of, flag_set};
