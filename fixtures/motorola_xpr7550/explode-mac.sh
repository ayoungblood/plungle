#!/usr/bin/env bash

input_xml="$1"

# Check if an input file was provided
if [[ -z "$input_xml" ]]; then
  echo "Usage: $0 input_xml [output_dir]"
  exit 1
fi

# Check if the input file exists
if [[ ! -f "$input_xml" ]]; then
  echo "Error: Input file '$input_xml' not found."
  exit 1
fi

# Make sure that an output directory exists
output_dir="output"
# if an output_dir was provided, use it
if [[ -n "$2" ]]; then
  output_dir="$2"
fi
mkdir -p "$output_dir"

# Build a list of ListIDs
list_ids=$(cat $1 | grep "<CNV_PER_CMP_TYPE Applicable=\"Enabled\"" | \
# get the last column
awk '{print $NF}' | \
# remove the ListID=" prefix
sed 's/ListID="//g' | \
# remove the closing quote
sed 's/">//g' | \
# sort
sort -sn )

# Loop through the list of ListIDs
for id in $list_ids; do
    tag=$(printf "<CNV_PER_CMP_TYPE Applicable=\"Enabled\" ListID=\"$id\"")
    filename=$(printf "CH-%03d" $id)
    path="$output_dir/$filename"
    printf "%s => %s\n" "$filename" "$path"
    cat $1 | \
    # grab the entire channel
    grep "$tag" -A249 | \
    # strip off the start/end tags
    grep -v "CNV_PER_CMP_TYPE" | \
    # strip off leading whitespace
    sed 's/^[[:space:]]*//g' | \
    # write to files
    tee "$output_dir/$filename" | \
    # change the color
    sed 's/\(.*\)/\x1b[90m\1\x1b[0m/'
done
