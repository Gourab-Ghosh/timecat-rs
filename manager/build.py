import os


def get_environment_variables(*flags):
    return "-C target-cpu=native {}".format(
        " ".join("-C link-args=" + flag.strip() for flag in set(flags))
    ).strip()


def run_command_with_target_cpu_native(command: str):
    try:
        os.environ["RUSTFLAGS"] = get_environment_variables()
        # os.environ["RUSTFLAGS"] = get_environment_variables("-Ofast", "-mavx2", "-funroll-loops")
        # os.environ["RUSTFLAGS"] = get_environment_variables("-mavx2", "-funroll-loops")
        return os.system(command)
    finally:
        del os.environ["RUSTFLAGS"]


def build_binary(is_release: bool, is_debug: bool = True):
    commands = ["cargo", "build"]
    if is_debug:
        commands += ["--no-default-features", "--features", "debug"]
    if is_release:
        commands.append("--release")
        return run_command_with_target_cpu_native(" ".join(commands))
    return os.system(" ".join(commands))
