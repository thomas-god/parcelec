<script lang="ts">
  import { match } from "ts-pattern";
  import { parseMessage, type OrderBook, type Trade } from "./message";
  import OrderBookEntry from "./orderBookEntry.svelte";

  const socket = new WebSocket(import.meta.env.VITE_APP_URL);

  let price: number = $state(50);
  let volume: number = $state(100);
  let orderBook: OrderBook = $state({
    bids: [],
    offers: [],
  });
  let trades: Trade[] = $state([]);

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
        trades.push(new_trade);
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
    // console.log(`sending order request: ${payload}`);
    socket.send(payload);
  };
</script>

<main class="p-6">
  <h2 class="text-2xl font-bold mb-4 text-center">Order book</h2>
  {#if !Number.isNaN(spread)}
    <p class="mb-4 text-center text-lg">Spread: {spread} €</p>
  {/if}
  <div class="grid grid-cols-2 gap-6">
    <div class="flex flex-col h-64 overflow-y-auto p-4">
      <h3 class="text-xl font-semibold mb-2 text-center">Achats</h3>
      <ul class="space-y-2">
        {#each orderBook.bids as bid}
          <li
            class="flex justify-between p-2 rounded border-dashed border-2 border-purple-500"
          >
            <OrderBookEntry price={bid.price} volume={bid.volume} />
          </li>
        {/each}
      </ul>
    </div>

    <div class="flex flex-col h-64 overflow-y-auto p-4">
      <h3 class="text-xl font-semibold mb-2 text-center">Ventes</h3>
      <ul class="space-y-2">
        {#each orderBook.offers as offer}
          <li
            class="flex justify-between p-2 rounded border-dashed border-2 border-purple-500"
          >
            <OrderBookEntry price={offer.price} volume={offer.volume} />
          </li>
        {/each}
      </ul>
    </div>
  </div>

  <div class="mt-8">
    <h3 class="text-xl font-semibold mb-2">Ajouter une offre</h3>
    <div class="grid grid-cols-2 gap-6 mb-6">
      <label class="block">
        <span class="text-gray-700">Price</span>
        <input
          type="number"
          bind:value={price}
          class="mt-1 block w-full rounded border-gray-300 shadow-sm p-2"
        />
      </label>

      <label class="block">
        <span class="text-gray-700">Volume</span>
        <input
          type="number"
          bind:value={volume}
          class="mt-1 block w-full rounded border-gray-300 shadow-sm p-2"
        />
      </label>
    </div>
    <div class="flex justify-center space-x-4">
      <button
        class="px-4 py-2 bg-green-500 text-white rounded"
        onclick={sendBuyRequest}>BUY</button
      >
      <button
        class="px-4 py-2 bg-red-500 text-white rounded"
        onclick={sendSellRequest}>SELL</button
      >
    </div>
  </div>

  <div class="mt-8">
    <h3 class="text-xl font-semibold mb-2 text-center">Trades</h3>
    <div class="grid grid-cols-2 gap-6 h-64 overflow-y-auto p-4">
      <ul class="space-y-2">
        {#each trades as trade, i}
          {#if i % 2 === 0}
            <li
              class="flex justify-between p-2 rounded border-dashed border-2 {trade.direction ===
              'Buy'
                ? 'border-green-500'
                : 'border-red-500'}"
            >
              <span>Price: {trade.price / 100} €</span>
              <span>Volume: {trade.volume}</span>
              <span>Direction: {trade.direction}</span>
            </li>
          {/if}
        {/each}
      </ul>
      <ul class="space-y-2">
        {#each trades as trade, i}
          {#if i % 2 !== 0}
            <li
              class="flex justify-between p-2 rounded border-dashed border-2 {trade.direction ===
              'Buy'
                ? 'border-green-500'
                : 'border-red-500'}"
            >
              <span>Price: {trade.price / 100} €</span>
              <span>Volume: {trade.volume}</span>
              <span>Direction: {trade.direction}</span>
            </li>
          {/if}
        {/each}
      </ul>
    </div>
  </div>
</main>
