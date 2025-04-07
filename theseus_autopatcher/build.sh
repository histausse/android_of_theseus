#!/usr/bin/bash

#rustup target add x86_64-unknown-linux-musl
#doas pacman -S musl

FOLDER=$(dirname "$(realpath $0)")

env --chdir "${FOLDER}/../patcher" cargo build --profile minsizerelease --target=x86_64-unknown-linux-musl
cp "${FOLDER}/../patcher/target/x86_64-unknown-linux-musl/minsizerelease/patcher" "${FOLDER}/src/theseus_autopatcher/patcher_86_64_musl"

env --chdir "${FOLDER}" poetry build
