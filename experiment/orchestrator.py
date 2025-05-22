from pathlib import Path

import os
import json
import shutil
import time
import subprocess
import threading
import argparse
import queue
import datetime
import traceback

EMULATORS = [f"root34-{i}" for i in range(20)]
ANDROID_IMG = "system-images;android-34;default;x86_64"
TIMEOUT = 400

if "ANDROID_HOME" in os.environ:
    ANDROID_HOME = Path(os.environ["ANDROID_HOME"])
else:
    ANDROID_HOME = Path.home() / "Android" / "Sdk"

EMULATOR = str(ANDROID_HOME / "emulator" / "emulator")
AVDMANAGER = str(ANDROID_HOME / "cmdline-tools" / "latest" / "bin" / "avdmanager")
ADB = str(ANDROID_HOME / "platform-tools" / "adb")


class AdbFailed(RuntimeError):
    pass


def adb_run(emu: str, cmd: list[str], timeout: int | None = None) -> str:
    """Run an adb command,
    Warning: don't use this to run a command with long output:
    will hang due to deadlock on process.run when capturing output"""
    cmd_l = [ADB, "-s", emu, *cmd]
    cmd_txt = " ".join(cmd_l)
    for i in range(3):
        r = subprocess.run(
            cmd_l, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout
        )
        if b"error: could not connect to TCP port" in r.stderr:
            print(f"failled to run `{cmd_txt}`: error '{r.stderr.decode('utf-8')}'")
            time.sleep(i + 1)
            if i != 2:
                print("retrying")
        else:
            return r.stdout.decode("utf-8")
    raise AdbFailed("Failed to run `{cmd_txt}`")


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
            make_snapshot(emu)


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


FRIDA_SETUP_SCRIPT = (
    Path(__file__).parent.parent / "frida" / "theseus_frida" / "setup_frida.py"
)


def make_snapshot(emu: str):
    console_port, adb_port = get_ports(emu)
    # First run with debug stuff, for because android emulator black magic fuckery ? probably ?
    proc = subprocess.Popen(
        [
            EMULATOR,
            "-avd",
            emu,
            "-no-window",
            "-no-metrics",
            "-debug-init",
            "-logcat",
            "*:v",
            "-ports",
            f"{console_port},{adb_port}",
        ]
    )
    adb_run(f"emulator-{console_port}", ["wait-for-device"])
    time.sleep(10)
    # stop emulator
    try:
        adb_run(f"emulator-{console_port}", ["emu", "kill"], timeout=25)
        time.sleep(25)
    except subprocess.TimeoutExpired:
        pass
    if proc.poll() is None:
        proc.kill()
        time.sleep(3)

    # start the emulator without the debug stuff
    proc = subprocess.Popen(
        [
            EMULATOR,
            "-avd",
            emu,
            "-no-window",
            "-no-metrics",
            "-ports",
            f"{console_port},{adb_port}",
        ]
    )
    adb_run(f"emulator-{console_port}", ["wait-for-device"])
    time.sleep(1)

    # setup frida, uggly, but meh, at this point
    import importlib.util

    spec = importlib.util.spec_from_file_location(
        "setup_frida", str(FRIDA_SETUP_SCRIPT)
    )
    assert spec is not None
    setup_frida = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(setup_frida)
    setup_frida.setup_frida(f"emulator-{console_port}", os.environ, ADB)

    time.sleep(10)
    adb_run(
        f"emulator-{console_port}",
        [
            "emu",
            "avd",
            "snapshot",
            "save",
            "baseline",
        ],
    )
    # stop emulator
    try:
        adb_run(
            f"emulator-{console_port}",
            [
                "emu",
                "kill",
            ],
            timeout=25,
        )
        time.sleep(25)
    except subprocess.TimeoutExpired:
        pass
    if proc.poll() is None:
        proc.kill()
        time.sleep(3)


def restore_emu(emu: str, proc: None | subprocess.Popen) -> subprocess.Popen:
    console_port, adb_port = get_ports(emu)
    if proc is not None and proc.poll() is None:
        adb_run(
            f"emulator-{console_port}",
            [
                "emu",
                "avd",
                "snapshot",
                "save",
                "baseline",
            ],
        )
        time.sleep(3)
        return proc
    proc = subprocess.Popen(
        [
            EMULATOR,
            "-avd",
            emu,
            "-no-window",
            "-no-metrics",
            "-ports",
            f"{console_port},{adb_port}",
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    adb_run(f"emulator-{console_port}", ["wait-for-device"])
    time.sleep(3)
    adb_run(
        f"emulator-{console_port}",
        [
            "emu",
            "avd",
            "snapshot",
            "load",
            "baseline",
        ],
    )
    time.sleep(3)
    return proc


def worker(emu: str, apklist: queue.Queue[str], out_folder: Path, script: Path):
    console_port, adb_port = get_ports(emu)
    script_env = os.environ.copy()
    script_env["ANDROID_HOME"] = str(ANDROID_HOME)
    proc_emu = restore_emu(emu, None)
    marked_done = False
    try:
        while True:
            apk = apklist.get()
            marked_done = False
            folder_name = apk.split("/")[-1].removesuffix(".apk")
            folder = out_folder / folder_name

            # Check if XP has already run without error or timeout
            if folder.exists() and (folder / "data.json").exists():
                has_error = False
                with (folder / "data.json").open() as fp:
                    try:
                        data = json.load(fp)
                        if "error" in data:
                            has_error = True
                    except json.JSONDecodeError:
                        has_error = True
                if (folder / "TIMEOUT").exists():
                    has_error = True
                if has_error:
                    # print(
                    #    f"Previous result for {apk=} found but with error of timeout, remove old result and rerun it"
                    # )
                    shutil.rmtree(str(folder))
                else:
                    # We already have a valid result, mark task done and skip xp
                    apklist.task_done()
                    marked_done = True
                    continue
            if folder.exists():
                shutil.rmtree(str(folder))
            folder.mkdir(parents=True)

            with (
                (folder / "analysis.out").open("w") as fp_anly_stdout,
                (folder / "analysis.err").open("w") as fp_anly_stderr,
            ):

                # print(f"START ANALYSIS: {apk=}, emulator-{console_port}")

                # Reset the emulator and make sure it is runing
                i = 0
                started = False
                while not started:
                    if i != 0 and i % 10 == 0:
                        print(
                            f"Warning: tried to start emulator-{console_port} (avd {emu}) for the {i}th time without success"
                        )
                    proc_emu = restore_emu(emu, proc_emu)
                    try:
                        adb_run(
                            f"emulator-{console_port}", ["wait-for-device"], timeout=30
                        )
                    except subprocess.TimeoutExpired:
                        print(f"Wait for device emulator-{console_port} timedout")
                    j = 0
                    while not started:
                        started = f"emulator-{console_port}\tdevice" in subprocess.run(
                            [ADB, "devices"], stdout=subprocess.PIPE, timeout=30
                        ).stdout.decode("utf-8")
                        if not started:
                            time.sleep(1)
                            if j != 0 and j % 10 == 0:
                                print(
                                    f"emulator-{console_port} has been offline for 10s, restarting it now"
                                )
                                proc_emu.kill()
                                break
                            j += 1
                    i += 1

                # print(f"emulator-{console_port} running")
                fp_anly_stdout.write(
                    f"START ANALYSIS: {apk=}, emulator-{console_port}\n"
                )
                # should help debuging:
                subprocess.run(
                    [ADB, "devices"],
                    stdout=fp_anly_stdout,
                    stderr=fp_anly_stderr,
                    timeout=30,
                )

                # Run script
                try:
                    subprocess.run(
                        [
                            "bash",
                            str(script),
                            apk,
                            f"emulator-{console_port}",
                            str(folder),
                        ],
                        env=script_env,
                        stdout=fp_anly_stdout,
                        stderr=fp_anly_stderr,
                        timeout=TIMEOUT,
                    )
                    # print(f"FINISHED ANALYSIS: {apk=}, emulator-{console_port}")
                # If timeout:
                except subprocess.TimeoutExpired:
                    with (folder / "TIMEOUT").open("w") as fp:
                        fp.write("Process timedout")
                    print(f"TIMEOUT ANALYSIS: {apk=}, emulator-{console_port}")
                # again, for debuging:
                with (folder / "emu").open("w") as fp:
                    fp.write(f"Used emulator {emu}:  emulator-{console_port}")
            apklist.task_done()
            marked_done = True
            nb_emu_running = sum(
                1
                for _ in filter(
                    lambda s: "emulator-" in s,
                    subprocess.run(
                        [ADB, "devices"],
                        stdout=subprocess.PIPE,
                    )
                    .stdout.decode()
                    .split("\n"),
                )
            )
            print(
                f"[{datetime.datetime.now()}][{emu}(emulator-{console_port})] end loop, \
                    {len(list(threading.enumerate()))} threads running, \
                    {nb_emu_running} emulators running"
            )
    except Exception as e:
        msg = f"[{datetime.datetime.now()}] worker for {emu} (emulator-{console_port}) terminated after {e}"
        print(msg)
        with (out_folder / f"worker_{emu}").open("w") as fp:
            fp.write(msg)
            fp.write("\n")
            fp.write("\n".join(traceback.format_exception(e)))
        if not marked_done:
            apklist.task_done()
        if not apklist.empty():
            worker(emu, apklist, out_folder, script)


def run(apklist: list[str], out_folder: Path, script: Path):
    gen_emulators()
    workers = []
    q: queue.Queue[str] = queue.Queue()
    for apk in apklist:
        q.put(apk)
    for emu in EMULATORS:
        workers.append(
            threading.Thread(target=lambda: worker(emu, q, out_folder, script))
        )
        workers[-1].start()
    q.join()


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
