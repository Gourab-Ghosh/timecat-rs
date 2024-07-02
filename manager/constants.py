import tomllib

with open("Cargo.toml", "rb") as rbf:
    CARGO_TOML_FILE_DATA = tomllib.load(rbf)