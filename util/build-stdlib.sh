#!/bin/bash
# set -e # exit on error

echo -e "\n0L: Building ol-framework ..."

$ZAPATOS/target/release/aptos-framework custom --packages ./ol-framework --rust-bindings "" --output ./ol-framework/releases/head.mrb

printf "0L: Success building release bundle 'head.mrb' in ./ol-framework/releases/head.mrb"