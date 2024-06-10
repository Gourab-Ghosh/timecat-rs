import os, sys, time
from shutil import which

def get_environment_variables(*flags):
    rustflags = "-C target-cpu=native {}".format(" ".join("-C link-args=" + flag.strip() for flag in set(flags))).strip()
    return f"RUSTFLAGS={repr(rustflags)}"

def timed_run(func):
    start = time.time()
    res = func()
    if res and time.time() - start < 0.5:
        return res
    print(f"Run time: {round(time.time() - start, 3)} seconds")
    return 0

def run_package(current_path, args = None, environment_variables = None, perf = True, dry_run = False):
    if dry_run:
        os.system = lambda command: print(f"Running: {command}")
    if args is None:
        args = set()
    if environment_variables is None:
        environment_variables = set()
    else:
        args = set(args)
    is_error_free = True if {"--disable-check", "--no-check"}.intersection(args) else not os.system("cargo check --no-default-features --features debug")
    if not is_error_free:
        return
    commands = ["cargo", "build", "--no-default-features", "--features", "debug"]
    is_release = "--debug" not in args
    if is_release:
        commands.append("--release")
        commands.insert(0, get_environment_variables(*environment_variables))
        # commands.insert(0, get_environment_variables("-Ofast", "-mavx2", "-funroll-loops"))
        # commands.insert(0, get_environment_variables("-mavx2", "-funroll-loops"))
    if not os.system(" ".join(commands)):
        executable = "timecat.exe" if sys.platform == "win32" else "timecat"
        executable_path = os.path.join(current_path, "target", "release" if is_release else "debug", executable)
        need_to_run_without_perf = True
        if perf and which("perf") is not None:
            need_to_run_without_perf = bool(timed_run(lambda: os.system(f"perf record \"{executable_path}\" --test")))
        if need_to_run_without_perf:
            print("Running without using perf")
            timed_run(lambda: os.system(f"\"{executable_path}\" --test"))