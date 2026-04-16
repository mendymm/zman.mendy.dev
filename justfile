# Build the cities.db sqlite database from data files
regen-db:
    cargo run -r -p data-cli -- regen-sqlite

# Build cleaned data with elevation and population
build-data:
    #!/usr/bin/env bash
    cargo run -r -p data-cli -- build-admin1
    cargo run -r -p data-cli -- build-data


# Build WASM package
build-wasm:
    wasm-pack build --target web --profile wasm-release wasm-funcs --no-opt --no-pack --out-dir ../frontend/src/lib/wasm-gen-output
    

# build web bundle
build-web: build-wasm build-data
    #!/usr/bin/env bash
    cd frontend
    bun run build

serve: build-web
    #!/usr/bin/env bash
    cd frontend
    bun run dev

deploy: build-web
    # Set environment variables
    CLOUDFLARE_API_TOKEN=$(op --account my.1password.ca read op://Personal/zman.mendy.dev/api_token) \
    CLOUDFLARE_ACCOUNT_ID=$(op --account my.1password.ca read op://Personal/zman.mendy.dev/account_id) \
    bunx wrangler@4.82.2 pages deploy ./frontend/build --project-name=zman