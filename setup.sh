#!/bin/bash
set -e # Stop script immediately if any command fails

# 1. Detect Architecture
ARCH=$(uname -m)
VERSION="3.18.4"

echo "Detected architecture: $ARCH"

if [ "$ARCH" = "x86_64" ]; then
    URL="https://dl-cdn.alpinelinux.org/alpine/v3.18/releases/x86_64/alpine-minirootfs-${VERSION}-x86_64.tar.gz"
    FILE="alpine-minirootfs-${VERSION}-x86_64.tar.gz"
elif [ "$ARCH" = "aarch64" ]; then
    URL="https://dl-cdn.alpinelinux.org/alpine/v3.18/releases/aarch64/alpine-minirootfs-${VERSION}-aarch64.tar.gz"
    FILE="alpine-minirootfs-${VERSION}-aarch64.tar.gz"
else
    echo "Error: Unsupported architecture $ARCH"
    exit 1
fi

# 2. Prepare the folder
if [ -d "rootfs" ]; then
    echo "Removing existing rootfs..."
    rm -rf rootfs
fi
mkdir -p rootfs

# 3. Download and Extract
echo "Downloading Alpine Linux..."
cd rootfs
wget "$URL"

echo "Extracting..."
tar -xf "$FILE"
rm "$FILE"

echo "Success! Alpine filesystem created in ./rootfs"