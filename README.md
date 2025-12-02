# TouchRelay

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

> Transform your smartphone into a wireless touchpad and keyboard for your Windows PC

TouchRelay is a lightweight, self-contained application that allows you to control your Windows computer's mouse and keyboard from any device with a web browser over your local network.

## ✨ Features

- **🖱️ Full Mouse Control** - Move, click, double-click, right-click, and scroll
- **⌨️ Text Input** - Send text directly to your PC with Enter key support
- **⚙️ Adjustable Sensitivity** - Customize movement speed (0.5x - 3.0x)
- **🚀 Zero Configuration** - Single executable, all assets embedded
- **💫 System Tray** - Runs silently in background with quick access menu

## 📋 Requirements

- **Server**: Windows 10/11 (64-bit)
- **Client**: Any modern web browser
- **Network**: Same local network

## 🚀 Quick Start

### Download & Run

1. Download `touch-relay.exe` from [Releases](https://github.com/DeltaFoundry/TouchRelay/releases)
2. Double-click to start - it runs in system tray
3. Find your PC's IP: press `Win + R`, type `cmd`, then `ipconfig`

### Connect from Mobile

1. Open browser on your phone
2. Go to `http://<PC_IP>:8000/` (e.g., `http://192.168.1.100:8000/`)
3. Start controlling!

### Controls

- **Move mouse**: Drag one finger
- **Left click**: Tap once
- **Double click**: Tap twice quickly
- **Right click**: Tap with two fingers
- **Scroll**: Swipe with two fingers
- **Send text**: Type and press Send

## 🛠️ Building from Source

```bash
# Clone and build
git clone https://github.com/DeltaFoundry/TouchRelay.git
cd TouchRelay
cargo build --release

# Executable at: target/release/touch-relay.exe
```

**Prerequisites**: Rust 1.70+ and Visual Studio Build Tools

## 🔒 Security

⚠️ **No authentication** - Only use on trusted local networks. Do not expose to the internet.

## 🐛 Troubleshooting

**Can't connect?**
- Ensure same WiFi network
- Check Windows Firewall for port 8000
- Test locally first: `http://localhost:8000`

**Mouse not responding?**
- Try running as Administrator
- Check connection status in web interface

**Sensitivity issues?**
- Adjust the slider in web interface (saved automatically)

## 📝 License

MIT License - see [LICENSE](LICENSE) file

## 🤝 Contributing

Contributions welcome! Fork, create a feature branch, and submit a PR.

## 📧 Support

- **Issues**: [GitHub Issues](https://github.com/DeltaFoundry/TouchRelay/issues)
- **Discussions**: [GitHub Discussions](https://github.com/DeltaFoundry/TouchRelay/discussions)

---

**Made with ❤️ by [DeltaFoundry](https://github.com/DeltaFoundry)**

⭐ Star this repo if you find it useful!
