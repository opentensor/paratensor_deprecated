SHELL:=/bin/bash

build:
	cargo build

build-release:
	cargo build --release	

build-validator:
	DOCKER_BUILDKIT=1 docker build \
		--file Dockerfile \
		--target validator \
		--build-arg POLKADOT_VERSION=v0.9.10 \
		--tag opentensor/parachain/validator \
		.

build-collator:
	DOCKER_BUILDKIT=1 docker build \
		--file Dockerfile \
		--target collator \
		--tag opentensor/parachain/collator \
		.

up-and-scale-testnet:
	docker compose up -d --scale validator=2 --scale collator=1
