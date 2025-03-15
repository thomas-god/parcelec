<script lang="ts">
  import type { Trade } from "$lib/message";
  import CloseButton from "../atoms/CloseButton.svelte";

  let {
    trade,
    removeTradeToDisplay,
  }: { trade: Trade; removeTradeToDisplay: (trade: Trade) => void } = $props();
</script>

<div class="alert alert-secondary self-center">
  <span
    >{trade.volume} MWh {trade.direction === "Buy" ? "achetés" : "vendus"}
    à {0.01 * (trade.price as number)} €/MWh !

    <CloseButton onClick={() => removeTradeToDisplay(trade)} />
  </span>
</div>
