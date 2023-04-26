#!/bin/bash
set -e # exit on error

echo -e "\n0L: Warning: Dev. usage only - Building ol-framework WITHOUT dependencies ...\n"

$ZAPATOS/target/release/aptos-framework custom \
  --packages $LIBRA_V7/ol-framework \
  --rust-bindings "" \
  --output ol.mrb

printf "0L: Success building ol bundle 'ol.mrb' WITHOUT dependencies in current dir\n"