#!/usr/bin/env python3

import argparse
import os
import glob
import json as json_lib

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
    json = radio_module.csv2json(args.input)
    # print(json)
    print(json_lib.dumps(json, indent=4))

if __name__ == "__main__":
    main()
