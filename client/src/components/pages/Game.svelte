<script lang="ts">
import OrderBookElement from "../molecules/OrderBook.svelte";
import { match } from "ts-pattern";
import { parseMessage, type OrderBook, type Trade } from "../../lib/message";

const socket = new WebSocket(import.meta.env.VITE_APP_URL);

let orderBook: OrderBook = $state({
	bids: [],
	offers: [],
});
let trades: Trade[] = $state([]);

socket.addEventListener("message", (msg) => {
	const parseRes = parseMessage(msg.data);
	if (!parseRes.success) {
		console.log(`Error while parsing message ${msg.data}: ${parseRes.error}`);
		return;
	}
	match(parseRes.data)
		.with({ type: "OrderBookSnapshot" }, (snapshot) => {
			orderBook.bids = snapshot.bids.toSorted((a, b) => b.price - a.price);
			orderBook.offers = snapshot.offers.toSorted((a, b) => a.price - b.price);
		})
		.with({ type: "NewTrade" }, (new_trade) => {
			trades.push(new_trade);
		})
		.exhaustive();
});

const sendMessage = (msg: string) => {
	socket.send(msg);
};
</script>

<main class="p-2">
  <OrderBookElement {orderBook} send={sendMessage} />

  <!-- <div class="mt-8">
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
  </div> -->
</main>
