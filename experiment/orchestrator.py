from pathlib import Path

import os
import time
import subprocess
import threading
import argparse

EMULATORS = [f"root34-{i}" for i in range(4)]
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
    script_env = os.environ.copy()
    script_env["ANDROID_HOME"] = str(ANDROID_HOME)
    while apklist:
        apk = apklist.pop()
        folder_name = apk.split("/")[-1].removesuffix(".apk")
        folder = out_folder / folder_name
        if folder.exists():
            continue
        folder.mkdir(parents=True)

        with (
            (folder / "emu.out").open("w") as fp_emu_stdout,
            (folder / "emu.err").open("w") as fp_emu_stderr,
            (folder / "analysis.out").open("w") as fp_anly_stdout,
            (folder / "analysis.err").open("w") as fp_anly_stderr,
        ):

            # Start emulator with wipped data
            print(f"START ANALYSIS: {apk=}, emulator-{console_port}")
            i = 0
            started = False
            while not started:
                if i != 0 and i % 10 == 0:
                    print(
                        f"Warning: tried to start emulator-{console_port} (avd {emu}) for the {i}th time without success"
                    )
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
                    ],
                    stdout=fp_emu_stdout,
                    stderr=fp_emu_stderr,
                )
                subprocess.run(
                    [ADB, "-s", f"emulator-{console_port}", "wait-for-device"],
                    stdout=fp_anly_stdout,
                    stderr=fp_anly_stderr,
                )
                j = 0
                while not started:
                    started = f"emulator-{console_port}\t device" not in subprocess.run(
                        [ADB, "devices"], stdout=subprocess.PIPE
                    ).stdout.decode("utf-8")
                    if not started:
                        time.sleep(1)
                        if j != 0 and j % 10 == 0:
                            print(
                                f"emulator-{console_port} has been offline for 10s, restarting it now"
                            )
                            proc.kill()
                            break
                        j += 1
                i += 1
            print(f"emulator-{console_port} started")
            fp_anly_stdout.write(f"START ANALYSIS: {apk=}, emulator-{console_port}\n")
            subprocess.run(
                [ADB, "devices"],
                stdout=fp_anly_stdout,
                stderr=fp_anly_stderr,
            )
            print(f"FINISHED ANALYSIS: {apk=}, emulator-{console_port}")

            # Run script
            subprocess.run(
                ["bash", str(script), apk, f"emulator-{console_port}", str(folder)],
                env=script_env,
                stdout=fp_anly_stdout,
                stderr=fp_anly_stderr,
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
            print(f"emulator-{console_port} stoped")


def run(apklist: list[str], out_folder: Path, script: Path):
    gen_emulators()
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
        apklist = list(map(str.strip, fp.readlines()))
    run(apklist, args.out_folder, args.analysis_script)


if __name__ == "__main__":
    main()
