#!/bin/bash

# sudo apt install gcc-multilib
linux_i686='i686-unknown-linux-gnu'
linux_x86='x86_64-unknown-linux-gnu'
linux_arm64='aarch64-unknown-linux-gnu'

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
	if cross build --quiet --release --target=${!1} > /dev/null 2>&1; then
		mkdir -p $build_path
		cp $stored_path $build_path/pngifier
		echo "Success $stored_path -> $build_path/pngifier"
	else
		echo "Failed $stored_path -> $build_path/pngifier"
	fi
}

build() {
	if [ $1 != bin ]; then
		switch $1
	fi
	build_single linux_i686 $1
	build_single linux_x86 $1
	build_single linux_arm64 $1
}

rm -r $BUILD
build zlib
build miniz
build bin
build flate2
build cloudflare
switch miniz

zip() {
	new_path="pngifier-${TRAVIS_TAG}-$2"
	mv $BUILD/$1 $BUILD/$new_path
	cp LICENSE $BUILD/$new_path
	tar -C $BUILD -czf $BUILD/$new_path.tar.gz $new_path
}
zip linux_i686 linux-i686
zip linux_x86 linux-x86
zip linux_arm64 linux-arm64