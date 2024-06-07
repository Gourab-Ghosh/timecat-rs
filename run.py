#!/usr/bin/env python3

import os
import sys
import time
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
    print("Cargo not found. Please install Rust from https://www.rust-lang.org/tools/install")
    sys.exit(1)

args = set(sys.argv)

if "--update" in args:
    os.system("cargo update")

is_error_free = True if {"--disable-check", "--no-check"}.intersection(args) else not (os.system("cargo check --no-default-features") or os.system("cargo check --all-features"))

if is_error_free:
    is_test = "--test" in args
    build_or_test_command = "cargo test" if is_test else "cargo build"
    is_release = "--debug" not in args
    if is_release:
        build_or_test_command += " --release --no-default-features --features debug"
        update_environment_variables()
        # update_environment_variables("-Ofast", "-mavx2", "-funroll-loops")
        # update_environment_variables("-mavx2", "-funroll-loops")
    if is_test:
        os.system(build_or_test_command)
        sys.exit(0)
    if not os.system(build_or_test_command):
        executable = "timecat.exe" if sys.platform == "win32" else "timecat"
        file_to_run = "\"" + os.path.join(current_path, "target", "release" if is_release else "debug", executable) + "\""
        need_to_run = True
        if which("perf") is not None:
            need_to_run = bool(timed_run(lambda: os.system(f"perf record {file_to_run} --test")))
        if need_to_run:
            print("Running without using perf")
            timed_run(lambda: os.system(file_to_run + " --test"))