<script lang="ts">
  import { match } from "ts-pattern";
  import { parseMessage, type OrderBook } from "./message";
  import OrderBookEntry from "./orderBookEntry.svelte";

  const socket = new WebSocket(import.meta.env.VITE_APP_URL);

  let price: number = $state(50);
  let volume: number = $state(100);
  let orderBook: OrderBook = $state({
    bids: [],
    offers: [],
  });

  const spread = $derived.by(() => {
    if (orderBook.bids.length === 0 || orderBook.offers.length === 0) {
      return Number.NaN;
    }
    return (orderBook.offers[0].price - orderBook.bids[0].price) / 100;
  });

  socket.addEventListener("message", (msg) => {
    const parseRes = parseMessage(msg.data);
    if (!parseRes.success) {
      console.log(`Error while parsing message ${msg.data}: ${parseRes.error}`);
      return;
    }
    match(parseRes.data)
      .with({ type: "OrderBookSnapshot" }, (snapshot) => {
        orderBook.bids = snapshot.bids.toSorted((a, b) => b.price - a.price);
        orderBook.offers = snapshot.offers.toSorted(
          (a, b) => a.price - b.price
        );
      })
      .with({ type: "NewTrade" }, (new_trade) => {
        console.log(new_trade);
      })
      .exhaustive();
  });

  const sendBuyRequest = () => {
    const orderRequest = {
      price: price * 100,
      volume,
      direction: "Buy",
      owner: "toto",
    };
    const payload = JSON.stringify({ OrderRequest: orderRequest });
    console.log(`sending order request: ${payload}`);
    socket.send(payload);
  };

  const sendSellRequest = () => {
    const orderRequest = {
      price: price * 100,
      volume,
      direction: "Sell",
      owner: "toto",
    };
    const payload = JSON.stringify({ OrderRequest: orderRequest });
    console.log(`sending order request: ${payload}`);
    socket.send(payload);
  };
</script>

<main>
  <h2>Order book</h2>
  {#if !Number.isNaN(spread)}
    Spread: {spread} €
  {/if}
  <div class="grid columns-2">
    <div class="flex-col">
      <h3>Achats</h3>
      <ul class="OrderBook">
        {#each orderBook.bids as bid}
          <li>
            <OrderBookEntry price={bid.price} volume={bid.volume} />
          </li>
        {/each}
      </ul>
    </div>

    <div class="flex-col">
      <h3>Ventes</h3>
      <ul>
        {#each orderBook.offers as offer}
          <li>
            <OrderBookEntry price={offer.price} volume={offer.volume} />
          </li>
        {/each}
      </ul>
    </div>
  </div>

  <div>
    <div>
      <label>
        Price
        <input type="number" bind:value={price} />
      </label>

      <label>
        Volume
        <input type="number" bind:value={volume} />
      </label>
    </div>
    <div>
      <button onclick={sendBuyRequest}>BUY</button>
      <button onclick={sendSellRequest}>SELL</button>
    </div>
  </div>
</main>
