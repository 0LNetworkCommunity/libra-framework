#!/bin/bash
# set -e # exit on error

echo -e "\n0L: running smoke tests"

export MRB_PATH=$(pwd | xargs -I % echo '%/framework/releases/framework_fresh.mrb')
# export ZAPATOS_BIN_PATH=(~/code/rust/zapatos/target/release)
# echo $ZAPATOS_BIN_PATH
echo $MRB_PATH

(cd smoke-tests && MRB_PATH=$MRB_PATH ZAPATOS_BIN_PATH=$(echo ~/code/rust/zapatos/target/release) cargo test -- --nocapture)

