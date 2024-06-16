def get_environment_variables(*flags):
    rustflags = "-C target-cpu=native {}".format(" ".join("-C link-args=" + flag.strip() for flag in set(flags))).strip()
    return f"RUSTFLAGS={repr(rustflags)}"