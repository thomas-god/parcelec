<script lang="ts">
  import type { OrderRequest } from "$lib/message";
  import CloseButton from "../atoms/CloseButton.svelte";

  let {
    sendOrderRequest,
  }: { sendOrderRequest: (request: OrderRequest) => void } = $props();

  let price = $state("50");
  let volume = $state("100");

  const send = (direction: OrderRequest["direction"]) => {
    const orderRequest = {
      price: Number.parseInt(price) * 100,
      volume: Number.parseInt(volume),
      direction,
    };
    sendOrderRequest(orderRequest);
  };
</script>

<fieldset class="fieldset">
  <div class="flex flex-row justify-between content-center">
    <legend class="fieldset-legend text-xl">Ajouter un ordre</legend>
    <div class="modal-action !my-auto">
      <form method="dialog">
        <CloseButton onClick={() => {}} />
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
    Prix (€)
    <input
      type="text"
      inputmode="numeric"
      pattern="[0-9]*"
      class="input max-w-48 text-base"
      bind:value={price}
    />
  </label>
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
