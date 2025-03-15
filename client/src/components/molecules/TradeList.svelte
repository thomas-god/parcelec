<script lang="ts">
  import type { Trade } from "$lib/message";

  let { trades }: { trades: Trade[] } = $props();

  const legend: Record<Trade["direction"], string> = {
    Buy: "Achat",
    Sell: "vente",
  };
</script>

<h2 class="text-xl font-semibold pb-3">Transactions</h2>
<ul class="list max-h-96 overflow-y-auto">
  {#each trades as trade (`${trade.direction}-${trade.execution_time}`)}
    <li class="list-row">
      <div>
        {legend[trade.direction].toUpperCase()}
      </div>
      <div class="list-col-grow">
        {trade.volume} MWh à {trade.price / 100} €/MWh
      </div>
    </li>
  {:else}
    <i> Pas de transactions pour cette période </i>
  {/each}
</ul>
<div class="modal-action">
  <form method="dialog">
    <button class="btn btn-success"> Fermer </button>
  </form>
</div>
