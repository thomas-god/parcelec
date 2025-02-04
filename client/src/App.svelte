<script lang="ts">
  import OrderBookEntry from "./lib/orderBookEntry.svelte";
  const socket = new WebSocket("wss://app.parcelec.org");
  let price: number = $state(50);
  let volume: number = $state(100);
  interface OrderEntry {
    direction: "Buy" | "Sell";
    volume: number;
    price: number;
    created_at: Date;
  }
  interface OrderBook {
    bids: OrderEntry[];
    offers: OrderEntry[];
  }
  let snapshot: OrderBook = $state({
    bids: [],
    offers: [],
  });
  const spread = $derived.by(() => {
    if (snapshot.bids.length === 0 || snapshot.offers.length === 0) {
      return Number.NaN;
    }
    return (snapshot.offers[0].price - snapshot.bids[0].price) / 100;
  });
  socket.addEventListener("message", (msg) => {
    let data = JSON.parse(msg.data)["OrderBookSnapshot"];
    console.log(data);
    snapshot = data;
    snapshot.bids.sort((a, b) => b.price - a.price);
    snapshot.offers.sort((a, b) => a.price - b.price);
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
  <div class="OrderBook">
    <div class="OrderBookColumn">
      <h3>Achats</h3>
      <ul class="OrderBook">
        {#each snapshot.bids as bid}
          <li>
            <OrderBookEntry price={bid.price} volume={bid.volume} />
          </li>
        {/each}
      </ul>
    </div>

    <div class="OrderBookColumn">
      <h3>Ventes</h3>
      <ul class="OrderBook">
        {#each snapshot.offers as offer}
          <li>
            <OrderBookEntry price={offer.price} volume={offer.volume} />
          </li>
        {/each}
      </ul>
    </div>
  </div>

  <div class="SendOrder">
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

<style>
  div.OrderBook {
    display: grid;
    grid-template-columns: 1fr 1fr;
  }

  .OrderBook > li {
    list-style: none;
  }
</style>
