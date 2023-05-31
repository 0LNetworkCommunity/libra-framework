#!/bin/bash
#set -e # exit on error

echo -e "\n0L: Building move framework ..."
unset OUT
export OUT=$(cd ./framework/releases/ && pwd -P | xargs -I {} echo "{}/fresh.mrb")
# echo $OUT
$ZAPATOS/target/release/aptos-framework custom \
  --packages $ZAPATOS/aptos-move/framework/move-stdlib \
  --packages $ZAPATOS/aptos-move/framework/aptos-stdlib \
  --packages ./framework \
  --rust-bindings "" \
  --rust-bindings "" \
  --rust-bindings "" \
  --output $OUT

printf "0L: Success building .mrb release bundle. Saved to: $OUT \n"