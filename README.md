# WSL Manager

A full-featured WSL (Windows Subsystem for Linux) management GUI built with **Iced 0.13** and **Rust**.

![Language](https://img.shields.io/badge/Language-Rust-orange)
![Framework](https://img.shields.io/badge/Framework-Iced_0.13-blue)
![Platform](https://img.shields.io/badge/Platform-Windows_10/11-brightgreen)
![License](https://img.shields.io/badge/License-MIT-yellow)

## Features

- **Overview**: View all installed WSL distros with running/stopped status
- **Distro Management**: Start, stop, restart, delete, rename, import, export distros
- **WSL Version Switch**: Toggle between WSL1 and WSL2 for any distro
- **Config Editors**:
  - `.wslconfig` — Edit global WSL settings (memory, processors, swap, etc.)
  - `wsl.conf` — Edit per-distro settings (systemd, automount, networking, etc.)
- **System Monitor**: View memory, disk usage, and process count per running distro
- **Quick Actions**: Open terminal, Explorer, or VS Code directly in a distro
- **Install Distro**: Install from Microsoft Store (online) or from a local tar file
- **Bilingual UI**: English and Chinese language support

## Screenshots

| Overview | Distro Detail | Config Editor |
|----------|---------------|---------------|
| ![Overview](screenshots/overview.png) | ![Detail](screenshots/detail.png) | ![Config](screenshots/config.png) |

## Prerequisites

- **Windows 10/11** with WSL enabled
- **VS 2022 Build Tools** (install via `winget install Microsoft.VisualStudio.2022.BuildTools`)
- **Rust stable toolchain** (install via [rustup-init.exe](https://rustup.rs))

## Build

```bash
# Activate MSVC environment and build
cmd /c "call ""C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat"" x64 >nul 2>&1 && cargo build --release"
```

The binary will be at `target/release/wsl-manager.exe`.

> Use `CARGO_BUILD_JOBS=2` if your page file is small.

## Usage

Simply run `wsl-manager.exe` — no installation needed. The GUI will automatically detect all installed WSL distros.

- Click a distro in the sidebar to select it
- Use the sidebar buttons to switch between tabs (Overview, Monitor, .wslconfig, wsl.conf, Log)
- If no distro is selected when opening the wsl.conf editor, the default distro is auto-selected

## Project Structure

```
wslmanager/
├── Cargo.toml
├── LICENSE
├── README.md
├── .gitignore
├── favicon48_ico/
│   └── favicon48.ico
└── src/
    ├── main.rs       # Entry point, wires Iced application
    ├── app.rs        # MVU core: state, messages, view functions, styles
    ├── wsl.rs        # WSL command execution, output parsing, config I/O
    ├── monitor.rs    # System metrics collection via WSL commands
    └── i18n.rs       # English/Chinese translation strings
```

## Architecture

- **Iced 0.13** MVU (Model-View-Update) architecture
- All WSL commands run asynchronously via `tokio`
- UTF-16LE output from `wsl.exe` is decoded automatically
- `CREATE_NO_WINDOW` flag prevents console window flash
- Dark theme only (`Theme::Dark`)

## Key Technical Details

- `wsl.conf` is at `/etc/wsl.conf` inside each distro — requires `sudo cp` via WSL
- `.wslconfig` is at `%USERPROFILE%\.wslconfig` on the Windows side
- Rename operation uses export → unregister → import (3-step process)
- Distro names with spaces are handled correctly in the parser

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.
