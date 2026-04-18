#!/bin/bash
set -e

# posthog-cli-rs installer (Rust implementation of PostHog CLI)
# Usage: curl -sSL https://raw.githubusercontent.com/sapihav/posthog-cli-rs/main/install.sh | bash

REPO="sapihav/posthog-cli-rs"
BINARY_NAME="posthog"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Map uname output to Rust target triple.
detect_target() {
    local os arch
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *) error "Unsupported architecture: $arch" ;;
    esac

    case "$os" in
        darwin) TARGET="${arch}-apple-darwin" ;;
        linux)  TARGET="${arch}-unknown-linux-gnu" ;;
        *) error "Unsupported operating system: $os" ;;
    esac
}

get_latest_version() {
    if ! command -v jq &> /dev/null; then
        error "jq is required but not installed. Install with: brew install jq (macOS) or apt install jq (Linux)"
    fi
    LATEST_VERSION=$(curl -sS "https://api.github.com/repos/${REPO}/releases/latest" | jq -r '.tag_name // empty')
    if [ -z "$LATEST_VERSION" ]; then
        error "Failed to get latest version. Check your internet connection or GitHub API rate limits."
    fi
    if ! echo "$LATEST_VERSION" | grep -qE '^v[0-9]+\.[0-9]+\.[0-9]+'; then
        error "Unexpected version format: ${LATEST_VERSION}"
    fi
}

install() {
    detect_target
    get_latest_version

    info "Installing posthog-cli-rs ${LATEST_VERSION} for ${TARGET}..."

    ARCHIVE_NAME="${BINARY_NAME}-${TARGET}.tar.gz"
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_VERSION}/${ARCHIVE_NAME}"
    CHECKSUM_URL="${DOWNLOAD_URL}.sha256"

    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT

    info "Downloading from ${DOWNLOAD_URL}..."
    if ! curl -sSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${ARCHIVE_NAME}"; then
        error "Failed to download. Release may not exist yet."
    fi

    info "Downloading checksum..."
    if ! curl -sSL "$CHECKSUM_URL" -o "${TMP_DIR}/${ARCHIVE_NAME}.sha256"; then
        error "Failed to download checksum. Cannot verify integrity."
    fi

    info "Verifying checksum..."
    cd "$TMP_DIR"
    EXPECTED=$(awk '{print $1}' "${ARCHIVE_NAME}.sha256")
    if [ -z "$EXPECTED" ]; then
        error "No checksum found in ${ARCHIVE_NAME}.sha256"
    fi
    if command -v sha256sum &> /dev/null; then
        ACTUAL=$(sha256sum "${ARCHIVE_NAME}" | awk '{print $1}')
    elif command -v shasum &> /dev/null; then
        ACTUAL=$(shasum -a 256 "${ARCHIVE_NAME}" | awk '{print $1}')
    else
        error "No SHA-256 tool found. Install coreutils (sha256sum) or shasum."
    fi
    if [ "$EXPECTED" != "$ACTUAL" ]; then
        error "Checksum mismatch!\n  Expected: ${EXPECTED}\n  Got:      ${ACTUAL}\nThe download may be corrupted or tampered with."
    fi
    info "Checksum verified."

    info "Extracting..."
    tar -xzf "$ARCHIVE_NAME"

    if [ -w "$INSTALL_DIR" ]; then
        mv "$BINARY_NAME" "$INSTALL_DIR/"
    else
        warn "${INSTALL_DIR} is not writable. Run with sudo or set INSTALL_DIR to a writable path:"
        warn "  INSTALL_DIR=\$HOME/.local/bin bash install.sh"
        error "Cannot install without write access to ${INSTALL_DIR}"
    fi

    if command -v "$BINARY_NAME" &> /dev/null; then
        info "Successfully installed posthog-cli-rs!"
        echo ""
        "$BINARY_NAME" --version
        echo ""
        info "Run 'posthog --help' to get started"
        info "Run 'posthog login' to authenticate"
    else
        warn "Installation complete, but '$BINARY_NAME' not found in PATH"
        warn "You may need to add ${INSTALL_DIR} to your PATH"
        echo ""
        echo "Add this to your shell profile:"
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi
}

install
