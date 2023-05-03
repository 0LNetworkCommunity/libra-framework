#!/bin/bash
set -e # exit on error

echo -e "\n0L: Warning: Dev. usage only - Building ol-framework WITHOUT dependencies ...\n"

$ZAPATOS/target/release/aptos-framework custom \
  --packages $LIBRA_V7/ol-framework \
  --rust-bindings "" \
  --output /tmp/ol.mrb

printf "0L: Success building ol bundle WITHOUT dependencies in '/tmp/ol.mrb'\n\n"