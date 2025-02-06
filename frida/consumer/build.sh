#!/usr/bin/env bash


JAVAC='/usr/lib/jvm/java-17-openjdk/bin/javac'
SDK_TOOLS="${HOME}/Android/Sdk/"
VERSION='34.0.0'
D8="${SDK_TOOLS}/build-tools/${VERSION}/d8"
VERSION_B=$(echo "${VERSION}" | sed 's/\..*//')
ANDROID_JAR="${SDK_TOOLS}/platforms/android-${VERSION_B}/android.jar"

FOLDER=$(dirname "$(realpath $0)")
BUILD_F="${FOLDER}/build"
OUT_FILE="${FOLDER}/../theseus_frida/StackConsumer.dex.b64"
rm -r "${BUILD_F}"
mkdir "${BUILD_F}"

"${JAVAC}" -d "${BUILD_F}" -classpath "${ANDROID_JAR}" "${FOLDER}/StackConsumer.java"

mkdir "${BUILD_F}/classes"
"${D8}" --classpath "${ANDROID_JAR}" "${BUILD_F}/theseus/android/StackConsumer.class" --output "${BUILD_F}/classes"

base64 "${BUILD_F}/classes/classes.dex" > "${OUT_FILE}"
