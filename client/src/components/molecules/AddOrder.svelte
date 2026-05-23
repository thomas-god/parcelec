<script lang="ts">
  import type { OrderBookEntry, OrderRequest } from "$lib/message";
  import { isSome, type Some } from "$lib/Options";
  import type { BBO } from "$lib/trades";
  import CloseButton from "../atoms/CloseButton.svelte";

  let {
    sendOrderRequest,
    bbo,
  }: { sendOrderRequest: (request: OrderRequest) => void; bbo: BBO } = $props();

  let price = $state("50");
  let volume = $state("100");

  const send = (direction: OrderRequest["direction"]) => {
    const orderRequest = {
      price: Number.parseInt(price),
      volume: Number.parseInt(volume),
      direction,
    };
    sendOrderRequest(orderRequest);
  };
</script>

<fieldset class="fieldset">
  <div class="flex flex-row justify-between content-center">
    <legend class="fieldset-legend text-xl">Ajouter un ordre</legend>
    <div class="modal-action">
      <form method="dialog">
        <CloseButton onClick={() => {}} success={false} />
      </form>
    </div>
  </div>

  <label class="fieldset-label">
    Volume (MW)
    <input
      type="text"
      inputmode="numeric"
      pattern="[0-9]*"
      class="input max-w-48 text-base"
      bind:value={volume}
    />
  </label>
  <label class="fieldset-label">
    Prix (€/MWh)
    <input
      type="text"
      inputmode="numeric"
      pattern="[0-9]*"
      class="input max-w-48 text-base"
      bind:value={price}
    />
  </label>
  <div class="italic opacity-80 flex flex-col gap-1">
    <div class="flex flex-row items-center gap-2">
      <div class="shrink-0">Meilleur acheteur :</div>
      {#if isSome(bbo.bestBid)}
        <div class="flex flex-row flex-wrap gap-1">
          <span class="font-semibold">{bbo.bestBid.value.price} €/MWh</span>
          <span>
            ({bbo.bestBid.value.volume} MWh)
          </span>
        </div>
        <div>
          <button
            class="btn btn-xs btn-outline btn-warning"
            onclick={() =>
              (price = (
                bbo.bestBid as Some<OrderBookEntry>
              ).value.price.toString())}
            >s'aligner
          </button>
        </div>
      {:else}
        <div>-</div>
      {/if}
    </div>
    <div class="flex flex-row items-center gap-2">
      <div class="shrink-0">Meilleur vendeur :</div>
      {#if isSome(bbo.bestOffer)}
        <div class="flex flex-row flex-wrap gap-1">
          <span class="font-semibold">{bbo.bestOffer.value.price} €/MWh</span>
          <span>
            ({bbo.bestOffer.value.volume} MWh)
          </span>
        </div>
        <div>
          <button
            class="btn btn-xs btn-outline btn-success"
            onclick={() =>
              (price = (
                bbo.bestOffer as Some<OrderBookEntry>
              ).value.price.toString())}>s'aligner</button
          >
        </div>
      {:else}
        <div>-</div>
      {/if}
    </div>
  </div>
  <div class="flex flex-row justify-around mt-2">
    <button
      class="px-4 py-2 bg-success text-white rounded"
      onclick={() => {
        send("Buy");
      }}>Acheter</button
    >
    <button
      class="px-4 py-2 bg-warning text-white rounded"
      onclick={() => {
        send("Sell");
      }}>Vendre</button
    >
  </div>
</fieldset>
