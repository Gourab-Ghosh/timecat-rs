#!/usr/bin/env python3

import os, subprocess
from inquirer.shortcuts import confirm

add_command = "git add ."
status_command = "git status"
os.system(add_command)
print()
os.system(status_command)
print()

if confirm("Do you want to continue?", default = True):
    print()
    commit_message = input("Enter commit message: ").strip()
    if commit_message == "":
        commit_message = ", ".join(line.strip() for line in subprocess.getoutput("git status -s").splitlines())
        print(f"\nGenerated commit message: {commit_message}\n")
    commit_command = f"git commit -m {repr(commit_message)}"
    push_command = "git push -u origin master"
    print()
    os.system(commit_command)
    os.system(push_command)