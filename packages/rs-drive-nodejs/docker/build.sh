#!/usr/bin/env bash

## Setup arguments
while getopts t:a:l: flag
do
    case "${flag}" in
        t) target=${OPTARG};;
        a) arch=${OPTARG};;
        l) libc=${OPTARG};;
        *) echo "invalid arguments" && exit 1;;
    esac
done

## Install multilib
apt update
apt install -y gcc-multilib
if [[ $target = "aarch64-unknown-linux-gnu" ]]
then
    apt install -y gcc-aarch64-linux-gnu libstdc++-11-dev-arm64-cross
fi

# sccache requirements
#apt install libssl-dev pkg-config

## Update toolchain
rustup update stable

## Install build target
rustup target install $target

chmod 777 -R /root/.cargo
mkdir -p /github/workspace/target
chmod 777 -R /github/workspace/target

## Install Node.JS
curl -fsSL https://deb.nodesource.com/setup_16.x | sudo -E bash -
apt install -y nodejs

#cargo install sccache@0.4.1

corepack enable

yarn install

#SCCACHE_GHA_ENABLED=true \
#RUSTC_WRAPPER=sccache \
#CARGO_INCREMENTAL=false \
CARGO_BUILD_TARGET=$target \
CARGO_BUILD_PROFILE=release \
ARCH=$arch \
LIBC=$libc \
yarn workspace @dashevo/rs-drive run build
