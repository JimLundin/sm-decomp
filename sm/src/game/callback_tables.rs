// Callback aliases name the exact function-pointer type of each table slot.
macro_rules! callback_table {
    (nonnull $callback:ty; $($function:ident),* $(,)?) => {
        [$($function as $callback),*]
    };
    (@entry $callback:ty; None) => { None };
    (@entry $callback:ty; { $expression:expr }) => { $expression };
    (@entry $callback:ty; $function:ident) => { Some($function as $callback) };
    (@array $callback:ty; $($entry:tt),* $(,)?) => {
        [$($crate::game::callback_tables::callback_table!(@entry $callback; $entry)),*]
    };
    (unsafe $callback:ty; $($entry:tt),* $(,)?) => {
        unsafe { $crate::game::callback_tables::callback_table!(@array $callback; $($entry),*) }
    };
    ($callback:ty; $($entry:tt),* $(,)?) => {
        $crate::game::callback_tables::callback_table!(@array $callback; $($entry),*)
    };
}
pub(crate) use callback_table;
