VERSION=34.0.0
VERSION_B=$(basename $(basename $(VERSION)))
ANDROID_HOME ?= $(HOME)/Android/Sdk
SDK_TOOLS=$(ANDROID_HOME)
JAVA_PATH=/usr/lib/jvm/java-17-openjdk/bin
JAVAC=$(JAVA_PATH)/javac
JAR=$(JAVA_PATH)/jar
JARSIGNER=$(JAVA_PATH)/jarsigner
KEYTOOL=$(JAVA_PATH)/keytool

ADB=$(SDK_TOOLS)/platform-tools/adb
D8=$(SDK_TOOLS)/build-tools/$(VERSION)/d8
AAPT=$(SDK_TOOLS)/build-tools/$(VERSION)/aapt
ZIPALIGN=$(SDK_TOOLS)/build-tools/$(VERSION)/zipalign
APKSIGNER=$(SDK_TOOLS)/build-tools/$(VERSION)/apksigner

APP=test_dyn_and_ref
PACKAGE=com.example.theseus.dynandref
MAIN_ACTIVITY=MainActivity

JAVAC_ARGS =
D8_ARGS = 

pass=ahahah

export PATH := $(JAVA_PATH):$(PATH)
export ANDROID_HOME := $(SDK_TOOLS)

all: $(shell mkdir -p build)
all: clean build/$(APP).apk
signature_v1: clean build/$(APP).v1.apk

debug: JAVAC_ARGS += -g
debug: D8_ARGS += --debug
debug: all

test: all #grodd-venv
	$(ADB) install build/$(APP).apk
	$(ADB) logcat -s THESEUS -c
	python3 tests/test_apk.py
	$(ADB) logcat -s THESEUS -d
	#$(ADB) shell am start -n $(PACKAGE)/.$(MAIN_ACTIVITY)
	#grodd-venv/bin/grodd-runner -d emulator-5554 -r grodd -t 900 -p $(PACKAGE) -s 0.5

build/%.v1signed.apk: ./build/%.unsigned.apk ./ToyKey.keystore
	$(JARSIGNER) -verbose -keystore ./ToyKey.keystore -storepass $(pass) -keypass $(pass) -signedjar $@ $< SignKey

build/%.v1.apk: ./build/%.v1signed.apk
	$(SDK_TOOLS)/build-tools/$(VERSION)/zipalign -v -f 4 $< $@

# TODO: fix dep somehow? cannot find a way to use % or $* in (shell ..)
build/%/classes: $(shell find java/ -type f -regex ".*\.java" )
	mkdir -p ./build/$*/classes
	$(JAVAC) $(JAVAC_ARGS) -d ./build/$*/classes -classpath build/deps.jar:$(SDK_TOOLS)/platforms/android-$(VERSION_B)/android.jar $$(find java/$*/ -type f -regex ".*\.java")

build/%/classes.dex: build/%/classes
	mkdir -p ./build/$*
	$(D8) $(D8_ARGS) --classpath $(SDK_TOOLS)/platforms/android-$(VERSION_B)/android.jar $(shell find build/$*/classes -type f -regex ".*\.class" -printf "'%p'\n") --output ./build/$*/

build/a/classes.dex: build/a/classes
	mkdir -p ./build/a
	$(D8) $(D8_ARGS) --classpath $(SDK_TOOLS)/platforms/android-$(VERSION_B)/android.jar $(shell find build/a/classes -type f -regex ".*\.class" -printf "'%p'\n") --output ./build/a/
	#$(D8) $(D8_ARGS) --classpath $(SDK_TOOLS)/platforms/android-$(VERSION_B)/android.jar $(shell find build/a/classes -type f -regex ".*\.class" -not -name "ICommonInterface.class" -printf "'%p'\n") --output ./build/a/

build/%.unsigned.apk: build/classes/classes.dex build/a/classes.dex
	mkdir -p ./build/$*_files ./build/$*_files/assets
	mv ./build/classes/classes.dex ./build/$*_files/classes.dex
	mv build/a/classes.dex ./build/$*_files/assets/a.dex
	$(AAPT) package -v -f -M ./AndroidManifest.xml -I $(SDK_TOOLS)/platforms/android-$(VERSION_B)/android.jar -F $@ ./build/$*_files

build/%.v2aligned.apk: ./build/%.unsigned.apk ./ToyKey.keystore
	$(ZIPALIGN) -v -f 4 $< $@

build/%.apk: ./build/%.v2aligned.apk
	$(APKSIGNER) sign -ks ./ToyKey.keystore --v2-signing-enabled true --in $< --out $@ --ks-pass pass:$(pass)

ToyKey.keystore :
	$(KEYTOOL) -genkeypair -validity 1000 -dname "CN=SomeKey,O=SomeOne,C=FR" -keystore $@ -storepass $(pass) -keypass $(pass) -alias SignKey -keyalg RSA -v

#grodd-venv:
#	python3 -m venv grodd-venv
#	grodd-venv/bin/pip install 'git+ssh://git@gitlab.inria.fr/CIDRE/malware/grodd-runner.git'

clean:
	$(RM) -r build/*

clean_all: clean
	$(RM) ToyKey.keystore
