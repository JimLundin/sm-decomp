// Keep SDL's enum names while retaining each caller's original constant types.
#[doc(hidden)]
#[macro_export]
macro_rules! sdl_constants {
    ($sdl_enum:ident as $ty:ty; $($name:ident),+ $(,)?) => {
        $(pub const $name: $ty = ::sdl2_sys::$sdl_enum::$name as $ty;)+
    };
    ($sdl_enum:ident: $ty:ty; $($name:ident),+ $(,)?) => {
        $(pub const $name: $ty = ::sdl2_sys::$sdl_enum::$name;)+
    };
}
