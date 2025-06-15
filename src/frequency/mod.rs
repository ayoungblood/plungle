use std::fmt;
use std::num::ParseFloatError;
use std::ops::{Add, Sub};

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Frequency {
    pub freq_hz: f64,
}

#[allow(unused)]
impl Frequency {
    pub fn new() -> Self {
        Frequency { freq_hz: 0.0 }
    }

    pub fn from_hz(freq_hz: f64) -> Self {
        Frequency { freq_hz }
    }
    pub fn from_khz(freq_khz: f64) -> Self {
        Frequency { freq_hz: freq_khz * 1000.0 }
    }
    pub fn from_mhz(freq_mhz: f64) -> Self {
        Frequency { freq_hz: freq_mhz * 1_000_000.0 }
    }
    pub fn from_ghz(freq_ghz: f64) -> Self {
        Frequency { freq_hz: freq_ghz * 1_000_000_000.0 }
    }

    pub fn hz(&self) -> f64 {
        self.freq_hz
    }
    pub fn khz(&self) -> f64 {
        self.freq_hz / 1000.0
    }
    pub fn mhz(&self) -> f64 {
        self.freq_hz / 1_000_000.0
    }
    pub fn ghz(&self) -> f64 {
        self.freq_hz / 1_000_000_000.0
    }

    pub fn to_pretty_str(&self) -> String {
        if self.freq_hz < 1000.0 {
            format!("{:.} Hz", self.freq_hz)
        } else if self.freq_hz < 1_000_000.0 {
            format!("{:.} kHz", self.freq_hz / 1000.0)
        } else if self.freq_hz < 1_000_000_000.0 {
            format!("{:.} MHz", self.freq_hz / 1_000_000.0)
        } else {
            format!("{:.} GHz", self.freq_hz / 1_000_000_000.0)
        }
    }

    pub fn from_hz_str(str: &str) -> Result<Self, ParseFloatError> {
        str.chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect::<String>()
            .parse()
            .map(|freq_hz| Frequency { freq_hz })
    }
    pub fn from_khz_str(str: &str) -> Result<Self, ParseFloatError> {
        str.chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect::<String>()
            .parse()
            .map(|freq_khz: f64| Frequency { freq_hz: freq_khz * 1000.0 })
    }
    pub fn from_mhz_str(str: &str) -> Result<Self, ParseFloatError> {
        str.chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect::<String>()
            .parse()
            .map(|freq_mhz: f64| Frequency { freq_hz: freq_mhz * 1_000_000.0 })
    }
    pub fn from_ghz_str(str: &str) -> Result<Self, ParseFloatError> {
        str.chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect::<String>()
            .parse()
            .map(|freq_ghz: f64| Frequency { freq_hz: freq_ghz * 1_000_000_000.0 })
    }
}

impl Add for Frequency {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Frequency { freq_hz: self.freq_hz + other.freq_hz }
    }
}

impl Sub for Frequency {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Frequency { freq_hz: self.freq_hz - other.freq_hz }
    }
}

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(precision) = f.precision() {
            // If a precision was specified, format the f64 with that precision
                write!(f, "{:.width$} Hz", self.freq_hz, width = precision)
            } else {
                // Otherwise, use the default formatting for Display
                write!(f, "{} Hz", self.freq_hz)
            }
    }
}

// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_frequency() {
        let freq = Frequency::new();
        assert_eq!(freq.freq_hz, 0.0);
    }

    #[test]
    fn test_from_to_hz() {
        let freq = Frequency::from_hz(123.45);
        assert_eq!(freq.hz(), 123.45);
        assert_eq!(freq.khz(), 0.12345);
        assert_eq!(freq.mhz(), 0.00012345);
        assert_eq!(freq.ghz(), 0.00000012345);
        assert_eq!(format!("{}", freq), "123.45 Hz");
        assert_eq!(format!("{:.1}", freq), "123.5 Hz");
    }

    #[test]
    fn test_from_to_khz() {
        let freq = Frequency::from_khz(123.456);
        assert_eq!(freq.khz(), 123.456);
    }

    #[test]
    fn test_from_to_mhz() {
        let freq = Frequency::from_mhz(123.456789);
        assert_eq!(freq.mhz(), 123.456789);
    }

    #[test]
    fn test_from_to_ghz() {
        let freq = Frequency::from_ghz(987.654);
        assert_eq!(freq.ghz(), 987.654);
    }

    #[test]
    fn test_add() {
        let f1 = Frequency::from_hz(789.0);
        let f2 = Frequency::from_khz(456.0);
        let f3 = Frequency::from_mhz(123.0);
        let sum = f1 + f2 + f3;
        assert_eq!(sum.hz(), 789.0 + 456000.0 + 123000000.0);
    }

    #[test]
    fn test_sub() {
        let f1 = Frequency::from_hz(789.0);
        let f2 = Frequency::from_khz(456.0);
        let f3 = Frequency::from_mhz(123.0);
        let diff = f1 - f2 - f3;
        assert_eq!(diff.hz(), 789.0 - 456000.0 - 123000000.0);
    }

    #[test]
    fn test_to_pretty_str() {
        let f1 = Frequency::from_hz(789.0);
        let f2 = Frequency::from_khz(456.0);
        let f3 = Frequency::from_mhz(123.0);
        assert_eq!(f1.to_pretty_str(), "789 Hz");
        assert_eq!(f2.to_pretty_str(), "456 kHz");
        assert_eq!(f3.to_pretty_str(), "123 MHz");
        let sum = f1 + f2 + f3;
        assert_eq!(sum.to_pretty_str(), "123.456789 MHz");
        let f4 = Frequency::from_hz(12.3);
        let f5 = Frequency::from_khz(45.6);
        let f6 = Frequency::from_mhz(78.9);
        assert_eq!(f4.to_pretty_str(), "12.3 Hz");
        assert_eq!(f5.to_pretty_str(), "45.6 kHz");
        assert_eq!(f6.to_pretty_str(), "78.9 MHz");
    }

    #[test]
    fn test_from_x_str() {
        let f1 = Frequency::from_hz_str("67.0");
        assert_eq!(f1.unwrap().hz(), 67.0);
        let f2 = Frequency::from_hz_str("71.9Hz");
        assert_eq!(f2.unwrap().hz(), 71.9);
        let f3 = Frequency::from_hz_str("74.4 Hz");
        assert_eq!(f3.unwrap().hz(), 74.4);
        let f4 = Frequency::from_hz_str("-88.5");
        assert_eq!(f4.unwrap().hz(), -88.5);

        let f5 = Frequency::from_khz_str("103.5");
        assert_eq!(f5.unwrap().khz(), 103.5);
        let f6 = Frequency::from_mhz_str("123.4");
        assert_eq!(f6.unwrap().mhz(), 123.4);
        let f7 = Frequency::from_ghz_str("1.234");
        assert_eq!(f7.unwrap().ghz(), 1.234);
    }
}
