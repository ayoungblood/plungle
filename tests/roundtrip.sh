#!/usr/bin/env bash

# "roundtrip" testing:
# Test conversions:
# - from radio A to radio B
# - then from radio B to radio A
# - compare

# executable
target='../target/debug/plungle'

# create a directory if it doesn't exist
tempdir=$(mktemp -d -t plungle.tmp.XXXXXX 2>/dev/null || mktemp -d -t 'plungle.tmp.XXXXXX')
echo $tempdir

printf "\n[roundtrip.sh] OpenGD77 RT3S > Alinco DJ-MD5T > OpenGD77 RT3S"

# Parse RT3S fixture and write to opengd77_rt3s.json
$target parse -q opengd77_rt3s ../fixtures/opengd77_rt3s/basic/ $tmpdir/opengd77_rt3s.json
