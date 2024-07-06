import os
from .build import run_command_with_target_cpu_native
from .constants import CARGO_TOML_FILE_DATA

def test_package(release_mode = False):
    features = list(CARGO_TOML_FILE_DATA["features"].keys())
    features.remove("wasm")
    features.sort()
    features = ",".join(features)
    if release_mode:
        return bool(run_command_with_target_cpu_native(f"cargo test --no-default-features --features {features} --release"))
    else:
        return bool(os.system(f"cargo test --no-default-features --features {features}"))