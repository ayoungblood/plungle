// src/printer.rs

use std::error::Error;
use rust_decimal::prelude::ToPrimitive;
use crate::*;
use crate::structures::*;


fn pretty_tone(tone: &Option<Tone>) -> String {
    if let Some(tone) = tone {
        match tone.mode {
            ToneMode::CTCSS => {
                let freq = tone.ctcss.unwrap().to_f64().unwrap();
                return format!("{:5.1}", freq);
            },
            ToneMode::DCS   => {
                return format!("{}", tone.dcs.as_ref().unwrap());
            },
        }
    } else {
        return "none".to_string();
    }
}

fn pretty_tx_permit(tx_permit: &Option<TxPermit>) -> String {
    if let Some(tx_permit) = tx_permit {
        match tx_permit {
            TxPermit::Always => { return "always".to_string(); },
            TxPermit::ChannelFree => { return "chfree".to_string(); },
            TxPermit::CtcssDcsDifferent => { return "ctcss".to_string(); },
            TxPermit::ColorCodeSame => { return "ccsame".to_string(); },
            TxPermit::ColorCodeDifferent => { return "ccdiff".to_string(); },
        }
    } else {
        return "-".to_string();
    }
}

fn pretty_scan(scan: &Option<Scan>) -> String {
    if let Some(scan) = scan {
        return format!("{:2}{:2}",
            if scan.zone_skip { "ZS" } else { " " },
            if scan.all_skip { "AS" } else { " " },
        );
    } else {
        return "-".to_string();
    }
}

fn pretty_squelch(squelch: &Squelch) -> String {
    if squelch.default {
        return "def".to_string();
    }
    if let Some(percent) = squelch.percent {
        return format!("{:3}%", percent);
    } else {
        return "???".to_string();
    }
}

fn pretty_channel(_opt: &Opt, channel: &Channel) -> String {
    let mut line = String::new();
    // print common stuff
    line.push_str(&format!("    "));
    line.push_str(&format!("{:width$} ", channel.index, width = 4));
    line.push_str(&format!("{:width$} ", channel.name, width = 16));
    line.push_str(&format!("{}  ", channel.mode));
    line.push_str(&format!("{} ", freq2str(&channel.frequency_rx)));
    line.push_str(&format!("{} ", freq2str(&channel.frequency_tx)));
    line.push_str(&format!("{} ", if channel.rx_only { "RXO" } else { "   " }));
    line.push_str(&format!("{} ", channel.tx_tot));
    line.push_str(&format!("{} ", channel.power));
    line.push_str(&format!("{:7} ", pretty_tx_permit(&channel.tx_permit)));
    line.push_str(&format!("{:7} ", pretty_scan(&channel.scan)));
    // print mode specific stuff
    match channel.mode {
        ChannelMode::AM  => line.push_str(&format!(" ")),
        ChannelMode::FM  => line.push_str(&format!("{} {} {} {}",
            format!("bw={:4.1}k", &channel.fm.clone().unwrap().bandwidth.to_f64().unwrap()/1000.0),
            format!("sq={:>4}", pretty_squelch(&channel.fm.clone().unwrap().squelch)),
            format!("rx={:5}", pretty_tone(&channel.fm.clone().unwrap().tone_rx)),
            format!("tx={:5}", pretty_tone(&channel.fm.clone().unwrap().tone_tx)),
        )),
        ChannelMode::DMR => line.push_str(&format!("slot={:1} color={:2} tg={} tgl={} id={}",
            channel.dmr.clone().unwrap().timeslot,
            channel.dmr.clone().unwrap().color_code,
            if channel.dmr.clone().unwrap().talkgroup.is_some() {
                channel.dmr.clone().unwrap().talkgroup.as_ref().unwrap().clone()
            } else {
                "".to_string()
            },
            if channel.dmr.clone().unwrap().talkgroup_list.is_some() {
                channel.dmr.clone().unwrap().talkgroup_list.as_ref().unwrap().clone()
            } else {
                "".to_string()
            },
            if channel.dmr.clone().unwrap().id_name.is_some() {
                channel.dmr.clone().unwrap().id_name.as_ref().unwrap().clone()
            } else {
                "".to_string()
            },
        )),
    }
    line
}

fn print_channels(opt: &Opt, codeplug: &Codeplug) -> Result<String, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());

    let mut output = String::new();
    output.push_str(&format!("Channels\n"));
    output.push_str(&format!("    {:4} {:16} {:4} {:12} {:12} {:3} {:7} {:7} {:7} {:7}\n",
        "no", "name", "mode", "rxf", "txf", "rxo", "tot", "power", "txprmit", "scan"));
    for channel in &codeplug.channels {
        output.push_str(&pretty_channel(opt, channel));
        output.push('\n');
    }

    Ok(output)
}

fn print_zones(opt: &Opt, codeplug: &Codeplug) -> Result<String, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());

    let mut output = String::new();
    output.push_str(&format!("Zones\n"));
    output.push_str(&format!("    {:16} {:7} {}\n",
        "name", "len", "channels"));
    for zone in &codeplug.zones {
        output.push_str(&format!("    {:16} {:7} {}\n",
            zone.name,
            zone.channels.len(),
            zone.channels.iter().map(|ch| ch.to_string()).collect::<Vec<String>>().join(", "),
        ));
    }

    Ok(output)
}

fn print_talkgroups(opt: &Opt, codeplug: &Codeplug) -> Result<String, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());

    let mut output = String::new();
    output.push_str(&format!("Talkgroups\n"));
    output.push_str(&format!("    {:16} {:8} {:4}\n",
        "name", "id", "type"));
    for tg in &codeplug.talkgroups {
        output.push_str(&format!("    {:16} {:8} {:4}\n",
            tg.name,
            tg.id,
            match tg.call_type {
                DmrTalkgroupCallType::Group => "grp",
                DmrTalkgroupCallType::Private => "priv",
                DmrTalkgroupCallType::AllCall => "all",
            },
        ));
    }

    Ok(output)
}

fn print_talkgroup_lists(opt: &Opt, codeplug: &Codeplug) -> Result<String, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());

    let mut output = String::new();
    output.push_str(&format!("Talkgroup Lists\n"));
    output.push_str(&format!("    {:16} {:3} {}\n",
        "name", "len", "talkgroups"));
    for tg in &codeplug.talkgroup_lists {
        output.push_str(&format!("    {:16} {:3} {}\n",
            tg.name,
            tg.talkgroups.len(),
            tg.talkgroups.iter().map(|tg| tg.name.to_string()).collect::<Vec<String>>().join(", "),
        ));
    }

    Ok(output)
}

pub fn pretty(opt: &Opt, codeplug: &Codeplug) -> Result<String, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());

    let mut output = String::new();

    output.push_str(print_channels(opt, codeplug).unwrap().as_str());
    output.push_str(print_zones(opt, codeplug).unwrap().as_str());
    output.push_str(print_talkgroups(opt, codeplug).unwrap().as_str());
    output.push_str(print_talkgroup_lists(opt, codeplug).unwrap().as_str());
    output.push_str(&format!("Source: {}\n", codeplug.source));

    Ok(output)
}
