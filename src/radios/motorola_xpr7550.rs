// src/radios/motorola_xpr7550.rs

use std::error::Error;
//use std::fs;
use std::path::PathBuf;
use std::path::Path;
//use std::collections::HashMap;
//use rust_decimal::prelude::*;
use std::sync::OnceLock;
// use std::fs::File;
// use std::io::BufReader;

use quick_xml::events::{Event};
use quick_xml::reader::Reader;
use quick_xml::name::QName;

use crate::*;
use crate::structures::*;

static PROPS: OnceLock<structures::RadioProperties> = OnceLock::new();
fn get_props() -> &'static structures::RadioProperties {
    PROPS.get_or_init(|| {
        let mut props = structures::RadioProperties::default();
        props.channels_max = 1000;
        props.channel_name_width_max = 16;
        // dynamically set
        props.channel_index_width = (props.channels_max as f64).log10().ceil() as usize;
        props
    })
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

    //let mut count = 0;
    //let mut txt: Vec<T> = Vec::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            // exits the loop when reaching end of file
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"CNV_PER_CMP_TYPE" => {},
                    b"CP_CNVPERSALIAS" => {
                        let name = reader.read_text(QName(b"CP_CNVPERSALIAS"))?.into_owned();
                        uprintln!(opt, Stderr, Color::Green, None, "name = {:?}", name);
                    },
                    _ => {},
                }
            }
            // Ok(Event::End(e)) => {

            // }
            // Ok(Event::Text(e)) => txt.push(e.unescape().unwrap().into_owned()),

            // There are several other `Event`s we do not consider here
            _ => (),
        }
        buf.clear();
    }
    //println!("count: {}", count);

    Ok(codeplug)
}