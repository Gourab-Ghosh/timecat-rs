#!/usr/bin/env python3

import os
import time

is_error_free = not os.system("cargo check")

def timed_run(func):
    start = time.time()
    res = func()
    if res:
        return res
    print(f"Run time: {round(time.time() - start, 3)} seconds")
    return res

if is_error_free:
    # external_flags = "-static -Ofast -mavx2 -funroll-loops"
    external_flags = "-Ofast -mavx2 -funroll-loops"
    # external_flags = ""

    rustflags_command = "-C target-cpu=native {}".format(" ".join("-Clink-args=" + flag.strip() for flag in external_flags.split())).strip()
    command = f"RUSTFLAGS={repr(rustflags_command)} cargo build --release"
    # print(command)
    if not os.system(command):
        os.environ["RUST_BACKTRACE"] = "1"
        if timed_run(lambda: os.system("perf record -g ./target/release/timecat")):
            print("Running without using perf")
            timed_run(lambda: os.system("./target/release/timecat"))