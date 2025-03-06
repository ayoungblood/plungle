#!/bin/bash

# This script extracts each <CNV_PER_CMP_TYPE> section from an XML file
# and writes it to a separate file named after the CP_CNVPERSALIAS value.
# Usage
#   cd fixtures/motorola_xpr7550/exploded/
#   ../explode-xml.sh ../<input_xml_file>

# Input XML file from command-line argument
input_xml="$1"

# Check if an input file was provided
if [[ -z "$input_xml" ]]; then
  echo "Usage: $0 <input_xml_file>"
  exit 1
fi

# Check if the input file exists
if [[ ! -f "$input_xml" ]]; then
  echo "Error: Input file '$input_xml' not found."
  exit 1
fi

# Use awk to iterate through the <CNV_PER_CMP_TYPE> sections
awk -v input_file="$input_xml" '
  /<CNV_PER_CMP_TYPE[[:space:]]/{
    in_section = 1;
    section = "";
  }
  in_section {
    section = section $0 "\n";
  }
  /<\/CNV_PER_CMP_TYPE>/ {
    in_section = 0;
    # Extract the CP_CNVPERSALIAS value
    if (match(section, /<CP_CNVPERSALIAS[[:space:]][^>]*>(.*)<\/CP_CNVPERSALIAS>/, matches)) {
      filename = matches[1] ".xml";
      # Remove any characters that are not safe for filenames.
      gsub(/[^a-zA-Z0-9._-]/, "_", filename);

      # Output the section to a separate file
      print section > filename;
      close(filename); #close the file so it can be re-opened for the next section.
    } else {
      print "Warning: CP_CNVPERSALIAS not found in section:" > "/dev/stderr";
      print section > "/dev/stderr";
    }
  }
' "$input_xml"

echo "XML sections extracted."
