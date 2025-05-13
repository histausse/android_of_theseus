#!/usr/bin/bash

FOLDER=$(dirname "$(realpath $0)")

"${FOLDER}/venv/bin/collect-runtime-data" --apk "${1}" --device "${2}" --output "${3}/data.json" --dex-dir "${3}"
