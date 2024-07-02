import os
from .build import run_command_with_target_cpu_native

def test_package(release_mode = False):
    if release_mode:
        return bool(run_command_with_target_cpu_native(f"cargo test --all-features --release"))
    else:
        return bool(os.system("cargo test --all-features"))