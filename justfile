
[linux]
test:
    cargo test -p bitcoin-waila --target=x86_64-unknown-linux-gnu --all-features

[macos]
test:
    cargo test -p bitcoin-waila --target=aarch64-apple-darwin --all-features

test-nix:
    cargo test -p bitcoin-waila --target=aarch64-unknown-linux-gnu --all-features
