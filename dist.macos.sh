#!/bin/sh
echo "Cleaning release"
rm -rf ./target/release/tavern.app
echo "Building"
cargo build --release
echo "Stripping"
strip target/release/tavern
echo "Creating base Bundle"
cargo bundle --release -d resources
echo "Copying fonts"
cp -R resources/* target/release/tavern.app/Contents/Resources
echo "Copying OpenAL"
cp ./native/openal.dylib target/release/tavern.app/Contents/Resources/openal.dylib 
echo "Zipping"