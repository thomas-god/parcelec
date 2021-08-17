from backend.domain.spot import Spot, Bid, Side

def test_create_spot_module():
    spot = Spot()

def test_post_bid():
    spot = Spot()
    bid = Bid(auction_id='auction', user_id='toto', side=Side.BUY, volume=1_000, price=50_00)
    spot.register(bid)
