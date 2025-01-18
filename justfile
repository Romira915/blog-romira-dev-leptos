setup:
    #!/bin/bash -eux
    rustup target add wasm32-unknown-unknown
    cargo install wasm-bindgen-cli
    cargo install cargo-leptos
    cargo install leptosfmt
    cargo install stylance-cli

watch:
    #!/bin/bash -eux
    stylance --watch ./app &
    RUST_BACKTRACE=1 cargo leptos watch --hot-reload &
    trap "kill %1; kill %2" EXIT
    wait
