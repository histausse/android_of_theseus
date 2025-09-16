# PEP 723 inline deps (https://peps.python.org/pep-0723/):
#
# /// script
# requires-python = ">=3.13"
# dependencies = [
#   "androguard==4.1.3",
# ]
# ///

import zipfile

from argparse import ArgumentParser
from pathlib import Path

from androguard.misc import AnalyzeAPK
from androguard.core.analysis.analysis import Analysis
from androguard.core import dex
from androguard.util import set_log

from networkx.classes.digraph import DiGraph

set_log("CRITICAL")


GLUE_METHODS: set[str] = {
    "Ljava/lang/reflect/Method;->invoke(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;",
    "Ljava/lang/reflect/Method;->getName()Ljava/lang/String;",
    "Ljava/lang/reflect/Method;->getParameterTypes()[Ljava/lang/Class;",
    "Ljava/lang/reflect/Method;->getReturnType()Ljava/lang/Class;",
    "Ljava/lang/reflect/Method;->getDeclaringClass()Ljava/lang/Class;",
    "Ljava/lang/String;->equals(Ljava/lang/Object;)Z"
    "Ljava/lang/Class;->newInstance()Ljava/lang/Object;",
    "Ljava/lang/reflect/Constructor;->newInstance([Ljava/lang/Object;)Ljava/lang/Object;",
    "Ljava/lang/reflect/Constructor;->getParameterTypes()[Ljava/lang/Class;",
    "Ljava/lang/reflect/Constructor;->getDeclaringClass()Ljava/lang/Class;",
    "Ljava/lang/Class;->descriptorString()Ljava/lang/String;",
    "Ljava/lang/Boolean;->booleanValue()Z"
    "Ljava/lang/Byte;->byteValue()B"
    "Ljava/lang/Short;->shortValue()S"
    "Ljava/lang/Character;->charValue()C"
    "Ljava/lang/Integer;->intValue()I"
    "Ljava/lang/Long;->longValue()J"
    "Ljava/lang/Float;->floatValue()F"
    "Ljava/lang/Double;->doubleValue()D"
    "Ljava/lang/Boolean;->valueOf(Z)Ljava/lang/Boolean;",
    "Ljava/lang/Byte;->valueOf(B)Ljava/lang/Byte;",
    "Ljava/lang/Short;->valueOf(S)Ljava/lang/Short;",
    "Ljava/lang/Character;->valueOf(C)Ljava/lang/Character;",
    "Ljava/lang/Integer;->valueOf(I)Ljava/lang/Integer;",
    "Ljava/lang/Long;->valueOf(J)Ljava/lang/Long;",
    "Ljava/lang/Float;->valueOf(F)Ljava/lang/Float;",
    "Ljava/lang/Double;->valueOf(D)Ljava/lang/Double;",
    "Ljava/lang/Class;->getClassLoader()Ljava/lang/ClassLoader;",
    "Ljava/lang/ClassLoader;->getParent()Ljava/lang/ClassLoader;",
    "Ljava/lang/Object;->getClass()Ljava/lang/Class;",
    "Ljava/lang/Object;->toString()Ljava/lang/String;",
    # Classes used:
    #
    # "Ljava/lang/BootClassLoader;",
    # "Ljava/lang/Object;",
    # "Ldalvik/system/DelegateLastClassLoader;",
    # "Ljava/lang/Boolean;",
    # "Ljava/lang/Byte;",
    # "Ljava/lang/Short;",
    # "Ljava/lang/Character;",
    # "Ljava/lang/Integer;",
    # "Ljava/lang/Long;",
    # "Ljava/lang/Float;",
    # "Ljava/lang/Double;",
}


def is_generated_method(method) -> bool:
    class_def = method.get_class_name()
    if class_def.startswith("Ltheseus/") and class_def.endswith("/T;"):
        return True
    return False


def is_glue_method(method) -> bool:
    if is_generated_method(method):
        return True
    full_name = (
        f"{method.get_class_name()}->{method.get_name()}{method.get_descriptor()}"
    )
    return full_name in GLUE_METHODS


def count_edges(cg: DiGraph) -> tuple[int, int]:
    """Count method calls and method calls that we may have added (glue methods).
    Comparing this number of glue edges allows to compute how many actuall edges we added.
    """
    n = 0
    glue = 0
    for u, v in cg.edges():
        n += 1
        if is_generated_method(u) or is_glue_method(v):
            glue += 1
        # print(f"{u.get_name()} -> {v.get_name()}")

    return n, glue


def main():
    parser = ArgumentParser(
        description="Compare the call graph of an application and its patched version"
    )
    parser.add_argument("app", help="The original application", type=Path)
    parser.add_argument("patched_app", help="The patched apk", type=Path)
    parser.add_argument(
        "--show-new-methods", action="store_true", help="Show added methods edges"
    )
    parser.add_argument(
        "--csv-format",
        action="store_true",
        help="Show the results in a CSV format (apk sha256, nb edge before, nb edges after, added, added ref only)",
    )
    parser.add_argument(
        "--dyn-bytecode", action="extend", nargs="+", type=Path, default=[]
    )

    args = parser.parse_args()

    apk, _, dx = AnalyzeAPK(args.app)
    cg = dx.get_call_graph()
    _, _, dx2 = AnalyzeAPK(args.patched_app)
    cg_patched = dx2.get_call_graph()

    dyn_cgs = []
    for dyn in args.dyn_bytecode:
        if zipfile.is_zipfile(dyn):
            _, _, dx = AnalyzeAPK(dyn)
        else:
            print(dyn)
            dx = Analysis()
            with dyn.open("rb") as fp:
                raw = fp.read()
                d = dex.DEX(raw, using_api=apk.get_target_sdk_version())
            dx.add(d)
            dx.create_xref()

        dyn_cgs.append(dx.get_call_graph())

    nb_methods_app = cg.number_of_nodes()
    nb_methods_pch = cg_patched.number_of_nodes()
    nb_methods_dyn = sum(map(lambda x: x.number_of_nodes(), dyn_cgs))

    nb_edges_app, nb_glue_app = count_edges(cg)
    nb_edges_pch, nb_glue_pch = count_edges(cg_patched)
    nb_edges_dyn, nb_glue_dyn = 0, 0
    for cgd in dyn_cgs:
        nb_e, nb_g = count_edges(cgd)
        nb_edges_dyn += nb_e
        nb_glue_dyn += nb_g

    added_glue = nb_glue_pch - nb_glue_dyn - nb_glue_app
    # added_edges = nb_edges_pch - nb_edges_app - added_glue # meh, don't works for 35065C683441E62C59C0DA0D86E6793256E33E54834E22AD0F70F44C99419E2F?
    added_edges = nb_edges_pch - nb_edges_app
    added_ref_only = 0

    all_original_edges = set()
    for u, v in cg.edges():
        all_original_edges.add((u.full_name, v.full_name))
    for cgd in dyn_cgs:
        for u, v in cgd.edges():
            all_original_edges.add((u.full_name, v.full_name))

    for u, v in cg_patched.edges():
        if is_generated_method(u) or is_glue_method(v):
            continue
        if (u.full_name, v.full_name) in all_original_edges:
            continue
        added_ref_only += 1

    if args.csv_format:
        import hashlib

        with args.app.open("rb") as fp:
            hash = hashlib.file_digest(fp, "sha256").hexdigest().upper()
        print(f"{hash},{nb_edges_app},{nb_edges_pch},{added_edges},{added_ref_only}")
        # apk sha256, nb edge before, nb edges after, added, added ref only
    else:
        print(f"app: {args.app}\npatched: {args.patched_app}")

        print("app:")
        print(f"  nodes: {nb_methods_app}")
        print(f"  nb edges {nb_edges_app}")
        print(f"  glue edges {nb_glue_app}")
        print("dyn loaded:")
        print(f"  nodes: {nb_methods_dyn}")
        print(f"  nb edges {nb_edges_dyn}")
        print(f"  glue edges {nb_glue_dyn}")
        print("patched:")
        print(f"  nb node: {nb_methods_pch}")
        print(f"  nb edges {nb_edges_pch}")
        print(f"  glue edges {nb_glue_pch}")
        print("")
        print(f"Total edges added: {added_edges} ({added_ref_only} ref only)")

    if args.show_new_methods:
        for u, v in cg_patched.edges():
            if is_generated_method(u) or is_glue_method(v):
                continue
            if (u.full_name, v.full_name) in all_original_edges:
                continue
            # print(
            #    f"{u.get_class_name()}->{u.get_name()}  ==>  {v.get_class_name()}->{v.get_name()}"
            # )
            print(f"{u.full_name}  ==>  {v.full_name}")

    return cg_patched


if __name__ == "__main__":
    cg = main()
