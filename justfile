default: run

build-guest:
    cd guest && cargo build --release -p hyperlight-guest-bin && cargo build --release

run: build-guest
    cargo run
