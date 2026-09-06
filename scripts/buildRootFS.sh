#!/usr/bin/env sh
set -e

cd "$(dirname "$0")"
cd ..

if [ "${PWD##*/}" != "textury" ]; then
    echo "Error: Expected project directory to be named 'textury'." >&2
    exit 1
fi

cd ..
if [ ! -d "textury" ]; then
  echo "No child directory 'textury' in $PWD"
fi
if [ ! -d "fireframe" ]; then
  echo "No child directory 'fireframe' in $PWD"
fi


ROOTFS_TAR_PATH="./textury/target/rootfs.tar"
ROOTFS_PATH="./textury/target/rootfs.ext4"

if [ ! -d "textury/target" ]; then
  mkdir "textury/target"
fi

docker build -f "textury/scripts/Guest.Dockerfile" --build-context textury=./textury --build-context fireframe=./fireframe -t textury-guest .

echo "Storing image to tar"
docker rm -f extract || :
rm -f "$ROOTFS_TAR_PATH" || :
docker create --name extract textury-guest
docker export extract -o "$ROOTFS_TAR_PATH"
docker rm -f extract

echo "Creating ext4"
rm -f $ROOTFS_PATH || :
fallocate -l 1500M $ROOTFS_PATH
mkfs.ext4 -d $ROOTFS_TAR_PATH $ROOTFS_PATH
