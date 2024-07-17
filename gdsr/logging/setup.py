import logging


class ColouredFormatter(logging.Formatter):
    COLOURS = {
        "DEBUG": "\033[34m",  # Blue
        "INFO": "\033[32m",  # Green
        "WARNING": "\033[33m",  # Yellow
        "ERROR": "\033[31m",  # Red
        "CRITICAL": "\033[35m",  # Magenta
    }
    RESET = "\033[0m"

    def format(self, record: logging.LogRecord) -> str:
        color = self.COLOURS.get(record.levelname, self.RESET)
        message = super().format(record)
        return f"{color}{message}{self.RESET}"


def setup_logger() -> None:
    logger = logging.getLogger()
    logger.setLevel(logging.INFO)

    handler = logging.StreamHandler()
    handler.setFormatter(ColouredFormatter("%(levelname)s: %(message)s"))
    logger.addHandler(handler)
