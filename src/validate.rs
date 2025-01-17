// src/validate.rs

use crate::*;
use rust_decimal::prelude::*;

// this function performs validation steps that are common across all radios
pub fn validate_generic(codeplug: &structures::Codeplug, opt: &Opt) -> Result<(), Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    let mut error_count: usize = 0;
    let mut warning_count: usize = 0;
    let mut info_count: usize = 0;
    // validate the codeplug
    if codeplug.channels.is_empty() {
        cprintln!(ANSI_C_RED, "Codeplug has no channels");
        error_count += 1;
    }
    for channel in &codeplug.channels {
        if channel.name.is_empty() {
            cprintln!(ANSI_C_RED, "Error:   Channel {:4} {:24} Name is empty", channel.index, channel.name);
            error_count += 1;
        }
        if channel.name.len() > 24 {
            cprintln!(ANSI_C_YLW, "Warning: Channel {:4} {:24} Name is too long (len: {})", channel.index, channel.name, channel.name.len());
            warning_count += 1;
        }
        let rx_band = get_band(channel.frequency_rx);
        let tx_band = get_band(channel.frequency_rx);
        // warn less strongly if we don't know the RX band
        if rx_band.is_none() {
            cprintln!(ANSI_C_CYN, "Info:    Channel {:4} {:24} Unrecognized RX band: {}", channel.index, channel.name, freq2str(&channel.frequency_rx));
            info_count += 1;
        }
        // warn if we don't know the TX band, but only if the channel is not RX only
        if tx_band.is_none() && !channel.rx_only {
            cprintln!(ANSI_C_YLW, "Warning: Channel {:4} {:24} Unrecognized TX band: {}", channel.index, channel.name, freq2str(&channel.frequency_tx));
            warning_count += 1;
        }
        // if we have both bands
        if !rx_band.is_none() && !tx_band.is_none() {
            // warn on crossband
            if rx_band != tx_band {
                cprintln!(ANSI_C_YLW, "Warning: Channel {:4} {:24} RX/TX band mismatch tx: {} rx: {}", channel.index, channel.name, freq2str(&channel.frequency_tx), freq2str(&channel.frequency_rx));
                warning_count += 1;
            } else { // if not crossband, check the offset
                // if we have a nominal offset, check if it matches the difference between RX and TX
                let mut offset = Decimal::new(0, 0);
                if rx_band.as_ref().unwrap().nominal_offset.is_some() {
                    offset = rx_band.as_ref().unwrap().nominal_offset.unwrap();
                }
                if tx_band.as_ref().unwrap().nominal_offset.is_some() {
                    offset = tx_band.as_ref().unwrap().nominal_offset.unwrap();
                }
                if offset != Decimal::new(0, 0) {
                    let diff = (channel.frequency_tx - channel.frequency_rx).abs();
                    if diff != Decimal::new(0, 0) && diff != offset {
                        cprintln!(ANSI_C_YLW, "Warning: Channel {:4} {:24} Nominal offset mismatch: {} (tx: {} rx: {})", channel.index, channel.name, freq2str(&diff), freq2str(&channel.frequency_tx), freq2str(&channel.frequency_rx));
                        warning_count += 1;
                    }
                }
            }

            // if we have a TX band, check if we're transmitting outside the amateur bands
            if !tx_band.is_none() && !channel.rx_only {
                if tx_band.as_ref().unwrap().is_amateur == false {
                    // check if it's a known non-amateur band
                    if tx_band.as_ref().unwrap().name == "MURS" {
                        // warn less strongly about MURS
                        cprintln!(ANSI_C_CYN, "Info:    Channel {:4} {:24} TX on MURS: {}", channel.index, channel.name, freq2str(&channel.frequency_tx));
                        info_count += 1;
                    } else if tx_band.as_ref().unwrap().name == "FRS/GMRS" {
                        // warn less strongly about FRS/GMRS
                        cprintln!(ANSI_C_CYN, "Info:    Channel {:4} {:24} TX on FRS/GMRS: {}", channel.index, channel.name, freq2str(&channel.frequency_tx));
                        info_count += 1;
                    } else {
                        cprintln!(ANSI_C_YLW, "Warning: Channel {:4} {:24} TX outside amateur band: {}", channel.index, channel.name, freq2str(&channel.frequency_tx));
                        warning_count += 1;
                    }
                }
            }
        }
    }
    eprintln!("");
    if error_count > 0 {
        cprintln!(ANSI_C_RED, "Generic validation: {} infos, {} errors, {} warnings", info_count, error_count, warning_count);
    } else if warning_count > 0 {
        cprintln!(ANSI_C_YLW, "Generic validation: {} infos, {} errors, {} warnings", info_count, error_count, warning_count);
    } else {
        cprintln!(ANSI_C_GRN, "Generic validation: {} infos, {} errors, {} warnings", info_count, error_count, warning_count);
    }
    Ok(())
}

#[derive(Debug, PartialEq, Clone)]
pub struct Band {
    name: String,
    freq_min: Decimal,
    freq_max: Decimal,
    is_amateur: bool,
    nominal_offset: Option<Decimal>,
}

fn get_bands() -> Vec<Band> {
    let mut bands = Vec::new();
    bands.push(Band {
        name: "Amateur 2200m".to_string(),
        freq_min: Decimal::from_str("135.7").unwrap() * Decimal::new(1_000, 0), // 135.7 kHz
        freq_max: Decimal::from_str("137.8").unwrap() * Decimal::new(1_000, 0), // 137.8 kHz
        is_amateur: true,
        nominal_offset: None,
    });
    bands.push(Band {
        name: "Amateur 630m".to_string(),
        freq_min: Decimal::from_str("472.0").unwrap() * Decimal::new(1_000, 0), // 472 kHz
        freq_max: Decimal::from_str("479.0").unwrap() * Decimal::new(1_000, 0), // 479 kHz
        is_amateur: true,
        nominal_offset: None,
    });
    bands.push(Band {
        name: "Amateur 160m".to_string(),
        freq_min: Decimal::from_str("1.8").unwrap() * Decimal::new(1_000_000, 0), // 1.8 MHz
        freq_max: Decimal::from_str("2.0").unwrap() * Decimal::new(1_000_000, 0), // 2.0 MHz
        is_amateur: true,
        nominal_offset: None,
    });
    bands.push(Band {
        name: "Amateur 80m".to_string(),
        freq_min: Decimal::from_str("3.5").unwrap() * Decimal::new(1_000_000, 0), // 3.5 MHz
        freq_max: Decimal::from_str("4.0").unwrap() * Decimal::new(1_000_000, 0), // 4.0 MHz
        is_amateur: true,
        nominal_offset: None,
    });
    bands.push(Band {
        name: "Amateur 60m".to_string(),
        freq_min: Decimal::from_str("5351.5").unwrap() * Decimal::new(1_000, 0), // 5351.5 kHz
        freq_max: Decimal::from_str("5366.5").unwrap() * Decimal::new(1_000, 0), // 5366.5 kHz
        is_amateur: true,
        nominal_offset: None,
    });
    bands.push(Band {
        name: "Amateur 40m".to_string(),
        freq_min: Decimal::from_str("7.0").unwrap() * Decimal::new(1_000_000, 0), // 7.0 MHz
        freq_max: Decimal::from_str("7.3").unwrap() * Decimal::new(1_000_000, 0), // 7.3 MHz
        is_amateur: true,
        nominal_offset: None,
    });
    bands.push(Band {
        name: "Amateur 30m".to_string(),
        freq_min: Decimal::from_str("10.1").unwrap() * Decimal::new(1_000_000, 0), // 10.1 MHz
        freq_max: Decimal::from_str("10.15").unwrap() * Decimal::new(1_000_000, 0), // 10.15 MHz
        is_amateur: true,
        nominal_offset: None,
    });
    bands.push(Band {
        name: "Amateur 20m".to_string(),
        freq_min: Decimal::from_str("14.0").unwrap() * Decimal::new(1_000_000, 0), // 14.0 MHz
        freq_max: Decimal::from_str("14.35").unwrap() * Decimal::new(1_000_000, 0), // 14.35 MHz
        is_amateur: true,
        nominal_offset: None,
    });
    bands.push(Band {
        name: "Amateur 17m".to_string(),
        freq_min: Decimal::from_str("18.068").unwrap() * Decimal::new(1_000_000, 0), // 18.068 MHz
        freq_max: Decimal::from_str("18.168").unwrap() * Decimal::new(1_000_000, 0), // 18.168 MHz
        is_amateur: true,
        nominal_offset: None,
    });
    bands.push(Band {
        name: "Amateur 15m".to_string(),
        freq_min: Decimal::from_str("21.0").unwrap() * Decimal::new(1_000_000, 0), // 21.0 MHz
        freq_max: Decimal::from_str("21.45").unwrap() * Decimal::new(1_000_000, 0), // 21.45 MHz
        is_amateur: true,
        nominal_offset: None,
    });
    bands.push(Band {
        name: "Amateur 12m".to_string(),
        freq_min: Decimal::from_str("24.89").unwrap() * Decimal::new(1_000_000, 0), // 24.89 MHz
        freq_max: Decimal::from_str("24.99").unwrap() * Decimal::new(1_000_000, 0), // 24.99 MHz
        is_amateur: true,
        nominal_offset: None,
    });
    bands.push(Band {
        name: "Amateur 10m".to_string(),
        freq_min: Decimal::from_str("28.0").unwrap() * Decimal::new(1_000_000, 0), // 28.0 MHz
        freq_max: Decimal::from_str("29.7").unwrap() * Decimal::new(1_000_000, 0), // 29.7 MHz
        is_amateur: true,
        nominal_offset: Some(Decimal::from_str("100.0").unwrap() * Decimal::new(1_000, 0)), // 100 kHz
    });
    bands.push(Band {
        name: "Amateur 6m".to_string(),
        freq_min: Decimal::from_str("50.0").unwrap() * Decimal::new(1_000_000, 0), // 50.0 MHz
        freq_max: Decimal::from_str("54.0").unwrap() * Decimal::new(1_000_000, 0), // 54.0 MHz
        is_amateur: true,
        nominal_offset: Some(Decimal::from_str("1.0").unwrap() * Decimal::new(1_000_000, 0)), // 1.0 MHz
    });
    bands.push(Band {
        name: "Amateur 2m".to_string(),
        freq_min: Decimal::from_str("144.0").unwrap() * Decimal::new(1_000_000, 0), // 144.0 MHz
        freq_max: Decimal::from_str("148.0").unwrap() * Decimal::new(1_000_000, 0), // 148.0 MHz
        is_amateur: true,
        nominal_offset: Some(Decimal::from_str("600.0").unwrap() * Decimal::new(1_000, 0)), // 500 kHz
    });
    bands.push(Band {
        name: "MURS".to_string(),
        freq_min: Decimal::from_str("151.820").unwrap() * Decimal::new(1_000_000, 0), // 151.820 MHz
        freq_max: Decimal::from_str("154.600").unwrap() * Decimal::new(1_000_000, 0), // 154.600 MHz
        is_amateur: false,
        nominal_offset: None,
    });
    bands.push(Band {
        name: "Amateur 1.25m".to_string(),
        freq_min: Decimal::from_str("222.0").unwrap() * Decimal::new(1_000_000, 0), // 222.0 MHz
        freq_max: Decimal::from_str("225.0").unwrap() * Decimal::new(1_000_000, 0), // 225.0 MHz
        is_amateur: true,
        nominal_offset: Some(Decimal::from_str("1.6").unwrap() * Decimal::new(1_000_000, 0)), // 1.6 MHz
    });
    bands.push(Band {
        name: "Amateur 70cm".to_string(),
        freq_min: Decimal::from_str("420.0").unwrap() * Decimal::new(1_000_000, 0), // 420.0 MHz
        freq_max: Decimal::from_str("450.0").unwrap() * Decimal::new(1_000_000, 0), // 450.0 MHz
        is_amateur: true,
        nominal_offset: Some(Decimal::from_str("5.0").unwrap() * Decimal::new(1_000_000, 0)), // 5.0 MHz
    });
    bands.push(Band {
        name: "FRS/GMRS".to_string(),
        freq_min: Decimal::from_str("462.550").unwrap() * Decimal::new(1_000_000, 0), // 462.5625 MHz
        freq_max: Decimal::from_str("467.725").unwrap() * Decimal::new(1_000_000, 0), // 467.7250 MHz
        is_amateur: false,
        nominal_offset: Some(Decimal::from_str("5.0").unwrap() * Decimal::new(1_000_000, 0)), // 5.0 MHz
    });
    bands.push(Band {
        name: "Amateur 33cm".to_string(),
        freq_min: Decimal::from_str("902.0").unwrap() * Decimal::new(1_000_000, 0), // 902.0 MHz
        freq_max: Decimal::from_str("928.0").unwrap() * Decimal::new(1_000_000, 0), // 928.0 MHz
        is_amateur: true,
        nominal_offset: Some(Decimal::from_str("25.0").unwrap() * Decimal::new(1_000_000, 0)), // 25.0 MHz // @TODO FIXME
    });
    bands.push(Band {
        name: "Amateur 23cm".to_string(),
        freq_min: Decimal::from_str("1240.0").unwrap() * Decimal::new(1_000_000, 0), // 1240.0 MHz
        freq_max: Decimal::from_str("1300.0").unwrap() * Decimal::new(1_000_000, 0), // 1300.0 MHz
        is_amateur: true,
        nominal_offset: None,
    });
    bands
}


pub fn get_band(freq: Decimal) -> Option<Band> {
    let bands = get_bands();
    for band in bands {
        if freq >= band.freq_min && freq <= band.freq_max {
            return Some(band);
        }
    }
    None
}