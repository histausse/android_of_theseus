from pathlib import Path
import time

try:
    from grodd_runner import grodd_runner  # type: ignore

    USE_GRODD = True
except ModuleNotFoundError:
    USE_GRODD = False


def explore_app(
    package: str,
    device: str = "emulator-5554",
    android_sdk: Path | None = None,
):
    if USE_GRODD:
        time.sleep(1)  # let the app load
        grodd_runner(
            "grodd",
            device,
            timeout=300,
            package=package,
            android_sdk=android_sdk,
            slowdown=1.0,
        )

    else:
        print(
            "\033[31mGrodd is not available, you need to explore the app manually\033[0m"
        )
        manual_exploration()


def manual_exploration():
    print("==> Press ENTER to end the analysis <==")
    input()
