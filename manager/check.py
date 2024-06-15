import os, sys
from tqdm import tqdm

def generate_command(feature_set, quiet):
    if feature_set is None:
        return "cargo check --all-features --quiet"
    commands = ["cargo", "check", "--no-default-features"]
    if quiet:
        commands.append("--quiet")
    if feature_set:
        features = sorted(feature_set)
        commands += ["--features", ",".join(features)]
    return " ".join(commands)

def check_errors(feature_sets_check, verbose = True) -> bool:
    feature_sets_check = set(tuple(sorted(set(feature_set))) for feature_set in feature_sets_check)
    feature_sets_check.add(None)
    feature_sets_check = sorted(feature_sets_check, key = lambda k: (-1 if k is None else len(k), k))
    if verbose:
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