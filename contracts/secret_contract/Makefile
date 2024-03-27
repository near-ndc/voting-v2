# This is a local build with debug-prints activated. Debug prints only show up
# in the local development chain (see the `start-server` command below)
# and mainnet won't accept contracts built with the feature enabled.
.PHONY: build _build
build: _build compress-wasm
_build:
	RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown

# This is a build suitable for uploading to mainnet.
# Calls to `debug_print` get removed by the compiler.
.PHONY: build-mainnet _build-mainnet
build-mainnet: _build-mainnet compress-wasm
_build-mainnet:
	RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown

.PHONY: compress-wasm
compress-wasm:
	cp ../../target/wasm32-unknown-unknown/release/secret_contract.wasm ./contract.wasm
	@## The following line is not necessary, may work only on linux (extra size optimization)
	@wasm-opt -Os ./contract.wasm -o ./contract.wasm
	cat ./contract.wasm | gzip -9 > ./contract.wasm.gz

# Run local development chain with four funded accounts (named a, b, c, and d)
.PHONY: localsecret
localsecret: # CTRL+C to stop
	docker run -it --rm \
		-p 26657:26657 -p 1317:1317 -p 5000:5000 -p 9090:9090 -p 9091:9091 \
		-v $$(pwd):/root/code \
		--name localsecret ghcr.io/scrtlabs/localsecret:v1.8.0
