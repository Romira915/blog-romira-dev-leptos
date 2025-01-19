setup:
    #!/bin/bash -eux
    rustup target add wasm32-unknown-unknown
    cargo install --force wasm-bindgen-cli --version=0.2.100 --locked
    cargo install --force cargo-leptos --version=0.2.24 --locked
    cargo install --force leptosfmt
    cargo install --force stylance-cli --version=0.5.2 --locked

watch:
    #!/bin/bash -eux
    stylance --watch ./app &
    RUST_BACKTRACE=1 cargo leptos watch --hot-reload &
    trap "kill %1; kill %2" EXIT
    wait
