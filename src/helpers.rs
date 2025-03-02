// src/helpers.rs

use rust_decimal::prelude::ToPrimitive;
use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum, PartialEq, Default)]
#[clap(rename_all = "kebab_case")]
pub enum Format {
    #[default]
    Json,
    Toml,
    Text,
}

#[derive(Debug)]
pub enum Dest {
    Stdout,
    Stderr,
}

// unified print function
// signature uprintln!(opt, dest, color, verbose, args..)
#[macro_export]
macro_rules! uprintln {
    // handle the case where color and verbose are None
    ($opt:expr, $dest:expr, None, None, $($arg:tt)*) => {
        match $dest {
            Dest::Stdout => {
                 println!($($arg)*);
            },
            Dest::Stderr => {
                eprintln!($($arg)*);
            },
        }
    };
    // handle the case where color is Some, verbose is None
    ($opt:expr, $dest:expr, $color:expr, None, $($arg:tt)*) => {
        match $dest {
            Dest::Stdout => {
                cprintln!($opt.color, $color, $($arg)*);
            },
            Dest::Stderr => {
                ceprintln!($opt.color, $color, $($arg)*);
            },
        }
    };
    // handle the case where color None, verbose is Some
    ($opt:expr, $dest:expr, None, $verbose:expr, $($arg:tt)*) => {
        match $dest {
            Dest::Stdout => {
                vprintln!($opt.verbose, $verbose, $($arg)*);
            },
            Dest::Stderr => {
                veprintln!($opt.verbose, $verbose, $($arg)*);
            },
        }
    };
    // handle the case where color and verbose are Some
    ($opt:expr, $dest:expr, $color:expr, $verbose:expr, $($arg:tt)*) => {
        match $dest {
            Dest::Stdout => {
                cvprintln!($opt.color, $color, $opt.verbose, $verbose, $($arg)*);
            },
            Dest::Stderr => {
                cveprintln!($opt.color, $color, $opt.verbose, $verbose, $($arg)*);
            },
        }
    };
}

// color print (stdout)
#[macro_export]
macro_rules! cprintln {
    ($opt_color:expr, $message_color:expr, $($arg:tt)*) => {
        // map from clap::ColorChoice to termcolor::ColorChoice
        let mut stdout = StandardStream::stdout(match $opt_color {
            clap::ColorChoice::Auto => ColorChoice::Auto,
            clap::ColorChoice::Always => ColorChoice::Always,
            clap::ColorChoice::Never => ColorChoice::Never,
        });
        let _ = stdout.set_color(ColorSpec::new().set_fg(Some($message_color)));
        writeln!(&mut stdout, $($arg)*).ok();
        let _ = stdout.reset();
    };
}

// verbose print (stdout)
#[macro_export]
macro_rules! vprintln {
    ($opt_verbose:expr, $message_verbose:expr, $($arg:tt)*) => {
        if $opt_verbose >= $message_verbose {
            println!($($arg)*);
        }
    };
}

// color verbose print (stdout)
#[macro_export]
macro_rules! cvprintln {
    ($opt_color:expr, $message_color:expr, $opt_verbose:expr, $message_verbose:expr, $($arg:tt)*) => {
        if $opt_verbose >= $message_verbose {
            // map from clap::ColorChoice to termcolor::ColorChoice
            let mut stdout = StandardStream::stdout(match $opt_color {
                clap::ColorChoice::Auto => ColorChoice::Auto,
                clap::ColorChoice::Always => ColorChoice::Always,
                clap::ColorChoice::Never => ColorChoice::Never,
            });
            let _ = stdout.set_color(ColorSpec::new().set_fg(Some($message_color)));
            writeln!(&mut stdout, $($arg)*).ok();
            let _ = stdout.reset();
        }
    };
}

// color print (stderr)
#[macro_export]
macro_rules! ceprintln {
    ($opt_color:expr, $message_color:expr, $($arg:tt)*) => {
        // map from clap::ColorChoice to termcolor::ColorChoice
        let mut stderr = StandardStream::stderr(match $opt_color {
            clap::ColorChoice::Auto => ColorChoice::Auto,
            clap::ColorChoice::Always => ColorChoice::Always,
            clap::ColorChoice::Never => ColorChoice::Never,
        });
        let _ = stderr.set_color(ColorSpec::new().set_fg(Some($message_color)));
        writeln!(&mut stderr, $($arg)*).ok();
        let _ = stderr.reset();
    };
}

// verbose print (stderr)
#[macro_export]
macro_rules! veprintln {
    ($opt_verbose:expr, $message_verbose:expr, $($arg:tt)*) => {
        if $opt_verbose >= $message_verbose {
            eprintln!($($arg)*);
        }
    };
}

// color verbose print (stderr)
#[macro_export]
macro_rules! cveprintln {
    ($opt_color:expr, $message_color:expr, $opt_verbose:expr, $message_verbose:expr, $($arg:tt)*) => {
        if $opt_verbose >= $message_verbose {
            // map from clap::ColorChoice to termcolor::ColorChoice
            let mut stderr = StandardStream::stderr(match $opt_color {
                clap::ColorChoice::Auto => ColorChoice::Auto,
                clap::ColorChoice::Always => ColorChoice::Always,
                clap::ColorChoice::Never => ColorChoice::Never,
            });
            let _ = stderr.set_color(ColorSpec::new().set_fg(Some($message_color)));
            writeln!(&mut stderr, $($arg)*).ok();
            let _ = stderr.reset();
        }
    };
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

// print a Decimal as a frequency
pub fn freq2str(freq: &rust_decimal::Decimal) -> String {
    let f = freq.to_f64().unwrap();
    if f >= 1e9 {
        return format!("{:8.4} GHz", f/1e9)
    } else if f >= 1e6 {
        return format!("{:8.4} MHz", f/1e6)
    } else if f >= 1e3 {
        return format!("{:8.4} kHz", f/1e3)
    } else if f >= 1.0 {
        return format!("{:8.4} Hz", f)
    }
    format!("{:.6}", f)
}
