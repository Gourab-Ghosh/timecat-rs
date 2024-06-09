import os

def test_package():
    os.system("cargo test --all-features")