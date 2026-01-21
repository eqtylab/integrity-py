class Cid:
    @property
    def cid(self) -> str:
        """Get the cid string."""
        return self._cid

    def __init__(self, cid: str):
        self._cid: str = cid

    def __str__(self) -> str:
        return self._cid
