# Build cleaned data with elevation and population
build-data:
    cargo run -r -p data-cli  -- build-admin1
    cargo run -r -p data-cli  -- build-data

# Build WASM package
build-wasm:
    wasm-pack build --target web --release wasm-funcs
    cp wasm-funcs/pkg/wasm_funcs_bg.wasm public/dist/
    brotli -f public/dist/wasm_funcs_bg.wasm

# build web bundle
build-web: build-wasm
    bun build --outdir ./public/dist/  --target=browser --minify js-src/app.ts --sourcemap=external

# Build the zip bundle for deployment
build-zip: build-web build-data
    rm -f bundle.zip && \
        cd public && \
        zip -r ../bundle.zip ./

