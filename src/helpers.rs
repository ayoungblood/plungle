// src/helpers.rs

// ANSI color codes
pub const ANSI_C_RED: &str = "31";
pub const ANSI_C_GRN: &str = "32";
pub const ANSI_C_YLW: &str = "33";
pub const ANSI_C_BLU: &str = "34";
pub const ANSI_C_MAG: &str = "35";
pub const ANSI_C_CYN: &str = "36";

// debug print
#[macro_export]
macro_rules! dprintln {
    ($opt_verbose:expr, $message_verbose:expr, $($arg:tt)*) => {
        if $opt_verbose >= $message_verbose {
            eprintln!($($arg)*);
        }
    }
}

// color print
#[macro_export]
macro_rules! cprintln {
    ($color:expr, $($arg:tt)*) => {
        eprintln!("\x1b[{}m{}\x1b[0m", $color, format!($($arg)*));
    }
}

// debug function name
#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f").unwrap()
    }}
}
