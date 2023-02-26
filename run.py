#!/usr/bin/env python3

import os
import sys
import time
import requests
from shutil import which

def update_environment_variables(*flags):
    rustflags_command = "-Ctarget-cpu=native {}".format(" ".join("-Clink-args=" + flag.strip() for flag in flags)).strip()
    os.environ["RUSTFLAGS"] = rustflags_command
    os.environ["RUST_BACKTRACE"] = "1"

def timed_run(func):
    start = time.time()
    res = func()
    if res and time.time() - start < 0.1:
        return res
    print(f"Run time: {round(time.time() - start, 3)} seconds")
    return 0

root_url = "https://media.githubusercontent.com/media/Gourab-Ghosh/timecat-rs/master/"
current_path = os.path.dirname(__file__)
files = ["src/nnue_weights.rs"]

for file in files:
    file_path = os.path.join(current_path, file)
    if os.stat(file_path).st_size <= 1024:
        full_url = root_url + file
        print(f"Downloading {file} from url {full_url}...")
        with open(file_path, "w") as f:
            f.write(requests.get(full_url).text)

if which("cargo") is None:
    if sys.platform == "win32":
        os.system("Please install Rust manually and add it to PATH and run this script again.")
        sys.exit(1)
    else:
        print(f"Installing Rust...")
        os.system("curl --proto '=https' -sSf https://sh.rustup.rs | sh")

is_error_free = not os.system("cargo check")

if is_error_free:
    update_environment_variables()
    # update_environment_variables("-Ofast", "-mavx2", "-funroll-loops")
    # update_environment_variables("-mavx2", "-funroll-loops")
    if not os.system("cargo build --release"):
        if timed_run(lambda: os.system("perf record -g ./target/release/timecat")):
            print("Running without using perf")
            timed_run(lambda: os.system("./target/release/timecat"))