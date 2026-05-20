<script lang="ts">
  import type { StackSnapshot, Trade } from "$lib/message";
  import { marketPnl, plantsCosts } from "$lib/pnl";
  import {
    computePortfolio,
    marketPosition,
    plantsPosition,
  } from "$lib/position";
  import PortfolioChart from "../molecules/PortfolioChart.svelte";

  interface Props {
    plants: StackSnapshot;
    trades: Trade[];
  }

  let { plants, trades }: Props = $props();

  let volumes = $derived(computePortfolio(plants, trades));

  let plants_position = $derived(plantsPosition(plants));
  let trades_position = $derived(marketPosition(trades));
  let position = $derived(plants_position + trades_position);

  let costs = $derived(plantsCosts(plants) + marketPnl(trades));
</script>

<div class="portfolio-container">
  <div class="w-full">
    <PortfolioChart {volumes} height={100} iconSize={0.8} />
  </div>

  <div class="score-container">
    <div class="score-item">
      <img src="/icons/balance.svg" alt="Balance icon" class="w-7 h-7 inline" />
      <span>
        {#if position === 0}
          A l'équilibre ✅
        {:else}
          {position.toLocaleString("fr-FR", { signDisplay: "always" })} MW
        {/if}
      </span>
    </div>
    <div class="score-item">
      <img src="/icons/coin.svg" alt="Money coin icon" class="w-6 h-6 inline" />
      <span>
        {costs.toLocaleString("fr-FR")} €
      </span>
    </div>
  </div>
</div>

<style>
  .portfolio-container {
    display: flex;
    flex-direction: column;
    gap: calc(var(--spacing) * 0);
    padding-top: calc(var(--spacing) * 2);

    @media (width > 600px) {
      gap: calc(var(--spacing) * 3);
      flex-direction: row;
    }
  }

  .score-container {
    display: flex;
    flex-direction: row;
    gap: calc(var(--spacing) * 2);
    justify-content: space-between;

    @media (width > 600px) {
      flex-direction: column;
      align-items: end;
      justify-content: space-around;
      flex-shrink: 0;
    }
  }

  .score-item {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: calc(var(--spacing) * 1);

    @media (width > 600px) {
      flex-direction: row-reverse;
    }
  }
</style>
