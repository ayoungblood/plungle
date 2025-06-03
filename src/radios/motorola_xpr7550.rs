// src/radios/motorola_xpr7550.rs

use std::error::Error;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;
use itertools::Itertools;
use escaper::decode_html;

use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;

use crate::structures::*;
use crate::*;
use frequency::Frequency;

static PROPS: OnceLock<structures::RadioProperties> = OnceLock::new();
pub fn get_props() -> &'static structures::RadioProperties {
    PROPS.get_or_init(|| {
        let mut props = structures::RadioProperties::default();
        props.modes = vec![ChannelMode::FM, ChannelMode::DMR];
        props.channels_max = 1000;
        props.channel_name_width_max = 16;
        props.zones_max = 250;
        props.zone_name_width_max = 16;
        // dynamically set
        props.channel_index_width = (props.channels_max as f64).log10().ceil() as usize;
        props.zone_index_width = (props.zones_max as f64).log10().ceil() as usize;
        props
    })
}

// XPR 7000e series specs
// 136-174 MHz VHF 5W
// 403-512 MHz UHF 4W
// 806-825/851-870 MHz 800 Band 2.5W
// 896-902/934-941 MHz 900 Band 2.5W

// We are parsing the decrypted XML from a CPS 16 build 828 codeplug
// XML is structured as follows:
// Channels are contained in <CNV_PER_CMP_TYPE> elements, order of elements inside these tags varies
// <CNV_PER_CMP_TYPE ..> - attribute ListID is an index, but not in the order that channels appear in the codeplug(!)
//   <CP_CNVPERSALIAS> - channel name, with HTML entities for special characters (e.g. &lt;/&gt; for < and >)
//   <CP_RXFREQ> - receive frequency in MHz, six decimal places
//   <CP_TXFREQ> - transmit frequency in MHz, six decimal places, Applicable=Disabled for receive-only channels
//   <CP_RXONLYEN> - receive only, 1 or 0
//       when set, sets Applicable=Disabled for CP_TXFREQ, CP_TXINHXPLEN, CP_TXPWR, CP_TXREFFREQ, CP_TXSIGSYSIT, CP_TXSIGSYSITID, CP_TXSIGSYSITTYPE, CP_TXSQCHTY, CP_TOT, CP_TOTREKEYDELAY, CP_TOTWRN, CP_VOXSTATE
//   <CP_TOT> - transmit timeout timer in seconds, Applicable=Disabled for receive-only channels
//   <CP_TXPWR> - transmit power, HIGHPWR or LOWPWR, Applicable=Disabled for receive-only channels
//   <CP_USELD> - not sure what this is but it changes when it shouldn't [OFF, ON]
// Because the order of tags is not consistent, we read every element into a hashmap, and then parse the channel data from the hashmap
// This separates the XML parsing from the channel parsing, which is useful for debugging, and is useful when a field value is dependent on multiple tag values
#[derive(PartialEq, Debug)]
enum XmlApplicable {
    Enabled,
    Disabled,
    Na,
}
#[derive(Debug)]
struct XmlChannelFieldContent {
    value: String,
    type_id: Option<String>,
    applicable: XmlApplicable,
    list_id: usize,
}

type XmlChannelHash = HashMap<String, XmlChannelFieldContent>;

// READ ///////////////////////////////////////////////////////////////////////

fn get_list_id(e: &quick_xml::events::BytesStart) -> Option<usize> {
    for attr in e.attributes() {
        let a = attr.unwrap();
        if a.key == QName(b"ListID") {
            return Some(
                std::str::from_utf8(&a.value)
                    .unwrap()
                    .parse::<usize>()
                    .unwrap(),
            );
        }
    }
    None
}

fn parse_channel_record(opt: &Opt, id: usize, contents: &str) -> Result<Channel, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    let mut channel = Channel::default();
    channel.index = id + 1; // channels are zero-indexed in the XML
                            // contents is a string of XML
    let mut reader = Reader::from_str(contents);
    let mut channel_hash = XmlChannelHash::new();
    //eprintln!("contents = {}", contents);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                // add to hashmap
                let field = XmlChannelFieldContent {
                    value: reader.read_text(e.name())?.into_owned(),
                    type_id: e
                        .attributes()
                        .find(|a| a.as_ref().unwrap().key == quick_xml::name::QName(b"TypeID"))
                        .map(|a| a.unwrap().value.to_vec())
                        .map(|v| String::from_utf8(v).unwrap()),
                    applicable: e
                        .attributes()
                        .find(|a| a.as_ref().unwrap().key == quick_xml::name::QName(b"Applicable"))
                        .map(|a| a.unwrap().value)
                        .as_ref()
                        .map(|v| match std::str::from_utf8(v).unwrap() {
                            "Enabled" => XmlApplicable::Enabled,
                            "Disabled" => XmlApplicable::Disabled,
                            "NA" => XmlApplicable::Na,
                            _ => panic!(
                                "Unknown Applicable value: {}",
                                std::str::from_utf8(v).unwrap()
                            ),
                        })
                        .unwrap(),
                    list_id: e
                        .attributes()
                        .find(|a| a.as_ref().unwrap().key == quick_xml::name::QName(b"ListID"))
                        .map(|a| a.unwrap().value)
                        .as_ref()
                        .map(|v| std::str::from_utf8(v).unwrap().parse::<usize>().unwrap())
                        .unwrap(),
                };
                // println!("e.name = {:?}", e.name());
                // println!("    value = {:?}", reader.read_text(e.name())?.into_owned());
                // println!("    attributes = {:?}", e.attributes().map(|a| a.unwrap()).collect::<Vec<_>>());
                // add to the hashmap
                channel_hash.insert(
                    String::from_utf8_lossy(e.name().as_ref()).to_string(),
                    field,
                );
            }
            _ => (),
        }
    }
    // print out the channel_hash
    for fieldname in channel_hash.keys().sorted() {
        let field = channel_hash.get(fieldname).unwrap();
        if field.applicable == XmlApplicable::Enabled {
            println!("{:03} {:40} {:40} {:?}", field.list_id, fieldname, field.value, field.type_id);
        }
    }
    eprintln!("channel.index: {}, field.list_id: {}", channel.index, channel_hash.get("CP_TOT").unwrap().list_id);
    // set channel parameters
    // CP_CNVPERSALIAS: channel name, with HTML entities for special characters
    channel.name = match decode_html(&channel_hash.get("CP_CNVPERSALIAS").unwrap().value.to_string()) {
        Ok(s) => s,
        Err(reason) => return Err(format!("Error {:?} while decoding channel name (bad CP_CNVPERSALIAS)", reason.kind).into()),
    };
    // CP_PERSTYPE: "ANLGCONV" for FM, "DGTLCONV6PT25" for DMR
    let cp_perstype = channel_hash.get("CP_PERSTYPE");
    channel.mode = match cp_perstype.unwrap().value.as_str() {
        "ANLGCONV" => ChannelMode::FM,
        "DGTLCONV6PT25" => ChannelMode::DMR,
        _ => return Err(format!("Cannot parse mode (unrecognized CP_PERSTYPE): {}", cp_perstype.unwrap().value).into()),
    };
    // CP_RXFREQ: RX frequency in MHz
    let cp_rxfreq_str = channel_hash.get("CP_RXFREQ").unwrap().value.trim();
    channel.frequency_rx = Frequency::from_mhz_str(cp_rxfreq_str)?;
    // CP_TXFREQ: TX frequency in MHz
    let cp_txfreq_str = channel_hash.get("CP_TXFREQ").unwrap().value.trim();
    channel.frequency_tx = Frequency::from_mhz_str(cp_txfreq_str)?;
    // CP_RXONLYEN: "1" for receive-only, "0" otherwise
    channel.rx_only = channel_hash.get("CP_RXONLYEN").unwrap().value.trim() == "1";
    // CP_TOT: TX timeout in seconds, 0 for disabled
    let cp_tot_str = channel_hash.get("CP_TOT").unwrap().value.trim();
    if cp_tot_str == "0" {
        channel.tx_tot = Timeout::Infinite;
    } else {
        channel.tx_tot = Timeout::Seconds(cp_tot_str.parse::<u32>()?);
    }
    // CP_TXPWR: transmit power, "LOWPWR" or "HIGHPWR"
    // 800/900 - low 1.0W, high 2.5W
    // UHF - low 1.0W, high 4.0W
    // VHF - low 1.0W, high 5.0W
    let cp_txpwr = channel_hash.get("CP_TXPWR");
    if cp_txpwr.is_some() && cp_txpwr.unwrap().applicable == XmlApplicable::Enabled {
        channel.power = match cp_txpwr.unwrap().value.as_str() {
            "LOWPWR" => Power::Watts(1.0),
            "HIGHPWR" => if channel.frequency_tx < Frequency::from_mhz(174.0) {
                Power::Watts(5.0)
            } else if channel.frequency_tx < Frequency::from_mhz(512.0) {
                Power::Watts(4.0)
            } else {
                Power::Watts(2.5)
            }
            _ => return Err(format!("Cannot parse power (unrecognized CP_TXPWR): {}", cp_txpwr.unwrap().value).into()),
        };
    }
    // CP_TXINHXPLEN: TX inhibit, [ALWAYS,ONCHNNLFREE,MTCHCLRCD] // @TODO review this
    let cp_txinhxplen = channel_hash.get("CP_TXINHXPLEN");
    if cp_txinhxplen.is_some() && cp_txinhxplen.unwrap().applicable == XmlApplicable::Enabled {
        channel.tx_permit = Some(
            match cp_txinhxplen.unwrap().value.as_str() {
                "ALWAYS" => TxPermit::Always,
                "ONCHNNLFREE" => TxPermit::ChannelFree,
                "MTCHCLRCD" => TxPermit::ColorCodeSame, // @TODO review this
                _ => return Err(format!("Cannot parse TX inhibit (unrecognized TXINHXPLEN): {}", cp_txinhxplen.unwrap().value).into()),
            },
        );
    }
    // mode-specific fields
    if channel.mode == ChannelMode::FM {
        let mut fm = FmChannel::default();
        // CP_CHNLBWDTH: bandwidth [STR_25KHZ, STR_20KHZ, STR_12PT5KHZ]
        let cp_chnlbwdth_str = channel_hash.get("CP_CHNLBWDTH").unwrap().value.as_str();
        fm.bandwidth = match cp_chnlbwdth_str {
            "STR_25KHZ" => Frequency::from_khz(25.0),
            "STR_20KHZ" => Frequency::from_khz(20.0),
            "STR_12PT5KHZ" => Frequency::from_khz(12.5),
            _ => return Err(format!("Cannot parse bandwidth (unrecognized CP_CHNLBWDTH): {}", cp_chnlbwdth_str).into()),
        };
        // @TODO set squelch
        // CP_XSQCHTY
        let cp_xsqchty = channel_hash.get("CP_XSQCHTY");
        let cp_rxtplfreq = channel_hash.get("CP_RXTPLFREQ");
        let cp_rxdplcd = channel_hash.get("CP_RXDPLCD");
        let cp_rxdplinv = channel_hash.get("CP_RXDPLINV");
        if cp_xsqchty.is_some() && cp_xsqchty.unwrap().applicable == XmlApplicable::Enabled {
            fm.tone_rx = match cp_xsqchty.unwrap().value.as_str() {
                "CSQ" => None,
                "TPL" => Some(Tone::Ctcss(cp_rxtplfreq.unwrap().value.parse::<f64>()?)),
                "DPL" => Some(Tone::Dcs(cp_rxdplcd.unwrap().value.clone()
                    + if cp_rxdplinv.unwrap().value == "1" { "I" } else { "N" })),
                _ => return Err(format!("Cannot parse RX squelch (unrecognized CP_XSQCHTY): {}", cp_xsqchty.unwrap().value).into()),
            };
        }
        // CP_TXSQCHTY
        let cp_txsqchty = channel_hash.get("CP_TXSQCHTY");
        let cp_txttplfreq = channel_hash.get("CP_TXTTPLFREQ");
        let cp_txtdplcd = channel_hash.get("CP_TXTDPLCD");
        let cp_txdplinv = channel_hash.get("CP_TXDPLINV");
        if cp_txsqchty.is_some() && cp_txsqchty.unwrap().applicable == XmlApplicable::Enabled {
            fm.tone_tx = match cp_txsqchty.unwrap().value.as_str() {
                "CSQ" => None,
                "TPL" => Some(Tone::Ctcss(cp_txttplfreq.unwrap().value.parse::<f64>()?)),
                "DPL" => Some(Tone::Dcs(cp_txtdplcd.unwrap().value.clone()
                    + if cp_txdplinv.unwrap().value == "1" { "I" } else { "N" })),
                _ => return Err(format!("Cannot parse TX squelch (unrecognized CP_TXSQCHTY): {}", cp_txsqchty.unwrap().value).into()),
            };
        }
        channel.fm = Some(fm);
    }
    if channel.mode == ChannelMode::DMR {
        let mut dmr = DmrChannel::default();
        // CP_SLTASSGMNT: timeslot [SLOT1, SLOT2]
        let cp_sltassgmnt_str = channel_hash.get("CP_SLTASSGMNT").unwrap().value.as_str();
        dmr.timeslot = cp_sltassgmnt_str.replace("SLOT", "").parse::<u8>()?;
        // CP_COLORCODE: color code
        let cp_colorcode_str = channel_hash.get("CP_COLORCODE").unwrap().value.as_str();
        dmr.color_code = cp_colorcode_str.parse::<u8>()?;
        // @TODO
        // dmr.talkgroup, dmr.talkgrouplist, dmr.id_name
        channel.dmr = Some(dmr);
    }
    Ok(channel)
}

// This is specific to CPS 16 build 828 codeplugs
// The CPS saves an encrypted XML file (*.ctb), which must be decrypted for this to work
// Channel data lives in <LTD_CODEPLUG<APP_PARTITION<CNV_PER_CMP_TYPE_GRP<CNV_PER_CMP_TYPE
pub fn read(opt: &Opt, input_path: &PathBuf) -> Result<Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    let mut codeplug = Codeplug::default();
    codeplug.source = format!(
        "{}",
        Path::new(file!()).file_stem().unwrap().to_str().unwrap()
    );

    // check that the input path is a file
    if !input_path.is_file() {
        uprintln!(
            opt,
            Stderr,
            Color::Red,
            None,
            "You lied to me when you told me this was a file: {}",
            input_path.display()
        );
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{} is not a file", input_path.display()),
        )));
    }
    // set up the XML parser
    // let file = File::open(input_path)?;
    // let reader = BufReader::new(file);
    // read the file in as bytes
    let contents = std::fs::read(input_path)?;
    // parse the XML
    let mut reader = Reader::from_str(std::str::from_utf8(&contents).unwrap());
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            // exits the loop when reaching end of file
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"CNV_PER_CMP_TYPE" => {
                        // this is the beginning of an analog channel
                        let id = get_list_id(&e);
                        if let Some(id) = id {
                            let contents =
                                reader.read_text(QName(b"CNV_PER_CMP_TYPE"))?.into_owned();
                            let channel = parse_channel_record(opt, id, &contents)?;
                            codeplug.channels.push(channel);
                        }
                    }
                    _ => {}
                }
            }
            // There are several other `Event`s we do not consider here
            _ => (),
        }
        buf.clear();
    }

    Ok(codeplug)
}
