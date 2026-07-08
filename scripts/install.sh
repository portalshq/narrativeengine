#!/usr/bin/env bash

set -euo pipefail

###############################################################################
# Configuration
###############################################################################

REPO="YOUR_GITHUB_ORG/nap"
BINARY_NAME="nap"
VERSION="${VERSION:-latest}"

###############################################################################
# Utilities
###############################################################################

require() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "Error: '$1' is required but not installed."
        exit 1
    }
}

require curl
require chmod
require uname
require mktemp

###############################################################################
# Detect platform
###############################################################################

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64)
                TARGET="x86_64-unknown-linux-gnu"
                ;;
            aarch64|arm64)
                TARGET="aarch64-unknown-linux-gnu"
                ;;
            *)
                echo "Unsupported Linux architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64)
                TARGET="x86_64-apple-darwin"
                ;;
            arm64)
                TARGET="aarch64-apple-darwin"
                ;;
            *)
                echo "Unsupported macOS architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    *)
        echo "Unsupported operating system: $OS"
        exit 1
        ;;
esac

###############################################################################
# Download
###############################################################################

ASSET="${BINARY_NAME}-${TARGET}"

if [[ "$VERSION" == "latest" ]]; then
    URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
else
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
fi

echo "Installing ${BINARY_NAME}..."
echo "Platform : ${TARGET}"
echo "Version  : ${VERSION}"

TMP_FILE="$(mktemp)"

cleanup() {
    rm -f "$TMP_FILE"
}

trap cleanup EXIT

curl \
    --fail \
    --location \
    --progress-bar \
    "$URL" \
    --output "$TMP_FILE"

chmod +x "$TMP_FILE"

###############################################################################
# Install location
###############################################################################

INSTALL_DIR="/usr/local/bin"

if [[ ! -w "$INSTALL_DIR" ]]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

mv "$TMP_FILE" "$INSTALL_DIR/$BINARY_NAME"

###############################################################################
# PATH hint
###############################################################################

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    cat <<EOF

${INSTALL_DIR} is not on your PATH.

Add this to your shell profile:

export PATH="${INSTALL_DIR}:\$PATH"

EOF
fi

###############################################################################
# Verify
###############################################################################

echo
echo "Installed successfully."

"$INSTALL_DIR/$BINARY_NAME" --version