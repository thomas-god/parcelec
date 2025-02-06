<script lang="ts">
  import type { OrderBook } from "./message";
  import NumberInput from "./NumberInput.svelte";
  import { some } from "./Options";
  import OrderBookEntry from "./OrderBookEntry.svelte";
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

  const sendOfferRequest = (direction: "Sell" | "Buy") => {
    const orderRequest = {
      price: price * 100,
      volume,
      direction,
      owner: "toto",
    };
    send(JSON.stringify({ OrderRequest: orderRequest }));
  };
</script>

<div class="flex flex-col">
  <h2 class="text-2xl font-bold mb-4 text-center">Carnet d'ordres</h2>
  <!-- {#if !Number.isNaN(spread)}
    <p class="mb-4 text-center text-lg">Spread: {spread} €</p>
    {/if} -->

  <div class="grid grid-cols-2 gap-4">
    <div
      class="flex flex-col h-64 overflow-y-auto p-2 max-w-64 justify-self-end"
    >
      <h3 class="text-xl font-semibold mb-2 text-end">Achats</h3>
      <ul class="space-y-2">
        {#each orderBook.bids as bid (bid.created_at)}
          <li
            class="flex justify-end p-2 shadow-md shadow-green-500 rounded animate-fade-in-scale"
          >
            <OrderBookEntry price={bid.price} volume={bid.volume} />
          </li>
        {/each}
      </ul>
    </div>

    <div
      class="flex flex-col h-64 overflow-y-auto p-2 max-w-64 justify-self-start"
    >
      <h3 class="text-xl font-semibold mb-2 text-start">Ventes</h3>
      <ul class="space-y-2">
        {#each orderBook.offers as offer (offer.created_at)}
          <li
            class="flex justify-between p-2 shadow-md shadow-red-500 rounded animate-fade-in-scale"
          >
            <OrderBookEntry price={offer.price} volume={offer.volume} />
          </li>
        {/each}
      </ul>
    </div>
  </div>

  <!-- Add an offer -->
  <div class="mt-8">
    <h3 class="text-xl font-semibold mb-2 text-center">Ajouter une offre</h3>
    <div class="grid grid-cols-2 gap-2 mb-6">
      <NumberInput bind:value={price} label={"Price"} bigIncr={some(1)} />
      <NumberInput bind:value={volume} label={"Volume"} bigIncr={some(10)} />
    </div>
    <div class="flex justify-center space-x-4">
      <button
        class="px-4 py-2 bg-green-500 text-white rounded"
        onclick={() => sendOfferRequest("Buy")}>BUY</button
      >
      <button
        class="px-4 py-2 bg-red-500 text-white rounded"
        onclick={() => sendOfferRequest("Sell")}>SELL</button
      >
    </div>
  </div>
</div>
