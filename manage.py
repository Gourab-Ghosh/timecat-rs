#!/usr/bin/env python3

import sys
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
    if sys.platform == "linux":
        home_dir = os.path.expanduser("~")
        possible_cargo_path = os.path.join(home_dir, ".cargo", "bin")
        sys.path.append(possible_cargo_path)

    if which("cargo") is None:
        print("Cargo not found. Please install Rust from https://www.rust-lang.org/tools/install")
        sys.exit(1)
    
    args, binary_args = process_args()
    
    if "check" in args:
        errors_check()

    if "test" in args:
        test_package()

    if "run" in args:
        run_package(os.path.dirname(__file__), args = binary_args)

    if "backup" in args:
        pass

    if "publish" in args:
        publish_package()

if __name__ == "__main__":
    main()