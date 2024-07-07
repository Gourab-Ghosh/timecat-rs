import os, sys
from tqdm import tqdm

def generate_command(feature_set, clippy, quiet):
    check_command = "clippy" if clippy else "check"
    if feature_set is None:
        return f"cargo {check_command} --all-features --quiet"
    commands = ["cargo", check_command, "--no-default-features"]
    if quiet:
        commands.append("--quiet")
    if feature_set:
        features = sorted(feature_set)
        commands += ["--features", ",".join(features)]
    return " ".join(commands)

def check_errors(feature_sets_check, clippy = False, verbose = True) -> bool:
    feature_sets_check = set(tuple(sorted(set(feature_set))) for feature_set in feature_sets_check)
    feature_sets_check.add(None)
    feature_sets_check = sorted(feature_sets_check, key = lambda k: (-1 if k is None else len(k), k))
    if verbose:
        print("Updating Packages...")
    if os.system("cargo update"):
        sys.exit(1)
    try:
        os.environ["NNUE_DOWNLOAD"] = "PAUSE"
        for feature_set in tqdm(feature_sets_check, desc = "Checking Feature Combinations", leave = False):
            if os.system(generate_command(feature_set, clippy, True)):
                if verbose:
                    print(f"Feature Set {feature_set} has errors!")
                    print(f"Command: {generate_command(feature_set, clippy, False)}")
                return True
        return False
    finally:
        del os.environ["NNUE_DOWNLOAD"]