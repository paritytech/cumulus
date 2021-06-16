check: githooks
	./scripts/run.sh check --no-default-features --target=wasm32-unknown-unknown

check-tests: githooks
	./scripts/run.sh check --tests

test: githooks
	./scripts/run.sh test

GITHOOKS_SRC = $(wildcard githooks/*)
GITHOOKS_DEST = $(patsubst githooks/%, $(GITHOOK)/%, $(GITHOOKS_SRC))

GITHOOK := $(shell git rev-parse --git-path hooks)

$(GITHOOK):
	mkdir $(GITHOOK)

$(GITHOOK)/%: githooks/%
	cp "$^" "$(GITHOOK)"

githooks: $(GITHOOK) $(GITHOOKS_DEST)

init: githooks

format:
	./scripts/run.sh "fmt"


# Standalone development workflow targets
# Running those inside existing workspace will break due to Cargo unable to support nested worksapce

Cargo.toml: Cargo.dev.toml
	cp Cargo.dev.toml Cargo.toml

dev-format: Cargo.toml
	cargo fmt --all

dev-format-check: Cargo.toml
	cargo fmt --all -- --check

# needs to use run.sh to check individual projects because
#   --no-default-features is not allowed in the root of a virtual workspace
dev-check: Cargo.toml check

dev-check-tests: Cargo.toml
	cargo check --tests --all

dev-test: Cargo.toml
	cargo test --all --features runtime-benchmarks

# run benchmarks via Acala node
benchmark-all:
	cargo run --release --bin=acala --features=runtime-benchmarks -- benchmark --chain=dev --steps=50 --repeat=20 --pallet=orml_auction --extrinsic="*" --execution=wasm --wasm-execution=compiled --heap-pages=4096 --output=./auction/src/weights.rs --template ../templates/orml-weight-template.hbs

	cargo run --release --bin=acala --features=runtime-benchmarks -- benchmark --chain=dev --steps=50 --repeat=20 --pallet=orml_authority --extrinsic="*" --execution=wasm --wasm-execution=compiled --heap-pages=4096 --output=./authority/src/weights.rs --template ../templates/orml-weight-template.hbs

	cargo run --release --bin=acala --features=runtime-benchmarks -- benchmark --chain=dev --steps=50 --repeat=20 --pallet=module_currencies --extrinsic="*" --execution=wasm --wasm-execution=compiled --heap-pages=4096 --output=./currencies/src/weights.rs --template ../templates/orml-weight-template.hbs

	cargo run --release --bin=acala --features=runtime-benchmarks -- benchmark --chain=dev --steps=50 --repeat=20 --pallet=orml_oracle --extrinsic="*" --execution=wasm --wasm-execution=compiled --heap-pages=4096 --output=./oracle/src/weights.rs --template ../templates/orml-weight-template.hbs

	cargo run --release --bin=acala --features=runtime-benchmarks -- benchmark --chain=dev --steps=50 --repeat=20 --pallet=orml_tokens --extrinsic="*" --execution=wasm --wasm-execution=compiled --heap-pages=4096 --output=./tokens/src/weights.rs --template ../templates/orml-weight-template.hbs

	cargo run --release --bin=acala --features=runtime-benchmarks -- benchmark --chain=dev --steps=50 --repeat=20 --pallet=orml_vesting --extrinsic="*" --execution=wasm --wasm-execution=compiled --heap-pages=4096 --output=./vesting/src/weights.rs --template ../templates/orml-weight-template.hbs
