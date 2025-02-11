<script lang="ts">
  import type { OrderBook } from "$lib/message";
  import { some } from "$lib/Options";
  import NumberInput from "../atoms/NumberInput.svelte";
  import OrderBookEntry from "../atoms/OrderBookEntry.svelte";

  let {
    orderBook,
    send,
  }: { orderBook: OrderBook; send: (msg: string) => void } = $props();

  let price: number = $state(50);
  let volume: number = $state(100);

  const spread = $derived.by(() => {
    if (orderBook.bids.length === 0 || orderBook.offers.length === 0) {
      return Number.NaN;
    }
    return (orderBook.offers[0].price - orderBook.bids[0].price) / 100;
  });

  const sendOrderRequest = (direction: "Sell" | "Buy") => {
    const orderRequest = {
      price: price * 100,
      volume,
      direction,
      owner: "toto",
    };
    send(JSON.stringify({ OrderRequest: orderRequest }));
  };

  const deleteOrder = (order_id: String) => {
    send(JSON.stringify({ DeleteOrder: { order_id } }));
  };
</script>

<div class="flex flex-col">
  <h2 class="text-lg font-bold">Marché</h2>
  <!-- Add an offer -->
  <div class="">
    <!-- <h3 class="text-xl font-semibold mb-2 text-center">Ajouter une offre</h3> -->
    <div class="flex flex-row flex-wrap justify-center gap-2 mb-3">
      <NumberInput bind:value={price} label={"Prix"} bigIncr={some(1)} />
      <NumberInput bind:value={volume} label={"Volume"} bigIncr={some(10)} />
    </div>
    <div class="flex justify-center space-x-4">
      <button
        class="px-4 py-2 bg-green-500 text-white rounded"
        onclick={() => sendOrderRequest("Buy")}>BUY</button
      >
      <button
        class="px-4 py-2 bg-red-500 text-white rounded"
        onclick={() => sendOrderRequest("Sell")}>SELL</button
      >
    </div>
  </div>
  <!-- {#if !Number.isNaN(spread)}
    <p class="mb-4 text-center text-lg">Spread: {spread} €</p>
    {/if} -->

  <div
    class="grid grid-cols-2 gap-4 border-4 border-double rounded-2xl border-gray-400 mt-3"
  >
    <div
      class="flex flex-col h-64 overflow-y-auto p-2 max-w-64 justify-self-end"
    >
      <h3 class="text-xl font-semibold mb-2 text-end">Acheteurs</h3>
      <ul class="space-y-2">
        {#each orderBook.bids as bid (bid.created_at)}
          <li>
            <OrderBookEntry entry={bid} deleteEntry={deleteOrder} />
          </li>
        {/each}
      </ul>
    </div>

    <div
      class="flex flex-col h-64 overflow-y-auto p-2 max-w-64 justify-self-start"
    >
      <h3 class="text-xl font-semibold mb-2 text-start">Vendeurs</h3>
      <ul class="space-y-2">
        {#each orderBook.offers as offer (offer.created_at)}
          <li>
            <OrderBookEntry entry={offer} deleteEntry={deleteOrder} />
          </li>
        {/each}
      </ul>
    </div>
  </div>
</div>
