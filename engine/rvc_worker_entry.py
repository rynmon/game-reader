"""PyInstaller entry point for the RVC worker sidecar."""

import runpy

if __name__ == "__main__":
    runpy.run_module("engine.rvc_worker", run_name="__main__")
