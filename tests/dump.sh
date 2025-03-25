#!/usr/bin/env bash

# get the project directory
script_dir=$(dirname $0)
plungle_dir=$(realpath $script_dir/..)

# executable target
target="$plungle_dir/target/debug/plungle"

$target parse ailunce_hd1 "$plungle_dir/fixtures/ailunce_hd1/basic/"

$target parse alinco_djmd5t "$plungle_dir/fixtures/alinco_dj-md5t/basic/"

$target parse anytone_x78 "$plungle_dir/fixtures/anytone_d878uv/basic/"

$target parse chirp_generic "$plungle_dir/fixtures/chirp_generic/basic.csv"

$target parse opengd77_rt3s "$plungle_dir/fixtures/opengd77_rt3s/basic/"

$target parse tyt_mduv390 "$plungle_dir/fixtures/tyt_mduv390/basic/"
