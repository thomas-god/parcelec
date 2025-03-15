<script lang="ts">
  import type { OrderBook, Trade } from "$lib/message";

  let {
    orderBook,
    send,
    trades,
  }: {
    orderBook: OrderBook;
    send: (msg: string) => void;
    trades: Trade[];
  } = $props();

  let price = $state("50");
  let volume = $state("100");

  let market_position = $derived(
    trades.reduce(
      (acc, trade) =>
        acc + (trade.direction === "Buy" ? trade.volume : -trade.volume),
      0,
    ),
  );
  let sendOrderDebouncedInterval: ReturnType<typeof setTimeout>;
  const sendOrderRequest = (direction: "Sell" | "Buy") => {
    clearTimeout(sendOrderDebouncedInterval);
    sendOrderDebouncedInterval = setTimeout(() => {
      const orderRequest = {
        price: Number.parseInt(price) * 100,
        volume: Number.parseInt(volume),
        direction,
      };
      send(JSON.stringify({ OrderRequest: orderRequest }));
    }, 200);
  };

  const deleteOrder = (order_id: String) => {
    send(JSON.stringify({ DeleteOrder: { order_id } }));
  };
</script>

<div class="flex flex-col">
  <!-- <div class="flex flex-row justify-between">
    <div class="pb-2">
      {#if market_position > 0}
        Acheté : {market_position} MWh
      {:else if market_position < 0}
        Vendu : {Math.abs(market_position)} MWh
      {/if}
    </div>
  </div> -->
  <!-- Add an offer -->
  <div class="flex flex-row justify-center">
    <button
      class="btn"
      onclick={() => document.getElementById("add_order").showModal()}
      >Ajouter un ordre</button
    >
    <dialog id="add_order" class="modal">
      <div class="modal-box bg-base-200 border border-base-300 p-4 rounded-box">
        <fieldset class="fieldset">
          <legend class="fieldset-legend text-xl pb-3">Ajouter un ordre</legend>

          <label class="fieldset-label">
            Volume (MW)
            <input
              type="text"
              inputmode="numeric"
              pattern="[0-9]*"
              class="input max-w-48"
              bind:value={volume}
            />
          </label>
          <label class="fieldset-label">
            Prix (€)
            <input
              type="text"
              inputmode="numeric"
              pattern="[0-9]*"
              class="input max-w-48"
              bind:value={price}
            />
          </label>
          <div class="flex flex-row justify-around mt-2">
            <button
              class="px-4 py-2 bg-success text-white rounded"
              onclick={() => {
                document.getElementById("add_order").close();
                sendOrderRequest("Buy");
              }}>Acheter</button
            >
            <button
              class="px-4 py-2 bg-warning text-white rounded"
              onclick={() => {
                document.getElementById("add_order").close();
                sendOrderRequest("Sell");
              }}>Vendre</button
            >
          </div>
        </fieldset>
      </div>
    </dialog>
  </div>

  <div class="grid grid-cols-2 gap-4 mt-3">
    <div
      class="flex flex-col h-64 overflow-y-auto p-2 max-w-64 justify-self-end"
    >
      <h3 class="text-xl font-semibold mb-2 text-end">Acheteurs</h3>
      <table class="table table-zebra table-sm sm:table-md">
        <thead>
          <tr>
            <th>🗑️</th>
            <th>Volume</th>
            <th>Prix</th>
          </tr>
        </thead>
        <tbody>
          {#each orderBook.bids as bid (bid.created_at)}
            <tr>
              <td>
                {#if bid.owned}
                  <button onclick={() => deleteOrder(bid.order_id)}>🗑️</button>
                {/if}
              </td>
              <td class="text-center">{bid.volume}</td>
              <td class="text-right">{bid.price / 100}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div
      class="flex flex-col h-64 overflow-y-auto p-2 max-w-64 justify-self-start"
    >
      <h3 class="text-xl font-semibold mb-2 text-start">Vendeurs</h3>
      <table class="table table-zebra table-sm sm:table-md">
        <thead>
          <tr>
            <th>Prix</th>
            <th>Volume</th>
            <th>🗑️</th>
          </tr>
        </thead>
        <tbody>
          {#each orderBook.offers as offer (offer.created_at)}
            <tr>
              <td class="text-left">{offer.price / 100}</td>
              <td class="text-center">{offer.volume}</td>
              <td>
                {#if offer.owned}
                  <button onclick={() => deleteOrder(offer.order_id)}>🗑️</button
                  >
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>
