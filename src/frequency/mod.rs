// Fixed-point frequency representation
// Author: Akira Youngblood 2025

use std::fmt;
use std::num::ParseFloatError;
use std::ops::{Add, Sub, Mul, Div};
use serde::{Serialize,Deserialize,Serializer,Deserializer};
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Frequency {
    pub freq_uhz: i64,
}

#[allow(unused)]
impl Frequency {
    pub fn new() -> Self {
        Frequency { freq_uhz: 0 }
    }

    pub fn from_hz(freq_hz: f64) -> Self {
        Frequency { freq_uhz: (freq_hz * 1e6) as i64 }
    }
    pub fn from_khz(freq_khz: f64) -> Self {
        Frequency { freq_uhz: (freq_khz * 1e9) as i64 }
    }
    pub fn from_mhz(freq_mhz: f64) -> Self {
        Frequency { freq_uhz: (freq_mhz * 1e12) as i64 }
    }
    pub fn from_ghz(freq_ghz: f64) -> Self {
        Frequency { freq_uhz: (freq_ghz * 1e15) as i64 }
    }

    pub fn hz(&self) -> f64 {
        self.freq_uhz as f64 / 1e6
    }
    pub fn khz(&self) -> f64 {
        self.freq_uhz as f64 / 1e9
    }
    pub fn mhz(&self) -> f64 {
        self.freq_uhz as f64 / 1e12
    }
    pub fn ghz(&self) -> f64 {
        self.freq_uhz as f64 / 1e15
    }

    pub fn to_pretty_str(&self) -> String {
        if self.hz() < 1e3 {
            format!("{:.} Hz", self.hz())
        } else if self.hz() < 1e6 {
            format!("{:.} kHz", self.khz())
        } else if self.hz() < 1e9 {
            format!("{:.} MHz", self.mhz())
        } else {
            format!("{:.} GHz", self.ghz())
        }
    }

    pub fn to_pretty_str_fixed(&self) -> String {
        if self.hz() < 1e3 {
            format!("{:8.4} Hz", self.hz())
        } else if self.hz() < 1e6 {
            format!("{:8.4} kHz", self.khz())
        } else if self.hz() < 1e9 {
            format!("{:8.4} MHz", self.mhz())
        } else {
            format!("{:8.4} GHz", self.ghz())
        }
    }

    pub fn from_hz_str(str: &str) -> Result<Self, ParseFloatError> {
        str.chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect::<String>()
            .parse()
            .map(|freq_hz: f64| Frequency { freq_uhz: (freq_hz * 1e6) as i64 })
    }
    pub fn from_khz_str(str: &str) -> Result<Self, ParseFloatError> {
        str.chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect::<String>()
            .parse()
            .map(|freq_khz: f64| Frequency { freq_uhz: (freq_khz * 1e9) as i64 })
    }
    pub fn from_mhz_str(str: &str) -> Result<Self, ParseFloatError> {
        str.chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect::<String>()
            .parse()
            .map(|freq_mhz: f64| Frequency { freq_uhz: (freq_mhz * 1e12) as i64 })
    }
    pub fn from_ghz_str(str: &str) -> Result<Self, ParseFloatError> {
        str.chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect::<String>()
            .parse()
            .map(|freq_ghz: f64| Frequency { freq_uhz: (freq_ghz * 1e15) as i64 })
    }

    pub fn abs(&self) -> Self {
        if self.freq_uhz < 0 {
            Frequency { freq_uhz: -self.freq_uhz }
        } else {
            *self
        }
    }
}

impl Add for Frequency {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Frequency { freq_uhz: self.freq_uhz + other.freq_uhz }
    }
}

impl Sub for Frequency {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Frequency { freq_uhz: self.freq_uhz - other.freq_uhz }
    }
}

impl Mul<f64> for Frequency {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Frequency { freq_uhz: (self.freq_uhz as f64 * other) as i64 }
    }
}

impl Div<f64> for Frequency {
    type Output = Self;

    fn div(self, other: f64) -> Self {
        Frequency { freq_uhz: (self.freq_uhz as f64 / other) as i64 }
    }
}

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(precision) = f.precision() {
                // If a precision was specified, format the f64 with that precision
                write!(f, "{:.width$} Hz", self.hz(), width = precision)
            } else {
                // Otherwise, use the default formatting for Display
                write!(f, "{} Hz", self.hz())
            }
    }
}

// Custom serialization and deserialization using fixed point
impl Serialize for Frequency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let whole_hz = self.freq_uhz / 1_000_000;
        let fractional_hz = self.freq_uhz % 1_000_000;
        let display_fractional_hz = fractional_hz.abs();
        let formatted_hz = format!("{}.{:06}", whole_hz, display_fractional_hz);
        let mut state = serializer.serialize_struct("Frequency", 1)?;
        state.serialize_field("freq_hz", &formatted_hz)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Frequency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FrequencyVisitor;

        impl<'de> Visitor<'de> for FrequencyVisitor {
            type Value = Frequency;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a struct with a single field named `freq_hz`")
            }
            fn visit_map<V>(self, mut map: V) -> Result<Frequency, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut freq_hz_str: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "freq_hz" => {
                            if freq_hz_str.is_some() {
                                return Err(de::Error::duplicate_field("freq_hz"));
                            }
                            freq_hz_str = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                let s = freq_hz_str.ok_or_else(|| de::Error::missing_field("freq_hz"))?;

                let parts: Vec<&str> = s.split('.').collect();
                if parts.len() > 2 {
                    return Err(de::Error::custom("Invalid fixed-point number format: too many decimal points"));
                }

                let mut whole_part_str = parts[0];
                let mut fractional_part_str = "";

                if parts.len() == 2 {
                    fractional_part_str = parts[1];
                }

                let is_negative = whole_part_str.starts_with('-');
                if whole_part_str.starts_with('+') {
                    whole_part_str = &whole_part_str[1..];
                }

                let mut whole_uhz = whole_part_str.parse::<i64>()
                    .map_err(|e| de::Error::custom(format!("Invalid whole part: {}", e)))?;
                whole_uhz *= 1_000_000;

                let mut fractional_uhz = 0_i64;
                if !fractional_part_str.is_empty() {
                    let mut digits_str = fractional_part_str;
                    if digits_str.len() > 6 {
                        digits_str = &fractional_part_str[..6];
                    }

                    fractional_uhz = digits_str.parse::<i64>()
                        .map_err(|e| de::Error::custom(format!("Invalid fractional part: {}", e)))?;

                    let scale_factor = 10_i64.pow((6 - digits_str.len()) as u32);
                    fractional_uhz *= scale_factor;
                }

                let total_uhz = if is_negative && (whole_uhz != 0 || fractional_uhz != 0) {
                    whole_uhz - fractional_uhz
                } else {
                    whole_uhz + fractional_uhz
                };

                Ok(Frequency { freq_uhz: total_uhz })
            }
        }

        deserializer.deserialize_struct("Frequency", &["freq_hz"], FrequencyVisitor)
    }
}

// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::{min, max};

    #[test]
    fn test_new_frequency() {
        let freq = Frequency::new();
        assert_eq!(freq.freq_uhz, 0);
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
    fn test_mul() {
        let f1 = Frequency::from_hz(789.0);
        let f2 = Frequency::from_khz(456.0);
        let product = f1 * 1000.0;
        assert_eq!(product, Frequency::from_khz(789.0));
        let product = f2 * 11.0;
        assert_eq!(product, Frequency::from_khz(5016.0));
    }

    #[test]
    fn test_div() {
        let f1 = Frequency::from_hz(789.0);
        let f2 = Frequency::from_khz(456.0);
        let quotient = f1 / 1000.0;
        assert_eq!(quotient, Frequency::from_hz(0.789));
        let quotient = f2 / 11.0;
        assert_eq!(quotient, Frequency::from_khz(41.45454545454545));
    }

    #[test]
    fn test_abs() {
        let f1 = Frequency::from_hz(-789.0);
        let f2 = Frequency::from_khz(456.0);
        let f3 = Frequency::from_mhz(-123.0);
        assert_eq!(f1.abs(), Frequency::from_hz(789.0));
        assert_eq!(f2.abs(), Frequency::from_khz(456.0));
        assert_eq!(f3.abs(), Frequency::from_mhz(123.0));
    }

    #[test]
    fn test_compare() {
        let f1 = Frequency::from_hz(0.789);
        let f2 = Frequency::from_khz(456.0);
        let f3 = Frequency::from_mhz(123.0);
        let f4 = Frequency::from_hz(0.789);
        assert!(f1 < f2);
        assert!(f3 > f2);
        assert!(f3 >= f1);
        assert!(f1 <= f3);
        assert!(f1 == f4);
        assert!(f1 != f2);
    }

    #[test]
    fn test_min_max_sort() {
        let mut freqs = vec![
            Frequency::from_hz(1.15),
            Frequency::from_khz(25.55575),
            Frequency::from_mhz(1.0),
            Frequency::from_khz(1.1),
            Frequency::from_hz(0.789),
        ];
        let min = min(freqs[0], freqs[1]);
        let max = max(freqs[1], freqs[2]);
        assert_eq!(min, Frequency::from_hz(1.15));
        assert_eq!(max, Frequency::from_mhz(1.0));
        let vec_max = freqs.iter().max().unwrap();
        assert_eq!(vec_max, &Frequency::from_mhz(1.0));
        let vec_min = freqs.iter().min().unwrap();
        assert_eq!(vec_min, &Frequency::from_hz(0.789));
        freqs.sort();
        assert_eq!(freqs, vec![
            Frequency::from_hz(0.789),
            Frequency::from_hz(1.15),
            Frequency::from_khz(1.1),
            Frequency::from_khz(25.55575),
            Frequency::from_mhz(1.0),
        ]);
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

    #[test]
    fn test_serialize_deserialize() {
        let f1 = Frequency::from_hz(0.0);
        let json = serde_json::to_string(&f1).unwrap();
        let deserialized = serde_json::from_str(&json).unwrap();
        assert_eq!(f1, deserialized);
        let f2 = Frequency::from_hz(123.456789);
        let json = serde_json::to_string(&f2).unwrap();
        let deserialized = serde_json::from_str(&json).unwrap();
        assert_eq!(f2, deserialized);
        let f3 = Frequency::from_khz(987.654321001);
        let json = serde_json::to_string(&f3).unwrap();
        let deserialized = serde_json::from_str(&json).unwrap();
        assert_eq!(f3, deserialized);

    }
}
