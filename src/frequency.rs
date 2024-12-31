// src/frequency.rs

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum FrequencyUnit {
    Millihertz,
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
            FrequencyUnit::Millihertz => (value * 1_000.0) as u64,
            FrequencyUnit::Hertz =>      (value * 1_000_000.0) as u64,
            FrequencyUnit::Kilohertz =>  (value * 1_000_000.0 * 1_000.0) as u64,
            FrequencyUnit::Megahertz =>  (value * 1_000_000.0 * 1_000_000.0) as u64,
            FrequencyUnit::Gigahertz =>  (value * 1_000_000.0 * 1_000_000.0 * 1_000.0) as u64,
        };
        Frequency { value_uhz }
    }

    pub fn as_millihz(&self) -> f64 {
        self.value_uhz as f64 / 1_000.0
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

    pub fn from_millihz(value: f64) -> Self {
        Frequency {
            value_uhz: (value * 1_000.0) as u64,
        }
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
        } else if self.value_uhz >= 1_000 {
            write!(f, "{} Hz", self.as_hz())
        } else {
            write!(f, "{} mHz", self.as_millihz())
        }
    }
}

// Custom serializer for u64 representing frequency
mod serde_freq {
    use serde::{Deserialize, Deserializer, Serializer};
    use super::FrequencyUnit;

    pub fn serialize_u64_as_f64<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(value as f64 / 1_000_000.0) // Convert to MHz for serialization
    }

    pub fn deserialize_u64_as_f64<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Ok((value * 1_000_000.0) as u64) // Convert back to uHz for deserialization
    }
}

impl serde::Serialize for u64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde_freq::serialize_u64_as_f64(self, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for u64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde_freq::deserialize_u64_as_f64(deserializer)
    }
}
