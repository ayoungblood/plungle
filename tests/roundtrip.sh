#!/usr/bin/env bash

# "roundtrip" testing:
# Test conversions:
# - from radio A to radio B
# - then from radio B to radio A
# - compare

# rebuild
cargo build

# executable
target='../target/debug/plungle'

# create a directory if it doesn't exist
tempdir="temp.roundtrip"
echo $tempdir
mkdir -p $tempdir

rm -rf $tempdir/*
printf "\n\x1b[4;36m[roundtrip.sh] OpenGD77 RT3S > Alinco DJ-MD5T > OpenGD77 RT3S\x1b[0m\n"
# Parse RT3S fixture and write to opengd77_rt3s.json
$target parse -q opengd77_rt3s ../fixtures/opengd77_rt3s/basic/ $tempdir/opengd77_rt3s.json
# Generate DJ-MD5T codeplug from opengd77_rt3s.json
$target generate -q alinco_djmd5t $tempdir/opengd77_rt3s.json $tempdir/djmd5t/
# Parse DJ-MD5T codeplug and write to djmd5t.json
$target parse -q alinco_djmd5t $tempdir/djmd5t/ $tempdir/djmd5t.json
# Generate RT3S codeplug from djmd5t.json
$target generate -q opengd77_rt3s $tempdir/djmd5t.json $tempdir/opengd77_rt3s/
# Compare generated codeplug with original, file by file
printf "[roundtrip.sh] Comparing files, ignoring line endings\n"
for file in $(ls $tempdir/opengd77_rt3s); do
    printf "[roundtrip.sh] Comparing $file\n"
    diff --strip-trailing-cr $tempdir/opengd77_rt3s/$file ../fixtures/opengd77_rt3s/basic/$file
    printf "[roundtrip.sh]     diff returned $?\n"
done
# debug
# meld ../fixtures/opengd77_rt3s/basic/ $tempdir/opengd77_rt3s/

rm -rf $tempdir/*
printf "\n\x1b[4;36m[roundtrip.sh] Anytone D878UV > Alinco DJ-MD5T > Anytone D878UV\x1b[0m\n"
# Parse D878UV fixture and write to anytone_d878uv.json
$target parse -q anytone_x78 ../fixtures/anytone_d878uv/basic/ $tempdir/anytone_d878uv.json
# Generate DJ-MD5T codeplug from anytone_d878uv.json
$target generate -q alinco_djmd5t $tempdir/anytone_d878uv.json $tempdir/djmd5t/
# Parse DJ-MD5T codeplug and write to djmd5t.json
$target parse -q alinco_djmd5t $tempdir/djmd5t/ $tempdir/djmd5t.json
# Generate D878UV codeplug from djmd5t.json
$target generate -q anytone_x78 $tempdir/djmd5t.json $tempdir/anytone_d878uv/
# Compare generated codeplug with original, file by file
printf "[roundtrip.sh] Comparing files, ignoring line endings\n"
for file in $(ls $tempdir/anytone_d878uv); do
    printf "[roundtrip.sh] Comparing $file\n"
    diff --strip-trailing-cr $tempdir/anytone_d878uv/$file ../fixtures/anytone_d878uv/basic/$file
    printf "[roundtrip.sh]     diff returned $?\n"
done
# debug
# meld ../fixtures/anytone_d878uv/basic/ $tempdir/anytone_d878uv/
# meld $tempdir/anytone_d878uv.json $tempdir/djmd5t.json

rm -rf $tempdir/*
printf "\n\x1b[4;36m[roundtrip.sh] RMHAM LARGE Anytone D878UV > Anytone D878UV\x1b[0m\n"
# Parse RMHAM Anytone fixture
$target parse -q anytone_x78 ../fixtures/anytone_d878uv/rmham_anytone_2024-11-27/Export/ $tempdir/rmham_anytone.json
# Generate Anytone D878UV codeplug from rmham_anytone.json
$target generate -q anytone_x78 $tempdir/rmham_anytone.json $tempdir/rmham_anytone/
# Compare generated codeplug with original, file by file
printf "[roundtrip.sh] Comparing files, ignoring line endings\n"
for file in $(ls $tempdir/rmham_anytone); do
    printf "[roundtrip.sh] Comparing $file\n"
    diff --strip-trailing-cr $tempdir/rmham_anytone/$file ../fixtures/anytone_d878uv/rmham_anytone_2024-11-27/Export/$file
    printf "[roundtrip.sh]     diff returned $?\n"
done
# debug
# meld ../fixtures/anytone_d878uv/rmham_anytone_2024-11-27/Export/ $tempdir/rmham_anytone/
