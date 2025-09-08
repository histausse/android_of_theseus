# Test APKs

## Requirements to build

- Openjdk 17 at `/usr/lib/jvm/java-17-openjdk`
- Android SDK at `$(HOME)/Android/Sdk` with `build-tools;34.0.0`, `platform-tools` and `platforms;android-34`

## Demo

Build the demo:

```
cd simple_demo/
make
```

### Flowdroid:

Get Flowdroid from https://github.com/secure-software-engineering/FlowDroid

Run flow analysis:

- `./simple_demo/build/tests.apk` is the apk
- `~/Android/Sdk/platforms/` is the platform directory, it must contains `android-34/android.jar` (if not, `sdkmanager platforms;android-34`)
- `-r`: "Enable support for reflective method calls"
- `./simple_demo/source_sink.txt` contains the sources and sinks for our demo app

```
java -jar soot-infoflow-cmd-jar-with-dependencies.jar -a ./simple_demo/build/tests.apk -p ~/Android/Sdk/platforms/ -r -s ./simple_demo/source_sink.txt

```

## Filtering logs:

```
adb logcat -s THESEUS
```
