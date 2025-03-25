// src/radios/opengd77_rt3s.rs
// reference https://burntsushi.net/csv/ for CSV parsing technique

use std::error::Error;
// use std::fs;
use std::path::PathBuf;
use std::path::Path;
use std::collections::HashMap;
use rust_decimal::prelude::*;
use std::sync::OnceLock;
use std::cmp::{max, min};

use crate::*;
use crate::structures::*;

static PROPS: OnceLock<structures::RadioProperties> = OnceLock::new();
pub fn get_props() -> &'static structures::RadioProperties {
    PROPS.get_or_init(|| {
        let mut props = structures::RadioProperties::default();
        props.modes = vec![structures::ChannelMode::AM, structures::ChannelMode::FM];
        props.channels_max = 1000;
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
//       (blank): simplex, TX/RX are the same
//       +,-: TX frequency is offset from RX frequency by Offset
//       split: Offset is the TX frequency
//       off: RX only
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
// return a tuple with tx and rx tones, in that order
// Tone Mode
// - (none): no tone or code is transmitted, receive squelch is open or carrier-triggered
// - Tone: CTCSS tone is transmitted, receive squelch is open or carrier-triggered. The tone used is set in the Tone column
// - TSQL: CTCSS tone is transmitted, receive squelch is tone-coded to the same tone. The tone used is set in the ToneSql column
// - DTCS: DCS code is transmitted, receive squelch is tone-coded to the same code. The code used is set in the DTCS Code column
// - Cross: something else. See Cross Mode
// Cross Mode
// - Tone->Tone: Use Tone (rToneFreq) value for transmit, and ToneSql (cToneFreq) value for receive
// - Tone->DTCS: Use Tone (rToneFreq) value for transmit, and DTCS Rx Code value for receive
// - DTCS->Tone: Use DTCS Code value for transmit, and ToneSql (cToneFreq) value for receive
// - ->Tone: No transmit tone, and ToneSql (cToneFreq) for receive
// - ->DTCS: No transmit tone, and DTCS Rx Code for receive
// - DTCS->: Use DTCS Code value for transmit, and no receive tone
// - DTCS->DTCS: Use DTCS Code value for transmit, and DTCS Rx Code value for receive
fn parse_tones(record: &CsvRecord) -> Result<(Option<Tone>, Option<Tone>), Box<dyn Error>> {
    let tone_mode = record.get("Tone").unwrap();
    if tone_mode == "" { // rx is carrier squelch, tx is carrier squelch
        Ok((None, None))
    } else if tone_mode == "Tone" { // rx is carrier squelch, tx is CTCSS (rTone)
        let tone_tx = Tone::Ctcss(record.get("rToneFreq").unwrap().parse::<f64>()?);
        Ok((Some(tone_tx), None))
    } else if tone_mode == "TSQL" { // rx is CTCSS (cTone or rTone), tx is same CTCSS
        let tone_rx = Tone::Ctcss(record.get("cToneFreq").unwrap().parse::<f64>()?);
        Ok((Some(tone_rx.clone()), Some(tone_rx)))
    } else if tone_mode == "DTCS" { // rx is DCS, tx is same DCS
        let dstr = record.get("DtcsCode").unwrap();
        let pstr = record.get("DtcsPolarity").unwrap().replace("R", "I");
        let tone_tx = Tone::Dcs(format!("D{}{}", dstr.to_string(), pstr.chars().nth(0).unwrap()));
        let tone_rx = Tone::Dcs(format!("D{}{}", dstr.to_string(), pstr.chars().nth(1).unwrap()));
        Ok((Some(tone_tx), Some(tone_rx)))
    } else if tone_mode == "Cross" { // look at CrossMode
        let cross_mode = record.get("CrossMode").unwrap();
        if cross_mode == "Tone->Tone" { // tx is CTCSS (rTone), rx is CTCSS (cTone)
            let tone_tx = Tone::Ctcss(record.get("rToneFreq").unwrap().parse::<f64>()?);
            let tone_rx = Tone::Ctcss(record.get("cToneFreq").unwrap().parse::<f64>()?);
            return Ok((Some(tone_tx), Some(tone_rx)));
        } else if cross_mode == "DTCS->" { // tx is DCS (DtcsCode), rx is carrier squelch
            let dstr = record.get("DtcsCode").unwrap();
            let pstr = record.get("DtcsPolarity").unwrap().replace("R", "I");
            let tone_tx = Tone::Dcs(format!("D{}{}", dstr.to_string(), pstr.chars().nth(0).unwrap()));
            return Ok((Some(tone_tx), None));
        } else if cross_mode == "->DTCS" { // tx is carrier squelch, rx is DCS (RxDtcsCode)
            let dstr = record.get("RxDtcsCode").unwrap();
            let pstr = record.get("DtcsPolarity").unwrap().replace("R", "I");
            let tone_rx = Tone::Dcs(format!("D{}{}", dstr.to_string(), pstr.chars().nth(1).unwrap()));
            return Ok((None, Some(tone_rx)));
        } else if cross_mode == "Tone->DTCS" { // tx is CTCSS (rTone), rx is DCS (RxDtcsCode)
            let tone_tx = Tone::Ctcss(record.get("rToneFreq").unwrap().parse::<f64>()?);
            let dstr = record.get("RxDtcsCode").unwrap();
            let pstr = record.get("DtcsPolarity").unwrap().replace("R", "I");
            let tone_rx = Tone::Dcs(format!("D{}{}", dstr.to_string(), pstr.chars().nth(1).unwrap()));
            return Ok((Some(tone_tx), Some(tone_rx)));
        } else if cross_mode == "DTCS->Tone" { // tx is DCS (DtcsCode), rx is CTCSS (cTone)
            let dstr = record.get("DtcsCode").unwrap();
            let pstr = record.get("DtcsPolarity").unwrap().replace("R", "I");
            let tone_tx = Tone::Dcs(format!("D{}{}", dstr.to_string(), pstr.chars().nth(0).unwrap()));
            let tone_rx = Tone::Ctcss(record.get("cToneFreq").unwrap().parse::<f64>()?);
            return Ok((Some(tone_tx), Some(tone_rx)));
        } else if cross_mode == "->Tone" { // tx is carrier squelch, rx is CTCSS (cTone)
            let tone_rx = Tone::Ctcss(record.get("cToneFreq").unwrap().parse::<f64>()?);
            return Ok((None, Some(tone_rx)));
        } else if cross_mode == "DTCS->DTCS" { // tx is DCS (DtcsCode), rx is DCS (RxDtcsCode)
            let tx_dstr = record.get("DtcsCode").unwrap();
            let rx_dstr = record.get("RxDtcsCode").unwrap();
            let pstr = record.get("DtcsPolarity").unwrap().replace("R", "I");
            let tone_tx = Tone::Dcs(format!("D{}{}", tx_dstr.to_string(), pstr.chars().nth(0).unwrap()));
            let tone_rx = Tone::Dcs(format!("D{}{}", rx_dstr.to_string(), pstr.chars().nth(1).unwrap()));
            return Ok((Some(tone_tx), Some(tone_rx)));
        } else if cross_mode == "Tone->" { // tx is CTCSS (rTone), rx is carrier squelch
            let tone_tx = Tone::Ctcss(record.get("rToneFreq").unwrap().parse::<f64>()?);
            return Ok((Some(tone_tx), None));
        }
        Err(format!("Unsupported cross mode: {}", cross_mode).into())
    } else {
        Err(format!("Unsupported tone mode: {}", tone_mode).into())
    }
}

pub fn parse_channel_record(record: &CsvRecord, opt: &Opt) -> Result<Channel, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);

    let mut channel = Channel::default();

    // chirp uses zero-index, +1 to match other CPS
    if record.get("Mode").unwrap().as_str() == "NFM" || record.get("Mode").unwrap().as_str() == "FM" {
        channel.index = record.get("Location").unwrap().parse::<usize>()? + 1;
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
        channel.power = Power::Watts(record.get("Power").unwrap().strip_suffix("W").unwrap().parse::<f64>()?);
        channel.scan = Some(Scan::Skip(ScanSkip {
            zone: false, // Chirp doesn't support zones
            all: match record.get("Skip").unwrap().as_str() {
            "S" => true,
            _ => false,
            }
        }));
        // FM specific properties
        let (tone_tx, tone_rx) = match parse_tones(record) {
            Ok((rx, tx)) => (rx, tx),
            Err(e) => return Err(e),
        };
        channel.fm = Some(FmChannel {
            bandwidth: match record.get("Mode").unwrap().as_str() {
                "FM" => Decimal::new(25_000, 0),
                "NFM" => Decimal::new(12_500, 0),
                _ => return Err(format!("Unsupported mode: {}", record.get("mode").unwrap()).into()),
            },
            squelch: Squelch::Default, // chirp doesn't support squelch
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

    let mut codeplug = Codeplug::default();
    codeplug.source = format!("{}", Path::new(file!()).file_stem().unwrap().to_str().unwrap());

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
            "0.600000".to_string(), // default (sometimes this is 0.500000, no idea why)
        )
    } else if channel.frequency_rx == channel.frequency_tx {
        (
            format!("{:0.6}", (channel.frequency_rx / Decimal::new(1_000_000, 0)).to_f64().unwrap()),
            "".to_string(),
            "0.600000".to_string(), // default
        )
    } else {
        // @TODO the right thing to do here is to reference a bandplan to determine crossband
        // the lazy way to handle this: check if TX/RX are within 15% of each other
        let high = max(channel.frequency_rx, channel.frequency_tx);
        let low = min(channel.frequency_rx, channel.frequency_tx);
        let diff = high - low;
        if (diff / high) > (Decimal::new(15, 0) / Decimal::new(100, 0)) {
            // crossband
            return (
                format!("{:0.6}", (channel.frequency_rx / Decimal::new(1_000_000, 0)).to_f64().unwrap()),
                "split".to_string(),
                format!("{:0.6}", (channel.frequency_tx / Decimal::new(1_000_000, 0)).to_f64().unwrap()),
            )
        } else {
            // same band
            let offset = channel.frequency_tx - channel.frequency_rx;
            let plus = offset > Decimal::new(0, 0);
            return (
                format!("{:0.6}", (channel.frequency_rx / Decimal::new(1_000_000, 0)).to_f64().unwrap()),
                if plus { "+".to_string() } else { "-".to_string() },
                format!("{:0.6}", ((if plus { offset } else { -offset }) / Decimal::new(1_000_000, 0)).to_f64().unwrap()),
            )
        }
    }
}

// ~ here be dragons. I hate Chirp so much ~
// returns a tuple (Tone, rToneFreq, cToneFreq, DtcsCode, DtcsPolarity, RxDtcsCode, CrossMode)
// https://chirpmyradio.com/projects/chirp/wiki/MemoryEditorColumns
// Tuple items are commented with the CSV column name and the Chirp column name (in parentheses)
// Tone (Tone Mode)
// rToneFreq (Tone)
// cToneFreq (Tone Squelch)
// DtcsCode (DTCS Code)
// DtcsPolarity (DTCS Pol)
// RxDtcsCode (DTCS Rx Code)
// CrossMode (Cross Mode)
fn write_tones(channel: &Channel) -> (String, String, String, String, String, String, String) {
    if channel.fm.is_some() {
        let fm = channel.fm.as_ref().unwrap();
        // if there is only a TX tone, and it is CTCSS: Tone Mode = Tone
        if fm.tone_rx.is_none() && fm.tone_tx.is_some() &&
           matches!(fm.tone_tx.as_ref().unwrap(), Tone::Ctcss(_)) {
            let tone_tx = fm.tone_tx.as_ref().unwrap();
            return (
                "Tone".to_string(), // Tone (Tone Mode)
                match tone_tx {
                    Tone::Ctcss(freq) => format!("{:.1}", freq),
                    _ => "88.5".to_string(),
                }, // rToneFreq (Tone)
                "88.5".to_string(), // cToneFreq (Tone Squelch)
                "023".to_string(), // DtcsCode (DTCS Code)
                "NN".to_string(), // DtcsPolarity (DTCS Pol)
                "023".to_string(), // RxDtcsCode (DTCS Rx Code)
                "Tone->Tone".to_string(), // CrossMode (Cross Mode)
            );
        }
        // if RX and TX tones are the same, and are CTCSS: Tone Mode = TSQL
        if fm.tone_rx.is_some() && fm.tone_tx.is_some() &&
           fm.tone_rx == fm.tone_tx &&
           matches!(fm.tone_rx.as_ref().unwrap(), Tone::Ctcss(_)) {
            let tone = fm.tone_rx.as_ref().unwrap();
            return (
                "TSQL".to_string(), // Tone (Tone Mode)
                "88.5".to_string(), // rToneFreq (Tone)
                match tone {
                    Tone::Ctcss(freq) => format!("{:.1}", freq),
                    _ => "88.5".to_string(),
                }, // cToneFreq (Tone Squelch)
                "023".to_string(), // DtcsCode (DTCS Code)
                "NN".to_string(), // DtcsPolarity (DTCS Pol)
                "023".to_string(), // RxDtcsCode (DTCS Rx Code)
                "Tone->Tone".to_string(), // CrossMode (Cross Mode)
            );
        }
        // if RX and TX tones are same (but may have different polarity), and are DCS: Tone Mode = DTCS
        if fm.tone_rx.is_some() && fm.tone_tx.is_some() &&
           matches!(
            (fm.tone_rx.as_ref().unwrap(), fm.tone_tx.as_ref().unwrap()),
            (Tone::Dcs(rx_dcs), Tone::Dcs(tx_dcs)) if rx_dcs[0..3] == tx_dcs[0..3]
           ) {
            let tone_rx = fm.tone_rx.as_ref().unwrap();
            let tone_tx = fm.tone_tx.as_ref().unwrap();
            let mut rx_dcs_str = "D000N".to_string();
            let mut tx_dcs_str = "D000N".to_string();
            match (tone_rx, tone_tx) {
                (Tone::Dcs(rx_dcs), Tone::Dcs(tx_dcs)) => {
                    rx_dcs_str = rx_dcs.to_string();
                    tx_dcs_str = tx_dcs.to_string();
                },
                _ => {}
            }
            return (
                "DTCS".to_string(), // Tone (Tone Mode)
                "88.5".to_string(), // rToneFreq (Tone)
                "88.5".to_string(), // cToneFreq (Tone Squelch)
                tx_dcs_str[1..4].to_string(), // DtcsCode (DTCS Code)
                format!("{}{}", tx_dcs_str.chars().nth(4).unwrap(), rx_dcs_str.chars().nth(4).unwrap()).replace("I", "R"), // DtcsPolarity (DTCS Pol)
                "023".to_string(), // RxDtcsCode (DTCS Rx Code)
                "Tone->Tone".to_string(), // CrossMode (Cross Mode)
            );
        }
        // if RX and TX tones are different, but are both CTCSS: Tone Mode = Cross, Cross Mode = Tone->Tone
        if fm.tone_rx.is_some() && fm.tone_tx.is_some() &&
           matches!(
            (fm.tone_rx.as_ref().unwrap(), fm.tone_tx.as_ref().unwrap()),
            (Tone::Ctcss(_), Tone::Ctcss(_))
           ) {
            let tone_rx = fm.tone_rx.as_ref().unwrap();
            let tone_tx = fm.tone_tx.as_ref().unwrap();
            return (
                "Cross".to_string(), // Tone (Tone Mode)
                match tone_tx {
                    Tone::Ctcss(freq) => format!("{:.1}", freq),
                    _ => "88.5".to_string(),
                }, // rToneFreq (Tone)
                match tone_rx {
                    Tone::Ctcss(freq) => format!("{:.1}", freq),
                    _ => "88.5".to_string(),
                }, // cToneFreq (Tone Squelch)
                "023".to_string(), // DtcsCode (DTCS Code)
                "NN".to_string(), // DtcsPolarity (DTCS Pol)
                "023".to_string(), // RxDtcsCode (DTCS Rx Code)
                "Tone->Tone".to_string(), // CrossMode (Cross Mode)
            );
        }
        // if there is no RX tone, and TX tone is DCS: Tone Mode = Cross, Cross Mode = DTCS->
        if fm.tone_rx.is_none() && fm.tone_tx.is_some() &&
           matches!(fm.tone_tx.as_ref().unwrap(), Tone::Dcs(_)) {
            let tone_tx = fm.tone_tx.as_ref().unwrap();
            let mut tx_dcs_str = "D000N".to_string();
            match tone_tx {
                Tone::Dcs(tx_dcs) => {
                    tx_dcs_str = tx_dcs.to_string();
                },
                _ => {}
            }
            return (
                "Cross".to_string(), // Tone (Tone Mode)
                "88.5".to_string(), // rToneFreq (Tone)
                "88.5".to_string(), // cToneFreq (Tone Squelch)
                tx_dcs_str[1..4].to_string(), // DtcsCode (DTCS Code)
                format!("{}{}", tx_dcs_str.chars().nth(4).unwrap(), "N").replace("I", "R"), // DtcsPolarity (DTCS Pol)
                "023".to_string(), // RxDtcsCode (DTCS Rx Code)
                "DTCS->".to_string(), // CrossMode (Cross Mode)
            );
        }
        // if RX tone is DCS, and there is no TX tone: Tone Mode = Cross Mode, Cross Mode = ->DTCS
        if fm.tone_rx.is_some() && fm.tone_tx.is_none() &&
           matches!(fm.tone_rx.as_ref().unwrap(), Tone::Dcs(_)) {
            let tone_rx = fm.tone_rx.as_ref().unwrap();
            let mut rx_dcs_str = "D000N".to_string();
            match tone_rx {
                Tone::Dcs(rx_dcs) => {
                    rx_dcs_str = rx_dcs.to_string();
                },
                _ => {}
            }
            return (
                "Cross".to_string(), // Tone (Tone Mode)
                "88.5".to_string(), // rToneFreq (Tone)
                "88.5".to_string(), // cToneFreq (Tone Squelch)
                "023".to_string(), // DtcsCode (DTCS Code)
                format!("{}{}", "N", rx_dcs_str.chars().nth(4).unwrap()).replace("I", "R"), // DtcsPolarity (DTCS Pol)
                rx_dcs_str[1..4].to_string(), // RxDtcsCode (DTCS Rx Code)
                "->DTCS".to_string(), // CrossMode (Cross Mode)
            );
        }
        // if TX tone is CTCSS, and RX tone is DCS: Tone Mode = Cross, Cross Mode = Tone->DTCS
        if fm.tone_tx.is_some() && fm.tone_rx.is_some() &&
           matches!(
            (fm.tone_tx.as_ref().unwrap(), fm.tone_rx.as_ref().unwrap()),
            (Tone::Ctcss(_), Tone::Dcs(_))
           ) {
            let tone_tx = fm.tone_tx.as_ref().unwrap();
            let tone_rx = fm.tone_rx.as_ref().unwrap();
            let mut rx_dcs_str = "D000N".to_string();
            match tone_rx {
                Tone::Dcs(rx_dcs) => {
                    rx_dcs_str = rx_dcs.to_string();
                },
                _ => {}
            }
            return (
                "Cross".to_string(), // Tone (Tone Mode)
                match tone_tx {
                    Tone::Ctcss(freq) => format!("{:.1}", freq),
                    _ => "88.5".to_string(),
                }, // rToneFreq (Tone)
                "88.5".to_string(), // cToneFreq (Tone Squelch)
                "023".to_string(), // DtcsCode (DTCS Code)
                format!("{}{}", "N", rx_dcs_str.chars().nth(4).unwrap()).replace("I", "R"), // DtcsPolarity (DTCS Pol)
                rx_dcs_str[1..4].to_string(), // RxDtcsCode (DTCS Rx Code)
                "Tone->DTCS".to_string(), // CrossMode (Cross Mode)
            );
        }
        // if TX tone is DCS, and RX tone is CTCSS: Tone Mode = Cross, Cross Mode = DTCS->Tone
        if fm.tone_tx.is_some() && fm.tone_rx.is_some() &&
           matches!(
                (fm.tone_tx.as_ref().unwrap(), fm.tone_rx.as_ref().unwrap()),
                (Tone::Dcs(_), Tone::Ctcss(_))
            ) {
            let tone_tx = fm.tone_tx.as_ref().unwrap();
            let tone_rx = fm.tone_rx.as_ref().unwrap();
            let mut tx_dcs_str = "D000N".to_string();
            match tone_tx {
                Tone::Dcs(tx_dcs) => {
                    tx_dcs_str = tx_dcs.to_string();
                },
                _ => {}
            }
            return (
                "Cross".to_string(), // Tone (Tone Mode)
                "88.5".to_string(), // rToneFreq (Tone)
                match tone_rx {
                    Tone::Ctcss(freq) => format!("{:.1}", freq),
                    _ => "88.5".to_string(),
                }, // cToneFreq (Tone Squelch)
                tx_dcs_str[1..4].to_string(), // DtcsCode (DTCS Code)
                format!("{}{}", tx_dcs_str.chars().nth(4).unwrap(), "N").replace("I", "R"), // DtcsPolarity (DTCS Pol)
                "023".to_string(), // RxDtcsCode (DTCS Rx Code)
                "DTCS->Tone".to_string(), // CrossMode (Cross Mode)
            );
        }
        // if there is no TX tone, and RX tone is CTCSS: Tone Mode = Cross, Cross Mode = ->Tone
        if fm.tone_tx.is_none() && fm.tone_rx.is_some() &&
           matches!(fm.tone_rx.as_ref().unwrap(), Tone::Ctcss(_)) {
            let tone_rx = fm.tone_rx.as_ref().unwrap();
            return (
                "Cross".to_string(), // Tone (Tone Mode)
                "88.5".to_string(), // rToneFreq (Tone)
                match tone_rx {
                    Tone::Ctcss(freq) => format!("{:.1}", freq),
                    _ => "88.5".to_string(),
                }, // cToneFreq (Tone Squelch)
                "023".to_string(), // DtcsCode (DTCS Code)
                "NN".to_string(), // DtcsPolarity (DTCS Pol)
                "023".to_string(), // RxDtcsCode (DTCS Rx Code)
                "->Tone".to_string(), // CrossMode (Cross Mode)
            );
        }
        // if TX is DCS, RX is DCS, and they are different
        if fm.tone_tx.is_some() && fm.tone_rx.is_some() &&
           matches!(
                (fm.tone_tx.as_ref().unwrap(), fm.tone_rx.as_ref().unwrap()),
                (Tone::Dcs(_), Tone::Dcs(_))
            ) {
            let tone_tx = fm.tone_tx.as_ref().unwrap();
            let tone_rx = fm.tone_rx.as_ref().unwrap();
            let mut tx_dcs_str = "D000N".to_string();
            let mut rx_dcs_str = "D000N".to_string();
            match tone_tx {
                Tone::Dcs(tx_dcs) => {
                    tx_dcs_str = tx_dcs.to_string();
                },
                _ => {}
            }
            match tone_rx {
                Tone::Dcs(rx_dcs) => {
                    rx_dcs_str = rx_dcs.to_string();
                },
                _ => {}
            }
            return (
                "Cross".to_string(), // Tone (Tone Mode)
                "88.5".to_string(), // rToneFreq (Tone)
                "88.5".to_string(), // cToneFreq (Tone Squelch)
                tx_dcs_str[1..4].to_string(), // DtcsCode (DTCS Code)
                format!("{}{}", tx_dcs_str.chars().nth(4).unwrap(), rx_dcs_str.chars().nth(4).unwrap()).replace("I", "R"), // DtcsPolarity (DTCS Pol)
                rx_dcs_str[1..4].to_string(), // RxDtcsCode (DTCS Rx Code)
                "DTCS->DTCS".to_string(), // CrossMode (Cross Mode)
            );
        }
    }
    return ( // Chirp fills out these fields with defaults
        "".to_string(), // Tone
        "88.5".to_string(), // rToneFreq
        "88.5".to_string(), // cToneFreq
        "023".to_string(), // DtcsCode
        "NN".to_string(), // DtcsPolarity
        "023".to_string(), // RxDtcsCode
        "Tone->Tone".to_string(), // CrossMode
    );
}

fn write_mode(channel: &Channel) -> Result<String, Box<dyn Error>> {
    let bandwidth = channel.fm.as_ref().unwrap().bandwidth;
    match bandwidth.to_u32().unwrap() {
        25_000 => Ok("FM".to_string()),
        12_500 => Ok("NFM".to_string()),
        _ => Err("Unsupported bandwidth".into()),
    }
}

fn write_power(power: &Power) -> String {
    match power {
        Power::Default => "5.0".to_string(),
        Power::Watts(w) if *w < 10.0 => format!("{:.1}W", w),
        Power::Watts(w) if *w < 1.0 => format!("{:.2}W", w),
        Power::Watts(w) => format!("{:.0}W", w),
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
            uprintln!(opt, Stderr, Color::Red, 4, "    tone: {}, rToneFreq: {}, cToneFreq: {}, DtcsCode: {}, DtcsPolarity: {}, RxDtcsCode: {}, CrossMode: {}",
                tone, r_tone_freq, c_tone_freq, dtcs_code, dtcs_polarity, rx_dtcs_code, cross_mode);
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
                "5.00".to_string(), // TStep
                match &channel.scan {
                    Some(Scan::Skip(skip)) if skip.all => "S".to_string(),
                    _ => "".to_string(),
                }, // Skip
                write_power(&channel.power), // Power
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
