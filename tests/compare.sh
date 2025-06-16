#!/usr/bin/env bash

# rebuild
cargo build

# executable target
target='../target/debug/plungle'

# create temp directory if it doesn't exist
tempdir='temp.compare'
mkdir -p $tempdir

printf "\n\x1b[4;36m[compare.sh] Testing Ailunce HD1 >>>>>>>>>>>>>>>>>>>>>>>>>>>>\x1b[0m\n"
rm -rf $tempdir/*
# Parse HD1 fixture and write to output.json
$target parse -q ailunce_hd1 ../fixtures/ailunce_hd1/basic/ $tempdir/output.json
printf "\n[compare.sh] parse finished with return code $?\n"
# Generate HD1 codeplug from output.json
$target generate -q ailunce_hd1 $tempdir/output.json $tempdir/output
printf "\n[compare.sh] generate finished with return code $?\n\n"
# Compare generated codeplug with original, file by file
printf "[compare.sh] Comparing files, ignoring line endings\n"
for file in $(ls $tempdir/output); do
    printf "[compare.sh] Comparing $file\n"
    # diff --strip-trailing-cr $tempdir/output/$file ../fixtures/ailunce_hd1/basic/$file
    printf "[compare.sh]     diff returned $?\n"
done

printf "\n\x1b[4;36m[compare.sh] Testing Alinco DJ-MD5T >>>>>>>>>>>>>>>>>>>>>>\x1b[0m\n"
rm -rf $tempdir/*
# Parse Alinco DJ-MD5T fixture and write to output.json
$target parse -q alinco_djmd5t ../fixtures/alinco_dj-md5t/basic/ $tempdir/output.json
printf "\n[compare.sh] parse finished with return code $?\n"
# Generate Alinco DJ-MD5T codeplug from output.json
$target generate -q alinco_djmd5t $tempdir/output.json $tempdir/output
printf "\n[compare.sh] generate finished with return code $?\n\n"
# Compare generated codeplug with original, file by file
printf "[compare.sh] Comparing files, ignoring line endings\n"
for file in $(ls $tempdir/output); do
    printf "[compare.sh] Comparing $file\n"
    diff --strip-trailing-cr $tempdir/output/$file ../fixtures/alinco_dj-md5t/basic/$file
    printf "[compare.sh]     diff returned $?\n"
done
# debug
# meld ../fixtures/alinco_dj-md5t/basic/ $tempdir/output

printf "\n\x1b[4;36m[compare.sh] Testing Anytone AT-D878UV >>>>>>>>>>>>>>>>>>>>>>\x1b[0m\n"
rm -rf $tempdir/*
# Parse AT-D878UV fixture and write to output.json
$target parse -q anytone_x78 ../fixtures/anytone_d878uv/basic/ $tempdir/output.json
printf "\n[compare.sh] parse finished with return code $?\n"
# Generate AT-D878UV codeplug from output.json
$target generate -q anytone_x78 $tempdir/output.json $tempdir/output
printf "\n[compare.sh] generate finished with return code $?\n\n"
# Compare generated codeplug with original, file by file
printf "[compare.sh] Comparing files, ignoring line endings\n"
for file in $(ls $tempdir/output); do
    printf "[compare.sh] Comparing $file\n"
    diff --strip-trailing-cr $tempdir/output/$file ../fixtures/anytone_d878uv/basic/$file
    printf "[compare.sh]     diff returned $?\n"
done
# debug
# meld ../fixtures/anytone_d878uv/basic/ $tempdir/output

printf "\n\x1b[4;36m[compare.sh] Testing Chirp (generic) >>>>>>>>>>>>>>>>>>>>>\x1b[0m\n"
rm -rf $tempdir/*
# Parse chirp fixture and write to output.json
$target parse -q chirp_generic ../fixtures/chirp_generic/basic.csv $tempdir/output.json
printf "\n[compare.sh] parse finished with return code $?\n"
# Generate chirp codeplug from output.json
$target generate -q chirp_generic $tempdir/output.json $tempdir/output.csv
printf "\n[compare.sh] generate finished with return code $?\n\n"
# Compare generated codeplug with original, file by file
printf "[compare.sh] Comparing files, ignoring line endings\n"
file="basic.csv"
printf "[compare.sh] Comparing $file\n"
diff --strip-trailing-cr $tempdir/output.csv ../fixtures/chirp_generic/$file
printf "[compare.sh]     diff returned $?\n"
# debug
# meld ../fixtures/chirp_generic/basic.csv $tempdir/output.csv

printf "\n\x1b[4;36m[compare.sh] Testing OpenGD77 RT3S >>>>>>>>>>>>>>>>>>>>>>>>>>\x1b[0m\n"
rm -rf $tempdir/*
# Parse RT3S fixture and write to output.json
$target parse -q opengd77_rt3s ../fixtures/opengd77_rt3s/basic/ $tempdir/output.json
printf "\n[compare.sh] parse finished with return code $?\n"
# Generate RT3S codeplug from output.json
$target generate -q opengd77_rt3s $tempdir/output.json $tempdir/output
printf "\n[compare.sh] generate finished with return code $?\n\n"
# Compare generated codeplug with original, file by file
printf "[compare.sh] Comparing files, ignoring line endings\n"
for file in $(ls $tempdir/output); do
    printf "[compare.sh] Comparing $file\n"
    diff --strip-trailing-cr $tempdir/output/$file ../fixtures/opengd77_rt3s/basic/$file
    printf "[compare.sh]     diff returned $?\n"
done
# debug
# meld ../fixtures/opengd77_rt3s/basic/ $tempdir/output
