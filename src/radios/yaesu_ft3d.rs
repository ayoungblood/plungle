// src/radios/yaesu_ft3d.rs

use std::error::Error;
use std::fs;
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
// - Channel No: index (1-900)
// - Priority CH: [OFF,ON] default OFF
// - Receive Frequency: MHz, zero-padded to 6 decimal places
// - Transmit Frequency: MHz, zero-padded to 6 decimal places
// - Offset Frequency: MHz, zero-padded to 6 decimal places
// - Offset Direction: [OFF,+RPT,-RPT,-/+]
// - AUTO MODE: [OFF,ON] default OFF
// - Operating Mode: [FM, AM] default FM, FM if AUTO MODE is ON
// - DIG/ANALOG: [FM, AMS, DN]
// - TAG: [OFF,ON] default ON (global)
// - Name: string, 16 characters max
// - Tone Mode: [OFF,TONE,TONE SQL,DCS,REV TONE,PR FREQ,PAGER]
// - CTCSS Frequency: [67.0 Hz,..,254.1 Hz]
// - DCS Code: [023,..,754] default 023
// - DCS Polarity: [RX Normal TX Normal,RX Invert TX Normal,RX Both TX Normal,RX Normal TX Invert,RX Invert TX Invert,RX Both TX Invert]
// - User CTCSS: default 1600 Hz
// - RX DG-ID: [RX 00,..,RX 99] default RX 00
// - TX DG-ID: [TX 00,..,TX 99] default TX 00
// - TX Power: [L1 (0.3W),L2 (1W),L3 (2.5W),High (5W)]
// - Skip: [OFF,SKIP,SELECT] default OFF
// - AUTO STEP: [OFF,ON] default ON
// - Step: [5.0KHz,25.0KHz,??] default 5.0KHz for VHF, default 25.0KHz for UHF
// - Memory Mask: [OFF,ON] default OFF
// - ATT: [OFF,ON] default OFF
// - S-Meter SQL: [OFF,??] default OFF
// - Bell: [OFF,ON] default OFF
// - Narrow: [OFF,ON] default OFF
// - Clock Shift: [OFF,ON] default OFF
// - BANK1..BANK24: [OFF,ON] default OFF
// - Comment:
