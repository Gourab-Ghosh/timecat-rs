#!/usr/bin/env python3

import os

is_error_free = not os.system("cargo check")

if is_error_free:
    # external_flags = "-static -Ofast -mavx2 -funroll-loops"
    external_flags = "-Ofast -mavx2 -funroll-loops"
    # external_flags = ""

    rustflags_command = "-C target-cpu=native {}".format(" ".join("-Clink-args=" + flag.strip() for flag in external_flags.split())).strip()
    command = f"RUSTFLAGS={repr(rustflags_command)} cargo build --release"
    # print(command)
    if not os.system(command):
        os.environ["RUST_BACKTRACE"] = "1"
        os.system("time perf record -g ./target/release/timecat")