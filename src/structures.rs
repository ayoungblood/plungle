// src/structures.rs

use serde::{Deserialize, Serialize};
use crate::frequency::Frequency;
use crate::power::Power;

/// Channel mode
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum ChannelMode {
    AM,
    FM,
    DMR,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Channel {
    pub index: u32,
    pub name: String,
    pub mode: ChannelMode,
    pub frequency_rx: Frequency,
    pub frequency_tx: Frequency,
    pub rx_only: bool,
    pub power: Power,
    // FM specific fields
    pub bandwidth: Option<Frequency>,
    pub squelch: Option<String>,
    pub tone_rx: Option<String>,
    pub tone_tx: Option<String>,
    // DMR specific fields
    pub timeslot: Option<u8>,
    pub color_code: Option<u8>,
    pub talkgroup: Option<Talkgroup>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Zone {
    pub name: String,
    pub channels: Vec<Channel>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Talkgroup {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct TalkgroupList {
    pub name: String,
    pub talkgroups: Vec<Talkgroup>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Codeplug {
    pub channels: Vec<Channel>,
    pub zones: Vec<Zone>,
    pub lists: Vec<TalkgroupList>,
}
