// src/radios/motorola_xpr7550.rs

use std::error::Error;
//use std::fs;
use std::path::PathBuf;
use std::path::Path;
use std::collections::HashMap;
// use rust_decimal::prelude::*;
use std::sync::OnceLock;
// use std::fs::File;
// use std::io::BufReader;

use quick_xml::events::{Event};
use quick_xml::reader::Reader;
use quick_xml::name::QName;

use crate::*;
use crate::structures::*;

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
#[derive(Debug)]
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
            return Some(std::str::from_utf8(&a.value).unwrap().parse::<usize>().unwrap());
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
    eprintln!("contents = {}", contents);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            Ok(Event::Eof ) => break,
            Ok(Event::Start(e)) => {
                // add to hashmap
                let field = XmlChannelFieldContent {
                    value: reader.read_text(e.name())?.into_owned(),
                    type_id: e.attributes().find(|a| a.as_ref().unwrap().key == b"TypeID").map(|a| a.unwrap().value.to_vec()).map(|v| String::from_utf8(v).unwrap()),
                    applicable: e.attributes().find(|a| a.as_ref().unwrap().key == b"Applicable").map(|a| a.unwrap().value).map(|v| {
                        match std::str::from_utf8(v).unwrap() {
                            "Enabled" => XmlApplicable::Enabled,
                            "Disabled" => XmlApplicable::Disabled,
                            "NA" => XmlApplicable::Na,
                            _ => panic!("Unknown Applicable value: {}", std::str::from_utf8(v).unwrap()),
                        }
                    }).unwrap(),
                    list_id: e.attributes().find(|a| a.as_ref().unwrap().key == b"ListID").map(|a| a.unwrap().value).map(|v| std::str::from_utf8(v).unwrap().parse::<usize>().unwrap()).unwrap(),
                };
                println!("e.name = {:?}", e.name());
                println!("    value = {:?}", reader.read_text(e.name())?.into_owned());
                println!("    attributes = {:?}", e.attributes().map(|a| a.unwrap()).collect::<Vec<_>>());
                // // common channel attributes
                // match e.name().as_ref() {
                //     b"CP_PERSTYPE" => { // channel type
                //         let perstype = reader.read_text(QName(b"CP_PERSTYPE"))?.into_owned();
                //         if perstype == "ANLGCONV" {
                //             channel.mode = ChannelMode::FM;
                //         } else if perstype == "DGTLCONV6PT25" {
                //             channel.mode = ChannelMode::DMR;
                //         } else {
                //             panic!("Unknown channel type: {}", perstype);
                //         }
                //     }
                //     b"CP_CNVPERSALIAS" => { // channel name
                //         channel.name = reader.read_text(QName(b"CP_CNVPERSALIAS"))?.into_owned();
                //     },
                //     b"CP_RXFREQ" => { // receive frequency
                //         let freq_str = reader.read_text(QName(b"CP_RXFREQ"))?.into_owned();
                //         channel.frequency_rx = Decimal::from_str(&freq_str)? * Decimal::new(1_000_000, 0);
                //     },
                //     b"CP_TXFREQ" => { // transmit frequency
                //         let freq_str = reader.read_text(QName(b"CP_TXFREQ"))?.into_owned();
                //         channel.frequency_tx = Decimal::from_str(&freq_str)? * Decimal::new(1_000_000, 0);
                //     },
                //     b"CP_RXONLYEN" => { // receive only
                //         let rxonlyen = reader.read_text(QName(b"CP_RXONLYEN"))?.into_owned();
                //         channel.rx_only = rxonlyen == "1";
                //     },
                //     b"CP_TOT" => { // TOT
                //         let tot = reader.read_text(QName(b"CP_TOT"))?.into_owned();
                //         channel.tx_tot = Timeout::Seconds(tot.parse::<u32>().unwrap());
                //     },
                //     b"CP_TXPWR" => { // power
                //         let txpwr = reader.read_text(QName(b"CP_TXPWR"))?.into_owned();
                //         let mhz_tx = channel.frequency_tx.to_f64().unwrap() / 1_000_000.0;

                //         channel.power = if txpwr == "HIGHPWR" {
                //             if mhz_tx > 136.0 && mhz_tx < 174.0 {
                //                 Power::Watts(5.0)
                //             } else if mhz_tx >= 403.0 && mhz_tx <= 512.0 {
                //                 Power::Watts(4.0)
                //             } else if (mhz_tx >= 806.0 && mhz_tx <= 825.0) || (mhz_tx >= 851.0 && mhz_tx <= 870.0) {
                //                 Power::Watts(2.5)
                //             } else if (mhz_tx >= 896.0 && mhz_tx <= 902.0) || (mhz_tx >= 934.0 && mhz_tx <= 941.0) {
                //                 Power::Watts(2.5)
                //             } else {
                //                 return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Unknown frequency range for power: {}", mhz_rx))));
                //             }
                //         } else {
                //             Power::Watts(1.0)
                //         };
                //     },
                //     b"CP_TXINHXPLEN" => {
                //         // @TODO: implement
                //     },
                //     // @TODO implement scan
                //     _ => {},
                // }
            }
            _ => (),
        }
    }
    Ok(channel)
}

// This is specific to CPS 16 build 828 codeplugs
// The CPS saves an encrypted XML file (*.ctb), which must be decrypted for this to work
// Channel data lives in <LTD_CODEPLUG<APP_PARTITION<CNV_PER_CMP_TYPE_GRP<CNV_PER_CMP_TYPE
pub fn read(input_path: &PathBuf, opt: &Opt) -> Result<Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    let mut codeplug = Codeplug::default();
    codeplug.source = format!("{}", Path::new(file!()).file_stem().unwrap().to_str().unwrap());

    // check that the input path is a file
    if !input_path.is_file() {
        uprintln!(opt, Stderr, Color::Red, None, "You lied to me when you told me this was a file: {}", input_path.display());
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, format!("{} is not a file", input_path.display()))));
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
                            let contents = reader.read_text(QName(b"CNV_PER_CMP_TYPE"))?.into_owned();
                            let channel = parse_channel_record(opt, id, &contents)?;
                            codeplug.channels.push(channel);
                        }
                    },
                    _ => {},
                }
            }
            // There are several other `Event`s we do not consider here
            _ => (),
        }
        buf.clear();
    }

    Ok(codeplug)
}