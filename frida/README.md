# Theseus Data Collector

Collect runtime information about reflection operation done by en application to feed them to the patcher.


# Modifying StackConsumer.java

The Frida hook uses a Java class to collect the stack information. If you modify it, before building/installing the python package, you need to build regenerate the base64 encoded class used by frida:

```shell

# If the default values do not match, set the variables:
#
# JAVAC: path to the java compiler
# ANDROID_SDK: path to the android sdk
# VERSION: Android SDK version to use
# D8: path to the d8 executable
# ANDROID_JAR: path to the android.jar file to link
# BUILD_F: build folder (will be delated if exist)
# OUT_FILE: the file where to put the b64 of the compiled dex

bash consumer/build.sh

# Then you can build the package / install it
poetry build # / poetry install / pip install .
```
