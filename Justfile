linux_aarch64 := 'aarch64-unknown-linux-gnu'

lint:
    just ensure_installed sort
    cargo fmt --all -- --check
    cargo sort --workspace --check
    cargo clippy --tests --workspace -- -D warnings

fmt:
    just ensure_installed sort
    cargo fmt
    cargo fix --allow-dirty
    cargo sort --workspace

cross:
    just ensure_installed cross
    cross build --target {{linux_aarch64}} --release

ensure_installed *args:
    #!/bin/bash
    cargo install --list | grep -q {{ args }}
    if [[ $? -ne 0 ]]; then
        echo "error: cargo-{{ args }} is not installed"
        exit 1
    fi