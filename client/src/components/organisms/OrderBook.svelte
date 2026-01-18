<script lang="ts">
  import type { OrderBook, OrderRequest, Trade } from "$lib/message";
  import AddOrder from "../molecules/AddOrder.svelte";
  import TradeList from "../molecules/TradeList.svelte";

  let {
    orderBook,
    send,
    trades,
  }: {
    orderBook: OrderBook;
    send: (msg: string) => void;
    trades: Trade[];
  } = $props();

  let sendOrderDebouncedInterval: ReturnType<typeof setTimeout>;
  const sendOrderRequest = (orderRequest: OrderRequest) => {
    addOrderModal.close();
    clearTimeout(sendOrderDebouncedInterval);
    sendOrderDebouncedInterval = setTimeout(() => {
      send(JSON.stringify({ OrderRequest: orderRequest }));
    }, 200);
  };

  const deleteOrder = (order_id: String) => {
    send(JSON.stringify({ DeleteOrder: { order_id } }));
  };
  let addOrderModal: HTMLDialogElement;
  let tradeListModal: HTMLDialogElement;
</script>

<div class="flex flex-col">
  <!-- Add an offer -->
  <div class="flex flex-row justify-center gap-2">
    <button class="btn btn-success" onclick={() => addOrderModal.showModal()}
      >Ajouter un ordre</button
    >
    <dialog class="modal" bind:this={addOrderModal}>
      <div class="modal-box bg-base-200 border border-base-300 p-4 rounded-box">
        <AddOrder {sendOrderRequest} />
      </div>
      <form method="dialog" class="modal-backdrop">
        <button>close</button>
      </form>
    </dialog>

    <button class="btn btn-success" onclick={() => tradeListModal.showModal()}
      >Transactions passées</button
    >
    <dialog class="modal" bind:this={tradeListModal}>
      <div class="modal-box bg-base-200 border border-base-300 p-4 rounded-box">
        <TradeList {trades} />
      </div>
      <form method="dialog" class="modal-backdrop">
        <button>close</button>
      </form>
    </dialog>
  </div>

  <div class="grid grid-cols-2 gap-4 mt-3">
    <div class="flex flex-col p-2 justify-self-end">
      <h3 class="text-xl font-semibold mb-2 text-end">Acheteurs</h3>
      <table class="table table-zebra table-xs min-[400px]:table-sm">
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
              <td class="text-right">{bid.price}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div
      class="flex flex-col overflow-y-auto p-2 max-w-64 max-h-64 justify-self-start"
    >
      <h3 class="text-xl font-semibold mb-2 text-start">Vendeurs</h3>
      <table class="table table-zebra table-xs min-[400px]:table-sm">
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
              <td class="text-left">{offer.price}</td>
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
