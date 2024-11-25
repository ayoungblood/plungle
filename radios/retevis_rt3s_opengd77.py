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
# - Squelch: blank for digital channels, generally None for analog channels
# - Power: [P3,P6,P9] or Master
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
    json_data["channels"] = []
    file_path = os.path.join(input_path, 'Channels.csv')
    if not os.path.isfile(file_path):
        print(f"Error: {file_path} does not exist.")
        print(f"Please export the codeplug and try again.")
        return
    with open(file_path, 'r') as csv_file:
        csv_reader = csv.DictReader(csv_file)
        for row in csv_reader:
            channel = {
                "index": row["Channel Number"],
                "name": row["Channel Name"],
                "mode": "DMR" if row["Channel Type"] == "Digital" else "FM",
                "freq_rx": RadioHelper.mhzStr_to_hzInt(row["Rx Frequency"]),
                "freq_tx": RadioHelper.mhzStr_to_hzInt(row["Tx Frequency"]),
                "rx_only": row["Rx Only"] == "Yes",
            }
            # if channel["mode"] == "DMR":

            json_data["channels"].append(channel)

    # Validate JSON data
    if len(json_data["channels"]) == 0:
        print("Error: No channels found in the CSV file.")
        return
    else:
        print(f"Found {len(json_data['channels'])} channels.")

    for channel in json_data["channels"]:
        if channel["freq_rx"] != channel["freq_tx"]:
            if RadioHelper.is_2m(channel["freq_rx"]) and RadioHelper.is_2m(channel["freq_tx"]):
                # 2m monoband
                if abs(channel["freq_rx"] - channel["freq_tx"]) != 600_000:
                    print(f"Error: Channel {channel['index']} has invalid frequency separation: rx={RadioHelper.hzInt_to_mhzStr(channel['freq_rx'])}, tx={RadioHelper.hzInt_to_mhzStr(channel['freq_tx'])}, offset={RadioHelper.hzInt_to_mhzStr(channel['freq_rx'] - channel['freq_tx'])}")
            if RadioHelper.is_70cm(channel["freq_rx"]) and RadioHelper.is_70cm(channel["freq_tx"]):
                # 70cm monoband
                if abs(channel["freq_rx"] - channel["freq_tx"]) != 5_000_000:
                    print(f"Error: Channel {channel['index']} has invalid frequency separation: rx={RadioHelper.hzInt_to_mhzStr(channel['freq_rx'])}, tx={RadioHelper.hzInt_to_mhzStr(channel['freq_tx'])}, offset={RadioHelper.hzInt_to_mhzStr(channel['freq_rx'] - channel['freq_tx'])}")

    return json_data