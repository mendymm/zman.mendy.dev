# Build cleaned data with elevation and population
build-data:
    #!/usr/bin/env bash
    set -e
    if [ -f public/data/admin1.json.br ] && [ -f public/data/cities.jsonl.br ]; then
        echo "Data files already exist, skipping regeneration"
    else
        rm -f public/data/*
        cargo run -r -p data-cli -- build-admin1
        cargo run -r -p data-cli -- build-data
    fi

# Build WASM package
build-wasm:
    rm -f public/dist/*
    rm -f wasm-funcs/pkg/*
    wasm-pack build --target web --profile wasm-release wasm-funcs --no-opt --no-pack
    brotli -f wasm-funcs/pkg/wasm_funcs_bg.wasm
    cp wasm-funcs/pkg/wasm_funcs_bg.wasm.br public/dist/
    

# build web bundle
build-web: build-wasm
    bun build --outdir ./public/dist/  --target=browser --minify js-src/app.ts --sourcemap=external

# Build the zip bundle for deployment
build-zip: build-web build-data
    rm -f bundle.zip && \
        cd public && \
        zip -r ../bundle.zip ./

deploy: build-zip
    cargo run -r -p data-cli -- \
        deploy-cf-pages \
        --account-id $(op --account my.1password.ca read op://Personal/zman.mendy.dev/account_id) \
        --token $(op --account my.1password.ca read op://Personal/zman.mendy.dev/api_token)