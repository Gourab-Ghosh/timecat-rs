#!/usr/bin/env python3

import os

try:
    from inquirer.shortcuts import confirm
except ImportError:
    try:
        import pip
    except ImportError:
        print("\npip not found, installing...\n")
        import ensurepip
        ensurepip.bootstrap()
    print("\ninquirer not found, installing...\n")
    import sys, subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "inquirer"])
    from inquirer.shortcuts import confirm

add_command = "git add ."
status_command = "git status"
os.system(add_command)
print()
os.system(status_command)
print()

if confirm("Do you want to continue", default = True):
    print()
    commit_message = input("Enter commit message: ").strip()
    if commit_message == "":
        commit_message = "random commit"
    commit_command = f"git commit -m {repr(commit_message)}"
    push_command = "git push -u origin master"
    print()
    os.system(commit_command)
    os.system(push_command)