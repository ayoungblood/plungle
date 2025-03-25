#!/usr/bin/env bash

# executable target
target='../target/debug/plungle'

# create temp directory if it doesn't exist
tempdir='temp'
mkdir -p $tempdir

printf "\n[compare.sh] Testing OpenGD77 RT3S >>>>>>>>>>>>>>>>>>>>>>>>>>\n"
rm -rf $tempdir/*

# Parse RT3S fixture and write to output.json
$target parse opengd77_rt3s ../fixtures/opengd77_rt3s/basic/ $tempdir/output.json
printf "\n[compare.sh] parse finished with return code $?\n\n"

# Generate RT3S codeplug from output.json
$target generate opengd77_rt3s $tempdir/output.json $tempdir/output
printf "\n[compare.sh] generate finished with return code $?\n\n"

# Compare generated codeplug with original, file by file
printf "[compare.sh] Comparing files, ignoring line endings\n"
for file in $(ls $tempdir/output); do
    printf "[compare.sh] Comparing $file\n"
    diff --strip-trailing-cr $tempdir/output/$file ../fixtures/opengd77_rt3s/basic/$file
    printf "[compare.sh]     diff returned $?\n"
done

printf "\n[compare.sh] Testing Anytone AT-D878UV >>>>>>>>>>>>>>>>>>>>>>\n"
rm -rf $tempdir/*

# Parse AT-D878UV fixture and write to output.json
$target parse anytone_x78 ../fixtures/anytone_d878uv/basic/ $tempdir/output.json
printf "\n[compare.sh] parse finished with return code $?\n\n"

# Generate AT-D878UV codeplug from output.json
$target generate anytone_x78 $tempdir/output.json $tempdir/output
printf "\n[compare.sh] generate finished with return code $?\n\n"

# Compare generated codeplug with original, file by file
printf "[compare.sh] Comparing files, ignoring line endings\n"
for file in $(ls $tempdir/output); do
    printf "[compare.sh] Comparing $file\n"
    diff --strip-trailing-cr $tempdir/output/$file ../fixtures/anytone_d878uv/basic/$file
    printf "[compare.sh]     diff returned $?\n"
done

printf "\n[compare.sh] Testing Alinco DJ-MD5T >>>>>>>>>>>>>>>>>>>>>>\n"
rm -rf $tempdir/*

# Parse Alinco DJ-MD5T fixture and write to output.json
$target parse alinco_djmd5t ../fixtures/alinco_dj-md5t/basic/ $tempdir/output.json
printf "\n[compare.sh] parse finished with return code $?\n\n"

# Generate Alinco DJ-MD5T codeplug from output.json
$target generate alinco_djmd5t $tempdir/output.json $tempdir/output
printf "\n[compare.sh] generate finished with return code $?\n\n"

# Compare generated codeplug with original, file by file
printf "[compare.sh] Comparing files, ignoring line endings\n"
for file in $(ls $tempdir/output); do
    printf "[compare.sh] Comparing $file\n"
    diff --strip-trailing-cr $tempdir/output/$file ../fixtures/alinco_dj-md5t/basic/$file
    printf "[compare.sh]     diff returned $?\n"
done

printf "\n[compare.sh] Testing Chirp (generic) >>>>>>>>>>>>>>>>>>>>>\n"
rm -rf $tempdir/*

# Parse chirp fixture and write to output.json
$target parse chirp_generic ../fixtures/chirp_generic/basic.csv $tempdir/output.json
printf "\n[compare.sh] parse finished with return code $?\n\n"

# Generate chirp codeplug from output.json
$target generate chirp_generic $tempdir/output.json $tempdir/output.csv
printf "\n[compare.sh] generate finished with return code $?\n\n"

# Compare generated codeplug with original, file by file
printf "[compare.sh] Comparing files, ignoring line endings\n"
file="basic.csv"
printf "[compare.sh] Comparing $file\n"
diff --strip-trailing-cr $tempdir/output.csv ../fixtures/chirp_generic/$file
printf "[compare.sh]     diff returned $?\n"
