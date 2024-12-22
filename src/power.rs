// src/power.rs

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum PowerUnit {
    Milliwatt,
    Watt,
    Kilowatt,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Power {
    value_uw: u64, // Power value in microwatts (uW)
}

impl Power {
    pub fn new(value: f64, unit: PowerUnit) -> Self {
        let value_uw = match unit {
            PowerUnit::Milliwatt => (value * 1_000.0) as u64,
            PowerUnit::Watt => (value * 1_000_000.0) as u64,
            PowerUnit::Kilowatt => (value * 1_000_000.0 * 1_000.0) as u64,
        };
        Power { value_uw }
    }
    pub fn as_mw(&self) -> f64 {
        self.value_uw as f64 / 1_000.0
    }
    pub fn as_w(&self) -> f64 {
        self.value_uw as f64 / 1_000_000.0
    }
    pub fn as_kw(&self) -> f64 {
        self.value_uw as f64 / 1_000.0 / 1_000.0
    }

    pub fn from_mw(value: f64) -> Self {
        Power {
            value_uw: (value * 1_000.0) as u64,
        }
    }

    pub fn from_w(value: f64) -> Self {
        Power {
            value_uw: (value * 1_000_000.0) as u64,
        }
    }
}

impl fmt::Display for Power {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Determine the appropriate unit for display
        if self.value_uw >= 1_000_000 {
            write!(f, "{} W", self.as_w())
        } else if self.value_uw >= 1_000 {
            write!(f, "{} mW", self.as_mw())
        } else {
            write!(f, "{} uW", self.value_uw)
        }
    }
}
