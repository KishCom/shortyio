# Shortyio

A lightning-fast GUI application for creating short.io URLs with custom paths.

## Features

- **Lightning Fast**: Built with Rust and egui for instant startup and response
- **Auto-Paste**: Automatically detects URLs in your clipboard on startup
- **Persistent Config**: API key and domain saved locally for quick reuse
- **Custom Paths**: Create memorable short links with custom paths
- **One-Click Copy**: Copy shortened URLs to clipboard instantly
- **Cross-Platform**: Works on Linux, macOS, and Windows

## Installation

### From Pre-built Binaries

Download the latest release for your platform from the [Releases](../../releases) page.

### From Source

```bash
git clone https://github.com/KishCom/shortyio.git
cd shortyio
cargo build --release
./install-linux.sh  # Optional, for icon support on Linux
```

The binary will be at `target/release/shortyio`

## Usage

1. Launch the application
2. Enter your [short.io](https://short.io) API key (get one from your short.io dashboard)
3. Optionally enter your custom domain
4. The Original URL field will auto-fill if you have a URL in your clipboard
5. Optionally add a custom path for your short link
6. Press Enter or click "Create Short Link"
7. Click "Copy" to copy the shortened URL to your clipboard

#### macOS "Installation"

1. Download `shortyio-Darwin-*.zip` for your architecture (Intel/x86_64 or Apple Silicon/aarch64)
2. Extract the zip file
3. Drag `Shorty.app` to your Applications folder
4. **First launch**: Right-click `Shorty.app` → select "Open" → click "Open" in the dialog
   - This is required because the app is not signed with an Apple Developer certificate
   - After the first launch, you can open it normally
5. Launch Shorty from Applications or Spotlight

## Workflow

The fastest way to use Shorty:

1. Copy any URL (Ctrl+C / Cmd+C)
2. Launch Shorty
3. Press Enter (or add custom path + Enter)
4. Click Copy
5. Done!

## Configuration

Configuration is stored in:
- **Linux**: `~/.config/shortyio/config.json`
- **macOS**: `~/Library/Application Support/systems.weedmark.shortyio/config.json`
- **Windows**: `%APPDATA%\weedmark\shortyio\config.json`

The config file stores:
- `api_key`: Your short.io API key
- `domain`: Your custom domain (optional)

## Requirements

- A [short.io](https://short.io) account and API key

## Linux Wayland/X11 Notes

Wayland support is enabled by default. It relies on the data-control protocol extension(s), which are not supported by all Wayland compositors. You can check compositor support on wayland.app:

- [ext-data-control-v1](https://wayland.app/protocols/ext-data-control-v1)
- [wlr-data-control-unstable-v1](https://wayland.app/protocols/wlr-data-control-unstable-v1)

If you or a user's desktop doesn't support these protocols, shortyio won't be able to automatically pick up the URL on a clipboard in a pure Wayland environment. It is recommended to enable XWayland for these cases. If your're running shortyio inside an isolated sandbox, such as Flatpak or Snap, you'll need to expose the X11 socket to the application in addition to the Wayland communication interface.

For better icon support on Wayland, and various desktops app menu integration, use the install script. This script merely installs the app to `~/.local/bin` (or `/usr/local/bin`), adds a desktop entry, and registers the app icon:

```bash
cd shortyio-Linux-x86_64
chmod +x install-linux.sh
./install-linux.sh
```

## Building

Requirements:
- Rust 1.70 or newer
- System dependencies for egui (usually pre-installed on most systems)

```bash
cargo build --release
```

## License

MIT
