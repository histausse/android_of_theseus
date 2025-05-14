#!/usr/bin/bash

FOLDER=$(dirname "$(realpath $0)")

APK="${1}"
DEVICE="${2}"
OUT_DIR="${3}"

echo "APK=${APK}"
echo "DEVICE=${DEVICE}"
echo "OUT_DIR=${OUT_DIR}"

"${FOLDER}/venv/bin/collect-runtime-data" --apk "${APK}" --device "${DEVICE}" --output "${OUT_DIR}/data.json" --dex-dir "${3}"
