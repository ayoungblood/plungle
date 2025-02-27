// src/radios/alinco_drmd500.rs

static PROPS: OnceLock<structures::RadioProperties> = OnceLock::new();
pub fn get_props() -> &'static structures::RadioProperties {
    PROPS.get_or_init(|| {
        let mut props = structures::RadioProperties::default();
        props.modes = vec![structures::ChannelMode::FM, structures::ChannelMode::DMR];
        props.channels_max = 4000;
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
// Alinco DR-MD500 CPS Version v1.06
/* Files
 * 2ToneEncode.CSV
 * 5ToneEncode.CSV
 * APRS.CSV
 * AnalogAddressBook.CSV
 * AutoRepeaterOffsetFrequencys.CSV
 * Channel.CSV
 * ContactTalkGroups.CSV
 * DTMFEncode.CSV
 * DigitalContactList.CSV
 * FM.CSV
 * HotKey_HotKey.CSV
 * HotKey_QuickCall.CSV
 * HotKey_State.CSV
 * PrefabricatedSMS.CSV
 * RadioIDList.CSV
 * ReceiveGroupCallList.CSV
 * RoamingChannel.CSV
 * RoamingZone.CSV
 * ScanList.CSV
 * Zone.CSV
 */

// Channel.CSV
// - No.: index
// - Channel Name: name
// - Receive Frequency: in MHz
// - Transmit Frequency: in MHz
// - Channel Type: [A-Analog, D-Digital]
// - Transmit Power: [High,Middle,Low,Small]
// - Band Width: [25K,12.5K]
// - CTCSS/DCS Decode: [Off,62.5-254.1,D021N,D777I]
// - CTCSS/DCS Encode: [Off,62.5-254.1,D021N,D777I]
// - Contact: talkgroup name
// - Contact Call Type: [Group Call, All Call, Private Call]
// - Contact TG/DMR ID: talkgroup or DMR ID
// - Radio ID: radio ID name
// - Busy Lock/TX Permit: [Off,Always,ChannelFree,Different Color Code,Same Color Code]
// - Squelch Mode: [Carrier,CTCSS/DCS]
// - Optional Signal: [Off,??]
// - DTMF ID: [1,??]
// - 2Tone ID: [1,??]
// - 5Tone ID: [1,??]
// - PTT ID: [Off,??]
// - Color Code: [0-15], 1 for analog
// - Slot: [1,2]
// - Scan List: [None,scanlist name]
// - Receive Group List: [None,group name]
// - PTT Prohibit: [Off,??]
// - Reverse: [Off,??]
// - TDMA: [Off,??]
// - TDMA Adaptive: [Off,??]
// - AES Digital Encryption: [Normal Encryption,??]
// - Digital Encryption: [Off,??]
// - Call Confirmation: [Off,??]
// - Talk Around(Simplex): [Off,??]
// - Work Alone: [Off,??]
// - Custom CTCSS: frequency in Hz, default 251.1
// - 2TONE Decode: [0, ??]
// - Ranging: [Off, ??]
// - Simplex: [Off, ??]
// - APRS RX: [Off, ??]
// - Analog APRS PTT Mode: [Off, ??]
// - Digital APRS PTT Mode: [Off, ??]
// - APRS Report Type: [Off, ??]
// - Digital APRS Report Channel: [1, ??]
// - Correct Frequency[Hz]: [0, ??]
// - SMS Confirmation: [Off, ??]
// - Exclude Channel From Roaming: [0, ??]
// - DMR Mode: 1 for analog, 0 for digital??
// - DataACK Disable: [0, ??]

// ContactTalkGroups.CSV
// - No.: index
// - Radio ID: talkgroup/contact ID
// - Name: name
// - Call Type: [Group Call, All Call, Private Call]
// - Call Alert: [None, ??]

// RadioIDList.CSV
// - No.: index
// - Radio ID: radio ID
// - Name: name

// ReceiveGroupCallList.CSV
// - No.: index
// - Group Name: name
// - Contact: talkgroup names, "|" separated\
// - Contact TG/DMR ID: talkgroup or DMR IDs, "|" separated

// ScanList.CSV
// - No.: index
// - Scan List Name: name
// - Scan Channel Member: channel names, "|" separated
// - Scan Channel Member RX Frequency: in MHz, "|" separated
// - Scan Channel Member TX Frequency: in MHz, "|" separated
// - Scan Mode: [Off, ??]
// - Priority Channel Select: [Off, ??]
// - Priority Channel 1: [Off, ??]
// - Priority Channel RX Frequency: empty or in MHz
// - Priority Channel TX Frequency: empty or in MHz
// - Priority Channel 2: [Off, ??]
// - Priority Channel RX Frequency: empty or in MHz
// - Priority Channel TX Frequency: empty or in MHz
// - Revert Channel: [Selected, ??]
// - Look Back Time A[s]: default 2.0
// - Look Back Time B[s]: default 3.0
// - Dropout Delay Time[s]: default 3.1
// - Dwell Time[s]: default 3.1

// Zone.CSV
// - No.: index
// - Zone Name: name
// - Zone Channel Member: channel names, "|" separated
// - Zone Channel Member RX Frequency: in MHz, "|" separated
// - Zone Channel Member TX Frequency: in MHz, "|" separated
// - A Channel: channel name
// - A Channel RX Frequency: in MHz
// - A Channel TX Frequency: in MHz
// - B Channel: channel name
// - B Channel RX Frequency: in MHz
// - B Channel TX Frequency: in MHz

