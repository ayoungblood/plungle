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
