#!/usr/bin/env bash


JAVAC="${JAVAC:-/usr/lib/jvm/java-17-openjdk/bin/javac}"
ANDROID_SDK="${ANDROID_SDK:-${HOME}/Android/Sdk/}"
VERSION="${VERSION:-34.0.0}"
D8="${D8:-${ANDROID_SDK}/build-tools/${VERSION}/d8}"
VERSION_B="${VERSION_B:-$(echo "${VERSION}" | sed 's/\..*//')}"
ANDROID_JAR="${ANDROID_JAR:-${ANDROID_SDK}/platforms/android-${VERSION_B}/android.jar}"

FOLDER=$(dirname "$(realpath $0)")
BUILD_F="${BUILD_F:-${FOLDER}/build}"
OUT_FILE="${OUT_FILE:-${FOLDER}/../theseus_frida/StackConsumer.dex.b64}"

echo "JAVAC = ${JAVAC}"
echo "ANDROID_SDK = ${ANDROID_SDK}"
echo "VERSION = ${VERSION}"
echo "D8 = ${D8}"
# echo "VERSION_B = ${VERSION_B}"
echo "ANDROID_JAR = ${ANDROID_JAR}"
echo "BUILD_F = ${BUILD_F}"
echo "OUT_FILE = ${OUT_FILE}"

rm -r "${BUILD_F}"
mkdir "${BUILD_F}"

"${JAVAC}" -Xlint -d "${BUILD_F}" -classpath "${ANDROID_JAR}" "${FOLDER}/StackConsumer.java"

mkdir "${BUILD_F}/classes"
"${D8}" --classpath "${ANDROID_JAR}" "${BUILD_F}/theseus/android/StackConsumer.class" --output "${BUILD_F}/classes"

base64 "${BUILD_F}/classes/classes.dex" > "${OUT_FILE}"
