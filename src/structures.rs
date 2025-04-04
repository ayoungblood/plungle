// src/structures.rs

use serde::{Deserialize, Serialize};

/// Channel mode
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Default)]
pub enum ChannelMode {
    #[default]
    AM,
    FM,
    DMR,
}

/// Squelch
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Default)]
pub enum Squelch {
    #[default]
    Default,
    Percent(u8), // 0-100
}

/// Tone
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum Tone {
    Ctcss(f64),
    Dcs(String),
}

/// Channel FM properties
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct FmChannel {
    pub bandwidth: rust_decimal::Decimal,
    pub squelch: Squelch,
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
    pub id_name: Option<String>,
}

/// Timeout
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Default)]
pub enum Timeout {
    #[default]
    Default,
    Seconds(u32),
    Infinite,
}

/// Power
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Default)]
pub enum Power {
    #[default]
    Default,
    Watts(f64),
}

/// Tx Permit
// aka TX Admit, TX Authority, TX Inhibit
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Default)]
pub enum TxPermit {
    #[default]
    Always,
    ChannelFree,
    CtcssDcsDifferent,
    ColorCodeSame,
    ColorCodeDifferent,
}

/// ScanSkip
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct ScanSkip {
    pub zone: bool,
    pub all: bool,
}

/// Scan
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum Scan {
    Skip(ScanSkip),
    ScanList(String),
}

/// Channel
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Default)]
pub struct Channel {
    pub index: usize,
    pub name: String,
    pub mode: ChannelMode,
    pub frequency_rx: rust_decimal::Decimal,
    pub frequency_tx: rust_decimal::Decimal,
    pub rx_only: bool,
    pub tx_tot: Timeout,
    pub power: Power,
    pub tx_permit: Option<TxPermit>,
    pub scan: Option<Scan>,
    // mode-specific properties
    pub fm: Option<FmChannel>,
    pub dmr: Option<DmrChannel>,
}

/// Zone (a zone is a collection of channels)
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Zone {
    pub index: usize,
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
    pub index: usize,
    pub id: u32,
    pub name: String,
    pub call_type: DmrTalkgroupCallType,
    pub alert: bool,
}

/// DMR Talkgroup List
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct DmrTalkgroupList {
    pub index: usize,
    pub name: String,
    pub talkgroups: Vec<DmrTalkgroup>,
}

/// Scan List
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct ScanList {
    pub index: usize,
    pub name: String,
    pub channels: Vec<String>,
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
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Default)]
pub struct Codeplug {
    pub channels: Vec<Channel>,
    pub zones: Vec<Zone>,
    pub scanlists: Vec<ScanList>,
    pub talkgroups: Vec<DmrTalkgroup>,
    pub talkgroup_lists: Vec<DmrTalkgroupList>,
    pub config: Option<Configuration>,
    pub source: String, // source radio
}

/// Radio Properties (e.g. supported modes, bands, counts)
#[derive(Debug, Default, Clone)]
pub struct RadioProperties {
    pub modes: Vec<ChannelMode>,
    pub channels_max: usize,
    pub channel_name_width_max: usize,
    pub zones_max: usize,
    pub zone_name_width_max: usize,
    // dynamically set
    pub channel_index_width: usize,
    pub zone_index_width: usize,
}
