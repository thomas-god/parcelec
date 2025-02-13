<script lang="ts">
  import type { StackSnapshot, Trade } from "$lib/message";
  import { marketPnl, plantsPnl } from "$lib/pnl";
  import { marketPosition, plantsPosition } from "$lib/position";

  let { plants, trades }: { plants: StackSnapshot; trades: Trade[] } = $props();

  let plants_position = $derived(plantsPosition(plants));
  let trades_position = $derived(marketPosition(trades));
  let position = $derived(plants_position + trades_position);

  let plants_pnl = $derived(plantsPnl(plants));
  let market_pnl = $derived(marketPnl(trades));
  let pnl = $derived(plants_pnl + market_pnl);
</script>

<div class="flex flex-row justify-around">
  <div class="text-2xl text-center">
    {#if position > 0}
      ⚠️ Surplus d'énergie: {Math.abs(position)} MW
    {:else if position < 0}
      ⚠️ Manque d'énergie: {Math.abs(position)} MW
    {:else}
      ✅ A l'équilibre
    {/if}
  </div>
  <div class="text-2xl text-center">
    Score: {pnl.toLocaleString("fr-FR")} €
  </div>
  <!-- Equilibre: {plants_position + trades_position} MW -->
</div>
