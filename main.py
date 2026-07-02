"""
Legacy entry point — use the Tauri desktop app instead.

Dev mode (engine only):
    python -m engine.service

Or from repo root with PYTHONPATH set:
    set PYTHONPATH=.
    python engine/service.py
"""

import sys


def main():
    print("Game Reader has moved to a standalone desktop app.")
    print("Run `npm run tauri dev` from the repo root, or install the release build.")
    print()
    print("Engine-only dev mode: python -m engine.service")
    sys.exit(0)


if __name__ == "__main__":
    main()
