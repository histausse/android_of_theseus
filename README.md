# Test

## Install

```
python -m venv venv
source venv/bin/activate
pip install 'theseus-autopatcher[grodd] @ git+ssh://git@gitlab.inria.fr/androidoftheseus/android-of-theseus.git#subdirectory=theseus_autopatcher/'
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
