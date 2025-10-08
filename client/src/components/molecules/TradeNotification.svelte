<script lang="ts">
  import type { Trade } from "$lib/message";
  import CloseButton from "../atoms/CloseButton.svelte";

  let {
    trade,
    removeTradeToDisplay,
  }: { trade: Trade; removeTradeToDisplay: (trade: Trade) => void } = $props();
</script>

<div class="alert alert-success self-center">
  <span
    >{trade.volume} MWh {trade.direction === "Buy" ? "achetés" : "vendus"}
    à {trade.price} €/MWh !

    <CloseButton onClick={() => removeTradeToDisplay(trade)} success={true} />
  </span>
</div>
