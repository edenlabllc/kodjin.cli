#!/bin/env bash
set -e

URL="https://edenlabllc-kodjin-cli.s3.eu-north-1.amazonaws.com/kodjin-cli"
VERSION="${1:-latest}"

OS=$(uname -s)
ARCH=$(uname -m)

if [ "${ARCH}" == "aarch64" ]; then
    ARCH="arm64"
fi

FILE_URL="${URL}/${VERSION}/kodjin-cli_${OS}_${ARCH}.tar.gz"
echo "Downloading ${FILE_URL}"
DOWNLOAD_PATH=/tmp/kodjin-cli.tar.gz

curl "${FILE_URL}" -o "${DOWNLOAD_PATH}"

TARGET_PATH="/usr/local/bin"
echo "Extracting binary to ${TARGET_PATH}"
sudo sh -c "mkdir -p ${TARGET_PATH} && tar xf ${DOWNLOAD_PATH} -C ${TARGET_PATH}"

SHELL_NAME=$(basename "${SHELL}")


case "${SHELL_NAME}" in
  bash*)
    echo "Installing bash completions"
    sudo sh -c "mkdir -p /etc/bash_completion.d && kodjin-cli generate-completions bash > /etc/bash_completion.d/kodjin-cli.bash"
    ;;
  fish*)
    echo "Installing fish completions"
    sudo sh -c "mkdir -p /etc/fish/completions && kodjin-cli generate-completions fish > /etc/fish/completions/kodjin-cli.fish"
    ;;
esac

echo "Installation finished!"
kodjin-cli --version
