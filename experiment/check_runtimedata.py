from pathlib import Path
import hashlib
import argparse
import json

import androguard  # type: ignore
from androguard.core.dex import DEX  # type: ignore
from androguard.core.apk import APK  # type: ignore

androguard.util.set_log("SUCCESS")  # type: ignore


def get_bytecode_classes(bytecode: bytes) -> list[str]:
    try:
        dex = DEX(bytecode)
        return list(map(lambda x: x.get_name(), dex.get_classes()))
    except ValueError:
        apk = APK(bytecode, raw=True, skip_analysis=True)
        classes = []
        for dex_bin in apk.get_all_dex():
            dex = DEX(dex_bin)
            classes.extend(dex.get_classes())
        return list(map(lambda x: x.get_name(), classes))


def check_app_result(
    path: Path, app_folder: Path, summary: dict, keep_ref_data: bool = False
):
    if (path / "TIMEOUT").exists():
        summary["nb_timeout"] += 1
        return
    if not (path / "data.json").exists():
        return
    with (path / "data.json").open() as fp:
        data = json.load(fp)
    if "error" in data:
        summary["nb_failed"] += 1
        return

    nb_visited_activity = None
    with (path / "analysis.err").open("r") as fp:
        for line in fp:
            line = line.strip()
            if "Visited activities:" in line:
                nb_visited_activity = int(line.split("Visited activities:")[1].strip())

    does_reflection = False
    boot_cl_id = ""
    for cl in data["classloaders"].values():
        if cl["cname"] == "Ljava/lang/BootClassLoader;":
            boot_cl_id = cl["id"]

    reflections = []
    nb_class_collision_at_invoke = 0
    seen = {}
    for invoke_data in data["invoke_data"]:
        if invoke_data["caller_cl_id"] != boot_cl_id:
            does_reflection = True
        call_site = (
            invoke_data["caller_method"],
            invoke_data["caller_cl_id"],
            invoke_data["addr"],
        )
        clazz = invoke_data["method"].split("->")[0]
        id_ = (call_site, clazz)
        cl = invoke_data["method_cl_id"]
        if id_ not in seen:
            seen[id_] = {cl}  # first call
        if cl not in seen[id_]:
            nb_class_collision_at_invoke += 1
        seen[id_].add(cl)

        cl_class = "unknown"
        if invoke_data["caller_cl_id"] in data["classloaders"]:
            cl_class = data["classloaders"][invoke_data["caller_cl_id"]]["cname"]
        reflections.append(
            (
                invoke_data["caller_method"],
                cl_class,
                invoke_data["addr"],
                invoke_data["method"],
            )
        )

    seen = {}
    for invoke_data in data["class_new_inst_data"]:
        if invoke_data["caller_cl_id"] != boot_cl_id:
            does_reflection = True
        call_site = (
            invoke_data["caller_method"],
            invoke_data["caller_cl_id"],
            invoke_data["addr"],
        )
        clazz = invoke_data["constructor"].split("->")[0]
        id_ = (call_site, clazz)
        cl = invoke_data["constructor_cl_id"]
        if id_ not in seen:
            seen[id_] = {cl}  # first call
        if cl not in seen[id_]:
            nb_class_collision_at_invoke += 1
        seen[id_].add(cl)

        cl_class = "unknown"
        if invoke_data["caller_cl_id"] in data["classloaders"]:
            cl_class = data["classloaders"][invoke_data["caller_cl_id"]]["cname"]
        reflections.append(
            (
                invoke_data["caller_method"],
                cl_class,
                invoke_data["addr"],
                invoke_data["constructor"],
            )
        )

    seen = {}
    for invoke_data in data["cnstr_new_inst_data"]:
        if invoke_data["caller_cl_id"] != boot_cl_id:
            does_reflection = True
        call_site = (
            invoke_data["caller_method"],
            invoke_data["caller_cl_id"],
            invoke_data["addr"],
        )
        clazz = invoke_data["constructor"].split("->")[0]
        id_ = (call_site, clazz)
        cl = invoke_data["constructor_cl_id"]
        if id_ not in seen:
            seen[id_] = {cl}  # first call
        if cl not in seen[id_]:
            nb_class_collision_at_invoke += 1
        seen[id_].add(cl)

        cl_class = "unknown"
        if invoke_data["caller_cl_id"] in data["classloaders"]:
            cl_class = data["classloaders"][invoke_data["caller_cl_id"]]["cname"]
        reflections.append(
            (
                invoke_data["caller_method"],
                cl_class,
                invoke_data["addr"],
                invoke_data["constructor"],
            )
        )

    if len(data["dyn_code_load"]) != 0:
        does_reflection = True

    classes_by_cl: dict[str, list[str]] = {}
    dyn_load_classes = set()
    dyn_loaded_files = {}
    for dyn_load in data["dyn_code_load"]:
        dyn_load_classes.add(dyn_load["classloader_class"])
        cl_id = dyn_load["classloader"]
        if cl_id not in classes_by_cl:
            classes_by_cl[cl_id] = []
        for file in dyn_load["files"]:
            with open(file, "rb") as fp:
                dex_bin = fp.read()
            hasher = hashlib.sha256()
            hasher.update(dex_bin)
            h = hasher.hexdigest()
            classes = get_bytecode_classes(dex_bin)

            dyn_loaded_files[h] = {
                "classes": classes,
                "facebook_ads": any(
                    map(lambda x: x.startswith("Lcom/facebook/ads/"), classes)
                ),
                "google_ads": any(
                    map(lambda x: x.startswith("Lcom/google/android/ads/"), classes)
                ),
            }
            classes_by_cl[cl_id].extend(classes)

    # Don't do androguard scan when there is no other dynloading
    if len(data["dyn_code_load"]) != 0:
        apk_name = f"{path.name}.apk"
        cl_id = data["apk_cl_id"]
        if cl_id not in classes_by_cl:
            classes_by_cl[cl_id] = []
        with (app_folder / apk_name).open("rb") as fp:
            apk_bin = fp.read()
        classes_by_cl[cl_id].extend(get_bytecode_classes(apk_bin))

    nb_class_collision = 0
    already_found: set[str] = set()
    for cls_l in classes_by_cl.values():
        cls: set[str] = set(cls_l)
        nb_class_collision += len(already_found.intersection(cls))
        already_found.update(cls)

    summary["apks"][path.name] = {
        "nb_class_collision": nb_class_collision,
        "nb_class_collision_at_invoke": nb_class_collision_at_invoke,
        "nb_ref": len(reflections),
        "reflections": reflections,
        "does_reflection": does_reflection,
        "nb_visited_activity": nb_visited_activity,
        "nb_dyn_loading": len(data["dyn_code_load"]),
    }
    if not keep_ref_data:
        summary["apks"][path.name]["reflections"] = None

    if nb_class_collision:
        summary["nb_with_class_collision"] += 1
    if nb_class_collision_at_invoke:
        summary["nb_with_class_collision_at_invoke"] += 1


def run(folder: Path, app_folder: Path):
    summary = {
        "nb_timeout": 0,
        "nb_failed": 0,
        "nb_with_class_collision": 0,
        "nb_with_class_collision_at_invoke": 0,
        "apks": {},
        "baseline_reflection": [],
    }
    for p in folder.iterdir():
        if p.is_dir():
            check_app_result(p, app_folder, summary, keep_ref_data=True)

    # Strange, looks like there is no baseline? This need investigation
    # apk_data = summary["apks"]
    # assert isinstance(apk_data, dict)
    #
    # summary["baseline_reflection"] = list(
    #     set.intersection(*map(lambda x: set(x["reflections"]), apk_data.values()))
    # )

    print(json.dumps(summary, indent="  "))


def main():
    parser = argparse.ArgumentParser(
        prog="check_collision",
        description="check the result of the dynamic analysis and look at class collision",
    )
    parser.add_argument(
        "result_folder",
        type=Path,
        help="Folder containing the result of the experiment",
    )
    parser.add_argument(
        "app_folder",
        type=Path,
        help="Folder containing the apks of the experiment",
    )
    args = parser.parse_args()
    run(args.result_folder, args.app_folder)


if __name__ == "__main__":
    main()
