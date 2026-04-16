# Zmanim Calculator

A web application for calculating Jewish prayer times (Zmanim) with a Rust WASM backend.

## Clone

```bash
# Ensure git-lfs is installed first
# See: https://docs.github.com/en/repositories/working-with-files/managing-large-files/installing-git-large-file-storage
git lfs install

# Clone the repository
git clone https://github.com/mendymm/zman.mendy.dev.git
cd zman.mendy.dev
```

## Prerequisites

- **Rust** - Install via [rustup](https://rustup.rs/)
- **wasm-pack** - `cargo install wasm-pack`
- **Bun** - Install via [bun.sh](https://bun.sh/)
- **just** - `cargo install just`

## Quick Start

```bash
# Build everything and deploy
just deploy
```

## Development

```bash
# Build web bundle and serve with hot reload
just serve

# Or build manually
just build-web

# Open http://localhost:5173
```

## Commands

Run `just --list` to see all available commands:

| Command | Description |
|---------|-------------|
| `just regen-db` | Rebuild cities.db from source data |
| `just build-data` | Build client data files |
| `just build-wasm` | Build WASM package |
| `just build-web` | Build web bundle (includes WASM) |
| `just serve` | Build and serve locally with dev server |
| `just deploy` | Deploy to Cloudflare Pages |

## Project Structure

```
.
├── data/                  # Source data (git-lfs)
│   ├── all-cities.jsonl   # GeoNames cities data
│   ├── coords_to_elevation.json # mapping of coordinates to elevation data
│   └── admin1CodesASCII.txt
├── data-cli/              # Rust CLI tool
├── wasm-funcs/            # Rust WASM functions
├── frontend/              # SvelteKit web app
│   ├── src/
│   ├── static/
│   └── build/             # Build output
├── LICENSE.txt            # LGPL-3.0-or-later
└── justfile               # Build commands
```

## Data Sources

- City data: [GeoNames](https://geonames.org/)
- Elevation data: [OpenTopoData](https://opentopodata.org/)

## Acknowledgments

- Zmanim calculations: [rust-zmanim](https://github.com/YSCohen/rust-zmanim) by YSCohen
