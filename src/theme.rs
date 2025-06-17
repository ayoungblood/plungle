// src/theme.rs
use termcolor::Color;

pub struct AppColors {
    pub trace: Color,
    pub info: Color,
    pub warn: Color,
    pub err: Color,
}

pub const THEME: AppColors = AppColors {
    trace: Color::Ansi256(244),
    info: Color::Green,
    warn: Color::Yellow,
    err: Color::Red,
};
