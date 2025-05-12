#!/usr/bin/env bash

APK_LIST="${1}"
APK_DIR="${2}"

ANDROZOO_URL="https://androzoo.uni.lu/api/download"


if [ ! -f "${APK_LIST}" ]; then 
    echo "Usage: bash ${0} apk_list.txt /path/to/apk/dir"
    echo "    apk_list.txt is a file containing the sha256 of the applications to download"
    echo "    /path/to/apk/dir is the folder where to store the application downloaded"
    exit
fi

read -s -p "Androzoo Key: " APIKEY

TMP_DIR=$(mktemp -d)
mkdir -p "${APK_DIR}"

# split the apks to download into 10 chunks
mkdir -p "${TMP_DIR}/apks"
N_CHUNK=$(python3 -c "print($(cat ${APK_LIST} | wc -l)//10 + 1)")
split -a 2 -d -l "${N_CHUNK}" "${APK_LIST}" "${TMP_DIR}/apks/"

androzoo() {
  sha="${1}"
  curl -o "${APK_DIR}/${sha}.apk" -G -d apikey=${APIKEY} -d sha256=${sha} "${ANDROZOO_URL}"
}

worker() {
  for sha in $(cat "${TMP_DIR}/apks/${1}"); do
    if [ ! -f "${APK_DIR}/${sha}.apk" ]; then
      echo "Download ${sha}"
      androzoo "${sha}"
    fi
  done
  echo "Finished ${1}"
}

for lst in $(ls "${TMP_DIR}/apks/"); do
  worker "${lst}" &
done

rm -rf "${TMP_DIR}"
