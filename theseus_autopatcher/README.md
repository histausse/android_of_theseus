# Android Theseus Patcher

This is mostly glueware between the [theseus frida](../frida) python package (used to get runtime information) and the [theseus patcher](../patcher) (rust binary used tp patch the apk).

This package embed  the patcher binary for ease of use. The embedded version is build for linux x86_64, statically linked to musl. For other target platform (windows, arm, ect), a different patcher binary can provided at runtime.

## Build

TODO: use nix build the project

Before building this package, the patcher binary must be built with the musl target. This require the `x86_64-unknown-linux-musl` to be installed, as well as `musl-gcc`:

```
rustup target add x86_64-unknown-linux-musl
doas pacman -S musl
```

Build the patcher:

```
cd ../patcher
cargo build --release --target=x86_64-unknown-linux-musl
cd -
```

Copy to patcher to the python directory:

```
cp ../patcher/target/x86_64-unknown-linux-musl/release/patcher src/theseus_autopatcher/patcher_86_64_musl
```

Build the package:

```
poetry build
```
