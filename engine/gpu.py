"""CUDA availability check for engine init."""

import sys


def check_gpu() -> dict:
    result = {
        "available": False,
        "device_name": None,
        "cuda_version": None,
        "error": None,
    }
    try:
        import torch

        if not torch.cuda.is_available():
            result["error"] = (
                "NVIDIA GPU with CUDA is required. No CUDA device was detected."
            )
            return result
        result["available"] = True
        result["device_name"] = torch.cuda.get_device_name(0)
        result["cuda_version"] = getattr(torch.version, "cuda", None)
    except ImportError:
        result["error"] = "PyTorch is not installed."
    except Exception as e:
        result["error"] = str(e)
    return result


def require_gpu() -> dict | None:
    """Return error dict if GPU unavailable, else None."""
    info = check_gpu()
    if not info["available"]:
        return info
    return None
