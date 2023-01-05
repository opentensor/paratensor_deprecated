FROM ubuntu:22.10 AS base
SHELL ["/bin/bash", "-c"]

# This is being set so that no interactive components are allowed when updating.
ARG DEBIAN_FRONTEND=noninteractive

LABEL ai.opentensor.image.authors="operations@opentensor.ai" \
        ai.opentensor.image.vendor="Opentensor Foundation" \
        ai.opentensor.image.title="opentensor/paratensor" \
        ai.opentensor.image.description="Opentensor Paratensor Blockchain"

# show backtraces
ENV RUST_BACKTRACE 1

# install tools and dependencies
RUN apt-get update && \
        DEBIAN_FRONTEND=noninteractive apt-get install -y \
                build-essential \
                git make clang curl \
                libssl-dev llvm libudev-dev protobuf-compiler \
                curl && \
# apt cleanup
        apt-get autoremove -y && \
        apt-get clean && \
        find /var/lib/apt/lists/ -type f -not -name lock -delete;

# Install cargo and Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN apt remove -y curl

RUN rustup default stable
RUN rustup update

RUN rustup update nightly
RUN rustup target add wasm32-unknown-unknown --toolchain nightly



FROM base as validator

ARG POLKADOT_VERSION

RUN git clone --branch ${POLKADOT_VERSION} https://github.com/paritytech/polkadot.git
RUN cd polkadot && cargo b -r

EXPOSE 30333 9933 9944



FROM base as collator

RUN mkdir /root/paratensor
COPY ./ /root/paratensor/
RUN cd /root/paratensor && cargo build --release

EXPOSE 30333 9933 9944
