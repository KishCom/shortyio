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

# Convert PNG to ICNS if sips is available (macOS tool)
if command -v sips &> /dev/null && command -v iconutil &> /dev/null; then
    echo "Creating ICNS icon..."
    ICONSET="${RESOURCES_DIR}/icon.iconset"
    mkdir -p "${ICONSET}"

    # Generate different sizes for iconset
    sips -z 16 16     icon.png --out "${ICONSET}/icon_16x16.png" &> /dev/null
    sips -z 32 32     icon.png --out "${ICONSET}/icon_16x16@2x.png" &> /dev/null
    sips -z 32 32     icon.png --out "${ICONSET}/icon_32x32.png" &> /dev/null
    sips -z 64 64     icon.png --out "${ICONSET}/icon_32x32@2x.png" &> /dev/null
    sips -z 128 128   icon.png --out "${ICONSET}/icon_128x128.png" &> /dev/null
    sips -z 256 256   icon.png --out "${ICONSET}/icon_128x128@2x.png" &> /dev/null
    sips -z 256 256   icon.png --out "${ICONSET}/icon_256x256.png" &> /dev/null
    sips -z 512 512   icon.png --out "${ICONSET}/icon_256x256@2x.png" &> /dev/null
    sips -z 512 512   icon.png --out "${ICONSET}/icon_512x512.png" &> /dev/null
    cp icon.png "${ICONSET}/icon_512x512@2x.png"

    # Convert to ICNS
    iconutil -c icns "${ICONSET}" -o "${RESOURCES_DIR}/icon.icns"
    rm -rf "${ICONSET}"
    echo "  ✓ Icon created"
else
    echo "  Note: sips/iconutil not available, skipping icon (icon will be set by app at runtime)"
fi

# Create Info.plist
sed "s/VERSION/${VERSION}/g" Info.plist.template > "${CONTENTS_DIR}/Info.plist"

# Create PkgInfo file
echo -n "APPL????" > "${CONTENTS_DIR}/PkgInfo"

# Make binary executable
chmod +x "${MACOS_DIR}/${BINARY_NAME}"

echo "✓ ${APP_NAME}.app created successfully"
echo "  Location: ${APP_DIR}"
echo "  Version: ${VERSION}"
echo "  Target: ${TARGET}"
echo "  Bundle ID: ${BUNDLE_ID}"
