import tomllib
from .utils import get_environment_variables

RUST_FLAGS_STRING = get_environment_variables()

with open("Cargo.toml", "rb") as rbf:
    CARGO_TOML_FILE_DATA = tomllib.load(rbf)