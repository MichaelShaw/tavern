#!/bin/sh
set -e
echo "Removing old dist"
rm -rf ./dist/win/*
echo "Making Dist directories"
mkdir -p ./dist/win/tavern
mkdir -p ./dist/win/tavern/native
mkdir -p ./dist/win/tavern/resources
echo "Cleaning zip file"
rm -f ./dist/win/tavern.zip
echo "Building"
cargo build --release
echo "Stripping"
strip target/release/tavern.exe
echo "Copying binary"
cp target/release/tavern.exe dist/win/tavern/tavern.exe
echo "Copying resources"
cp -R resources/* dist/win/tavern/resources
echo "Copying OpenAL"
cp ./native/OpenAL64.dll dist/win/tavern/native/OpenAL64.dll
echo "Zipping"
cd ./dist/win && zip -r ./tavern.zip ./tavern -x ".*" -x "*/.*" && cd ../..
