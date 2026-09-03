#!/bin/env bash
set -e

URL="https://edenlabllc-kodjin-cli.s3.eu-north-1.amazonaws.com/kodjin-cli"
VERSION="${1:-latest}"
# Version-pinned artifacts are stored under the bare semver, e.g. kodjin-cli/0.2.0,
# while release tags are v-prefixed. Accept both forms.
VERSION="${VERSION#v}"

OS=$(uname -s)
ARCH=$(uname -m)

if [ "${ARCH}" == "aarch64" ]; then
    ARCH="arm64"
fi

FILE_URL="${URL}/${VERSION}/kodjin-cli_${OS}_${ARCH}.tar.gz"
echo "Downloading ${FILE_URL}"
DOWNLOAD_PATH=/tmp/kodjin-cli.tar.gz

curl "${FILE_URL}" -o "${DOWNLOAD_PATH}"

TARGET_PATH="${HOME}"/.local/bin
echo "Extracting binary to ${TARGET_PATH}"
sudo sh -c "mkdir -p ${TARGET_PATH} && tar xf ${DOWNLOAD_PATH} -C ${TARGET_PATH}"

# Linux case, you need to create a symlink manually due to
# restrictions on permissions in the /usr/local/bin directory
if [[ ! -f /usr/local/bin/kodjin-cli ]]; then
  if ! (ln -s "${TARGET_PATH}"/kodjin-cli /usr/local/bin/kodjin-cli &> /dev/null) then
    printf "\nWARNING: The symlink was not created automatically, please complete the installation by running the command: %s\n" \
      "sudo ln -s ${TARGET_PATH}/kodjin-cli /usr/local/bin/kodjin-cli"
    exit 0
  fi
fi

echo "Installing shell completions"
kodjin-cli generate-completions --install || true

echo "Installation finished!"
kodjin-cli --version
