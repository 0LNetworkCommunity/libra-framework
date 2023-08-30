#!/bin/bash

echo -e "\n0L: running smoke tests"

if [[ ! -v DIEM_FORGE_NODE_BIN_PATH ]]
then
    echo $DIEM_FORGE_NODE_BIN_PATH
    echo "0L: '\$DIEM_FORGE_NODE_BIN_PATH' source path does not exist,"
    return
fi


unset MRB_PATH
export MRB_PATH=$(cd ./framework/releases/ && pwd -P | xargs -I {} echo "{}/head.mrb")

(cd smoke-tests && cargo test -- --nocapture)

