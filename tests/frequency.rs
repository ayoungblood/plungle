#[path = "../src/frequency.rs"] mod frequency;

use crate::frequency::{Frequency, FrequencyUnit};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new_hertz() {
        let freq = Frequency::new(100.0, FrequencyUnit::Hertz);
        assert_eq!(freq.value_uhz, 100_000_000);
    }

    #[test]
    fn test_new_kilohertz() {
        let freq = Frequency::new(150.0, FrequencyUnit::Kilohertz);
        assert_eq!(freq.value_uhz, 150_000_000);
    }

    #[test]
    fn test_new_megahertz() {
        let freq = Frequency::new(200.0, FrequencyUnit::Megahertz);
        assert_eq!(freq.value_uhz, 200_000_000_000);
    }

    #[test]
    fn test_new_gigahertz() {
        let freq = Frequency::new(3.0, FrequencyUnit::Gigahertz);
        assert_eq!(freq.value_uhz, 3_000_000_000_000);
    }

    #[test]
    fn test_as_hz() {
        let freq = Frequency::new(123.45, FrequencyUnit::Megahertz);
        assert_eq!(freq.as_hz(), 123.45);
    }

    #[test]
    fn test_as_khz() {
        let freq = Frequency::new(567.8, FrequencyUnit::Megahertz);
        assert_eq!(freq.as_khz(), 567_800.0);
    }

    #[test]
    fn test_as_mhz() {
        let freq = Frequency::new(890.123, FrequencyUnit::Gigahertz);
        assert_eq!(freq.as_mhz(), 890_123.0);
    }

    #[test]
    fn test_as_ghz() {
        let freq = Frequency::new(1.5, FrequencyUnit::Kilohertz);
        assert_eq!(freq.as_ghz(), 0.000_001_5);
    }

    #[test]
    fn test_from_hz() {
        let freq = Frequency::from_hz(250.0);
        assert_eq!(freq.value_uhz, 250_000_000);
    }

    #[test]
    fn test_from_khz() {
        let freq = Frequency::from_khz(789.0);
        assert_eq!(freq.value_uhz, 789_000_000);
    }

    #[test]
    fn test_from_mhz() {
        let freq = Frequency::from_mhz(1.234);
        assert_eq!(freq.value_uhz, 1_234_000_000);
    }

    #[test]
    fn test_from_ghz() {
        let freq = Frequency::from_ghz(0.005);
        assert_eq!(freq.value_uhz, 5_000_000);
    }

    #[test]
    fn test_display_hertz() {
        let freq = Frequency::new(1234.56, FrequencyUnit::Hertz);
        assert_eq!(format!("{}", freq), "1.23456 kHz");
    }

    #[test]
    fn test_display_kilohertz() {
        let freq = Frequency::new(7890.0, FrequencyUnit::Hertz);
        assert_eq!(format!("{}", freq), "7.89 kHz");
    }

    #[test]
    fn test_display_megahertz() {
        let freq = Frequency::new(156.7, FrequencyUnit::Megahertz);
        assert_eq!(format!("{}", freq), "156.7 MHz");
    }

    #[test]
    fn test_display_gigahertz() {
        let freq = Frequency::new(2.345, FrequencyUnit::Gigahertz);
        assert_eq!(format!("{}", freq), "2.345 GHz");
    }
}
