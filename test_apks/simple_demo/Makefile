VERSION=34.0.0
SDK_TOOLS=$(HOME)/Android/Sdk
JAVA_PATH=/usr/lib/jvm/java-17-openjdk/bin
JAVAC=/usr/lib/jvm/java-17-openjdk/bin/javac
JAR=/usr/lib/jvm/java-17-openjdk/bin/jar
PYTHON=python3
APP=demo

PACKAGE=com.example.theseus
MAIN_ACTIVITY=MainActivity

JAVAC_ARGS =
D8_ARGS = 


VERSION_B=$(basename $(basename $(VERSION)))

pass=ahahah

export PATH := $(JAVA_PATH):$(PATH)

all: $(shell mkdir -p build)
all: clean build/$(APP).apk
signature_v1: clean build/$(APP).v1.apk

debug: JAVAC_ARGS += -g
debug: D8_ARGS += --debug
debug: all

test: all
	adb install build/$(APP).apk
	adb shell am start -n $(PACKAGE)/.$(MAIN_ACTIVITY)

build/%.v1signed.apk: ./build/%.unsigned.apk ./ToyKey.keystore
	jarsigner -verbose -keystore ./ToyKey.keystore -storepass $(pass) -keypass $(pass) -signedjar $@ $< SignKey

build/%.v1.apk: ./build/%.v1signed.apk
	$(SDK_TOOLS)/build-tools/$(VERSION)/zipalign -v -f 4 $< $@

build/deps.jar: $(shell find java/ -type f -regex ".*\.java" )
	mkdir -p ./build/jar/
	$(JAVAC) -d ./build/jar/ -classpath $(SDK_TOOLS)/platforms/android-$(VERSION_B)/android.jar $$(find java -type f -regex ".*\.java")
	$(JAR) cvf build/deps.jar -C build/jar/ .

# TODO: fix dep somehow? cannot find a way to use % or $* in (shell ..)
build/%/classes: build/deps.jar $(shell find java/ -type f -regex ".*\.java" )
	mkdir -p ./build/$*/classes
	$(JAVAC) $(JAVAC_ARGS) -d ./build/$*/classes -classpath build/deps.jar:$(SDK_TOOLS)/platforms/android-$(VERSION_B)/android.jar $$(find java/$*/ -type f -regex ".*\.java")

build/classes/classes: build/deps.jar build/inline/classes.dex $(shell find java/ -type f -regex ".*\.java" )
	mkdir -p ./build/classes/classes
	sed -i "s#    private static final String DEX =.*#    private static final String DEX = \"$$(base64 -w 0 build/inline/classes.dex)\";#" java/classes/com/example/theseus/Main.java
	$(JAVAC) $(JAVAC_ARGS) -d ./build/classes/classes -classpath build/deps.jar:$(SDK_TOOLS)/platforms/android-$(VERSION_B)/android.jar $$(find java/$*/ -type f -regex ".*\.java")
	rm build/classes/classes/com/example/theseus/Malicious.class

build/%/classes.dex: build/%/classes
	mkdir -p ./build/$*
	$(SDK_TOOLS)/build-tools/$(VERSION)/d8 $(D8_ARGS) --classpath $(SDK_TOOLS)/platforms/android-$(VERSION_B)/android.jar $(shell find build/$*/classes -type f -regex ".*\.class" -printf "'%p'\n") --output ./build/$*/

build/%.unsigned.apk: build/classes/classes.dex
	mkdir -p ./build/$*_files
	mv ./build/classes/classes.dex ./build/$*_files/classes.dex
	$(SDK_TOOLS)/build-tools/$(VERSION)/aapt package -v -f -M ./AndroidManifest.xml -I $(SDK_TOOLS)/platforms/android-$(VERSION_B)/android.jar -F $@ ./build/$*_files

build/%.v2aligned.apk: ./build/%.unsigned.apk ./ToyKey.keystore
	$(SDK_TOOLS)/build-tools/$(VERSION)/zipalign -v -f 4 $< $@

build/%.apk: ./build/%.v2aligned.apk
	$(SDK_TOOLS)/build-tools/$(VERSION)/apksigner sign -ks ./ToyKey.keystore --v2-signing-enabled true --in $< --out $@ --ks-pass pass:$(pass)

ToyKey.keystore :
	keytool -genkeypair -validity 1000 -dname "CN=SomeKey,O=SomeOne,C=FR" -keystore $@ -storepass $(pass) -keypass $(pass) -alias SignKey -keyalg RSA -v

clean:
	$(RM) -r build/*

clean_all: clean
	$(RM) ToyKey.keystore
