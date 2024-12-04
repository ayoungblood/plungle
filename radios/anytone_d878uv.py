import os, sys
import csv
import json

from radios.radio_helper import RadioHelper

RADIO_NAME = "AnyTone D878UV"
SUPPORTED_MODES = ["FM", "DMR"]

# CSV Export Format:
# Channel.CSV
# - No.: Channel Index
# - Channel Name: 16 characters?
# - Receive Frequency: frequency in MHz
# - Transmit Frequency: frequency in MHz
# - Channel Type: [A-Analog, D-Digital]
# - Transmit Power: [Turbo, High, Med, Low], corresponding to ~7W, 5W, 2.5W, 1W
# - Band Width: [12.5K, 25K]
# - CTCSS/DCS Decode: Off, or CTCSS/DCS frequency/code
# - CTCSS/DCS Encode: Off, or CTCSS/DCS frequency/code
# - Contact: DMR contact
# - Contact Call Type: [Group Call, ???]
# - Contact TG/DMR ID: DMR talkgroup ID
# - Radio ID: Radio ID name (not DMR ID), generally callsign
# - Busy Lock/TX Permit: [Off, Always, ???]
# - Squelch Mode: [Carrier, CTCSS/DCS], Carrier for digital channels
# - Optional Signal: Off
# - DTMF ID: 1
# - 2Tone ID: 1
# - 5Tone ID: 1
# - PTT ID: Off
# - Color Code: DMR color code, 0-15
# - Slot: DMR timeslot, [1, 2]
# - Scan List: None or Scan List name
# - Receive Group List: None or RX Group List name
# - PTT Prohibit: [Off, On]
# - Reverse: [Off, On]
# - Simplex TDMA: [Off, ??]
# - Slot Suit: [Off, ??]
# - AES Digital Encryption: Normal Encryption
# - Digital Encryption Type: [Off, ???]
# - Call Confirmation: [Off, ???]
# - Talk Around(Simplex): [Off, ???]
# - Work Alone: [Off, ???]
# - Custom CTCSS: 251.1 or custom frequency
# - 2TONE Decode: 0
# - Ranging: [Off, ???]
# - Through Mode: [Off, ???]
# - APRS RX: [Off, ???]
# - Analog APRS PTT Mode: [Off, ???]
# - Digital APRS PTT Mode: [Off, ???]
# - APRS Report Type: [Off, ???]
# - Digital APRS Report Channel: 1
# - Correct Frequency[Hz]: 0
# - SMS Confirmation: [Off, ???]
# - Exclude channel from roaming: [0, 1]
# - DMR MODE: 0
# - DataACK Disable: 0
# - R5toneBot: 0
# - R5ToneEot: 0
# - Auto Scan: 0
# - Ana Aprs Mute: 0
# - Send Talker Alias: 0
# - AnaAprsTxPath: 0
# - ARC4: 0
# - ex_emg_kind: 0
# TalkGroups.CSV
# - No.: DMR talkgroup index
# - Radio ID: DMR talkgroup ID
# - Name: DMR talkgroup name (@TODO length??)
# - Call Type: [Group Call, ???]
# - Call Alert: [None, ???]
# ReceiveGroupCallList.CSV
# - No.: talkgroup list index
# - Group Name: DMR talkgroup list name
# - Contact: list of DMR talkgroup names, "|" separated
# - Contact TG/DMR ID: list of DMR talkgroup IDs, "|" separated
# ScanList.CSV
# - No.: scan list index
# - Scan List Name: scan list name
# - Scan Channel Member: list of channel names, "|" separated
# - Scan Channel Member RX Frequency: list of channel RX frequencies in MHz, "|" separated
# - Scan Channel Member TX Frequency: list of channel TX frequencies in MHz, "|" separated
# - Scan Mode: [Off, ???]
# - Priority Channel Select: [Off, ???]
# - Priority Channel 1: [Off, ???]
# - Priority Channel 1 RX Frequency: [blank, ???]
# - Priority Channel 1 TX Frequency: [blank, ???]
# - Priority Channel 2: [Off, ???]
# - Priority Channel 2 RX Frequency: [blank, ???]
# - Priority Channel 2 TX Frequency: [blank, ???]
# - Revert Channel: [Selected, ???]
# - Look Back Time A[s]: default 2
# - Look Back Time B[s]: default 3
# - Dropout Delay Time[s]: default 3.1
# - Priority Sample Time[s]: default 3.1
# Zone.CSV
# - No.: zone index
# - Zone Name: zone name
# - Zone Channel Member: list of channel names, "|" separated
# - Zone Channel Member RX Frequency: list of channel RX frequencies in MHz, "|" separated
# - Zone Channel Member TX Frequency: list of channel TX frequencies in MHz, "|" separated
# - A Channel: name of selected channel in zone
# - A Channel RX Frequency: RX frequency in MHz of selected channel in zone
# - A Channel TX Frequency: TX frequency in MHz of selected channel in zone
# - B Channel: name of selected channel in zone
# - B Channel RX Frequency: RX frequency in MHz of selected channel in zone
# - B Channel TX Frequency: TX frequency in MHz of selected channel in zone
# - Zone Hide: [0, ???]


def get_channel_by_name(channels, name):
    for channel in channels:
        if channel["name"] == name:
            return channel
    return None

def export_power(power_mw):
    if 6000 < power_mw <= 7000:
        return "Turbo"
    if 3000 < power_mw <= 6000:
        return "High"
    if 1500 < power_mw <= 3000:
        return "Med"
    if 400 < power_mw <= 1500:
        return "Low"
    print(f"Warning: Unable to match power level: {power_mw}") # @TODO log this better
    return "Low"

def export_tone(tone):
    if not tone:
        return "Off"
    if tone["type"] == "CTCSS":
        return f"{tone['freq']}"
    if tone["type"] == "DCS":
        return f"{tone['code']}"

def json2csv(json_path, output_path):
    if not os.path.exists(json_path):
        print(f"Error: {json_path} does not exist.")
        return
    if not json_path.endswith(".json"):
        print(f"Error: {json_path} is not a JSON file.")
        return
    with open(json_path, 'r') as json_file:
        json_data = json.load(json_file)

    if not os.path.exists(output_path):
        os.makedirs(output_path)
    else:
        print(f"Error: {output_path} already exists.")

    # Export talkgroups to TalkGroups.csv
    talkgroups_file = os.path.join(output_path, "TalkGroups.CSV")
    print(f"Exporting to {talkgroups_file}")
    with open(talkgroups_file, 'w', newline='') as csv_file:
        fieldnames = ["No.", "Radio ID", "Name", "Call Type", "Call Alert"]
        writer = csv.DictWriter(csv_file, fieldnames=fieldnames)
        writer.writeheader()
        for ii, talkgroup in enumerate(json_data["talkgroups"], start=1):
            row = {
                "No.": ii,
                "Radio ID": talkgroup["id"],
                "Name": talkgroup["name"],
                "Call Type": "Group Call",
                "Call Alert": "None",
            }
            writer.writerow(row)
    print(f"Exported to {talkgroups_file}")

    # Export talkgroup lists to ReceiveGroupCallList.csv
    talkgroup_lists_file = os.path.join(output_path, "ReceiveGroupCallList.CSV")
    print(f"Exporting to {talkgroup_lists_file}")
    with open(talkgroup_lists_file, 'w', newline='') as csv_file:
        fieldnames = ["No.", "Group Name", "Contact", "Contact TG/DMR ID"]
        writer = csv.DictWriter(csv_file, fieldnames=fieldnames)
        writer.writeheader()
        for ii, talkgroup_list in enumerate(json_data["talkgroup_lists"], start=1):
            row = {
                "No.": ii,
                "Group Name": talkgroup_list["name"],
                "Contact": "|".join([tgn for tgn in talkgroup_list["talkgroups"]]),
                "Contact TG/DMR ID": "|".join([str(tg["id"]) for tgn in talkgroup_list["talkgroups"] for tg in json_data["talkgroups"] if tg["name"] == tgn]),
            }

            writer.writerow(row)

    # Export zones to ScanList.CSV
    scanlist_file = os.path.join(output_path, "ScanList.CSV")
    print(f"Exporting to {scanlist_file}")
    with open(scanlist_file, 'w', newline='') as csv_file:
        fieldnames = [
            "No.", "Scan List Name", "Scan Channel Member", "Scan Channel Member RX Frequency",
            "Scan Channel Member TX Frequency", "Scan Mode", "Priority Channel Select",
            "Priority Channel 1", "Priority Channel 1 RX Frequency", "Priority Channel 1 TX Frequency",
            "Priority Channel 2", "Priority Channel 2 RX Frequency", "Priority Channel 2 TX Frequency",
            "Revert Channel", "Look Back Time A[s]", "Look Back Time B[s]", "Dropout Delay Time[s]",
            "Priority Sample Time[s]"
        ]
        writer = csv.DictWriter(csv_file, fieldnames=fieldnames)
        writer.writeheader()
        for ii, zone in enumerate(json_data["zones"], start=1):
            row = {
                "No.": ii,
                "Scan List Name": zone["name"],
                "Scan Channel Member": "|".join([ch for ch in zone["channels"]]),
                "Scan Channel Member RX Frequency": "|".join([str(get_channel_by_name(json_data["channels"], ch)["freq_rx"] / 1_000_000) for ch in zone["channels"]]),
                "Scan Channel Member TX Frequency": "|".join([str(get_channel_by_name(json_data["channels"], ch)["freq_tx"] / 1_000_000) for ch in zone["channels"]]),
                "Scan Mode": "Off",
                "Priority Channel Select": "Off",
                "Priority Channel 1": "Off",
                "Priority Channel 1 RX Frequency": "",
                "Priority Channel 1 TX Frequency": "",
                "Priority Channel 2": "Off",
                "Priority Channel 2 RX Frequency": "",
                "Priority Channel 2 TX Frequency": "",
                "Revert Channel": "Selected",
                "Look Back Time A[s]": 2,
                "Look Back Time B[s]": 3,
                "Dropout Delay Time[s]": 3.1,
                "Priority Sample Time[s]": 3.1,
            }
            writer.writerow(row)

    # Export zones to Zone.CSV
    zones_file = os.path.join(output_path, "Zone.CSV")
    print(f"Exporting to {zones_file}")
    with open(zones_file, 'w', newline='') as csv_file:
        fieldnames = [
            "No.", "Zone Name", "Zone Channel Member", "Zone Channel Member RX Frequency",
            "Zone Channel Member TX Frequency", "A Channel", "A Channel RX Frequency",
            "A Channel TX Frequency", "B Channel", "B Channel RX Frequency", "B Channel TX Frequency",
            "Zone Hide"
        ]
        writer = csv.DictWriter(csv_file, fieldnames=fieldnames)
        writer.writeheader()
        for ii, zone in enumerate(json_data["zones"], start=1):
            row = {
                "No.": ii,
                "Zone Name": zone["name"],
                "Zone Channel Member": "|".join([ch for ch in zone["channels"]]),
                "Zone Channel Member RX Frequency": "|".join([str(get_channel_by_name(json_data["channels"], ch)["freq_rx"] / 1_000_000) for ch in zone["channels"]]),
                "Zone Channel Member TX Frequency": "|".join([str(get_channel_by_name(json_data["channels"], ch)["freq_tx"] / 1_000_000) for ch in zone["channels"]]),
                "A Channel": zone["channels"][0],
                "A Channel RX Frequency": str(get_channel_by_name(json_data["channels"], zone["channels"][0])["freq_rx"] / 1_000_000),
                "A Channel TX Frequency": str(get_channel_by_name(json_data["channels"], zone["channels"][0])["freq_tx"] / 1_000_000),
                "B Channel": zone["channels"][1] if len(zone["channels"]) > 1 else zone["channels"][0],
                "B Channel RX Frequency": str(get_channel_by_name(json_data["channels"], zone["channels"][1] if len(zone["channels"]) > 1 else zone["channels"][0])["freq_rx"] / 1_000_000),
                "B Channel TX Frequency": str(get_channel_by_name(json_data["channels"], zone["channels"][1] if len(zone["channels"]) > 1 else zone["channels"][0])["freq_tx"] / 1_000_000),
                "Zone Hide": 0,
            }
            writer.writerow(row)

    # Export channels to Channel.csv
    channels_file = os.path.join(output_path, "Channel.CSV")
    print(f"Exporting to {channels_file}")
    with open(channels_file, 'w', newline='') as csv_file:
        fieldnames = [
            "No.", "Channel Name", "Receive Frequency", "Transmit Frequency", "Channel Type",
            "Transmit Power", "Band Width", "CTCSS/DCS Decode", "CTCSS/DCS Encode", "Contact",
            "Contact Call Type", "Contact TG/DMR ID", "Radio ID", "Busy Lock/TX Permit", "Squelch Mode",
            "Optional Signal", "DTMF ID", "2Tone ID", "5Tone ID", "PTT ID", "Color Code", "Slot",
            "Scan List", "Receive Group List", "PTT Prohibit", "Reverse", "Simplex TDMA", "Slot Suit",
            "AES Digital Encryption", "Digital Encryption Type", "Call Confirmation", "Talk Around(Simplex)",
            "Work Alone", "Custom CTCSS", "2TONE Decode", "Ranging", "Through Mode", "APRS RX",
            "Analog APRS PTT Mode", "Digital APRS PTT Mode", "APRS Report Type", "Digital APRS Report Channel",
            "Correct Frequency[Hz]", "SMS Confirmation", "Exclude channel from roaming", "DMR MODE",
            "DataACK Disable", "R5toneBot", "R5ToneEot", "Auto Scan", "Ana Aprs Mute", "Send Talker Alias",
            "AnaAprsTxPath", "ARC4", "ex_emg_kind"
        ]
        writer = csv.DictWriter(csv_file, fieldnames=fieldnames)
        writer.writeheader()
        for channel in json_data["channels"]:
            if channel["mode"] not in SUPPORTED_MODES:
                print(f"Warning: Skipping unsupported mode: {channel['mode']}")
                continue
            row = {
                "No.": channel["index"],
                "Channel Name": channel["name"],
                "Receive Frequency": channel["freq_rx"] / 1_000_000,  # Convert Hz to MHz
                "Transmit Frequency": channel["freq_tx"] / 1_000_000,  # Convert Hz to MHz
                "Channel Type": "D-Digital" if channel["mode"] == "DMR" else "A-Analog",
                "Transmit Power": export_power(channel["power_mw"]),
                "Band Width": "25K" if channel["mode"] == "DMR" else ("25K" if channel["analog"]["bandwidth_hz"] == 25_000 else "12.5K"),
                "CTCSS/DCS Decode": export_tone(channel["analog"]["tone_rx"]) if channel["mode"] == "FM" else "Off",
                "CTCSS/DCS Encode": export_tone(channel["analog"]["tone_tx"]) if channel["mode"] == "FM" else "Off",
                "Contact": channel["dmr"]["contact"] if channel["mode"] == "DMR" else "0 None",
                "Contact Call Type": "Group Call" if channel["mode"] == "DMR" else "Group Call",
                "Contact TG/DMR ID": channel["dmr"]["tg_list"] if channel["mode"] == "DMR" else "0",
                "Radio ID": "KF0QMP", # @TODO FIXME
                "Busy Lock/TX Permit": "Off",
                "Squelch Mode": "Carrier",
                "Optional Signal": "Off",
                "DTMF ID": 1,
                "2Tone ID": 1,
                "5Tone ID": 1,
                "PTT ID": "Off",
                "Color Code": channel["dmr"]["color_code"] if channel["mode"] == "DMR" else "0",
                "Slot": channel["dmr"]["timeslot"] if channel["mode"] == "DMR" else "1",
                "Scan List": "None",
                "Receive Group List": "None",
                "PTT Prohibit": "Off",
                "Reverse": "Off",
                "Simplex TDMA": "Off",
                "Slot Suit": "Off",
                "AES Digital Encryption": "Normal Encryption",
                "Digital Encryption Type": "Off",
                "Call Confirmation": "Off",
                "Talk Around(Simplex)": "Off",
                "Work Alone": "Off",
                "Custom CTCSS": 251.1,
                "2TONE Decode": 0,
                "Ranging": "Off",
                "Through Mode": "Off",
                "APRS RX": "Off",
                "Analog APRS PTT Mode": "Off",
                "Digital APRS PTT Mode": "Off",
                "APRS Report Type": "Off",
                "Digital APRS Report Channel": 1,
                "Correct Frequency[Hz]": 0,
                "SMS Confirmation": "Off",
                "Exclude channel from roaming": 0,
                "DMR MODE": 0,
                "DataACK Disable": 0,
                "R5toneBot": 0,
                "R5ToneEot": 0,
                "Auto Scan": 0,
                "Ana Aprs Mute": 0,
                "Send Talker Alias": 0,
                "AnaAprsTxPath": 0,
                "ARC4": 0,
                "ex_emg_kind": 0,
            }
            writer.writerow(row)
    print(f"Exported to {channels_file}")

    # Write out a filelist (output.LST) for the AnyTone CPS
    filelist_file = os.path.join(output_path, 'output.LST')
    print(f'Writing filelist to {filelist_file}')
    with open(filelist_file, 'w') as lst_file:
        lst_file.write('5\n')
        lst_file.write('0,"Channel.CSV"\n')
        lst_file.write('1,"ReceiveGroupCallList.CSV"\n')
        lst_file.write('2,"ScanList.CSV"\n')
        lst_file.write('3,"TalkGroups.CSV"\n')
        lst_file.write('4,"Zone.CSV"\n')
