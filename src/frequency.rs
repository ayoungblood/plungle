// src/frequency.rs

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum FrequencyUnit {
    Hertz,
    Kilohertz,
    Megahertz,
    Gigahertz,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Frequency {
    value_uhz: u64, // Frequency value in microhertz (uHz)
}

impl Frequency {
    pub fn new(value: f64, unit: FrequencyUnit) -> Self {
        let value_uhz = match unit {
            FrequencyUnit::Hertz => (value * 1_000_000.0) as u64,
            FrequencyUnit::Kilohertz => (value * 1_000_000.0 * 1_000.0) as u64,
            FrequencyUnit::Megahertz => (value * 1_000_000.0 * 1_000_000.0) as u64,
            FrequencyUnit::Gigahertz => (value * 1_000_000.0 * 1_000_000.0 * 1_000.0) as u64,
        };
        Frequency { value_uhz }
    }
    pub fn as_hz(&self) -> f64 {
        self.value_uhz as f64 / 1_000_000.0
    }

    pub fn as_khz(&self) -> f64 {
        self.value_uhz as f64 / 1_000_000.0 / 1_000.0
    }

    pub fn as_mhz(&self) -> f64 {
        self.value_uhz as f64 / 1_000_000.0 / 1_000_000.0
    }

    pub fn as_ghz(&self) -> f64 {
        self.value_uhz as f64 / 1_000_000.0 / 1_000_000_000.0
    }

    pub fn from_hz(value: f64) -> Self {
        Frequency {
            value_uhz: (value * 1_000_000.0) as u64,
        }
    }

    pub fn from_khz(value: f64) -> Self {
        Frequency {
            value_uhz: (value * 1_000_000.0 * 1_000.0) as u64,
        }
    }

    pub fn from_mhz(value: f64) -> Self {
        Frequency {
            value_uhz: (value * 1_000_000_000.0) as u64,
        }
    }

    pub fn from_ghz(value: f64) -> Self {
        Frequency {
            value_uhz: (value * 1_000_000_000.0 * 1_000.0) as u64,
        }
    }
}

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Determine the appropriate unit for display
        if self.value_uhz >= 1_000_000_000_000 {
            write!(f, "{} GHz", self.as_ghz())
        } else if self.value_uhz >= 1_000_000_000 {
            write!(f, "{} MHz", self.as_mhz())
        } else if self.value_uhz >= 1_000_000 {
            write!(f, "{} kHz", self.as_khz())
        } else {
            write!(f, "{} Hz", self.as_hz())
        }
    }
}
