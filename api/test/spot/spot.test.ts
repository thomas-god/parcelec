import { SPOT, Bid, Side } from "../../src/domain/spot";
import "chai";

describe("SPOT module", () => {
  it("Should post a bid to the SPOT module", () => {
    const spot = new SPOT();

    const bid = new Bid('toto', Side.BUY, 10, 10)

    spot.register(bid)
  });
});
