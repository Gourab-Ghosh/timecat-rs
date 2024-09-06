import os, sys, time
from shutil import which
from .build import build_binary


def timed_run(func):
    start = time.time()
    res = func()
    if res and time.time() - start < 0.5:
        return res
    print(f"Run time: {round(time.time() - start, 3)} seconds")
    return 0


def run_package(
    current_path,
    args=None,
    binary_args=None,
    environment_variables=None,
    perf=True,
    dry_run=False,
):
    run_command = (
        (lambda command: print(f"Running: {command}")) if dry_run else os.system
    )

    if args is None:
        args = set()
    if binary_args is None:
        binary_args = set()
    if environment_variables is None:
        environment_variables = set()

    is_error_free = (
        True
        if {"--disable-check", "--no-check"}.intersection(args)
        else not run_command("cargo check --no-default-features --features debug")
    )
    if not is_error_free:
        return
    is_release = "--debug" not in args
    if not build_binary(is_release):
        executable = "timecat.exe" if sys.platform == "win32" else "timecat"
        executable_path = os.path.join(
            current_path, "target", "release" if is_release else "debug", executable
        )
        need_to_run_without_perf = True
        executable_run_command = f'"{executable_path}" --test'
        if binary_args:
            executable_run_command += " " + " ".join(binary_args)
        if perf and which("perf") is not None:
            need_to_run_without_perf = bool(
                timed_run(lambda: run_command(f"perf record {executable_run_command}"))
            )
        if need_to_run_without_perf:
            print("Running without using perf")
            timed_run(lambda: run_command(executable_run_command))
