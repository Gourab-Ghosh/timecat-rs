def install_package(package):
    try:
        import pip
    except ImportError:
        print("\npip not found, installing...\n")
        import ensurepip
        ensurepip.bootstrap()
    import sys, subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", package])