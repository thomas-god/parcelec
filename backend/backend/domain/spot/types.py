import attr
from enum import Enum

class Side(Enum):
    """Side"""

    BUY = "BUY"
    SELL = "SELL"

@attr.s(auto_attribs=True, slots=True)
class Bid:
    """Spot Bid"""

    auction_id: str
    user_id: str
    side: Side
    volume: int
    price: int