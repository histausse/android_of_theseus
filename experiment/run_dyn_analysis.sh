#!/usr/bin/bash

FOLDER=$(dirname "$(realpath $0)")

APK_DIR="${1}"
RES_DIR="${2}"

if [ ! -d "${APK_DIR}" ]; then
    echo "Usage: bash ${0} /path/to/apk/dir /path/to/result/dir"
    echo "    /path/to/apk/dir is the folder where to store the application downloaded"
    echo "    /path/to/result/dir is the folder where to store the analysis results"
    exit
fi
if [ ! -n "${RES_DIR}" ]; then
    echo "Usage: bash ${0} /path/to/apk/dir /path/to/result/dir"
    echo "    /path/to/apk/dir is the folder where to store the application downloaded"
    echo "    /path/to/result/dir is the folder where to store the analysis results"
    exit
fi

mkdir -p "${RES_DIR}"

TMP_DIR=$(mktemp -d)

python3 -m venv "${FOLDER}/venv"

#"${FOLDER}/venv/bin/pip" install "${FOLDER}/../frida"
#"${FOLDER}/venv/bin/pip" install "git+ssh://git@gitlab.inria.fr/CIDRE/malware/grodd-runner.git"

ls "${APK_DIR}"/*.apk > "${TMP_DIR}/apklist.txt"

python3 "${FOLDER}/orchestrator.py" "${TMP_DIR}/apklist.txt" "${RES_DIR}" "${FOLDER}/frida_collect.sh"
