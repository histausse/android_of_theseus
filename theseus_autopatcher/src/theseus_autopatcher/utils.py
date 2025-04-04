import os
import argparse
import subprocess
import tempfile
from pathlib import Path
from shutil import which


def spinner(symbs: str = "-\\|/"):
    while True:
        for s in symbs:
            yield s


def get_android_sdk_path() -> Path | None:
    if "ANDROID_HOME" in os.environ:
        return Path(os.environ["ANDROID_HOME"])
    default = Path.home() / "Android" / "Sdk"
    if default.exists():
        return default
    return None


def get_build_tools_path(toolname: str) -> Path | None:
    def score_version(name: str):
        score = []
        for n in name.split("."):
            if n.isdecimal():
                score.append(int(n))
            else:
                score.append(-1)
        return score

    path = which(toolname)
    if path is not None:
        return Path(path)
    path = which(toolname + ".exe")
    if path is not None:
        return Path(path)

    sdk = get_android_sdk_path()
    if sdk is None:
        return None
    tools = sdk / "build-tools"
    if not tools.exists():
        return None
    options = []
    for d in tools.iterdir():
        if (d / toolname).exists():
            options.append(d / toolname)
        if (d / (toolname + ".exe")).exists():
            options.append(d / (toolname + ".exe"))
    if not options:
        return None
    return max(options, key=lambda d: score_version(d.parent.name))


def get_keytool_path() -> Path | None:
    path = which("keytool")
    if path is not None:
        return Path(path)
    path = which("keytool.exe")
    if path is not None:
        return Path(path)
    else:
        return None


def gen_keystore(keytool: Path, storepath: Path):
    print(f"{str(storepath)} does not exist, creating it.")
    subprocess.run(
        [
            str(keytool),
            "-genkeypair",
            "-validity",
            "1000",
            "-dname",
            "CN=SomeKey,O=SomeOne,C=FR",
            "-keystore",
            str(storepath),
            "-alias",
            "SignKey",
            "-keyalg",
            "RSA",
            "-v",
        ]
    )
