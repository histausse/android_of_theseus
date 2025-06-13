#!/usr/bin/env bash

FOLDER=$(dirname "$(realpath $0)")
PATCHER_DIR="${FOLDER}/../patcher"
PATCHER="${FOLDER}/../patcher/target/release/patcher"
KEY_PASS='P@ssw0rd!'

ANDROID_HOME=${ANDROID_HOME:-"${HOME}/Android/Sdk"}
ZIPALIGN=${ZIPALIGN:-"${ANDROID_HOME}/build-tools/34.0.0/zipalign"}
APKSIGNER=${APKSIGNER:-"${ANDROID_HOME}/build-tools/34.0.0/apksigner"}

APK_DIR="${1}"
RES_DIR="${2}"

export RUST_LOG='warn'

if [ ! -d "${RES_DIR}" ]; then 
    echo "Usage: bash ${0} /path/to/apk/dir /path/to/result/dir"
    echo "    /path/to/apk/dir is the folder where to store the application downloaded"
    echo "    /path/to/result/dir is the folder where the dynamic analysis result where stored"
    exit
fi

if [ ! -f "${ZIPALIGN}" ]; then
  echo "zipalign not found, please install 'build-tools;34.0.0' with `sdkmanager 'build-tools;34.0.0'` or call `ZIPALIGN=/path/to/zipalign bash ${0} /path/to/apk/dir /path/to/result/dir`"
  exit
fi
if [ ! -f "${APKSIGNER}" ]; then
  echo "apksigner not found, please install 'build-tools;34.0.0' with `sdkmanager 'build-tools;34.0.0'` or call `APKSIGNER=/path/to/zipalign bash ${0} /path/to/apk/dir /path/to/result/dir`"
  exit
fi


if [ ! -f "${FOLDER}/kestore.key" ]; then
    keytool -genkeypair -validity 1000 -dname 'CN=SomeKey,O=SomeOne,C=FR' -keystore "${FOLDER}/kestore.key" -storepass "${KEY_PASS}" -keypass "${KEY_PASS}" -alias SignKey -keyalg RSA -v
fi

cd "${PATCHER_DIR}"
cargo build --release
cd -


TMP_DIR=$(mktemp -d)

# split the apks to patch into 10 chunks
mkdir -p "${TMP_DIR}/apks"
N_CHUNK=$(python3 -c "print($(ls ${APK_DIR} | wc -l)//10 + 1)")
ls "${APK_DIR}" | sed 's#.*/##' | sed 's/\.apk//' > "${TMP_DIR}/apk_list"

split -a 2 -d -l "${N_CHUNK}" "${TMP_DIR}/apk_list" "${TMP_DIR}/apks/"

worker() {
  for sha in $(cat "${TMP_DIR}/apks/${1}"); do
    echo "worker ${1} started"
    # Check the result folder exist
    if [ ! -d "${RES_DIR}/${sha}" ]; then
      echo "Dynamic result not found for ${sha} (folder ${RES_DIR}/${sha} not found)"
      continue
    fi
    if [ ! -f "${RES_DIR}/${sha}/data.json" ]; then
      echo "Dynamic result not found for ${sha} (folder ${RES_DIR}/${sha}/data.json not found)"
      continue
    fi

    "${PATCHER}" --runtime-data "${RES_DIR}/${sha}/data.json" --path "${APK_DIR}/${sha}.apk" --out "${RES_DIR}/${sha}/patched.apk" -k "${FOLDER}/kestore.key" --keypassword "${KEY_PASS}" -z "${ZIPALIGN}" -a "${APKSIGNER}" --code-loading-patch-strategy model-class-loaders > "${RES_DIR}/${sha}/patcher.stdout" 2> "${RES_DIR}/${sha}/patcher.stderr"
  done
  echo "Finished ${1}"
}

for lst in $(ls "${TMP_DIR}/apks/"); do
  worker "${lst}" &
done

# delating the file used by the workers might not be a good idea
#rm -rf "${TMP_DIR}"
