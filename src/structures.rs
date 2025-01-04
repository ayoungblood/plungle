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
pub struct FmChannel {
    pub bandwidth: rust_decimal::Decimal,
    pub squelch_level: u8, // @TODO this needs work
    pub tone_rx: Option<Tone>,
    pub tone_tx: Option<Tone>,
}

/// Channel DMR properties
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct DmrChannel {
    pub timeslot: u8,
    pub color_code: u8,
    pub talkgroup: Option<String>,
    pub talkgroup_list: Option<String>,
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
    pub fm: Option<FmChannel>,
    pub dmr: Option<DmrChannel>,
}

/// Zone (a zone is a collection of channels)
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Zone {
    pub name: String,
    pub channels: Vec<String>,
}

/// DMR TalkgroupCallType
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum DmrTalkgroupCallType {
    Group,
    Private,
    AllCall,
}

/// DMR Talkgroup
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct DmrTalkgroup {
    pub id: u32,
    pub name: String,
    pub call_type: DmrTalkgroupCallType,
}

/// DMR Talkgroup List
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct DmrTalkgroupList {
    pub name: String,
    pub talkgroups: Vec<DmrTalkgroup>,
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

/// Configuration (radio options, settings, and user data/IDs/callsigns)
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Configuration {
    pub dmr_configuration: Option<DmrConfiguration>,
}

/// Codeplug
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Codeplug {
    pub channels: Vec<Channel>,
    pub zones: Vec<Zone>,
    pub talkgroups: Vec<DmrTalkgroup>,
    pub talkgroup_lists: Vec<DmrTalkgroupList>,
    pub config: Option<Configuration>,
}

/// Radio Properties (e.g. supported modes, bands, counts)
#[derive(Debug, Default)]
pub struct RadioProperties {
    // pub modes: Vec<ChannelMode>,
    pub channels_max: usize,
    pub channel_name_width_max: usize,
    // dynamically set
    pub channel_index_width: usize,
}
