// src/validate.rs

use rust_decimal::prelude::*;
use crate::*;
use crate::bandplan::Bandplan;

/// Severity
#[derive(Debug, Default, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    #[default]
    Info,
}

/// Complaint
#[derive(Debug, Default)]
pub struct Complaint {
    pub severity: Severity,
    pub message: String,
    pub source_index: Option<usize>,
    pub source_name: Option<String>,
}

// this function performs validation steps that are common across all radios
pub fn validate_generic(codeplug: &structures::Codeplug, bandplan: &Bandplan, opt: &Opt) -> Result<Vec<Complaint>, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 1, "Validating codeplug against bandplan: {} (source: {})", bandplan.name, bandplan.source.as_ref().unwrap());
    let mut complaints: Vec<Complaint> = Vec::new();
    // validate the codeplug
    if codeplug.channels.is_empty() {
        complaints.push(Complaint {
            severity: Severity::Error,
            message: format!("Codeplug has no channels"),
            source_index: None,
            source_name: None,
        });
    }
    for channel in &codeplug.channels {
        if channel.name.is_empty() {
            complaints.push(Complaint {
                severity: Severity::Error,
                message: format!("Name is empty"),
                source_index: Some(channel.index),
                source_name: Some(channel.name.clone()),
            });
        }
        let rx_band = bandplan::get_band(bandplan, channel.frequency_rx);
        let tx_band = bandplan::get_band(bandplan, channel.frequency_tx);
        // warn less strongly if we don't know the RX band
        if rx_band.is_none() {
            complaints.push(Complaint {
                severity: Severity::Info,
                message: format!("Unrecognized RX band: {}", freq2str(&channel.frequency_rx)),
                source_index: Some(channel.index),
                source_name: Some(channel.name.clone()),
            });
        }
        // warn if we don't know the TX band, but only if the channel is not RX only
        if tx_band.is_none() && !channel.rx_only {
            complaints.push(Complaint {
                severity: Severity::Warning,
                message: format!("Unrecognized TX band: {}", freq2str(&channel.frequency_tx)),
                source_index: Some(channel.index),
                source_name: Some(channel.name.clone()),
            });
        }
        // if we have both bands
        if !rx_band.is_none() && !tx_band.is_none() {
            // warn on crossband
            if rx_band != tx_band {
                complaints.push(Complaint {
                    severity: Severity::Warning,
                    message: format!("Crossband: {} rx: {}", freq2str(&channel.frequency_tx), freq2str(&channel.frequency_rx)),
                    source_index: Some(channel.index),
                    source_name: Some(channel.name.clone()),
                });
            } else { // if not crossband, check the offset
                // compute the difference between RX and TX
                let diff = (channel.frequency_tx - channel.frequency_rx).abs();
                // if diff is non-zero and we have a nominal offset, warn
                if diff != Decimal::new(0, 0) && tx_band.as_ref().unwrap().nominal_offsets.is_some() {
                    // check if diff matches one of the offsets
                    let mut matched = false;
                    for offset in tx_band.as_ref().unwrap().nominal_offsets.as_ref().unwrap() {
                        if diff == *offset {
                            matched = true;
                            break;
                        }
                    }
                    if !matched {
                        complaints.push(Complaint {
                            severity: Severity::Warning,
                            message: format!("Unusual offset: {} (tx: {} rx: {})", freq2str(&diff), freq2str(&channel.frequency_tx), freq2str(&channel.frequency_rx)),
                            source_index: Some(channel.index),
                            source_name: Some(channel.name.clone()),
                        });
                    }
                }
            }

            // if TX enabled, check if we're transmitting outside the amateur bands
            if !channel.rx_only {
                if tx_band.as_ref().unwrap().is_amateur == false {
                    // check if it's a known non-amateur band
                    if tx_band.as_ref().unwrap().name == "MURS" {
                        // warn less strongly about MURS
                        complaints.push(Complaint {
                            severity: Severity::Info,
                            message: format!("TX on MURS: {}", freq2str(&channel.frequency_tx)),
                            source_index: Some(channel.index),
                            source_name: Some(channel.name.clone()),
                        });
                    } else if tx_band.as_ref().unwrap().name == "FRS/GMRS" {
                        // warn less strongly about FRS/GMRS
                        complaints.push(Complaint {
                            severity: Severity::Info,
                            message: format!("TX on FRS/GMRS: {}", freq2str(&channel.frequency_tx)),
                            source_index: Some(channel.index),
                            source_name: Some(channel.name.clone()),
                        });
                    } else {
                        complaints.push(Complaint {
                            severity: Severity::Error,
                            message: format!("TX outside amateur band: {}", freq2str(&channel.frequency_tx)),
                            source_index: Some(channel.index),
                            source_name: Some(channel.name.clone()),
                        });
                    }
                }
            }
        }
    }
    Ok(complaints)
}

pub fn validate_specific(codeplug: &structures::Codeplug, props: &structures::RadioProperties, opt: &Opt) -> Result<Vec<Complaint>, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    let mut complaints: Vec<Complaint> = Vec::new();
    // check codeplug
    if codeplug.channels.len() > props.channels_max as usize {
        complaints.push(Complaint {
            severity: Severity::Error,
            message: format!("Too many channels: {} (max: {})", codeplug.channels.len(), props.channels_max),
            source_index: None,
            source_name: None,
        });
    }
    if codeplug.zones.len() > props.zones_max as usize {
        complaints.push(Complaint {
            severity: Severity::Error,
            message: format!("Too many zones: {} (max: {})", codeplug.zones.len(), props.zones_max),
            source_index: None,
            source_name: None,
        });
    }
    // check channels
    for channel in &codeplug.channels {
        if channel.name.len() > props.channel_name_width_max  {
            complaints.push(Complaint {
                severity: Severity::Warning,
                message: format!("Name is too long (len: {})", channel.name.len()),
                source_index: Some(channel.index),
                source_name: Some(channel.name.clone()),
            });
        }
        if !props.modes.contains(&channel.mode) {
            complaints.push(Complaint {
                severity: Severity::Error,
                message: format!("Unsupported channel mode: {:?}", channel.mode),
                source_index: Some(channel.index),
                source_name: Some(channel.name.clone()),
            });
        }
    }
    // check zones
    for zone in &codeplug.zones {
        if zone.name.len() > props.zone_name_width_max  {
            complaints.push(Complaint {
                severity: Severity::Warning,
                message: format!("Zone name is too long (len: {})", zone.name.len()),
                source_index: None,
                source_name: Some(zone.name.clone()),
            });
        }
    }
    Ok(complaints)
}

pub fn print_complaints(complaints: &Vec<Complaint>, opt: &Opt) {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, Color::Magenta, None, "{:-^1$}", " Validation Output ", 79);

    // print the complaints
    for complaint in complaints {
        let line;
        if complaint.source_index.is_some() && complaint.source_name.is_some() {
            line = format!("{:4} {:24} {}", complaint.source_index.unwrap(), complaint.source_name.as_ref().unwrap(), complaint.message);
        } else {
            line = format!("{}", complaint.message);
        }
        match complaint.severity {
            Severity::Error => {
                uprintln!(opt, Stderr, Color::Red, None, "[Error  ] {}", line);
            },
            Severity::Warning => {
                uprintln!(opt, Stderr, Color::Yellow, None, "[Warning] {}", line);
            },
            Severity::Info => {
                uprintln!(opt, Stderr, Color::Cyan, None, "[Info   ] {}", line);
            },
        }
    }
    // total everything up
    let error_count = complaints.iter().filter(|c| c.severity == Severity::Error).count();
    let warning_count = complaints.iter().filter(|c| c.severity == Severity::Warning).count();
    let info_count = complaints.iter().filter(|c| c.severity == Severity::Info).count();
    uprintln!(opt, Stderr, Color::Magenta, None, "{}", "- ".repeat(40));
    if error_count > 0 {
        uprintln!(opt, Stderr, Color::Red, None, "Validation: {} errors, {} warnings, {} infos", error_count, warning_count, info_count);
    } else if warning_count > 0 {
        uprintln!(opt, Stderr, Color::Yellow, None, "Validation: {} errors, {} warnings, {} infos", error_count, warning_count, info_count);
    } else {
        uprintln!(opt, Stderr, Color::Cyan, None, "Validation: {} errors, {} warnings, {} infos", error_count, warning_count, info_count);
    }
    uprintln!(opt, Stderr, Color::Magenta, None, "{}", "-".repeat(79));
}
