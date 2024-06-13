import os
from .check import errors_check

def publish_package():
    os.system("cargo publish")