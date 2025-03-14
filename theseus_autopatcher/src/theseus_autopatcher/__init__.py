import os
import argparse
import subprocess
import tempfile
from pathlib import Path
from shutil import which

from theseus_frida import collect_runtime


def get_android_sdk_path() -> Path | None:
    if "ANDROID_HOME" in os.environ:
        return os.environ["ANDROID_HOME"]
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
        return path
    path = which(toolname + ".exe")
    if path is not None:
        return path

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
        return path
    return which("keytool.exe")


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


PATCHER_BIN_PATH = Path(__file__).parent / "patcher_86_64_musl"


def patch_apk(
    runtime_data: Path,
    apk: Path,
    apkout: Path,
    zipalign: Path,
    apksigner: Path,
    keystore: Path,
):
    def dbg(l):
        print(" ".join(l))
        return l

    subprocess.run(
        dbg(
            [
                str(PATCHER_BIN_PATH.absolute()),
                "--runtime-data",
                str(runtime_data.absolute()),
                "--path",
                str(apk.absolute()),
                "--out",
                str(apkout.absolute()),
                "-k",
                str(keystore.absolute()),
                "-z",
                str(zipalign.absolute()),
                "-a",
                str(apksigner.absolute()),
            ]
        )
    )


def main():
    parser = argparse.ArgumentParser(prog="Android Theseus project")
    parser.add_argument(
        "-a", "--apk", required=True, help="Target application", type=Path
    )
    parser.add_argument(
        "-s",
        "--device",
        default="",
        help="The android device to connect to, eg: 'emulator-5554'",
        type=str,
    )
    parser.add_argument(
        "-o",
        "--output-apk",
        required=True,
        help="Where to write the repackaged apk",
        type=Path,
    )
    parser.add_argument(
        "--zipalign",
        required=False,
        help="Path to the zipalign executable to use",
        type=Path,
    )
    parser.add_argument(
        "--apksigner",
        required=False,
        help="Path to the apksigner executable to use",
        type=Path,
    )
    parser.add_argument(
        "-k",
        "--keystore",
        required=False,
        help="Path to the apksigner executable to use",
        type=Path,
        default=Path(".") / "TheseusKey.keystore",
    )
    parser.add_argument(
        "--keytool",
        required=False,
        help="Path to the keytool executable to use",
        type=Path,
    )
    parser.add_argument(
        "--patch",
        required=False,
        help="Path to the patcher executable to use. By default, use the one embeded with \
            the package. (static x86_64 linux build with musl)",
        type=Path,
    )
    args = parser.parse_args()

    if args.zipalign is None:
        zipalign = get_build_tools_path("zipalign")
    else:
        zipalign = args.zipalign
    if args.apksigner is None:
        apksigner = get_build_tools_path("apksigner")
    else:
        apksigner = args.apksigner
    if args.keytool is None:
        keytool = get_keytool_path()
    else:
        keytool = args.keytool

    if zipalign is None:
        print(
            "Could not find zipalign, please install an android build-tools package. "
            "If one is already installed, please use `--zipalign` to provide the path "
            "to the zipalign executable."
        )
        exit(1)
    if apksigner is None:
        print(
            "Could not find apksigner, please install an android build-tools package. "
            "If one is already installed, please use `--apksigner` to provide the path "
            "to the apksigner executable."
        )
        exit(1)
    if keytool is None and not args.keystore.exists():
        print(
            f"Could not find keytool and {str(args.keystore)} does not exist. Either "
            "provide an existing keystore with -k or install a JDK. If one is already installed, "
            "please use --keytool to provide the path to the keytool executable."
        )
        exit(1)

    if not args.keystore.exists():
        gen_keystore(keytool, args.keystore)

    with tempfile.TemporaryDirectory() as tmpdname:
        tmpd = Path(tmpdname)
        (tmpd / "dex").mkdir()
        with (tmpd / "runtime.json").open("w") as fp:
            collect_runtime(
                apk=args.apk, device=args.device, file_storage=tmpd / "dex", output=fp
            )
        patch_apk(
            runtime_data=tmpd / "runtime.json",
            apk=args.apk,
            apkout=args.output_apk,
            zipalign=zipalign,
            apksigner=apksigner,
            keystore=args.keystore,
        )
