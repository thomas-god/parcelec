<script lang="ts">
  import type { StackSnapshot, Trade } from "$lib/message";
  import { marketPnl, plantsCosts } from "$lib/pnl";
  import {
    computePortfolio,
    marketPosition,
    plantsPosition,
  } from "$lib/position";
  import CurrentScore from "../molecules/CurrentScore.svelte";
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

<div class="flex flex-col items-stretch p-3">
  <div class="max-w-124 mx-auto w-full">
    <CurrentScore {costs} {position} />
  </div>
  <div class="max-w-132 mx-auto w-full">
    <PortfolioChart {volumes} height={100} iconSize={0.8} />
  </div>
</div>
