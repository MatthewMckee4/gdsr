from importlib.util import find_spec
from typing import Dict

MODULE_AVAILABILITY: Dict[str, bool] = {}


def check_module(module_name: str) -> bool:
    """Check if the specified module is available."""
    if module_name not in MODULE_AVAILABILITY:
        MODULE_AVAILABILITY[module_name] = find_spec(module_name) is not None
    return MODULE_AVAILABILITY[module_name]
