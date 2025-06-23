// src/theme.rs
use termcolor::Color;

pub struct AppColors {
    pub err: Color,
    pub warn: Color,
    pub info: Color,
    pub trace: Color,
    pub noise: Color,
}

pub const THEME: AppColors = AppColors {
    err: Color::Red,
    warn: Color::Yellow,
    info: Color::Green,
    trace: Color::Ansi256(244),
    noise: Color::Ansi256(23),
};
