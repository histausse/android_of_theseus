# Android of Theseus

This is the code implementing the method presented in chapter 5 of the thesis 'The Woes of Android Reverse Engineering: from Large Scale Analysis to Dynamic Deobfuscation', by Jean-Marie Mineau.

The idea is collecting dynamic data like reflection calls and dynamic code loading using Frida, then patch the application to include this data statically. The application can then be analyse with any static analysis tools taking an application as input.

## Install

```
python -m venv venv
source venv/bin/activate
pip install ./theseus-autopatcher
```

## Run

```
theseus-autopatch -a test_dynloading.apk -o patched.apk -k keystore.ks --keypass 'P@ssw0rd!'
```

 Note: `theseus-autopatch` embed a patcher binary that will only work on x86_64 linux computer, en even then, the binary is optimized for size instead of speed. You should probably build your own patcher binary for your own architecture and pass it to `theseus-autopatch` with `--patch`:

```
cd patcher
cargo build --release
theseus-autopatch -a test_dynloading.apk -o patched.apk -k keystore.ks --keypass 'P@ssw0rd!' --patch target/release/patcher
```
