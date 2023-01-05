SHELL:=/bin/bash

build:
	cargo build

build-release:
	cargo build --release	

build-validator:
	DOCKER_BUILDKIT=1 docker build \
		--file Dockerfile \
		--target validator \
		--build-arg POLKADOT_VERSION=v0.9.26 \
		--tag opentensor/parachain/validator \
		.

build-collator:
	DOCKER_BUILDKIT=1 docker build \
		--file Dockerfile \
		--target collator \
		--tag opentensor/parachain/collator \
		.

up-and-scale-testnet:
	docker compose up -d --scale validator=4 --scale collator=2
