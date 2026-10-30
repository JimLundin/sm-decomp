//! The original's callback tables: address-keyed dispatch over named bodies,
//! forwarding wrappers and countdown callbacks, ported from sm's macros.
//! Exported as `game_*` and re-exported here under their original names.

#[doc(hidden)]
#[macro_export]
macro_rules! game_instruction_callbacks {
    ($visibility:vis fn $parameters:tt; @prefix $prefix:tt; $($name:ident $tail:tt $body:tt;)*) => {
        $($crate::game_instruction_callbacks!(@function [$visibility fn $name $parameters] $prefix $tail $body);)*
    };
    ($visibility:vis fn $parameters:tt -> $result:ty; @prefix $prefix:tt; $($name:ident $tail:tt $body:tt;)*) => {
        $($crate::game_instruction_callbacks!(@function [$visibility fn $name $parameters -> $result] $prefix $tail $body);)*
    };
    ($visibility:vis fn $parameters:tt -> $result:ty; $($entries:tt)*) => {
        $crate::game_instruction_callbacks!($visibility fn $parameters -> $result; @prefix {}; $($entries)*);
    };
    (@function [$($header:tt)*] {$($prefix:tt)*} [$tail:expr] {$($body:tt)*}) => {
        $($header)* { $($prefix)* $($body)* $tail }
    };
    (@function [$($header:tt)*] {$($prefix:tt)*} [] {$($body:tt)*}) => {
        $($header)* { $($prefix)* $($body)* }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! game_forward_calls {
    ($visibility:vis fn $parameters:tt -> $result:ty; $($name:ident => $call:expr;)+) => {
        $($visibility fn $name $parameters -> $result { $call })+
    };
    ($visibility:vis fn $parameters:tt; $($name:ident => $call:expr;)+) => {
        $($visibility fn $name $parameters { $call; })+
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! game_countdown_callbacks {
        (
            $visibility:vis fn $parameters:tt;
            before $before:tt
            when $condition:expr;
            $(
                $name:ident $effects:tt
                $(else $otherwise:tt)?
                $(after $after:tt)?;
            )+
        ) => {
            $(
                $crate::game_countdown_callbacks! {
                    @safe [$visibility fn $name $parameters]
                    $before [$condition] $effects
                    $(else $otherwise)? $(after $after)?
                }
            )+
        };
        (
            @safe [$($header:tt)*]
            { $($before:tt)* } [$condition:expr] $effects:block
            $(else $otherwise:block)? $(after { $($after:tt)* })?
        ) => {
            $($header)* {
                $($before)*
                if $condition $effects $(else $otherwise)?
                $($($after)*)?
            }
        };
    }

#[doc(hidden)]
#[macro_export]
macro_rules! game_address_dispatch {
        ($($(#[$metadata:meta])* $visibility:vis fn $name:ident $parameters:tt $(-> $result:ty)?
            => $selector:expr; $(setup { $($setup:tt)* })? call $arguments:tt; default $fallback:expr;
            { $($pattern:pat => $target:tt $(::$tail:ident)*;)* })*) => {
            $(
                $(#[$metadata])*
                $visibility fn $name $parameters $(-> $result)? {
                    $($($setup)*)?
                    match $selector {
                        $($pattern => $crate::game_address_dispatch!(@call $arguments $target $(::$tail)*),)*
                        _ => $fallback
                    }
                }
            )*
        };
        (@call ($($argument:expr),* $(,)?) $target:path) => {
            $target($($argument),*)
        };
        (@call $arguments:tt [$($expression:tt)*]) => {
            $($expression)*
        };
    }

#[doc(hidden)]
#[macro_export]
macro_rules! game_callback_dispatch {
        (
            $(#[$metadata:meta])* $visibility:vis fn $name:ident $parameters:tt $(-> $result:ty)?
            => $selector:expr; $(setup { $($setup:tt)* })? call $arguments:tt; default $fallback:expr;
            $(@body $body_mode:ident;)?
            $(
                @functions [$($header:tt)*]; $(@prefix {$($prefix:tt)*};)?
                $($([$pattern:pat])? $callback:ident $tail:tt $body:tt;)*
            )*
            $(
                @forward [$($forward_header:tt)*];
                $($([$forward_pattern:pat])? $forward_callback:ident => $forward_call:expr;)*
            )*
            $(
                @countdown [$($count_header:tt)*]; before $before:tt when $condition:expr;
                $($([$count_pattern:pat])? $count_callback:ident $effects:tt
                    $(else $otherwise:tt)? $(after $after:tt)?;)*
            )*
            $(@external {$($external_pattern:pat => $target:tt $(::$path_tail:ident)*;)*})?
        ) => {
            $(
                $crate::game_instruction_callbacks! {
                    $($header)*; @prefix {$($($prefix)*)?};
                    $($callback $tail $body;)*
                }
            )*
            $(
                $crate::game_forward_calls! {
                    $($forward_header)*; $($forward_callback => $forward_call;)*
                }
            )*
            $(
                $crate::game_countdown_callbacks! {
                    $($count_header)*; before $before when $condition;
                    $($count_callback $effects $(else $otherwise)? $(after $after)?;)*
                }
            )*
            $(#[$metadata])*
            $visibility fn $name $parameters $(-> $result)? {
                $crate::game_callback_dispatch!(@body [checked] {
                $($($setup)*)?
                match $selector {
                    $($($($pattern => $crate::game_address_dispatch!(@call $arguments $callback),)?)*)*
                    $($($($forward_pattern => $crate::game_address_dispatch!(@call $arguments $forward_callback),)?)*)*
                    $($($($count_pattern => $crate::game_address_dispatch!(@call $arguments $count_callback),)?)*)*
                    $($($external_pattern => $crate::game_address_dispatch!(@call $arguments $target $(::$path_tail)*),)*)?
                    _ => $fallback
                }
                })
            }
        };
        (@body [checked] {$($body:tt)*}) => { { $($body)* } };
    }

pub use crate::{
    game_address_dispatch as address_dispatch, game_callback_dispatch as callback_dispatch,
    game_countdown_callbacks as countdown_callbacks, game_forward_calls as forward_calls,
    game_instruction_callbacks as instruction_callbacks,
};
