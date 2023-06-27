#!/usr/bin/env python3

import os
import sys
import time
import requests
from shutil import which

def update_environment_variables(*flags):
    rustflags_command = "-C target-cpu=native {}".format(" ".join("-C link-args=" + flag.strip() for flag in flags)).strip()
    os.environ["RUSTFLAGS"] = rustflags_command

def timed_run(func):
    start = time.time()
    res = func()
    if res and time.time() - start < 0.5:
        return res
    print(f"Run time: {round(time.time() - start, 3)} seconds")
    return 0

current_path = os.path.dirname(__file__)

if sys.platform == "linux":
    home_dir = os.path.expanduser("~")
    possible_cargo_path = os.path.join(home_dir, ".cargo", "bin")
    sys.path.append(possible_cargo_path)

if which("cargo") is None:
    if sys.platform == "win32":
        print("Please install Rust manually and add it to PATH and run this script again.")
        sys.exit(1)
    else:
        print(f"Installing Rust...")
        os.system("curl --proto '=https' -sSf https://sh.rustup.rs | sh")

is_error_free = True if "--disable-check" in sys.argv else not os.system("cargo check")

if is_error_free:
    build_command = "cargo build"
    is_release = "--debug" not in sys.argv
    if is_release:
        build_command += " --release"
        update_environment_variables()
        # update_environment_variables("-Ofast", "-mavx2", "-funroll-loops")
        # update_environment_variables("-mavx2", "-funroll-loops")
    if not os.system(build_command):
        executable = "timecat.exe" if sys.platform == "win32" else "timecat"
        file_to_run = "\"" + os.path.join(current_path, "target", "release" if is_release else "debug", executable) + "\""
        need_to_run = True
        if which("perf") is not None:
            need_to_run = bool(timed_run(lambda: os.system(f"perf record {file_to_run} --test")))
        if need_to_run:
            print("Running without using perf")
            timed_run(lambda: os.system(file_to_run + " --test"))