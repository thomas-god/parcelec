<script lang="ts">
  import type { Trade } from "$lib/message";

  let {
    trade,
    removeTradeToDisplay,
  }: { trade: Trade; removeTradeToDisplay: (trade: Trade) => void } = $props();
</script>

<div class="alert alert-info self-center">
  <span
    >Nouveau trade: {trade.volume}MW {trade.direction === "Buy"
      ? "achetés"
      : "vendus"}
    @ {0.01 * (trade.price as number)}€
    <button class="text-lg pl-2" onclick={() => removeTradeToDisplay(trade)}
      >✖️</button
    >
  </span>
</div>
