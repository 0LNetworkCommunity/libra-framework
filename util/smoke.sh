#!/bin/bash
# set -e # exit on error

echo -e "\n0L: running smoke tests"

export MRB_PATH=$(pwd | xargs -I % echo '%/framework/releases')
# export ZAPATOS_BIN_PATH=(~/code/rust/zapatos/target/release)
# echo $ZAPATOS_BIN_PATH
echo $MRB_PATH

ZAPATOS_BIN_PATH=$(echo /code/rust/zapatos/target/release) cargo test -- --nocapture

