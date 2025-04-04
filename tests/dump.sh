#!/usr/bin/env bash

# colors
ANSI_BLK_GRN="\e[42m\e[30m"
ANSI_RESET="\e[0m"

# get the project directory
script_dir=$(dirname $0)
plungle_dir=$(realpath $script_dir/..)

# executable target
target="$plungle_dir/target/debug/plungle"

printf "$ANSI_BLK_GRN# plungle parse ailunce_hd1$ANSI_RESET\n"
$target parse ailunce_hd1 -q "$plungle_dir/fixtures/ailunce_hd1/basic/"

printf "$ANSI_BLK_GRN# plungle parse alinco_djmd5t$ANSI_RESET\n"
$target parse alinco_djmd5t -q "$plungle_dir/fixtures/alinco_dj-md5t/basic/"

printf "$ANSI_BLK_GRN# plungle parse anytone_x78$ANSI_RESET\n"
$target parse anytone_x78 -q "$plungle_dir/fixtures/anytone_d878uv/basic/"

printf "$ANSI_BLK_GRN# plungle parse chirp_generic$ANSI_RESET\n"
$target parse chirp_generic -q "$plungle_dir/fixtures/chirp_generic/basic.csv"

printf "$ANSI_BLK_GRN# plungle parse opengd77_rt3s$ANSI_RESET\n"
$target parse opengd77_rt3s -q "$plungle_dir/fixtures/opengd77_rt3s/basic/"

printf "$ANSI_BLK_GRN# plungle parse tyt_mduv390$ANSI_RESET\n"
$target parse tyt_mduv390 -q "$plungle_dir/fixtures/tyt_mduv390/basic/"
