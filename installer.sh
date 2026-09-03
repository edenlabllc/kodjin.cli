#!/usr/bin/env bash
set -Eeuo pipefail

readonly S3_URL="https://edenlabllc-kodjin-cli.s3.eu-north-1.amazonaws.com/kodjin-cli"
VERSION="${1:-latest}"
# GoReleaser stores versioned artifacts without the tag's leading `v`.
readonly VERSION="${VERSION#v}"
readonly LOCAL_BIN_DIR="${HOME}/.local/bin"
readonly BINARY_PATH="${LOCAL_BIN_DIR}/kodjin-cli"

case "$(uname -s)" in
  Darwin) OS="Darwin" ;;
  Linux) OS="Linux" ;;
  *)
    printf 'ERROR: Unsupported operating system: %s\n' "$(uname -s)" >&2
    exit 1
    ;;
esac

case "$(uname -m)" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="arm64" ;;
  *)
    printf 'ERROR: Unsupported architecture: %s\n' "$(uname -m)" >&2
    exit 1
    ;;
esac

readonly ARCHIVE_NAME="kodjin-cli_${OS}_${ARCH}.tar.gz"
readonly FILE_URL="${S3_URL}/${VERSION}/${ARCHIVE_NAME}"
TEMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/kodjin-cli.XXXXXX")"
readonly TEMP_DIR
trap 'rm -rf "${TEMP_DIR}"' EXIT

printf 'Downloading %s\n' "${FILE_URL}"
curl --fail --location --retry 3 --retry-delay 1 \
  --output "${TEMP_DIR}/${ARCHIVE_NAME}" \
  "${FILE_URL}"

tar -xzf "${TEMP_DIR}/${ARCHIVE_NAME}" -C "${TEMP_DIR}"
if [[ ! -f "${TEMP_DIR}/kodjin-cli" ]]; then
  printf 'ERROR: The downloaded archive does not contain kodjin-cli.\n' >&2
  exit 1
fi

mkdir -p "${LOCAL_BIN_DIR}"
install -m 0755 "${TEMP_DIR}/kodjin-cli" "${BINARY_PATH}"

printf 'Installing shell completions\n'
"${BINARY_PATH}" generate-completions --install || \
  printf 'WARNING: Shell completions could not be installed.\n' >&2

printf 'Installation finished: '
"${BINARY_PATH}" --version

case ":${PATH}:" in
  *":${LOCAL_BIN_DIR}:"*) ;;
  *)
    printf '\nWARNING: %s is not in PATH.\n' "${LOCAL_BIN_DIR}" >&2
    printf 'Add this line to your shell configuration:\n  export PATH="$HOME/.local/bin:$PATH"\n' >&2
    ;;
esac

RESOLVED_BINARY="$(command -v kodjin-cli 2>/dev/null || true)"
if [[ -n "${RESOLVED_BINARY}" && "${RESOLVED_BINARY}" != "${BINARY_PATH}" ]]; then
  if [[ ! -L "${RESOLVED_BINARY}" || "$(readlink "${RESOLVED_BINARY}")" != "${BINARY_PATH}" ]]; then
    printf '\nWARNING: Your shell currently resolves kodjin-cli to another installation:\n  %s\n' \
      "${RESOLVED_BINARY}" >&2
    printf 'Run `type -a kodjin-cli`, remove the obsolete installation, and then run `rehash`.\n' >&2
  fi
fi
