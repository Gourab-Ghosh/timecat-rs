#!/usr/bin/env python3

import sys, tomllib
from manager import *

def process_args():
    binary_args_started = False
    args = set()
    binary_args = set()
    for arg in sys.argv:
        if binary_args_started:
            binary_args.add(arg)
        else:
            if arg == "--":
                binary_args_started = True
                continue
            args.add(arg)
    return args, binary_args

def main():
    with open("Cargo.toml", "rb") as rbf:
        cargo_toml_file_data = tomllib.load(rbf)
    assert set(cargo_toml_file_data["features"]["default"]) == {'binary', 'colored_output', 'speed'}

    FEATURE_SETS_CHECK = [
        [],
        ["default"],
        ["nnue_reader"],
        ["nnue_reader", "speed"],
        ["inbuilt_nnue"],
        ["inbuilt_nnue", "speed"],
        ["binary"],
        ["binary", "speed"],
        ["binary", "serde"],
        ["binary", "speed", "serde"],
    ]

    if sys.platform == "linux":
        home_dir = os.path.expanduser("~")
        possible_cargo_path = os.path.join(home_dir, ".cargo", "bin")
        sys.path.append(possible_cargo_path)

    if which("cargo") is None:
        print("Cargo not found. Please install Rust from https://www.rust-lang.org/tools/install")
        sys.exit(1)

    args, binary_args = process_args()

    if "check" in args:
        check_errors(FEATURE_SETS_CHECK)

    if "test" in args:
        test_package("--release" in args)

    if "build" in args:
        os.system(f"{RUST_FLAGS_STRING} cargo build --release")

    if "run" in args:
        run_package(os.path.dirname(__file__), args = args, binary_args = binary_args)

    if "backup" in args:
        backup_code("--noconfirm" in args)

    if "publish" in args:
        if not check_errors(FEATURE_SETS_CHECK):
            if not test_package():
                backup_code("--noconfirm" in args)
                publish_package()

if __name__ == "__main__":
    main()