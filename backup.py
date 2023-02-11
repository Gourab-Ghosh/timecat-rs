#!/usr/bin/env python3

import os
from inquirer.shortcuts import confirm

add_command = "git add ."
status_command = "git status"
commit_message = input("Enter commit message: ").strip()
if commit_message == "":
    commit_message = "random commit"
commit_command = "git commit -m {}".format(commit_message)
push_command = "git push -u origin master"

os.system(add_command)
print()
os.system(status_command)
print()

if confirm("Do you want to continue", default = True):
    print()
    os.system(push_command)