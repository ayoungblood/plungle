import os, sys
import csv
import json

from radios.radio_helper import RadioHelper

RADIO_NAME = "Retevis RT3S (OpenGD77)"

# CSV Export Format:
# Channels.csv
# - Channel Number
# - Channel Name: 15 characters
# - Channel Type: [Analogue, Digital]
# - Rx Frequency: frequency in MHz, 5 decimal places
# - Tx Frequency: frequency in MHz, 5 decimal places
# - Bandwidth (kHz): [12.5, 25], blank for digital channels
# - Colour Code: [0-15], blank for analog channels
# - Timeslot: [1, 2], blank for analog channels
# - Contact: DMR contact, blank for analog channels, set for static TS, None for dynamic TS
# - TG List: DMR talkgroup list, blank for analog channels, set for dynamic TS, None for static TS
# - DMR ID: DMR ID, blank for analog channels, generally can be left on None
# - TS1_TA_Tx: ???, "Off" for digital channels, blank for analog channels
# - TS2_TA_Tx ID: ???, "Off" for digital channels, blank for analog channels
# - RX Tone: None, or CTCSS frequency in Hz, blank for digital channels
# - TX Tone: None, or CTCSS frequency in Hz, blank for digital channels
# - Squelch: blank for digital channels, None, Disabled, Open, 5%-95%, or Closed for analog channels
# - Power: P1-P9, -W+, or Master
# - Rx Only: [Yes, No]
# - Zone Skip: skip channel in zone scan, [Yes, No]
# - All Skip: skip channel in all scan, [Yes, No]
# - TOT: custom TOT, generally 0
# - VOX: [Off, ???]
# - No Beep:
# - No Eco:
# - APRS:
# - Latitude:
# - Longitude:
# - Use location:

def parse_power_mw(power_str):
    if power_str == "Master":
        return 5000
    if power_str == "P1":
        return 50
    if power_str == "P2":
        return 250
    if power_str == "P3":
        return 500
    if power_str == "P4":
        return 750
    if power_str == "P5":
        return 1000
    if power_str == "P6":
        return 2000
    if power_str == "P7":
        return 3000
    if power_str == "P8":
        return 4000
    if power_str == "P9":
        return 5000
    if power_str == "-W+": # 5W+, map to 5W
        return 5000
    print(f"Warning: Unknown power level: {power_str}") # @TODO log this better

def parse_tone(tone_str):
    if tone_str == "None":
        return None
    if tone_str.startswith("D"): # DCS
        return {"type": "DCS", "code": tone_str}
    if RadioHelper.is_float_str(tone_str): # CTCSS
        return {"type": "CTCSS", "freq": float(tone_str)}
    print(f"Warning: Unknown tone type: {tone_str}") # @TODO log this better

def parse_squelch(squelch_str):
    if squelch_str == "Disabled":
        return "Default"
    if "%" in squelch_str:
        return squelch_str
    if squelch_str == "Open":
        return "0%"
    if squelch_str == "Closed":
        return "100%"

def csv2json(input_path):
    # Check that we have a directory, and print the files in the directory
    if not os.path.isdir(input_path):
        print(f"Error: {input_path} is not a directory.")
    else:
        print(f"Files in {input_path}:")
        for filename in os.listdir(input_path):
            print(f"  {filename}")
    print(f"Converting {RADIO_NAME} CSV export to JSON...")

    json_data = {}

    # Read channels

    file_path = os.path.join(input_path, 'Channels.csv')
    if not os.path.isfile(file_path):
        print(f"Error: {file_path} does not exist.")
        print(f"Please export the codeplug and try again.")
        return
    json_data["channels"] = []
    with open(file_path, 'r') as csv_file:
        csv_reader = csv.DictReader(csv_file)
        for row in csv_reader:
            channel = {
                "index": int(row["Channel Number"]),
                "name": row["Channel Name"],
                "mode": "DMR" if row["Channel Type"] == "Digital" else "FM",
                "freq_rx": RadioHelper.mhzStr_to_hzInt(row["Rx Frequency"]),
                "freq_tx": RadioHelper.mhzStr_to_hzInt(row["Tx Frequency"]),
                "rx_only": row["Rx Only"] == "Yes",
                "power_mw": parse_power_mw(row["Power"]),
            }
            if channel["mode"] == "FM":
                channel["analog"] = {
                    "bandwidth_hz": RadioHelper.khzStr_to_hzInt(row["Bandwidth (kHz)"]),
                    "squelch": parse_squelch(row["Squelch"]),
                    "rx_tone": parse_tone(row["RX Tone"]),
                    "tx_tone": parse_tone(row["TX Tone"]),
                }
            if channel["mode"] == "DMR":
                channel["dmr"] = {
                    "timeslot": int(row["Timeslot"]),
                    "color_code": int(row["Colour Code"]),
                    "contact": None if row["Contact"] == "None" else row["Contact"],
                    "tg_list": None if row["TG List"] == "None" else row["TG List"],
                }

            json_data["channels"].append(channel)
    print(f"Parsed {len(json_data['channels']):4d} channels...")

    # Read zones
    file_path = os.path.join(input_path, 'Zones.csv')
    if not os.path.isfile(file_path):
        print(f"Error: {file_path} does not exist.")
        print(f"Please export the codeplug and try again.")
        return
    json_data["zones"] = []
    with open(file_path, 'r') as csv_file:
        csv_reader = csv.DictReader(csv_file)
        for row in csv_reader:
            zone = {
                "name": row["Zone Name"],
                "channels": [],
            }
            for ii in range(1, 80): # up to 80 channels per zone
                if row[f"Channel{ii}"]:
                    zone["channels"].append(row[f"Channel{ii}"])
            json_data["zones"].append(zone)
    print(f"Parsed {len(json_data['zones']):4d} zones...")

    # Read talkgroups
    file_path = os.path.join(input_path, 'Contacts.csv')
    if not os.path.isfile(file_path):
        print(f"Error: {file_path} does not exist.")
        print(f"Please export the codeplug and try again.")
        return
    json_data["talkgroups"] = []
    with open(file_path, 'r') as csv_file:
        csv_reader = csv.DictReader(csv_file)
        for row in csv_reader:
            contact = {
                "id": int(row["ID"]),
                "name": row["Contact Name"],
            }
            json_data["talkgroups"].append(contact)
    print(f"Parsed {len(json_data['talkgroups']):4d} talkgroups...")

    # Read talkgroup lists
    file_path = os.path.join(input_path, 'TG_Lists.csv')
    if not os.path.isfile(file_path):
        print(f"Error: {file_path} does not exist.")
        print(f"Please export the codeplug and try again.")
        return
    json_data["talkgroup_lists"] = []
    with open(file_path, 'r') as csv_file:
        csv_reader = csv.DictReader(csv_file)
        for row in csv_reader:
            tg_list = {
                "name": row["TG List Name"],
                "tgs": [],
            }
            tgs = []
            for ii in range(1, 32): # up to 32 contacts per talkgroup list
                if row[f"Contact{ii}"]:
                    tg_list["tgs"].append(row[f"Contact{ii}"])
            json_data["talkgroup_lists"].append(tg_list)
    print(f"Parsed {len(json_data['talkgroup_lists']):4d} talkgroup lists...")

    return json_data