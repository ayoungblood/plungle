#!/bin/bash

# This script sorts the XML tags within each <CNV_PER_CMP_TYPE> section
# and removes the ListID attributes from the XML files in a directory.
# Usage
#   ./filter-sort-xml.sh <directory>

# Directory containing XML files
directory="$1"

# Check if a directory was provided
if [[ -z "$directory" ]]; then
  echo "Usage: $0 <directory>"
  exit 1
fi

# Check if the directory exists
if [[ ! -d "$directory" ]]; then
  echo "Error: Directory '$directory' not found."
  exit 1
fi

# Iterate through XML files in the directory
for xml_file in "$directory"/*.xml; do
  if [[ -f "$xml_file" ]]; then
    # Remove ListID attributes using sed
    filtered_xml=$(sed 's/ ListID="[0-9]*"//g' "$xml_file")
    # Remove Alias attributes using sed as they just cause noise in the diff
    filtered_xml=$(echo "$filtered_xml" | sed 's/ Alias="[0-9]*"//g')

    # Sort the XML tags using awk and sort
    sorted_xml=$(echo "$filtered_xml" | awk '
      /<CNV_PER_CMP_TYPE[[:space:]]/{
        in_section = 1;
        print $0; # Print the opening tag
        next;
      }

      in_section {
        if (/<\/CNV_PER_CMP_TYPE>/) {
          in_section = 0;
          for (i = 1; i <= num_tags; i++) {
            print tags[i];
          }
          print $0; # Print the closing tag
          num_tags = 0;
        } else {
          tags[++num_tags] = $0;
        }
      }

      END {
        if (num_tags > 0) {
          for (i = 1; i <= num_tags; i++) {
            print tags[i];
          }
        }
      }
    ' | sort)

    # Re-assemble the xml file.
    sorted_xml=$(echo "$sorted_xml" | awk '
    NR==1{print $0;next;}
    {
        if(match($0, /<CP_[A-Z]/)){
            print $0;
        }
    }
    ' | sort | awk '
    NR==1{print $0;next;}
    {print $0;}
    ' | awk -v first_line="$(echo "$filtered_xml" | head -n 1)" -v last_line="$(echo "$filtered_xml" | tail -n 1)" '
    NR==1{print first_line;next;}
    {print $0;}
    END{print last_line;}
    ')

    # Create the sorted and filtered XML filename
    sorted_file="${xml_file%.xml}-sorted-filtered.xml"

    # Write the sorted and filtered XML to the new file
    echo "$sorted_xml" > "$sorted_file"

    echo "Sorted and filtered XML written to '$sorted_file'."
  fi
done

echo "XML sorting and filtering complete."
