setup:
    #!/bin/bash -eux
    rustup target add wasm32-unknown-unknown
    if ! cargo install --list | grep -q 'wasm-bindgen-cli v0.2.106'; then
        cargo install --force wasm-bindgen-cli --version=0.2.106 --locked
    fi
    if ! cargo install --list | grep -q 'cargo-leptos v0.3.2'; then
        cargo install --force cargo-leptos --version=0.3.2 --locked
    fi
    just setup_leptosfmt
    if ! cargo install --list | grep -q 'stylance-cli v0.7.4'; then
        cargo install --force stylance-cli --version=0.7.4 --locked
    fi
    if ! cargo install --list | grep -q 'sqlx-cli v0.8.6'; then
        cargo install --force sqlx-cli --version=0.8.6 --no-default-features --features postgres,native-tls --locked
    fi

setup_leptosfmt:
    #!/bin/bash -eux
    if ! cargo install --list | grep -q 'leptosfmt v0.1.33'; then
        cargo install --force leptosfmt --version=0.1.33 --locked
    fi

watch:
    #!/bin/bash -eux
    docker compose up -d
    stylance --watch ./app &
    RUST_BACKTRACE=1 cargo leptos watch --hot-reload &
    trap "kill %1; kill %2" EXIT
    wait

db-reset:
    docker compose down -v && docker compose up -d

# Format code
format:
    cargo fmt --all
    leptosfmt ./**/*.rs

# Lint and format checks
check-format:
    cargo fmt --all -- --check
    leptosfmt --check ./**/*.rs

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test --all-targets --all-features

# Run all CI checks
ci: check-format clippy test
