import os
from .check import errors_check

def publish_package():
    has_errors = errors_check()
    if not has_errors:
        os.system("cargo publish")