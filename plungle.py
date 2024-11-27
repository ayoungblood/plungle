#!/usr/bin/env python3

import argparse
import os, sys
import signal
import glob
import json as json_lib
from validator import validate

def main():
    parser = argparse.ArgumentParser(
        description="Convert codeplug data between formats for different radios"
    )
    # Enumerate radio models so that they can be listed as choices
    radio_dir = os.path.join(os.path.dirname(__file__), "radios")
    radio_models = [
        os.path.splitext(os.path.basename(f))[0]
        for f in glob.glob(os.path.join(radio_dir, "*.py"))
        if os.path.isfile(f)
    ]
    parser.add_argument("-D", "--dump", type=str, help="Comma separated options for what to dump to the console: all, chan, zone, tg, tgl, or channel number(s)")
    parser.add_argument("radio", choices=radio_models, help="Radio model")
    parser.add_argument("input", help="Input file or directory")
    #parser.add_argument("output_path", help="Path to save the output (directory or file)")

    args = parser.parse_args()

    # Check that the input argument is a file or a directory
    if not os.path.exists(args.input):
        print(f"Error: {args.input} does not exist.")
        return

    # Import the radio-specific module
    radio_module = __import__(f"radios.{args.radio}", fromlist=[""])

    # Parse
    codeplug_json = radio_module.csv2json(args.input)

    # Dump JSON data to the console
    try: # catch exceptions so output can be piped without errors
        if args.dump:
            dump_options = args.dump.split(",")
            for option in dump_options:
                if "all" in option:
                    print(json_lib.dumps(codeplug_json, indent=4))
                elif option.startswith("ch"):
                    print(json_lib.dumps(codeplug_json["channels"], indent=4))
                elif option.startswith("z"):
                    print(json_lib.dumps(codeplug_json["zones"], indent=4))
                elif "tg" in option:
                    print(json_lib.dumps(codeplug_json["talkgroups"], indent=4))
                elif "tgl" in option:
                    print(json_lib.dumps(codeplug_json["talkgroup_lists"], indent=4))
                elif "-" in option: # range
                    start, end = option.split("-")
                    if not start.isnumeric() or not end.isnumeric():
                        print(f"Invalid range: {option}")
                        continue
                    start, end = int(start), int(end)
                    for channel in codeplug_json["channels"]:
                        if start <= channel["index"] <= end:
                            print(json_lib.dumps(channel, indent=4))
                elif option.isnumeric():
                    channel_number = int(option)
                    found = False
                    for channel in codeplug_json["channels"]:
                        if channel["index"] == channel_number:
                            print(json_lib.dumps(channel, indent=4))
                            found = True
                            break
                    if not found:
                        print(f"Channel {channel_number} not found, cannot dump")

    except BrokenPipeError:
        sys.stderr.close()

    # Validate JSON
    if not validate(codeplug_json):
        print("Error: Invalid JSON data.")
        return

if __name__ == "__main__":
    # Handle SIGPIPE signal
    signal.signal(signal.SIGPIPE, signal.SIG_DFL)
    main()
