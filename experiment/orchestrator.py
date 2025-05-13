from pathlib import Path

import os
import time
import subprocess
import threading
import argparse

EMULATORS = ["root34-1", "root34-2"]
ANDROID_IMG = "system-images;android-34;default;x86_64"

if "ANDROID_HOME" in os.environ:
    ANDROID_HOME = Path(os.environ["ANDROID_HOME"])
else:
    ANDROID_HOME = Path.home() / "Android" / "Sdk"

EMULATOR = str(ANDROID_HOME / "emulator" / "emulator")
AVDMANAGER = str(ANDROID_HOME / "cmdline-tools" / "latest" / "bin" / "avdmanager")
ADB = str(ANDROID_HOME / "platform-tools" / "adb")


def get_ports(emu: str) -> tuple[int, int]:
    """Return the console port and adb port for the emulator."""
    i = EMULATORS.index(emu) * 2
    return (5554 + i, 5554 + i + 1)


def get_installed_emu() -> set[str]:
    """List name of installed emulators"""
    return set(
        subprocess.run([EMULATOR, "-list-avds"], stdout=subprocess.PIPE)
        .stdout.decode("utf-8")
        .strip()
        .split("\n")
    )


def gen_emulators():
    emu_lst = get_installed_emu()
    for emu in EMULATORS:
        if emu not in emu_lst:
            subprocess.run(
                [
                    AVDMANAGER,
                    "create",
                    "avd",
                    "--name",
                    emu,
                    "--package",
                    ANDROID_IMG,
                    "--sdcard",
                    "512M",
                    "--device",
                    "medium_phone",
                ]
            )


def del_emulators():
    emu_lst = get_installed_emu()
    for emu in EMULATORS:
        if emu in emu_lst:
            subprocess.run(
                [
                    AVDMANAGER,
                    "delete",
                    "avd",
                    "--name",
                    emu,
                ]
            )


# def make_snapshot(folder: Path):
#    for emu in EMULATORS:
#        console_port, adb_port = get_ports(emu)
#        proc = subprocess.Popen(
#            [
#                EMULATOR,
#                "-avd",
#                emu,
#                "-no-window",
#                "-no-metrics",
#                "-debug-init",
#                "-logcat",
#                "*:v",
#                "-ports",
#                f"{console_port},{adb_port}",
#            ]
#        )
#        subprocess.run([ADB, "-s", f"emulator-{console_port}", "wait-for-device"])
#        subprocess.run(
#            [
#                ADB,
#                "-s",
#                f"emulator-{console_port}",
#                "emu",
#                "avd",
#                "snapshot",
#                "save",
#                "baseline",
#            ]
#        )


def worker(emu: str, apklist: list[str], out_folder: Path, script: Path):
    console_port, adb_port = get_ports(emu)
    while apklist:
        apk = apklist.pop()
        folder_name = apk.split("/")[-1].removesuffix(".apk")
        folder = out_folder / folder_name
        if folder.exists():
            continue
        folder.mkdir(parents=True)

        # Start emulator with wipped data
        proc = subprocess.Popen(
            [
                EMULATOR,
                "-avd",
                emu,
                "-wipe-data",
                "-no-window",
                "-no-metrics",
                "-debug-init",  # dunno why but sometime needed
                "-ports",
                f"{console_port},{adb_port}",
            ]
        )

        # Run script
        subprocess.run(
            ["bash", str(script), f"emulator-{console_port}", apk, str(out_folder)]
        )

        # stop emulator
        try:
            subprocess.run(
                [
                    ADB,
                    "-s",
                    f"emulator-{console_port}",
                    "emu",
                    "kill",
                ],
                timeout=3,
            )
        except subprocess.TimeoutExpired:
            pass
        if proc.poll() is None:
            proc.kill()
            time.sleep(3)


def run(apklist: list[str], out_folder: Path, script: Path):
    workers = []
    for emu in EMULATORS:
        workers.append(
            threading.Thread(target=lambda: worker(emu, apklist, out_folder, script))
        )
        workers[-1].start()
    for w in workers:
        w.join()


def main():
    parser = argparse.ArgumentParser(
        prog="orchestrator",
        description="Run several android emulators en run analysis on applications",
    )
    parser.add_argument(
        "applist",
        type=Path,
        help="File containing the path to applications, one by line",
    )
    parser.add_argument(
        "out_folder",
        type=Path,
        help="The folder where to store the results of the analysis, a folder for each application will be created in it",
    )
    parser.add_argument(
        "analysis_script",
        type=Path,
        help=(
            "The script to run the analysis. The script will be invoke with "
            "`bash analysis_script.sh path/of/app.apk emulator-5554 path/of/out_folder/app/`"
        ),
    )
    args = parser.parse_args()
    with args.applist.open("r") as fp:
        apklist = fp.readlines()
    run(apklist, args.out_folder, args.analysis_script)


if __name__ == "__main__":
    main()
