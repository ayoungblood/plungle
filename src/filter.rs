// src/filter.rs

use crate::*;

// Filters:
// Multiple filter arguments can be specified
// Each filter argument is a string, and can contain multiple space-separated parts
// Each part is a string, and must start with a prefix that indicates the type of filter:
// - "c:" for channel
// - "z:" for zone
// - "tg:" for talkgroup
// - "tgl:" for group
// Each part may have multiple criteria, separated by semicolons. These apply in the order they appear.
// Example:
// - "c:3-7;mode=DMR" would first select channels 3-7, and then select only DMR channels from those
// - "c:3-7,9-13 c:200-204" would select channels 3-7, 9-13, and 200-204

fn get_channel_by_index(codeplug: &structures::Codeplug, index: usize) -> Option<&structures::Channel> {
    for channel in &codeplug.channels {
        if channel.index == index {
            return Some(channel);
        }
    }
    None
}

fn get_zone_by_index(codeplug: &structures::Codeplug, index: usize) -> Option<&structures::Zone> {
    for zone in &codeplug.zones {
        if zone.index == index {
            return Some(zone);
        }
    }
    None
}

fn get_talkgroup_by_index(codeplug: &structures::Codeplug, index: usize) -> Option<&structures::DmrTalkgroup> {
    for talkgroup in &codeplug.talkgroups {
        if talkgroup.index == index {
            return Some(talkgroup);
        }
    }
    None
}

fn get_talkgroup_list_by_index(codeplug: &structures::Codeplug, index: usize) -> Option<&structures::DmrTalkgroupList> {
    for talkgroup_list in &codeplug.talkgroup_lists {
        if talkgroup_list.index == index {
            return Some(talkgroup_list);
        }
    }
    None
}

fn filter_channels(opt: &Opt, source_codeplug: &structures::Codeplug, dest_codeplug: &mut structures::Codeplug, part: &String) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}(): part=\"{}\"", file!(), function!(), part);
    // remove the prefix (split on ':' and discard the first part), then split on semicolons
    let mut criteria = part.split(':').nth(1).unwrap().split(';');
    // get the first criterion, this is basis for future filtering
    let first_criterion = criteria.next().unwrap();
    // if the first criterion is "*" or "all", select all channels
    if first_criterion == "*" || first_criterion == "all" {
        dest_codeplug.channels = source_codeplug.channels.clone();
    } else {
        // split on commas and parse as channel indices
        let mut indices: Vec<usize> = Vec::new();
        for part in first_criterion.split(',') {
            // if the part is a range, parse it as a range of indices
            if part.contains('-') {
                let range: Vec<usize> = part.split('-').map(|s| s.parse().unwrap()).collect();
                if range[0] > range[1] {
                    return Err(format!("Invalid range: {}-{}", range[0], range[1]).into());
                }
                indices.extend(range[0]..=range[1]);
            } else {
                // otherwise, parse it as a single index
                indices.push(part.parse().unwrap());
            }
        }
        // sort and deduplicate the indices
        indices.sort();
        indices.dedup();
        // grab the channels from the source codeplug
        for index in indices {
            // get channel by index, add it if it exists
            if let Some(channel) = get_channel_by_index(source_codeplug, index) {
                dest_codeplug.channels.push(channel.clone());
            }
        }
    }
    // iterate through the rest of the criteria
    for criterion in criteria {
        uprintln!(opt, Stderr, None, 2, "    criterion: \"{}\"", criterion);
        // Apply the criterion to filter the channels
        // This is a placeholder for the actual filtering logic
    }
    Ok(())
}

fn filter_zones(opt: &Opt, source_codeplug: &structures::Codeplug, dest_codeplug: &mut structures::Codeplug, part: &String) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}(): part=\"{}\"", file!(), function!(), part);
    // remove the prefix (split on ':' and discard the first part), then split on semicolons
    let mut criteria = part.split(':').nth(1).unwrap().split(';');
    // get the first criterion, this is basis for future filtering
    let first_criterion = criteria.next().unwrap();
    // if the first criterion is "*" or "all", select all zones
    if first_criterion == "*" || first_criterion == "all" {
        dest_codeplug.zones = source_codeplug.zones.clone();
    } else if first_criterion == "auto" {
        // if the first criterion is "auto", add all zones that have at least one channel in the destination codeplug
        for zone in &source_codeplug.zones {
            // check if the zone has any channels in the destination codeplug
            let mut has_channel = false;
            for channel_name in &zone.channels {
                if dest_codeplug.channels.iter().any(|c| c.name == *channel_name) {
                    has_channel = true;
                    break;
                }
            }
            // if the zone has at least one channel, add it to the destination codeplug
            if has_channel {
                dest_codeplug.zones.push(zone.clone());
            }
        }
    } else {
        // split on commas and parse as zone indices
        let mut indices: Vec<usize> = Vec::new();
        for part in first_criterion.split(',') {
            // if the part is a range, parse it as a range of indices
            if part.contains('-') {
                let range: Vec<usize> = part.split('-').map(|s| s.parse().unwrap()).collect();
                if range[0] > range[1] {
                    return Err(format!("Invalid range: {}-{}", range[0], range[1]).into());
                }
                indices.extend(range[0]..=range[1]);
            } else {
                // otherwise, parse it as a single index
                indices.push(part.parse().unwrap());
            }
        }
        // sort and deduplicate the indices
        indices.sort();
        indices.dedup();
        // grab the zones from the source codeplug
        for index in indices {
            // get zone by index, add it if it exists
            if let Some(zone) = get_zone_by_index(source_codeplug, index) {
                dest_codeplug.zones.push(zone.clone());
            }
        }
    }
    // iterate through the rest of the criteria
    for criterion in criteria {
        uprintln!(opt, Stderr, None, 2, "    criterion: \"{}\"", criterion);
        // Apply the criterion to filter the zones
        // This is a placeholder for the actual filtering logic
    }
    Ok(())
}

fn filter_talkgroups(opt: &Opt, source_codeplug: &structures::Codeplug, dest_codeplug: &mut structures::Codeplug, part: &String) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}(): part=\"{}\"", file!(), function!(), part);
    // remove the prefix (split on ':' and discard the first part), then split on semicolons
    let mut criteria = part.split(':').nth(1).unwrap().split(';');
    // get the first criterion, this is basis for future filtering
    let first_criterion = criteria.next().unwrap();
    // if the first criterion is "*" or "all", select all talkgroups
    if first_criterion == "*" || first_criterion == "all" {
        dest_codeplug.talkgroups = source_codeplug.talkgroups.clone();
    } else if first_criterion == "auto" {
        // if the first criterion is "auto", add all talkgroups that have at least one channel in the destination codeplug
        for talkgroup in &source_codeplug.talkgroups {
            // check if any channels in the destination codeplug use this talkgroup
            let mut uses_talkgroup = false;
            for channel in &dest_codeplug.channels {
                if let Some(dmr) = &channel.dmr {
                    if let Some(tg_name) = &dmr.talkgroup {
                        if *tg_name == talkgroup.name {
                            uses_talkgroup = true;
                            break;
                        }
                    }
                }
            }
            // if a channel uses the talkgroup, add it to the destination codeplug
            if uses_talkgroup {
                dest_codeplug.talkgroups.push(talkgroup.clone());
            }
        }
    } else {
        // split on commas and parse as talkgroup IDs
        let mut indices: Vec<u32> = Vec::new();
        for part in first_criterion.split(',') {
            // if the part is a range, parse it as a range of indicies
            if part.contains('-') {
                let range: Vec<u32> = part.split('-').map(|s| s.parse().unwrap()).collect();
                if range[0] > range[1] {
                    return Err(format!("Invalid range: {}-{}", range[0], range[1]).into());
                }
                indices.extend(range[0]..=range[1]);
            } else {
                // otherwise, parse it as a single ID
                indices.push(part.parse().unwrap());
            }
        }
        // sort and deduplicate the indices
        indices.sort();
        indices.dedup();
        // grab the talkgroups from the source codeplug
        for index in indices {
            // get talkgroup by ID, add it if it exists
            if let Some(talkgroup) = get_talkgroup_by_index(source_codeplug, index as usize) {
                dest_codeplug.talkgroups.push(talkgroup.clone());
            }
        }
    }
    // iterate through the rest of the criteria
    for criterion in criteria {
        uprintln!(opt, Stderr, None, 2, "    criterion: \"{}\"", criterion);
        // Apply the criterion to filter the talkgroups
        // This is a placeholder for the actual filtering logic
    }
    Ok(())
}

fn filter_talkgroup_lists(opt: &Opt, source_codeplug: &structures::Codeplug, dest_codeplug: &mut structures::Codeplug, part: &String) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}(): part=\"{}\"", file!(), function!(), part);
    // remove the prefix (split on ':' and discard the first part), then split on semicolons
    let mut criteria = part.split(':').nth(1).unwrap().split(';');
    // get the first criterion, this is basis for future filtering
    let first_criterion = criteria.next().unwrap();
    // if the first criterion is "*" or "all", select all talkgroup lists
    if first_criterion == "*" || first_criterion == "all" {
        dest_codeplug.talkgroup_lists = source_codeplug.talkgroup_lists.clone();
    } else if first_criterion == "auto" {
        // if the first criterion is "auto", add all talkgroup lists that have at least one talkgroup in the destination codeplug
        for talkgroup_list in &source_codeplug.talkgroup_lists {
            // check if any channel in the destination codeplug uses this talkgroup list
            let mut uses_talkgroup_list = false;
            for channel in &dest_codeplug.channels {
                // if the channel is dmr
                if let Some(dmr) = &channel.dmr {
                    // check if the talkgroup list is in the channel's rx group list
                    if let Some(tg_list_name) = &dmr.talkgroup_list {
                        if *tg_list_name == talkgroup_list.name {
                            uses_talkgroup_list = true;
                            break;
                        }
                    }
                }
            }
            // if a channel uses the talkgroup list, add it to the destination codeplug
            if uses_talkgroup_list {
                dest_codeplug.talkgroup_lists.push(talkgroup_list.clone());
                // then make sure all the talkgroups in the talkgroup list are in the destination codeplug
                for talkgroup in &talkgroup_list.talkgroups {
                    // if the talkgroup is not in the destination codeplug, add it
                    if !dest_codeplug.talkgroups.iter().any(|tg| tg.name == talkgroup.name) {
                        dest_codeplug.talkgroups.push(talkgroup.clone());
                    }
                }
            }
        }
    } else {
        // split on commas and parse as talkgroup list indices
        let mut indices: Vec<usize> = Vec::new();
        for part in first_criterion.split(',') {
            // if the part is a range, parse it as a range of indices
            if part.contains('-') {
                let range: Vec<usize> = part.split('-').map(|s| s.parse().unwrap()).collect();
                if range[0] > range[1] {
                    return Err(format!("Invalid range: {}-{}", range[0], range[1]).into());
                }
                indices.extend(range[0]..=range[1]);
            } else {
                // otherwise, parse it as a single index
                indices.push(part.parse().unwrap());
            }
        }
        // sort and deduplicate the indices
        indices.sort();
        indices.dedup();
        // grab the talkgroup lists from the source codeplug
        for index in indices {
            // get talkgroup list by index, add it if it exists
            if let Some(talkgroup_list) = get_talkgroup_list_by_index(source_codeplug, index) {
                dest_codeplug.talkgroup_lists.push(talkgroup_list.clone());
            }
        }
    }
    // iterate through the rest of the criteria
    for criterion in criteria {
        uprintln!(opt, Stderr, None, 2, "    criterion: \"{}\"", criterion);
        // Apply the criterion to filter the talkgroup lists
        // This is a placeholder for the actual filtering logic
    }
    Ok(())
}

pub fn filter_codeplug(opt: &Opt, codeplug: &structures::Codeplug, filter_strings: &Option<Vec<String>>) -> Result<structures::Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    match filter_strings {
        Some(filters) => {
            uprintln!(opt, Stderr, Color::Cyan, None, "--filter: {:4} channels, {:3} zones, {:3} talkgroups, {:3} talkgroup lists before filtering",
            codeplug.channels.len(), codeplug.zones.len(), codeplug.talkgroups.len(), codeplug.talkgroup_lists.len());
            let mut filtered_codeplug = structures::Codeplug::default();
            for filter in filters {
                // split each filter on spaces
                let filter_parts = filter.split_whitespace();
                for part in filter_parts {
                    if part == "auto" {
                        filter_zones(opt, codeplug, &mut filtered_codeplug, &"z:auto".to_string())?;
                        filter_talkgroups(opt, codeplug, &mut filtered_codeplug, &"tg:auto".to_string())?;
                        filter_talkgroup_lists(opt, codeplug, &mut filtered_codeplug, &"tgl:auto".to_string())?;
                    } else if part.starts_with("c:") {
                        filter_channels(opt, codeplug, &mut filtered_codeplug, &part.to_string())?;
                    } else if part.starts_with("z:") {
                        filter_zones(opt, codeplug, &mut filtered_codeplug, &part.to_string())?;
                    } else if part.starts_with("tg:") {
                        filter_talkgroups(opt, codeplug, &mut filtered_codeplug, &part.to_string())?;
                    } else if part.starts_with("tgl:") {
                        filter_talkgroup_lists(opt, codeplug, &mut filtered_codeplug, &part.to_string())?;
                    } else {
                        uprintln!(opt, Stderr, Color::Red, None, "--filter: Invalid filter part: \"{}\"", part);
                    }
                }
            }
            uprintln!(opt, Stderr, Color::Cyan, None, "--filter: {:4} channels, {:3} zones, {:3} talkgroups, {:3} talkgroup lists after filtering",
                filtered_codeplug.channels.len(), filtered_codeplug.zones.len(), filtered_codeplug.talkgroups.len(), filtered_codeplug.talkgroup_lists.len());
            return Ok(filtered_codeplug);
        }
        None => {
            return Ok(codeplug.clone());
        }
    }
}
