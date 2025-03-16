// src/merge.rs

use crate::*;

fn merge_all(opt: &Opt, input_codeplug: &structures::Codeplug, target_codeplug: &mut structures::Codeplug) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    // merge channels
    // find the highest channel index in the target codeplug
    let mut max_channel_index: usize = 0;
    for channel in &target_codeplug.channels {
        if channel.index > max_channel_index {
            max_channel_index = channel.index;
        }
    }
    for channel in &input_codeplug.channels {
        // check if a channel with the same name already exists in the target codeplug
        if let Some(_) = target_codeplug.channels.iter_mut().rev().find(|c| c.name == channel.name) {
            uprintln!(opt, Stderr, Color::Yellow, None, "Channel already exists in codeplug, skipping: {:4} {}",
                channel.index, channel.name);
        } else {
            let mut new_channel = channel.clone();
            max_channel_index += 1;
            new_channel.index = max_channel_index;
            // add the channel to the target codeplug
            target_codeplug.channels.push(new_channel);
        }
    }
    // merge zones
    // find the highest zone index in the target codeplug
    let mut max_zone_index: usize = 0;
    for zone in &target_codeplug.zones {
        if zone.index > max_zone_index {
            max_zone_index = zone.index;
        }
    }
    for zone in &input_codeplug.zones {
        // check if a zone with the same name already exists in the target codeplug
        if let Some(_) = target_codeplug.zones.iter_mut().find(|z| z.name == zone.name) {
            uprintln!(opt, Stderr, Color::Yellow, None, "Zone already exists in codeplug, skipping: {:4} {}",
                zone.index, zone.name);
        } else {
            let mut new_zone = zone.clone();
            max_zone_index += 1;
            new_zone.index = max_zone_index;
            // add the zone to the target codeplug
            target_codeplug.zones.push(new_zone);
        }
    }
    // merge talkgroups
    // find the highest talkgroup index in the target codeplug
    let mut max_talkgroup_index: usize = 0;
    for talkgroup in &target_codeplug.talkgroups {
        if talkgroup.index > max_talkgroup_index {
            max_talkgroup_index = talkgroup.index;
        }
    }
    for talkgroup in &input_codeplug.talkgroups {
        // check if a talkgroup with the same name already exists in the target codeplug
        if let Some(_) = target_codeplug.talkgroups.iter_mut().find(|t| t.name == talkgroup.name) {
            uprintln!(opt, Stderr, Color::Yellow, None, "Talkgroup already exists in codeplug, skipping: {:4} {}",
                talkgroup.index, talkgroup.name);
        } else {
            let mut new_talkgroup = talkgroup.clone();
            max_talkgroup_index += 1;
            new_talkgroup.index = max_talkgroup_index;
            // add the talkgroup to the target codeplug
            target_codeplug.talkgroups.push(new_talkgroup);
        }
    }
    // merge talkgroup lists
    // find the highest talkgroup list index in the target codeplug
    let mut max_talkgroup_list_index: usize = 0;
    for talkgroup_list in &target_codeplug.talkgroup_lists {
        if talkgroup_list.index > max_talkgroup_list_index {
            max_talkgroup_list_index = talkgroup_list.index;
        }
    }
    for talkgroup_list in &input_codeplug.talkgroup_lists {
        // check if a talkgroup list with the same name already exists in the target codeplug
        if let Some(_) = target_codeplug.talkgroup_lists.iter_mut().find(|tl| tl.name == talkgroup_list.name) {
            uprintln!(opt, Stderr, Color::Yellow, None, "Talkgroup list already exists in codeplug, skipping: {:4} {}",
                talkgroup_list.index, talkgroup_list.name);
        } else {
            let mut new_talkgroup_list = talkgroup_list.clone();
            max_talkgroup_list_index += 1;
            new_talkgroup_list.index = max_talkgroup_list_index;
            // add the talkgroup list to the target codeplug
            target_codeplug.talkgroup_lists.push(new_talkgroup_list);
        }
    }
    Ok(())
}

pub fn merge_codeplug(opt: &Opt, inputs: &[PathBuf]) -> Result<structures::Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    let mut codeplug: structures::Codeplug = structures::Codeplug::default();
    // iterate through the inputs
    for input in inputs {
        uprintln!(opt, Stderr, None, None, "Merging: {}", input.display());
        // read the codeplug
        let input_codeplug = read_codeplug(opt, input)?;
        merge_all(&opt, &input_codeplug, &mut codeplug)?;
    }
    Ok(codeplug.clone())
}