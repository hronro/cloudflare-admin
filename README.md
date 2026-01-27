# Cloudflare DNS Manager

A fast, lightweight desktop application for managing Cloudflare DNS records.

![Cloudflare DNS Manager](assets/icon/icon.svg)

## Features

- **Zone Management** - Switch between multiple Cloudflare domains
- **DNS Record Operations** - Create, edit, and delete DNS records
- **Full Record Type Support** - A, AAAA, CNAME, MX, TXT, NS, SRV, CAA, and more
- **Proxy Toggle** - Enable/disable Cloudflare proxy per record
- **Dark Mode** - Light, dark, and system-following themes
- **Secure Storage** - API tokens stored in your system's keychain

## Why This App?

### Native Performance

Built entirely in Rust with GPU-accelerated rendering. No Electron, no web technologies, no HTML/CSS/JS runtime overhead. The app launches instantly and responds immediately to every interaction.

### Lightweight

The entire application is a single small binary. Minimal memory footprint and CPU usage compared to browser-based alternatives.

### Secure

- Written in Rust, eliminating entire classes of memory safety vulnerabilities
- API tokens stored securely in your operating system's native keychain (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux)
- No telemetry, no analytics, no network requests except to Cloudflare's API

## Installation

### From Source

Requires Rust 1.75 or later.

```bash
git clone https://github.com/yourusername/cloudflare-admin.git
cd cloudflare-admin
cargo build --release
```

The binary will be available at `target/release/cloudflare-admin`.

## Getting Started

1. **Get your Cloudflare API Token**
   - Log in to [Cloudflare Dashboard](https://dash.cloudflare.com)
   - Go to **My Profile** → **API Tokens**
   - Create a token with **Zone:DNS:Edit** permissions for your zones

2. **Launch the app** and paste your API token when prompted

3. **Select a zone** from the dropdown to view and manage its DNS records

## Usage

### Managing DNS Records

- Click on any record in the list to edit it
- Use the form on the right to modify record details
- Click **Save** to update or **Create** to add new records
- Click the delete button on a record to remove it

### Settings

Access settings via the gear icon to:
- Update your API token
- Switch between light/dark/auto themes
- Clear stored credentials

## System Requirements

- **macOS** 11.0 (Big Sur) or later
- **Windows** 10 or later
- **Linux** with Wayland or X11

## License

MIT License

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
