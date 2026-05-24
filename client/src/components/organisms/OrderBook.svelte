<script lang="ts">
  import type { OrderBook, OrderRequest, Trade } from "$lib/message";
  import { extract_bbo } from "$lib/trades";
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

  let bbo = $derived(extract_bbo(orderBook));

  // We explicitely want the initial state
  // svelte-ignore state_referenced_locally
  const initialOrderIds = new Set([
    ...orderBook.bids.map((b) => b.order_id),
    ...orderBook.offers.map((o) => o.order_id),
  ]);

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

<div class="@container flex flex-col">
  <!-- Add an offer -->
  <div class="flex flex-row justify-center gap-2">
    <button
      class="btn btn-secondary btn-sm"
      onclick={() => addOrderModal.showModal()}
      >Ajouter un ordre
      <img
        src="/icons/plus.svg"
        alt="plus icon"
        class="inline h-5 w-5"
      /></button
    >
    <dialog class="modal" bind:this={addOrderModal}>
      <div class="modal-box bg-base-200 border border-base-300 p-4 rounded-box">
        <AddOrder {sendOrderRequest} {bbo} />
      </div>
      <form method="dialog" class="modal-backdrop">
        <button>close</button>
      </form>
    </dialog>

    <button
      class="btn btn-outline btn-sm"
      onclick={() => tradeListModal.showModal()}
      >Historique
      <img src="/icons/pile.svg" alt="folder icon" class="inline h-5 w-5" />
    </button>
    <dialog class="modal" bind:this={tradeListModal}>
      <div class="modal-box bg-base-200 border border-base-300 p-4 rounded-box">
        <TradeList {trades} />
      </div>
      <form method="dialog" class="modal-backdrop">
        <button>close</button>
      </form>
    </dialog>
  </div>

  <div class="grid grid-cols-2 gap-0 mt-3">
    <div class="flex flex-col p-1 justify-self-end">
      <h3 class="text-lg font-semibold mb-2 text-end">Acheteurs</h3>
      <table class="table table-zebra table-xs">
        <thead>
          <tr>
            <th></th>
            <th>Volume</th>
            <th>Prix</th>
          </tr>
        </thead>
        <tbody>
          {#each orderBook.bids as bid (bid.created_at)}
            <tr class={initialOrderIds.has(bid.order_id) ? "" : "order-entry"}>
              <td class="p-0">
                {#if bid.owned}
                  <button
                    onclick={() => deleteOrder(bid.order_id)}
                    class="btn btn-ghost h-5 px-1 ml-1"
                  >
                    <img
                      src="/icons/delete.svg"
                      alt="delete bin icon"
                      class="inline h-4.5 w-4.5"
                    />
                  </button>
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
      class="flex flex-col overflow-y-auto p-1 max-w-64 max-h-64 justify-self-start"
    >
      <h3 class="text-lg font-semibold mb-2 text-start">Vendeurs</h3>
      <table class="table table-zebra table-xs">
        <thead>
          <tr>
            <th>Prix</th>
            <th>Volume</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each orderBook.offers as offer (offer.created_at)}
            <tr
              class={initialOrderIds.has(offer.order_id) ? "" : "order-entry"}
            >
              <td class="text-left">{offer.price}</td>
              <td class="text-center">{offer.volume}</td>
              <td class="p-0">
                {#if offer.owned}
                  <button
                    onclick={() => deleteOrder(offer.order_id)}
                    class="btn btn-ghost h-5 px-1 mr-1"
                  >
                    <img
                      src="/icons/delete.svg"
                      alt="delete bin icon"
                      class="inline h-4.5 w-4.5"
                    />
                  </button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>

<style>
  @keyframes order-entry-animation {
    0% {
      background-color: inherit;
      color: inherit;
      opacity: inherit;
    }

    25%,
    75% {
      background-color: var(--color-info);
      color: var(--color-info-content);
      opacity: 40%;
    }

    100% {
      background-color: inherit;
      color: inherit;
      opacity: inherit;
    }
  }

  .order-entry {
    animation: order-entry-animation 2s linear;
  }
</style>
