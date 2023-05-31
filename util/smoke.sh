#!/bin/bash
# set -e # exit on error

echo -e "\n0L: running smoke tests"
unset MRB_PATH
export MRB_PATH=$(cd ./framework/releases/ && pwd -P | xargs -I {} echo "{}/head.mrb")
export ZAPATOS_BIN_PATH=$ZAPATOS/target/release
# export ZAPATOS_BIN_PATH=(~/code/rust/zapatos/target/release)
# echo $ZAPATOS_BIN_PATH
echo $MRB_PATH

(cd smoke-tests && MRB_PATH=$MRB_PATH ZAPATOS_BIN_PATH=$ZAPATOS/target/release cargo test -- --nocapture)

