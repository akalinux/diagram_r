#!/bin/bash
wasm-pack build --target web --dev
rm -f demo_apps/compass/public/js/diagram_r_bg.wasm
cp pkg/diagram_r_bg.wasm demo_apps/compass/public/js/diagram_r_bg.wasm
