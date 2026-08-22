#!/bin/bash
rm -rf ./pkg
wasm-pack build --target web 
cd demo_apps/compass
npm run build
