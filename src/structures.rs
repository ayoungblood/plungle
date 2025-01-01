// src/structures.rs

use serde::{Deserialize, Serialize};

/// Channel mode
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum ChannelMode {
    AM,
    FM,
    DMR,
}

/// Tone mode
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum ToneMode {
    CTCSS,
    DCS,
}

/// Tone
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Tone {
    pub mode: ToneMode,
    pub ctcss: Option<rust_decimal::Decimal>,
    pub dcs: Option<String>,
}
/// Channel FM properties
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct FM {
    pub bandwidth: rust_decimal::Decimal,
    pub squelch_level: u8, // squelch level as a percentage, 0-100
    pub tone_rx: Option<Tone>,
    pub tone_tx: Option<Tone>,
}

/// Channel DMR properties
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct DMR {
    pub timeslot: u8,
    pub color_code: u8,
    pub talkgroup: String,
}

/// Channel
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Channel {
    pub index: u32,
    pub name: String,
    pub mode: ChannelMode,
    pub frequency_rx: rust_decimal::Decimal,
    pub frequency_tx: rust_decimal::Decimal,
    pub rx_only: bool,
    pub power: rust_decimal::Decimal,
    pub fm: Option<FM>,
    pub dmr: Option<DMR>,
}

/// Zone
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Zone {
    pub name: String,
    pub channels: Vec<u32>,
}

/// Talkgroup
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Talkgroup {
    pub id: u32,
    pub name: String,
}

/// Talkgroup List
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct TalkgroupList {
    pub name: String,
    pub talkgroups: Vec<Talkgroup>,
}

/// DMR ID
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct DmrId {
    pub id: u32,
    pub name: String,
}

/// DMR Configuration
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct DmrConfiguration {
    pub id_list: Vec<DmrId>,
}

/// Configuration
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Configuration {
    pub dmr_configuration: Option<DmrConfiguration>,
}

/// Codeplug
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Codeplug {
    pub channels: Vec<Channel>,
    pub zones: Vec<Zone>,
    pub lists: Vec<TalkgroupList>,
    pub config: Option<Configuration>,
}
