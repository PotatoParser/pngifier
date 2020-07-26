#!/bin/bash

windows_x86='i686-pc-windows-msvc'
windows_x64='x86_64-pc-windows-msvc'

TOML='Cargo.toml'

CONFIGS='configs'
BUILD='builds'

zlib="$CONFIGS/zlib.toml"
miniz="$CONFIGS/miniz.toml"
flate2="$CONFIGS/flate2.toml"
cloudflare="$CONFIGS/cloudflare.toml"

switch() {
	cp ${!1} Cargo.toml
}

build_single() {
	build_path="$BUILD/$1/$2"
	stored_path="target/${!1}/release/pngifier.exe"
	if cargo build --quiet --release --target=${!1} > /dev/null 2>&1; then
		mkdir -p $build_path
		cp $stored_path $build_path/pngifier.exe
		echo "Success $stored_path -> $build_path/pngifier.exe"
	else
		echo "Failed $stored_path -> $build_path/pngifier.exe"
	fi
}

build() {
	switch $1
	build_single windows_x64 $1
	build_single windows_x86 $1
}

rm -r $BUILD
build zlib
build miniz
build_single windows_x64 bin
build_single windows_x86 bin
build flate2
build cloudflare
switch miniz

zip() {
	new_path="pngifier-${TRAVIS_TAG}-$2"
	mv $BUILD/$1 $BUILD/$new_path
	cp LICENSE $BUILD/$new_path
	powershell Compress-Archive $BUILD/$new_path $BUILD/$new_path.zip -Force
}
zip windows_x64 windows-x64
zip windows_x86 windows-x86