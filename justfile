setup:
    #!/bin/bash -eux
    rustup target add wasm32-unknown-unknown
    if ! cargo install --list | grep -q 'wasm-bindgen-cli v0.2.100'; then
        cargo install --force wasm-bindgen-cli --version=0.2.100 --locked
    fi
    if ! cargo install --list | grep -q 'cargo-leptos v0.2.27'; then
        cargo install --force cargo-leptos --version=0.2.27 --locked
    fi
    just setup_leptosfmt
    if ! cargo install --list | grep -q 'stylance-cli v0.5.4'; then
        cargo install --force stylance-cli --version=0.5.4 --locked
    fi

setup_leptosfmt:
    #!/bin/bash -eux
    if ! cargo install --list | grep -q 'leptosfmt v0.1.32'; then
        cargo install --force leptosfmt --version=0.1.32 --locked
    fi

watch:
    #!/bin/bash -eux
    stylance --watch ./app &
    RUST_BACKTRACE=1 cargo leptos watch --hot-reload &
    trap "kill %1; kill %2" EXIT
    wait
