#!/bin/bash

# Naru Installer
# Detects OS and architecture, then downloads/installs the latest binary.

set -e

REPO="Luvion1/naru"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="naru"

# Colors for output
RED='\033[0-31m'
GREEN='\033[0-32m'
BLUE='\033[0-34m'
NC='\033[0m' # No Color

cat << "EOF"
      |\      _,,,---,,_
ZZZzz /,`.-'`'    -.  ;-;;,_
     |,4-  ) )-,_. ,\ (  `'-'
    '---''(_/--'  `-'\_)
EOF

printf "${BLUE}==>${NC} Installing Naru Secure Configuration Manager...\n"

# Check if cargo is available as fallback
if ! command -v cargo &> /dev/null; then
    NEED_CARGO=true
fi

# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux*)     PLATFORM="linux";;
    Darwin*)    PLATFORM="macos";;
    *)          PLATFORM="unknown";;
esac

if [ "$PLATFORM" == "unknown" ]; then
    printf "${RED}Error:${NC} Unsupported Operating System: ${OS}\n"
    exit 1
fi

# Try to download from GitHub Releases
printf "${BLUE}==>${NC} Fetching latest release from GitHub...\n"
LATEST_TAG=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    printf "${RED}Warning:${NC} No GitHub release found. Attempting to build from source...\n"
    if [ "$NEED_CARGO" = true ]; then
        printf "${RED}Error:${NC} Cargo not found. Please install Rust/Cargo to build from source.\n"
        exit 1
    fi
    cargo install --path .
    printf "${GREEN}Success!${NC} Naru has been installed via Cargo.\n"
    exit 0
fi

# Construct download filename
if [ "$PLATFORM" == "linux" ]; then
    DL_NAME="naru-linux-x64"
elif [ "$PLATFORM" == "macos" ]; then
    if [ "$ARCH" == "arm64" ]; then
        DL_NAME="naru-macos-arm64"
    else
        DL_NAME="naru-macos-x64"
    fi
fi

DL_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${DL_NAME}"

printf "${BLUE}==>${NC} Downloading ${DL_NAME} (${LATEST_TAG})...\n"
curl -L "$DL_URL" -o "$BINARY_NAME"
chmod +x "$BINARY_NAME"

printf "${BLUE}==>${NC} Moving binary to ${INSTALL_DIR} (may require sudo)...
"
if [ -w "$INSTALL_DIR" ]; then
    mv "$BINARY_NAME" "${INSTALL_DIR}/${BINARY_NAME}"
else
    sudo mv "$BINARY_NAME" "${INSTALL_DIR}/${BINARY_NAME}"
fi

printf "${GREEN}Success!${NC} Naru $(naru --version) has been installed to ${INSTALL_DIR}\n"
printf "Run 'naru --help' to get started.\n"
