#!/usr/bin/bash

FOLDER=$(dirname "$(realpath $0)")

if adb devices | grep -q 'emulator-'; then
    echo 'Emulator already started'
else
    echo 'Emulator no started'
    QT_QPA_PLATFORM=xcb alacritty -e ~/Android/Sdk/emulator/emulator -avd root34 &
fi

env --chdir "${FOLDER}" poetry build

TMP=$(mktemp -d)
python -m venv "${TMP}"
source "${TMP}/bin/activate"
pip install "${FOLDER}/dist/theseus_autopatcher-0.1.0-py3-none-any.whl[grodd]"

#source .venv/bin/activate

adb wait-for-device

#theseus-autopatch -a "${FOLDER}/../test_apks/dynloading/build/test_dynloading.apk" -o /tmp/patched_dynloading.apk -k "${FOLDER}/../test_apks/dynloading/ToyKey.keystore"
theseus-autopatch -a "${FOLDER}/../test_apks/dyn_and_ref/build/test_dyn_and_ref.apk" -o /tmp/patched_dynloading.apk -k /tmp/kstore.keystore -kp 'P@ssw0rd!' --runner-script "${FOLDER}/../test_apks/dyn_and_ref/tests/test_apk.py" --patch "${FOLDER}/../patcher/target/release/patcher"

rm -rf "${TMP}"
