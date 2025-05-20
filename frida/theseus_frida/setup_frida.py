import subprocess
import tempfile
import lzma
import shutil
import time

from pathlib import Path

FRIDA_SERVER_BIN = Path(__file__).parent / "frida-server-16.7.4-android-x86_64.xz"
FRIDA_SERVER_ANDROID_PATH = "/data/local/tmp/frida-server"


def setup_frida(device_name: str, env: dict[str, str], adb: str):
    env = env.copy()
    if "ANDROID_SERIAL" not in env and device_name != "":
        env["ANDROID_SERIAL"] = device_name
    # Start server
    proc: subprocess.CompletedProcess[str] | subprocess.CompletedProcess[bytes] = (
        subprocess.run(
            [adb, "shell", "whoami"],
            encoding="utf-8",
            stdout=subprocess.PIPE,
            env=env,
        )
    )
    if proc.stdout.strip() != "root":
        proc = subprocess.run([adb, "root"], env=env)

    perm = subprocess.run(
        [adb, "shell", "stat", "-c", "%a", FRIDA_SERVER_ANDROID_PATH],
        encoding="utf-8",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=env,
    ).stdout.strip()
    need_perm_resset = (perm == "") or perm[0] not in [
        "1",
        "3",
        "5",
        "7",
    ]  # int(perm[0]) & 1 == 1
    if perm == "":
        with tempfile.TemporaryDirectory() as tmpdname:
            tmpd = Path(tmpdname)
            with (
                lzma.open(str(FRIDA_SERVER_BIN.absolute())) as fin,
                (tmpd / "frida-server").open("wb") as fout,
            ):
                shutil.copyfileobj(fin, fout)

            subprocess.run(
                [
                    adb,
                    "push",
                    str((tmpd / "frida-server").absolute()),
                    FRIDA_SERVER_ANDROID_PATH,
                ],
                env=env,
            )
    if need_perm_resset:
        subprocess.run(
            [adb, "shell", "chmod", "755", FRIDA_SERVER_ANDROID_PATH], env=env
        )
    subprocess.Popen([adb, "shell", "nohup", FRIDA_SERVER_ANDROID_PATH], env=env)
