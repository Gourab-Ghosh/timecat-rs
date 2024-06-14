import os

def test_package():
    return bool(os.system("cargo test --all-features"))