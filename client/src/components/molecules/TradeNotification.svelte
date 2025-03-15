<script lang="ts">
  import type { Trade } from "$lib/message";

  let {
    trade,
    removeTradeToDisplay,
  }: { trade: Trade; removeTradeToDisplay: (trade: Trade) => void } = $props();
</script>

<div class="alert alert-secondary self-center">
  <span
    >{trade.volume} MWh {trade.direction === "Buy" ? "achetés" : "vendus"}
    à {0.01 * (trade.price as number)} €/MWh !
    <button
      class="btn btn-xs"
      onclick={() => removeTradeToDisplay(trade)}
      aria-label="close notification"
    >
      <svg
        width="24px"
        height="24px"
        stroke-width="1.5"
        viewBox="0 0 24 24"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        ><path
          d="M6.75827 17.2426L12.0009 12M17.2435 6.75736L12.0009 12M12.0009 12L6.75827 6.75736M12.0009 12L17.2435 17.2426"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        ></path></svg
      >
    </button>
  </span>
</div>
