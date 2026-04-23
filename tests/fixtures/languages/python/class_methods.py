"""Repository module docs."""

MAX_RETRIES = 3


class Repository:
    """Repository docs."""

    @classmethod
    def create(cls) -> "Repository":
        """Create new repo."""
        return cls()

    @staticmethod
    def validate(value):
        return bool(value)

    def save(self) -> None:
        pass
