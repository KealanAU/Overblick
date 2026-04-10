# Överblick

A macOS developer dashboard built with Tauri + Vue 3. Tracks GitHub activity, commits, and time — all in one place.

## Install

Pre-built binaries are available on the [Releases](https://github.com/KealanAU/Overblick/releases) page.

Download the `.dmg` for macOS, open it, and drag Överblick to your Applications folder.

## Build from Source

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Tauri CLI: `cargo install tauri-cli`

### Setup

```bash
git clone https://github.com/KealanAU/Overblick.git
cd Overblick
npm install
```

### Development

```bash
npm run tauri dev
```

### Production Build

```bash
npm run tauri build
```

The output will be in `src-tauri/target/release/bundle/`. On macOS this produces a `.dmg` and `.app`.
