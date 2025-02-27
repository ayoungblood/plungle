// src/radios/tyt_mduv390.rs

use std::error:Error;
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
        props.channels_max = 3000;
        props.channel_name_width_max = 16;
        props.zones_max = 250;
        props.zone_name_width_max = 16;
        // dynamically set
        props.channel_index_width = (props.channels_max as f64).log10().ceil() as usize;
        props.zone_index_width = (props.zones_max as f64).log10().ceil() as usize;
        props
    })
}

// CSV Export Format
// TYT MD-UV380 CPS V2.41
/* Files
 * channels.csv
 * contacts.csv
 */

// channels.csv
// - Channel Mode
// - Channel Name
// - RX Frequency(MHz)
// - TX Frequency(MHz)
// - Band Width
// - Scan List
// - Squelch
// - RX Ref Frequency
// - TX Ref Frequency
// - TOT[s]
// - TOT Rekey Delay[s]
// - Power
// - Admit Criteria
// - Auto Scan
// - Rx Only
// - Lone Worker
// - VOX
// - Allow Talkaround
// - Send GPS Info
// - Receive GPS Info
// - Private Call Confirmed
// - Emergency Alarm Ack
// - Data Call Confirmed
// - Allow Interrupt
// - DCDCM Switch
// - Leader/MS
// - Emergency System
// - Contact Name
// - Group List
// - Color Code
// - Repeater Slot
// - In Call Criteria
// - Privacy
// - Privacy No.
// - GPS System
// - CTCSS/DCS Dec
// - CTCSS/DCS Enc
// - RX Signaling System
// - Tx Signaling System
// - QT Reverse
// - Non-QT/DQT Turn-off Freq
// - Display PTT ID
// - Reverse Burst/Turn-off Code
// - Decode 1
// - Decode 2
// - Decode 3
// - Decode 4
// - Decode 5
// - Decode 6
// - Decode 7
// - Decode 8

// contacts.csv
// - Contact Name
// - Call Type
// - Call ID
// - Call Receive Tone

type CsvRecord = HashMap<String, String>;

// READ ///////////////////////////////////////////////////////////////////////

pub fn read(input_path: &PathBuf, opt: &Opt) -> Result<Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    Ok(())
}