#!/bin/bash
# set -e # exit on error

echo -e "\n0L: Building move framework ..."

export OUT=(./framework/releases/framework_fresh.mrb)
aptos-framework custom --packages ./framework --rust-bindings "" --output $OUT 

printf "0L: Success building .mrb release bundle. Saved to: $OUT \n"