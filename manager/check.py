import os, sys
from tqdm import tqdm

def generate_command(feature_set, quiet):
    commands = ["cargo", "check", "--no-default-features"]
    if quiet:
        commands.append("--quiet")
    if feature_set:
        features = sorted(feature_set)
        commands += ["--features", ",".join(features)]
    return " ".join(commands)

def errors_check(feature_sets_check, verbose = True) -> bool:
    print("Updating Packages...")
    if os.system("cargo update --quiet"):
        sys.exit(1)
    try:
        os.environ["NNUE_DOWNLOAD"] = "PAUSE"
        for feature_set in tqdm(feature_sets_check, desc = "Checking Feature Combinations", leave = False):
            if os.system(generate_command(feature_set, True)):
                if verbose:
                    print(f"Feature Set {feature_set} has errors!")
                    print(f"Command: {generate_command(feature_set, False)}")
                return True
        return False
    finally:
        del os.environ["NNUE_DOWNLOAD"]