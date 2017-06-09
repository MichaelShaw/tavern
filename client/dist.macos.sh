#!/bin/sh
echo "Making Dist directory"
mkdir -p dist/macos
echo "Cleaning zip file"
rm ./dist/macos/tavern.app.zip
echo "Cleaning release"
rm -rf ./target/release/tavern.app
echo "Building"
cargo build --release
echo "Stripping"
strip target/release/tavern_client
echo "Creating base Bundle"
cargo bundle --release -d resources
echo "Copying fonts"
cp -R resources/* target/release/tavern.app/Contents/Resources
echo "Copying OpenAL"
cp ./native/openal.dylib target/release/tavern.app/Contents/Resources/openal.dylib 
echo "Zipping"
cd ./target/release && zip -r ./tavern.app.zip ./tavern.app -x ".*" -x "*/.*" && cd ../..
echo "Copying zip"
cp ./target/release/tavern.app.zip ./dist/macos/tavern.app.zip