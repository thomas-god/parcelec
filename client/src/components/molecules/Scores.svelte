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

<div class="flex flex-row justify-around text-xl min-[400px]:text-2xl">
  <div class="text-left grow">
    {#if position > 0}
      ⚠️ Surplus : {Math.abs(position)} MW
    {:else if position < 0}
      ⚠️ Manque : {Math.abs(position)} MW
    {:else}
      ✅ A l'équilibre
    {/if}
  </div>
  <div class="@container-normal grow text-right">
    <span class="hidden @3xs:inline"> Score : </span>
    <span class="inline @3xs:hidden"> 💰</span>
    {pnl.toLocaleString("fr-FR")} €
  </div>
  <!-- Equilibre: {plants_position + trades_position} MW -->
</div>
