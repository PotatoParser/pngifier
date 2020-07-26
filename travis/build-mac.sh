#!/bin/bash

mac='x86_64-apple-darwin'

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
	stored_path="target/${!1}/release/pngifier"
	if cargo build --quiet --release --target=${!1} > /dev/null 2>&1; then
		mkdir -p $build_path
		cp $stored_path $build_path/pngifier
		echo "Success $stored_path -> $build_path/pngifier"
	else
		echo "Failed $stored_path -> $build_path/pngifier"
	fi
}

build() {
	switch $1
	build_single mac $1
}

rm -r $BUILD
build zlib
build miniz
build_single mac bin
build flate2
build cloudflare
switch miniz

zip() {
	new_path="pngifier-${TRAVIS_TAG}-$2"
	mv $BUILD/$1 $BUILD/$new_path
	cp LICENSE $BUILD/$new_path
	tar -C $BUILD -czf $BUILD/$new_path.tar.gz $new_path
}
zip mac mac