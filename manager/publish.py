import os
from .constants import CARGO_TOML_FILE_DATA


def assert_publish_condition():
    assert set(CARGO_TOML_FILE_DATA["features"]["default"]) == {
        "colored",
        "inbuilt_nnue",
    }


def publish_package():
    assert_publish_condition()
    os.system("cargo publish")
