"""Set up the logger for the project."""

import logging
from typing import ClassVar


class ColouredFormatter(logging.Formatter):  # noqa: D101
    COLOURS: ClassVar[dict[str, str]] = {
        "DEBUG": "\033[94m",  # Blue
        "INFO": "\033[92m",  # Green
        "WARNING": "\033[93m",  # Yellow
        "ERROR": "\033[91m",  # Red
        "CRITICAL": "\033[41m",  # White
    }
    RESET = "\033[0m"

    def format(self, record: logging.LogRecord) -> str:  # noqa: D102
        color = self.COLOURS.get(record.levelname, self.RESET)
        message = super().format(record)
        return f"{color}{message}{self.RESET}"


def setup_logger() -> None:  # noqa: D103
    logger = logging.getLogger()
    logger.setLevel(logging.DEBUG)

    handler = logging.StreamHandler()
    handler.setFormatter(ColouredFormatter("%(levelname)s: %(message)s"))
    logger.addHandler(handler)
