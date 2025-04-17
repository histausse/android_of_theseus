#!/usr/bin/env python3

from pathlib import Path
import time
import subprocess
import os


def get_adb() -> str | Path:
    if "ANDROID_HOME" in os.environ:
        sdk_path = Path(os.environ["ANDROID_HOME"])
    else:
        sdk_path = Path.home() / "Android" / "Sdk"
    if not sdk_path.exists():
        return "adb"
    adb_path = sdk_path / "platform-tools" / "adb"
    if adb_path.exists():
        return adb_path
    else:
        return "adb"


def start_activity(
    adb: str | Path,
    activity: str,
    str_extra: dict[str, str],
    bool_extra: dict[str, bool],
):
    str_extra_args = [
        c for k, v in str_extra.items() for c in ("--es", k, v.replace(" ", "\\ "))
    ]
    bool_extra_args = [
        c for k, v in bool_extra.items() for c in ("--ez", k, "true" if v else "false")
    ]
    print(
        " ".join(
            [
                str(adb),
                "shell",
                "am",
                "start-activity",
                "-n",
                activity,
                *str_extra_args,
                *bool_extra_args,
            ]
        )
    )
    subprocess.Popen(
        [
            adb,
            "shell",
            "am",
            "start-activity",
            "-n",
            activity,
            *str_extra_args,
            *bool_extra_args,
        ]
    )


def kill_app(adb: str | Path, app_id: str):
    subprocess.Popen([adb, "shell", "am", "force-stop", app_id])


def clear_log(adb: str | Path):
    subprocess.Popen([adb, "logcat", "-c"])


def get_log_proc(adb: str | Path, tag: str) -> subprocess.Popen:
    return subprocess.Popen(
        [adb, "logcat", "-s", tag, "--format=raw"],
        encoding="utf-8",
        stdout=subprocess.PIPE,
    )


def wait_for_tst_log(
    logs: subprocess.Popen,
    clname: str,
    hasCollision: bool,
    hasParent: bool,
    methodType: str,
):
    hasCollision_p = "true" if hasCollision else "false"
    hasParent_p = "true" if hasParent else "false"
    first_line_found = False
    assert logs.stdout is not None
    for line in logs.stdout:
        # print(f"= {line}")
        # print(f"= {first_line_found=}")
        if first_line_found and line.startswith("POPUP,"):
            return
        if not first_line_found and (
            f"clname: {clname}," in line
            and f"hasCollision: {hasCollision_p}," in line
            and f"hasParent: {hasParent_p},"
            and f"methodType: {methodType}" in line
        ):
            first_line_found = True


CLASS_LOADERS = [
    "DelegateLastClassLoader",
    "DexClassLoader",
    "InMemoryDexClassLoader",
    "PathClassLoader",
]
METHOD_TYPES = [
    "Virtual",
    "Static",
    "Extended",
    "Interface",
    # "Interface Static",
    "Factory Pattern Interface",
    "Factory Pattern Extend",
]
COLLISION_OPT = [True, False]
PARENT_OPT = [True, False]


def main():
    adb = get_adb()
    clear_log(adb)
    logs = get_log_proc(adb, "THESEUS")
    for clname in CLASS_LOADERS:
        for hasCollision in COLLISION_OPT:
            for hasParent in PARENT_OPT:
                for methodType in METHOD_TYPES:
                    if (
                        clname == "DelegateLastClassLoader"
                        and "Factory Pattern" in methodType
                    ):
                        continue
                    if not hasParent and "Factory Pattern" in methodType:
                        continue
                    start_activity(
                        adb,
                        "com.example.theseus.dynandref/.MethodActivity",
                        {
                            "classLoaderName": clname,
                            "methodType": methodType,
                        },
                        {"collision": hasCollision, "parent": hasParent},
                    )
                    wait_for_tst_log(
                        logs,
                        clname,
                        hasCollision,
                        hasParent,
                        methodType,
                    )
                    # kill_app(adb, "com.example.theseus.dynandref")
                    # time.sleep(0.2)
    # save result afterward:
    # adb logcat -s THESEUS -d --format=raw > expected_result.txt


if __name__ == "__main__":
    main()
