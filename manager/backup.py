import os, subprocess

def confirm(message, default = False):
    message += " [Y/n]" if default else " [y/N]"
    response = ""
    while response not in ("y", "n"):
        response = input(message).strip().lower()
        if response == "":
            return default
    return response == "y"

def backup_code(no_confirm = False):
    add_command = "git add ."
    status_command = "git status"
    os.system(add_command)
    print()
    os.system(status_command)
    if not no_confirm:
        print()

    confirmed_continue = True if no_confirm else confirm("Do you want to continue?", default = True)
    if confirmed_continue:
        if not no_confirm:
            print()
        commit_message = "" if no_confirm else input("Enter commit message: ").strip()
        if not commit_message:
            commit_message = ", ".join(line.strip() for line in subprocess.getoutput("git status -s").splitlines())
            print(f"\nGenerated commit message: {commit_message}\n")
        commit_command = f"git commit -m {repr(commit_message)}"
        push_command = "git push -u origin master"
        print()
        os.system(commit_command)
        os.system(push_command)