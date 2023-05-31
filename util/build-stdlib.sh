#!/bin/bash
#set -e # exit on error

echo -e "\n0L: Building move framework ..."

# release without source code
cargo r -p libra-framework -- release -w

printf "0L: Success building .mrb release bundle. Saved to: framework/releases/head.mrb \n"