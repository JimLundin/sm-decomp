// Enemy overlays retain their public field names and explicit byte padding.
// The declarations share only layout boilerplate; RAM operations stay at callers.
macro_rules! enemy_layout {
    ($name:ident {
        $base:ident: $base_type:ty,
        words: [$($head:ident),* $(,)?],
        $(
            $array:ident: [$element:ty; $length:expr],
            words: [$($word:ident),* $(,)?],
        )*
    }) => {
        #[derive(Copy, Clone)]
        #[repr(C)]
        pub struct $name {
            pub $base: $base_type,
            $(pub $head: ::core::primitive::u16,)*
            $(
                pub $array: [$element; $length],
                $(pub $word: ::core::primitive::u16,)*
            )*
        }
    };
}

pub(crate) use enemy_layout;
