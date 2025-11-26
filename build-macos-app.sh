#!/bin/bash
# Build macOS .app bundle for Shorty

set -e

# Configuration
APP_NAME="Shorty"
BUNDLE_ID="systems.weedmark.shortyio"
VERSION=$(grep '^version' Cargo.toml | head -n1 | cut -d'"' -f2)
TARGET="${1:-x86_64-apple-darwin}"
BINARY_NAME="shortyio"

echo "Building ${APP_NAME}.app for ${TARGET}..."

# Build the binary if not already built
if [ ! -f "target/${TARGET}/release/${BINARY_NAME}" ]; then
    cargo build --release --target ${TARGET}
fi

# Create .app bundle structure
APP_DIR="${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

rm -rf "${APP_DIR}"
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

# Copy binary
cp "target/${TARGET}/release/${BINARY_NAME}" "${MACOS_DIR}/"

# Copy icon (macOS supports PNG directly, no need to convert to ICNS)
cp icon.png "${RESOURCES_DIR}/icon.icns"

# Create Info.plist
sed "s/VERSION/${VERSION}/g" Info.plist.template > "${CONTENTS_DIR}/Info.plist"

# Make binary executable
chmod +x "${MACOS_DIR}/${BINARY_NAME}"

echo "✓ ${APP_NAME}.app created successfully"
echo "  Location: ${APP_DIR}"
echo "  Version: ${VERSION}"
echo "  Target: ${TARGET}"
