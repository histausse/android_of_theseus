#!/usr/bin/env bash

SDK_TOOLS="${HOME}/Android/Sdk/"
VERSION='34.0.0'
VERSION_B=$(echo "${VERSION}" | sed 's/\..*//')
ANDROID_JAR="${SDK_TOOLS}/platforms/android-${VERSION_B}/android.jar"

FOLDER=$(dirname "$(realpath $0)")

FLOWDROID="${FOLDER}/soot-infoflow-cmd-jar-with-dependencies.jar"
SOURCE_SINK="${FOLDER}/source_sink.txt"
JAVA='/usr/lib/jvm/java-17-openjdk/bin/java'

"${JAVA}" -jar "${FLOWDROID}" -a "${1}" -p "${ANDROID_JAR}" -s "${SOURCE_SINK}"
