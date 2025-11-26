#!/bin/bash
# Install script for Shorty on Linux

set -e

echo "Installing Shorty..."

# Create directories
mkdir -p ~/.local/share/applications
mkdir -p ~/.local/share/icons/hicolor/512x512/apps

# Copy files
cp shortyio ~/.local/bin/shortyio 2>/dev/null || sudo cp shortyio /usr/local/bin/shortyio
cp icon.png ~/.local/share/icons/hicolor/512x512/apps/systems.weedmark.shortyio.png
cp systems.weedmark.shortyio.desktop ~/.local/share/applications/systems.weedmark.shortyio.desktop

# Update desktop file to use absolute path if needed
if [ -f ~/.local/bin/shortyio ]; then
    sed -i "s|Exec=shortyio|Exec=$HOME/.local/bin/shortyio|g" ~/.local/share/applications/systems.weedmark.shortyio.desktop
fi

# Update icon cache
gtk-update-icon-cache ~/.local/share/icons/hicolor/ 2>/dev/null || true

echo "✓ Shorty installed successfully!"
echo "You can now launch it from your application menu or run 'shortyio' from terminal"
