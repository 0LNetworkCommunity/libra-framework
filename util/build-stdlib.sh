#!/bin/bash
set -e # exit on error

echo -e "\n0L: Building ol-framework ..."

$ZAPATOS/target/release/aptos-framework custom \
  --packages  $ZAPATOS/aptos-move/framework/move-stdlib \
  --packages  $ZAPATOS/aptos-move/framework/aptos-stdlib \
  --packages  $ZAPATOS/aptos-move/framework/aptos-framework \
  --packages  $ZAPATOS/aptos-move/framework/aptos-token \
  --packages $LIBRA_V7/ol-framework \
  --rust-bindings "" \
  --rust-bindings "" \
  --rust-bindings "" \
  --rust-bindings "" \
  --rust-bindings "" \
  --output head.mrb

printf "0L: Success building release bundle 'head.mrb' in current dir\n\n"