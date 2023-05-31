#!/bin/bash

echo -e "\n0L: running smoke tests"

if [[ ! -v ZAPATOS ]]
then
    echo $ZAPATOS
    echo "0L: '\$ZAPATOS' source path does not exist,"
    return
fi

export ZAPATOS_BIN_PATH=$ZAPATOS/target/release

unset MRB_PATH
export MRB_PATH=$(cd ./framework/releases/ && pwd -P | xargs -I {} echo "{}/head.mrb")

(cd smoke-tests && ZAPATOS_BIN_PATH=$ZAPATOS/target/release cargo test -- --nocapture)

