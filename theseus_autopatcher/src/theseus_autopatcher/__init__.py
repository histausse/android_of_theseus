import os
import argparse
import subprocess
import tempfile
from pathlib import Path
from shutil import which

from theseus_frida import collect_runtime

from .utils import *

PATCHER_BIN_PATH = Path(__file__).parent / "patcher_86_64_musl"


def patch_apk(
    runtime_data: Path,
    apk: Path,
    apkout: Path,
    zipalign: Path,
    apksigner: Path,
    keystore: Path,
    keypass: None | str = None,
):
    optional_args = []
    if keypass is not None:
        optional_args.append("--keypassword")
        optional_args.append(keypass)
    subprocess.run(
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
            "--code-loading-patch-strategy",
            "model-class-loaders",
            *optional_args,
        ]
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
        "-kp",
        "--keypass",
        required=False,
        help="Password for the key in the keystore",
        type=str,
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
            the package. (static x86_64 linux build with musl optimized for binary size instead of speed)",
        type=Path,
    )
    parser.add_argument(
        "--runner-script",
        required=False,
        help="Script to run to test the application. Must be a .py (python) or .sh (bash).",
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

    runner_f = None
    if args.runner_script is not None and args.runner_script.name.endswith(".py"):
        runner_f = lambda: subprocess.run(["python3", str(args.runner_script)])
    elif args.runner_script is not None and args.runner_script.name.endswith(".sh"):
        runner_f = lambda: subprocess.run(["bash", str(args.runner_script)])

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
        if args.keypass is None:
            args.keypass = "P@ssw0rd!"
        gen_keystore(keytool, args.keystore, args.keypass)

    with tempfile.TemporaryDirectory() as tmpdname:
        tmpd = Path(tmpdname)
        (tmpd / "dex").mkdir()
        with (tmpd / "runtime.json").open("w") as fp:
            collect_runtime(
                apk=args.apk,
                device_name=args.device,
                file_storage=tmpd / "dex",
                output=fp,
                android_sdk_path=get_android_sdk_path(),
                apk_explorer=runner_f,
            )
        patch_apk(
            runtime_data=tmpd / "runtime.json",
            apk=args.apk,
            apkout=args.output_apk,
            zipalign=zipalign,
            apksigner=apksigner,
            keystore=args.keystore,
            keypass=args.keypass,
        )
