import os
from .check import check_errors

def publish_package():
    os.system("cargo publish")