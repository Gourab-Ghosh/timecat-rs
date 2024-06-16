import os
from .constants import RUST_FLAGS_STRING

def test_package(release_mode = False):
    if release_mode:
        return bool(os.system(f"{RUST_FLAGS_STRING} cargo test --all-features --release"))
    else:
        return bool(os.system("cargo test --all-features"))