// src/radios/opengd77_rt3s.rs
// reference https://burntsushi.net/csv/ for CSV parsing technique

use std::error::Error;
// use std::fs;
use std::path::PathBuf;
use std::path::Path;
use std::collections::HashMap;
use rust_decimal::prelude::*;
use std::sync::OnceLock;

use crate::*;
use crate::structures::*;

static PROPS: OnceLock<structures::RadioProperties> = OnceLock::new();
pub fn get_props() -> &'static structures::RadioProperties {
    PROPS.get_or_init(|| {
        let mut props = structures::RadioProperties::default();
        props.modes = vec![structures::ChannelMode::AM, structures::ChannelMode::FM];
        props.channels_max = 1024;
        props.channel_name_width_max = 16;
        props.zones_max = 0; // chirp doesn't support zones
        props.zone_name_width_max = 0;
        // dynamically set
        props.channel_index_width = (props.channels_max as f64).log10().ceil() as usize;
        props.zone_index_width = (props.zones_max as f64).log10().ceil() as usize;
        props
    })
}

// CSV Export Format
// CHIRP next-20241108
// CHIRP exports a single CSV file:
// - Location: channel index
// - Name: channel name
// - Frequency: receive frequency in MHz
// - Duplex: [+, -, (blank), split, off]
// - Offset: transmit offset in MHz, typ [0, 0.6, 5]
// - Tone: [none,Tone,TSQL,DTCS,Cross]
// - rToneFreq: RX CTCSS frequency in Hz, 88.5 default
// - cToneFreq: TX(?) CTCSS frequency in Hz, 88.5 default
// - DtcsCode: DCS code, 23 default
// - DtcsPolarity: DCS polarity, NN default
// - RxDtcsCode: RX DCS code, 23 default
// - CrossMode: [Tone->Tone,Tone->DTCS,DTCS->Tone,->Tone,->DTCS,DTCS->,DTCS->DTCS]
// - Mode: [FM, NFM, ??]
// - TStep: default 5
// - Skip: [(blank), ??]
// - Power: power in watts with W suffix, e.g. [1.0W, 4.0W, 50W]
// - Comment: blank by default
// - URCALL: blank by default
// - RPT1CALL: blank by default
// - RPT2CALL: blank by default
// - DVCODE: blank by default

type CsvRecord = HashMap<String, String>;

// READ ///////////////////////////////////////////////////////////////////////

// Chirp makes it horrifically difficult to parse tones
// See https://chirpmyradio.com/projects/chirp/wiki/MemoryEditorColumns
// and https://chirpmyradio.com/projects/chirp/wiki/DevelopersToneModes
// Practically, we support the following Tone Modes: [Tone,TSQL,DTCS,Cross]
// return a tuple with rx and tx tones, in that order
fn parse_tones(record: &CsvRecord) -> Result<(Option<Tone>, Option<Tone>), Box<dyn Error>> {
    let tone_mode = record.get("Tone").unwrap();
    let cross_mode = record.get("CrossMode").unwrap();
    if tone_mode == "" {
        Ok((None, None))
    } else if tone_mode == "Tone" { // rx is carrier squelch, tx is CTCSS (rTone)
        let tone_tx = Tone {
            mode: ToneMode::CTCSS,
            ctcss: Some(Decimal::from_str(record.get("rToneFreq").unwrap()).unwrap()),
            dcs: None,
        };
        Ok((None, Some(tone_tx)))
    } else if tone_mode == "TSQL" { // rx is CTCSS (cTone or rTone), tx is same CTCSS
        let tstr = if record.get("cToneFreq").unwrap() == "" {
            record.get("rToneFreq").unwrap()
        } else {
            record.get("cToneFreq").unwrap()
        };
        let tone = Tone {
            mode: ToneMode::CTCSS,
            ctcss: Some(Decimal::from_str(tstr).unwrap()),
            dcs: None,
        };
        Ok((Some(tone.clone()), Some(tone)))
    } else if tone_mode == "DTCS" { // rx is DCS, tx is same DCS
        let pstr;
        let dstr = if record.get("RxDtcsCode").unwrap() == "" {
            // first character of DtcsPolarity is DCS polarity, but needs to be mapped from NR to NI
            pstr = match record.get("DtcsPolarity").unwrap().chars().nth(0).unwrap() {
                'N' => "N",
                'R' => "I",
                _ => "N",
            };
            record.get("DtcsCode").unwrap()
        } else {
            // second character of DtcsPolarity is DCS polarity, but needs to be mapped from NR to NI
            pstr = match record.get("DtcsPolarity").unwrap().chars().nth(1).unwrap() {
                'N' => "N",
                'R' => "I",
                _ => "N",
            };
            record.get("RxDtcsCode").unwrap()
        };
        let dcs_str = format!("D{}{}",
            dstr.to_string(),
            pstr.to_string(),
        );
        let tone = Tone {
            mode: ToneMode::DCS,
            ctcss: None,
            dcs: Some(dcs_str),
        };
        Ok((Some(tone.clone()), Some(tone)))
    } else if tone_mode == "Cross" { // look at CrossMode
        if cross_mode == "Tone->Tone" { // rx is CTCSS (cTone), tx is CTCSS (rTone)
            let tone_rx = Tone {
                mode: ToneMode::CTCSS,
                ctcss: Some(Decimal::from_str(record.get("cToneFreq").unwrap()).unwrap()),
                dcs: None,
            };
            let tone_tx = Tone {
                mode: ToneMode::CTCSS,
                ctcss: Some(Decimal::from_str(record.get("rToneFreq").unwrap()).unwrap()),
                dcs: None,
            };
            Ok((Some(tone_rx), Some(tone_tx)))
        } else if cross_mode == "Tone->DTCS" { // rx is DCS (RxDtcs or Dtcs), tx is CTCSS (rTone)
            let dstr = if record.get("RxDtcsCode").unwrap() == "" {
                record.get("DtcsCode").unwrap()
            } else {
                record.get("RxDtcsCode").unwrap()
            };
            let dcs_str = format!("D{}{}",
                dstr.to_string(),
                record.get("DtcsPolarity").unwrap().chars().next().unwrap() // @TODO FIXME
            );
            let tone_rx = Tone {
                mode: ToneMode::DCS,
                ctcss: None,
                dcs: Some(dcs_str),
            };
            let tone_tx = Tone {
                mode: ToneMode::CTCSS,
                ctcss: Some(Decimal::from_str(record.get("rToneFreq").unwrap()).unwrap()),
                dcs: None,
            };
            Ok((Some(tone_rx), Some(tone_tx)))
        } else if cross_mode == "DTCS->Tone" { // rx is CTCSS (rTone or cTone), tx is DCS (Dtcs)
            let tstr = if record.get("rToneFreq").unwrap() == "" {
                record.get("cToneFreq").unwrap()
            } else {
                record.get("rToneFreq").unwrap()
            };
            let tone_rx = Tone {
                mode: ToneMode::CTCSS,
                ctcss: Some(Decimal::from_str(tstr).unwrap()),
                dcs: None,
            };
            let dstr = record.get("DtcsCode").unwrap();
            // second character of DtcsPolarity is DCS polarity, but needs to be mapped from NR to NI
            let pstr = match record.get("DtcsPolarity").unwrap().chars().nth(1).unwrap() {
                'N' => "N",
                'R' => "I",
                _ => "N",
            };
            let dcs_str = format!("D{}{}",
                dstr.to_string(),
                pstr.to_string(),
            );
            let tone_tx = Tone {
                mode: ToneMode::DCS,
                ctcss: None,
                dcs: Some(dcs_str),
            };
            Ok((Some(tone_rx), Some(tone_tx)))
        } else if cross_mode == "->Tone" { // rx is CTCSS (rTone or cTone), tx is carrier squelch
            let tstr = if record.get("rToneFreq").unwrap() == "" {
                record.get("cToneFreq").unwrap()
            } else {
                record.get("rToneFreq").unwrap()
            };
            let tone_rx = Tone {
                mode: ToneMode::CTCSS,
                ctcss: Some(Decimal::from_str(tstr).unwrap()),
                dcs: None,
            };
            Ok((Some(tone_rx), None))
        } else if cross_mode == "->DTCS" { // rx is DCS (Dtcs or RxDtcs), tx is carrier squelch
            let pstr;
            let dstr = if record.get("RxDtcsCode").unwrap() == "" {
                // first character of DtcsPolarity is DCS polarity, but needs to be mapped from NR to NI
                pstr = match record.get("DtcsPolarity").unwrap().chars().nth(0).unwrap() {
                    'N' => "N",
                    'R' => "I",
                    _ => "N",
                };
                record.get("DtcsCode").unwrap()
            } else {
                // second character of DtcsPolarity is DCS polarity, but needs to be mapped from NR to NI
                pstr = match record.get("DtcsPolarity").unwrap().chars().nth(1).unwrap() {
                    'N' => "N",
                    'R' => "I",
                    _ => "N",
                };
                record.get("RxDtcsCode").unwrap()
            };
            let dcs_str = format!("D{}{}",
                dstr.to_string(),
                pstr.to_string(),
            );
            let tone_rx = Tone {
                mode: ToneMode::DCS,
                ctcss: None,
                dcs: Some(dcs_str),
            };
            Ok((Some(tone_rx), None))
        } else if cross_mode == "Tone->" { // rx is carrier squelch, tx is CTCSS (rTone or cTone)
            let tstr = if record.get("rToneFreq").unwrap() == "" {
                record.get("cToneFreq").unwrap()
            } else {
                record.get("rToneFreq").unwrap()
            };
            let tone_tx = Tone {
                mode: ToneMode::CTCSS,
                ctcss: Some(Decimal::from_str(tstr).unwrap()),
                dcs: None,
            };
            Ok((None, Some(tone_tx)))
        } else if cross_mode == "DTCS->" { // rx is carrier squelch, tx is DCS (Dtcs)
            let dstr = record.get("DtcsCode").unwrap();
            let pstr = match record.get("DtcsPolarity").unwrap().chars().nth(0).unwrap() {
                'N' => "N",
                'R' => "I",
                _ => "N",
            };
            let dcs_str = format!("D{}{}",
                dstr.to_string(),
                pstr.to_string(),
            );
            let tone_tx = Tone {
                mode: ToneMode::DCS,
                ctcss: None,
                dcs: Some(dcs_str),
            };
            Ok((None, Some(tone_tx)))
        } else if cross_mode == "DTCS->DTCS" { // rx is DCS (RxDtcs), tx is DCS (Dtcs)
            let tx_dstr = record.get("DtcsCode").unwrap();
            let tx_pstr = match record.get("DtcsPolarity").unwrap().chars().nth(0).unwrap() {
                'N' => "N",
                'R' => "I",
                _ => "N",
            };
            let tx_dcs_str = format!("D{}{}",
                tx_dstr.to_string(),
                tx_pstr.to_string(),
            );
            let tone_tx = Tone {
                mode: ToneMode::DCS,
                ctcss: None,
                dcs: Some(tx_dcs_str),
            };
            let rx_dstr = record.get("RxDtcsCode").unwrap();
            let rx_pstr = match record.get("DtcsPolarity").unwrap().chars().nth(1).unwrap() {
                'N' => "N",
                'R' => "I",
                _ => "N",
            };
            let rx_dcs_str = format!("D{}{}",
                rx_dstr.to_string(),
                rx_pstr.to_string(),
            );
            let tone_rx = Tone {
                mode: ToneMode::DCS,
                ctcss: None,
                dcs: Some(rx_dcs_str),
            };
            Ok((Some(tone_rx), Some(tone_tx)))
        } else {
            Err(format!("Unsupported tone mode: {} cross mode: {}", tone_mode, cross_mode).into())
        }
    } else {
        Err(format!("Unsupported tone mode: {}", tone_mode).into())
    }
}

pub fn parse_channel_record(record: &CsvRecord, opt: &Opt) -> Result<Channel, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);

    let mut channel = Channel::default();

    // chirp uses zero-index, +1 to match other CPS
    if record.get("Mode").unwrap().as_str() == "NFM" || record.get("Mode").unwrap().as_str() == "FM" {
        channel.index = record.get("Location").unwrap().parse::<u32>()? + 1;
        channel.name = record.get("Name").unwrap().to_string();
        channel.mode = ChannelMode::FM;
        channel.frequency_rx = Decimal::from_str(record.get("Frequency").unwrap())? * Decimal::new(1_000_000, 0);
        let offset = match record.get("Offset").unwrap().as_str() {
            "" => Decimal::new(0, 0),
            s => Decimal::from_str(s)? * Decimal::new(1_000_000, 0),
        };
        channel.frequency_tx = match record.get("Duplex").unwrap().as_str() {
            "+" => channel.frequency_rx + offset,
            "-" => channel.frequency_rx - offset,
            "split" => offset,
            "off" => channel.frequency_rx,
            _ => channel.frequency_rx,
        };
        channel.rx_only = match record.get("Duplex").unwrap().as_str() {
            "off" => true,
            _ => false,
        };
        channel.power = Power::Watts(record.get("Power").unwrap().parse::<f64>()?);
        channel.scan = Some(Scan {
            zone_skip: false, // Chirp doesn't support zones
            all_skip: match record.get("Skip").unwrap().as_str() {
                "S" => true,
                _ => false,
            }
        });
        // FM specific properties
        let (tone_rx, tone_tx) = match parse_tones(record) {
            Ok((rx, tx)) => (rx, tx),
            Err(e) => return Err(e),
        };
        channel.fm = Some(FmChannel {
            bandwidth: match record.get("Mode").unwrap().as_str() {
                "FM" => Decimal::new(25_000, 0),
                "NFM" => Decimal::new(12_500, 0),
                _ => return Err(format!("Unsupported mode: {}", record.get("mode").unwrap()).into()),
            },
            squelch: Squelch {
                default: true, // Chirp doesn't support configurable squelch
                percent: None,
            },
            tone_rx: tone_rx,
            tone_tx: tone_tx,
        });
    } else {
        uprintln!(opt, Stderr, Color::Red, None, "Unsupported mode: {}", record.get("Mode").unwrap());
    }
    Ok(channel)
}

pub fn read(input_path: &PathBuf, opt: &Opt) -> Result<Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    let mut codeplug = Codeplug {
        channels: Vec::new(),
        zones: Vec::new(),
        talkgroups: Vec::new(),
        talkgroup_lists: Vec::new(),
        config: None,
        source: format!("{}", Path::new(file!()).file_stem().unwrap().to_str().unwrap()),
    };

    // check that the input path is a file
    if !input_path.is_file() {
        uprintln!(opt, Stderr, Color::Red, None, "You lied to me when you told me this was a file: {}", input_path.display());
        return Err("Bad input path".into());
    }

    uprintln!(opt, Stderr, None, 3, "Reading {}", input_path.display());
    let mut reader = csv::Reader::from_path(input_path)?;
    for result in reader.deserialize() {
        let record: CsvRecord = result?;
        // convert from CSV record to Channel
        let channel = parse_channel_record(&record, &opt)?;
        if channel.index > 0 {
            // append to codeplug.channels
            codeplug.channels.push(channel);
        }
    }

    Ok(codeplug)
}

// WRITE //////////////////////////////////////////////////////////////////////

// returns a tuple with frequency, duplex, and offset
fn write_frequencies(channel: &Channel) -> (String, String, String) {
    if channel.rx_only {
        (
            format!("{:0.6}", (channel.frequency_rx / Decimal::new(1_000_000, 0)).to_f64().unwrap()),
            "off".to_string(),
            "".to_string(),
        )
    } else {
        // doing this properly is incredibly annoying. For now, just make everything an offset
        // @TODO add band lookup here to determine if crossband and set duplex to "split"
        let offset = channel.frequency_tx - channel.frequency_rx;
        let plus = offset > Decimal::new(0, 0);
        (
            format!("{:0.6}", (channel.frequency_rx / Decimal::new(1_000_000, 0)).to_f64().unwrap()),
            if plus { "+".to_string() } else { "-".to_string() },
            format!("{:0.6}", (offset / Decimal::new(1_000_000, 0)).to_f64().unwrap()),
        )
    }
}

// ~ here be dragons. I hate Chirp so much ~
// returns a tuple (Tone, rToneFreq, cToneFreq, DtcsCode, DtcsPolarity, RxDtcsCode, CrossMode)
fn write_tones(channel: &Channel) -> (String, String, String, String, String, String, String) {
    if channel.fm.is_none() {
        // we should never get here
        (
            "".to_string(), // Tone
            "88.5".to_string(), // rToneFreq
            "88.5".to_string(), // cToneFreq
            "023".to_string(), // DtcsCode
            "NN".to_string(), // DtcsPolarity
            "023".to_string(), // RxDtcsCode
            "Tone->Tone".to_string(), // CrossMode
        )
    } else {
        let fm = channel.fm.as_ref().unwrap();
        if fm.tone_rx.is_some() && fm.tone_tx.is_some() {
            let tone_rx = fm.tone_rx.as_ref().unwrap();
            let tone_tx = fm.tone_tx.as_ref().unwrap();
            if tone_rx == tone_tx {
                match tone_rx.mode {
                    // identical CTCSS tones
                    ToneMode::CTCSS => {
                        (
                            "Tone".to_string(), // Tone
                            freq2str(&tone_rx.ctcss.as_ref().unwrap()), // rToneFreq
                            freq2str(&tone_tx.ctcss.as_ref().unwrap()), // cToneFreq
                            "023".to_string(), // DtcsCode
                            "NN".to_string(), // DtcsPolarity
                            "023".to_string(), // RxDtcsCode
                            "Tone->Tone".to_string(), // CrossMode
                        )
                    }
                    ToneMode::DCS => {
                        // identical DCS tones
                        (
                            "DTCS".to_string(), // Tone
                            "88.5".to_string(), // rToneFreq
                            "88.5".to_string(), // cToneFreq
                            tone_rx.dcs.as_ref().unwrap().to_string(), // DtcsCode @TODO
                            "".to_string(), // DtcsPolarity @TODO
                            tone_rx.dcs.as_ref().unwrap().to_string(), // RxDtcsCode @TODO
                            "DTCS".to_string(), // CrossMode
                        )
                    }
                }
            } else {
                // @TODO FIXME
                ( // Chirp fills out these fields with defaults
                    "".to_string(), // Tone
                    "88.5".to_string(), // rToneFreq
                    "88.5".to_string(), // cToneFreq
                    "023".to_string(), // DtcsCode
                    "NN".to_string(), // DtcsPolarity
                    "023".to_string(), // RxDtcsCode
                    "Tone->Tone".to_string(), // CrossMode
                )
            }
        } else if fm.tone_rx.is_some() && fm.tone_tx.is_none() {
            // ->Tone
            let tone_rx = fm.tone_rx.as_ref().unwrap();
            return (
                "Tone".to_string(),
                freq2str(&tone_rx.ctcss.as_ref().unwrap()),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "->Tone".to_string(),
            );
        } else if fm.tone_rx.is_none() && fm.tone_tx.is_some() {
            // Tone->
            let tone_tx = fm.tone_tx.as_ref().unwrap();
            return (
                "Tone".to_string(),
                "".to_string(),
                freq2str(&tone_tx.ctcss.as_ref().unwrap()),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "Tone->".to_string(),
            );
        } else { // no tones
            ( // Chirp fills out these fields with defaults
                "".to_string(), // Tone
                "88.5".to_string(), // rToneFreq
                "88.5".to_string(), // cToneFreq
                "023".to_string(), // DtcsCode
                "NN".to_string(), // DtcsPolarity
                "023".to_string(), // RxDtcsCode
                "Tone->Tone".to_string(), // CrossMode
            )
        }
    }
}

fn write_mode(channel: &Channel) -> Result<String, Box<dyn Error>> {
    let bandwidth = channel.fm.as_ref().unwrap().bandwidth;
    match bandwidth.to_u32().unwrap() {
        25_000 => Ok("FM".to_string()),
        12_500 => Ok("NFM".to_string()),
        _ => Err("Unsupported bandwidth".into()),
    }
}

fn write_channels(codeplug: &Codeplug, path: &Path, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 1, "Writing {}", path.display());

    // open the output file
    let mut writer = csv::WriterBuilder::new()
        .from_path(path)?;

    // write the header
    writer.write_record(&[
        "Location",
        "Name",
        "Frequency",
        "Duplex",
        "Offset",
        "Tone",
        "rToneFreq",
        "cToneFreq",
        "DtcsCode",
        "DtcsPolarity",
        "RxDtcsCode",
        "CrossMode",
        "Mode",
        "TStep",
        "Skip",
        "Power",
        "Comment",
        "URCALL",
        "RPT1CALL",
        "RPT2CALL",
        "DVCODE",
    ])?;

    for channel in &codeplug.channels {
        uprintln!(opt, Stderr, None, 4, "Writing channel {:width$}: {}", channel.index, channel.name, width=get_props().channel_index_width);
        if channel.mode == ChannelMode::FM {
            let (frequency, duplex, offset) = write_frequencies(channel);
            let (tone, r_tone_freq, c_tone_freq, dtcs_code, dtcs_polarity, rx_dtcs_code, cross_mode) = write_tones(channel);
            writer.write_record(&[
                (channel.index - 1).to_string(), // Location (zero-indexed)
                channel.name.clone(), // Name
                frequency, // Frequency
                duplex, // Duplex
                offset, // Offset
                tone, // Tone
                r_tone_freq, // rToneFreq
                c_tone_freq, // cToneFreq
                dtcs_code, // DtcsCode
                dtcs_polarity, // DtcsPolarity
                rx_dtcs_code, // RxDtcsCode
                cross_mode, // CrossMode
                write_mode(channel)?, // Mode
                "5".to_string(), // TStep
                match channel.scan.as_ref().unwrap().all_skip {
                    true => "S",
                    false => "",
                }.to_string(), // Skip
                match channel.power {
                    Power::Default => format!("{:0.1}W", 5.0), // default to 5W
                    Power::Watts(w) => format!("{:0.1}W", w),
                }, // Power
                "".to_string(), // Comment
                "".to_string(), // URCALL
                "".to_string(), // RPT1CALL
                "".to_string(), // RPT2CALL
                "".to_string(), // DVCODE
            ])?;
        } else {
            uprintln!(opt, Stderr, Color::Red, None, "Unsupported mode: index = {}, mode = {:?}", channel.index, channel.mode);
        }
    }

    writer.flush()?;
    Ok(())
}


pub fn write(codeplug: &Codeplug, output_path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    // if the output path exists, complain
    if output_path.exists() {
        uprintln!(opt, Stderr, Color::Red, None, "Output path already exists: {}", output_path.display());
        return Err("Output path already exists".into());
    }

    // write channels
    let channels_path: PathBuf = output_path.clone();
    write_channels(&codeplug, &channels_path, &opt)?;

    Ok(())
}
