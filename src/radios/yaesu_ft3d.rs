// src/radios/yaesu_ft3d.rs

use std::error::Error;
use std::path::PathBuf;
use std::path::Path;
use std::sync::OnceLock;

use crate::*;
use crate::structures::*;
use frequency::Frequency;

static PROPS: OnceLock<structures::RadioProperties> = OnceLock::new();
pub fn get_props() -> &'static structures::RadioProperties {
    PROPS.get_or_init(|| {
        let mut props = structures::RadioProperties::default();
        props.modes = vec![structures::ChannelMode::FM, structures::ChannelMode::DMR];
        props.channels_max = 900;
        props.channel_name_width_max = 16;
        props.zones_max = 24;
        props.zone_name_width_max = 16;
        // dynamically set
        props.channel_index_width = (props.channels_max as f64).log10().ceil() as usize;
        props.zone_index_width = (props.zones_max as f64).log10().ceil() as usize;
        props
    })
}

// CSV Export Format
// FT3D Programmer ADMS-11 Ver 1.0.0.0

// *.csv Fields (no header)
//  0 Channel No: index (1-900)
//  1 Priority CH: [OFF,ON] default OFF
//  2 Receive Frequency: MHz, zero-padded to 6 decimal places
//  3 Transmit Frequency: MHz, zero-padded to 6 decimal places
//  4 Offset Frequency: MHz, zero-padded to 6 decimal places
//  5 Offset Direction: [OFF,+RPT,-RPT,-/+]
//  6 AUTO MODE: [OFF,ON] default OFF
//  7 Operating Mode: [FM, AM] default FM, FM if AUTO MODE is ON
//  8 DIG/ANALOG: [FM, AMS, DN]
//  9 TAG: [OFF,ON] default ON (global)
// 10 Name: string, 16 characters max
// 11 Tone Mode: [OFF,TONE,TONE SQL,DCS,REV TONE,PR FREQ,PAGER]
// 12 CTCSS Frequency: [67.0 Hz,..,254.1 Hz]
// 13 DCS Code: [023,..,754] default 023
// 14 DCS Polarity: [RX Normal TX Normal,RX Invert TX Normal,RX Both TX Normal,RX Normal TX Invert,RX Invert TX Invert,RX Both TX Invert]
// 15 User CTCSS: default 1600 Hz
// 16 RX DG-ID: [RX 00,..,RX 99] default RX 00
// 17 TX DG-ID: [TX 00,..,TX 99] default TX 00
// 18 TX Power: [L1 (0.3W),L2 (1W),L3 (2.5W),High (5W)]
// 19 Skip: [OFF,SKIP,SELECT] default OFF
// 20 AUTO STEP: [OFF,ON] default ON
// 21 Step: [5.0KHz,25.0KHz,??] default 5.0KHz for VHF, default 25.0KHz for UHF
// 22 Memory Mask: [OFF,ON] default OFF
// 23 ATT: [OFF,ON] default OFF
// 24 S-Meter SQL: [OFF,ON] default OFF
// 25 Bell: [OFF,ON] default OFF
// 26 Narrow: [OFF,ON] default OFF
// 27 Clock Shift: [OFF,ON] default OFF
// 28 BANK1..BANK24: [OFF,ON] default OFF
// 29 Comment:

type CsvRecord = Vec<String>;

fn parse_channel_record(opt: &Opt, record: &CsvRecord) -> Result<Channel, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "{:?}", record);

    let mut channel = Channel::default();

    // if the second column is not empty, this is a real channel
    if !record[1].is_empty() {
        channel.index = record[0].parse::<usize>()?;
        channel.frequency_rx = Frequency::from_khz(record[2].parse::<f64>()?);
        channel.frequency_tx = Frequency::from_khz(record[3].parse::<f64>()?);
        channel.mode = match record[7].as_str() {
            "FM" => ChannelMode::FM,
            _ => ChannelMode::AM,
        };
        channel.power = match record[18].as_str() {
            "L1" => Power::Watts(0.3),
            "L2" => Power::Watts(1.0),
            "L3" => Power::Watts(2.5),
            _ => Power::Watts(5.0),
        };
        channel.name = record[10].clone();
    }



    Ok(channel)
}

pub fn read(opt: &Opt, input_path: &PathBuf) -> Result<Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    let mut codeplug = Codeplug::default();
    codeplug.source = format!("{}", Path::new(file!()).file_stem().unwrap().to_str().unwrap());

    // check that the input path is a file
    if !input_path.is_file() {
        return Err(format!("You lied to me when you told me this was a regular file: {}", input_path.display()).into());
    }
    let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(&input_path)?;
    for result in reader.deserialize() {
        let record: CsvRecord = result?;
        // convert from CSV record to Channel struct
        let channel = parse_channel_record(&opt, &record)?;
        if channel.index > 0 {
            // append to codpelug.channels
            codeplug.channels.push(channel);
        }
    }
    Ok(codeplug)
}
