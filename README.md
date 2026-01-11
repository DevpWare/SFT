# DevpWareSoft v2.0

Dependency analyzer for multi-language projects with 3D visualization.

## Requirements

### Windows

1. **Rust** - https://rustup.rs/
2. **Node.js** - https://nodejs.org/ (LTS)
3. **Visual Studio Build Tools** - https://visualstudio.microsoft.com/visual-cpp-build-tools/
   - Instalar "Desktop development with C++"
4. **WebView2** - Ya viene en Windows 10/11 moderno

### Linux

1. **Rust** - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. **Node.js** - https://nodejs.org/
3. **Dependencias del sistema**:
   ```bash
   # Arch
   sudo pacman -S webkit2gtk-4.1 base-devel

   # Ubuntu/Debian
   sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev
   ```

## Setup

```bash
npm install
```

## Development

```bash
npm run tauri dev
```

## Build

```bash
npm run tauri build
```
