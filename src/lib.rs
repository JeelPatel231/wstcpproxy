#[macro_export]
macro_rules! debug_print {
    ($($arg:expr),*) => {
        if cfg!(debug_assertions) {
            dbg!($($arg),*)
        } else {
            ($($arg),*)
        }
    };
}
