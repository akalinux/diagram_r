#!/bin/bash
echo "Clearing old node modules"
rm -rf node_modules
echo "Creating local yarn lock"
touch yarn.lock
echo "installing npm packages"
yarn install

echo "removing dummy package"
rm -rf node_modules/diagram_r

echo "linking local wasm binary"
target="$(cd $(pwd)/../../;pwd)/pkg"
ln -s $target node_modules/diagram_r
