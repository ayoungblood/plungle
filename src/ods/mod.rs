// src/csv.rs

use std::u32;

use spreadsheet_ods::{WorkBook, Sheet};
use icu_locid::locale;
use crate::*;
use serde_plain;

pub fn write(opt: &Opt, codeplug: &structures::Codeplug, path: &Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());
    let ods_path;
    if path.is_none() {
        return Err("No path provided, cannot write ODS to stdout".into());
    } else {
        ods_path = path.clone().unwrap().clone();
    }
    let mut workbook = WorkBook::new(locale!("en_US"));

    // create a talkgroups sheet
    let mut talkgroups_sheet = Sheet::new("DMR Talkgroups");
    // build the header
    talkgroups_sheet.set_value(0, 0, "id");
    talkgroups_sheet.set_value(0, 1, "name");
    talkgroups_sheet.set_value(0, 2, "call_type");
    talkgroups_sheet.set_value(0, 3, "alert");
    // write the talkgroups
    for (ii, talkgroup) in codeplug.talkgroups.iter().enumerate() {
        talkgroups_sheet.set_value(ii as u32 + 1, 0, talkgroup.id.to_string());
        talkgroups_sheet.set_value(ii as u32 + 1, 1, talkgroup.name.clone());
        talkgroups_sheet.set_value(ii as u32 + 1, 2, serde_plain::to_string(&talkgroup.call_type)?);
        talkgroups_sheet.set_value(ii as u32 + 1, 3, talkgroup.alert.clone());
    }
    workbook.push_sheet(talkgroups_sheet);

    // create a talkgroup lists sheet
    let mut talkgroup_lists_sheet = Sheet::new("DMR Talkgroup Lists");
    // build the header
    talkgroup_lists_sheet.set_value(0, 0, "name");
    talkgroup_lists_sheet.set_value(0, 1, "talkgroups");
    // write the talkgroup lists
    for (ii, talkgroup_list) in codeplug.talkgroup_lists.iter().enumerate() {
        talkgroup_lists_sheet.set_value(ii as u32 + 1, 0, talkgroup_list.name.clone());
        talkgroup_lists_sheet.set_value(ii as u32 + 1, 1, "foo");
    }
    workbook.push_sheet(talkgroup_lists_sheet);



    // Write the workbook to the specified path
    spreadsheet_ods::write_ods(&mut workbook, &ods_path).unwrap();
    uprintln!(opt, Stderr, THEME.info, None, "Workbook written to {}", ods_path.display());
    Ok(())
}
