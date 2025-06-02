from pathlib import Path
import argparse
import json


def run(
    summary_p: Path,
    show_distribution: bool,
    list_apks_nz_dyn: bool,
    show_dload_ads: bool,
):
    with summary_p.open("r") as fp:
        summary = json.load(fp)

    nb_failed = summary["nb_failed"] + summary["nb_timeout"]
    apk_with_zero_act_visited = set()
    apk_with_non_zero_act_visited = set()
    apk_ref = set()
    apk_z_ref = set()
    apk_nz_ref = set()
    apk_dyn = set()
    apk_z_dyn = set()
    apk_nz_dyn = set()

    nb_dload = 0
    nb_dload_fb = 0
    nb_dload_goo = 0
    nb_dlaod_fb_and_fb = 0
    dload_hashes_occ = {}
    for apk, apk_data in summary["apks"].items():
        for h in apk_data["dyn_loaded_files"].keys():
            if h not in dload_hashes_occ:
                dload_hashes_occ[h] = 0
            dload_hashes_occ[h] += 1
        for dload in apk_data["dyn_loaded_files"].values():
            nb_dload += 1
            if dload["facebook_ads"] and dload["google_ads"]:
                nb_dlaod_fb_and_fb += 1
            elif dload["facebook_ads"]:
                nb_dload_fb += 1
            elif dload["google_ads"]:
                nb_dload_goo += 1
        if apk_data["nb_visited_activity"] == 0:
            apk_with_zero_act_visited.add(apk)
            if apk_data["does_reflection"]:
                apk_ref.add(apk)
                apk_z_ref.add(apk)
            if apk_data["nb_dyn_loading"] != 0:
                apk_dyn.add(apk)
                apk_z_dyn.add(apk)
        else:
            apk_with_non_zero_act_visited.add(apk)
            if apk_data["does_reflection"]:
                apk_ref.add(apk)
                apk_nz_ref.add(apk)
            if apk_data["nb_dyn_loading"] != 0:
                apk_dyn.add(apk)
                apk_nz_dyn.add(apk)

    if show_dload_ads:
        print(
            f"number of dyn load                                            {nb_dload}"
        )
        print(
            f"number of dyn load of fb ads                                  {nb_dload_fb}"
        )
        print(
            f"number of dyn load of google ads                              {nb_dload_goo}"
        )
        print(
            f"number of dyn load of google+fb (not supposed to happen)      {nb_dlaod_fb_and_fb}"
        )
        print()
        hashes = list(dload_hashes_occ.keys())
        hashes.sort(key=lambda x: dload_hashes_occ[x], reverse=True)
        i = len(hashes)
        print(
            f"bytecode hash                                                     | number of occurence"
        )
        for h in hashes:
            occ = dload_hashes_occ[h]
            if occ <= 1:
                print(f"{i} other uniq bytecode files")
                break
            i -= 1
            print(f" {h} | {occ}")

    if show_distribution:
        print(
            "                  | nb apk | nb failled | nb 0 activity | nb non zero activities "
        )
        print(
            f"                  | {nb_failed+len(apk_with_zero_act_visited)+len(apk_with_non_zero_act_visited):5}  | {nb_failed:5}      | {len(apk_with_zero_act_visited):5}         | {len(apk_with_non_zero_act_visited):5} "
        )
        print(
            f" With Reflection  | {len(apk_ref):5}  |      X     | {len(apk_z_ref):5}         | {len(apk_nz_ref):5} "
        )
        print(
            f" With Dyn Loading | {len(apk_dyn):5}  |      X     | {len(apk_z_dyn):5}         | {len(apk_nz_dyn):5} "
        )

    if list_apks_nz_dyn:
        for apk in apk_nz_dyn:
            print(apk)


def main():
    parser = argparse.ArgumentParser(
        prog="get_app_runtime_result_distribution",
        description="check the result of the dynamic analysis and look at class collision",
    )
    parser.add_argument(
        "summary_runtime",
        type=Path,
        help="Summary computed by check_runtimedata.py",
    )
    parser.add_argument("--list-apks-nz-dyn", action="store_true")
    parser.add_argument("--show-distribution", action="store_true")
    parser.add_argument("--show-ads", action="store_true")
    args = parser.parse_args()
    run(
        args.summary_runtime,
        args.show_distribution,
        args.list_apks_nz_dyn,
        args.show_ads,
    )


if __name__ == "__main__":
    main()
